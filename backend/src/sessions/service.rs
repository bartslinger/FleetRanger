use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use super::db::{self, SessionSummary};
use crate::{
    auth::Actor,
    error::{ApiError, ApiResult},
    flights::{self, db::FlightDetail},
    policy,
};

/// One page of the sessions the actor may see, and how many there are in all.
pub async fn list(
    db: &PgPool,
    actor: &Actor,
    limit: i64,
    offset: i64,
) -> ApiResult<(Vec<SessionSummary>, i64)> {
    let scope = policy::flight_scope(actor);
    let sessions = db::list(db, &scope, limit, offset).await?;
    let total = db::count(db, &scope).await?;
    Ok((sessions, total))
}

/// A session with the flights of it the actor may see.
#[derive(Serialize)]
pub struct Session {
    pub id: Uuid,
    /// In takeoff order.
    pub flights: Vec<FlightDetail>,
}

/// A session the actor may see a flight of. Any other session is not found,
/// so its existence isn't revealed.
pub async fn get(db: &PgPool, actor: &Actor, id: Uuid) -> ApiResult<Session> {
    let scope = policy::flight_scope(actor);
    let flights = flights::db::of_session(db, &scope, id).await?;
    if flights.is_empty() {
        return Err(ApiError::NotFound);
    }
    Ok(Session { id, flights })
}
