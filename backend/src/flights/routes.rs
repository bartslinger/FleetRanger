//! Flight API. Until login exists it is only served when
//! `FLEETRANGER_INSECURE_DEV_API` is set, and without any access checks.

use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::get,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::db::{self, FlightSummary, Sort, SortKey};

const DEFAULT_PAGE_SIZE: u32 = 50;
const MAX_PAGE_SIZE: u32 = 100;

pub fn router() -> Router<PgPool> {
    Router::new().route("/api/flights", get(list))
}

#[derive(Deserialize)]
struct ListQuery {
    /// 1-based.
    page: Option<u32>,
    size: Option<u32>,
    /// A column name, prefixed with `-` for descending order.
    sort: Option<String>,
}

#[derive(Serialize)]
struct FlightPage {
    flights: Vec<FlightSummary>,
    total: i64,
}

async fn list(
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

    let offset = i64::from(page - 1) * i64::from(size);
    let flights = db::list(&db, &sort, i64::from(size), offset).await;
    let total = db::count(&db).await;
    match (flights, total) {
        (Ok(flights), Ok(total)) => Ok(Json(FlightPage { flights, total })),
        (Err(error), _) | (_, Err(error)) => {
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
