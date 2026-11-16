//! Logging in and out. The web app sends the browser to `/auth/login`; the
//! provider sends it back to `/auth/callback`.

use crate::db::{Db, System, Tenants};
use std::collections::BTreeMap;

use anyhow::Context;
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
use openidconnect::url::form_urlencoded;
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use uuid::Uuid;

use super::db;
use super::oidc::PendingLogin;
use super::{
    Actor, Auth, Person, SESSION_IDLE_DAYS, client_ip, impersonation_cookie, login_cookie,
    session_cookie, token_hash,
};
use crate::oauth::AppsOn;
use crate::oauth::OAuth;
use crate::organisations::signup;
use crate::policy::{Grants, Permission};

pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    crate::db::System: FromRef<S>,
    crate::db::Tenants: FromRef<S>,
    Arc<Auth>: FromRef<S>,
    OAuth: FromRef<S>,
    AppsOn: FromRef<S>,
{
    Router::new()
        .route("/auth/login", get(login))
        .route("/auth/callback", get(callback))
        .route("/auth/signup", post(signup))
        .route("/api/signup", get(signup_status))
        .route("/api/signup/enabled", get(signup_enabled))
        .route("/api/signup/existing", get(existing_status))
        .route("/auth/signup/existing", post(signup_existing))
        .route("/auth/logout", post(logout))
        .route("/auth/impersonate", post(impersonate))
        .route("/auth/impersonate/stop", post(stop_impersonating))
        .route("/auth/organisation", post(switch_organisation))
        .route("/api/me", get(me))
        .route("/api/person", get(person))
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
    let return_to = local_path(query.return_to.as_deref());
    start_login(auth, jar, return_to, None, "/login?error=failed").await
}

#[derive(Deserialize)]
struct SignupForm {
    organisation_name: String,
    time_zone: String,
    /// The checkbox: present only when ticked.
    terms: Option<String>,
}

/// The sign-up page's form: checked, then on to Google, which brings the
/// browser back to `callback` with the choices in the login cookie.
async fn signup(
    State(auth): State<Arc<Auth>>,
    State(system): State<System>,
    jar: CookieJar,
    Form(form): Form<SignupForm>,
) -> Response {
    if auth.signup.is_none() {
        return StatusCode::NOT_FOUND.into_response();
    }
    let request = signup::Request {
        organisation_name: form.organisation_name,
        time_zone: form.time_zone,
    };
    let checked = async {
        let mut db = system.acquire().await?;
        checked_signup(&mut db, request).await
    };
    let request = match checked.await {
        Ok(Some(request)) if form.terms.is_some() => request,
        Ok(_) => return Redirect::to("/signup?error=invalid").into_response(),
        Err(error) => {
            tracing::error!(%error, "checking a sign-up");
            return Redirect::to("/signup?error=failed").into_response();
        }
    };
    start_login(
        auth,
        jar,
        "/".to_owned(),
        Some(request),
        "/signup?error=failed",
    )
    .await
}

/// The sign-up choices, if they are acceptable, the name trimmed. Checked
/// when the form is sent and again when Google sends the browser back, as
/// they come back from a cookie.
async fn checked_signup(
    db: &mut PgConnection,
    request: signup::Request,
) -> sqlx::Result<Option<signup::Request>> {
    let Some(organisation_name) = crate::organisations::valid_name(&request.organisation_name)
    else {
        return Ok(None);
    };
    if !crate::organisations::known_time_zone(&mut *db, &request.time_zone).await? {
        return Ok(None);
    }
    Ok(Some(signup::Request {
        organisation_name,
        time_zone: request.time_zone,
    }))
}

/// Signs someone up whose sign-up choices came back in a cookie, moving
/// `session` to the new organisation if given.
async fn sign_up(
    db: &mut PgConnection,
    request: signup::Request,
    person: &signup::Person<'_>,
    cap: i64,
    session: Option<Uuid>,
) -> anyhow::Result<signup::Outcome> {
    let request = checked_signup(db, request)
        .await?
        .context("sign-up choices changed in the cookie")?;
    Ok(signup::sign_up(db, &request, person, cap, session).await?)
}

