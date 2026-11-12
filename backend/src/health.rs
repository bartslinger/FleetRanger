use axum::{extract::State, http::StatusCode};

use crate::db::System;

/// Reports whether the server is up and can reach the database. Without a
/// login, so in no organisation.
pub async fn check(State(system): State<System>) -> (StatusCode, &'static str) {
    match sqlx::query("SELECT 1").execute(&*system).await {
        Ok(_) => (StatusCode::OK, "ok"),
        Err(error) => {
            tracing::warn!(%error, "health check: database unreachable");
            (StatusCode::SERVICE_UNAVAILABLE, "database unreachable")
        }
    }
}
