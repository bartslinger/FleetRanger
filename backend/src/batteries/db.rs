//! A battery is seen through flights: whoever sees one of its flights sees the
//! battery, with the flights they see. Its figures (flights, cycles, capacity)
//! count all its flights: they describe the battery's wear. The visibility
//! condition is the one in the flight queries (see flights/db.rs); keep it in
//! step.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::policy::FlightScope;

/// A battery with its state at its latest flight.
#[derive(Serialize)]
pub struct Battery {
    pub id: Uuid,
    pub serial: String,
    pub name: Option<String>,
    pub flights: i64,
    pub last_flight_at: DateTime<Utc>,
    pub aircraft_id: Option<Uuid>,
    pub aircraft_serial: Option<String>,
    pub aircraft_name: Option<String>,
    pub cycles: Option<i32>,
    pub design_capacity_mah: Option<i32>,
    /// What it held at its latest flight that reported it.
    pub full_capacity_mah: Option<i32>,
}

/// One flight of a battery.
#[derive(Serialize)]
pub struct BatteryFlight {
    pub flight_id: Uuid,
    pub takeoff_at: DateTime<Utc>,
    pub air_time_s: i32,
    pub aircraft_serial: Option<String>,
    pub aircraft_name: Option<String>,
    pub charge_start_pct: Option<i16>,
    pub charge_end_pct: Option<i16>,
    pub full_capacity_mah: Option<i32>,
    pub design_capacity_mah: Option<i32>,
    pub cycles: Option<i32>,
    pub temperature_min_c: Option<f32>,
    pub temperature_max_c: Option<f32>,
    pub cell_voltage_min_v: Option<f32>,
    pub cell_spread_max_v: Option<f32>,
}

/// The batteries with a flight in the scope, by name or else serial; `id`
/// narrows it to one.
pub async fn list(
    db: &PgPool,
    scope: &FlightScope,
    id: Option<Uuid>,
) -> sqlx::Result<Vec<Battery>> {
    sqlx::query_as!(
        Battery,
        r#"with flown as (
               select fb.*, f.takeoff_at, f.aircraft_serial, f.pilot_id, f.group_id
               from flight_batteries fb
               join flights f on f.id = fb.flight_id
               where f.archived_at is null
           )
           select b.id, b.serial, b.name,
                  count(*) as "flights!",
                  max(s.takeoff_at) as "last_flight_at!",
                  a.id as "aircraft_id?", latest.aircraft_serial as "aircraft_serial?",
                  a.name as "aircraft_name?",
                  latest.cycles, latest.design_capacity_mah,
                  (select full_capacity_mah from flown
                   where battery_id = b.id and full_capacity_mah is not null
                   order by takeoff_at desc limit 1) as "full_capacity_mah?"
           from batteries b
           join flown s on s.battery_id = b.id
           cross join lateral (
               select aircraft_serial, cycles, design_capacity_mah from flown
               where battery_id = b.id
               order by takeoff_at desc limit 1
           ) latest
           left join aircraft a on a.serial = latest.aircraft_serial
           where ($3::uuid is null or b.id = $3)
             and exists (select 1 from flown v
                         where v.battery_id = b.id
                           and ($1
                                or v.pilot_id = $2
                                or v.group_id = any($4)))
           group by b.id, latest.aircraft_serial, latest.cycles, latest.design_capacity_mah,
                    a.id, a.name
           order by lower(coalesce(b.name, b.serial)), b.id"#,
        scope.all,
        scope.user_id,
        id,
        &scope.group_ids,
    )
    .fetch_all(db)
    .await
}

/// A battery's flights in the scope, newest first.
pub async fn flights(
    db: &PgPool,
    scope: &FlightScope,
    id: Uuid,
) -> sqlx::Result<Vec<BatteryFlight>> {
    sqlx::query_as!(
        BatteryFlight,
        r#"select f.id as flight_id, f.takeoff_at, f.air_time_s, f.aircraft_serial,
                  (select a.name from aircraft a where a.serial = f.aircraft_serial)
                      as "aircraft_name?",
                  fb.charge_start_pct, fb.charge_end_pct, fb.full_capacity_mah,
                  fb.design_capacity_mah, fb.cycles, fb.temperature_min_c, fb.temperature_max_c,
                  fb.cell_voltage_min_v, fb.cell_spread_max_v
           from flight_batteries fb
           join flights f on f.id = fb.flight_id
           where fb.battery_id = $3 and f.archived_at is null
             and ($1
                  or f.pilot_id = $2
                  or f.group_id = any($4))
           order by f.takeoff_at desc, f.id"#,
        scope.all,
        scope.user_id,
        id,
        &scope.group_ids,
    )
    .fetch_all(db)
    .await
}

