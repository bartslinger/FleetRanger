//! Pilot API, for admins.

use axum::{
    Json, Router,
    extract::{FromRef, Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use super::{
    db::Pilot,
    service::{self, PilotInput},
};
use crate::{auth::Actor, error::ApiResult};

pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    PgPool: FromRef<S>,
{
    Router::new()
        .route("/api/pilots", get(list).post(create))
        .route("/api/pilots/{id}", get(show).put(update).delete(remove))
        .route("/api/pilots/{id}/archive", post(archive))
        .route("/api/pilots/{id}/restore", post(restore))
}

#[derive(Deserialize)]
struct ListQuery {
    /// Include archived pilots.
    #[serde(default)]
    archived: bool,
}

#[derive(Deserialize)]
struct Body {
    name: String,
    team_id: Uuid,
    account_id: Option<Uuid>,
}

impl From<Body> for PilotInput {
    fn from(body: Body) -> Self {
        Self {
            name: body.name,
            team_id: body.team_id,
            account_id: body.account_id,
        }
    }
}

async fn list(
    actor: Actor,
    State(db): State<PgPool>,
    Query(query): Query<ListQuery>,
) -> ApiResult<Json<Vec<Pilot>>> {
    Ok(Json(service::list(&db, &actor, query.archived).await?))
}

async fn show(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Pilot>> {
    Ok(Json(service::get(&db, &actor, id).await?))
}

async fn create(
    actor: Actor,
    State(db): State<PgPool>,
    Json(body): Json<Body>,
) -> ApiResult<Json<Pilot>> {
    Ok(Json(service::create(&db, &actor, &body.into()).await?))
}

async fn update(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(body): Json<Body>,
) -> ApiResult<Json<Pilot>> {
    Ok(Json(service::update(&db, &actor, id, &body.into()).await?))
}

async fn remove(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    service::delete(&db, &actor, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn archive(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Pilot>> {
    Ok(Json(service::set_archived(&db, &actor, id, true).await?))
}

async fn restore(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Pilot>> {
    Ok(Json(service::set_archived(&db, &actor, id, false).await?))
}
