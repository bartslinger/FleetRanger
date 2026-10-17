use axum::{extract::State, http::StatusCode};
use sqlx::PgPool;

/// Reports whether the server is up and can reach the database.
pub async fn check(State(db): State<PgPool>) -> (StatusCode, &'static str) {
    match sqlx::query("SELECT 1").execute(&db).await {
        Ok(_) => (StatusCode::OK, "ok"),
        Err(error) => {
            tracing::warn!(%error, "health check: database unreachable");
            (StatusCode::SERVICE_UNAVAILABLE, "database unreachable")
        }
    }
}
