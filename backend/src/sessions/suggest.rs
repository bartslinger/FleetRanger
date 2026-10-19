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
    flights::db::{self as flights, Filter, FlightSummary, Sort, SortKey},
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
    mut filter: Filter,
    gap: SessionGap,
    limit: usize,
    offset: usize,
) -> sqlx::Result<(Vec<Suggestion>, usize)> {
    filter.without_session = true;
    let oldest_first = Sort {
        key: SortKey::TakeoffAt,
        descending: false,
    };
    let scope = policy::flight_scope(actor);
    let flights = flights::list(db, &scope, &filter, &oldest_first, i64::MAX, 0).await?;
    let mut groups = group(flights, Duration::minutes(gap.0.into()));
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

/// A suggestion as it's being formed: what a flight must fit.
struct Open {
    flights: Vec<FlightSummary>,
    landed_at: DateTime<Utc>,
    team_id: Option<Uuid>,
    pilot_id: Option<Uuid>,
}

/// Groups flights given in takeoff order. A flight joins a group when it took
/// off less than `gap` after the group's last landing, and its team and pilot
/// don't differ from the group's; an unknown team or pilot fits any (user's
/// decision, 2026-10-19), so two teams flying the same evening make two
/// groups. Of several groups it fits, it joins the one with the most of its
/// known team and pilot, then the one that landed last.
fn group(flights: Vec<FlightSummary>, gap: Duration) -> Vec<Vec<FlightSummary>> {
    let fits = |group: Option<Uuid>, flight: Option<Uuid>| {
        group.is_none() || flight.is_none() || group == flight
    };
    let mut groups: Vec<Open> = Vec::new();
    for flight in flights {
        let joins = groups
            .iter_mut()
            .filter(|open| {
                flight.takeoff_at - open.landed_at < gap
                    && fits(open.team_id, flight.team_id)
                    && fits(open.pilot_id, flight.pilot_id)
            })
            .max_by_key(|open| {
                let same = |group: Option<Uuid>, flight: Option<Uuid>| {
                    usize::from(group.is_some() && group == flight)
                };
                let matches =
                    same(open.team_id, flight.team_id) + same(open.pilot_id, flight.pilot_id);
                (matches, open.landed_at)
            });
        match joins {
            Some(open) => {
                open.landed_at = open.landed_at.max(flight.landed_at);
                open.team_id = open.team_id.or(flight.team_id);
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

    /// Two teams flying the same evening stay apart, flights without a team
    /// or pilot join in, and a long break starts a new suggestion.
    #[test]
    fn teams_pilots_and_breaks_keep_suggestions_apart() {
        let (north, south) = (Some(Uuid::from_u128(1)), Some(Uuid::from_u128(2)));
        let (anna, ben) = (Some(Uuid::from_u128(3)), Some(Uuid::from_u128(4)));
        let flights = vec![
            flight(0, north, anna),
            flight(10, south, None),
            flight(30, None, anna),
            flight(40, south, None),
            flight(60, north, ben),
            // 20 minutes in the air, then more than two hours later.
            flight(60 + 20 + 121, north, anna),
        ];
        let groups = group(flights, Duration::minutes(120));
        assert_eq!(
            minutes(&groups),
            [vec![0, 30], vec![10, 40], vec![60], vec![201]]
        );
    }
}