/// What the battery held at a flight, for the chart over all its flights;
/// nothing else about the flight, which the user may not see.
#[derive(Serialize)]
pub struct CapacityReading {
    pub takeoff_at: DateTime<Utc>,
    pub full_capacity_mah: i32,
}

/// The full charge capacity at each of a battery's flights that reported it,
/// oldest first.
pub async fn capacity_readings(db: &PgPool, id: Uuid) -> sqlx::Result<Vec<CapacityReading>> {
    sqlx::query_as!(
        CapacityReading,
        r#"select f.takeoff_at, fb.full_capacity_mah as "full_capacity_mah!"
           from flight_batteries fb
           join flights f on f.id = fb.flight_id
           where fb.battery_id = $1 and f.archived_at is null
             and fb.full_capacity_mah is not null
           order by f.takeoff_at, f.id"#,
        id,
    )
    .fetch_all(db)
    .await
}

/// Returns false if there is no such battery.
pub async fn rename(db: &PgPool, id: Uuid, name: Option<&str>) -> sqlx::Result<bool> {
    let result = sqlx::query!("update batteries set name = $2 where id = $1", id, name)
        .execute(db)
        .await?;
    Ok(result.rows_affected() == 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn id(db: &PgPool, sql: &'static str) -> Uuid {
        sqlx::query_scalar(sql).fetch_one(db).await.unwrap()
    }

    /// A flight of the team with the battery, taking off `hours` ago.
    async fn flight(db: &PgPool, team: Uuid, battery: Uuid, hours: i32, cycles: i32) -> Uuid {
        let raw_log = id(
            db,
            "insert into raw_logs (sha256, path, size_bytes)
             values (sha256(gen_random_uuid()::text::bytea), gen_random_uuid()::text, 0)
             returning id",
        )
        .await;
        let flight: Uuid = sqlx::query_scalar(
            "insert into flights (raw_log_id, takeoff_at, landed_at, air_time_s, distance_m,
                                  track, group_id)
             values ($1, now() - make_interval(hours => $2), now(), 0, 0, '{}', $3)
             returning id",
        )
        .bind(raw_log)
        .bind(hours)
        .bind(team)
        .fetch_one(db)
        .await
        .unwrap();
        sqlx::query(
            "insert into flight_batteries (flight_id, slot, battery_id, cycles)
             values ($1, 0, $2, $3)",
        )
        .bind(flight)
        .bind(battery)
        .bind(cycles)
        .execute(db)
        .await
        .unwrap();
        flight
    }

    /// A member sees a battery through their team's flights only, with only
    /// those flights, but its figures over all its flights.
    #[sqlx::test]
    async fn members_see_batteries_through_their_teams_flights(db: PgPool) {
        let north = crate::testing::group(&db, "North").await;
        let south = crate::testing::group(&db, "South").await;
        let me = id(&db, "insert into users (name) values ('A') returning id").await;
        crate::testing::join(&db, me, north).await;
        let shared = id(
            &db,
            "insert into batteries (serial) values ('TESTB1') returning id",
        )
        .await;
        let theirs = id(
            &db,
            "insert into batteries (serial) values ('TESTB2') returning id",
        )
        .await;
        let ours = flight(&db, north, shared, 3, 10).await;
        flight(&db, south, shared, 1, 11).await;
        flight(&db, south, theirs, 1, 5).await;

        let member = crate::testing::scope(&db, me).await;
        let seen = list(&db, &member, None).await.unwrap();
        assert_eq!(seen.len(), 1);
        assert_eq!(
            (seen[0].id, seen[0].flights, seen[0].cycles),
            (shared, 2, Some(11))
        );
        let flights: Vec<Uuid> = flights(&db, &member, shared)
            .await
            .unwrap()
            .into_iter()
            .map(|flight| flight.flight_id)
            .collect();
        assert_eq!(flights, [ours]);
        assert!(list(&db, &member, Some(theirs)).await.unwrap().is_empty());

        let admin = FlightScope {
            all: true,
            user_id: me,
            group_ids: Vec::new(),
        };
        let all = list(&db, &admin, Some(shared)).await.unwrap();
        assert_eq!((all[0].flights, all[0].cycles), (2, Some(11)));
    }
}
