//! Battery API. Who sees what follows the flights (`policy::flight_scope`);
//! naming one is for admins.

use axum::{
    Json, Router,
    extract::{FromRef, Path, State},
    routing::get,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use super::{
    db::Battery,
    service::{self, BatteryDetail},
};
use crate::{auth::Actor, error::ApiResult};

pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    PgPool: FromRef<S>,
{
    Router::new()
        .route("/api/batteries", get(list))
        .route("/api/batteries/{id}", get(show).put(update))
}

#[derive(Deserialize)]
struct Body {
    name: Option<String>,
}

async fn list(actor: Actor, State(db): State<PgPool>) -> ApiResult<Json<Vec<Battery>>> {
    Ok(Json(service::list(&db, &actor).await?))
}

async fn show(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<BatteryDetail>> {
    Ok(Json(service::get(&db, &actor, id).await?))
}

async fn update(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(body): Json<Body>,
) -> ApiResult<Json<BatteryDetail>> {
    Ok(Json(
        service::rename(&db, &actor, id, body.name.as_deref()).await?,
    ))
}
