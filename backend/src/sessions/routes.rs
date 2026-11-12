//! Session API. A session is visible when one of its flights is, and shows
//! only the flights the policy allows.

use crate::db::Db;
use axum::{
    Json, Router,
    extract::{FromRef, Path, Query},
    http::StatusCode,
    routing::{get, post, put},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use chrono::{DateTime, Utc};

use super::comments;
use super::db::{Neighbours, SessionSummary, Window};
use super::service::{self, Session};
use crate::{
    auth::Actor,
    error::{ApiError, ApiResult},
    flights::{self, db::Filter},
};

const DEFAULT_PAGE_SIZE: u32 = 50;
const MAX_PAGE_SIZE: u32 = 100;

pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    crate::db::System: FromRef<S>,
    crate::db::Tenants: FromRef<S>,
{
    Router::new()
        .route("/api/sessions", get(list).post(create))
        .route("/api/sessions/{id}", get(show).delete(delete))
        .route("/api/sessions/{id}/comments", post(add_comment))
        .route(
            "/api/sessions/{id}/comments/{comment_id}",
            put(edit_comment).delete(delete_comment),
        )
        .route("/api/sessions/{id}/neighbours", get(neighbours))
        .route("/api/sessions/{id}/flights", post(add))
        .route("/api/sessions/{id}/group", post(set_group))
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

async fn merge(actor: Actor, mut db: Db, Json(change): Json<Merge>) -> ApiResult<Json<Changed>> {
    let id = service::merge(&mut db, &actor, &change.session_ids).await?;
    Ok(Json(Changed { id }))
}

async fn split(actor: Actor, mut db: Db, Json(change): Json<Split>) -> ApiResult<Json<Changed>> {
    let id = service::split(&mut db, &actor, &change.flight_ids).await?;
    Ok(Json(Changed { id }))
}

/// Flights to put into a new session, or to add to one.
#[derive(Deserialize)]
struct Flights {
    flight_ids: Vec<Uuid>,
}

/// Flights to put into a new session of a group.
#[derive(Deserialize)]
struct NewSession {
    flight_ids: Vec<Uuid>,
    /// The session's group; left out, the flights' group when they share
    /// one.
    group_id: Option<Uuid>,
}

async fn create(
    actor: Actor,
    mut db: Db,
    Json(change): Json<NewSession>,
) -> ApiResult<Json<Changed>> {
    let id = service::create(&mut db, &actor, &change.flight_ids, change.group_id).await?;
    Ok(Json(Changed { id }))
}

#[derive(Deserialize)]
struct GroupChange {
    group_id: Uuid,
    /// Moves the session's flights to the group as well.
    #[serde(default)]
    move_flights: bool,
}

/// Moves a session to another group.
async fn set_group(
    actor: Actor,
    mut db: Db,
    Path(id): Path<Uuid>,
    Json(change): Json<GroupChange>,
) -> ApiResult<Json<Changed>> {
    service::set_group(&mut db, &actor, id, change.group_id, change.move_flights).await?;
    Ok(Json(Changed { id }))
}

#[derive(Serialize)]
struct Removed {
    /// Flights that were in a session.
    removed: u64,
}

/// Takes the flights out of their sessions.
async fn remove(actor: Actor, mut db: Db, Json(change): Json<Flights>) -> ApiResult<Json<Removed>> {
    let removed = service::remove(&mut db, &actor, &change.flight_ids).await?;
    Ok(Json(Removed { removed }))
}

async fn add(
    actor: Actor,
    mut db: Db,
    Path(id): Path<Uuid>,
    Json(change): Json<Flights>,
) -> ApiResult<Json<Changed>> {
    service::add(&mut db, &actor, id, &change.flight_ids).await?;
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
    /// `tracks=1`: each flight with a coarse track, for a map of them all.
    #[serde(default)]
    tracks: Option<u8>,
}

/// The Flights list's filters, from the same query string: only sessions
/// with a flight that passes them.
#[derive(Deserialize)]
struct FilterQuery {
    /// Drone serials, comma-separated.
    aircraft: Option<String>,
    /// Controller serials, comma-separated.
    controller: Option<String>,
    /// The sessions' own groups' ids, comma-separated; `none` for sessions
    /// in the organisation.
    group: Option<String>,
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
            &self.group,
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
    mut db: Db,
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
        &mut db,
        &actor,
        &filter.parse()?,
        window,
        i64::from(size),
        offset,
        query.tracks == Some(1),
    )
    .await?;
    Ok(Json(SessionPage { sessions, total }))
}

async fn show(actor: Actor, mut db: Db, Path(id): Path<Uuid>) -> ApiResult<Json<Session>> {
    Ok(Json(service::get(&mut db, &actor, id).await?))
}

/// The session's place in the list with these filters, newest first: the
/// sessions before and after it, and when they start.
async fn neighbours(
    actor: Actor,
    mut db: Db,
    Path(id): Path<Uuid>,
    Query(filter): Query<FilterQuery>,
) -> ApiResult<Json<Neighbours>> {
    Ok(Json(
        service::neighbours(&mut db, &actor, &filter.parse()?, id).await?,
    ))
}

/// Deletes a session and its comments; its flights stay, without a session.
async fn delete(actor: Actor, mut db: Db, Path(id): Path<Uuid>) -> ApiResult<StatusCode> {
    service::delete(&mut db, &actor, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct CommentText {
    body: String,
}

#[derive(Serialize)]
struct CommentId {
    id: Uuid,
}

async fn add_comment(
    actor: Actor,
    mut db: Db,
    Path(id): Path<Uuid>,
    Json(comment): Json<CommentText>,
) -> ApiResult<Json<CommentId>> {
    let id = comments::add(&mut db, &actor, id, &comment.body).await?;
    Ok(Json(CommentId { id }))
}

async fn edit_comment(
    actor: Actor,
    mut db: Db,
    Path((session, id)): Path<(Uuid, Uuid)>,
    Json(comment): Json<CommentText>,
) -> ApiResult<Json<CommentId>> {
    comments::edit(&mut db, &actor, session, id, &comment.body).await?;
    Ok(Json(CommentId { id }))
}

async fn delete_comment(
    actor: Actor,
    mut db: Db,
    Path((session, id)): Path<(Uuid, Uuid)>,
) -> ApiResult<StatusCode> {
    comments::delete(&mut db, &actor, session, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
