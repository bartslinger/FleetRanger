//! Pairing API: the controller asks to pair and later unpairs itself; a user
//! approves; an admin can unpair a controller.

use axum::{
    Json, Router,
    extract::{FromRef, Path, State},
    http::StatusCode,
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use super::{
    Controller,
    db::{self, Request},
    service::{self, Started},
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
        .route("/api/pairing", post(start))
        .route("/api/pairing/{code}", get(show))
        .route("/api/pairing/{code}/approve", post(approve))
        .route("/api/controller", get(me).delete(unpair_self))
        .route("/api/remote-controllers/{id}/pairing", delete(unpair))
}

#[derive(Deserialize)]
struct Start {
    serial: String,
    model: String,
    /// Hex.
    token_sha256: String,
}

async fn start(State(db): State<PgPool>, Json(body): Json<Start>) -> ApiResult<Json<Started>> {
    Ok(Json(
        service::start(&db, &body.serial, &body.model, &body.token_sha256).await?,
    ))
}

async fn show(
    actor: Actor,
    State(db): State<PgPool>,
    Path(code): Path<String>,
) -> ApiResult<Json<Request>> {
    Ok(Json(service::request(&db, &actor, &code).await?))
}

async fn approve(
    actor: Actor,
    State(db): State<PgPool>,
    Path(code): Path<String>,
) -> ApiResult<StatusCode> {
    service::approve(&db, &actor, &code).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
struct Me {
    paired: bool,
}

/// For the controller to check that it is paired: until a user approves it,
/// its token is refused.
async fn me(_controller: Controller) -> Json<Me> {
    Json(Me { paired: true })
}

async fn unpair_self(controller: Controller, State(db): State<PgPool>) -> ApiResult<StatusCode> {
    if !db::unpair(&db, controller.id).await? {
        return Err(ApiError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn unpair(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    service::unpair(&db, &actor, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
