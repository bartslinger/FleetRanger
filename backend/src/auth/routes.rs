//! Logging in and out. The web app sends the browser to `/auth/login`; the
//! provider sends it back to `/auth/callback`.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    Json, Router,
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
use super::{Actor, Auth, SESSION_IDLE_DAYS, client_ip, login_cookie, session_cookie, token_hash};
use crate::oauth::OAuth;

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
    let jar = jar.remove(Cookie::build(session_cookie()).path("/"));
    (jar, Redirect::to("/login?logged-out")).into_response()
}

#[derive(Serialize)]
struct Me {
    id: Uuid,
    email: String,
    name: String,
    /// So the web app shows only what the user can use; the backend still
    /// checks every request.
    role: &'static str,
    /// Whether apps (such as AI assistants) can connect to this install, so
    /// the web app offers the list of connected apps only then.
    apps: bool,
}

/// The logged-in user, or 401.
async fn me(actor: Actor, State(oauth): State<OAuth>) -> Json<Me> {
    Json(Me {
        id: actor.user_id,
        email: actor.email,
        name: actor.name,
        role: actor.role.as_str(),
        apps: !oauth.resources.is_empty(),
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
