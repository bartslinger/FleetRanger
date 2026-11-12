use sqlx::PgConnection;
use uuid::Uuid;

use super::db::{self, Equipment, Kind};
use crate::{
    auth::Actor,
    error::{ApiError, ApiResult},
    policy::{self, Permission},
};

/// What is entered for a drone or controller.
pub struct EquipmentInput {
    pub name: Option<String>,
    pub group_id: Uuid,
}

/// The equipment the actor sees: with `equipment.read` on its group, or
/// through one of its flights.
pub async fn list(db: &mut PgConnection, actor: &Actor, kind: Kind) -> ApiResult<Vec<Equipment>> {
    let read = actor.grants.scope(Permission::EquipmentRead);
    Ok(db::list(db, kind, &read, &policy::flight_scope(actor), None).await?)
}

/// One the actor doesn't see is not found, so its existence isn't revealed.
pub async fn get(
    db: &mut PgConnection,
    actor: &Actor,
    kind: Kind,
    id: Uuid,
) -> ApiResult<Equipment> {
    let read = actor.grants.scope(Permission::EquipmentRead);
    db::list(db, kind, &read, &policy::flight_scope(actor), Some(id))
        .await?
        .pop()
        .ok_or(ApiError::NotFound)
}

/// Renames and moves a drone or controller to an existing group, or to the
/// organisation.
pub async fn update(
    db: &mut PgConnection,
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
        input.group_id,
        name != item.name.as_deref(),
    )?;
    if input.group_id != item.group_id && !db::group_exists(db, input.group_id).await? {
        return Err(ApiError::Invalid("Choose an existing group.".to_owned()));
    }
    if !db::update(db, kind, id, name, input.group_id).await? {
        return Err(ApiError::NotFound);
    }
    get(db, actor, kind, id).await
}
