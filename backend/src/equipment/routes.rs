//! Drone and remote controller API. Who sees what follows `equipment.read` and
//! the flights; who changes what is the policy's decision
//! (`change_equipment`).

use crate::db::Db;
use axum::{
    Json, Router,
    extract::{FromRef, Path},
    routing::get,
};
use serde::Deserialize;
use uuid::Uuid;

use super::{
    db::{Equipment, Kind},
    service::{self, EquipmentInput},
};
use crate::{auth::Actor, error::ApiResult};

pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    crate::db::System: FromRef<S>,
    crate::db::Tenants: FromRef<S>,
{
    Router::new()
        .route("/api/aircraft", get(list_aircraft))
        .route(
            "/api/aircraft/{id}",
            get(show_aircraft).put(update_aircraft),
        )
        .route("/api/remote-controllers", get(list_controllers))
        .route(
            "/api/remote-controllers/{id}",
            get(show_controller).put(update_controller),
        )
}

#[derive(Deserialize)]
struct Body {
    name: Option<String>,
    group_id: Uuid,
}

impl From<Body> for EquipmentInput {
    fn from(body: Body) -> Self {
        Self {
            name: body.name,
            group_id: body.group_id,
        }
    }
}

async fn list_aircraft(actor: Actor, mut db: Db) -> ApiResult<Json<Vec<Equipment>>> {
    Ok(Json(service::list(&mut db, &actor, Kind::Aircraft).await?))
}

async fn show_aircraft(
    actor: Actor,
    mut db: Db,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Equipment>> {
    Ok(Json(
        service::get(&mut db, &actor, Kind::Aircraft, id).await?,
    ))
}

async fn update_aircraft(
    actor: Actor,
    mut db: Db,
    Path(id): Path<Uuid>,
    Json(body): Json<Body>,
) -> ApiResult<Json<Equipment>> {
    let input = body.into();
    Ok(Json(
        service::update(&mut db, &actor, Kind::Aircraft, id, &input).await?,
    ))
}

async fn list_controllers(actor: Actor, mut db: Db) -> ApiResult<Json<Vec<Equipment>>> {
    Ok(Json(
        service::list(&mut db, &actor, Kind::RemoteController).await?,
    ))
}

async fn show_controller(
    actor: Actor,
    mut db: Db,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Equipment>> {
    Ok(Json(
        service::get(&mut db, &actor, Kind::RemoteController, id).await?,
    ))
}

async fn update_controller(
    actor: Actor,
    mut db: Db,
    Path(id): Path<Uuid>,
    Json(body): Json<Body>,
) -> ApiResult<Json<Equipment>> {
    let input = body.into();
    Ok(Json(
        service::update(&mut db, &actor, Kind::RemoteController, id, &input).await?,
    ))
}
