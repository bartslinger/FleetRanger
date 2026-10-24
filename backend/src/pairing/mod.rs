//! Pairing remote controllers with the install, and how a paired controller
//! authenticates. See "Clients" in docs/architecture.md.

mod db;
pub mod routes;
pub mod service;

use axum::{
    Json,
    extract::{FromRef, FromRequestParts, Request},
    http::{HeaderMap, StatusCode, header, request::Parts},
    middleware::Next,
    response::{IntoResponse, Response},
};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

/// A paired controller, authenticated by the token it sends as
/// `Authorization: Bearer <token>`.
pub struct Controller {
    pub id: Uuid,
    pub serial: String,
    /// Its group; its pilots come first.
    pub group_id: Option<Uuid>,
}

impl<S> FromRequestParts<S> for Controller
where
    S: Send + Sync,
    PgPool: FromRef<S>,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let hash = token_hash(&parts.headers).ok_or(StatusCode::UNAUTHORIZED)?;
        let controller = db::by_token(&PgPool::from_ref(state), &hash)
            .await
            .map_err(|error| {
                tracing::error!(%error, "looking up a controller token");
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .ok_or(StatusCode::UNAUTHORIZED)?;
        Ok(Self {
            id: controller.id,
            serial: controller.serial,
            group_id: controller.group_id,
        })
    }
}

/// A SHA-256 as the controller sends it: 64 hex digits.
pub fn parse_sha256(hex: &str) -> Option<Vec<u8>> {
    if hex.len() != 64 {
        return None;
    }
    (0..64)
        .step_by(2)
        .map(|i| u8::from_str_radix(hex.get(i..i + 2)?, 16).ok())
        .collect()
}

/// The errors of the controller's API from v1 on: JSON `{"message": …}` for
/// any refusal that says why, and a malformed request (axum's 422) as a 400
/// like any other. See "Clients" in docs/architecture.md.
pub async fn v1_errors(request: Request, next: Next) -> Response {
    let response = next.run(request).await;
    let status = response.status();
    let is_json = response
        .headers()
        .get(header::CONTENT_TYPE)
        .is_some_and(|value| value.as_bytes().starts_with(b"application/json"));
    if !status.is_client_error() || is_json {
        return response;
    }
    // Messages are a sentence; anything larger isn't one of ours.
    let Ok(body) = axum::body::to_bytes(response.into_body(), 64 * 1024).await else {
        return status.into_response();
    };
    if body.is_empty() {
        return status.into_response();
    }
    let status = if status == StatusCode::UNPROCESSABLE_ENTITY {
        StatusCode::BAD_REQUEST
    } else {
        status
    };
    let message = String::from_utf8_lossy(&body).into_owned();
    (status, Json(serde_json::json!({ "message": message }))).into_response()
}

/// The SHA-256 of the bearer token, which is what the database stores.
fn token_hash(headers: &HeaderMap) -> Option<Vec<u8>> {
    let token = headers
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")?;
    Some(Sha256::digest(token.as_bytes()).to_vec())
}
