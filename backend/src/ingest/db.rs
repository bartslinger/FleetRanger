use std::collections::HashSet;

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::types::Uuid;
use sqlx::{PgConnection, PgPool};

use super::flight::{BatteryUse, Flight};
use crate::areas;

pub struct PendingLog {
    pub id: Uuid,
    pub path: String,
}

pub async fn known_paths(db: &PgPool) -> sqlx::Result<HashSet<String>> {
    let paths = sqlx::query_scalar!("select path from raw_logs")
        .fetch_all(db)
        .await?;
    Ok(paths.into_iter().collect())
}

/// Registers a raw log as pending. Returns false if a log with the same content
/// (or path) is already registered.
pub async fn insert_raw_log(
    db: &PgPool,
    sha256: &[u8],
    path: &str,
    size_bytes: i64,
) -> sqlx::Result<bool> {
    let inserted = sqlx::query_scalar!(
        "insert into raw_logs (sha256, path, size_bytes) values ($1, $2, $3)
         on conflict do nothing
         returning id",
        sha256,
        path,
        size_bytes,
    )
    .fetch_optional(db)
    .await?;
    Ok(inserted.is_some())
}

/// What happened to an uploaded log.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Stored {
    /// A new log, waiting to be processed.
    Added,
    /// A new version of a log at the same path, to be processed again.
    Replaced,
    /// The same content is already stored, at this or another path.
    Duplicate,
}

/// Whether a log with this content is already stored, at any path.
pub async fn has_content(db: &PgPool, sha256: &[u8]) -> sqlx::Result<bool> {
    sqlx::query_scalar!(
        r#"select exists(select 1 from raw_logs where sha256 = $1) as "exists!""#,
        sha256
    )
    .fetch_one(db)
    .await
}

/// Which of these contents are already stored, at any path.
pub async fn stored_contents(db: &PgPool, sha256s: &[Vec<u8>]) -> sqlx::Result<HashSet<Vec<u8>>> {
    let stored = sqlx::query_scalar!(
        "select sha256 from raw_logs where sha256 = any($1)",
        sha256s
    )
    .fetch_all(db)
    .await?;
    Ok(stored.into_iter().collect())
}

/// Registers an uploaded log at its path, with the pilot chosen on the
/// controller (ignored if there's no such user). A log already at that path is
/// a previous version: it's replaced and processed again.
pub async fn save_upload(
    db: &PgPool,
    sha256: &[u8],
    path: &str,
    size_bytes: i64,
    pilot_id: Option<Uuid>,
    team_id: Option<Uuid>,
) -> sqlx::Result<Stored> {
    // The team only with a pilot who is in it.
    let replaced = sqlx::query_scalar!(
        r#"insert into raw_logs (sha256, path, size_bytes, pilot_id, team_id)
           values ($1, $2, $3, (select id from users where id = $4),
                   (select team_id from team_members where user_id = $4 and team_id = $5))
           on conflict (path) do update set
               sha256 = excluded.sha256,
               size_bytes = excluded.size_bytes,
               pilot_id = excluded.pilot_id,
               team_id = excluded.team_id,
               received_at = now(),
               status = 'pending',
               attempts = 0,
               last_error = null,
               processed_at = null
           -- xmax is set on a row that "on conflict" updated rather than inserted.
           returning (xmax <> 0) as "replaced!""#,
        sha256,
        path,
        size_bytes,
        pilot_id,
        team_id,
    )
    .fetch_one(db)
    .await?;
    Ok(if replaced {
        Stored::Replaced
    } else {
        Stored::Added
    })
}

/// The oldest pending raw log, counting this as an attempt to process it.
pub async fn next_pending(db: &PgPool) -> sqlx::Result<Option<PendingLog>> {
    sqlx::query_as!(
        PendingLog,
        "update raw_logs set attempts = attempts + 1
         where id = (
             select id from raw_logs where status = 'pending'
             order by received_at, id
             limit 1
             for update skip locked
         )
         returning id, path",
    )
    .fetch_optional(db)
    .await
}

