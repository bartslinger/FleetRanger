//! Session API. A session is visible when one of its flights is, and shows
//! only the flights the policy allows.

use axum::{
    Json, Router,
    extract::{FromRef, Path, Query, State},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use chrono::{DateTime, Utc};

use super::db::{SessionSummary, Window};
use super::service::{self, Session};
use crate::{
    auth::Actor,
    error::{ApiError, ApiResult},
};

const DEFAULT_PAGE_SIZE: u32 = 50;
const MAX_PAGE_SIZE: u32 = 100;

pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    PgPool: FromRef<S>,
{
    Router::new()
        .route("/api/sessions", get(list).post(create))
        .route("/api/sessions/{id}", get(show))
        .route("/api/sessions/{id}/flights", post(add))
        .route("/api/sessions/merge", post(merge))
        .route("/api/sessions/split", post(split))
}

#[derive(Deserialize)]
struct Merge {
    session_ids: Vec<Uuid>,
}

#[derive(Deserialize)]
struct Split {
    flight_ids: Vec<Uuid>,
}

/// The session that resulted.
#[derive(Serialize)]
struct Changed {
    id: Uuid,
}

async fn merge(
    actor: Actor,
    State(db): State<PgPool>,
    Json(change): Json<Merge>,
) -> ApiResult<Json<Changed>> {
    let id = service::merge(&db, &actor, &change.session_ids).await?;
    Ok(Json(Changed { id }))
}

async fn split(
    actor: Actor,
    State(db): State<PgPool>,
    Json(change): Json<Split>,
) -> ApiResult<Json<Changed>> {
    let id = service::split(&db, &actor, &change.flight_ids).await?;
    Ok(Json(Changed { id }))
}

/// Flights to put into a new session, or to add to one.
#[derive(Deserialize)]
struct Flights {
    flight_ids: Vec<Uuid>,
}

async fn create(
    actor: Actor,
    State(db): State<PgPool>,
    Json(change): Json<Flights>,
) -> ApiResult<Json<Changed>> {
    let id = service::create(&db, &actor, &change.flight_ids).await?;
    Ok(Json(Changed { id }))
}

async fn add(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(change): Json<Flights>,
) -> ApiResult<Json<Changed>> {
    service::add(&db, &actor, id, &change.flight_ids).await?;
    Ok(Json(Changed { id }))
}

#[derive(Deserialize)]
struct ListQuery {
    /// 1-based.
    page: Option<u32>,
    size: Option<u32>,
    /// With `to`: only sessions with a flight in this span.
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
struct SessionPage {
    sessions: Vec<SessionSummary>,
    total: i64,
}

async fn list(
    actor: Actor,
    State(db): State<PgPool>,
    Query(query): Query<ListQuery>,
) -> ApiResult<Json<SessionPage>> {
    let page = query.page.unwrap_or(1).max(1);
    let size = query.size.unwrap_or(DEFAULT_PAGE_SIZE);
    if !(1..=MAX_PAGE_SIZE).contains(&size) {
        return Err(ApiError::Invalid(format!(
            "size must be between 1 and {MAX_PAGE_SIZE}"
        )));
    }
    let window = match (query.from, query.to) {
        (Some(from), Some(to)) => Some(Window { from, to }),
        (None, None) => None,
        _ => return Err(ApiError::Invalid("give both from and to".to_owned())),
    };
    let offset = i64::from(page - 1) * i64::from(size);
    let (sessions, total) = service::list(&db, &actor, window, i64::from(size), offset).await?;
    Ok(Json(SessionPage { sessions, total }))
}

async fn show(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Session>> {
    Ok(Json(service::get(&db, &actor, id).await?))
}
