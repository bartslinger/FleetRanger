//! Pairing API: the controller asks to pair and later unpairs itself; a user
//! approves; an admin can unpair a controller.

use axum::{
    Json, Router,
    extract::{FromRef, Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use super::{
    Controller,
    db::{self, Approved, Waiting},
    service::{self, Check, PilotList, Started},
};
use crate::{
    auth::Actor,
    error::{ApiError, ApiResult},
    events::Changes,
    flights::db::TeamChoice,
};

pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    PgPool: FromRef<S>,
    Changes: FromRef<S>,
{
    Router::new()
        .route("/api/pairing", get(waiting).post(start))
        .route("/api/pairing/teams", get(teams))
        .route("/api/pairing/{code}/approve", post(approve))
        .route("/api/controller", get(me).delete(unpair_self))
        .route("/api/controller/pilots", get(pilots))
        .route("/api/controller/sync", post(synced))
        .route("/api/remote-controllers/{id}/pairing", delete(unpair))
}

#[derive(Deserialize)]
struct Start {
    serial: String,
    model: String,
    /// Hex.
    token_sha256: String,
}

async fn start(
    State(db): State<PgPool>,
    State(changes): State<Changes>,
    Json(body): Json<Start>,
) -> ApiResult<Json<Started>> {
    let started = service::start(&db, &body.serial, &body.model, &body.token_sha256).await?;
    changes.notify();
    Ok(Json(started))
}

async fn waiting(actor: Actor, State(db): State<PgPool>) -> ApiResult<Json<Vec<Waiting>>> {
    Ok(Json(service::waiting(&db, &actor).await?))
}

/// The teams a controller the actor approves can join.
async fn teams(actor: Actor, State(db): State<PgPool>) -> ApiResult<Json<Vec<TeamChoice>>> {
    Ok(Json(service::teams(&db, &actor).await?))
}

#[derive(Deserialize)]
struct ApproveQuery {
    /// The approver's team the controller joins, if they're in several.
    team: Option<Uuid>,
}

async fn approve(
    actor: Actor,
    State(db): State<PgPool>,
    State(changes): State<Changes>,
    Path(code): Path<String>,
    Query(query): Query<ApproveQuery>,
) -> ApiResult<Json<Approved>> {
    let approved = service::approve(&db, &actor, &code, query.team).await?;
    changes.notify();
    Ok(Json(approved))
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

async fn pilots(controller: Controller, State(db): State<PgPool>) -> ApiResult<Json<PilotList>> {
    Ok(Json(service::pilots(&db, &controller).await?))
}

#[derive(Deserialize)]
struct Synced {
    /// The app's version name, e.g. "0.1.0" or "dev".
    app_version: String,
}

/// The controller synced: on a connection its settings allow, it looked for
/// logs and uploaded any. Also without logs, so the server sees it's alive.
async fn synced(
    controller: Controller,
    State(db): State<PgPool>,
    Json(body): Json<Synced>,
) -> ApiResult<StatusCode> {
    let version = body.app_version.trim();
    if version.is_empty() || version.len() > 64 {
        return Err(ApiError::Invalid(
            "app_version must be 1 to 64 characters".to_owned(),
        ));
    }
    db::synced(&db, controller.id, version).await?;
    Ok(StatusCode::NO_CONTENT)
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
