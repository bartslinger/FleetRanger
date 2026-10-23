//! The tables are separate, and a table can't be a query parameter, so every
//! query is written once per kind.
//!
//! A device is seen with `equipment.read` on its group, or through one of its
//! flights; the flight condition is the one in the flight queries (see
//! flights/db.rs), keep it in step. Its figures count all its flights, also
//! those the user doesn't see: they describe the machine.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::policy::{FlightScope, Scope};

#[derive(Clone, Copy)]
pub enum Kind {
    Aircraft,
    RemoteController,
}

#[derive(Serialize)]
pub struct Equipment {
    pub id: Uuid,
    pub serial: String,
    pub name: Option<String>,
    pub team_id: Option<Uuid>,
    /// Its team's group, with the same id; decides who changes it.
    #[serde(skip)]
    pub group_id: Option<Uuid>,
    pub team_name: Option<String>,
    /// Over all its flights that aren't archived.
    pub flights: i64,
    pub air_time_s: i64,
    pub last_flight_at: Option<DateTime<Utc>>,
    /// Who flew its last flight; None without flights or when that flight has
    /// no pilot.
    pub last_pilot_name: Option<String>,
    /// Whether a controller is paired with the install; None for drones.
    pub paired: Option<bool>,
    /// When a controller last synced (also without logs); None for drones
    /// and controllers that never did.
    pub synced_at: Option<DateTime<Utc>>,
    /// The app version a controller reported at its last sync.
    pub app_version: Option<String>,
}

/// The ones of a kind someone sees, by name or else serial; `id` narrows it
/// to one.
///
/// - `read`: where they hold `equipment.read`.
/// - `flights`: the flights they see.
pub async fn list(
    db: &PgPool,
    kind: Kind,
    read: &Scope,
    flights: &FlightScope,
    id: Option<Uuid>,
) -> sqlx::Result<Vec<Equipment>> {
    match kind {
        Kind::Aircraft => {
            sqlx::query_as!(
                Equipment,
                r#"with flown as (
                       select aircraft_serial as serial, count(*) as flights,
                              sum(air_time_s)::bigint as air_time_s,
                              max(takeoff_at) as last_flight_at
                       from flights
                       where archived_at is null and aircraft_serial is not null
                       group by aircraft_serial
                   ), last as (
                       select distinct on (aircraft_serial) aircraft_serial as serial, pilot_id
                       from flights
                       where archived_at is null and aircraft_serial is not null
                       order by aircraft_serial, takeoff_at desc, id desc
                   )
                   select e.id, e.serial, e.name, e.team_id, e.group_id,
                          t.name as "team_name?",
                          coalesce(s.flights, 0) as "flights!",
                          coalesce(s.air_time_s, 0) as "air_time_s!",
                          s.last_flight_at as "last_flight_at?",
                          p.name as "last_pilot_name?",
                          null::bool as "paired?",
                          null::timestamptz as "synced_at?",
                          null::text as "app_version?"
                   from aircraft e
                   left join teams t on t.id = e.team_id
                   left join flown s on s.serial = e.serial
                   left join last l on l.serial = e.serial
                   left join users p on p.id = l.pilot_id
                   where ($1::uuid is null or e.id = $1)
                     and ($2 or e.group_id = any($3)
                          or exists (select 1 from flights f
                                     where f.aircraft_serial = e.serial
                                       and f.archived_at is null
                                       and ($4 or f.pilot_id = $5 or f.group_id = any($6))))
                   order by lower(coalesce(e.name, e.serial)), e.id"#,
                id,
                read.all,
                &read.groups,
                flights.all,
                flights.user_id,
                &flights.group_ids,
            )
            .fetch_all(db)
            .await
        }
        Kind::RemoteController => {
            sqlx::query_as!(
                Equipment,
                r#"with flown as (
                       select rc_serial as serial, count(*) as flights,
                              sum(air_time_s)::bigint as air_time_s,
                              max(takeoff_at) as last_flight_at
                       from flights
                       where archived_at is null and rc_serial is not null
                       group by rc_serial
                   ), last as (
                       select distinct on (rc_serial) rc_serial as serial, pilot_id
                       from flights
                       where archived_at is null and rc_serial is not null
                       order by rc_serial, takeoff_at desc, id desc
                   )
                   select e.id, e.serial, e.name, e.team_id, e.group_id,
                          t.name as "team_name?",
                          coalesce(s.flights, 0) as "flights!",
                          coalesce(s.air_time_s, 0) as "air_time_s!",
                          s.last_flight_at as "last_flight_at?",
                          p.name as "last_pilot_name?",
                          e.token_sha256 is not null as "paired?",
                          e.synced_at, e.app_version
                   from remote_controllers e
                   left join teams t on t.id = e.team_id
                   left join flown s on s.serial = e.serial
                   left join last l on l.serial = e.serial
                   left join users p on p.id = l.pilot_id
                   where ($1::uuid is null or e.id = $1)
                     and ($2 or e.group_id = any($3)
                          or exists (select 1 from flights f
                                     where f.rc_serial = e.serial
                                       and f.archived_at is null
                                       and ($4 or f.pilot_id = $5 or f.group_id = any($6))))
                   order by lower(coalesce(e.name, e.serial)), e.id"#,
                id,
                read.all,
                &read.groups,
                flights.all,
                flights.user_id,
                &flights.group_ids,
            )
            .fetch_all(db)
            .await
        }
    }
}

