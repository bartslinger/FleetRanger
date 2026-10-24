//! The processing status API, for admins, and log uploads from controllers.

use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, FromRef, Path, State},
    http::{HeaderMap, StatusCode},
    middleware,
    routing::{get, post, put},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use super::LogDir;
use super::activity::Activity;
use super::db::Stored;
use super::service::{self, Status};
use super::upload;
use crate::{
    auth::Actor,
    error::{ApiError, ApiResult},
    events::Changes,
    pairing::Controller,
};

/// DJI logs are a few megabytes; long flights stay well under this.
const MAX_LOG_BYTES: usize = 256 * 1024 * 1024;

pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    PgPool: FromRef<S>,
    Activity: FromRef<S>,
    LogDir: FromRef<S>,
    Changes: FromRef<S>,
{
    // The controller's API, see "Clients" in docs/architecture.md.
    let v1 = Router::new()
        .route(
            "/api/controller/v1/logs/{file_name}",
            put(put_log).layer(DefaultBodyLimit::max(MAX_LOG_BYTES)),
        )
        .route("/api/controller/v1/logs/check", post(missing))
        .route_layer(middleware::from_fn(crate::pairing::v1_errors));
    // What android-v0.3.0 calls, until no controller reports it.
    let v0 = Router::new()
        .route(
            "/api/controller/logs",
            post(receive_v0).layer(DefaultBodyLimit::max(MAX_LOG_BYTES)),
        )
        .route("/api/controller/logs/missing", post(missing));
    Router::new()
        .route("/api/logs/processing", get(status))
        .merge(v1)
        .merge(v0)
}

#[derive(Deserialize)]
struct Hashes {
    sha256: Vec<String>,
}

#[derive(Serialize)]
struct Missing {
    missing: Vec<String>,
}

/// Which logs, by SHA-256 in hex, the server doesn't have: a paired controller
/// asks before uploading, so a log already stored (also after a reinstall, or
/// when another install's logs were moved here) isn't sent again.
async fn missing(
    _controller: Controller,
    State(db): State<PgPool>,
    Json(hashes): Json<Hashes>,
) -> ApiResult<Json<Missing>> {
    Ok(Json(Missing {
        missing: upload::missing(&db, hashes.sha256).await?,
    }))
}

async fn status(
    actor: Actor,
    State(db): State<PgPool>,
    State(activity): State<Activity>,
) -> ApiResult<Json<Status>> {
    Ok(Json(service::status(&db, &actor, &activity).await?))
}

#[derive(Serialize)]
struct Received {
    result: Stored,
}

/// A log from a paired controller, put at its name on the controller: the
/// file itself as the body, with headers `X-SHA256` (hex) and, if a pilot was
/// chosen, `X-Pilot-Id` and `X-Group-Id`. 201 for a new log, 200 for a new
/// version of one or a duplicate, so sending it again is harmless.
async fn put_log(
    controller: Controller,
    State(db): State<PgPool>,
    State(LogDir(log_dir)): State<LogDir>,
    State(changes): State<Changes>,
    Path(file_name): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<(StatusCode, Json<Received>)> {
    let chosen = upload::Chosen {
        pilot_id: id_header(&headers, "x-pilot-id")?,
        group_id: id_header(&headers, "x-group-id")?,
    };
    let sha256 = text_header(&headers, "x-sha256");
    let result = upload::receive(
        &db,
        &log_dir,
        &controller,
        &file_name,
        sha256,
        chosen,
        &body,
    )
    .await?;
    if result != Stored::Duplicate {
        // A log waiting to be processed: admins see it come in.
        changes.notify();
    }
    let status = match result {
        Stored::Added => StatusCode::CREATED,
        Stored::Replaced | Stored::Duplicate => StatusCode::OK,
    };
    Ok((status, Json(Received { result })))
}

/// android-v0.3.0's upload: the name in `X-File-Name`, and the group chosen
/// with the pilot in `X-Team-Id`, which is the group's id: the groups kept
/// their teams' ids, and its pilot list gives each group's id as the team's.
/// The same checks as for later apps then decide whether the flight gets it.
/// Anything that isn't an id counts as no choice, so an older app's upload
/// never fails over it.
async fn receive_v0(
    controller: Controller,
    State(db): State<PgPool>,
    State(LogDir(log_dir)): State<LogDir>,
    State(changes): State<Changes>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<(StatusCode, Json<Received>)> {
    let chosen = upload::Chosen {
        pilot_id: id_header(&headers, "x-pilot-id")?,
        group_id: id_header(&headers, "x-team-id").unwrap_or(None),
    };
    let file_name = text_header(&headers, "x-file-name");
    let sha256 = text_header(&headers, "x-sha256");
    let result =
        upload::receive(&db, &log_dir, &controller, file_name, sha256, chosen, &body).await?;
    let status = match result {
        Stored::Duplicate => StatusCode::OK,
        Stored::Added | Stored::Replaced => {
            changes.notify();
            StatusCode::CREATED
        }
    };
    Ok((status, Json(Received { result })))
}

fn text_header<'a>(headers: &'a HeaderMap, name: &str) -> &'a str {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
}

fn id_header(headers: &HeaderMap, name: &str) -> ApiResult<Option<Uuid>> {
    match headers.get(name) {
        Some(value) => value
            .to_str()
            .ok()
            .and_then(|id| id.parse().ok())
            .map(Some)
            .ok_or_else(|| ApiError::Invalid(format!("{name} is not an id."))),
        None => Ok(None),
    }
}
