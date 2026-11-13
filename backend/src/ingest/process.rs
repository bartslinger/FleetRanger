//! Processes pending raw logs, one at a time per backend: requests decryption
//! keys from DJI when needed, decrypts and parses the log, and saves the
//! flight in it. Several backends can run it at once: a log is claimed for a
//! while (`db::next_pending`), its result counts only for that claim, and
//! DJI's request rate and outages are shared through the database.
//!
//! An attempt at a log ends in one of four ways: done (a flight, or none);
//! the log's fault (it fails); DJI's fault (it waits for DJI, without using
//! up an attempt, and logs that don't need DJI go on); or an error that may
//! pass, such as a failed save (this log, later: tried again after a growing
//! delay, and failed after `MAX_ATTEMPTS`).

use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use dji_log_parser::DJILog;
use dji_log_parser::frame::records_to_frames;
use dji_log_parser::record::Record;
use dji_log_parser::record::smart_battery_group::SmartBatteryGroup;
use sqlx::PgConnection;

use super::activity::{Activity, Stage};
use super::db::{self, PendingLog, Saved};
use super::dji::{self, Keychains, Reply};
use super::flight::{self, BatteryInfo, Flight, Serials};
use crate::db::{System, Tenants};
use crate::organisations;

/// How long to wait before looking for new logs when none are due.
const IDLE_PAUSE: Duration = Duration::from_secs(10);
/// How long to wait after a problem outside any log, such as the database
/// being unreachable.
const ERROR_PAUSE: Duration = Duration::from_secs(60);
/// After this many attempts that ended in an error that may pass, a log fails.
const MAX_ATTEMPTS: i32 = 8;

/// How long a log waits after its `attempts`th attempt ended in an error that
/// may pass: 1 minute, 5, 30, 2 hours, then 6 hours; None once it has had
/// `MAX_ATTEMPTS` and fails.
fn retry_delay_s(attempts: i32) -> Option<i32> {
    match attempts {
        ..=1 => Some(60),
        2 => Some(5 * 60),
        3 => Some(30 * 60),
        4 => Some(2 * 3600),
        _ if attempts < MAX_ATTEMPTS => Some(6 * 3600),
        _ => None,
    }
}

/// The record types the parser knows (the `magic` values in its `record/mod.rs`).
/// A record of one of these types that comes out as `Unknown` failed to decode.
const KNOWN_RECORD_TYPES: [u8; 24] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 14, 15, 19, 22, 24, 25, 33, 40, 49, 50, 56, 62,
];

/// What a log contains, or why it can't give a flight (the log's fault).
type Processed = std::result::Result<Box<Decoded>, String>;

struct Decoded {
    /// `None` if the aircraft never took off.
    flight: Option<Flight>,
    /// The summary DJI's app wrote into the log, logged for comparison with ours.
    dji_total_time_s: f64,
    dji_total_distance_m: f32,
}

/// How one attempt at a log ended, short of an error that may pass.
enum Outcome {
    /// Its flight, or none; or why it fails.
    Processed(Processed),
    /// It needs keys while DJI is down: it waits until then.
    DjiDown {
        until: DateTime<Utc>,
        reason: String,
    },
}

pub async fn run(
    system: System,
    tenants: Tenants,
    log_dir: PathBuf,
    dji_app_key: Option<String>,
    activity: Activity,
) {
    loop {
        let result = process_next(
            &system,
            &tenants,
            &log_dir,
            dji_app_key.as_deref(),
            &activity,
        )
        .await;
        let pause = match result {
            Ok(true) => continue,
            Ok(false) => {
                activity.idle();
                IDLE_PAUSE
            }
            Err(error) => {
                tracing::warn!("log processing paused: {error:#}");
                activity.pause(ERROR_PAUSE);
                ERROR_PAUSE
            }
        };
        tokio::time::sleep(pause).await;
    }
}

