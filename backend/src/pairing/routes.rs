//! Pairing API: a user creates and revokes codes; a controller pairs with one
//! and later unpairs itself; an admin can unpair a controller.

use axum::{
    Json, Router,
    extract::{FromRef, Path, State},
    http::StatusCode,
    middleware,
    routing::{delete, get, post, put},
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use super::{
    Controller,
    db::{self, OpenCode},
    service::{self, Created, Destinations, PilotList, Pilots},
};
use crate::{
    auth::Actor,
    error::{ApiError, ApiResult},
    flights::db::GroupChoice,
};

pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    PgPool: FromRef<S>,
{
    // The controller's API, see "Clients" in docs/architecture.md.
    let v1 = Router::new()
        .route("/api/controller/v1/pairing", post(pair).delete(unpair_self))
        .route("/api/controller/v1/pilots", get(pilots))
        .route("/api/controller/v1/status", put(status))
        .route_layer(middleware::from_fn(super::v1_errors));
    // What android-v0.3.0 calls, until no controller reports it.
    let v0 = Router::new()
        .route("/api/controller", delete(unpair_self))
        .route("/api/controller/pairing", post(pair_v0))
        .route("/api/controller/pilots", get(pilots_v0))
        .route("/api/controller/sync", post(synced_v0));
    Router::new()
        .route("/api/pairing-codes", get(open_codes).post(create))
        .route("/api/pairing-codes/groups", get(destinations))
        .route("/api/pairing-codes/{id}", delete(revoke))
        .route("/api/remote-controllers/{id}/pairing", delete(unpair))
        .merge(v1)
        .merge(v0)
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

#[derive(Serialize)]
struct PairedV1 {
    /// None for the organisation.
    group: Option<GroupChoice>,
}

async fn pair(State(db): State<PgPool>, Json(body): Json<Pair>) -> ApiResult<Json<PairedV1>> {
    let paired = service::pair(&db, &body.code, &body.serial, &body.token_sha256).await?;
    Ok(Json(PairedV1 {
        group: paired.group,
    }))
}

#[derive(Serialize)]
struct PairedV0 {
    /// The group's name, which android-v0.3.0 calls a team.
    team: Option<String>,
}

async fn pair_v0(State(db): State<PgPool>, Json(body): Json<Pair>) -> ApiResult<Json<PairedV0>> {
    let paired = service::pair(&db, &body.code, &body.serial, &body.token_sha256).await?;
    Ok(Json(PairedV0 {
        team: paired.group.map(|g| g.name),
    }))
}

async fn pilots(controller: Controller, State(db): State<PgPool>) -> ApiResult<Json<Pilots>> {
    Ok(Json(service::pilots(&db, &controller).await?))
}

async fn pilots_v0(controller: Controller, State(db): State<PgPool>) -> ApiResult<Json<PilotList>> {
    Ok(Json(service::pilots_v0(&db, &controller).await?))
}

/// How far ahead of the server's clock a controller's may be, my choice: a
/// controller sets its clock from GPS or the network, so more is a wrong
/// clock rather than drift.
const CLOCK_SKEW: Duration = Duration::minutes(5);

#[derive(Deserialize)]
struct Status {
    /// The app's version name, e.g. "0.4.0" or "dev".
    app_version: String,
    /// When its last sync finished: it looked for logs on a connection its
    /// settings allow, and uploaded any. Left out until the first one.
    last_sync: Option<DateTime<Utc>>,
}

/// The controller's status, sent whole, also without a sync (at start).
async fn status(
    controller: Controller,
    State(db): State<PgPool>,
    Json(body): Json<Status>,
) -> ApiResult<StatusCode> {
    let version = app_version(&body.app_version)?;
    if body
        .last_sync
        .is_some_and(|at| at > Utc::now() + CLOCK_SKEW)
    {
        return Err(ApiError::Invalid(
            "The last sync is in the future: check the controller's clock.".to_owned(),
        ));
    }
    db::status(&db, controller.id, version, body.last_sync).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct Synced {
    app_version: String,
}

/// android-v0.3.0's sync report: it synced just now.
async fn synced_v0(
    controller: Controller,
    State(db): State<PgPool>,
    Json(body): Json<Synced>,
) -> ApiResult<StatusCode> {
    let version = app_version(&body.app_version)?;
    db::status(&db, controller.id, version, Some(Utc::now())).await?;
    Ok(StatusCode::NO_CONTENT)
}

fn app_version(version: &str) -> ApiResult<&str> {
    let version = version.trim();
    if version.is_empty() || version.len() > 64 {
        return Err(ApiError::Invalid(
            "app_version must be 1 to 64 characters".to_owned(),
        ));
    }
    Ok(version)
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
