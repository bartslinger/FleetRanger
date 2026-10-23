use sqlx::PgPool;
use uuid::Uuid;

use super::db::{self, Equipment, EquipmentFields, Kind};
use crate::{
    auth::Actor,
    error::{ApiError, ApiResult},
    policy::{self, Permission},
};

/// What is entered for a drone or controller; all optional.
pub struct EquipmentInput {
    pub name: Option<String>,
    pub team_id: Option<Uuid>,
    pub holder_user_id: Option<Uuid>,
}

/// Whether the actor sees this item: `equipment.read` on its group.
fn sees(actor: &Actor, item: &Equipment) -> bool {
    actor.grants.may(Permission::EquipmentRead, item.group_id)
}

/// The equipment the actor may see; the lists are short, so the policy
/// filters them here rather than in the query.
pub async fn list(db: &PgPool, actor: &Actor, kind: Kind) -> ApiResult<Vec<Equipment>> {
    let mut items = db::list(db, kind).await?;
    items.retain(|item| sees(actor, item));
    Ok(items)
}

/// One the actor may not see is not found, so its existence isn't revealed.
pub async fn get(db: &PgPool, actor: &Actor, kind: Kind, id: Uuid) -> ApiResult<Equipment> {
    db::get(db, kind, id)
        .await?
        .filter(|item| sees(actor, item))
        .ok_or(ApiError::NotFound)
}

/// The team decides the team of new flights without a pilot, so it must be
/// an existing group; the holder must be active.
pub async fn update(
    db: &PgPool,
    actor: &Actor,
    kind: Kind,
    id: Uuid,
    input: &EquipmentInput,
) -> ApiResult<Equipment> {
    let item = get(db, actor, kind, id).await?;
    let holder_groups = match input.holder_user_id {
        Some(user) => Some(db::holder_groups(db, user).await?),
        None => None,
    };
    policy::change_equipment(
        &actor.grants,
        item.group_id,
        input.team_id,
        holder_groups.as_deref(),
    )?;
    if let Some(group_id) = input.team_id
        && !db::group_exists(db, group_id).await?
    {
        return Err(ApiError::Invalid("Choose an active team.".to_owned()));
    }
    if let Some(user_id) = input.holder_user_id
        && !db::user_is_active(db, user_id).await?
    {
        return Err(ApiError::Invalid("Choose an active user.".to_owned()));
    }
    let name = input
        .name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty());
    let fields = EquipmentFields {
        name,
        team_id: input.team_id,
        holder_user_id: input.holder_user_id,
    };
    if !db::update(db, kind, id, &fields).await? {
        return Err(ApiError::NotFound);
    }
    get(db, actor, kind, id).await
}
