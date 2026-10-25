//! The public API for apps, at `/api/v1`: read-only, versioned, and separate
//! from the web app's `/api`. Apps act for a user with an OAuth token for
//! this resource, under that user's access; its scopes say which resources
//! and how deep. On only when the install switches it on
//! (`FLEETRANGER_API`). See docs/api-v1.md.

mod db;
mod routes;
mod types;

use std::{collections::BTreeSet, sync::Arc};

use axum::{
    Json, Router,
    extract::{Request, State},
    http::{HeaderValue, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use base64::Engine;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sqlx::PgPool;
use utoipa::{
    Modify, OpenApi,
    openapi::security::{AuthorizationCode, Flow, OAuth2, Scopes, SecurityScheme},
};
use utoipa_axum::router::OpenApiRouter;

use crate::{
    auth::Actor,
    error::ApiError,
    oauth::{self, API_SCOPES, Bearer},
};

/// The address tokens for the API are issued for.
pub fn resource(issuer: &str) -> String {
    format!("{issuer}/api/v1")
}

/// What the API's handlers need.
#[derive(Clone)]
pub struct Api {
    pub db: PgPool,
    pub time_zone: Arc<str>,
    pub organisation_name: Option<Arc<str>>,
    /// This install's `/api/v1`, e.g. `https://fleet.example.org/api/v1`.
    pub resource: Arc<str>,
}

/// The user a request acts for, and the scopes their token carries.
#[derive(Clone)]
pub struct Caller {
    pub actor: Actor,
    scopes: Vec<String>,
}

impl Caller {
    /// Whether the token allows this scope: a level includes the ones below
    /// it (`read` includes `list`, `write` both).
    fn allows(&self, needed: &str) -> bool {
        let (resource, level) = needed.split_once(':').unwrap_or((needed, ""));
        let rank = |level: &str| match level {
            "list" => 1,
            "read" => 2,
            "write" => 3,
            _ => 0,
        };
        self.scopes.iter().any(|granted| {
            granted
                .split_once(':')
                .is_some_and(|(granted_resource, granted_level)| {
                    granted_resource == resource && rank(granted_level) >= rank(level)
                })
        })
    }

    /// Refuses a request the token's scopes don't cover.
    fn require(&self, scope: &'static str) -> Result<(), V1Error> {
        if self.allows(scope) {
            Ok(())
        } else {
            Err(V1Error::MissingScope(scope))
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "FleetRanger API",
        version = "1",
        description = include_str!("description.md")
    ),
    tags(
        (name = "Me"), (name = "Organisation"), (name = "Sessions"), (name = "Flights"),
        (name = "Groups"), (name = "Areas"), (name = "Drones"), (name = "Controllers"),
        (name = "Batteries")
    ),
    // An expanded session in a flight: named here, as the recursion between
    // the two leaves it out otherwise.
    components(schemas(types::Expandable<types::Session>)),
    modifiers(&Security)
)]
struct ApiDoc;

/// The OAuth scheme; its addresses are the install's, filled in by `router`.
struct Security;

impl Modify for Security {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let scopes = Scopes::from_iter(API_SCOPES.iter().map(|&scope| {
            let description = match scope {
                "sessions:list" => "Sessions: their figures, group, pilots and areas.",
                "sessions:read" => "Sessions with their comments.",
                "flights:list" => {
                    "Flights: their facts, pilot, group, equipment and areas, without tracks."
                }
                "groups:list" => "The list of groups.",
                "areas:list" => "The list of areas, by name.",
                "equipment:list" => "The lists of drones, controllers and batteries.",
                other => other,
            };
            (scope, description)
        }));
        let flow = Flow::AuthorizationCode(AuthorizationCode::new(
            "/connect",
            "/auth/oauth/token",
            scopes,
        ));
        openapi
            .components
            .get_or_insert_default()
            .add_security_scheme("oauth", SecurityScheme::OAuth2(OAuth2::new([flow])));
    }
}

/// `/api/v1`, and its description at `/api/v1/openapi.json`, which needs no
/// token.
pub fn router<S>(api: Api) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let (routes, openapi) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(routes::router())
        .split_for_parts();
    let openapi = Json(openapi);
    routes
        .layer(middleware::from_fn_with_state(api.clone(), authenticate))
        .route("/api/v1/openapi.json", get(move || async move { openapi }))
        .with_state(api)
}

