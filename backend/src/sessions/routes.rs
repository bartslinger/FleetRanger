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
    flights::{
        self,
        db::{Filter, Neighbours},
    },
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
        .route("/api/sessions/{id}/neighbours", get(neighbours))
        .route("/api/sessions/{id}/flights", post(add))
        .route("/api/sessions/merge", post(merge))
        .route("/api/sessions/split", post(split))
        .route("/api/sessions/remove", post(remove))
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

#[derive(Serialize)]
struct Removed {
    /// Flights that were in a session.
    removed: u64,
}

/// Takes the flights out of their sessions.
async fn remove(
    actor: Actor,
    State(db): State<PgPool>,
    Json(change): Json<Flights>,
) -> ApiResult<Json<Removed>> {
    let removed = service::remove(&db, &actor, &change.flight_ids).await?;
    Ok(Json(Removed { removed }))
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
    /// With `span_to`: only sessions with a flight in this span, e.g. the
    /// ones near flights to be added to a session.
    span_from: Option<DateTime<Utc>>,
    span_to: Option<DateTime<Utc>>,
}

/// The Flights list's filters, from the same query string: only sessions
/// with a flight that passes them.
#[derive(Deserialize)]
struct FilterQuery {
    /// Drone serials, comma-separated.
    aircraft: Option<String>,
    /// Controller serials, comma-separated.
    controller: Option<String>,
    /// Team ids, comma-separated; `none` for flights without a team.
    team: Option<String>,
    /// Pilots' user ids, comma-separated; `none` for flights without a pilot.
    pilot: Option<String>,
    /// Takeoff at or after this instant.
    from: Option<DateTime<Utc>>,
    /// Takeoff before this instant.
    to: Option<DateTime<Utc>>,
    /// Area ids, comma-separated.
    area: Option<String>,
}

impl FilterQuery {
    fn parse(&self) -> ApiResult<Filter> {
        flights::routes::filter(
            &self.aircraft,
            &self.controller,
            &self.team,
            &self.pilot,
            &self.area,
            self.from,
            self.to,
        )
        .map_err(ApiError::Invalid)
    }
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
    Query(filter): Query<FilterQuery>,
) -> ApiResult<Json<SessionPage>> {
    let page = query.page.unwrap_or(1).max(1);
    let size = query.size.unwrap_or(DEFAULT_PAGE_SIZE);
    if !(1..=MAX_PAGE_SIZE).contains(&size) {
        return Err(ApiError::Invalid(format!(
            "size must be between 1 and {MAX_PAGE_SIZE}"
        )));
    }
    let window = match (query.span_from, query.span_to) {
        (Some(from), Some(to)) => Some(Window { from, to }),
        (None, None) => None,
        _ => {
            return Err(ApiError::Invalid(
                "give both span_from and span_to".to_owned(),
            ));
        }
    };
    let offset = i64::from(page - 1) * i64::from(size);
    let (sessions, total) = service::list(
        &db,
        &actor,
        &filter.parse()?,
        window,
        i64::from(size),
        offset,
    )
    .await?;
    Ok(Json(SessionPage { sessions, total }))
}

async fn show(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Session>> {
    Ok(Json(service::get(&db, &actor, id).await?))
}

/// The session's place in the list with these filters, newest first: the
/// sessions before and after it, its position and the list's length.
async fn neighbours(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
    Query(filter): Query<FilterQuery>,
) -> ApiResult<Json<Neighbours>> {
    Ok(Json(
        service::neighbours(&db, &actor, &filter.parse()?, id).await?,
    ))
}
