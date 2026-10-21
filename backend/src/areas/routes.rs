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
        .route("/api/areas", get(list).post(create))
        .route("/api/areas/{id}", put(update).delete(delete))
}

#[derive(Deserialize)]
struct NewArea {
    name: String,
    /// GeoJSON Polygon, as drawn.
    boundary: Value,
}

/// A new name, a new shape, or both.
#[derive(Deserialize)]
struct AreaChange {
    name: Option<String>,
    boundary: Option<Value>,
}

async fn list(actor: Actor, State(db): State<PgPool>) -> ApiResult<Json<Vec<Area>>> {
    Ok(Json(service::list(&db, &actor).await?))
}

async fn create(
    actor: Actor,
    State(db): State<PgPool>,
    Json(input): Json<NewArea>,
) -> ApiResult<Json<Saved>> {
    Ok(Json(
        service::save(&db, &actor, None, Some(&input.name), &input.boundary).await?,
    ))
}

/// A new shape is clipped and may split the area, as a new one is; a new
/// name alone changes no figures.
async fn update(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(change): Json<AreaChange>,
) -> ApiResult<Json<Option<Saved>>> {
    let name = change.name.as_deref();
    match (&change.boundary, name) {
        (Some(boundary), _) => Ok(Json(Some(
            service::save(&db, &actor, Some(id), name, boundary).await?,
        ))),
        (None, Some(name)) => {
            service::rename(&db, &actor, id, name).await?;
            Ok(Json(None))
        }
        (None, None) => Err(ApiError::Invalid("Nothing to change.".to_owned())),
    }
}

async fn delete(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    service::delete(&db, &actor, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
