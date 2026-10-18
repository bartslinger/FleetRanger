//! Pairing remote controllers with the install, and how a paired controller
//! authenticates. See "Clients" in docs/architecture.md.

mod db;
pub mod routes;
mod service;

use axum::{
    extract::{FromRef, FromRequestParts},
    http::{HeaderMap, StatusCode, header, request::Parts},
};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

/// A paired controller, authenticated by the token it sends as
/// `Authorization: Bearer <token>`.
pub struct Controller {
    pub id: Uuid,
    pub serial: String,
    /// The team its flights without a pilot go to; its pilots come first.
    pub team_id: Option<Uuid>,
}

impl<S> FromRequestParts<S> for Controller
where
    S: Send + Sync,
    PgPool: FromRef<S>,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let hash = token_hash(&parts.headers).ok_or(StatusCode::UNAUTHORIZED)?;
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
            team_id: paired.team_id,
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

/// The SHA-256 of the bearer token, which is what the database stores.
fn token_hash(headers: &HeaderMap) -> Option<Vec<u8>> {
    let token = headers
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")?;
    Some(Sha256::digest(token.as_bytes()).to_vec())
}