/// Saves the flight found in a raw log, or removes an earlier one if the log turned
/// out to have none, and marks the log done. Unknown drone and controller serials
/// are registered. A new flight gets the pilot chosen on the controller, if they
/// are still active and in a team, and that pilot's team; without one, its
/// controller's team unless that team is archived. Re-processing updates the
/// flight in place, so its id, team and pilot stay the same. Returns whether
/// the flight is new.
pub async fn save_result(db: &PgPool, raw_log_id: Uuid, flight: Option<&Flight>) -> Result<bool> {
    let mut tx = db.begin().await?;
    let new = match flight {
        Some(flight) => {
            if let Some(serial) = &flight.aircraft_serial {
                sqlx::query!(
                    "insert into aircraft (serial) values ($1) on conflict (serial) do nothing",
                    serial,
                )
                .execute(&mut *tx)
                .await?;
            }
            if let Some(serial) = &flight.rc_serial {
                sqlx::query!(
                    "insert into remote_controllers (serial) values ($1)
                     on conflict (serial) do nothing",
                    serial,
                )
                .execute(&mut *tx)
                .await?;
            }
            let saved = sqlx::query!(
                // The flight's team (user's decisions, 2026-10-19): the one chosen
                // with the pilot on the controller; else the pilot's, if they're
                // in one team; else the controller's; else the drone's.
                "with pilot_teams as (
                     select u.id, m.team_id from raw_logs l
                     join users u on u.id = l.pilot_id and u.archived_at is null
                     join team_members m on m.user_id = u.id
                     join teams t on t.id = m.team_id and t.archived_at is null
                     where l.id = $1
                 )
                 insert into flights (raw_log_id, takeoff_at, landed_at, air_time_s, distance_m,
                                      aircraft_serial, rc_serial, track, metrics,
                                      pilot_id, team_id)
                 values ($1, $2, $3, $4, $5, $6, $7, $8, $9,
                         (select distinct id from pilot_teams),
                         coalesce((select p.team_id from pilot_teams p
                                   join raw_logs l on l.team_id = p.team_id
                                   where l.id = $1),
                                  (select (array_agg(team_id))[1] from pilot_teams
                                   having count(*) = 1),
                                  (select r.team_id from remote_controllers r
                                   join teams t on t.id = r.team_id
                                   where r.serial = $7 and t.archived_at is null),
                                  (select a.team_id from aircraft a
                                   join teams t on t.id = a.team_id
                                   where a.serial = $6 and t.archived_at is null)))
                 on conflict (raw_log_id) do update set
                     takeoff_at = excluded.takeoff_at,
                     landed_at = excluded.landed_at,
                     air_time_s = excluded.air_time_s,
                     distance_m = excluded.distance_m,
                     aircraft_serial = excluded.aircraft_serial,
                     rc_serial = excluded.rc_serial,
                     track = excluded.track,
                     metrics = excluded.metrics
                 -- xmax is set on a row that \"on conflict\" updated rather than inserted.
                 returning id, (xmax = 0) as \"new!\"",
                raw_log_id,
                flight.takeoff_at,
                flight.landed_at,
                flight.air_time_s,
                flight.distance_m,
                flight.aircraft_serial,
                flight.rc_serial,
                flight.track,
                serde_json::to_value(&flight.metrics)?,
            )
            .fetch_one(&mut *tx)
            .await?;
            areas::db::compute_flight(&mut tx, saved.id).await?;
            save_battery(&mut tx, saved.id, flight.battery.as_ref()).await?;
            saved.new
        }
        None => {
            sqlx::query!("delete from flights where raw_log_id = $1", raw_log_id)
                .execute(&mut *tx)
                .await?;
            false
        }
    };
    sqlx::query!(
        "update raw_logs set status = 'done', last_error = null, processed_at = now()
         where id = $1",
        raw_log_id,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(new)
}

/// Registers the flight's battery if it's new, and replaces what the flight
/// says about it.
async fn save_battery(
    tx: &mut PgConnection,
    flight_id: Uuid,
    battery: Option<&BatteryUse>,
) -> sqlx::Result<()> {
    sqlx::query!(
        "delete from flight_batteries where flight_id = $1",
        flight_id
    )
    .execute(&mut *tx)
    .await?;
    let Some(battery) = battery else {
        return Ok(());
    };
    sqlx::query!(
        "insert into batteries (serial) values ($1) on conflict (serial) do nothing",
        battery.serial,
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        // Logs name one battery, so it's in slot 0; where a log of a drone
        // with more batteries keeps the others' serials is still unknown.
        "insert into flight_batteries (flight_id, slot, battery_id, charge_start_pct, charge_end_pct,
                                      full_capacity_mah, design_capacity_mah, cycles,
                                      temperature_min_c, temperature_max_c,
                                      cell_voltage_min_v, cell_spread_max_v)
         values ($1, 0, (select id from batteries where serial = $2), $3, $4, $5, $6, $7, $8,
                 $9, $10, $11)",
        flight_id,
        battery.serial,
        battery.charge_start_pct,
        battery.charge_end_pct,
        battery.full_capacity_mah,
        battery.design_capacity_mah,
        battery.cycles,
        battery.temperature_min_c,
        battery.temperature_max_c,
        battery.cell_voltage_min_v,
        battery.cell_spread_max_v,
    )
    .execute(&mut *tx)
    .await?;
    Ok(())
}

