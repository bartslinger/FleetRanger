use sqlx::PgPool;
use uuid::Uuid;

use super::db::{self, Equipment, EquipmentFields, Kind};
use crate::{
    auth::Actor,
    error::{ApiError, ApiResult},
    policy,
};

/// What is entered for a drone or controller; all optional.
pub struct EquipmentInput {
    pub name: Option<String>,
    pub team_id: Option<Uuid>,
    pub holder_user_id: Option<Uuid>,
}

/// The equipment the actor may see; the lists are short, so the policy
/// filters them here rather than in the query.
pub async fn list(db: &PgPool, actor: &Actor, kind: Kind) -> ApiResult<Vec<Equipment>> {
    let own_teams = own_teams(db, actor).await?;
    let mut items = db::list(db, kind).await?;
    items.retain(|item| policy::see_equipment(actor.role, &own_teams, item.team_id));
    Ok(items)
}

/// One the actor may not see is not found, so its existence isn't revealed.
pub async fn get(db: &PgPool, actor: &Actor, kind: Kind, id: Uuid) -> ApiResult<Equipment> {
    let own_teams = own_teams(db, actor).await?;
    db::get(db, kind, id)
        .await?
        .filter(|item| policy::see_equipment(actor.role, &own_teams, item.team_id))
        .ok_or(ApiError::NotFound)
}

async fn own_teams(db: &PgPool, actor: &Actor) -> sqlx::Result<Vec<Uuid>> {
    crate::flights::db::own_teams(db, actor.user_id).await
}

/// The team decides the team of new flights without a pilot, so it must be
/// active; so must the holder. Members change only the name and holder.
pub async fn update(
    db: &PgPool,
    actor: &Actor,
    kind: Kind,
    id: Uuid,
    input: &EquipmentInput,
) -> ApiResult<Equipment> {
    let item = get(db, actor, kind, id).await?;
    let holder_teams = match input.holder_user_id {
        Some(user) => Some(crate::flights::db::pilot_groups(db, user).await?),
        None => None,
    };
    let own_teams = own_teams(db, actor).await?;
    policy::change_equipment(
        actor.role,
        &own_teams,
        item.team_id,
        input.team_id,
        holder_teams.as_deref(),
    )?;
    if let Some(team_id) = input.team_id
        && !db::team_is_active(db, team_id).await?
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
