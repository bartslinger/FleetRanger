use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use super::db::{self, Request};
use crate::{
    auth::Actor,
    error::{ApiError, ApiResult},
    flights, policy,
};

/// Without 0/O and 1/I, which are easily confused when typed. 32 letters, so a
/// random byte maps onto them evenly.
const CODE_ALPHABET: &[u8; 32] = b"23456789ABCDEFGHJKLMNPQRSTUVWXYZ";
const CODE_LENGTH: usize = 8;
/// How long a controller's code can be approved.
const REQUEST_LIFETIME: Duration = Duration::minutes(10);

/// What a controller shows while it waits for a user to approve it.
#[derive(Serialize)]
pub struct Started {
    pub code: String,
    pub expires_at: DateTime<Utc>,
}

/// A controller asks to pair. It sends the SHA-256 of a token it generated,
/// which becomes its token once a user approves. Needs no login.
pub async fn start(
    db: &PgPool,
    serial: &str,
    model: &str,
    token_sha256_hex: &str,
) -> ApiResult<Started> {
    let serial = serial.trim();
    let model = model.trim();
    if serial.is_empty() || serial.len() > 64 || model.is_empty() || model.len() > 100 {
        return Err(ApiError::Invalid(
            "Send the controller's serial and model.".to_owned(),
        ));
    }
    let token_sha256 = parse_sha256(token_sha256_hex)
        .ok_or_else(|| ApiError::Invalid("Send the token's SHA-256 in hex.".to_owned()))?;
    let expires_at = Utc::now() + REQUEST_LIFETIME;
    // A taken code is unlikely (32^8 codes); try a few times.
    for _ in 0..5 {
        let code = random_code();
        if db::add_request(db, &code, &token_sha256, serial, model, expires_at).await? {
            return Ok(Started { code, expires_at });
        }
    }
    Err(ApiError::Internal(anyhow::anyhow!("no free pairing code")))
}

/// A pairing request, for the user about to approve it. Any user may pair a
/// controller; see "Clients" in docs/architecture.md.
pub async fn request(db: &PgPool, _actor: &Actor, code: &str) -> ApiResult<Request> {
    db::request(db, &normalize(code))
        .await?
        .ok_or(ApiError::NotFound)
}

/// Approves a request: the controller is paired and joins the approver's team,
/// or no team when the approver has none.
pub async fn approve(db: &PgPool, actor: &Actor, code: &str) -> ApiResult<()> {
    let team = flights::db::own_team(db, actor.user_id).await?;
    if db::approve(db, &normalize(code), team).await? {
        Ok(())
    } else {
        Err(ApiError::NotFound)
    }
}

/// Unpairs a controller from the web app. Admins only; a controller can also
/// unpair itself.
pub async fn unpair(db: &PgPool, actor: &Actor, id: Uuid) -> ApiResult<()> {
    policy::require_admin(actor)?;
    if db::unpair(db, id).await? {
        Ok(())
    } else {
        Err(ApiError::NotFound)
    }
}

/// A code as typed: any case, with or without the dash or spaces.
fn normalize(code: &str) -> String {
    code.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect()
}

fn random_code() -> String {
    // Version 4 UUIDs come from the operating system's secure random source;
    // bytes 6 and 8 hold the version and variant, so they're skipped.
    let bytes = Uuid::new_v4().into_bytes();
    [0, 1, 2, 3, 4, 5, 7, 9][..CODE_LENGTH]
        .iter()
        .map(|&i| CODE_ALPHABET[usize::from(bytes[i] % 32)] as char)
        .collect()
}

fn parse_sha256(hex: &str) -> Option<Vec<u8>> {
    if hex.len() != 64 {
        return None;
    }
    (0..64)
        .step_by(2)
        .map(|i| u8::from_str_radix(hex.get(i..i + 2)?, 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A code is typed by hand, so dashes, spaces and case don't matter.
    #[test]
    fn codes_are_read_as_typed() {
        assert_eq!(normalize("kdw4-9txr"), "KDW49TXR");
        assert_eq!(normalize(" KDW4 9TXR "), "KDW49TXR");
    }
}
