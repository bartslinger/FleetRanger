//! Group API, for whoever manages groups.

use crate::db::Db;
use axum::{
    Json, Router,
    extract::{FromRef, Path, Query},
    http::StatusCode,
    routing::{get, put},
};
use serde::Deserialize;
use uuid::Uuid;

use super::{
    db::{Group, Member},
    service::{self, GroupDetail},
};
use crate::{
    auth::Actor,
    error::{ApiError, ApiResult},
};

pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    crate::db::System: FromRef<S>,
    crate::db::Tenants: FromRef<S>,
{
    Router::new()
        .route("/api/groups", get(list).post(create))
        .route("/api/groups/{id}", get(show).put(rename).delete(remove))
        .route(
            "/api/groups/{id}/members/{user_id}",
            put(set_roles).delete(remove_member),
        )
}

#[derive(Deserialize)]
struct GroupInput {
    name: String,
}

#[derive(Deserialize)]
struct RolesInput {
    roles: Vec<String>,
    /// Apps' roles, by id.
    #[serde(default)]
    app_roles: Vec<Uuid>,
}

#[derive(Deserialize)]
struct DeleteQuery {
    /// Move everything in the group first: a group's id, or "organisation".
    move_to: Option<String>,
}

async fn list(actor: Actor, mut db: Db) -> ApiResult<Json<Vec<Group>>> {
    Ok(Json(service::list(&mut db, &actor).await?))
}

async fn show(actor: Actor, mut db: Db, Path(id): Path<Uuid>) -> ApiResult<Json<GroupDetail>> {
    Ok(Json(service::get(&mut db, &actor, id).await?))
}

async fn create(
    actor: Actor,
    mut db: Db,
    Json(input): Json<GroupInput>,
) -> ApiResult<Json<GroupDetail>> {
    Ok(Json(service::create(&mut db, &actor, &input.name).await?))
}

async fn rename(
    actor: Actor,
    mut db: Db,
    Path(id): Path<Uuid>,
    Json(input): Json<GroupInput>,
) -> ApiResult<Json<GroupDetail>> {
    Ok(Json(
        service::rename(&mut db, &actor, id, &input.name).await?,
    ))
}

async fn remove(
    actor: Actor,
    mut db: Db,
    Path(id): Path<Uuid>,
    Query(query): Query<DeleteQuery>,
) -> ApiResult<StatusCode> {
    let move_to = match query.move_to.as_deref() {
        None => None,
        Some(value) => Some(
            value
                .parse()
                .map_err(|_| ApiError::Invalid("move_to must be a group id".to_owned()))?,
        ),
    };
    service::delete(&mut db, &actor, id, move_to).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn set_roles(
    actor: Actor,
    mut db: Db,
    Path((id, user_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<RolesInput>,
) -> ApiResult<Json<Vec<Member>>> {
    Ok(Json(
        service::set_roles(&mut db, &actor, id, user_id, &input.roles, &input.app_roles).await?,
    ))
}

async fn remove_member(
    actor: Actor,
    mut db: Db,
    Path((id, user_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<Vec<Member>>> {
    Ok(Json(
        service::remove_member(&mut db, &actor, id, user_id).await?,
    ))
}
