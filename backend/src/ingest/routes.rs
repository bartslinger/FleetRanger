//! The processing status API, for admins, and log uploads from controllers.

use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, FromRef, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
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
    Router::new()
        .route("/api/logs/processing", get(status))
        .route(
            "/api/controller/logs",
            post(receive).layer(DefaultBodyLimit::max(MAX_LOG_BYTES)),
        )
        .route("/api/controller/logs/missing", post(missing))
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

/// A log from a paired controller: the file itself as the body, with headers
/// `X-File-Name` (its name on the controller), `X-SHA256` (hex) and, if a pilot
/// was chosen, `X-Pilot-Id` with the group chosen with them, `X-Team-Id`.
async fn receive(
    controller: Controller,
    State(db): State<PgPool>,
    State(LogDir(log_dir)): State<LogDir>,
    State(changes): State<Changes>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<(StatusCode, Json<Received>)> {
    let header = |name: &str| headers.get(name).and_then(|value| value.to_str().ok());
    let file_name = header("x-file-name").unwrap_or_default();
    let sha256 = header("x-sha256").unwrap_or_default();
    let id = |name: &str| match header(name) {
        Some(id) => id
            .parse::<Uuid>()
            .map(Some)
            .map_err(|_| ApiError::Invalid(format!("{name} is not an id."))),
        None => Ok(None),
    };
    let chosen = upload::Chosen {
        pilot_id: id("x-pilot-id")?,
        team_id: id("x-team-id")?,
    };
    let result =
        upload::receive(&db, &log_dir, &controller, file_name, sha256, chosen, &body).await?;
    let status = match result {
        Stored::Duplicate => StatusCode::OK,
        Stored::Added | Stored::Replaced => {
            // A log waiting to be processed: admins see it come in.
            changes.notify();
            StatusCode::CREATED
        }
    };
    Ok((status, Json(Received { result })))
}
