use sqlx::PgPool;
use uuid::Uuid;

use super::db::{self, Pilot, PilotFields};
use crate::{
    auth::Actor,
    error::{self, ApiError, ApiResult},
    policy,
};

/// What an admin enters for a pilot.
pub struct PilotInput {
    pub name: String,
    pub team_id: Uuid,
    pub account_id: Option<Uuid>,
}

pub async fn list(db: &PgPool, actor: &Actor, archived: bool) -> ApiResult<Vec<Pilot>> {
    policy::require_admin(actor)?;
    Ok(db::list(db, archived).await?)
}

pub async fn get(db: &PgPool, actor: &Actor, id: Uuid) -> ApiResult<Pilot> {
    policy::require_admin(actor)?;
    db::get(db, id).await?.ok_or(ApiError::NotFound)
}

pub async fn create(db: &PgPool, actor: &Actor, input: &PilotInput) -> ApiResult<Pilot> {
    policy::require_admin(actor)?;
    let name = error::name(&input.name)?;
    let fields = checked(db, &name, input).await?;
    let id = db::insert(db, &fields).await.map_err(account_taken)?;
    get(db, actor, id).await
}

pub async fn update(db: &PgPool, actor: &Actor, id: Uuid, input: &PilotInput) -> ApiResult<Pilot> {
    policy::require_admin(actor)?;
    let name = error::name(&input.name)?;
    let fields = checked(db, &name, input).await?;
    if !db::update(db, id, &fields).await.map_err(account_taken)? {
        return Err(ApiError::NotFound);
    }
    get(db, actor, id).await
}

pub async fn set_archived(
    db: &PgPool,
    actor: &Actor,
    id: Uuid,
    archived: bool,
) -> ApiResult<Pilot> {
    policy::require_admin(actor)?;
    if !db::set_archived(db, id, archived).await? {
        return Err(ApiError::NotFound);
    }
    get(db, actor, id).await
}

/// Only a pilot without flights can be deleted; others are archived.
pub async fn delete(db: &PgPool, actor: &Actor, id: Uuid) -> ApiResult<()> {
    let pilot = get(db, actor, id).await?;
    if pilot.flights > 0 || !db::delete(db, id).await? {
        return Err(ApiError::Conflict(
            "This pilot has flights, so it can't be deleted. Archive it instead.".to_owned(),
        ));
    }
    Ok(())
}

/// Each pilot is in exactly one team, which must be active, and the linked user
/// must exist.
async fn checked<'a>(db: &PgPool, name: &'a str, input: &PilotInput) -> ApiResult<PilotFields<'a>> {
    if !db::team_is_active(db, input.team_id).await? {
        return Err(ApiError::Invalid("Choose an active team.".to_owned()));
    }
    if let Some(account_id) = input.account_id
        && !db::account_exists(db, account_id).await?
    {
        return Err(ApiError::Invalid("Choose an existing user.".to_owned()));
    }
    Ok(PilotFields {
        name,
        team_id: input.team_id,
        account_id: input.account_id,
    })
}

fn account_taken(error: sqlx::Error) -> ApiError {
    if error::is_unique_violation(&error) {
        ApiError::Conflict("This user is already linked to another pilot.".to_owned())
    } else {
        error.into()
    }
}
