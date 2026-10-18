use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

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

/// One page of flights in the given order.
pub async fn list(
    db: &PgPool,
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
    )
    .fetch_all(db)
    .await
}

pub async fn count(db: &PgPool) -> sqlx::Result<i64> {
    sqlx::query_scalar!(r#"select count(*) as "count!" from flights"#)
        .fetch_one(db)
        .await
}
