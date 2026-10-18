//! Pairing API: the controller asks to pair and later unpairs itself; a user
//! approves; an admin can unpair a controller.

use axum::{
    Json, Router,
    extract::{FromRef, Path, State},
    http::{HeaderMap, StatusCode},
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use super::{
    Controller,
    db::{self, Request, Waiting},
    service::{self, Check, Started},
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
        .route("/api/pairing", get(waiting).post(start))
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

async fn waiting(actor: Actor, State(db): State<PgPool>) -> ApiResult<Json<Vec<Waiting>>> {
    Ok(Json(service::waiting(&db, &actor).await?))
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
) -> ApiResult<Json<Approved>> {
    let serial = service::approve(&db, &actor, &code).await?;
    Ok(Json(Approved { serial }))
}

/// The controller that was paired.
#[derive(Serialize)]
struct Approved {
    serial: String,
}

#[derive(Serialize)]
struct Me {
    paired: bool,
}

/// For the controller to check its token: paired, still waiting for approval
/// (which keeps its request alive), or unknown (401): it starts over.
async fn me(State(db): State<PgPool>, headers: HeaderMap) -> ApiResult<Json<Me>> {
    let hash = super::token_hash(&headers).ok_or(ApiError::Unauthorized)?;
    match service::check(&db, &hash).await? {
        Check::Paired => Ok(Json(Me { paired: true })),
        Check::Waiting => Ok(Json(Me { paired: false })),
        Check::Unknown => Err(ApiError::Unauthorized),
    }
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
