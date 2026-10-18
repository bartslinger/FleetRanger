//! Pairing remote controllers with the install, and how a paired controller
//! authenticates. See "Clients" in docs/architecture.md.

mod db;
pub mod routes;
mod service;

use axum::{
    extract::{FromRef, FromRequestParts},
    http::{StatusCode, header, request::Parts},
};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

/// A paired controller, authenticated by the token it sends as
/// `Authorization: Bearer <token>`.
pub struct Controller {
    pub id: Uuid,
    #[expect(dead_code, reason = "read by uploads, which come next")]
    pub serial: String,
}

impl<S> FromRequestParts<S> for Controller
where
    S: Send + Sync,
    PgPool: FromRef<S>,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or(StatusCode::UNAUTHORIZED)?;
        let hash = Sha256::digest(token.as_bytes());
        let paired = db::by_token(&PgPool::from_ref(state), &hash)
            .await
            .map_err(|error| {
                tracing::error!(%error, "looking up a controller token");
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .ok_or(StatusCode::UNAUTHORIZED)?;
        Ok(Self {
            id: paired.id,
            serial: paired.serial,
        })
    }
}
