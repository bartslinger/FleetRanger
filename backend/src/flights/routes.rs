//! Flight API. Each account sees only the flights the policy allows.

use axum::{
    Json, Router,
    extract::{FromRef, Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use super::db::{Filter, FlightDetail, FlightSummary, Sort, SortKey};
use super::service;
use crate::{auth::Actor, error::ApiResult};

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
        .route("/api/flights/{id}", get(show))
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

async fn devices(actor: Actor, State(db): State<PgPool>) -> ApiResult<Json<service::Devices>> {
    Ok(Json(service::devices(&db, &actor).await?))
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
    /// Only the flights of this drone serial.
    aircraft: Option<String>,
    /// Only the flights of this controller serial.
    controller: Option<String>,
    /// `none`: only flights in no session.
    session: Option<String>,
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
    let sort = parse_sort(query.sort.as_deref().unwrap_or("-takeoff_at"))
        .ok_or_else(|| bad_request("unknown sort column".to_owned()))?;

    let without_session = match query.session.as_deref() {
        None => false,
        Some("none") => true,
        Some(_) => return Err(bad_request("session must be none".to_owned())),
    };
    let filter = Filter {
        aircraft_serial: query.aircraft,
        rc_serial: query.controller,
        without_session,
    };
    let offset = i64::from(page - 1) * i64::from(size);
    match service::list(&db, &actor, &filter, &sort, i64::from(size), offset).await {
        Ok((flights, total)) => Ok(Json(FlightPage { flights, total })),
        Err(error) => {
            tracing::error!(%error, "listing flights");
            Err((StatusCode::INTERNAL_SERVER_ERROR, String::new()))
        }
    }
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
