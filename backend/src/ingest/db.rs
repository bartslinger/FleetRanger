use std::collections::HashSet;

use anyhow::Result;
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
/// are registered. A new flight gets its controller's team unless that team is
/// archived; re-processing updates
/// the flight in place, so its id, team and pilot stay the same.
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
                "insert into flights (raw_log_id, takeoff_at, landed_at, air_time_s, distance_m,
                                      aircraft_serial, rc_serial, track, metrics, team_id)
                 values ($1, $2, $3, $4, $5, $6, $7, $8, $9,
                         (select r.team_id from remote_controllers r
                          join teams t on t.id = r.team_id
                          where r.serial = $7 and t.archived_at is null))
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
