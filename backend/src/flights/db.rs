use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::policy::FlightScope;

/// A flight as listed in the logbook.
#[derive(Serialize)]
pub struct FlightSummary {
    pub id: Uuid,
    pub takeoff_at: DateTime<Utc>,
    pub landed_at: DateTime<Utc>,
    pub air_time_s: i32,
    pub distance_m: f64,
    pub aircraft_serial: Option<String>,
    pub rc_serial: Option<String>,
}

/// Columns the logbook can be sorted by.
#[derive(Clone, Copy)]
pub enum SortKey {
    TakeoffAt,
    AirTime,
    Distance,
    AircraftSerial,
    RcSerial,
}

impl SortKey {
    /// From the column name used in the API.
    pub fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "takeoff_at" => Self::TakeoffAt,
            "air_time_s" => Self::AirTime,
            "distance_m" => Self::Distance,
            "aircraft_serial" => Self::AircraftSerial,
            "rc_serial" => Self::RcSerial,
            _ => return None,
        })
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::TakeoffAt => "takeoff_at",
            Self::AirTime => "air_time_s",
            Self::Distance => "distance_m",
            Self::AircraftSerial => "aircraft_serial",
            Self::RcSerial => "rc_serial",
        }
    }
}

pub struct Sort {
    pub key: SortKey,
    pub descending: bool,
}

// The visibility condition is the same in every flight query: the scope sees
// every flight ($all), or those of its account's pilot and of that pilot's team.
// Keep the copies in step; the tests check them against each other.

/// One page of the flights in the scope, in the given order.
pub async fn list(
    db: &PgPool,
    scope: &FlightScope,
    sort: &Sort,
    limit: i64,
    offset: i64,
) -> sqlx::Result<Vec<FlightSummary>> {
    // A column can't be a query parameter, so each sortable column has a CASE
    // pair; only the one matching $1 and $2 is non-null. The id makes the order
    // stable for equal values.
    sqlx::query_as!(
        FlightSummary,
        "select id, takeoff_at, landed_at, air_time_s, distance_m, aircraft_serial, rc_serial
         from flights
         where $5
            or pilot_id in (select id from pilots where account_id = $6)
            or team_id in (select m.team_id from team_members m
                           join pilots p on p.id = m.pilot_id where p.account_id = $6)
         order by
             case when $1 = 'takeoff_at' and not $2 then takeoff_at end asc,
             case when $1 = 'takeoff_at' and $2 then takeoff_at end desc,
             case when $1 = 'air_time_s' and not $2 then air_time_s end asc,
             case when $1 = 'air_time_s' and $2 then air_time_s end desc,
             case when $1 = 'distance_m' and not $2 then distance_m end asc,
             case when $1 = 'distance_m' and $2 then distance_m end desc,
             case when $1 = 'aircraft_serial' and not $2 then aircraft_serial end asc nulls last,
             case when $1 = 'aircraft_serial' and $2 then aircraft_serial end desc nulls last,
             case when $1 = 'rc_serial' and not $2 then rc_serial end asc nulls last,
             case when $1 = 'rc_serial' and $2 then rc_serial end desc nulls last,
             id
         limit $3 offset $4",
        sort.key.as_str(),
        sort.descending,
        limit,
        offset,
        scope.all,
        scope.account_id,
    )
    .fetch_all(db)
    .await
}

