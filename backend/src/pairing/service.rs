use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use super::db::{self, OpenCode, Paired, PilotChoice};
use super::{Controller, parse_sha256};
use crate::{
    auth::Actor,
    error::{ApiError, ApiResult},
    flights,
    policy::{self, Permission},
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
    /// The group the controller joins; None for the organisation.
    pub group: Option<String>,
    pub expires_at: DateTime<Utc>,
}

/// Creates a single-use code that pairs a controller, valid for `hours`, with
/// `controllers.pair` on the group the controller joins, or with None on the
/// organisation, where it goes in no group.
pub async fn create(
    db: &PgPool,
    actor: &Actor,
    group: Option<Uuid>,
    hours: i64,
) -> ApiResult<Created> {
    if !actor.grants.anywhere(Permission::ControllersPair) {
        return Err(ApiError::Forbidden);
    }
    if !(1..=MAX_HOURS).contains(&hours) {
        return Err(ApiError::Invalid(format!(
            "A code is valid for 1 to {MAX_HOURS} hours."
        )));
    }
    if !actor.grants.may(Permission::ControllersPair, group) {
        return Err(ApiError::Invalid(
            "Choose one of your groups for the controller.".to_owned(),
        ));
    }
    let group = match group {
        Some(id) => Some(
            flights::db::groups_named(db, &[id])
                .await?
                .pop()
                .ok_or_else(|| ApiError::Invalid("Choose an existing group.".to_owned()))?,
        ),
        None => None,
    };
    let expires_at = Utc::now() + Duration::hours(hours);
    // A taken code is unlikely (32^8 codes); try a few times.
    for _ in 0..5 {
        let code = random_code();
        if let Some(id) = db::add_code(
            db,
            &code,
            actor.user_id,
            group.as_ref().map(|g| g.id),
            expires_at,
        )
        .await?
        {
            return Ok(Created {
                id,
                code,
                group: group.map(|g| g.name),
                expires_at,
            });
        }
    }
    Err(ApiError::Internal(anyhow::anyhow!("no free pairing code")))
}

/// The codes the actor may see and revoke that can still be used: their own,
/// and those of the groups where they hold `controllers.manage`.
pub async fn open_codes(db: &PgPool, actor: &Actor) -> ApiResult<Vec<OpenCode>> {
    let scope = actor.grants.scope(Permission::ControllersManage);
    Ok(db::open_codes(db, actor.user_id, &scope).await?)
}

/// Revokes a code, so it can't pair a controller any more.
pub async fn revoke(db: &PgPool, actor: &Actor, id: Uuid) -> ApiResult<()> {
    // Used or expired just now, before the page caught up.
    let gone = || ApiError::Conflict("This code was used or has expired.".to_owned());
    let owner = db::code_owner(db, id).await?.ok_or_else(gone)?;
    if !policy::manage_pairing_code(actor, owner.created_by, owner.group_id) {
        return Err(ApiError::NotFound);
    }
    if db::revoke(db, id).await? {
        Ok(())
    } else {
        Err(gone())
    }
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

/// The pilots a controller offers: every active user who is a pilot in a
/// group, once per group. The controller shows its own team first, and keeps
/// the list for when it's offline.
#[derive(Serialize)]
pub struct PilotList {
    /// The controller's team; None if it has none.
    pub team_id: Option<Uuid>,
    pub pilots: Vec<PilotChoice>,
}

pub async fn pilots(db: &PgPool, controller: &Controller) -> ApiResult<PilotList> {
    Ok(PilotList {
        team_id: controller.group_id,
        pilots: db::pilot_choices(db).await?,
    })
}

/// Where a controller paired with the actor's code can go: the groups where
/// they hold `controllers.pair`, and the organisation itself (no group) when
/// they hold it there, which covers every group too (user's decision,
/// 2026-10-23).
#[derive(Serialize)]
pub struct Destinations {
    pub groups: Vec<flights::db::GroupChoice>,
    pub organisation: bool,
}

pub async fn destinations(db: &PgPool, actor: &Actor) -> ApiResult<Destinations> {
    let pair = actor.grants.scope(Permission::ControllersPair);
    let groups = if pair.all {
        flights::db::group_choices(db).await?
    } else {
        flights::db::groups_named(db, &pair.groups).await?
    };
    Ok(Destinations {
        groups,
        organisation: pair.all,
    })
}

/// Unpairs a controller from the web app, with `controllers.manage` on its
/// group; a controller can also unpair itself.
pub async fn unpair(db: &PgPool, actor: &Actor, id: Uuid) -> ApiResult<()> {
    // Refused before not found, so this doesn't tell whether it exists.
    if !actor.grants.anywhere(Permission::ControllersManage) {
        return Err(ApiError::Forbidden);
    }
    let group = db::controller_group(db, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    policy::require(actor, Permission::ControllersManage, group)?;
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
