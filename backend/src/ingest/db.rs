use std::collections::HashSet;

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::Connection as _;
use sqlx::PgConnection;
use sqlx::types::Uuid;

use super::flight::{BatteryUse, Flight};
use crate::{
    areas,
    policy::{self, Permission},
};

pub struct PendingLog {
    pub id: Uuid,
    pub path: String,
    pub organisation_id: Uuid,
}

pub async fn known_paths(db: &mut PgConnection) -> sqlx::Result<HashSet<String>> {
    let paths = sqlx::query_scalar!("select path from raw_logs")
        .fetch_all(&mut *db)
        .await?;
    Ok(paths.into_iter().collect())
}

/// Every group, by name.
pub async fn groups(db: &mut PgConnection) -> sqlx::Result<Vec<(Uuid, String)>> {
    let rows = sqlx::query!("select id, name from groups order by name")
        .fetch_all(&mut *db)
        .await?;
    Ok(rows.into_iter().map(|row| (row.id, row.name)).collect())
}

/// Registers a raw log as pending, for a group. Returns false if a log with the same content
/// (or path) is already registered.
pub async fn insert_raw_log(
    db: &mut PgConnection,
    sha256: &[u8],
    path: &str,
    size_bytes: i64,
    group_id: Uuid,
) -> sqlx::Result<bool> {
    let inserted = sqlx::query_scalar!(
        "insert into raw_logs (sha256, path, size_bytes, group_id) values ($1, $2, $3, $4)
         on conflict do nothing
         returning id",
        sha256,
        path,
        size_bytes,
        group_id,
    )
    .fetch_optional(&mut *db)
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
pub async fn has_content(db: &mut PgConnection, sha256: &[u8]) -> sqlx::Result<bool> {
    sqlx::query_scalar!(
        r#"select exists(select 1 from raw_logs where sha256 = $1) as "exists!""#,
        sha256
    )
    .fetch_one(&mut *db)
    .await
}

/// Which of these contents are already stored, at any path.
pub async fn stored_contents(
    db: &mut PgConnection,
    sha256s: &[Vec<u8>],
) -> sqlx::Result<HashSet<Vec<u8>>> {
    let stored = sqlx::query_scalar!(
        "select sha256 from raw_logs where sha256 = any($1)",
        sha256s
    )
    .fetch_all(&mut *db)
    .await?;
    Ok(stored.into_iter().collect())
}

/// Registers an uploaded log at its path, with the pilot and group chosen on
/// the controller (each ignored if there's no such user or group; whether
/// they count is decided when the log is processed). A log already at that
/// path is a previous version: it's replaced and processed again.
pub async fn save_upload(
    db: &mut PgConnection,
    sha256: &[u8],
    path: &str,
    size_bytes: i64,
    pilot_id: Option<Uuid>,
    group_id: Option<Uuid>,
) -> sqlx::Result<Stored> {
    let replaced = sqlx::query_scalar!(
        r#"insert into raw_logs (sha256, path, size_bytes, pilot_id, group_id)
           values ($1, $2, $3, (select id from users where id = $4),
                   (select id from groups where id = $5))
           on conflict (organisation_id, path) do update set
               sha256 = excluded.sha256,
               size_bytes = excluded.size_bytes,
               pilot_id = excluded.pilot_id,
               group_id = excluded.group_id,
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
        group_id,
    )
    .fetch_one(&mut *db)
    .await?;
    Ok(if replaced {
        Stored::Replaced
    } else {
        Stored::Added
    })
}

/// The next pending raw log of any organisation (with `System`), counting
/// this as an attempt to process it. Organisations take turns (user's
/// decision, 2026-11-12): the one whose last log was processed longest ago
/// goes first, so a big import in one doesn't hold up another's uploads.
/// Within an organisation, fresh logs come before ones tried before, then
/// the oldest.
pub async fn next_pending(db: &mut PgConnection) -> sqlx::Result<Option<PendingLog>> {
    sqlx::query_as!(
        PendingLog,
        "update raw_logs set attempts = attempts + 1
         where id = (
             select r.id
             from raw_logs r
             join (select organisation_id, max(processed_at) as last_served
                   from raw_logs group by organisation_id) s
               on s.organisation_id = r.organisation_id
             where r.status = 'pending'
             order by s.last_served nulls first, r.attempts, r.received_at, r.id
             limit 1
             for update of r skip locked
         )
         returning id, path, organisation_id",
    )
    .fetch_optional(&mut *db)
    .await
}

/// Saves the flight found in a raw log, or removes an earlier one if the log turned
/// out to have none, and marks the log done. Unknown drone and controller serials
/// are registered in the flight's group. A new flight gets the pilot chosen on the
/// controller, if they are still active and fly somewhere, and a group: the one
/// chosen with the pilot if they fly for it, else the import's, else the
/// controller's (user's decision, 2026-11-11), else the only group there is.
/// Re-processing updates the flight in place, so its id, group and pilot stay
/// the same. Returns whether the flight is new.
pub async fn save_result(
    db: &mut PgConnection,
    raw_log_id: Uuid,
    flight: Option<&Flight>,
) -> Result<bool> {
    let mut tx = db.begin().await?;
    let new = match flight {
        Some(flight) => {
            let placed = sqlx::query!(
                r#"with pilot as (
                       select u.id from raw_logs l
                       join users u on u.id = l.pilot_id and u.archived_at is null
                       where l.id = $1
                         and exists (select 1 from group_members m
                                     where m.user_id = u.id and m.role = any($3))
                   ),
                   chosen as (
                       select l.group_id from raw_logs l, pilot p
                       where l.id = $1
                         and exists (select 1 from group_members m
                                     where m.user_id = p.id and m.group_id = l.group_id
                                       and m.role = any($3))
                   )
                   select (select id from pilot) as pilot_id,
                          coalesce(
                              (select group_id from flights where raw_log_id = $1),
                              (select group_id from chosen),
                              -- An import has no pilot, and its group is given.
                              (select group_id from raw_logs where id = $1 and pilot_id is null),
                              (select group_id from remote_controllers where serial = $2),
                              (select (array_agg(id))[1] from groups having count(*) = 1)
                          ) as group_id"#,
                raw_log_id,
                flight.rc_serial,
                &policy::roles_granting(Permission::FlightsFlyFor),
            )
            .fetch_one(&mut *tx)
            .await?;
            let Some(group_id) = placed.group_id else {
                anyhow::bail!("no group for this flight: the controller is unknown");
            };
            if let Some(serial) = &flight.aircraft_serial {
                sqlx::query!(
                    "insert into aircraft (serial, group_id) values ($1, $2)
                     on conflict (organisation_id, serial) do nothing",
                    serial,
                    group_id,
                )
                .execute(&mut *tx)
                .await?;
            }
            if let Some(serial) = &flight.rc_serial {
                sqlx::query!(
                    "insert into remote_controllers (serial, group_id) values ($1, $2)
                     on conflict (organisation_id, serial) do nothing",
                    serial,
                    group_id,
                )
                .execute(&mut *tx)
                .await?;
            }
            let saved = sqlx::query!(
                "insert into flights (raw_log_id, takeoff_at, landed_at, air_time_s, distance_m,
                                      aircraft_serial, rc_serial, track, metrics,
                                      pilot_id, group_id)
                 values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
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
                placed.pilot_id,
                group_id,
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
        "insert into batteries (serial) values ($1) on conflict (organisation_id, serial) do nothing",
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

pub async fn mark_failed(db: &mut PgConnection, id: Uuid, error: &str) -> sqlx::Result<()> {
    sqlx::query!(
        "update raw_logs set status = 'failed', last_error = $2, processed_at = now()
         where id = $1",
        id,
        error,
    )
    .execute(&mut *db)
    .await?;
    Ok(())
}

/// The body of the latest successful key request for a raw log.
pub async fn stored_key_response(
    db: &mut PgConnection,
    raw_log_id: Uuid,
) -> sqlx::Result<Option<String>> {
    let body = sqlx::query_scalar!(
        "select response_body from dji_key_requests
         where raw_log_id = $1 and response_status = 200 and response_body is not null
         order by requested_at desc
         limit 1",
        raw_log_id,
    )
    .fetch_optional(&mut *db)
    .await?;
    Ok(body.flatten())
}

pub async fn insert_key_request(
    db: &mut PgConnection,
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
    .execute(&mut *db)
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

pub async fn batch_counts(
    db: &mut PgConnection,
    since: DateTime<Utc>,
) -> sqlx::Result<BatchCounts> {
    sqlx::query_as!(
        BatchCounts,
        r#"select count(*) filter (where status = 'pending') as "pending!",
                  count(*) filter (where status <> 'pending' and processed_at >= $1) as "processed!",
                  count(*) filter (where status = 'failed' and processed_at >= $1) as "failed!"
           from raw_logs"#,
        since,
    )
    .fetch_one(&mut *db)
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
    db: &mut PgConnection,
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
    .fetch_all(&mut *db)
    .await
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use serde_json::json;

    use super::*;
    use crate::ingest::flight::Metrics;

    /// Organisations take turns: B's one upload comes right after A's first
    /// of many, not after all of them; within A, a log tried before goes
    /// after the fresh ones.
    #[sqlx::test]
    async fn organisations_take_turns_in_the_queue(pool: sqlx::PgPool) {
        let mut a = crate::testing::db(&pool).await;
        let a_id = crate::testing::organisation(&mut a).await;
        let b_id = crate::testing::second_organisation(&mut a, "Test Org B").await;
        // A's three, oldest first, the oldest tried once before; then B's.
        for (organisation, path, minutes_ago, attempts) in [
            (a_id, "a-tried", 40, 1),
            (a_id, "a-1", 30, 0),
            (a_id, "a-2", 20, 0),
            (b_id, "b-1", 10, 0),
        ] {
            sqlx::query(
                "insert into raw_logs (organisation_id, sha256, path, size_bytes, received_at, attempts)
                 values ($1, sha256($2::bytea), $2, 0, now() - make_interval(mins => $3), $4)",
            )
            .bind(organisation)
            .bind(path)
            .bind(minutes_ago)
            .bind(attempts)
            .execute(&pool)
            .await
            .unwrap();
        }
        let mut system = pool.acquire().await.unwrap();
        let mut order = Vec::new();
        while let Some(log) = next_pending(&mut system).await.unwrap() {
            sqlx::query("update raw_logs set status = 'done', processed_at = now() where id = $1")
                .bind(log.id)
                .execute(&mut *system)
                .await
                .unwrap();
            order.push(log.path);
        }
        assert_eq!(order, ["a-1", "b-1", "a-2", "a-tried"]);
    }

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

    async fn pilot_and_group(db: &mut PgConnection, raw_log: Uuid) -> (Option<Uuid>, Option<Uuid>) {
        sqlx::query_as("select pilot_id, group_id from flights where raw_log_id = $1")
            .bind(raw_log)
            .fetch_one(&mut *db)
            .await
            .unwrap()
    }

    /// An upload's flight gets the pilot chosen on the controller; a new
    /// version of the log replaces it and keeps the flight.
    #[sqlx::test]
    async fn uploads_bring_their_pilot_and_replace_earlier_versions(pool: sqlx::PgPool) {
        let mut db = crate::testing::db(&pool).await;
        let team = crate::testing::group(&mut db, "North").await;
        let pilot: Uuid = sqlx::query_scalar("insert into users (name) values ('A') returning id")
            .fetch_one(&mut *db)
            .await
            .unwrap();
        crate::testing::join(&mut db, pilot, team).await;
        let path = "raw/TESTRC0003/FlightRecord/DJIFlightRecord_test.txt";

        let first = save_upload(&mut db, &[1; 32], path, 10, Some(pilot), Some(team))
            .await
            .unwrap();
        assert_eq!(first, Stored::Added);
        let log = next_pending(&mut db).await.unwrap().unwrap();
        assert!(save_result(&mut db, log.id, Some(&flight())).await.unwrap());
        assert_eq!(
            pilot_and_group(&mut db, log.id).await,
            (Some(pilot), Some(team))
        );

        let second = save_upload(&mut db, &[2; 32], path, 20, Some(pilot), Some(team))
            .await
            .unwrap();
        assert_eq!(second, Stored::Replaced);
        let again = next_pending(&mut db).await.unwrap().unwrap();
        assert_eq!(again.id, log.id, "the same log, to be processed again");
        assert!(
            !save_result(&mut db, log.id, Some(&flight())).await.unwrap(),
            "the same flight, so it isn't placed in a session again"
        );
        assert!(has_content(&mut db, &[2; 32]).await.unwrap());
        assert!(!has_content(&mut db, &[1; 32]).await.unwrap());
        let stored = stored_contents(&mut db, &[vec![1; 32], vec![2; 32]])
            .await
            .unwrap();
        assert_eq!(
            stored,
            HashSet::from([vec![2; 32]]),
            "only the latest version"
        );
    }

    /// A pilot in two groups: the flight belongs to the group chosen with them
    /// on the controller; without one, or with a group they don't fly for, it
    /// goes to the controller's group. An import's flight goes to the group
    /// it was imported for, whatever the controller's.
    #[sqlx::test]
    async fn the_flights_group_is_chosen_else_the_controllers(pool: sqlx::PgPool) {
        let mut db = crate::testing::db(&pool).await;
        use crate::testing::group;
        let (north, south, base, other) = (
            group(&mut db, "North").await,
            group(&mut db, "South").await,
            group(&mut db, "Base").await,
            group(&mut db, "Other").await,
        );
        let pilot: Uuid = sqlx::query_scalar("insert into users (name) values ('A') returning id")
            .fetch_one(&mut *db)
            .await
            .unwrap();
        for team in [north, south] {
            crate::testing::join(&mut db, pilot, team).await;
        }
        sqlx::query("insert into remote_controllers (serial, group_id) values ('TESTRC0003', $1)")
            .bind(base)
            .execute(&mut *db)
            .await
            .unwrap();

        let mut processed = Vec::new();
        for (n, chosen) in [Some(south), None, Some(other)].into_iter().enumerate() {
            let path = format!("raw/TESTRC0003/FlightRecord/DJIFlightRecord_test{n}.txt");
            let sha256 = [u8::try_from(n).unwrap(); 32];
            save_upload(&mut db, &sha256, &path, 10, Some(pilot), chosen)
                .await
                .unwrap();
            let log = next_pending(&mut db).await.unwrap().unwrap();
            save_result(&mut db, log.id, Some(&flight())).await.unwrap();
            processed.push(pilot_and_group(&mut db, log.id).await);
        }
        assert!(
            insert_raw_log(&mut db, &[9; 32], "old/test.txt", 10, other)
                .await
                .unwrap()
        );
        let log = next_pending(&mut db).await.unwrap().unwrap();
        save_result(&mut db, log.id, Some(&flight())).await.unwrap();
        processed.push(pilot_and_group(&mut db, log.id).await);
        assert_eq!(
            processed,
            [
                (Some(pilot), Some(south)),
                (Some(pilot), Some(base)),
                (Some(pilot), Some(base)),
                (None, Some(other)),
            ]
        );
    }
}
