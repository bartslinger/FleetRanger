use sqlx::PgPool;
use uuid::Uuid;

use super::db::{self, Equipment, EquipmentFields, Kind};
use crate::{
    auth::Actor,
    error::{ApiError, ApiResult},
    policy,
};

/// What an admin enters for a drone or controller; all optional.
pub struct EquipmentInput {
    pub name: Option<String>,
    pub team_id: Option<Uuid>,
    pub holder_pilot_id: Option<Uuid>,
}

pub async fn list(db: &PgPool, actor: &Actor, kind: Kind) -> ApiResult<Vec<Equipment>> {
    policy::require_admin(actor)?;
    Ok(db::list(db, kind).await?)
}

pub async fn get(db: &PgPool, actor: &Actor, kind: Kind, id: Uuid) -> ApiResult<Equipment> {
    policy::require_admin(actor)?;
    db::get(db, kind, id).await?.ok_or(ApiError::NotFound)
}

/// The team decides the team of new flights without a pilot, so it must be
/// active; so must the holder.
pub async fn update(
    db: &PgPool,
    actor: &Actor,
    kind: Kind,
    id: Uuid,
    input: &EquipmentInput,
) -> ApiResult<Equipment> {
    policy::require_admin(actor)?;
    if let Some(team_id) = input.team_id
        && !db::team_is_active(db, team_id).await?
    {
        return Err(ApiError::Invalid("Choose an active team.".to_owned()));
    }
    if let Some(pilot_id) = input.holder_pilot_id
        && !db::pilot_is_active(db, pilot_id).await?
    {
        return Err(ApiError::Invalid("Choose an active pilot.".to_owned()));
    }
    let name = input
        .name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty());
    let fields = EquipmentFields {
        name,
        team_id: input.team_id,
        holder_pilot_id: input.holder_pilot_id,
    };
    if !db::update(db, kind, id, &fields).await? {
        return Err(ApiError::NotFound);
    }
    get(db, actor, kind, id).await
}
