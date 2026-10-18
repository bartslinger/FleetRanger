//! Drone and remote controller API. Who sees and changes what is the policy's
//! decision (`see_equipment`, `change_equipment`).

use axum::{
    Json, Router,
    extract::{FromRef, Path, State},
    routing::get,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use super::{
    db::{Equipment, Kind},
    service::{self, EquipmentInput},
};
use crate::{auth::Actor, error::ApiResult};

pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    PgPool: FromRef<S>,
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
    team_id: Option<Uuid>,
    holder_user_id: Option<Uuid>,
}

impl From<Body> for EquipmentInput {
    fn from(body: Body) -> Self {
        Self {
            name: body.name,
            team_id: body.team_id,
            holder_user_id: body.holder_user_id,
        }
    }
}

async fn list_aircraft(actor: Actor, State(db): State<PgPool>) -> ApiResult<Json<Vec<Equipment>>> {
    Ok(Json(service::list(&db, &actor, Kind::Aircraft).await?))
}

async fn show_aircraft(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Equipment>> {
    Ok(Json(service::get(&db, &actor, Kind::Aircraft, id).await?))
}

async fn update_aircraft(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(body): Json<Body>,
) -> ApiResult<Json<Equipment>> {
    let input = body.into();
    Ok(Json(
        service::update(&db, &actor, Kind::Aircraft, id, &input).await?,
    ))
}

async fn list_controllers(
    actor: Actor,
    State(db): State<PgPool>,
) -> ApiResult<Json<Vec<Equipment>>> {
    Ok(Json(
        service::list(&db, &actor, Kind::RemoteController).await?,
    ))
}

async fn show_controller(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Equipment>> {
    Ok(Json(
        service::get(&db, &actor, Kind::RemoteController, id).await?,
    ))
}

async fn update_controller(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(body): Json<Body>,
) -> ApiResult<Json<Equipment>> {
    let input = body.into();
    Ok(Json(
        service::update(&db, &actor, Kind::RemoteController, id, &input).await?,
    ))
}