/// Who may use the API: a valid token for it, acting for its user. Debug
/// builds also accept a login session, with every scope, for trying it out.
async fn authenticate(State(api): State<Api>, request: Request, next: Next) -> Response {
    let (mut parts, body) = request.into_parts();
    let bearer = match oauth::bearer(&api.db, &parts, &api.resource, None).await {
        Ok(bearer) => bearer,
        Err(error) => return V1Error::from(error).into_response(),
    };
    let caller = match bearer {
        Bearer::Valid(actor, scopes) => Ok(Caller {
            actor: *actor,
            scopes,
        }),
        Bearer::Missing => Err(V1Error::Unauthorized(None)),
        Bearer::Invalid => Err(V1Error::Unauthorized(Some("invalid_token"))),
        Bearer::Insufficient => Err(V1Error::Unauthorized(Some("invalid_token"))),
    };
    #[cfg(debug_assertions)]
    let caller = match caller {
        Err(V1Error::Unauthorized(error)) => {
            use axum::extract::FromRequestParts;
            Actor::from_request_parts(&mut parts, &api.db)
                .await
                .map(|actor| Caller {
                    actor,
                    scopes: API_SCOPES.iter().map(|scope| (*scope).to_owned()).collect(),
                })
                .map_err(|_| V1Error::Unauthorized(error))
        }
        other => other,
    };
    match caller {
        Ok(caller) => {
            parts.extensions.insert(caller);
            next.run(Request::from_parts(parts, body)).await
        }
        Err(error) => error
            .challenge(&format!(
                "{}/.well-known/oauth-protected-resource/api/v1",
                issuer(&api)
            ))
            .into_response(),
    }
}

/// The install's address, from the API's.
fn issuer(api: &Api) -> &str {
    api.resource
        .strip_suffix("/api/v1")
        .unwrap_or(&api.resource)
}