/// The number of flights in the scope.
pub async fn count(db: &PgPool, scope: &FlightScope) -> sqlx::Result<i64> {
    sqlx::query_scalar!(
        r#"select count(*) as "count!"
           from flights
           where $1
              or pilot_id in (select id from pilots where account_id = $2)
              or team_id in (select m.team_id from team_members m
                             join pilots p on p.id = m.pilot_id where p.account_id = $2)"#,
        scope.all,
        scope.account_id,
    )
    .fetch_one(db)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Adds a flight with the given team and pilot, returning its id.
    async fn flight(db: &PgPool, team: Option<Uuid>, pilot: Option<Uuid>) -> Uuid {
        let raw_log: Uuid = sqlx::query_scalar(
            "insert into raw_logs (sha256, path, size_bytes)
             values (sha256(gen_random_uuid()::text::bytea), gen_random_uuid()::text, 0)
             returning id",
        )
        .fetch_one(db)
        .await
        .unwrap();
        sqlx::query_scalar(
            "insert into flights (raw_log_id, takeoff_at, landed_at, air_time_s, distance_m,
                                  track, team_id, pilot_id)
             values ($1, now(), now(), 0, 0, '{}', $2, $3)
             returning id",
        )
        .bind(raw_log)
        .bind(team)
        .bind(pilot)
        .fetch_one(db)
        .await
        .unwrap()
    }

    async fn insert_id(db: &PgPool, sql: &'static str) -> Uuid {
        sqlx::query_scalar(sql).fetch_one(db).await.unwrap()
    }

    async fn visible(db: &PgPool, scope: &FlightScope) -> Vec<Uuid> {
        let sort = Sort {
            key: SortKey::TakeoffAt,
            descending: false,
        };
        let mut ids: Vec<Uuid> = list(db, scope, &sort, 100, 0)
            .await
            .unwrap()
            .into_iter()
            .map(|flight| flight.id)
            .collect();
        assert_eq!(count(db, scope).await.unwrap(), ids.len() as i64);
        ids.sort();
        ids
    }

    /// A member sees their own pilot's flights, also from an earlier team, and
    /// all flights of their pilot's current team; not other teams or the queue.
    #[sqlx::test]
    async fn member_sees_own_and_team_flights(db: PgPool) {
        let account = insert_id(
            &db,
            "insert into accounts (email) values ('a@example.org') returning id",
        )
        .await;
        let north = insert_id(
            &db,
            "insert into teams (name) values ('North') returning id",
        )
        .await;
        let south = insert_id(
            &db,
            "insert into teams (name) values ('South') returning id",
        )
        .await;
        let me: Uuid = sqlx::query_scalar(
            "insert into pilots (name, account_id) values ('A', $1) returning id",
        )
        .bind(account)
        .fetch_one(&db)
        .await
        .unwrap();
        let teammate = insert_id(&db, "insert into pilots (name) values ('B') returning id").await;
        let other = insert_id(&db, "insert into pilots (name) values ('C') returning id").await;
        for (pilot, team) in [(me, north), (teammate, north), (other, south)] {
            sqlx::query("insert into team_members (pilot_id, team_id) values ($1, $2)")
                .bind(pilot)
                .bind(team)
                .execute(&db)
                .await
                .unwrap();
        }

        let mut expected = vec![
            flight(&db, Some(north), Some(me)).await,
            flight(&db, Some(north), Some(teammate)).await,
            flight(&db, Some(north), None).await,
            // Flown for South before moving to North.
            flight(&db, Some(south), Some(me)).await,
        ];
        flight(&db, Some(south), Some(other)).await;
        flight(&db, Some(south), None).await;
        flight(&db, None, None).await;
        expected.sort();

        let member = FlightScope {
            all: false,
            account_id: account,
        };
        assert_eq!(visible(&db, &member).await, expected);
        let everything = FlightScope {
            all: true,
            account_id: account,
        };
        assert_eq!(visible(&db, &everything).await.len(), 7);
    }

    /// An account without a pilot sees no flights unless it sees all.
    #[sqlx::test]
    async fn account_without_pilot_sees_nothing(db: PgPool) {
        let account = insert_id(
            &db,
            "insert into accounts (email) values ('a@example.org') returning id",
        )
        .await;
        let team = insert_id(
            &db,
            "insert into teams (name) values ('North') returning id",
        )
        .await;
        flight(&db, Some(team), None).await;
        flight(&db, None, None).await;
        let member = FlightScope {
            all: false,
            account_id: account,
        };
        assert!(visible(&db, &member).await.is_empty());
    }
}