pub async fn mark_failed(db: &PgPool, id: Uuid, error: &str) -> sqlx::Result<()> {
    sqlx::query!(
        "update raw_logs set status = 'failed', last_error = $2, processed_at = now()
         where id = $1",
        id,
        error,
    )
    .execute(db)
    .await?;
    Ok(())
}

/// The body of the latest successful key request for a raw log.
pub async fn stored_key_response(db: &PgPool, raw_log_id: Uuid) -> sqlx::Result<Option<String>> {
    let body = sqlx::query_scalar!(
        "select response_body from dji_key_requests
         where raw_log_id = $1 and response_status = 200 and response_body is not null
         order by requested_at desc
         limit 1",
        raw_log_id,
    )
    .fetch_optional(db)
    .await?;
    Ok(body.flatten())
}

pub async fn insert_key_request(
    db: &PgPool,
    raw_log_id: Uuid,
    request_body: &str,
    response_status: Option<i32>,
    response_body: Option<&str>,
    error: Option<&str>,
) -> sqlx::Result<()> {
    sqlx::query!(
        "insert into dji_key_requests
             (raw_log_id, request_body, response_status, response_body, error)
         values ($1, $2, $3, $4, $5)",
        raw_log_id,
        request_body,
        response_status,
        response_body,
        error,
    )
    .execute(db)
    .await?;
    Ok(())
}

/// Progress through a batch: the logs still pending, and those processed since
/// the batch started.
pub struct BatchCounts {
    pub pending: i64,
    pub processed: i64,
    pub failed: i64,
}

pub async fn batch_counts(db: &PgPool, since: DateTime<Utc>) -> sqlx::Result<BatchCounts> {
    sqlx::query_as!(
        BatchCounts,
        r#"select count(*) filter (where status = 'pending') as "pending!",
                  count(*) filter (where status <> 'pending' and processed_at >= $1) as "processed!",
                  count(*) filter (where status = 'failed' and processed_at >= $1) as "failed!"
           from raw_logs"#,
        since,
    )
    .fetch_one(db)
    .await
}

/// A log of a batch that has been processed.
#[derive(serde::Serialize)]
pub struct Processed {
    pub path: String,
    /// done or failed.
    pub status: String,
    pub error: Option<String>,
    /// The flight found in it; none if the aircraft never took off.
    pub flight_id: Option<Uuid>,
}

