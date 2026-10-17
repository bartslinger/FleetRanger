//! Flight API. Until login exists it is only served when
//! `FLEETRANGER_INSECURE_DEV_API` is set, and without any access checks.

use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use sqlx::PgPool;

use super::db::{self, FlightSummary};

pub fn router() -> Router<PgPool> {
    Router::new().route("/api/flights", get(list))
}

async fn list(State(db): State<PgPool>) -> Result<Json<Vec<FlightSummary>>, StatusCode> {
    db::list(&db).await.map(Json).map_err(|error| {
        tracing::error!(%error, "listing flights");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}