/// Sets the name, and the team and with it the group, which has the same id.
/// False if there is no such item.
pub async fn update(
    db: &PgPool,
    kind: Kind,
    id: Uuid,
    name: Option<&str>,
    team_id: Option<Uuid>,
) -> sqlx::Result<bool> {
    let result = match kind {
        Kind::Aircraft => {
            sqlx::query!(
                "update aircraft set name = $2, team_id = $3, group_id = $3 where id = $1",
                id,
                name,
                team_id,
            )
            .execute(db)
            .await?
        }
        Kind::RemoteController => {
            sqlx::query!(
                "update remote_controllers set name = $2, team_id = $3, group_id = $3
                 where id = $1",
                id,
                name,
                team_id,
            )
            .execute(db)
            .await?
        }
    };
    Ok(result.rows_affected() == 1)
}

pub async fn group_exists(db: &PgPool, group_id: Uuid) -> sqlx::Result<bool> {
    sqlx::query_scalar!(
        r#"select exists(select 1 from groups where id = $1) as "exists!""#,
        group_id,
    )
    .fetch_one(db)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{policy::Permission, testing};

    async fn id(db: &PgPool, sql: &'static str) -> Uuid {
        sqlx::query_scalar(sql).fetch_one(db).await.unwrap()
    }

    /// A flight of the drone for a group, by a pilot, taking off `hours` ago.
    async fn flight(db: &PgPool, serial: &str, group: Uuid, pilot: Uuid, hours: i32) {
        let raw_log = id(
            db,
            "insert into raw_logs (sha256, path, size_bytes)
             values (sha256(gen_random_uuid()::text::bytea), gen_random_uuid()::text, 0)
             returning id",
        )
        .await;
        sqlx::query(
            "insert into flights (raw_log_id, takeoff_at, landed_at, air_time_s, distance_m,
                                  track, aircraft_serial, team_id, group_id, pilot_id)
             values ($1, now() - make_interval(hours => $2), now(), 600, 0, '{}', $3, $4, $4, $5)",
        )
        .bind(raw_log)
        .bind(hours)
        .bind(serial)
        .bind(group)
        .bind(pilot)
        .execute(db)
        .await
        .unwrap();
    }

    /// A member sees their group's drones, and another group's drone once
    /// they see one of its flights; its figures count all its flights, also
    /// the other group's.
    #[sqlx::test]
    async fn equipment_is_seen_through_flights_with_all_its_figures(db: PgPool) {
        let north = testing::team(&db, "North").await;
        let south = testing::team(&db, "South").await;
        let me = id(&db, "insert into users (name) values ('A') returning id").await;
        let other = id(&db, "insert into users (name) values ('B') returning id").await;
        testing::join(&db, me, north).await;
        testing::join(&db, other, south).await;
        sqlx::query(
            "insert into aircraft (serial, team_id, group_id)
             values ('TESTOWN', $1, $1), ('TESTSHARED', $2, $2), ('TESTTHEIRS', $2, $2)",
        )
        .bind(north)
        .bind(south)
        .execute(&db)
        .await
        .unwrap();
        flight(&db, "TESTSHARED", north, me, 3).await;
        flight(&db, "TESTSHARED", south, other, 1).await;
        flight(&db, "TESTTHEIRS", south, other, 1).await;

        let grants = crate::auth::grants(&db, me).await.unwrap();
        let read = grants.scope(Permission::EquipmentRead);
        let seen = list(
            &db,
            Kind::Aircraft,
            &read,
            &testing::scope(&db, me).await,
            None,
        )
        .await
        .unwrap();
        let serials: Vec<&str> = seen.iter().map(|item| item.serial.as_str()).collect();
        assert_eq!(serials, ["TESTOWN", "TESTSHARED"]);
        let shared = &seen[1];
        assert_eq!((shared.flights, shared.air_time_s), (2, 1200));
        assert_eq!(shared.last_pilot_name.as_deref(), Some("B"));
    }
}
