//! Login and the [`Actor`]: who is making a request.

mod db;
mod oidc;
pub mod routes;

use std::net::{IpAddr, SocketAddr};

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

use crate::policy::Role;

/// Holds the login session token.
const SESSION_COOKIE: &str = "fleetranger_session";

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
/// valid login session, and pass it to the service functions they call.
pub struct Actor {
    pub user_id: Uuid,
    pub role: Role,
    #[expect(dead_code, reason = "read by the audit log, which comes next")]
    pub login_session_id: Uuid,
    #[expect(dead_code, reason = "read by the audit log, which comes next")]
    pub ip: IpAddr,
    pub email: String,
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
        let token = jar
            .get(SESSION_COOKIE)
            .ok_or(StatusCode::UNAUTHORIZED)?
            .value();
        let ConnectInfo(peer) = parts
            .extensions
            .get::<ConnectInfo<SocketAddr>>()
            .copied()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
        let ip = client_ip(&parts.headers, peer);
        let session = db::use_session(
            &PgPool::from_ref(state),
            &token_hash(token),
            SESSION_IDLE_DAYS,
        )
        .await
        .map_err(|error| {
            tracing::error!(%error, "looking up a login session");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;
        let role = Role::parse(&session.role).ok_or_else(|| {
            tracing::error!(role = session.role, "unknown user role");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        Ok(Actor {
            user_id: session.user_id,
            role,
            login_session_id: session.id,
            ip,
            email: session.email,
            name: session.name,
        })
    }
}

/// The database stores only this hash of a session token, so a leaked database
/// holds no usable cookies.
fn token_hash(token: &str) -> Vec<u8> {
    Sha256::digest(token.as_bytes()).to_vec()
}

/// The browser's address. Behind our own proxies (nginx, SvelteKit, the Vite dev
/// server), which connect from the internal network, it is the last address in
/// `X-Forwarded-For`, which they append. Otherwise it is the connecting peer.
fn client_ip(headers: &HeaderMap, peer: SocketAddr) -> IpAddr {
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
