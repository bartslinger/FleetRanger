//! The processing status API, for admins.

use axum::{
    Json, Router,
    extract::{FromRef, State},
    routing::get,
};
use sqlx::PgPool;

use super::activity::Activity;
use super::service::{self, Status};
use crate::{auth::Actor, error::ApiResult};

pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    PgPool: FromRef<S>,
    Activity: FromRef<S>,
{
    Router::new().route("/api/logs/processing", get(status))
}

async fn status(
    actor: Actor,
    State(db): State<PgPool>,
    State(activity): State<Activity>,
) -> ApiResult<Json<Status>> {
    Ok(Json(service::status(&db, &actor, &activity).await?))
}
