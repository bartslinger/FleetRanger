use chrono::{Duration, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use super::db::{self, Approved, Waiting};
use super::{Controller, parse_sha256};
use crate::{
    auth::Actor,
    error::{ApiError, ApiResult},
    flights::{self, db::PilotChoice},
    policy,
};

/// Without 0/O and 1/I, which are easily confused when typed. 32 letters, so a
/// random byte maps onto them evenly.
const CODE_ALPHABET: &[u8; 32] = b"23456789ABCDEFGHJKLMNPQRSTUVWXYZ";
const CODE_LENGTH: usize = 8;
/// How long a request stays valid after the controller last checked on it.
/// While its pairing screen is open, the controller checks every few seconds,
/// so its code stays the same; once it's off, the request soon expires.
const REQUEST_LIFETIME: Duration = Duration::minutes(1);

/// What a controller shows while it waits for a user to approve it.
#[derive(Serialize)]
pub struct Started {
    pub code: String,
}

/// Where a controller's token stands.
pub enum Check {
    Paired,
    /// Still waiting for approval; checking kept the request alive.
    Waiting,
    /// Neither: unpaired, or the request expired. The controller starts over.
    Unknown,
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
            return Ok(Started { code });
        }
    }
    Err(ApiError::Internal(anyhow::anyhow!("no free pairing code")))
}

/// What a controller learns when it checks its token, which it does every few
/// seconds while it waits for approval.
pub async fn check(db: &PgPool, token_sha256: &[u8]) -> ApiResult<Check> {
    if db::by_token(db, token_sha256).await?.is_some() {
        return Ok(Check::Paired);
    }
    let seconds = REQUEST_LIFETIME.as_seconds_f64();
    Ok(if db::keep_waiting(db, token_sha256, seconds).await? {
        Check::Waiting
    } else {
        Check::Unknown
    })
}

/// The pilots a controller offers: every active user in a team. The controller
/// shows its own team first, and keeps the list for when it's offline.
#[derive(Serialize)]
pub struct PilotList {
    /// The controller's team; None if it has none.
    pub team_id: Option<Uuid>,
    pub pilots: Vec<PilotChoice>,
}

pub async fn pilots(db: &PgPool, controller: &Controller) -> ApiResult<PilotList> {
    Ok(PilotList {
        team_id: controller.team_id,
        pilots: flights::db::pilot_choices(db, None).await?,
    })
}

/// The controllers waiting to be paired, so the web app can tell users. Any
/// user may pair a controller, so any user sees them; approving needs the code.
pub async fn waiting(db: &PgPool, _actor: &Actor) -> ApiResult<Vec<Waiting>> {
    Ok(db::waiting(db).await?)
}

/// The teams a controller the actor approves can join: theirs. One of them is
/// chosen when there are several.
pub async fn teams(db: &PgPool, actor: &Actor) -> ApiResult<Vec<flights::db::TeamChoice>> {
    Ok(flights::db::teams_of_user(db, actor.user_id).await?)
}

/// Approves a request: the controller is paired and joins the approver's team,
/// the one they choose if they're in several, or no team when they have none.
/// Any user may pair a controller; see "Clients" in docs/architecture.md.
pub async fn approve(
    db: &PgPool,
    actor: &Actor,
    code: &str,
    team: Option<Uuid>,
) -> ApiResult<Approved> {
    let own_teams = flights::db::own_teams(db, actor.user_id).await?;
    let team = match (team, own_teams.as_slice()) {
        (_, []) => None,
        (Some(team), teams) if teams.contains(&team) => Some(team),
        (Some(_), _) => return Err(ApiError::Invalid("Choose one of your teams.".to_owned())),
        (None, [team]) => Some(*team),
        (None, _) => {
            return Err(ApiError::Invalid(
                "Choose the team the controller joins.".to_owned(),
            ));
        }
    };
    db::approve(db, &normalize(code), team)
        .await?
        .ok_or_else(|| {
            ApiError::Invalid(
                "No controller is waiting with this code. Check the code on the controller’s \
                 screen."
                    .to_owned(),
            )
        })
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
