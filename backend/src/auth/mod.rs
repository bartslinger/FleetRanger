//! Login and the [`Actor`]: who is making a request.

mod db;
mod oidc;
pub mod routes;

use std::net::{IpAddr, SocketAddr};
use std::sync::OnceLock;

use axum::{
    extract::{ConnectInfo, FromRef, FromRequestParts, Request, State},
    http::{HeaderMap, Method, StatusCode, header, request::Parts},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::CookieJar;
use openidconnect::url::Url;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

pub use oidc::Provider;

use crate::policy::{Grants, Permission};

/// Whether the install is reached over HTTPS, set once at startup.
static HTTPS: OnceLock<bool> = OnceLock::new();

/// Over HTTPS the cookies' names start with `__Host-`: browsers then accept
/// them only from this very host, `Secure` and for every path, so a sibling
/// subdomain can't plant one (cookie tossing). Plain HTTP, in development,
/// can't have that prefix.
pub fn set_https(https: bool) {
    let _ = HTTPS.set(https);
}

fn cookie_name(name: &'static str, prefixed: &'static str) -> &'static str {
    if HTTPS.get().copied().unwrap_or(true) {
        prefixed
    } else {
        name
    }
}

/// Holds the login session token.
fn session_cookie() -> &'static str {
    cookie_name("fleetranger_session", "__Host-fleetranger_session")
}

/// Holds a login in progress, only on its way back from the provider.
fn login_cookie() -> &'static str {
    cookie_name("fleetranger_login", "__Host-fleetranger_login")
}

/// Holds the user an admin views as ("View as" in docs/architecture.md).
fn view_as_cookie() -> &'static str {
    cookie_name("fleetranger_view_as", "__Host-fleetranger_view_as")
}

/// Login sessions expire after this many days without use.
const SESSION_IDLE_DAYS: i32 = 30;

/// Login settings shared by all requests.
pub struct Auth {
    /// Where browsers reach the install.
    pub public_url: Url,
    /// `None` when no login provider is configured.
    pub provider: Option<Provider>,
}

impl Auth {
    /// Cookies are only marked `Secure` over HTTPS, so login also works on
    /// `http://localhost` in development.
    fn secure_cookies(&self) -> bool {
        self.public_url.scheme() == "https"
    }
}

/// Who is making a request: a logged-in user. Handlers get one only through a
/// valid login session, and pass it to the service functions they call. When
/// an admin views as another user, it is that user, with their roles.
#[derive(Clone)]
pub struct Actor {
    pub user_id: Uuid,
    /// Their roles, read once per request; every access decision asks these.
    pub grants: Grants,
    #[expect(dead_code, reason = "read by the audit log, which comes next")]
    pub login_session_id: Uuid,
    #[expect(dead_code, reason = "read by the audit log, which comes next")]
    pub ip: IpAddr,
    /// None for a user without a login, whom an admin views as.
    pub email: Option<String>,
    pub name: String,
    /// The admin viewing as this user, who is the one logged in.
    pub viewed_by: Option<Viewer>,
}

/// An admin viewing as another user.
#[derive(Clone)]
pub struct Viewer {
    pub user_id: Uuid,
    pub name: String,
}

