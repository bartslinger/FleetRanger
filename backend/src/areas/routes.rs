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
    service::{self, Drawing, Saved},
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

/// A new name or a new shape, one at a time.
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
    let drawing = Drawing::New(&input.name);
    Ok(Json(
        service::save(&db, &actor, drawing, &input.boundary).await?,
    ))
}

/// A new shape is clipped and may split the area, as a new one is; a new
/// name changes no figures.
async fn update(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(change): Json<AreaChange>,
) -> ApiResult<Json<Option<Saved>>> {
    match (change.boundary, change.name) {
        (Some(boundary), None) => Ok(Json(Some(
            service::save(&db, &actor, Drawing::Reshape(id), &boundary).await?,
        ))),
        (None, Some(name)) => {
            service::rename(&db, &actor, id, &name).await?;
            Ok(Json(None))
        }
        _ => Err(ApiError::Invalid(
            "Change either the name or the shape.".to_owned(),
        )),
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
