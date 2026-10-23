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
    sessions::{
        SessionGap,
        suggest::{self, Suggestion},
    },
};

const DEFAULT_PAGE_SIZE: u32 = 50;
const MAX_PAGE_SIZE: u32 = 100;

pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    PgPool: FromRef<S>,
    SessionGap: FromRef<S>,
{
    Router::new()
        .route("/api/flights", get(list))
        .route("/api/flights/suggestions", get(suggestions))
        .route("/api/flights/suggestions/{key}", get(suggestion))
        .route("/api/flights/choices", get(choices))
        .route("/api/flights/devices", get(devices))
        .route("/api/flights/ids", get(ids))
        .route("/api/flights/{id}", get(show))
        .route("/api/flights/{id}/neighbours", get(neighbours))
        .route("/api/flights/pilot", post(set_pilot))
        .route("/api/flights/group", post(set_group))
        .route("/api/flights/archive", post(archive))
        .route("/api/flights/restore", post(restore))
}

#[derive(Deserialize)]
struct Batch {
    flight_ids: Vec<Uuid>,
}

#[derive(Deserialize)]
struct PilotChange {
    flight_ids: Vec<Uuid>,
    /// None: unknown.
    pilot_id: Option<Uuid>,
}

#[derive(Deserialize)]
struct GroupChange {
    flight_ids: Vec<Uuid>,
    /// None: the organisation's queue.
    group_id: Option<Uuid>,
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

async fn set_group(
    actor: Actor,
    State(db): State<PgPool>,
    Json(change): Json<GroupChange>,
) -> ApiResult<Json<Changed>> {
    let changed = service::set_group(&db, &actor, &change.flight_ids, change.group_id).await?;
    Ok(Json(Changed { changed }))
}

#[derive(Deserialize)]
struct Archive {
    flight_ids: Vec<Uuid>,
    /// Takes them out of their sessions first, in the same step.
    #[serde(default)]
    leave_sessions: bool,
}

async fn archive(
    actor: Actor,
    State(db): State<PgPool>,
    Json(batch): Json<Archive>,
) -> ApiResult<Json<Changed>> {
    let changed = service::archive(&db, &actor, &batch.flight_ids, batch.leave_sessions).await?;
    Ok(Json(Changed { changed }))
}

async fn restore(
    actor: Actor,
    State(db): State<PgPool>,
    Json(batch): Json<Batch>,
) -> ApiResult<Json<Changed>> {
    let changed = service::restore(&db, &actor, &batch.flight_ids).await?;
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
    /// `none`: only flights in no session; `any`: only those in one (the
    /// Sessions list's filter options).
    session: Option<String>,
    /// Only flights taking off at or after this instant.
    from: Option<DateTime<Utc>>,
    /// Only flights taking off before this instant.
    to: Option<DateTime<Utc>>,
    /// Only the flights of these group ids, comma-separated; `none` for the
    /// flights without a group.
    group: Option<String>,
    /// Only the flights of these pilots' ids, comma-separated; `none` for the
    /// flights without a pilot.
    pilot: Option<String>,
    /// Only the flights with time in these areas, ids comma-separated.
    area: Option<String>,
    /// `suggestions`: a flight's neighbours are those of its suggested
    /// session (with `session=none`).
    view: Option<String>,
    /// `true`: only archived flights, which are otherwise left out.
    archived: Option<bool>,
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

#[derive(Serialize)]
struct SuggestionPage {
    suggestions: Vec<Suggestion>,
    total: usize,
}

/// One page of suggested sessions for the flights without a session that
/// pass the list's filters (same query as `list`; its order doesn't apply),
/// newest first. The page and size count suggestions.
async fn suggestions(
    actor: Actor,
    State(db): State<PgPool>,
    State(gap): State<SessionGap>,
    Query(query): Query<ListQuery>,
) -> ApiResult<Json<SuggestionPage>> {
    let page = query.page.unwrap_or(1).max(1);
    let size = query.size.unwrap_or(DEFAULT_PAGE_SIZE);
    if !(1..=MAX_PAGE_SIZE).contains(&size) {
        return Err(ApiError::Invalid(format!(
            "size must be between 1 and {MAX_PAGE_SIZE}"
        )));
    }
    let (filter, _) = filter_and_sort(&query).map_err(ApiError::Invalid)?;
    let offset = (page as usize - 1) * size as usize;
    let (suggestions, total) =
        suggest::list(&db, &actor, filter, gap, size as usize, offset).await?;
    Ok(Json(SuggestionPage { suggestions, total }))
}

/// The suggested session with this flight in it, with its tracks and its
/// neighbours in the list with the same query as `suggestions`. Not found if
/// the flight is in none.
async fn suggestion(
    actor: Actor,
    State(db): State<PgPool>,
    State(gap): State<SessionGap>,
    Path(key): Path<Uuid>,
    Query(query): Query<ListQuery>,
) -> ApiResult<Json<suggest::SuggestionPage>> {
    let (filter, _) = filter_and_sort(&query).map_err(ApiError::Invalid)?;
    suggest::get(&db, &actor, filter, gap, key)
        .await?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

/// The list's filter and order from the query; the same for a flight's neighbours.
fn filter_and_sort(query: &ListQuery) -> Result<(Filter, Sort), String> {
    let sort = parse_sort(query.sort.as_deref().unwrap_or("-takeoff_at"))
        .ok_or_else(|| "unknown sort column".to_owned())?;
    let in_session = match query.session.as_deref() {
        None => None,
        Some("none") => Some(false),
        Some("any") => Some(true),
        Some(_) => return Err("session must be none or any".to_owned()),
    };
    let filter = Filter {
        in_session,
        archived: query.archived.unwrap_or(false),
        ..filter(
            &query.aircraft,
            &query.controller,
            &query.group,
            &query.pilot,
            &query.area,
            query.from,
            query.to,
        )?
    };
    Ok((filter, sort))
}

/// The filters the flights and sessions lists share, from their query
/// parameters: drones, controllers, groups, pilots, areas and takeoff times.
pub fn filter(
    aircraft: &Option<String>,
    controller: &Option<String>,
    group: &Option<String>,
    pilot: &Option<String>,
    area: &Option<String>,
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
) -> Result<Filter, String> {
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
    let (group_ids, queue) = id_list(group).map_err(|()| "group must be group ids or none")?;
    let (pilot_ids, unknown_pilot) =
        id_list(pilot).map_err(|()| "pilot must be user ids or none")?;
    let (area_ids, _) = id_list(area).map_err(|()| "area must be area ids")?;
    Ok(Filter {
        aircraft_serials: serials(aircraft),
        rc_serials: serials(controller),
        in_session: None,
        from,
        to,
        group_ids,
        queue,
        pilot_ids,
        unknown_pilot,
        archived: false,
        area_ids,
    })
}

/// Comma-separated ids, and whether `none` was among them; nothing if not given.
fn id_list(value: &Option<String>) -> Result<(Option<Vec<Uuid>>, bool), ()> {
    let Some(value) = value else {
        return Ok((None, false));
    };
    let mut ids = Vec::new();
    let mut none = false;
    for part in value.split(',').filter(|part| !part.is_empty()) {
        if part == "none" {
            none = true;
        } else {
            ids.push(part.parse().map_err(|_| ())?);
        }
    }
    Ok((Some(ids), none))
}

/// The flight's place in the list with the same query as `list` (its page and
/// size don't matter): the flights before and after it, its position and the
/// list's length. Not found if the flight isn't in that list. With
/// `view=suggestions`, its place in its suggested session instead.
async fn neighbours(
    actor: Actor,
    State(db): State<PgPool>,
    State(gap): State<SessionGap>,
    Path(id): Path<Uuid>,
    Query(query): Query<ListQuery>,
) -> ApiResult<Json<Neighbours>> {
    let (filter, sort) = filter_and_sort(&query).map_err(ApiError::Invalid)?;
    if filter.in_session == Some(false) && query.view.as_deref() == Some("suggestions") {
        let place = suggest::neighbours(&db, &actor, filter, gap, id).await?;
        return place.map(Json).ok_or(ApiError::NotFound);
    }
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
