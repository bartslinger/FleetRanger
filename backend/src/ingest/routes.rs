//! The processing status API, for admins, and log uploads from controllers.

use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, FromRef, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use serde::Serialize;
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
/// was chosen, `X-Pilot-Id`.
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
    let pilot_id = match header("x-pilot-id") {
        Some(id) => Some(
            id.parse::<Uuid>()
                .map_err(|_| ApiError::Invalid("X-Pilot-Id is not a user id.".to_owned()))?,
        ),
        None => None,
    };
    let result = upload::receive(
        &db,
        &log_dir,
        &controller,
        file_name,
        sha256,
        pilot_id,
        &body,
    )
    .await?;
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
