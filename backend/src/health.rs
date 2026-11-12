use crate::db::Db;
use axum::http::StatusCode;

/// Reports whether the server is up and can reach the database.
pub async fn check(mut db: Db) -> (StatusCode, &'static str) {
    match sqlx::query("SELECT 1").execute(&mut *db).await {
        Ok(_) => (StatusCode::OK, "ok"),
        Err(error) => {
            tracing::warn!(%error, "health check: database unreachable");
            (StatusCode::SERVICE_UNAVAILABLE, "database unreachable")
        }
    }
}