/// Whether anyone can sign up here, for the login page's link: from the
/// configuration only, so the login page doesn't depend on the database.
#[derive(Serialize)]
struct SignupEnabled {
    enabled: bool,
}

async fn signup_enabled(State(auth): State<Arc<Auth>>) -> Json<SignupEnabled> {
    Json(SignupEnabled {
        enabled: auth.signup.is_some(),
    })
}

/// What the sign-up page needs; 404 where nobody can sign up.
#[derive(Serialize)]
struct SignupStatus {
    /// The cap is reached: signing up puts people on the waiting list.
    full: bool,
    terms_url: String,
    privacy_url: String,
    time_zones: Vec<String>,
}

async fn signup_status(
    State(auth): State<Arc<Auth>>,
    State(system): State<System>,
) -> Result<Json<SignupStatus>, StatusCode> {
    let Some(config) = &auth.signup else {
        return Err(StatusCode::NOT_FOUND);
    };
    let read = async {
        let mut db = system.acquire().await?;
        let organisations = sqlx::query_scalar!(
            r#"select count(*) as "count!" from organisations where archived_at is null"#
        )
        .fetch_one(&mut *db)
        .await?;
        let time_zones = crate::organisations::time_zones(&mut db).await?;
        Ok::<_, sqlx::Error>((organisations, time_zones))
    };
    let (organisations, time_zones) = read.await.map_err(|error| {
        tracing::error!(%error, "reading the sign-up status");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Json(SignupStatus {
        full: organisations >= config.cap,
        terms_url: config.terms_url.to_string(),
        privacy_url: config.privacy_url.to_string(),
        time_zones,
    }))
}

/// Whether an admin is impersonating someone in this browser: pages before
/// an organisation then act as nobody, as the impersonated user isn't the
/// one logged in.
fn impersonating(jar: &CookieJar) -> bool {
    jar.get(impersonation_cookie()).is_some()
}

/// The sign-up choices kept for someone who came back from Google with an
/// account they had already (user's decision, 2026-11-16: in the login
/// cookie, not in a link anyone could send).
fn pending_signup(jar: &CookieJar) -> Option<signup::Request> {
    jar.get(login_cookie())
        .and_then(|cookie| signup::Request::decode(cookie.value()))
}

/// For the page where someone who signed up with an account they had
/// already chooses: what they chose, and the organisation they are admin
/// of, if any, as they can't create another then. 404 without a sign-up
/// in progress.
#[derive(Serialize)]
struct ExistingStatus {
    organisation_name: String,
    admin_of: Option<OrganisationRef>,
}

#[derive(Serialize)]
struct OrganisationRef {
    id: Uuid,
    name: String,
}

async fn existing_status(
    State(auth): State<Arc<Auth>>,
    State(system): State<System>,
    person: Person,
    jar: CookieJar,
) -> Result<Json<ExistingStatus>, StatusCode> {
    if auth.signup.is_none() || impersonating(&jar) {
        return Err(StatusCode::NOT_FOUND);
    }
    let request = pending_signup(&jar).ok_or(StatusCode::NOT_FOUND)?;
    let read = async {
        let mut db = system.acquire().await?;
        signup::admin_of(&mut db, &person.subject).await
    };
    let admin_of = read.await.map_err(|error| {
        tracing::error!(%error, "reading whether someone is an admin");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Json(ExistingStatus {
        organisation_name: request.organisation_name,
        admin_of: admin_of.map(|(id, name)| OrganisationRef { id, name }),
    }))
}

/// Someone logged in creates the organisation they signed up for, from the
/// choices in the cookie, and the login session moves to it in the same
/// transaction.
async fn signup_existing(
    State(auth): State<Arc<Auth>>,
    State(system): State<System>,
    person: Person,
    jar: CookieJar,
) -> Response {
    let Some(config) = &auth.signup else {
        return StatusCode::NOT_FOUND.into_response();
    };
    if impersonating(&jar) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let Some(request) = pending_signup(&jar) else {
        // The choices last ten minutes; after that the form is filled in again.
        return Redirect::to("/signup?error=expired").into_response();
    };
    let signed_up = async {
        let Some(email) = &person.email else {
            anyhow::bail!("the login has no verified email");
        };
        let name = signup::person_name(person.name.as_deref(), email);
        let account = signup::Person {
            subject: &person.subject,
            email,
            name: &name,
        };
        let mut db = system.acquire().await?;
        sign_up(
            &mut db,
            request,
            &account,
            config.cap,
            Some(person.login_session_id),
        )
        .await
    };
    let outcome = signed_up.await;
    let jar = jar.remove(removal(&auth, login_cookie()));
    match outcome {
        Ok(signup::Outcome::Created {
            organisation_id, ..
        }) => {
            tracing::info!(%organisation_id, "organisation signed up by someone with an account");
            (jar, Redirect::to("/")).into_response()
        }
        Ok(signup::Outcome::Full) => (jar, Redirect::to("/signup?waiting")).into_response(),
        Ok(signup::Outcome::AlreadyAdmin) => (jar, Redirect::to("/")).into_response(),
        Err(error) => {
            tracing::error!(error = format!("{error:#}"), "signing up with an account");
            (jar, Redirect::to("/signup?error=failed")).into_response()
        }
    }
}

/// Sends the browser to the login provider, keeping what it needs on return
/// in a cookie. `failed` is where to go when that can't start.
async fn start_login(
    auth: Arc<Auth>,
    jar: CookieJar,
    return_to: String,
    signup: Option<signup::Request>,
    failed: &'static str,
) -> Response {
    if auth.provider.is_none() {
        return (StatusCode::SERVICE_UNAVAILABLE, "login is not configured").into_response();
    }
    let begun = {
        let auth = auth.clone();
        tokio::task::spawn_blocking(move || {
            let provider = auth.provider.as_ref().context("login is not configured")?;
            provider.begin(return_to, signup)
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
            Redirect::to(failed).into_response()
        }
        Err(error) => {
            tracing::error!(%error, "starting a login");
            Redirect::to(failed).into_response()
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
    State(system): State<System>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    jar: CookieJar,
    headers: HeaderMap,
    Query(query): Query<CallbackQuery>,
) -> Response {
    let pending = jar
        .get(login_cookie())
        .and_then(|cookie| PendingLogin::decode(cookie.value()));
    let jar = jar.remove(removal(&auth, login_cookie()));
    // Someone signing up who cancels at Google, or whose sign-up fails, goes
    // back to the sign-up page, not to logging in.
    let (cancelled, failed) = if pending.as_ref().is_some_and(|p| p.signup.is_some()) {
        ("/signup", "/signup?error=failed")
    } else {
        ("/login", "/login?error=failed")
    };

    let (Some(pending), Some(state), Some(code), None) =
        (pending, query.state, query.code, query.error.as_deref())
    else {
        if query.error.as_deref() == Some("access_denied") {
            return (jar, Redirect::to(cancelled)).into_response();
        }
        tracing::warn!(
            error = query.error,
            "login callback without a login in progress"
        );
        return (jar, Redirect::to(failed)).into_response();
    };
    let Some(_) = &auth.provider else {
        return (StatusCode::SERVICE_UNAVAILABLE, "login is not configured").into_response();
    };

    // Checked again: it comes back from a cookie, which another site on the
    // same domain could have planted, and must never lead off this site.
    let return_to = local_path(Some(&pending.return_to));
    let signing_up = pending.signup.clone();
    let finished = {
        let auth = auth.clone();
        tokio::task::spawn_blocking(move || {
            let provider = auth.provider.as_ref().context("login is not configured")?;
            provider.finish(pending, &state, code)
        })
        .await
    };
    let identity = match finished {
        Ok(Ok(identity)) => identity,
        Ok(Err(error)) => {
            tracing::warn!(error = format!("{error:#}"), "login failed");
            return (jar, Redirect::to(failed)).into_response();
        }
        Err(error) => {
            tracing::error!(%error, "finishing a login");
            return (jar, Redirect::to(failed)).into_response();
        }
    };

    // Before an organisation is known, so across organisations.
    let mut system = match system.acquire().await {
        Ok(connection) => connection,
        Err(error) => {
            tracing::error!(%error, "connecting for a login");
            return (jar, Redirect::to(failed)).into_response();
        }
    };
    let found = db::memberships(
        &mut system,
        &identity.subject,
        identity.verified_email.as_deref(),
    )
    .await;
    let mut return_to = return_to;
    let mut jar = jar;
    let membership = match found {
        Ok(found) if !found.is_empty() => {
            // Signing up with an account that has an organisation already:
            // they choose on the sign-up page whether to create this one.
            // The choices stay in the login cookie, not in a link (user's
            // decision, 2026-11-16).
            if let (Some(_), Some(request)) = (&auth.signup, &signing_up) {
                let cookie = Cookie::build((login_cookie(), request.encode()))
                    .path("/")
                    .http_only(true)
                    .secure(auth.secure_cookies())
                    .same_site(SameSite::Lax)
                    .max_age(Duration::minutes(10));
                jar = jar.add(cookie);
                return_to = "/signup/existing".to_owned();
            }
            match found.as_slice() {
                [only] => Some(*only),
                // Someone in several organisations chooses one first (user's
                // request, 2026-11-16), unless they came to sign up, which
                // asks its own question. The page they were going to comes
                // along.
                _ => {
                    if signing_up.is_none() {
                        return_to = format!(
                            "/choose-organisation?{}",
                            form_urlencoded::Serializer::new(String::new())
                                .append_pair("return_to", &return_to)
                                .finish()
                        );
                    }
                    None
                }
            }
        }
        Ok(_) => {
            let (Some(config), Some(request), Some(email)) = (
                &auth.signup,
                signing_up.clone(),
                identity.verified_email.as_deref(),
            ) else {
                // Signing up needs an email Google has verified; say so.
                if auth.signup.is_some() && signing_up.is_some() {
                    tracing::info!("sign-up refused: email not verified");
                    return (jar, Redirect::to("/signup?error=unverified")).into_response();
                }
                tracing::info!(email = identity.verified_email, "login refused: no login");
                return (jar, Redirect::to("/login?error=no-access")).into_response();
            };
            let name = signup::person_name(identity.name.as_deref(), email);
            let person = signup::Person {
                subject: &identity.subject,
                email,
                name: &name,
            };
            match sign_up(&mut system, request, &person, config.cap, None).await {
                Ok(signup::Outcome::Created {
                    user_id,
                    organisation_id,
                }) => {
                    tracing::info!(%organisation_id, "organisation signed up");
                    Some(db::Membership {
                        user_id,
                        organisation_id,
                    })
                }
                Ok(signup::Outcome::Full) => {
                    tracing::info!("sign-up full: on the waiting list");
                    return (jar, Redirect::to("/signup?waiting")).into_response();
                }
                // Only when the same new account signed up twice at once;
                // left as a refused login (user's choice: simplest code).
                Ok(signup::Outcome::AlreadyAdmin) => {
                    return (jar, Redirect::to("/login?error=no-access")).into_response();
                }
                Err(error) => {
                    tracing::error!(error = format!("{error:#}"), "signing up");
                    return (jar, Redirect::to("/signup?error=failed")).into_response();
                }
            }
        }
        Err(error) => {
            tracing::error!(%error, "looking up the user for a login");
            return (jar, Redirect::to(failed)).into_response();
        }
    };

    let token = CsrfToken::new_random_len(32).into_secret();
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|value| value.to_str().ok());
    let account = db::Account {
        subject: &identity.subject,
        email: identity.verified_email.as_deref(),
        name: identity.name.as_deref(),
    };
    let created = db::create_session(
        &mut system,
        &account,
        membership,
        &token_hash(&token),
        SESSION_IDLE_DAYS,
        client_ip(&headers, peer),
        user_agent,
    )
    .await;
    if let Err(error) = created {
        tracing::error!(%error, "creating a login session");
        return (jar, Redirect::to(failed)).into_response();
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

async fn logout(
    State(auth): State<Arc<Auth>>,
    State(system): State<System>,
    State(tenants): State<Tenants>,
    jar: CookieJar,
) -> Response {
    if let Some(cookie) = jar.get(session_cookie()) {
        // Also an expired session, whose request has no organisation.
        let revoked = async {
            let mut system = system.acquire().await?;
            db::revoke_session(&mut system, &token_hash(cookie.value())).await
        };
        match revoked.await {
            Ok(Some(db::Membership {
                user_id: user,
                organisation_id: organisation,
            })) => {
                let mut db = match tenants.connect(organisation).await {
                    Ok(db) => db,
                    Err(error) => {
                        tracing::error!(%error, "connecting to log out");
                        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                    }
                };
                if let Err(status) = end_impersonation(&mut db, user).await {
                    return status.into_response();
                }
            }
            Ok(None) => {}
            Err(error) => {
                tracing::error!(%error, "revoking a login session");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
    }
    let jar = jar
        .remove(removal(&auth, session_cookie()))
        .remove(removal(&auth, impersonation_cookie()));
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
    mut db: Db,
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
    // Apps connected as the previous user stay with that impersonation.
    if let Err(status) = end_impersonation(&mut db, admin).await {
        return status.into_response();
    }
    match db::impersonated_user(&mut db, form.user).await {
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
async fn stop_impersonating(
    State(auth): State<Arc<Auth>>,
    mut db: Db,
    actor: Actor,
    jar: CookieJar,
) -> Response {
    let admin = actor
        .impersonated_by
        .map_or(actor.user_id, |admin| admin.user_id);
    if let Err(status) = end_impersonation(&mut db, admin).await {
        return status.into_response();
    }
    let back = match jar.get(impersonation_cookie()) {
        Some(cookie) => match Uuid::parse_str(cookie.value()) {
            Ok(user) => format!("/users/{user}"),
            Err(_) => "/".to_owned(),
        },
        None => "/".to_owned(),
    };
    let jar = jar.remove(removal(&auth, impersonation_cookie()));
    (jar, Redirect::to(&back)).into_response()
}

#[derive(Deserialize)]
struct OrganisationForm {
    organisation: Uuid,
    /// Where to go after switching, from the organisation picker; home otherwise.
    return_to: Option<String>,
}

/// Puts the login session in one of the person's organisations, as their
/// user there, for every tab: after logging in to several, or switching
/// from another. An impersonation ends with it, as the impersonated user is
/// in the organisation left. The app starts again at its home page, as
/// nothing on the page shown was in the other one.
async fn switch_organisation(
    State(auth): State<Arc<Auth>>,
    State(system): State<System>,
    State(tenants): State<Tenants>,
    person: Person,
    jar: CookieJar,
    Form(form): Form<OrganisationForm>,
) -> Response {
    // Across organisations: the person's user in the other one.
    let switched = async {
        let mut system = system.acquire().await?;
        db::switch_organisation(&mut system, person.login_session_id, form.organisation).await
    };
    match switched.await {
        Ok(true) => {}
        Ok(false) => return StatusCode::NOT_FOUND.into_response(),
        Err(error) => {
            tracing::error!(%error, "switching organisation");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }
    // Only once switched, in the organisation left, where the
    // impersonation's app connections are.
    if let Some(left) = person.membership
        && impersonating(&jar)
    {
        let mut db = match tenants.connect(left.organisation_id).await {
            Ok(db) => db,
            Err(error) => {
                tracing::error!(%error, "connecting to end an impersonation");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };
        if let Err(status) = end_impersonation(&mut db, left.user_id).await {
            return status.into_response();
        }
    }
    let jar = jar.remove(removal(&auth, impersonation_cookie()));
    (jar, Redirect::to(&local_path(form.return_to.as_deref()))).into_response()
}

/// Takes back the apps an admin connected while impersonating someone
/// ("Impersonation in apps" in docs/architecture.md).
async fn end_impersonation(db: &mut PgConnection, admin: Uuid) -> Result<(), StatusCode> {
    crate::oauth::end_impersonation(db, admin)
        .await
        .map_err(|error| {
            tracing::error!(%error, "ending app connections made while impersonating");
            StatusCode::INTERNAL_SERVER_ERROR
        })
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
    /// The organisation's: the web app shows every day and time in it.
    time_zone: String,
    /// The organisations the person can switch between, the current one
    /// marked; one for most people.
    organisations: Vec<db::OrganisationChoice>,
    /// Whether the organisation has more than one group: until then the web
    /// app shows no groups (user's decision, 2026-11-11).
    several_groups: bool,
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
async fn me(
    actor: Actor,
    State(apps): State<AppsOn>,
    State(system): State<System>,
    mut db: Db,
) -> Result<Json<Me>, StatusCode> {
    let read = async {
        let several_groups = db::several_groups(&mut db).await?;
        let settings = crate::organisations::settings(&mut db).await?;
        // The person's other organisations are looked up across them.
        let organisations =
            db::organisations_of(&mut *system.acquire().await?, actor.login_session_id).await?;
        Ok::<_, sqlx::Error>((several_groups, settings, organisations))
    };
    let (several_groups, settings, organisations) = read.await.map_err(|error| {
        tracing::error!(%error, "reading the user's organisation");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Json(Me {
        id: actor.user_id,
        permissions: Permissions::from(&actor.grants),
        email: actor.email,
        name: actor.name,
        apps: (apps.mcp && settings.mcp) || apps.api,
        time_zone: settings.time_zone,
        organisations,
        impersonated_by: actor.impersonated_by.map(|admin| ImpersonatedBy {
            id: admin.user_id,
            name: admin.name,
        }),
        several_groups,
    }))
}

/// Who is logged in, also before choosing an organisation, for the pages
/// that come before one.
#[derive(Serialize)]
struct PersonView {
    email: Option<String>,
    name: Option<String>,
    /// Whether the login session is in an organisation yet.
    chosen: bool,
    organisations: Vec<db::OrganisationChoice>,
}

/// The person logged in, or 401.
async fn person(
    person: Person,
    State(system): State<System>,
) -> Result<Json<PersonView>, StatusCode> {
    let read = async {
        let mut system = system.acquire().await?;
        db::organisations_of(&mut system, person.login_session_id).await
    };
    let organisations = read.await.map_err(|error| {
        tracing::error!(%error, "reading someone's organisations");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Json(PersonView {
        email: person.email,
        name: person.name,
        chosen: person.membership.is_some(),
        organisations,
    }))
}

/// Deletes one of our cookies. Browsers ignore a `__Host-` cookie without
/// `Secure`, also the one that deletes it, so it carries what they were set
/// with.
fn removal(auth: &Auth, name: &'static str) -> Cookie<'static> {
    Cookie::build(name)
        .path("/")
        .secure(auth.secure_cookies())
        .build()
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
    use super::{Auth, checked_signup, impersonation_cookie, local_path, removal, session_cookie};
    use crate::organisations::signup::Request;

    /// The sign-up choices come back from a cookie anyone can edit: a name
    /// too long or an unknown time zone must create nothing.
    #[sqlx::test]
    async fn tampered_sign_up_choices_are_refused(pool: sqlx::PgPool) {
        let mut db = pool.acquire().await.unwrap();
        let choices = |name: &str, zone: &str| Request {
            organisation_name: name.to_owned(),
            time_zone: zone.to_owned(),
        };
        let long = "x".repeat(101);
        for refused in [
            choices(&long, "UTC"),
            choices("Test Org", "Mars/Olympus"),
            choices("  ", "UTC"),
        ] {
            assert_eq!(checked_signup(&mut db, refused).await.unwrap(), None);
        }
        assert_eq!(
            checked_signup(&mut db, choices("  Test Org ", "Europe/Amsterdam"))
                .await
                .unwrap(),
            Some(choices("Test Org", "Europe/Amsterdam"))
        );
    }

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

    /// Over HTTPS the cookies are `__Host-` ones, which browsers only delete
    /// with `Secure`: without it, Stop impersonating left the cookie in place.
    #[test]
    fn cookies_are_deleted_as_they_were_set() {
        let auth = Auth {
            public_url: "https://fleet.example".parse().unwrap(),
            provider: None,
            signup: None,
        };
        for name in [session_cookie(), impersonation_cookie()] {
            assert!(name.starts_with("__Host-"));
            let cookie = removal(&auth, name);
            assert_eq!(cookie.secure(), Some(true));
            assert_eq!(cookie.path(), Some("/"));
        }
    }
}
