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

use super::db::{self, LoginAccount};
use super::oidc::PendingLogin;
use super::{Actor, Auth, SESSION_COOKIE, SESSION_IDLE_DAYS, client_ip, token_hash};

/// Holds a login in progress, only on its way back from the provider.
const LOGIN_COOKIE: &str = "fleetranger_login";

pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    PgPool: FromRef<S>,
    Arc<Auth>: FromRef<S>,
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
            let cookie = Cookie::build((LOGIN_COOKIE, pending.encode()))
                .path("/auth")
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
        .get(LOGIN_COOKIE)
        .and_then(|cookie| PendingLogin::decode(cookie.value()));
    let jar = jar.remove(Cookie::build(LOGIN_COOKIE).path("/auth"));

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

    let return_to = pending.return_to.clone();
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

    let account = db::login_account(
        &db,
        &identity.subject,
        identity.verified_email.as_deref(),
        identity.name.as_deref(),
    )
    .await;
    let account_id = match account {
        Ok(LoginAccount::Allowed(id)) => id,
        Ok(LoginAccount::Refused) => {
            tracing::info!(email = identity.verified_email, "login refused: no account");
            return (jar, Redirect::to("/login?error=no-access")).into_response();
        }
        Err(error) => {
            tracing::error!(%error, "looking up the account for a login");
            return (jar, Redirect::to("/login?error=failed")).into_response();
        }
    };

    let token = CsrfToken::new_random_len(32).into_secret();
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|value| value.to_str().ok());
    let created = db::create_session(
        &db,
        account_id,
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

    let cookie = Cookie::build((SESSION_COOKIE, token))
        .path("/")
        .http_only(true)
        .secure(auth.secure_cookies())
        .same_site(SameSite::Lax)
        // The server decides when a session ends; browsers cap this at 400 days.
        .max_age(Duration::days(400));
    (jar.add(cookie), Redirect::to(&return_to)).into_response()
}

async fn logout(State(db): State<PgPool>, jar: CookieJar) -> Response {
    if let Some(cookie) = jar.get(SESSION_COOKIE)
        && let Err(error) = db::revoke_session(&db, &token_hash(cookie.value())).await
    {
        tracing::error!(%error, "revoking a login session");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    let jar = jar.remove(Cookie::build(SESSION_COOKIE).path("/"));
    (jar, Redirect::to("/login?logged-out")).into_response()
}

#[derive(Serialize)]
struct Me {
    email: String,
    name: Option<String>,
}

/// The logged-in account, or 401.
async fn me(actor: Actor) -> Json<Me> {
    Json(Me {
        email: actor.email,
        name: actor.name,
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