/// Why a request was refused.
#[derive(Debug)]
pub enum V1Error {
    /// A malformed request, filter or expansion (400).
    Invalid(String),
    /// No valid token for this API (401), with the OAuth error code if one
    /// was sent.
    Unauthorized(Option<&'static str>),
    /// The token lacks a scope the request needs (403).
    MissingScope(&'static str),
    /// It doesn't exist, or the user can't see it (404).
    NotFound,
    Internal,
}

impl V1Error {
    /// The refusal with its `WWW-Authenticate` header (RFC 6750, RFC 9728).
    fn challenge(self, metadata: &str) -> Response {
        let mut value = format!(r#"Bearer resource_metadata="{metadata}""#);
        if let Self::Unauthorized(Some(error)) = &self {
            value.push_str(&format!(r#", error="{error}""#));
        }
        let mut response = self.into_response();
        if let Ok(value) = HeaderValue::from_str(&value) {
            response
                .headers_mut()
                .insert(header::WWW_AUTHENTICATE, value);
        }
        response
    }
}

impl From<sqlx::Error> for V1Error {
    fn from(error: sqlx::Error) -> Self {
        tracing::error!(%error, "answering the public API");
        Self::Internal
    }
}

impl From<ApiError> for V1Error {
    fn from(error: ApiError) -> Self {
        match error {
            ApiError::NotFound | ApiError::Forbidden => Self::NotFound,
            ApiError::Invalid(message) | ApiError::Conflict(message) => Self::Invalid(message),
            ApiError::Internal(error) => {
                tracing::error!(error = format!("{error:#}"), "answering the public API");
                Self::Internal
            }
        }
    }
}

impl IntoResponse for V1Error {
    fn into_response(self) -> Response {
        // RFC 6750: a missing scope is named in the header too.
        let insufficient = match &self {
            Self::MissingScope(scope) => Some(format!(
                r#"Bearer error="insufficient_scope", scope="{scope}""#
            )),
            _ => None,
        };
        let (status, message) = match self {
            Self::Invalid(message) => (StatusCode::BAD_REQUEST, message),
            Self::Unauthorized(_) => (
                StatusCode::UNAUTHORIZED,
                "Connect the app to FleetRanger again: this request has no valid access token."
                    .to_owned(),
            ),
            Self::MissingScope(scope) => (
                StatusCode::FORBIDDEN,
                format!("This app wasn’t allowed this: it needs the {scope} scope."),
            ),
            Self::NotFound => (StatusCode::NOT_FOUND, "Not found.".to_owned()),
            Self::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Something went wrong in FleetRanger. Try again later.".to_owned(),
            ),
        };
        let mut response = (status, Json(types::Error { message })).into_response();
        if let Some(value) = insufficient.and_then(|value| HeaderValue::from_str(&value).ok()) {
            response
                .headers_mut()
                .insert(header::WWW_AUTHENTICATE, value);
        }
        response
    }
}

/// The fields a request asked to expand, such as `flights.batteries`.
pub struct Expand(BTreeSet<String>);

impl Expand {
    /// From the `expand` parameter (comma-separated), refusing paths this
    /// list doesn't offer and those the token's scopes don't cover.
    /// `offered` pairs each path with the scope it needs, if any.
    fn parse(
        value: Option<&str>,
        offered: &[(&str, Option<&'static str>)],
        caller: &Caller,
    ) -> Result<Self, V1Error> {
        let mut paths = BTreeSet::new();
        for path in value.unwrap_or("").split(',').map(str::trim) {
            if path.is_empty() {
                continue;
            }
            let Some((_, scope)) = offered.iter().find(|(offered, _)| *offered == path) else {
                let names: Vec<&str> = offered.iter().map(|(path, _)| *path).collect();
                return Err(V1Error::Invalid(format!(
                    "`{path}` can’t be expanded here. These can: {}.",
                    names.join(", ")
                )));
            };
            if let Some(scope) = scope {
                caller.require(scope)?;
            }
            // A path expands what leads to it too: `flights.batteries` the
            // flights.
            let mut prefix = path;
            while let Some((parent, _)) = prefix.rsplit_once('.') {
                if offered.iter().any(|(offered, _)| *offered == parent) {
                    paths.insert(parent.to_owned());
                }
                prefix = parent;
            }
            paths.insert(path.to_owned());
        }
        Ok(Self(paths))
    }

    fn has(&self, path: &str) -> bool {
        self.0.contains(path)
    }

    /// The paths under a prefix, without it: `flights.batteries` is
    /// `batteries` under `flights`.
    fn under(&self, prefix: &str) -> Self {
        Self(
            self.0
                .iter()
                .filter_map(|path| path.strip_prefix(prefix)?.strip_prefix('.'))
                .map(str::to_owned)
                .collect(),
        )
    }
}

/// A page's size: 1 to 100, 50 if not given.
fn limit(limit: Option<i64>) -> Result<i64, V1Error> {
    match limit {
        None => Ok(50),
        Some(limit @ 1..=100) => Ok(limit),
        Some(_) => Err(V1Error::Invalid(
            "`limit` must be between 1 and 100.".to_owned(),
        )),
    }
}

/// A cursor: where the last page ended, opaque to apps (base64url JSON).
fn encode<T: Serialize>(position: &T) -> String {
    let json = serde_json::to_vec(position).unwrap_or_default();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(json)
}

fn decode<T: DeserializeOwned>(cursor: Option<&str>) -> Result<Option<T>, V1Error> {
    let Some(cursor) = cursor else {
        return Ok(None);
    };
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(cursor)
        .ok()
        .and_then(|json| serde_json::from_slice(&json).ok())
        .map(Some)
        .ok_or_else(|| {
            V1Error::Invalid(
                "`cursor` must be a `next_cursor` from an earlier page, unchanged.".to_owned(),
            )
        })
}

/// Where a page of sessions or flights ended: the last one's time and id.
#[derive(Serialize, Deserialize)]
struct TimeCursor {
    at: chrono::DateTime<chrono::Utc>,
    id: uuid::Uuid,
}

/// Where a page of groups, areas or equipment ended: the last one's id.
#[derive(Serialize, Deserialize)]
struct IdCursor {
    id: uuid::Uuid,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A scope covers the lower levels of its own resource only, and an
    /// expansion that reaches another resource or level needs that scope:
    /// it is refused, not left out.
    #[sqlx::test]
    async fn scopes_cover_lower_levels_and_guard_expansions(db: PgPool) {
        let user = crate::testing::user(&db, "A", &["viewer"]).await;
        let caller = Caller {
            actor: crate::testing::actor(&db, user).await,
            scopes: vec!["sessions:read".to_owned()],
        };
        assert!(caller.allows("sessions:list"));
        assert!(caller.allows("sessions:read"));
        assert!(!caller.allows("flights:list"));
        let offered = [
            ("comments", Some("sessions:read")),
            ("flights", Some("flights:list")),
            ("flights.batteries", Some("flights:list")),
        ];
        assert!(Expand::parse(Some("comments"), &offered, &caller).is_ok());
        assert!(matches!(
            Expand::parse(Some("flights.batteries"), &offered, &caller),
            Err(V1Error::MissingScope("flights:list"))
        ));
    }
}
