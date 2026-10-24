//! Logging in and out. The web app sends the browser to `/auth/login`; the
//! provider sends it back to `/auth/callback`.

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    Form, Json, Router,
    extract::{ConnectInfo, FromRef, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use cookie::time::Duration;
use openidconnect::CsrfToken;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use super::db::{self, LoginUser};
use super::oidc::PendingLogin;
use super::{
    Actor, Auth, SESSION_IDLE_DAYS, client_ip, impersonation_cookie, login_cookie, session_cookie,
    token_hash,
};
use crate::oauth::OAuth;
use crate::policy::{Grants, Permission};

pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    PgPool: FromRef<S>,
    Arc<Auth>: FromRef<S>,
    OAuth: FromRef<S>,
{
    Router::new()
        .route("/auth/login", get(login))
        .route("/auth/callback", get(callback))
        .route("/auth/logout", post(logout))
        .route("/auth/impersonate", post(impersonate))
        .route("/auth/impersonate/stop", post(stop_impersonating))
        .route("/api/me", get(me))
}

#[derive(Deserialize)]
struct LoginQuery {
    return_to: Option<String>,
}

async fn login(
    State(auth): State<Arc<Auth>>,
    jar: CookieJar,
    Query(query): Query<LoginQuery>,
) -> Response {
    if auth.provider.is_none() {
        return (StatusCode::SERVICE_UNAVAILABLE, "login is not configured").into_response();
    }
    let return_to = local_path(query.return_to.as_deref());
    let begun = {
        let auth = auth.clone();
        tokio::task::spawn_blocking(move || {
            let provider = auth.provider.as_ref().expect("checked above");
            provider.begin(return_to)
        })
        .await
    };
    match begun {
        Ok(Ok((url, pending))) => {
            // Path "/" as `__Host-` requires; it lives ten minutes.
            let cookie = Cookie::build((login_cookie(), pending.encode()))
                .path("/")
                .http_only(true)
                .secure(auth.secure_cookies())
                // Lax: sent along when the provider sends the browser back.
                .same_site(SameSite::Lax)
                .max_age(Duration::minutes(10));
            (jar.add(cookie), Redirect::to(url.as_str())).into_response()
        }
        Ok(Err(error)) => {
            tracing::error!(error = format!("{error:#}"), "starting a login");
            Redirect::to("/login?error=failed").into_response()
        }
        Err(error) => {
            tracing::error!(%error, "starting a login");
            Redirect::to("/login?error=failed").into_response()
        }
    }
}

#[derive(Deserialize)]
struct CallbackQuery {
    state: Option<String>,
    code: Option<String>,
    /// Set by the provider instead of a code, e.g. when the user cancels.
    error: Option<String>,
}

async fn callback(
    State(auth): State<Arc<Auth>>,
    State(db): State<PgPool>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    jar: CookieJar,
    headers: HeaderMap,
    Query(query): Query<CallbackQuery>,
) -> Response {
    let pending = jar
        .get(login_cookie())
        .and_then(|cookie| PendingLogin::decode(cookie.value()));
    let jar = jar.remove(Cookie::build(login_cookie()).path("/"));

    let (Some(pending), Some(state), Some(code), None) =
        (pending, query.state, query.code, query.error.as_deref())
    else {
        if query.error.as_deref() == Some("access_denied") {
            return (jar, Redirect::to("/login")).into_response();
        }
        tracing::warn!(
            error = query.error,
            "login callback without a login in progress"
        );
        return (jar, Redirect::to("/login?error=failed")).into_response();
    };
    let Some(_) = &auth.provider else {
        return (StatusCode::SERVICE_UNAVAILABLE, "login is not configured").into_response();
    };

    // Checked again: it comes back from a cookie, which another site on the
    // same domain could have planted, and must never lead off this site.
    let return_to = local_path(Some(&pending.return_to));
    let finished = {
        let auth = auth.clone();
        tokio::task::spawn_blocking(move || {
            let provider = auth.provider.as_ref().expect("checked above");
            provider.finish(pending, &state, code)
        })
        .await
    };
    let identity = match finished {
        Ok(Ok(identity)) => identity,
        Ok(Err(error)) => {
            tracing::warn!(error = format!("{error:#}"), "login failed");
            return (jar, Redirect::to("/login?error=failed")).into_response();
        }
        Err(error) => {
            tracing::error!(%error, "finishing a login");
            return (jar, Redirect::to("/login?error=failed")).into_response();
        }
    };

    let user = db::login_user(&db, &identity.subject, identity.verified_email.as_deref()).await;
    let user_id = match user {
        Ok(LoginUser::Allowed(id)) => id,
        Ok(LoginUser::Refused) => {
            tracing::info!(email = identity.verified_email, "login refused: no login");
            return (jar, Redirect::to("/login?error=no-access")).into_response();
        }
        Err(error) => {
            tracing::error!(%error, "looking up the user for a login");
            return (jar, Redirect::to("/login?error=failed")).into_response();
        }
    };

    let token = CsrfToken::new_random_len(32).into_secret();
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|value| value.to_str().ok());
    let created = db::create_session(
        &db,
        user_id,
        &token_hash(&token),
        SESSION_IDLE_DAYS,
        client_ip(&headers, peer),
        user_agent,
    )
    .await;
    if let Err(error) = created {
        tracing::error!(%error, "creating a login session");
        return (jar, Redirect::to("/login?error=failed")).into_response();
    }

    let cookie = Cookie::build((session_cookie(), token))
        .path("/")
        .http_only(true)
        .secure(auth.secure_cookies())
        .same_site(SameSite::Lax)
        // The server decides when a session ends; browsers cap this at 400 days.
        .max_age(Duration::days(400));
    (jar.add(cookie), Redirect::to(&return_to)).into_response()
}

