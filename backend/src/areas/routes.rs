//! Area API: everyone lists them, admins change them.

use axum::{
    Json, Router,
    extract::{FromRef, Path, State},
    http::StatusCode,
    routing::{get, put},
};
use serde::Deserialize;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use super::{
    db::Area,
    service::{self, Saved},
};
use crate::{auth::Actor, error::ApiResult};

pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    PgPool: FromRef<S>,
{
    Router::new()
        .route("/api/areas", get(list).post(create))
        .route("/api/areas/{id}", put(update).delete(delete))
}

#[derive(Deserialize)]
struct AreaInput {
    name: String,
    /// GeoJSON Polygon or MultiPolygon, as drawn.
    boundary: Value,
}

async fn list(actor: Actor, State(db): State<PgPool>) -> ApiResult<Json<Vec<Area>>> {
    Ok(Json(service::list(&db, &actor).await?))
}

async fn create(
    actor: Actor,
    State(db): State<PgPool>,
    Json(input): Json<AreaInput>,
) -> ApiResult<Json<Saved>> {
    Ok(Json(
        service::save(&db, &actor, None, &input.name, &input.boundary).await?,
    ))
}

async fn update(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(input): Json<AreaInput>,
) -> ApiResult<Json<Saved>> {
    Ok(Json(
        service::save(&db, &actor, Some(id), &input.name, &input.boundary).await?,
    ))
}

async fn delete(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    service::delete(&db, &actor, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
