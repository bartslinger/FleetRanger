//! Suggested sessions: the flights without a session, grouped for someone to
//! review and turn into sessions. Nothing is stored; they're worked out anew
//! from the flights each time (user's decision, 2026-10-19).

use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use super::SessionGap;
use crate::{
    auth::Actor,
    flights::{
        db::{self as flights, Filter, FlightDetail, FlightSummary, Neighbours, Sort, SortKey},
        service::link_equipment,
    },
    policy,
};

/// Flights that look like one session.
#[derive(Serialize)]
pub struct Suggestion {
    /// The first flight's id: the same suggestion keeps it while its flights stay.
    pub key: Uuid,
    /// In takeoff order.
    pub flights: Vec<FlightSummary>,
}

/// One page of the suggestions for the flights without a session that the
/// actor may see and that pass the filter, newest first, and how many there
/// are in all.
pub async fn list(
    db: &PgPool,
    actor: &Actor,
    filter: Filter,
    gap: SessionGap,
    limit: usize,
    offset: usize,
) -> sqlx::Result<(Vec<Suggestion>, usize)> {
    let mut groups = all(db, actor, filter, gap).await?;
    let total = groups.len();
    groups.reverse();
    let page = groups
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(|flights| Suggestion {
            key: flights[0].id,
            flights,
        })
        .collect();
    Ok((page, total))
}

/// One suggestion with its flights' tracks, for its own page, and its place
/// in the list (newest first), for stepping from one to the next.
#[derive(Serialize)]
pub struct SuggestionPage {
    pub key: Uuid,
    /// In takeoff order.
    pub flights: Vec<FlightDetail>,
    /// The newer and older suggestions' keys.
    pub previous: Option<Uuid>,
    pub next: Option<Uuid>,
    /// From 1, newest first.
    pub position: usize,
    pub total: usize,
}

/// The suggestion with the flight `key` in it, or None if that flight is in
/// none (it's in a session by now, or doesn't pass the filter). A suggestion
/// is found by any of its flights, as it may have changed since the link was
/// made.
pub async fn get(
    db: &PgPool,
    actor: &Actor,
    filter: Filter,
    gap: SessionGap,
    key: Uuid,
) -> sqlx::Result<Option<SuggestionPage>> {
    let mut groups = all(db, actor, filter, gap).await?;
    groups.reverse();
    let Some(index) = groups
        .iter()
        .position(|flights| flights.iter().any(|flight| flight.id == key))
    else {
        return Ok(None);
    };
    let key_at = |index: Option<usize>| {
        index
            .and_then(|index| groups.get(index))
            .map(|flights| flights[0].id)
    };
    let ids: Vec<Uuid> = groups[index].iter().map(|flight| flight.id).collect();
    let mut flights = flights::details(db, &policy::flight_scope(actor), &ids).await?;
    link_equipment(db, actor, &mut flights).await?;
    Ok(Some(SuggestionPage {
        key: ids[0],
        flights,
        previous: key_at(index.checked_sub(1)),
        next: key_at(Some(index + 1)),
        position: index + 1,
        total: groups.len(),
    }))
}

/// A flight's place in its suggestion, for stepping through that suggestion's
/// flights in takeoff order; None if the flight isn't in one.
pub async fn neighbours(
    db: &PgPool,
    actor: &Actor,
    filter: Filter,
    gap: SessionGap,
    id: Uuid,
) -> sqlx::Result<Option<Neighbours>> {
    let groups = all(db, actor, filter, gap).await?;
    let Some(flights) = groups
        .into_iter()
        .find(|flights| flights.iter().any(|flight| flight.id == id))
    else {
        return Ok(None);
    };
    let index = flights
        .iter()
        .position(|flight| flight.id == id)
        .expect("the flight is in its suggestion");
    let id_at = |index: Option<usize>| index.and_then(|index| flights.get(index)).map(|f| f.id);
    Ok(Some(Neighbours {
        previous: id_at(index.checked_sub(1)),
        next: id_at(Some(index + 1)),
        position: i64::try_from(index + 1).unwrap_or(i64::MAX),
        total: i64::try_from(flights.len()).unwrap_or(i64::MAX),
    }))
}

