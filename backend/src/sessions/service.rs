use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use super::db::{self, SessionSummary, Window};
use crate::{
    auth::Actor,
    error::{ApiError, ApiResult},
    flights::{self, db::FlightDetail},
    policy,
};

/// One page of the sessions the actor may see, if given only those with a
/// flight in the window, and how many there are in all.
pub async fn list(
    db: &PgPool,
    actor: &Actor,
    window: Option<Window>,
    limit: i64,
    offset: i64,
) -> ApiResult<(Vec<SessionSummary>, i64)> {
    let scope = policy::flight_scope(actor);
    let sessions = db::list(db, &scope, window, limit, offset).await?;
    let total = db::count(db, &scope, window).await?;
    Ok((sessions, total))
}

/// Puts these flights into a new session, and returns it. They leave the
/// sessions they were in.
pub async fn create(db: &PgPool, actor: &Actor, flight_ids: &[Uuid]) -> ApiResult<Uuid> {
    let flight_ids = distinct(flight_ids);
    if flight_ids.is_empty() {
        return Err(ApiError::Invalid("Select flights.".to_owned()));
    }
    let teams = flights::db::teams_of(db, &flight_ids)
        .await?
        .ok_or(ApiError::NotFound)?;
    let own_team = flights::db::own_team(db, actor.user_id).await?;
    policy::change_sessions(actor.role, own_team, &teams)?;
    Ok(db::split(db, &flight_ids).await?)
}

/// Adds these flights to a session. They leave the sessions they were in.
pub async fn add(db: &PgPool, actor: &Actor, session: Uuid, flight_ids: &[Uuid]) -> ApiResult<()> {
    let flight_ids = distinct(flight_ids);
    if flight_ids.is_empty() {
        return Err(ApiError::Invalid("Select flights.".to_owned()));
    }
    let mut teams = flights::db::teams_of(db, &flight_ids)
        .await?
        .ok_or(ApiError::NotFound)?;
    teams.extend(
        db::flight_teams(db, &[session])
            .await?
            .ok_or(ApiError::NotFound)?,
    );
    let own_team = flights::db::own_team(db, actor.user_id).await?;
    policy::change_sessions(actor.role, own_team, &teams)?;
    Ok(db::add(db, session, &flight_ids).await?)
}

/// Takes these flights out of their sessions; they stay flights, without a
/// session. A session left without flights is deleted. Returns how many were
/// in a session.
pub async fn remove(db: &PgPool, actor: &Actor, flight_ids: &[Uuid]) -> ApiResult<u64> {
    let flight_ids = distinct(flight_ids);
    if flight_ids.is_empty() {
        return Err(ApiError::Invalid("Select flights.".to_owned()));
    }
    let teams = flights::db::teams_of(db, &flight_ids)
        .await?
        .ok_or(ApiError::NotFound)?;
    let own_team = flights::db::own_team(db, actor.user_id).await?;
    policy::change_sessions(actor.role, own_team, &teams)?;
    Ok(db::remove(db, &flight_ids).await?)
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
    let mut flights = flights::db::of_session(db, &scope, id).await?;
    if flights.is_empty() {
        return Err(ApiError::NotFound);
    }
    flights::service::link_equipment(db, actor, &mut flights).await?;
    Ok(Session { id, flights })
}

/// Merges sessions into the one that starts first, and returns it.
pub async fn merge(db: &PgPool, actor: &Actor, ids: &[Uuid]) -> ApiResult<Uuid> {
    let ids = distinct(ids);
    if ids.len() < 2 {
        return Err(ApiError::Invalid("Select two or more sessions.".to_owned()));
    }
    let teams = db::flight_teams(db, &ids)
        .await?
        .ok_or(ApiError::NotFound)?;
    let own_team = flights::db::own_team(db, actor.user_id).await?;
    policy::change_sessions(actor.role, own_team, &teams)?;
    Ok(db::merge(db, &ids).await?)
}

/// Moves flights of one session into a new session, and returns it.
pub async fn split(db: &PgPool, actor: &Actor, flight_ids: &[Uuid]) -> ApiResult<Uuid> {
    let flight_ids = distinct(flight_ids);
    let session = db::session_of(db, &flight_ids)
        .await?
        .ok_or_else(|| ApiError::Invalid("Select flights of one session.".to_owned()))?;
    let teams = db::flight_teams(db, &[session])
        .await?
        .ok_or(ApiError::NotFound)?;
    let own_team = flights::db::own_team(db, actor.user_id).await?;
    policy::change_sessions(actor.role, own_team, &teams)?;
    if flight_ids.len() == teams.len() {
        return Err(ApiError::Invalid(
            "Leave at least one flight in the session.".to_owned(),
        ));
    }
    Ok(db::split(db, &flight_ids).await?)
}

fn distinct(ids: &[Uuid]) -> Vec<Uuid> {
    let mut ids = ids.to_vec();
    ids.sort_unstable();
    ids.dedup();
    ids
}
