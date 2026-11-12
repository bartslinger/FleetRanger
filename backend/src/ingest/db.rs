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
    sessions::{self, SessionGap},
};

/// A pending log as the loop claimed it. Its result is written only while
/// the log still has this content and this attempt: a new version uploaded
/// meanwhile, or another worker's claim, makes it stale (see `save_result`).
pub struct PendingLog {
    pub id: Uuid,
    pub path: String,
    pub organisation_id: Uuid,
    pub sha256: Vec<u8>,
    pub attempts: i32,
}

/// What became of a log's result.
pub enum Saved {
    /// Saved, and the log is done.
    Done,
    /// The log can't give a flight here, for this reason: it fails.
    Refused(String),
    /// The claim was stale: nothing was saved, and the log is processed again.
    Superseded,
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
               processed_at = null,
               -- Due at once, and free: a claim on the previous version
               -- can't save its result anyway.
               next_attempt_at = now(),
               claimed_until = null
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

/// How long a claim holds a log: far longer than processing one takes, so
/// only a backend that died lets it run out, and then another takes the log.
const LEASE_MINUTES: i32 = 5;

/// Claims the next due raw log of any organisation (with `System`), counting
/// this as an attempt to process it: a pending log whose time has come and
/// that no backend holds. Organisations take turns (user's decision,
/// 2026-11-12): the one claimed from longest ago goes first, so a big import
/// in one doesn't hold up another's uploads. Within an organisation, fresh
/// logs come before ones tried before, then the oldest.
pub async fn next_pending(db: &mut PgConnection) -> sqlx::Result<Option<PendingLog>> {
    sqlx::query_as!(
        PendingLog,
        "update raw_logs
         set attempts = attempts + 1, claimed_until = now() + make_interval(mins => $1)
         where id = (
             select r.id
             from raw_logs r
             join (select organisation_id, max(claimed_until) as last_claimed
                   from raw_logs group by organisation_id) s
               on s.organisation_id = r.organisation_id
             where r.status = 'pending' and r.next_attempt_at <= now()
               and (r.claimed_until is null or r.claimed_until < now())
             order by s.last_claimed nulls first, r.attempts, r.received_at, r.id
             limit 1
             for update of r skip locked
         )
         returning id, path, organisation_id, sha256, attempts",
        LEASE_MINUTES,
    )
    .fetch_optional(&mut *db)
    .await
}

/// A log that hit an error that may pass (this log, later): it's tried again
/// after `delay_s`, and other logs go on meanwhile. False if the claim was
/// stale (see `save_result`).
pub async fn retry_later(
    db: &mut PgConnection,
    log: &PendingLog,
    delay_s: i32,
    error: &str,
) -> sqlx::Result<bool> {
    let marked = sqlx::query!(
        "update raw_logs
         set next_attempt_at = now() + make_interval(secs => $4), last_error = $5,
             claimed_until = now()
         where id = $1 and sha256 = $2 and attempts = $3 and status = 'pending'",
        log.id,
        log.sha256,
        log.attempts,
        f64::from(delay_s),
        error,
    )
    .execute(&mut *db)
    .await?;
    Ok(marked.rows_affected() == 1)
}

/// A log that needs DJI while DJI is down: it waits until `until`, and the
/// attempt doesn't count, as it's not the log's doing. False if the claim was
/// stale.
pub async fn wait_for_dji(
    db: &mut PgConnection,
    log: &PendingLog,
    until: DateTime<Utc>,
    reason: &str,
) -> sqlx::Result<bool> {
    let marked = sqlx::query!(
        "update raw_logs
         set next_attempt_at = $4, attempts = attempts - 1, last_error = $5,
             claimed_until = now()
         where id = $1 and sha256 = $2 and attempts = $3 and status = 'pending'",
        log.id,
        log.sha256,
        log.attempts,
        until,
        reason,
    )
    .execute(&mut *db)
    .await?;
    Ok(marked.rows_affected() == 1)
}

// DJI's key service as every backend sees it (the one row of `dji_api`), with
// `System`: requests have no access to it.

/// DJI's outage window and why, if it's still open.
pub async fn dji_blocked(db: &mut PgConnection) -> sqlx::Result<Option<(DateTime<Utc>, String)>> {
    let row = sqlx::query!(
        r#"select blocked_until as "until!", coalesce(last_error, '') as "reason!"
           from dji_api where blocked_until > now()"#
    )
    .fetch_optional(&mut *db)
    .await?;
    Ok(row.map(|row| (row.until, row.reason)))
}

/// Reserves the next request slot, one second after the previous one by any
/// backend, start to start; the caller waits until it.
pub async fn dji_slot(db: &mut PgConnection) -> sqlx::Result<DateTime<Utc>> {
    sqlx::query_scalar!(
        r#"update dji_api set next_request_at = greatest(next_request_at, now()) + interval '1 second'
           returning next_request_at - interval '1 second' as "slot!""#
    )
    .fetch_one(&mut *db)
    .await
}

/// DJI failed: no backend asks it for a while, one minute at first, doubling
/// with each failure in a row up to 15. Returns when it may be asked again.
pub async fn dji_failed(db: &mut PgConnection, reason: &str) -> sqlx::Result<DateTime<Utc>> {
    sqlx::query_scalar!(
        r#"update dji_api
           set blocked_until = now() + least(interval '15 minutes',
                                             interval '1 minute' * power(2, least(failures, 10))),
               failures = failures + 1, last_error = $1
           returning blocked_until as "until!""#,
        reason,
    )
    .fetch_one(&mut *db)
    .await
}

/// DJI answered: the outage, if any, is over.
pub async fn dji_answered(db: &mut PgConnection) -> sqlx::Result<()> {
    sqlx::query!(
        "update dji_api set failures = 0, blocked_until = null, last_error = null
         where failures > 0 or blocked_until is not null"
    )
    .execute(&mut *db)
    .await?;
    Ok(())
}

/// Saves the flight found in a raw log, or removes an earlier one if the log turned
/// out to have none, and marks the log done. Unknown drone and controller serials
/// are registered in the flight's group. A new flight gets the pilot chosen on the
/// controller, if they are still active and fly somewhere, and a group: the one
/// chosen with the pilot if they fly for it, else the import's, else the
/// controller's (user's decision, 2026-11-11), else the only group there is.
/// Re-processing updates the flight in place, so its id, group and pilot stay
/// the same. A flight seen for the first time is placed in a session in the
/// same transaction, so a flight is either new and placed or neither;
/// processing a log again never puts back a flight someone took out of its
/// session.
pub async fn save_result(
    db: &mut PgConnection,
    log: &PendingLog,
    flight: Option<&Flight>,
    gap: SessionGap,
) -> Result<Saved> {
    let raw_log_id = log.id;
    let mut tx = db.begin().await?;
    // First, and holding the row until the end: the claim must still be the
    // log's, or nothing is saved.
    let current = sqlx::query!(
        "update raw_logs
         set status = 'done', last_error = null, processed_at = now(), claimed_until = now()
         where id = $1 and sha256 = $2 and attempts = $3 and status = 'pending'",
        raw_log_id,
        log.sha256,
        log.attempts,
    )
    .execute(&mut *tx)
    .await?;
    if current.rows_affected() == 0 {
        return Ok(Saved::Superseded);
    }
    match flight {
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
            // Not the log's content, but nothing to retry either: it fails,
            // as docs/architecture.md says, until someone imports it with a group.
            let Some(group_id) = placed.group_id else {
                return Ok(Saved::Refused(
                    "no group for this flight: the controller is unknown".to_owned(),
                ));
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
            if saved.new {
                sessions::db::place_upload(&mut tx, raw_log_id, gap).await?;
            }
        }
        None => {
            sqlx::query!("delete from flights where raw_log_id = $1", raw_log_id)
                .execute(&mut *tx)
                .await?;
        }
    }
    tx.commit().await?;
    Ok(Saved::Done)
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

/// Marks a log failed, if the claim is still the log's (see `save_result`).
/// False if it was stale: the log is processed again.
pub async fn mark_failed(
    db: &mut PgConnection,
    log: &PendingLog,
    error: &str,
) -> sqlx::Result<bool> {
    let marked = sqlx::query!(
        "update raw_logs
         set status = 'failed', last_error = $4, processed_at = now(), claimed_until = now()
         where id = $1 and sha256 = $2 and attempts = $3 and status = 'pending'",
        log.id,
        log.sha256,
        log.attempts,
        error,
    )
    .execute(&mut *db)
    .await?;
    Ok(marked.rows_affected() == 1)
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

    /// A claimed log is no one else's until its claim runs out, as when its
    /// backend died; a log to be tried later waits, and others go on; a log
    /// waiting for DJI doesn't use up an attempt.
    #[sqlx::test]
    async fn claims_hold_and_logs_wait_their_turn(pool: sqlx::PgPool) {
        let mut db = crate::testing::db(&pool).await;
        for path in ["first", "second"] {
            sqlx::query(
                "insert into raw_logs (sha256, path, size_bytes, received_at)
                 values (sha256($1::bytea), $1, 0,
                         now() - case $1 when 'first' then interval '2 minutes' else interval '1 minute' end)",
            )
            .bind(path)
            .execute(&mut *db)
            .await
            .unwrap();
        }
        let first = next_pending(&mut db).await.unwrap().unwrap();
        assert_eq!(first.path, "first");
        let second = next_pending(&mut db).await.unwrap().unwrap();
        assert_eq!(second.path, "second", "the first is claimed");
        assert!(next_pending(&mut db).await.unwrap().is_none());

        // The first's backend died: its claim runs out, and it's taken again.
        sqlx::query(
            "update raw_logs set claimed_until = now() - interval '1 second' where id = $1",
        )
        .bind(first.id)
        .execute(&mut *db)
        .await
        .unwrap();
        let again = next_pending(&mut db).await.unwrap().unwrap();
        assert_eq!((again.id, again.attempts), (first.id, 2));

        // An error that may pass: it waits, and isn't due meanwhile.
        assert!(retry_later(&mut db, &again, 60, "Test").await.unwrap());
        assert!(next_pending(&mut db).await.unwrap().is_none());

        // Waiting for DJI gives the attempt back.
        let until = Utc::now() + chrono::Duration::minutes(1);
        assert!(wait_for_dji(&mut db, &second, until, "Test").await.unwrap());
        let attempts: i32 = sqlx::query_scalar("select attempts from raw_logs where id = $1")
            .bind(second.id)
            .fetch_one(&mut *db)
            .await
            .unwrap();
        assert_eq!(attempts, 0);
    }

    /// DJI's request rate and outages are shared by every backend: slots a
    /// second apart, an outage window doubling up to 15 minutes, cleared by
    /// the first answer.
    #[sqlx::test]
    async fn dji_requests_are_spaced_and_paused_for_everyone(pool: sqlx::PgPool) {
        let mut system = pool.acquire().await.unwrap();
        let first = dji_slot(&mut system).await.unwrap();
        let second = dji_slot(&mut system).await.unwrap();
        assert_eq!(second - first, chrono::Duration::seconds(1));

        assert!(dji_blocked(&mut system).await.unwrap().is_none());
        let mut windows = Vec::new();
        for _ in 0..6 {
            let until = dji_failed(&mut system, "Test").await.unwrap();
            windows.push((until - Utc::now()).num_seconds() + 1);
        }
        assert_eq!(windows, [60, 120, 240, 480, 900, 900]);
        assert!(dji_blocked(&mut system).await.unwrap().is_some());
        dji_answered(&mut system).await.unwrap();
        assert!(dji_blocked(&mut system).await.unwrap().is_none());
        let after = dji_failed(&mut system, "Test").await.unwrap();
        assert_eq!(
            (after - Utc::now()).num_seconds() + 1,
            60,
            "counting from 1 again"
        );
    }

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
        let gap = SessionGap(120);
        let log = next_pending(&mut db).await.unwrap().unwrap();
        let saved = save_result(&mut db, &log, Some(&flight()), gap)
            .await
            .unwrap();
        assert!(matches!(saved, Saved::Done));
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
        // A newer version arrives while that one is being processed: its
        // result is stale, nothing is saved, and the log waits again.
        save_upload(&mut db, &[3; 32], path, 30, Some(pilot), Some(team))
            .await
            .unwrap();
        let stale = save_result(&mut db, &again, Some(&flight()), gap)
            .await
            .unwrap();
        assert!(matches!(stale, Saved::Superseded));
        assert!(!mark_failed(&mut db, &again, "Test").await.unwrap());
        let latest = next_pending(&mut db).await.unwrap().unwrap();
        assert_eq!(latest.sha256, vec![3; 32]);
        let saved = save_result(&mut db, &latest, Some(&flight()), gap)
            .await
            .unwrap();
        assert!(matches!(saved, Saved::Done));
        assert!(has_content(&mut db, &[3; 32]).await.unwrap());
        assert!(!has_content(&mut db, &[1; 32]).await.unwrap());
        let stored = stored_contents(&mut db, &[vec![1; 32], vec![2; 32], vec![3; 32]])
            .await
            .unwrap();
        assert_eq!(
            stored,
            HashSet::from([vec![3; 32]]),
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
            save_result(&mut db, &log, Some(&flight()), SessionGap(120))
                .await
                .unwrap();
            processed.push(pilot_and_group(&mut db, log.id).await);
        }
        assert!(
            insert_raw_log(&mut db, &[9; 32], "old/test.txt", 10, other)
                .await
                .unwrap()
        );
        let log = next_pending(&mut db).await.unwrap().unwrap();
        save_result(&mut db, &log, Some(&flight()), SessionGap(120))
            .await
            .unwrap();
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

        // Without a group, chosen or given, from an unknown controller, the
        // log fails instead of being tried again and again.
        sqlx::query(
            "insert into raw_logs (sha256, path, size_bytes) values (sha256('test'), 'raw/test.txt', 0)",
        )
        .execute(&mut *db)
        .await
        .unwrap();
        let log = next_pending(&mut db).await.unwrap().unwrap();
        let unknown = Flight {
            rc_serial: Some("TESTUNKNOWN".to_owned()),
            ..flight()
        };
        let refused = save_result(&mut db, &log, Some(&unknown), SessionGap(120))
            .await
            .unwrap();
        let Saved::Refused(reason) = refused else {
            panic!("expected the log to be refused");
        };
        assert!(mark_failed(&mut db, &log, &reason).await.unwrap());
        assert!(next_pending(&mut db).await.unwrap().is_none());
    }
}