async fn logout(State(db): State<PgPool>, jar: CookieJar) -> Response {
    if let Some(cookie) = jar.get(session_cookie())
        && let Err(error) = db::revoke_session(&db, &token_hash(cookie.value())).await
    {
        tracing::error!(%error, "revoking a login session");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    let jar = jar
        .remove(Cookie::build(session_cookie()).path("/"))
        .remove(Cookie::build(impersonation_cookie()).path("/"));
    (jar, Redirect::to("/login?logged-out")).into_response()
}

#[derive(Deserialize)]
struct ImpersonateForm {
    user: Uuid,
}

/// An admin starts impersonating another user (or switches to another one): the
/// pages then show what that user sees, and changes are made as them, until
/// they stop or log out.
async fn impersonate(
    State(auth): State<Arc<Auth>>,
    State(db): State<PgPool>,
    actor: Actor,
    jar: CookieJar,
    Form(form): Form<ImpersonateForm>,
) -> Response {
    // While impersonating someone, the request acts as them: check the admin.
    let admin = match &actor.impersonated_by {
        Some(admin) => admin.user_id,
        None if actor.grants.may(Permission::UsersManage, None) => actor.user_id,
        None => return StatusCode::FORBIDDEN.into_response(),
    };
    if form.user == admin {
        return (StatusCode::BAD_REQUEST, "that is you").into_response();
    }
    match db::impersonated_user(&db, form.user).await {
        Ok(Some(_)) => {}
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(error) => {
            tracing::error!(%error, "looking up a user to impersonate");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }
    let cookie = Cookie::build((impersonation_cookie(), form.user.to_string()))
        .path("/")
        .http_only(true)
        .secure(auth.secure_cookies())
        .same_site(SameSite::Lax);
    (jar.add(cookie), Redirect::to("/")).into_response()
}

/// Back to the admin themselves, on the page of the user they impersonated.
async fn stop_impersonating(jar: CookieJar) -> Response {
    let back = match jar.get(impersonation_cookie()) {
        Some(cookie) => match Uuid::parse_str(cookie.value()) {
            Ok(user) => format!("/users/{user}"),
            Err(_) => "/".to_owned(),
        },
        None => "/".to_owned(),
    };
    let jar = jar.remove(Cookie::build(impersonation_cookie()).path("/"));
    (jar, Redirect::to(&back)).into_response()
}

#[derive(Serialize)]
struct Me {
    id: Uuid,
    /// None for a user without a login, whom an admin impersonates.
    email: Option<String>,
    name: String,
    /// So the web app shows only what the user can use; the backend still
    /// checks every request.
    permissions: Permissions,
    /// Whether apps (such as AI assistants) can connect to this install, so
    /// the web app offers the list of connected apps only then.
    apps: bool,
    /// The admin impersonating this user, for the web app's banner.
    impersonated_by: Option<ImpersonatedBy>,
}

#[derive(Serialize)]
struct ImpersonatedBy {
    id: Uuid,
    name: String,
}

/// What the user may do: on the whole organisation, where it counts for
/// every group too, and per group what their memberships add.
#[derive(Serialize)]
struct Permissions {
    organisation: Vec<&'static str>,
    groups: BTreeMap<Uuid, Vec<&'static str>>,
}

impl From<&Grants> for Permissions {
    fn from(grants: &Grants) -> Self {
        let names = |permissions: Vec<Permission>| {
            permissions.into_iter().map(Permission::as_str).collect()
        };
        let (organisation, groups) = grants.permissions();
        Self {
            organisation: names(organisation),
            groups: groups
                .into_iter()
                .map(|(group, permissions)| (group, names(permissions)))
                .collect(),
        }
    }
}

/// The logged-in user, or 401.
async fn me(actor: Actor, State(oauth): State<OAuth>) -> Json<Me> {
    Json(Me {
        id: actor.user_id,
        permissions: Permissions::from(&actor.grants),
        email: actor.email,
        name: actor.name,
        apps: !oauth.resources.is_empty(),
        impersonated_by: actor.impersonated_by.map(|admin| ImpersonatedBy {
            id: admin.user_id,
            name: admin.name,
        }),
    })
}

/// A path on this site to return to after logging in. Anything else, such as
/// `//evil.example` (another host), becomes `/`, so login can't be used to send
/// people to another site.
fn local_path(return_to: Option<&str>) -> String {
    match return_to {
        Some(path)
            if path.starts_with('/')
                && !path.starts_with("//")
                && !path.starts_with("/\\")
                && !path.chars().any(char::is_control) =>
        {
            path.to_owned()
        }
        _ => "/".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::local_path;

    #[test]
    fn return_to_stays_on_this_site() {
        assert_eq!(
            local_path(Some("/?page=2&sort=-air_time_s")),
            "/?page=2&sort=-air_time_s"
        );
        assert_eq!(local_path(None), "/");
        // Browsers treat all of these as another host.
        for path in [
            "https://evil.example/",
            "//evil.example",
            "/\\evil.example",
            "/\t/evil.example",
            "evil",
        ] {
            assert_eq!(local_path(Some(path)), "/", "{path:?}");
        }
    }
}
