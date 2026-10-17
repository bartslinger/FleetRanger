//! Processes pending raw logs, one at a time: requests decryption keys from DJI
//! when needed, decrypts and parses the log, and saves the flight in it.

use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use dji_log_parser::DJILog;
use dji_log_parser::frame::records_to_frames;
use dji_log_parser::keychain::KeychainFeaturePoint;
use dji_log_parser::record::Record;
use sqlx::PgPool;
use sqlx::types::Uuid;

use super::flight::{self, Flight, Serials};
use super::{db, dji};

/// How long to wait before looking for new logs when none are pending.
const IDLE_PAUSE: Duration = Duration::from_secs(10);
/// How long to wait after a problem that may go away, such as DJI being unreachable.
const ERROR_PAUSE: Duration = Duration::from_secs(60);
/// How long to wait after each request to DJI, so that a large import doesn't
/// flood their API.
const DJI_REQUEST_PAUSE: Duration = Duration::from_secs(2);

/// The record types the parser knows (the `magic` values in its `record/mod.rs`).
/// A record of one of these types that comes out as `Unknown` failed to decode.
const KNOWN_RECORD_TYPES: [u8; 24] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 14, 15, 19, 22, 24, 25, 33, 40, 49, 50, 56, 62,
];

/// The result of processing one log: what it contains, or why it can't be processed.
/// Problems that may go away are errors instead, and leave the log pending.
type Processed = std::result::Result<Decoded, String>;

struct Decoded {
    /// `None` if the aircraft never took off.
    flight: Option<Flight>,
    /// The summary DJI's app wrote into the log, logged for comparison with ours.
    dji_total_time_s: f64,
    dji_total_distance_m: f32,
}

pub async fn run(db: PgPool, log_dir: PathBuf, dji_app_key: Option<String>) {
    loop {
        let pause = match process_next(&db, &log_dir, dji_app_key.as_deref()).await {
            Ok(true) => continue,
            Ok(false) => IDLE_PAUSE,
            Err(error) => {
                tracing::warn!("log processing paused: {error:#}");
                ERROR_PAUSE
            }
        };
        tokio::time::sleep(pause).await;
    }
}

/// Processes the oldest pending log. Returns false if there was none.
async fn process_next(db: &PgPool, log_dir: &Path, dji_app_key: Option<&str>) -> Result<bool> {
    let Some(log) = db::next_pending(db).await? else {
        return Ok(false);
    };

    let processed = process(db, log.id, &log_dir.join(&log.path), dji_app_key)
        .await
        .with_context(|| format!("while processing {}", log.path))?;
    match processed {
        Ok(decoded) => {
            db::save_result(db, log.id, decoded.flight.as_ref()).await?;
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
        }
        Err(reason) => {
            db::mark_failed(db, log.id, &reason).await?;
            tracing::warn!(path = log.path, reason, "log failed to process");
        }
    }
    Ok(true)
}

async fn process(
    db: &PgPool,
    raw_log_id: Uuid,
    path: &Path,
    dji_app_key: Option<&str>,
) -> Result<Processed> {
    let bytes = match tokio::fs::read(path).await {
        Ok(bytes) => bytes,
        Err(error) => return Ok(Err(format!("failed to read the file: {error}"))),
    };
    let log = match blocking(move || DJILog::from_bytes(bytes)).await? {
        Ok(log) => log,
        Err(error) => return Ok(Err(format!("not a readable DJI log: {error}"))),
    };

    // Logs from format version 13 on are encrypted.
    let keychains = if log.version >= 13 {
        match keychains(db, raw_log_id, &log, dji_app_key).await? {
            Ok(keychains) => Some(keychains),
            Err(reason) => return Ok(Err(reason)),
        }
    } else {
        None
    };

    blocking(move || decode(&log, keychains)).await
}

/// The keychains for a log: the ones DJI returned before, or else new ones from DJI.
async fn keychains(
    db: &PgPool,
    raw_log_id: Uuid,
    log: &DJILog,
    dji_app_key: Option<&str>,
) -> Result<std::result::Result<Vec<Vec<KeychainFeaturePoint>>, String>> {
    if let Some(body) = db::stored_key_response(db, raw_log_id).await?
        && let Ok(keychains) = dji::keychains(&body)
    {
        return Ok(Ok(keychains));
    }

    let Some(api_key) = dji_app_key else {
        bail!("DJI_APP_KEY is not set, so encrypted logs can't be processed");
    };
    let request = match log.keychains_request() {
        Ok(request) => request,
        Err(error) => {
            return Ok(Err(format!(
                "failed to read the log's key records: {error}"
            )));
        }
    };
    let request_body = serde_json::to_string(&request)?;

    let exchange = {
        let api_key = api_key.to_owned();
        let request_body = request_body.clone();
        blocking(move || dji::send(&api_key, &request_body)).await?
    };
    tokio::time::sleep(DJI_REQUEST_PAUSE).await;
    db::insert_key_request(
        db,
        raw_log_id,
        &request_body,
        exchange.status.map(i32::from),
        exchange.body.as_deref(),
        exchange.error.as_deref(),
    )
    .await?;

    match (exchange.status, exchange.body) {
        (Some(200), Some(body)) => Ok(dji::keychains(&body).map_err(|error| format!("{error:#}"))),
        (None, _) => bail!("DJI did not answer: {}", exchange.error.unwrap_or_default()),
        // A rejected app key, rate limiting or trouble at DJI: none of it is the log's fault.
        (Some(status @ (401 | 403 | 429 | 500..)), _) => bail!("DJI answered {status}"),
        (Some(status), _) => Ok(Err(format!("DJI answered {status}"))),
    }
}

fn decode(log: &DJILog, keychains: Option<Vec<Vec<KeychainFeaturePoint>>>) -> Processed {
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

    let frames = records_to_frames(records, log.details.clone());
    Ok(Decoded {
        flight: flight::derive(&frames, Serials::from_details(&log.details)),
        dji_total_time_s: log.details.total_time,
        dji_total_distance_m: log.details.total_distance,
    })
}

async fn blocking<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> Result<T> {
    tokio::task::spawn_blocking(f)
        .await
        .context("processing task panicked")
}