impl<S> FromRequestParts<S> for Actor
where
    S: Send + Sync,
    PgPool: FromRef<S>,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);
        let token = match jar.get(session_cookie()) {
            Some(cookie) => cookie.value(),
            None => bearer(&parts.headers).ok_or(StatusCode::UNAUTHORIZED)?,
        };
        let ConnectInfo(peer) = parts
            .extensions
            .get::<ConnectInfo<SocketAddr>>()
            .copied()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
        let ip = client_ip(&parts.headers, peer);
        let db = PgPool::from_ref(state);
        let session = db::use_session(&db, &token_hash(token), SESSION_IDLE_DAYS)
            .await
            .map_err(|error| {
                tracing::error!(%error, "looking up a login session");
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .ok_or(StatusCode::UNAUTHORIZED)?;
        let grants = grants(&db, session.user_id).await.map_err(|error| {
            tracing::error!(%error, "reading a user's roles");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        let actor = Actor {
            user_id: session.user_id,
            grants,
            login_session_id: session.id,
            ip,
            email: Some(session.email),
            name: session.name,
            viewed_by: None,
        };
        // Only a browser sends the cookie, which counts only while the one
        // logged in may manage users; otherwise they simply see their own view.
        let viewed = jar
            .get(view_as_cookie())
            .and_then(|cookie| Uuid::parse_str(cookie.value()).ok());
        match viewed {
            Some(user_id) if may_view_as(&actor, user_id) => {
                view_as(&db, actor, user_id).await.map_err(|error| {
                    tracing::error!(%error, "viewing as another user");
                    StatusCode::INTERNAL_SERVER_ERROR
                })
            }
            _ => Ok(actor),
        }
    }
}

/// Whether this admin may view as that user: anyone but themselves.
fn may_view_as(admin: &Actor, user_id: Uuid) -> bool {
    admin.grants.may(Permission::UsersManage, None) && user_id != admin.user_id
}

/// The admin as the user they view as, or still themselves if that user is
/// archived or gone.
async fn view_as(db: &PgPool, admin: Actor, user_id: Uuid) -> sqlx::Result<Actor> {
    let Some(user) = db::viewed_user(db, user_id).await? else {
        return Ok(admin);
    };
    Ok(Actor {
        user_id,
        grants: grants(db, user_id).await?,
        email: user.email,
        name: user.name,
        viewed_by: Some(Viewer {
            user_id: admin.user_id,
            name: admin.name,
        }),
        ..admin
    })
}

/// A login session's token sent as `Authorization: Bearer …`, which only debug
/// builds accept: for trying the MCP server with a local assistant until it
/// has OAuth (see "Clients" in docs/architecture.md).
#[cfg(debug_assertions)]
fn bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
}

#[cfg(not(debug_assertions))]
fn bearer(_headers: &HeaderMap) -> Option<&str> {
    None
}

/// The database stores only this hash of a session token, so a leaked database
/// holds no usable cookies.
fn token_hash(token: &str) -> Vec<u8> {
    Sha256::digest(token.as_bytes()).to_vec()
}

/// The browser's address. Behind our own proxies (nginx, SvelteKit, the Vite dev
/// server), which connect from the internal network, it is the last address in
/// `X-Forwarded-For`, which they append. Otherwise it is the connecting peer.
/// A user's roles on groups and the whole organisation.
pub async fn grants(db: &PgPool, user_id: Uuid) -> sqlx::Result<Grants> {
    Grants::from_stored(db::roles(db, user_id).await?).ok_or_else(|| {
        sqlx::Error::Decode(format!("unknown role stored for user {user_id}").into())
    })
}

pub(crate) fn client_ip(headers: &HeaderMap, peer: SocketAddr) -> IpAddr {
    let peer = peer.ip();
    if !is_internal(peer) {
        return peer;
    }
    forwarded_for(headers).unwrap_or(peer)
}

fn forwarded_for(headers: &HeaderMap) -> Option<IpAddr> {
    let value = headers.get("x-forwarded-for")?.to_str().ok()?;
    value.rsplit(',').next()?.trim().parse().ok()
}

fn is_internal(ip: IpAddr) -> bool {
    match ip.to_canonical() {
        IpAddr::V4(ip) => ip.is_loopback() || ip.is_private(),
        IpAddr::V6(ip) => ip.is_loopback() || ip.is_unique_local(),
    }
}

/// Rejects changes sent by a browser from another site. Browsers send `Origin`
/// with every cross-site request that can change something; requests from our
/// own server side (SvelteKit) send none.
pub async fn same_origin(
    State(auth): State<std::sync::Arc<Auth>>,
    request: Request,
    next: Next,
) -> Response {
    let safe = matches!(
        *request.method(),
        Method::GET | Method::HEAD | Method::OPTIONS
    );
    let origin = request.headers().get(header::ORIGIN);
    let expected = auth.public_url.origin().ascii_serialization();
    if !safe && origin.is_some_and(|origin| origin.as_bytes() != expected.as_bytes()) {
        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body("cross-site request refused".into())
            .unwrap_or_default();
    }
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::{may_view_as, view_as};
    use crate::testing;

    #[sqlx::test]
    async fn only_admins_view_as_someone_else_with_that_users_roles(db: PgPool) {
        let admin = testing::user(&db, "Test Admin", &["admin"]).await;
        let pilot = testing::user(&db, "Test Pilot", &["pilot"]).await;
        let as_admin = testing::actor(&db, admin).await;
        let as_pilot = testing::actor(&db, pilot).await;

        assert!(may_view_as(&as_admin, pilot));
        assert!(!may_view_as(&as_admin, admin));
        assert!(!may_view_as(&as_pilot, admin));

        let viewing = view_as(&db, as_admin.clone(), pilot).await.unwrap();
        assert_eq!(viewing.user_id, pilot);
        assert!(!viewing.grants.may(super::Permission::UsersManage, None));
        assert_eq!(viewing.viewed_by.map(|viewer| viewer.user_id), Some(admin));

        // An archived user can't be viewed as: the admin sees their own view.
        sqlx::query("update users set archived_at = now() where id = $1")
            .bind(pilot)
            .execute(&db)
            .await
            .unwrap();
        let viewing = view_as(&db, as_admin, pilot).await.unwrap();
        assert_eq!(viewing.user_id, admin);
        assert!(viewing.viewed_by.is_none());
    }
}