/// Every suggestion for the flights without a session that the actor may see
/// and that pass the filter, oldest first.
async fn all(
    db: &PgPool,
    actor: &Actor,
    mut filter: Filter,
    gap: SessionGap,
) -> sqlx::Result<Vec<Vec<FlightSummary>>> {
    filter.without_session = true;
    let oldest_first = Sort {
        key: SortKey::TakeoffAt,
        descending: false,
    };
    let scope = policy::flight_scope(actor);
    let flights = flights::list(db, &scope, &filter, &oldest_first, i64::MAX, 0).await?;
    Ok(group(flights, Duration::minutes(gap.0.into())))
}

/// A suggestion as it's being formed: what a flight must fit.
struct Open {
    flights: Vec<FlightSummary>,
    landed_at: DateTime<Utc>,
    team_id: Option<Uuid>,
    pilot_id: Option<Uuid>,
}

/// Groups flights given in takeoff order. A flight joins a group when it took
/// off less than `gap` after the group's last landing, it has the group's
/// team, and its pilot doesn't differ from the group's (user's decisions,
/// 2026-10-19). Flights without a team are a team of their own: they're often
/// another team's, still to be sorted out. An unknown pilot fits any. So two
/// teams flying the same evening make two groups. Of several groups it fits,
/// it joins the one with its known pilot, then the one that landed last.
fn group(flights: Vec<FlightSummary>, gap: Duration) -> Vec<Vec<FlightSummary>> {
    let mut groups: Vec<Open> = Vec::new();
    for flight in flights {
        let joins = groups
            .iter_mut()
            .filter(|open| {
                flight.takeoff_at - open.landed_at < gap
                    && open.team_id == flight.team_id
                    && (open.pilot_id.is_none()
                        || flight.pilot_id.is_none()
                        || open.pilot_id == flight.pilot_id)
            })
            .max_by_key(|open| {
                let same_pilot = open.pilot_id.is_some() && open.pilot_id == flight.pilot_id;
                (same_pilot, open.landed_at)
            });
        match joins {
            Some(open) => {
                open.landed_at = open.landed_at.max(flight.landed_at);
                open.pilot_id = open.pilot_id.or(flight.pilot_id);
                open.flights.push(flight);
            }
            None => groups.push(Open {
                landed_at: flight.landed_at,
                team_id: flight.team_id,
                pilot_id: flight.pilot_id,
                flights: vec![flight],
            }),
        }
    }
    groups.into_iter().map(|open| open.flights).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flight(minute: i64, team: Option<Uuid>, pilot: Option<Uuid>) -> FlightSummary {
        let takeoff_at = DateTime::UNIX_EPOCH + Duration::minutes(minute);
        FlightSummary {
            id: Uuid::from_u128(minute.unsigned_abs().into()),
            takeoff_at,
            landed_at: takeoff_at + Duration::minutes(20),
            air_time_s: 1200,
            distance_m: 0.0,
            aircraft_serial: None,
            aircraft_name: None,
            rc_serial: None,
            rc_name: None,
            pilot_id: pilot,
            pilot_name: None,
            team_id: team,
            team_name: None,
        }
    }

    fn minutes(groups: &[Vec<FlightSummary>]) -> Vec<Vec<i64>> {
        groups
            .iter()
            .map(|group| {
                group
                    .iter()
                    .map(|flight| (flight.takeoff_at - DateTime::UNIX_EPOCH).num_minutes())
                    .collect()
            })
            .collect()
    }

    /// Two teams flying the same evening stay apart, and so do flights
    /// without a team; flights without a pilot join in, another pilot
    /// doesn't, and a long break starts a new suggestion.
    #[test]
    fn teams_pilots_and_breaks_keep_suggestions_apart() {
        let (north, south) = (Some(Uuid::from_u128(1)), Some(Uuid::from_u128(2)));
        let (anna, ben) = (Some(Uuid::from_u128(3)), Some(Uuid::from_u128(4)));
        let flights = vec![
            flight(0, north, anna),
            flight(10, south, None),
            flight(30, None, anna),
            flight(40, south, None),
            flight(45, north, None),
            flight(50, None, None),
            flight(60, north, ben),
            // 20 minutes in the air, then more than two hours later.
            flight(60 + 20 + 121, north, anna),
        ];
        let groups = group(flights, Duration::minutes(120));
        assert_eq!(
            minutes(&groups),
            [vec![0, 45], vec![10, 40], vec![30, 50], vec![60], vec![201]]
        );
    }
}
