//! Flight API. Each account sees only the flights the policy allows.

use axum::{
    Json, Router,
    extract::{FromRef, Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use super::db::{Filter, FlightDetail, FlightSummary, Neighbours, Sort, SortKey};
use super::service;
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
        .route("/api/flights", get(list))
        .route("/api/flights/choices", get(choices))
        .route("/api/flights/devices", get(devices))
        .route("/api/flights/ids", get(ids))
        .route("/api/flights/{id}", get(show))
        .route("/api/flights/{id}/neighbours", get(neighbours))
        .route("/api/flights/pilot", post(set_pilot))
        .route("/api/flights/team", post(set_team))
}

#[derive(Deserialize)]
struct PilotChange {
    flight_ids: Vec<Uuid>,
    /// None: unknown.
    pilot_id: Option<Uuid>,
}

#[derive(Deserialize)]
struct TeamChange {
    flight_ids: Vec<Uuid>,
    /// None: the organisation's queue.
    team_id: Option<Uuid>,
}

#[derive(Serialize)]
struct Changed {
    changed: u64,
}

async fn show(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<FlightDetail>> {
    Ok(Json(service::get(&db, &actor, id).await?))
}

async fn choices(actor: Actor, State(db): State<PgPool>) -> ApiResult<Json<service::Choices>> {
    Ok(Json(service::choices(&db, &actor).await?))
}

#[derive(Serialize)]
struct Ids {
    ids: Vec<Uuid>,
}

/// The ids of all flights that pass the list's filters (same query as
/// `list`), for a batch action on all of them.
async fn ids(
    actor: Actor,
    State(db): State<PgPool>,
    Query(query): Query<ListQuery>,
) -> ApiResult<Json<Ids>> {
    let (filter, _) = filter_and_sort(&query).map_err(ApiError::Invalid)?;
    Ok(Json(Ids {
        ids: service::ids(&db, &actor, &filter).await?,
    }))
}

/// The drones and controllers to offer in the filters, narrowed by the list's
/// other filters (same query as `list`).
async fn devices(
    actor: Actor,
    State(db): State<PgPool>,
    Query(query): Query<ListQuery>,
) -> ApiResult<Json<service::Devices>> {
    let (filter, _) = filter_and_sort(&query).map_err(ApiError::Invalid)?;
    Ok(Json(service::devices(&db, &actor, &filter).await?))
}

async fn set_pilot(
    actor: Actor,
    State(db): State<PgPool>,
    Json(change): Json<PilotChange>,
) -> ApiResult<Json<Changed>> {
    let changed = service::set_pilot(&db, &actor, &change.flight_ids, change.pilot_id).await?;
    Ok(Json(Changed { changed }))
}

async fn set_team(
    actor: Actor,
    State(db): State<PgPool>,
    Json(change): Json<TeamChange>,
) -> ApiResult<Json<Changed>> {
    let changed = service::set_team(&db, &actor, &change.flight_ids, change.team_id).await?;
    Ok(Json(Changed { changed }))
}

#[derive(Deserialize)]
struct ListQuery {
    /// 1-based.
    page: Option<u32>,
    size: Option<u32>,
    /// A column name, prefixed with `-` for descending order.
    sort: Option<String>,
    /// Only the flights of these drone serials, comma-separated.
    aircraft: Option<String>,
    /// Only the flights of these controller serials, comma-separated.
    controller: Option<String>,
    /// `none`: only flights in no session.
    session: Option<String>,
    /// Only flights taking off at or after this instant.
    from: Option<DateTime<Utc>>,
    /// Only flights taking off before this instant.
    to: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
struct FlightPage {
    flights: Vec<FlightSummary>,
    total: i64,
}

async fn list(
    actor: Actor,
    State(db): State<PgPool>,
    Query(query): Query<ListQuery>,
) -> Result<Json<FlightPage>, (StatusCode, String)> {
    let page = query.page.unwrap_or(1).max(1);
    let size = query.size.unwrap_or(DEFAULT_PAGE_SIZE);
    if !(1..=MAX_PAGE_SIZE).contains(&size) {
        return Err(bad_request(format!(
            "size must be between 1 and {MAX_PAGE_SIZE}"
        )));
    }
    let (filter, sort) = filter_and_sort(&query).map_err(bad_request)?;
    let offset = i64::from(page - 1) * i64::from(size);
    match service::list(&db, &actor, &filter, &sort, i64::from(size), offset).await {
        Ok((flights, total)) => Ok(Json(FlightPage { flights, total })),
        Err(error) => {
            tracing::error!(%error, "listing flights");
            Err((StatusCode::INTERNAL_SERVER_ERROR, String::new()))
        }
    }
}

/// The list's filter and order from the query; the same for a flight's neighbours.
fn filter_and_sort(query: &ListQuery) -> Result<(Filter, Sort), String> {
    let sort = parse_sort(query.sort.as_deref().unwrap_or("-takeoff_at"))
        .ok_or_else(|| "unknown sort column".to_owned())?;
    let without_session = match query.session.as_deref() {
        None => false,
        Some("none") => true,
        Some(_) => return Err("session must be none".to_owned()),
    };
    // Serials have no commas; empty parts are left out.
    let serials = |value: &Option<String>| {
        value.as_ref().map(|value| {
            value
                .split(',')
                .filter(|serial| !serial.is_empty())
                .map(str::to_owned)
                .collect()
        })
    };
    let filter = Filter {
        aircraft_serials: serials(&query.aircraft),
        rc_serials: serials(&query.controller),
        without_session,
        from: query.from,
        to: query.to,
    };
    Ok((filter, sort))
}

/// The flight's place in the list with the same query as `list` (its page and
/// size don't matter): the flights before and after it, its position and the
/// list's length. Not found if the flight isn't in that list.
async fn neighbours(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
    Query(query): Query<ListQuery>,
) -> ApiResult<Json<Neighbours>> {
    let (filter, sort) = filter_and_sort(&query).map_err(ApiError::Invalid)?;
    Ok(Json(
        service::neighbours(&db, &actor, &filter, &sort, id).await?,
    ))
}

fn parse_sort(value: &str) -> Option<Sort> {
    let (descending, name) = match value.strip_prefix('-') {
        Some(name) => (true, name),
        None => (false, value),
    };
    Some(Sort {
        key: SortKey::parse(name)?,
        descending,
    })
}

fn bad_request(message: String) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, message)
}
