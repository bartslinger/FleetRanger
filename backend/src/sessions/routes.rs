//! Session API. A session is visible when one of its flights is, and shows
//! only the flights the policy allows.

use axum::{
    Json, Router,
    extract::{FromRef, Path, Query, State},
    routing::get,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use super::db::SessionSummary;
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
        .route("/api/sessions", get(list))
        .route("/api/sessions/{id}", get(show))
}

#[derive(Deserialize)]
struct ListQuery {
    /// 1-based.
    page: Option<u32>,
    size: Option<u32>,
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
    let offset = i64::from(page - 1) * i64::from(size);
    let (sessions, total) = service::list(&db, &actor, i64::from(size), offset).await?;
    Ok(Json(SessionPage { sessions, total }))
}

async fn show(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Session>> {
    Ok(Json(service::get(&db, &actor, id).await?))
}
