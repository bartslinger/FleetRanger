//! Group API, for whoever manages groups.

use axum::{
    Json, Router,
    extract::{FromRef, Path, Query, State},
    http::StatusCode,
    routing::{get, put},
};
use serde::Deserialize;
use sqlx::PgPool;
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
    PgPool: FromRef<S>,
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

async fn list(actor: Actor, State(db): State<PgPool>) -> ApiResult<Json<Vec<Group>>> {
    Ok(Json(service::list(&db, &actor).await?))
}

async fn show(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<GroupDetail>> {
    Ok(Json(service::get(&db, &actor, id).await?))
}

async fn create(
    actor: Actor,
    State(db): State<PgPool>,
    Json(input): Json<GroupInput>,
) -> ApiResult<Json<GroupDetail>> {
    Ok(Json(service::create(&db, &actor, &input.name).await?))
}

async fn rename(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(input): Json<GroupInput>,
) -> ApiResult<Json<GroupDetail>> {
    Ok(Json(service::rename(&db, &actor, id, &input.name).await?))
}

async fn remove(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
    Query(query): Query<DeleteQuery>,
) -> ApiResult<StatusCode> {
    let move_to = match query.move_to.as_deref() {
        None => None,
        Some("organisation") => Some(None),
        Some(value) => Some(Some(value.parse().map_err(|_| {
            ApiError::Invalid("move_to must be a group id or organisation".to_owned())
        })?)),
    };
    service::delete(&db, &actor, id, move_to).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn set_roles(
    actor: Actor,
    State(db): State<PgPool>,
    Path((id, user_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<RolesInput>,
) -> ApiResult<Json<Vec<Member>>> {
    Ok(Json(
        service::set_roles(&db, &actor, id, user_id, &input.roles, &input.app_roles).await?,
    ))
}

async fn remove_member(
    actor: Actor,
    State(db): State<PgPool>,
    Path((id, user_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<Vec<Member>>> {
    Ok(Json(
        service::remove_member(&db, &actor, id, user_id).await?,
    ))
}