/// The latest logs processed since a batch started, newest first.
pub async fn latest_processed(
    db: &PgPool,
    since: DateTime<Utc>,
    limit: i64,
) -> sqlx::Result<Vec<Processed>> {
    sqlx::query_as!(
        Processed,
        r#"select r.path, r.status, r.last_error as error, f.id as "flight_id?"
           from raw_logs r
           left join flights f on f.raw_log_id = r.id
           where r.status <> 'pending' and r.processed_at >= $1
           order by r.processed_at desc, r.id
           limit $2"#,
        since,
        limit,
    )
    .fetch_all(db)
    .await
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use serde_json::json;

    use super::*;
    use crate::ingest::flight::Metrics;

    fn flight() -> Flight {
        Flight {
            takeoff_at: Utc::now(),
            landed_at: Utc::now(),
            air_time_s: 60,
            distance_m: 100.0,
            aircraft_serial: None,
            rc_serial: Some("TESTRC0003".to_owned()),
            track: json!({}),
            battery: None,
            metrics: Metrics {
                max_altitude_m: 10.0,
                camera_serial: None,
                battery_serial: None,
            },
        }
    }

    async fn pilot_and_team(db: &PgPool, raw_log: Uuid) -> (Option<Uuid>, Option<Uuid>) {
        sqlx::query_as("select pilot_id, team_id from flights where raw_log_id = $1")
            .bind(raw_log)
            .fetch_one(db)
            .await
            .unwrap()
    }

    /// An upload's flight gets the pilot chosen on the controller, with the
    /// pilot's team; a new version of the log replaces it and keeps the flight.
    #[sqlx::test]
    async fn uploads_bring_their_pilot_and_replace_earlier_versions(db: PgPool) {
        let team: Uuid =
            sqlx::query_scalar("insert into teams (name) values ('North') returning id")
                .fetch_one(&db)
                .await
                .unwrap();
        let pilot: Uuid = sqlx::query_scalar("insert into users (name) values ('A') returning id")
            .fetch_one(&db)
            .await
            .unwrap();
        sqlx::query("insert into team_members (user_id, team_id) values ($1, $2)")
            .bind(pilot)
            .bind(team)
            .execute(&db)
            .await
            .unwrap();
        let path = "raw/TESTRC0003/FlightRecord/DJIFlightRecord_test.txt";

        let first = save_upload(&db, &[1; 32], path, 10, Some(pilot), None)
            .await
            .unwrap();
        assert_eq!(first, Stored::Added);
        let log = next_pending(&db).await.unwrap().unwrap();
        assert!(save_result(&db, log.id, Some(&flight())).await.unwrap());
        assert_eq!(pilot_and_team(&db, log.id).await, (Some(pilot), Some(team)));

        let second = save_upload(&db, &[2; 32], path, 20, Some(pilot), None)
            .await
            .unwrap();
        assert_eq!(second, Stored::Replaced);
        let again = next_pending(&db).await.unwrap().unwrap();
        assert_eq!(again.id, log.id, "the same log, to be processed again");
        assert!(
            !save_result(&db, log.id, Some(&flight())).await.unwrap(),
            "the same flight, so it isn't placed in a session again"
        );
        assert!(has_content(&db, &[2; 32]).await.unwrap());
        assert!(!has_content(&db, &[1; 32]).await.unwrap());
        let stored = stored_contents(&db, &[vec![1; 32], vec![2; 32]])
            .await
            .unwrap();
        assert_eq!(
            stored,
            HashSet::from([vec![2; 32]]),
            "only the latest version"
        );
    }

    /// A pilot in two teams: the flight belongs to the team chosen with them on
    /// the controller; without one, or with a team they're not in, to the
    /// controller's team.
    #[sqlx::test]
    async fn the_flights_team_is_the_one_chosen_on_the_controller(db: PgPool) {
        let team = |name: &'static str| {
            let db = db.clone();
            async move {
                sqlx::query_scalar::<_, Uuid>("insert into teams (name) values ($1) returning id")
                    .bind(name)
                    .fetch_one(&db)
                    .await
                    .unwrap()
            }
        };
        let (north, south, base, other) = (
            team("North").await,
            team("South").await,
            team("Base").await,
            team("Other").await,
        );
        let pilot: Uuid = sqlx::query_scalar("insert into users (name) values ('A') returning id")
            .fetch_one(&db)
            .await
            .unwrap();
        for team in [north, south] {
            sqlx::query("insert into team_members (user_id, team_id) values ($1, $2)")
                .bind(pilot)
                .bind(team)
                .execute(&db)
                .await
                .unwrap();
        }
        sqlx::query("insert into remote_controllers (serial, team_id) values ('TESTRC0003', $1)")
            .bind(base)
            .execute(&db)
            .await
            .unwrap();

        let mut processed = Vec::new();
        for (n, chosen) in [Some(south), None, Some(other)].into_iter().enumerate() {
            let path = format!("raw/TESTRC0003/FlightRecord/DJIFlightRecord_test{n}.txt");
            let sha256 = [u8::try_from(n).unwrap(); 32];
            save_upload(&db, &sha256, &path, 10, Some(pilot), chosen)
                .await
                .unwrap();
            let log = next_pending(&db).await.unwrap().unwrap();
            save_result(&db, log.id, Some(&flight())).await.unwrap();
            processed.push(pilot_and_team(&db, log.id).await);
        }
        assert_eq!(
            processed,
            [
                (Some(pilot), Some(south)),
                (Some(pilot), Some(base)),
                (Some(pilot), Some(base))
            ]
        );
    }
}