/// Claims and processes the next due log, of any organisation, in its
/// organisation. Returns false if there was none. An error means trouble
/// outside the log, such as the database being unreachable; a log it held
/// is taken again when its claim runs out.
///
/// Each attempt runs as a task of its own, so that a panic, as in a library,
/// ends only that attempt: it counts as an error that may pass, and the loop
/// goes on.
async fn process_next(
    system: &System,
    tenants: &Tenants,
    log_dir: &Path,
    dji_app_key: Option<&str>,
    activity: &Activity,
) -> Result<bool> {
    let Some(log) = db::next_pending(&mut *system.acquire().await?).await? else {
        return Ok(false);
    };
    let mut db = tenants.connect(log.organisation_id).await?;
    // Every attempt ends by recording how it went, and the last one that
    // records an error fails the log. Claimed once more, the log stopped
    // the backend each time it was tried.
    if log.attempts > MAX_ATTEMPTS {
        let reason = format!("gave up after {MAX_ATTEMPTS} attempts: the backend stopped while processing it");
        tracing::warn!(path = log.path, reason, "log failed to process");
        if !db::mark_failed(&mut db, &log, &reason).await? {
            superseded(&log);
        }
        return Ok(true);
    }
    activity.start(log.organisation_id, &log.path);

    let path = super::organisation_dir(log_dir, log.organisation_id).join(&log.path);
    let attempt = {
        let (system, log, activity) = (system.clone(), log.clone(), activity.clone());
        let dji_app_key = dji_app_key.map(str::to_owned);
        tokio::spawn(async move {
            let outcome = process(
                &mut db,
                &system,
                &log,
                &path,
                dji_app_key.as_deref(),
                &activity,
            )
            .await?;
            activity.stage(Stage::Saving);
            finish(&mut db, &log, outcome).await
        })
        .await
    };
    activity.finished();
    let error = match attempt {
        Ok(Ok(())) => return Ok(true),
        Ok(Err(error)) => format!("{error:#}"),
        Err(error) => format!("processing stopped: {error}"),
    };
    // The attempt's connection may have gone with it.
    let db = &mut tenants.connect(log.organisation_id).await?;
    let still_ours = match retry_delay_s(log.attempts) {
        Some(delay_s) => {
            tracing::warn!(path = log.path, error, delay_s, "log will be tried again");
            db::retry_later(db, &log, delay_s, &error).await?
        }
        None => {
            let reason = format!("gave up after {MAX_ATTEMPTS} attempts: {error}");
            tracing::warn!(path = log.path, reason, "log failed to process");
            db::mark_failed(db, &log, &reason).await?
        }
    };
    if !still_ours {
        superseded(&log);
    }
    Ok(true)
}

/// Saves what became of an attempt, if the claim is still the log's.
async fn finish(db: &mut PgConnection, log: &PendingLog, outcome: Outcome) -> Result<()> {
    let failed = match outcome {
        // The status shows the outage from the database, to every organisation.
        Outcome::DjiDown { until, reason } => {
            if !db::wait_for_dji(db, log, until, &reason).await? {
                superseded(log);
            }
            return Ok(());
        }
        Outcome::Processed(Ok(decoded)) => {
            let gap = organisations::settings(db).await?.session_gap;
            match db::save_result(db, log, decoded.flight.as_ref(), gap).await? {
                Saved::Done => {
                    match &decoded.flight {
                        Some(flight) => tracing::info!(
                            path = log.path,
                            air_time_s = flight.air_time_s,
                            distance_m = flight.distance_m.round(),
                            dji_total_time_s = decoded.dji_total_time_s.round(),
                            dji_total_distance_m = decoded.dji_total_distance_m.round(),
                            "processed log"
                        ),
                        None => tracing::info!(path = log.path, "processed log: no takeoff"),
                    }
                    return Ok(());
                }
                Saved::Refused(reason) => reason,
                Saved::Superseded => {
                    superseded(log);
                    return Ok(());
                }
            }
        }
        Outcome::Processed(Err(reason)) => reason,
    };
    if db::mark_failed(db, log, &failed).await? {
        tracing::warn!(path = log.path, reason = failed, "log failed to process");
    } else {
        superseded(log);
    }
    Ok(())
}

/// A result that came too late: the log changed, or another backend took it.
fn superseded(log: &PendingLog) {
    tracing::info!(
        path = log.path,
        "log changed while processing: processed again"
    );
}

/// One attempt at a log. An error is one that may pass, such as the file
/// not being readable just now or the parser crashing: the log is tried
/// again later.
async fn process(
    db: &mut PgConnection,
    system: &System,
    raw_log: &PendingLog,
    path: &Path,
    dji_app_key: Option<&str>,
    activity: &Activity,
) -> Result<Outcome> {
    let bytes = tokio::fs::read(path)
        .await
        .with_context(|| format!("failed to read {}", raw_log.path))?;
    let log = match blocking(move || DJILog::from_bytes(bytes)).await? {
        Ok(log) => log,
        Err(error) => {
            return Ok(Outcome::Processed(Err(format!(
                "not a readable DJI log: {error}"
            ))));
        }
    };

    // Logs from format version 13 on are encrypted.
    let keychains = if log.version >= 13 {
        match keychains(db, system, raw_log, &log, dji_app_key, activity).await? {
            Ok(keychains) => Some(keychains),
            Err(outcome) => return Ok(outcome),
        }
    } else {
        None
    };

    activity.stage(Stage::Decoding);
    Ok(Outcome::Processed(
        blocking(move || decode(&log, keychains)).await?,
    ))
}

