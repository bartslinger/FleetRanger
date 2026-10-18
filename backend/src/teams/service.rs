use sqlx::PgPool;
use uuid::Uuid;

use super::db::{self, Team};
use crate::{
    auth::Actor,
    error::{self, ApiError, ApiResult},
    policy,
};

pub async fn list(db: &PgPool, actor: &Actor, archived: bool) -> ApiResult<Vec<Team>> {
    policy::require_admin(actor)?;
    Ok(db::list(db, archived).await?)
}

pub async fn get(db: &PgPool, actor: &Actor, id: Uuid) -> ApiResult<Team> {
    policy::require_admin(actor)?;
    db::get(db, id).await?.ok_or(ApiError::NotFound)
}

pub async fn create(db: &PgPool, actor: &Actor, name: &str) -> ApiResult<Team> {
    policy::require_admin(actor)?;
    let name = error::name(name)?;
    let id = db::insert(db, &name).await.map_err(name_taken)?;
    get(db, actor, id).await
}

pub async fn rename(db: &PgPool, actor: &Actor, id: Uuid, name: &str) -> ApiResult<Team> {
    policy::require_admin(actor)?;
    let name = error::name(name)?;
    if !db::rename(db, id, &name).await.map_err(name_taken)? {
        return Err(ApiError::NotFound);
    }
    get(db, actor, id).await
}

/// A team with active members can't be archived, or their new flights would go
/// to an archived team.
pub async fn set_archived(db: &PgPool, actor: &Actor, id: Uuid, archived: bool) -> ApiResult<Team> {
    let team = get(db, actor, id).await?;
    if archived && team.members > 0 {
        return Err(ApiError::Conflict(
            "This team still has members. Move them to another team or archive them first."
                .to_owned(),
        ));
    }
    db::set_archived(db, id, archived).await?;
    get(db, actor, id).await
}

fn name_taken(error: sqlx::Error) -> ApiError {
    if error::is_unique_violation(&error) {
        ApiError::Conflict("A team with this name already exists.".to_owned())
    } else {
        error.into()
    }
}
