use std::collections::HashSet;

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use sqlx::types::Uuid;

use super::flight::Flight;

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

/// Registers an uploaded log at its path, with the pilot chosen on the
/// controller (ignored if there's no such user). A log already at that path is
/// a previous version: it's replaced and processed again.
pub async fn save_upload(
    db: &PgPool,
    sha256: &[u8],
    path: &str,
    size_bytes: i64,
    pilot_id: Option<Uuid>,
) -> sqlx::Result<Stored> {
    let replaced = sqlx::query_scalar!(
        r#"insert into raw_logs (sha256, path, size_bytes, pilot_id)
           values ($1, $2, $3, (select id from users where id = $4))
           on conflict (path) do update set
               sha256 = excluded.sha256,
               size_bytes = excluded.size_bytes,
               pilot_id = excluded.pilot_id,
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
/// flight in place, so its id, team and pilot stay the same.
pub async fn save_result(db: &PgPool, raw_log_id: Uuid, flight: Option<&Flight>) -> Result<()> {
    let mut tx = db.begin().await?;
    match flight {
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
            sqlx::query!(
                "with pilot as (
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
                         (select id from pilot),
                         coalesce((select team_id from pilot),
                                  (select r.team_id from remote_controllers r
                                   join teams t on t.id = r.team_id
                                   where r.serial = $7 and t.archived_at is null)))
                 on conflict (raw_log_id) do update set
                     takeoff_at = excluded.takeoff_at,
                     landed_at = excluded.landed_at,
                     air_time_s = excluded.air_time_s,
                     distance_m = excluded.distance_m,
                     aircraft_serial = excluded.aircraft_serial,
                     rc_serial = excluded.rc_serial,
                     track = excluded.track,
                     metrics = excluded.metrics",
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
            .execute(&mut *tx)
            .await?;
        }
        None => {
            sqlx::query!("delete from flights where raw_log_id = $1", raw_log_id)
                .execute(&mut *tx)
                .await?;
        }
    }
    sqlx::query!(
        "update raw_logs set status = 'done', last_error = null, processed_at = now()
         where id = $1",
        raw_log_id,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
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

        let first = save_upload(&db, &[1; 32], path, 10, Some(pilot))
            .await
            .unwrap();
        assert_eq!(first, Stored::Added);
        let log = next_pending(&db).await.unwrap().unwrap();
        save_result(&db, log.id, Some(&flight())).await.unwrap();
        assert_eq!(pilot_and_team(&db, log.id).await, (Some(pilot), Some(team)));

        let second = save_upload(&db, &[2; 32], path, 20, Some(pilot))
            .await
            .unwrap();
        assert_eq!(second, Stored::Replaced);
        let again = next_pending(&db).await.unwrap().unwrap();
        assert_eq!(again.id, log.id, "the same log, to be processed again");
        assert!(has_content(&db, &[2; 32]).await.unwrap());
        assert!(!has_content(&db, &[1; 32]).await.unwrap());
    }
}