/// The keychains for a log: the ones DJI returned before, or else new ones
/// from DJI, at DJI's shared request rate and only while DJI isn't down.
async fn keychains(
    db: &mut PgConnection,
    system: &System,
    raw_log: &PendingLog,
    log: &DJILog,
    dji_app_key: Option<&str>,
    activity: &Activity,
) -> Result<std::result::Result<Keychains, Outcome>> {
    let request = match log.keychains_request() {
        Ok(request) => request,
        Err(error) => {
            return Ok(Err(Outcome::Processed(Err(format!(
                "failed to read the log's key records: {error}"
            )))));
        }
    };
    let request_body = serde_json::to_string(&request)?;
    // Only for the same request: a new version of the log may need other keys.
    if let Some(body) = db::stored_key_response(db, raw_log.id, &request_body).await?
        && let Ok(Ok(keychains)) = dji::keychains(&body)
    {
        return Ok(Ok(keychains));
    }

    if let Some((until, reason)) = db::dji_blocked(&mut *system.acquire().await?).await? {
        return Ok(Err(Outcome::DjiDown { until, reason }));
    }
    // Not DJI's doing, but the same for the log: it waits, and is tried
    // again a while later.
    let Some(api_key) = dji_app_key else {
        let reason = "DJI_APP_KEY is not set, so encrypted logs can't be processed";
        let until = db::dji_failed(&mut *system.acquire().await?, reason).await?;
        return Ok(Err(Outcome::DjiDown {
            until,
            reason: reason.to_owned(),
        }));
    };

    activity.stage(Stage::RequestingKeys);
    let slot = db::dji_slot(&mut *system.acquire().await?).await?;
    if let Ok(wait) = (slot - Utc::now()).to_std() {
        tokio::time::sleep(wait).await;
    }
    let exchange = {
        let api_key = api_key.to_owned();
        let request_body = request_body.clone();
        blocking(move || dji::send(&api_key, &request_body)).await?
    };
    db::insert_key_request(
        db,
        raw_log.id,
        &request_body,
        exchange.status.map(i32::from),
        exchange.body.as_deref(),
        exchange.error.as_deref(),
    )
    .await?;

    let dji_down = match dji::reply(&exchange) {
        Reply::Answered(answer) => {
            db::dji_answered(&mut *system.acquire().await?).await?;
            return Ok(answer.map_err(|reason| Outcome::Processed(Err(reason))));
        }
        Reply::Down(reason) => reason,
    };
    let until = db::dji_failed(&mut *system.acquire().await?, &dji_down).await?;
    Ok(Err(Outcome::DjiDown {
        until,
        reason: dji_down,
    }))
}

fn decode(log: &DJILog, keychains: Option<Keychains>) -> Processed {
    let records = log
        .records(keychains)
        .map_err(|error| format!("failed to decode the log: {error}"))?;

    let failed = records
        .iter()
        .filter(|record| match record {
            Record::Invalid(_) => true,
            Record::Unknown(record_type, _) => KNOWN_RECORD_TYPES.contains(record_type),
            _ => false,
        })
        .count();
    if failed > 0 {
        return Err(format!("{failed} records failed to decode"));
    }

    // The frames leave out the battery's static record.
    let battery_info = records
        .iter()
        .find_map(|record| match record {
            Record::SmartBatteryGroup(SmartBatteryGroup::SmartBatteryStatic(battery)) => {
                Some(BatteryInfo {
                    design_capacity_mah: i32::try_from(battery.designed_capacity)
                        .ok()
                        .filter(|&mah| mah > 0),
                    cycles: Some(i32::from(battery.loop_times)),
                })
            }
            _ => None,
        })
        .unwrap_or_default();
    let frames = records_to_frames(records, log.details.clone());
    Ok(Box::new(Decoded {
        flight: flight::derive(&frames, Serials::from_details(&log.details), battery_info),
        dji_total_time_s: log.details.total_time,
        dji_total_distance_m: log.details.total_distance,
    }))
}

async fn blocking<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> Result<T> {
    tokio::task::spawn_blocking(f)
        .await
        .context("processing task panicked")
}

#[cfg(test)]
mod tests {
    use super::{MAX_ATTEMPTS, retry_delay_s};

    /// A log that keeps hitting errors waits longer each time, and fails
    /// after its last attempt instead of being tried forever.
    #[test]
    fn retries_slow_down_and_end() {
        let delays: Vec<_> = (1..=MAX_ATTEMPTS).map(retry_delay_s).collect();
        assert_eq!(
            delays,
            [
                Some(60),
                Some(300),
                Some(1800),
                Some(7200),
                Some(21600),
                Some(21600),
                Some(21600),
                None
            ]
        );
    }
}
