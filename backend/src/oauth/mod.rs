//! OAuth: apps such as AI assistants act for a user with tokens FleetRanger
//! issues, after the user allowed it on the consent page. Each token is for
//! one resource (e.g. `/mcp`) and its scopes; what a resource then allows is
//! that resource's business. See "Identity and access" in docs/architecture.md.

mod db;
pub mod routes;

use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::ConnectInfo,
    http::{header, request::Parts},
};
use base64::Engine;
use sha2::{Digest, Sha256};
use sqlx::PgPool;

use crate::auth::Actor;

/// What an app can be allowed to do. Writes come later.
pub const SCOPES: &[&str] = &["flights:read"];

/// How long a code from the consent page is valid.
const CODE_SECONDS: f64 = 60.0;
/// How long an access token is valid; the app refreshes it after that.
const ACCESS_SECONDS: f64 = 3600.0;
/// A connection ends after this many days without use, like a login session.
const IDLE_DAYS: i32 = 30;

/// The install as an authorization server: its own address (the issuer) and
/// the resources it issues tokens for, e.g. `https://fleet.example.org/mcp`.
#[derive(Clone)]
pub struct OAuth {
    pub issuer: String,
    pub resources: Arc<[String]>,
}

/// What a request's bearer token is worth for a resource and scope.
pub enum Bearer {
    /// A valid access token for them, acting for this user.
    Valid(Actor),
    /// No bearer token.
    Missing,
    /// Unknown or expired, or its user may no longer log in (RFC 6750:
    /// `invalid_token`).
    Invalid,
    /// Valid, but for another resource or without the scope
    /// (`insufficient_scope`).
    Insufficient,
}

/// Checks the request's access token against this resource and scope.
pub async fn bearer(
    db: &PgPool,
    parts: &Parts,
    resource: &str,
    scope: &str,
) -> sqlx::Result<Bearer> {
    let Some(token) = parts
        .headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
    else {
        return Ok(Bearer::Missing);
    };
    let Some(access) = db::use_access(db, &sha256(token)).await? else {
        return Ok(Bearer::Invalid);
    };
    if access.resource != resource || !access.scopes.iter().any(|granted| granted == scope) {
        return Ok(Bearer::Insufficient);
    }
    let peer = parts
        .extensions
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(peer)| *peer);
    let Some(peer) = peer else {
        return Ok(Bearer::Invalid);
    };
    Ok(Bearer::Valid(Actor {
        user_id: access.user_id,
        grants: crate::auth::grants(db, access.user_id).await?,
        // The audit log will name the connection here rather than a login session.
        login_session_id: access.grant_id,
        ip: crate::auth::client_ip(&parts.headers, peer),
        email: Some(access.email),
        impersonated_by: None,
        name: access.name,
    }))
}

/// Removes expired codes, idle connections and apps that were never
/// connected, every hour while the server runs.
pub async fn clean_up_hourly(db: PgPool) {
    let mut hourly = tokio::time::interval(std::time::Duration::from_secs(3600));
    loop {
        hourly.tick().await;
        if let Err(error) = db::clean_up(&db, IDLE_DAYS).await {
            tracing::error!(%error, "cleaning up OAuth data");
        }
    }
}

/// Codes and tokens are stored only as this hash.
fn sha256(token: &str) -> Vec<u8> {
    Sha256::digest(token.as_bytes()).to_vec()
}

/// A random token, 32 bytes in base64url.
fn random_token() -> String {
    openidconnect::CsrfToken::new_random_len(32).into_secret()
}

/// Whether a PKCE verifier matches its S256 challenge.
fn pkce_matches(verifier: &str, challenge: &str) -> bool {
    let digest = Sha256::digest(verifier.as_bytes());
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest) == challenge
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The example from RFC 7636, appendix B.
    #[test]
    fn pkce_follows_the_rfc() {
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        assert!(pkce_matches(
            verifier,
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        ));
        assert!(!pkce_matches(
            verifier,
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cN"
        ));
    }
}
