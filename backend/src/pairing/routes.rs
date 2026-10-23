//! Pairing API: a user creates and revokes codes; a controller pairs with one
//! and later unpairs itself; an admin can unpair a controller.

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
    db::{self, OpenCode, Paired},
    service::{self, Created, Destinations, PilotList},
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
        .route("/api/pairing-codes", get(open_codes).post(create))
        .route("/api/pairing-codes/groups", get(destinations))
        .route("/api/pairing-codes/{id}", delete(revoke))
        .route("/api/controller", get(me).delete(unpair_self))
        .route("/api/controller/pairing", post(pair))
        .route("/api/controller/pilots", get(pilots))
        .route("/api/controller/sync", post(synced))
        .route("/api/remote-controllers/{id}/pairing", delete(unpair))
}

#[derive(Deserialize)]
struct Create {
    /// The group the controller joins; none for the organisation.
    group: Option<Uuid>,
    /// How long the code stays valid.
    hours: i64,
}

async fn create(
    actor: Actor,
    State(db): State<PgPool>,
    Json(body): Json<Create>,
) -> ApiResult<Json<Created>> {
    Ok(Json(
        service::create(&db, &actor, body.group, body.hours).await?,
    ))
}

async fn open_codes(actor: Actor, State(db): State<PgPool>) -> ApiResult<Json<Vec<OpenCode>>> {
    Ok(Json(service::open_codes(&db, &actor).await?))
}

async fn revoke(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    service::revoke(&db, &actor, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Where a controller paired with the actor's code can go.
async fn destinations(actor: Actor, State(db): State<PgPool>) -> ApiResult<Json<Destinations>> {
    Ok(Json(service::destinations(&db, &actor).await?))
}

#[derive(Deserialize)]
struct Pair {
    code: String,
    serial: String,
    /// Hex.
    token_sha256: String,
}

async fn pair(State(db): State<PgPool>, Json(body): Json<Pair>) -> ApiResult<Json<Paired>> {
    Ok(Json(
        service::pair(&db, &body.code, &body.serial, &body.token_sha256).await?,
    ))
}

#[derive(Serialize)]
struct Me {
    paired: bool,
}

/// For the controller to check its token: paired, or unknown (401), when it
/// was unpaired and has to pair again.
async fn me(State(db): State<PgPool>, headers: HeaderMap) -> ApiResult<Json<Me>> {
    let hash = super::token_hash(&headers).ok_or(ApiError::Unauthorized)?;
    if service::paired(&db, &hash).await? {
        Ok(Json(Me { paired: true }))
    } else {
        Err(ApiError::Unauthorized)
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
