//! Team API, for admins.

use axum::{
    Json, Router,
    extract::{FromRef, Path, Query, State},
    routing::{get, post},
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use super::{db::Team, service};
use crate::{auth::Actor, error::ApiResult};

pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    PgPool: FromRef<S>,
{
    Router::new()
        .route("/api/teams", get(list).post(create))
        .route("/api/teams/{id}", get(show).put(rename))
        .route("/api/teams/{id}/archive", post(archive))
        .route("/api/teams/{id}/restore", post(restore))
}

#[derive(Deserialize)]
struct ListQuery {
    /// Include archived teams.
    #[serde(default)]
    archived: bool,
}

#[derive(Deserialize)]
struct TeamInput {
    name: String,
}

async fn list(
    actor: Actor,
    State(db): State<PgPool>,
    Query(query): Query<ListQuery>,
) -> ApiResult<Json<Vec<Team>>> {
    Ok(Json(service::list(&db, &actor, query.archived).await?))
}

async fn show(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Team>> {
    Ok(Json(service::get(&db, &actor, id).await?))
}

async fn create(
    actor: Actor,
    State(db): State<PgPool>,
    Json(input): Json<TeamInput>,
) -> ApiResult<Json<Team>> {
    Ok(Json(service::create(&db, &actor, &input.name).await?))
}

async fn rename(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(input): Json<TeamInput>,
) -> ApiResult<Json<Team>> {
    Ok(Json(service::rename(&db, &actor, id, &input.name).await?))
}

async fn archive(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Team>> {
    Ok(Json(service::set_archived(&db, &actor, id, true).await?))
}

async fn restore(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Team>> {
    Ok(Json(service::set_archived(&db, &actor, id, false).await?))
}
