use sqlx::PgPool;
use uuid::Uuid;

use super::db::{self, Equipment, Kind};
use crate::{
    auth::Actor,
    error::{ApiError, ApiResult},
    policy::{self, Permission},
};

/// What is entered for a drone or controller; all optional.
pub struct EquipmentInput {
    pub name: Option<String>,
    pub team_id: Option<Uuid>,
}

/// The equipment the actor sees: with `equipment.read` on its group, or
/// through one of its flights.
pub async fn list(db: &PgPool, actor: &Actor, kind: Kind) -> ApiResult<Vec<Equipment>> {
    let read = actor.grants.scope(Permission::EquipmentRead);
    Ok(db::list(db, kind, &read, &policy::flight_scope(actor), None).await?)
}

/// One the actor doesn't see is not found, so its existence isn't revealed.
pub async fn get(db: &PgPool, actor: &Actor, kind: Kind, id: Uuid) -> ApiResult<Equipment> {
    let read = actor.grants.scope(Permission::EquipmentRead);
    db::list(db, kind, &read, &policy::flight_scope(actor), Some(id))
        .await?
        .pop()
        .ok_or(ApiError::NotFound)
}

/// Renames and moves a drone or controller; the team must be an existing
/// group.
pub async fn update(
    db: &PgPool,
    actor: &Actor,
    kind: Kind,
    id: Uuid,
    input: &EquipmentInput,
) -> ApiResult<Equipment> {
    let item = get(db, actor, kind, id).await?;
    let name = input
        .name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty());
    policy::change_equipment(
        &actor.grants,
        item.group_id,
        input.team_id,
        name != item.name.as_deref(),
    )?;
    if let Some(group_id) = input.team_id
        && !db::group_exists(db, group_id).await?
    {
        return Err(ApiError::Invalid("Choose an active team.".to_owned()));
    }
    if !db::update(db, kind, id, name, input.team_id).await? {
        return Err(ApiError::NotFound);
    }
    get(db, actor, kind, id).await
}
