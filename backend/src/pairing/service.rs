use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use super::db::{self, OpenCode, Paired};
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
/// How long a code may stay valid, chosen by whoever creates it. Longer gives
/// more time for guessing.
const MAX_HOURS: i64 = 7 * 24;

/// A code just created, to give to the pilot.
#[derive(Serialize)]
pub struct Created {
    pub id: Uuid,
    pub code: String,
    pub team: Option<String>,
    pub expires_at: DateTime<Utc>,
}

/// Creates a single-use code that pairs a controller, valid for `hours`. Any
/// user may; the controller joins one of their teams, the one they choose if
/// they're in several, or none when they have none.
pub async fn create(
    db: &PgPool,
    actor: &Actor,
    team: Option<Uuid>,
    hours: i64,
) -> ApiResult<Created> {
    if !(1..=MAX_HOURS).contains(&hours) {
        return Err(ApiError::Invalid(format!(
            "A code is valid for 1 to {MAX_HOURS} hours."
        )));
    }
    let teams = flights::db::teams_of_user(db, actor.user_id).await?;
    let team = match (team, teams.as_slice()) {
        (_, []) => None,
        (Some(id), teams) => Some(
            teams
                .iter()
                .find(|team| team.id == id)
                .ok_or_else(|| ApiError::Invalid("Choose one of your teams.".to_owned()))?,
        ),
        (None, [team]) => Some(team),
        (None, _) => {
            return Err(ApiError::Invalid(
                "Choose the team the controller joins.".to_owned(),
            ));
        }
    };
    let expires_at = Utc::now() + Duration::hours(hours);
    // A taken code is unlikely (32^8 codes); try a few times.
    for _ in 0..5 {
        let code = random_code();
        if let Some(id) =
            db::add_code(db, &code, actor.user_id, team.map(|t| t.id), expires_at).await?
        {
            return Ok(Created {
                id,
                code,
                team: team.map(|t| t.name.clone()),
                expires_at,
            });
        }
    }
    Err(ApiError::Internal(anyhow::anyhow!("no free pairing code")))
}

/// The codes the actor may see and revoke that can still be used.
pub async fn open_codes(db: &PgPool, actor: &Actor) -> ApiResult<Vec<OpenCode>> {
    let created_by = (!policy::all_pairing_codes(actor)).then_some(actor.user_id);
    Ok(db::open_codes(db, created_by).await?)
}

/// Revokes a code, so it can't pair a controller any more.
pub async fn revoke(db: &PgPool, actor: &Actor, id: Uuid) -> ApiResult<()> {
    let created_by = db::code_creator(db, id).await?.ok_or(ApiError::NotFound)?;
    if !policy::revoke_pairing_code(actor, created_by) {
        return Err(ApiError::NotFound);
    }
    db::revoke(db, id).await?;
    Ok(())
}

/// A controller pairs with a code it was given. It sends the SHA-256 of a
/// token it generated, which becomes its token. Needs no login: the code is
/// the proof.
pub async fn pair(
    db: &PgPool,
    code: &str,
    serial: &str,
    token_sha256_hex: &str,
) -> ApiResult<Paired> {
    let serial = serial.trim();
    if serial.is_empty() || serial.len() > 64 {
        return Err(ApiError::Invalid(
            "Send the controller's serial.".to_owned(),
        ));
    }
    let token_sha256 = parse_sha256(token_sha256_hex)
        .ok_or_else(|| ApiError::Invalid("Send the token's SHA-256 in hex.".to_owned()))?;
    db::use_code(db, &normalize(code), serial, &token_sha256)
        .await?
        .ok_or_else(|| {
            ApiError::Invalid(
                "This code doesn’t pair a controller: it may have been used, revoked or have \
                 expired. Ask for a new one."
                    .to_owned(),
            )
        })
}

/// Whether a controller's token is paired.
pub async fn paired(db: &PgPool, token_sha256: &[u8]) -> ApiResult<bool> {
    Ok(db::by_token(db, token_sha256).await?.is_some())
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

/// The teams a controller paired with the actor's code can join: theirs. One
/// of them is chosen when there are several.
pub async fn teams(db: &PgPool, actor: &Actor) -> ApiResult<Vec<flights::db::TeamChoice>> {
    Ok(flights::db::teams_of_user(db, actor.user_id).await?)
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
