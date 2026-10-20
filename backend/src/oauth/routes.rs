//! The OAuth endpoints. Apps find them through the metadata under
//! `/.well-known/`, register at `/auth/oauth/register` and get tokens at
//! `/auth/oauth/token`. The consent page is the web app's `/connect`, which
//! checks and approves a request through `/api/oauth/…`, as does the list of
//! connected apps.

use axum::{
    Form, Json, Router,
    extract::{DefaultBodyLimit, FromRef, Path, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};
use openidconnect::url::Url;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use super::{
    ACCESS_SECONDS, CODE_SECONDS, IDLE_DAYS, OAuth, SCOPES,
    db::{self, Code, Connection, Refreshed, Tokens},
    pkce_matches, random_token, sha256,
};
use crate::{
    auth::Actor,
    error::{ApiError, ApiResult},
};

pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    PgPool: FromRef<S>,
    OAuth: FromRef<S>,
{
    Router::new()
        .route(
            "/.well-known/oauth-authorization-server",
            get(server_metadata),
        )
        .route(
            "/.well-known/oauth-protected-resource/{*path}",
            get(resource_metadata),
        )
        // Anyone may register, so a registration is kept small.
        .route(
            "/auth/oauth/register",
            post(register).layer(DefaultBodyLimit::max(8 * 1024)),
        )
        .route("/auth/oauth/token", post(token))
        .route("/api/oauth/request", post(check))
        .route("/api/oauth/approve", post(approve))
        .route("/api/oauth/connections", get(connections))
        .route("/api/oauth/connections/{id}", delete(disconnect))
}

/// RFC 8414: where apps find the endpoints.
async fn server_metadata(State(oauth): State<OAuth>) -> Json<serde_json::Value> {
    let issuer = &oauth.issuer;
    Json(json!({
        "issuer": issuer,
        "authorization_endpoint": format!("{issuer}/connect"),
        "token_endpoint": format!("{issuer}/auth/oauth/token"),
        "registration_endpoint": format!("{issuer}/auth/oauth/register"),
        "scopes_supported": SCOPES,
        "response_types_supported": ["code"],
        "grant_types_supported": ["authorization_code", "refresh_token"],
        "code_challenge_methods_supported": ["S256"],
        "token_endpoint_auth_methods_supported": ["none"],
        // RFC 9207: the answer names its issuer, so apps can check it.
        "authorization_response_iss_parameter_supported": true,
    }))
}

/// RFC 9728: which server issues tokens for a resource, e.g. for
/// `/.well-known/oauth-protected-resource/mcp` the one for `/mcp`.
async fn resource_metadata(
    State(oauth): State<OAuth>,
    Path(path): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let resource = format!("{}/{path}", oauth.issuer);
    if !oauth.resources.contains(&resource) {
        return Err(ApiError::NotFound);
    }
    Ok(Json(json!({
        "resource": resource,
        "authorization_servers": [oauth.issuer],
        "scopes_supported": SCOPES,
        "bearer_methods_supported": ["header"],
    })))
}

#[derive(Deserialize)]
struct Registration {
    client_name: Option<String>,
    redirect_uris: Vec<String>,
}

/// RFC 7591: an app registers itself, which grants nothing until a user
/// allows it. Only apps without a secret (with PKCE), as AI assistants are.
async fn register(State(db): State<PgPool>, Json(registration): Json<Registration>) -> Response {
    let uris = &registration.redirect_uris;
    if uris.is_empty() || uris.len() > 10 || !uris.iter().all(|uri| redirect_uri_allowed(uri)) {
        return oauth_error(
            StatusCode::BAD_REQUEST,
            "invalid_redirect_uri",
            "Redirect URIs must use HTTPS, or HTTP on this computer.",
        );
    }
    let name: String = registration
        .client_name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .unwrap_or("Unnamed app")
        .chars()
        .take(100)
        .collect();
    match db::register(&db, &name, uris).await {
        Ok(id) => (
            StatusCode::CREATED,
            Json(json!({
                "client_id": id,
                "client_name": name,
                "redirect_uris": uris,
                "grant_types": ["authorization_code", "refresh_token"],
                "response_types": ["code"],
                "token_endpoint_auth_method": "none",
            })),
        )
            .into_response(),
        Err(error) => {
            tracing::error!(%error, "registering an OAuth app");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// HTTPS, or plain HTTP to the user's own computer (apps such as Claude Code
/// receive the code there); never with a fragment, and of a sensible length.
fn redirect_uri_allowed(uri: &str) -> bool {
    let Ok(url) = Url::parse(uri) else {
        return false;
    };
    uri.len() <= 2000
        && url.fragment().is_none()
        && (url.scheme() == "https" || (url.scheme() == "http" && loopback(&url)))
}

fn loopback(url: &Url) -> bool {
    matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"))
}

/// Whether a request's return address is one the app registered: exactly,
/// or on the user's own computer with any port, which such apps pick when
/// they start (RFC 8252, 7.3).
fn registered(given: &str, registered: &[String]) -> bool {
    if registered.iter().any(|uri| uri == given) {
        return true;
    }
    let Ok(mut given) = Url::parse(given) else {
        return false;
    };
    if given.scheme() != "http" || !loopback(&given) || given.set_port(None).is_err() {
        return false;
    }
    registered.iter().any(|uri| {
        Url::parse(uri)
            .is_ok_and(|mut uri| loopback(&uri) && uri.set_port(None).is_ok() && uri == given)
    })
}

/// An authorization request, as the app sent it to the consent page.
#[derive(Deserialize)]
struct AuthorizationRequest {
    response_type: Option<String>,
    client_id: Option<String>,
    redirect_uri: Option<String>,
    code_challenge: Option<String>,
    code_challenge_method: Option<String>,
    scope: Option<String>,
    resource: Option<String>,
    state: Option<String>,
}

/// A request found valid: who asks, for what, and where the answer goes.
struct Valid {
    client_id: Uuid,
    app: String,
    redirect_uri: String,
    code_challenge: String,
    scopes: Vec<String>,
    resource: String,
}

/// Checks a request. A message on failure, for the consent page: the app
/// isn't sent an error, since it may not be who it says it is.
async fn validate(db: &PgPool, oauth: &OAuth, request: &AuthorizationRequest) -> ApiResult<Valid> {
    let invalid = |message: &str| ApiError::Invalid(message.to_owned());
    let client = match request.client_id.as_deref().map(str::parse::<Uuid>) {
        Some(Ok(id)) => db::client(db, id).await?,
        _ => None,
    }
    .ok_or_else(|| invalid("The app isn’t registered with FleetRanger."))?;
    let redirect_uri = request
        .redirect_uri
        .clone()
        .filter(|uri| registered(uri, &client.redirect_uris))
        .ok_or_else(|| invalid("The app gave a return address it didn’t register."))?;
    if request.response_type.as_deref() != Some("code") {
        return Err(invalid("The app asked for an unsupported kind of access."));
    }
    let code_challenge = request
        .code_challenge
        .clone()
        .filter(|challenge| (43..=128).contains(&challenge.len()))
        .filter(|_| request.code_challenge_method.as_deref() == Some("S256"))
        .ok_or_else(|| invalid("The app didn’t secure its request (PKCE)."))?;
    let scopes: Vec<String> = match request.scope.as_deref().map(str::trim) {
        None | Some("") => SCOPES.iter().map(|&scope| scope.to_owned()).collect(),
        Some(scope) => scope.split(' ').map(str::to_owned).collect(),
    };
    if !scopes.iter().all(|scope| SCOPES.contains(&scope.as_str())) {
        return Err(invalid(
            "The app asked for access FleetRanger doesn’t give.",
        ));
    }
    let resource = match (&request.resource, &*oauth.resources) {
        (Some(resource), resources) if resources.contains(resource) => resource.clone(),
        (None, [only]) => only.clone(),
        _ => {
            return Err(invalid(
                "The app asked for a part of FleetRanger that doesn’t exist.",
            ));
        }
    };
    Ok(Valid {
        client_id: client.id,
        app: client.name,
        redirect_uri,
        code_challenge,
        scopes,
        resource,
    })
}

/// What the consent page shows: the app's name, where it sends the user
/// back (its host, as the name is only what the app says it is), and the
/// scopes it asks for.
#[derive(Serialize)]
struct RequestInfo {
    app: String,
    host: String,
    scopes: Vec<String>,
}

async fn check(
    _actor: Actor,
    State(db): State<PgPool>,
    State(oauth): State<OAuth>,
    Json(request): Json<AuthorizationRequest>,
) -> ApiResult<Json<RequestInfo>> {
    let valid = validate(&db, &oauth, &request).await?;
    Ok(Json(RequestInfo {
        host: host(&valid.redirect_uri),
        app: valid.app,
        scopes: valid.scopes,
    }))
}

fn host(uri: &str) -> String {
    Url::parse(uri)
        .ok()
        .and_then(|url| url.host_str().map(str::to_owned))
        .unwrap_or_default()
}

#[derive(Deserialize)]
struct Decision {
    #[serde(flatten)]
    request: AuthorizationRequest,
    allow: bool,
}

/// Where the browser goes next: back to the app, with a code or a refusal.
#[derive(Serialize)]
struct Redirect {
    redirect: String,
}

/// The user's answer on the consent page.
async fn approve(
    actor: Actor,
    State(db): State<PgPool>,
    State(oauth): State<OAuth>,
    Json(decision): Json<Decision>,
) -> ApiResult<Json<Redirect>> {
    let valid = validate(&db, &oauth, &decision.request).await?;
    let mut url = Url::parse(&valid.redirect_uri)
        .map_err(|_| ApiError::Invalid("The app’s return address is invalid.".to_owned()))?;
    let code = if decision.allow {
        let code = random_token();
        let stored = Code {
            client_id: valid.client_id,
            user_id: actor.user_id,
            redirect_uri: valid.redirect_uri,
            code_challenge: valid.code_challenge,
            scopes: valid.scopes,
            resource: valid.resource,
        };
        db::create_code(&db, &sha256(&code), &stored, CODE_SECONDS).await?;
        Some(code)
    } else {
        None
    };
    {
        let mut query = url.query_pairs_mut();
        match &code {
            Some(code) => query.append_pair("code", code),
            None => query.append_pair("error", "access_denied"),
        };
        if let Some(state) = &decision.request.state {
            query.append_pair("state", state);
        }
        query.append_pair("iss", &oauth.issuer);
    }
    Ok(Json(Redirect {
        redirect: url.into(),
    }))
}

#[derive(Deserialize)]
struct TokenRequest {
    grant_type: String,
    client_id: Option<String>,
    code: Option<String>,
    redirect_uri: Option<String>,
    code_verifier: Option<String>,
    refresh_token: Option<String>,
    resource: Option<String>,
}

/// The app exchanges a code, or a refresh token, for new tokens.
async fn token(State(db): State<PgPool>, Form(request): Form<TokenRequest>) -> Response {
    match issue(&db, &request).await {
        Ok(response) => response,
        Err(TokenError::Refused(error)) => {
            oauth_error(StatusCode::BAD_REQUEST, error, "The request was refused.")
        }
        Err(TokenError::Database(error)) => {
            tracing::error!(%error, "issuing OAuth tokens");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

enum TokenError {
    /// An OAuth error code, e.g. `invalid_grant`.
    Refused(&'static str),
    Database(sqlx::Error),
}

impl From<sqlx::Error> for TokenError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

async fn issue(db: &PgPool, request: &TokenRequest) -> Result<Response, TokenError> {
    let client_id: Uuid = request
        .client_id
        .as_deref()
        .and_then(|id| id.parse().ok())
        .ok_or(TokenError::Refused("invalid_client"))?;
    let access = random_token();
    let refresh = random_token();
    let (access_sha256, refresh_sha256) = (sha256(&access), sha256(&refresh));
    let tokens = Tokens {
        access_sha256: &access_sha256,
        refresh_sha256: &refresh_sha256,
        access_seconds: ACCESS_SECONDS,
    };
    let scopes = match request.grant_type.as_str() {
        "authorization_code" => {
            let (Some(code), Some(verifier)) = (&request.code, &request.code_verifier) else {
                return Err(TokenError::Refused("invalid_request"));
            };
            let code = db::take_code(db, &sha256(code))
                .await?
                .filter(|code| code.client_id == client_id)
                .filter(|code| request.redirect_uri.as_ref() == Some(&code.redirect_uri))
                .filter(|code| pkce_matches(verifier, &code.code_challenge))
                .ok_or(TokenError::Refused("invalid_grant"))?;
            // RFC 8707: a token for another resource than the one allowed.
            if request
                .resource
                .as_ref()
                .is_some_and(|r| *r != code.resource)
            {
                return Err(TokenError::Refused("invalid_target"));
            }
            db::create_grant(db, &code, &tokens).await?;
            code.scopes
        }
        "refresh_token" => {
            let refresh_token = request
                .refresh_token
                .as_deref()
                .ok_or(TokenError::Refused("invalid_request"))?;
            let resource = request.resource.as_deref();
            match db::refresh(
                db,
                &sha256(refresh_token),
                client_id,
                &tokens,
                IDLE_DAYS,
                resource,
            )
            .await?
            {
                Refreshed::Renewed(scopes) => scopes,
                Refreshed::Reused => {
                    tracing::warn!(%client_id, "an old refresh token came back; the app was disconnected");
                    return Err(TokenError::Refused("invalid_grant"));
                }
                Refreshed::Refused => return Err(TokenError::Refused("invalid_grant")),
            }
        }
        _ => return Err(TokenError::Refused("unsupported_grant_type")),
    };
    let mut response = Json(json!({
        "access_token": access,
        "token_type": "Bearer",
        "expires_in": ACCESS_SECONDS as u32,
        "refresh_token": refresh,
        "scope": scopes.join(" "),
    }))
    .into_response();
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    Ok(response)
}

fn oauth_error(status: StatusCode, error: &str, description: &str) -> Response {
    (
        status,
        Json(json!({ "error": error, "error_description": description })),
    )
        .into_response()
}

/// The apps the user connected.
async fn connections(actor: Actor, State(db): State<PgPool>) -> ApiResult<Json<Vec<Connection>>> {
    Ok(Json(db::connections(&db, actor.user_id).await?))
}

/// Disconnects one of the user's apps: its tokens stop working at once.
async fn disconnect(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    if db::disconnect(&db, actor.user_id, id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
    const CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
    const REDIRECT: &str = "https://app.example.org/callback";
    const RESOURCE: &str = "https://fleet.example.org/mcp";

    fn request(grant_type: &str, client_id: Uuid) -> TokenRequest {
        TokenRequest {
            grant_type: grant_type.to_owned(),
            client_id: Some(client_id.to_string()),
            code: None,
            redirect_uri: Some(REDIRECT.to_owned()),
            code_verifier: Some(VERIFIER.to_owned()),
            refresh_token: None,
            resource: Some(RESOURCE.to_owned()),
        }
    }

    /// The tokens in a successful response, or the OAuth error code.
    async fn tokens(db: &PgPool, request: &TokenRequest) -> Result<(String, String), &'static str> {
        let response = match issue(db, request).await {
            Ok(response) => response,
            Err(TokenError::Refused(error)) => return Err(error),
            Err(TokenError::Database(error)) => panic!("{error}"),
        };
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
        Ok((
            body["access_token"].as_str().unwrap().to_owned(),
            body["refresh_token"].as_str().unwrap().to_owned(),
        ))
    }

    /// A code works once, only with its PKCE verifier; a refresh token works
    /// once, its new access token replaces the old one, and a used one coming
    /// back ends the connection.
    /// Apps on the user's computer may come back on another port; anything
    /// else must be exactly as registered.
    #[test]
    fn return_addresses_match_as_registered() {
        let local = ["http://127.0.0.1:33418/callback".to_owned()];
        assert!(registered("http://127.0.0.1:50211/callback", &local));
        assert!(!registered("http://127.0.0.1:50211/other", &local));
        assert!(!registered("http://localhost:33418/callback", &local));
        let web = ["https://app.example.org/callback".to_owned()];
        assert!(registered("https://app.example.org/callback", &web));
        assert!(!registered("https://app.example.org:8443/callback", &web));
    }

    #[sqlx::test]
    async fn codes_and_refresh_tokens_work_once(db: PgPool) {
        let user: Uuid = sqlx::query_scalar("insert into users (name) values ('A') returning id")
            .fetch_one(&db)
            .await
            .unwrap();
        sqlx::query("insert into logins (user_id, email) values ($1, 'a@example.org')")
            .bind(user)
            .execute(&db)
            .await
            .unwrap();
        let client = db::register(&db, "Test app", &[REDIRECT.to_owned()])
            .await
            .unwrap();
        let code = Code {
            client_id: client,
            user_id: user,
            redirect_uri: REDIRECT.to_owned(),
            code_challenge: CHALLENGE.to_owned(),
            scopes: vec!["flights:read".to_owned()],
            resource: RESOURCE.to_owned(),
        };
        db::create_code(&db, &sha256("good"), &code, 60.0)
            .await
            .unwrap();
        db::create_code(&db, &sha256("guessed"), &code, 60.0)
            .await
            .unwrap();

        let mut wrong_verifier = request("authorization_code", client);
        wrong_verifier.code = Some("guessed".to_owned());
        wrong_verifier.code_verifier = Some("not-the-verifier".to_owned());
        assert_eq!(tokens(&db, &wrong_verifier).await, Err("invalid_grant"));
        // A failed attempt uses the code up too.
        wrong_verifier.code_verifier = Some(VERIFIER.to_owned());
        assert_eq!(tokens(&db, &wrong_verifier).await, Err("invalid_grant"));

        let mut exchange = request("authorization_code", client);
        exchange.code = Some("good".to_owned());
        let (access, refresh) = tokens(&db, &exchange).await.unwrap();
        assert_eq!(tokens(&db, &exchange).await, Err("invalid_grant"));
        assert!(
            db::use_access(&db, &sha256(&access))
                .await
                .unwrap()
                .is_some()
        );

        let mut renewal = request("refresh_token", client);
        renewal.refresh_token = Some(refresh);
        let (new_access, new_refresh) = tokens(&db, &renewal).await.unwrap();
        let works =
            async |token: &str| db::use_access(&db, &sha256(token)).await.unwrap().is_some();
        assert!(!works(&access).await);
        assert!(works(&new_access).await);

        // The old refresh token again: copied, so the connection ends for both.
        assert_eq!(tokens(&db, &renewal).await, Err("invalid_grant"));
        assert!(!works(&new_access).await);
        renewal.refresh_token = Some(new_refresh);
        assert_eq!(tokens(&db, &renewal).await, Err("invalid_grant"));
    }
}
