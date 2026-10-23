use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use super::db::{self, SessionSummary, Window};
use crate::{
    auth::Actor,
    error::{ApiError, ApiResult},
    flights::{
        self,
        db::{Filter, FlightDetail},
    },
    policy,
};

/// One page of the sessions the actor may see that pass the filter, if given
/// only those with a flight in the window, and how many there are in all.
pub async fn list(
    db: &PgPool,
    actor: &Actor,
    filter: &Filter,
    window: Option<Window>,
    limit: i64,
    offset: i64,
    with_tracks: bool,
) -> ApiResult<(Vec<SessionSummary>, i64)> {
    let scope = policy::flight_scope(actor);
    let mut sessions = db::list(db, &scope, filter, window, limit, offset, with_tracks).await?;
    if with_tracks {
        for session in &mut sessions {
            simplify_tracks(&mut session.flight_times);
        }
    }
    let total = db::count(db, &scope, filter, window).await?;
    Ok((sessions, total))
}

/// About 5 m, in degrees: finer than a map of a week's or month's sessions
/// can show.
const PREVIEW_TOLERANCE: f64 = 0.00005;

/// Each flight's `track` (its coordinates) as a coarse [lon, lat] line, for
/// a map of many sessions at once: a few dozen points instead of thousands.
fn simplify_tracks(flights: &mut serde_json::Value) {
    use geo::{LineString, Simplify};
    let Some(flights) = flights.as_array_mut() else {
        return;
    };
    for flight in flights {
        let Some(track) = flight.get_mut("track") else {
            continue;
        };
        let points: Vec<(f64, f64)> = track
            .as_array()
            .map(|points| {
                points
                    .iter()
                    .filter_map(|point| Some((point.get(0)?.as_f64()?, point.get(1)?.as_f64()?)))
                    .collect()
            })
            .unwrap_or_default();
        let line = LineString::from(points).simplify(PREVIEW_TOLERANCE);
        // Six decimals, about 10 cm: the rest is only bytes.
        let round = |value: f64| (value * 1e6).round() / 1e6;
        *track = line
            .points()
            .map(|point| serde_json::json!([round(point.x()), round(point.y())]))
            .collect();
    }
}

/// A session's place in the list of those the actor may see that pass the
/// filter. Not found if it isn't in that list.
pub async fn neighbours(
    db: &PgPool,
    actor: &Actor,
    filter: &Filter,
    id: Uuid,
) -> ApiResult<db::Neighbours> {
    let scope = policy::flight_scope(actor);
    db::neighbours(db, &scope, filter, id)
        .await?
        .ok_or(ApiError::NotFound)
}

/// Puts these flights into a new session, and returns it. They leave the
/// sessions they were in.
pub async fn create(db: &PgPool, actor: &Actor, flight_ids: &[Uuid]) -> ApiResult<Uuid> {
    let flight_ids = distinct(flight_ids);
    if flight_ids.is_empty() {
        return Err(ApiError::Invalid("Select flights.".to_owned()));
    }
    let groups = flights::db::groups_of(db, &flight_ids)
        .await?
        .ok_or(ApiError::NotFound)?;
    policy::change_sessions(&actor.grants, &groups)?;
    not_archived(db, &flight_ids).await?;
    Ok(db::split(db, &flight_ids).await?)
}

/// Archived flights are never in a session.
async fn not_archived(db: &PgPool, flight_ids: &[Uuid]) -> ApiResult<()> {
    if flights::db::any_archived(db, flight_ids).await? {
        return Err(ApiError::Invalid(
            "Archived flights can’t be in a session. Restore them first.".to_owned(),
        ));
    }
    Ok(())
}

/// Adds these flights to a session. They leave the sessions they were in.
pub async fn add(db: &PgPool, actor: &Actor, session: Uuid, flight_ids: &[Uuid]) -> ApiResult<()> {
    let flight_ids = distinct(flight_ids);
    if flight_ids.is_empty() {
        return Err(ApiError::Invalid("Select flights.".to_owned()));
    }
    let mut groups = flights::db::groups_of(db, &flight_ids)
        .await?
        .ok_or(ApiError::NotFound)?;
    groups.extend(
        db::flight_groups(db, &[session])
            .await?
            .ok_or(ApiError::NotFound)?,
    );
    policy::change_sessions(&actor.grants, &groups)?;
    not_archived(db, &flight_ids).await?;
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
    let groups = flights::db::groups_of(db, &flight_ids)
        .await?
        .ok_or(ApiError::NotFound)?;
    policy::change_sessions(&actor.grants, &groups)?;
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
    flights::service::link_equipment(actor, &mut flights);
    Ok(Session { id, flights })
}

/// Merges sessions into the one that starts first, and returns it.
pub async fn merge(db: &PgPool, actor: &Actor, ids: &[Uuid]) -> ApiResult<Uuid> {
    let ids = distinct(ids);
    if ids.len() < 2 {
        return Err(ApiError::Invalid("Select two or more sessions.".to_owned()));
    }
    let groups = db::flight_groups(db, &ids)
        .await?
        .ok_or(ApiError::NotFound)?;
    policy::change_sessions(&actor.grants, &groups)?;
    Ok(db::merge(db, &ids).await?)
}

/// Moves flights of one session into a new session, and returns it.
pub async fn split(db: &PgPool, actor: &Actor, flight_ids: &[Uuid]) -> ApiResult<Uuid> {
    let flight_ids = distinct(flight_ids);
    let session = db::session_of(db, &flight_ids)
        .await?
        .ok_or_else(|| ApiError::Invalid("Select flights of one session.".to_owned()))?;
    let groups = db::flight_groups(db, &[session])
        .await?
        .ok_or(ApiError::NotFound)?;
    policy::change_sessions(&actor.grants, &groups)?;
    if flight_ids.len() == groups.len() {
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
