//! Battery API. Who sees what follows the flights (`policy::flight_scope`);
//! naming one is for admins.

use crate::db::Db;
use axum::{
    Json, Router,
    extract::{FromRef, Path},
    routing::get,
};
use serde::Deserialize;
use uuid::Uuid;

use super::{
    db::Battery,
    service::{self, BatteryDetail},
};
use crate::{auth::Actor, error::ApiResult};

pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    crate::db::System: FromRef<S>,
    crate::db::Tenants: FromRef<S>,
{
    Router::new()
        .route("/api/batteries", get(list))
        .route("/api/batteries/{id}", get(show).put(update))
}

#[derive(Deserialize)]
struct Body {
    name: Option<String>,
}

async fn list(actor: Actor, mut db: Db) -> ApiResult<Json<Vec<Battery>>> {
    Ok(Json(service::list(&mut db, &actor).await?))
}

async fn show(actor: Actor, mut db: Db, Path(id): Path<Uuid>) -> ApiResult<Json<BatteryDetail>> {
    Ok(Json(service::get(&mut db, &actor, id).await?))
}

async fn update(
    actor: Actor,
    mut db: Db,
    Path(id): Path<Uuid>,
    Json(body): Json<Body>,
) -> ApiResult<Json<BatteryDetail>> {
    Ok(Json(
        service::rename(&mut db, &actor, id, body.name.as_deref()).await?,
    ))
}
