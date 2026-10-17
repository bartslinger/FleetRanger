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

/// All flights, latest first.
pub async fn list(db: &PgPool) -> sqlx::Result<Vec<FlightSummary>> {
    sqlx::query_as!(
        FlightSummary,
        "select id, takeoff_at, landed_at, air_time_s, distance_m, aircraft_serial, rc_serial
         from flights
         order by takeoff_at desc"
    )
    .fetch_all(db)
    .await
}
