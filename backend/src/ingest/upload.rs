//! Logs uploaded by paired controllers. A log is stored as
//! `raw/<controller serial>/FlightRecord/<file name>`, the name it had on the
//! controller; see "Data" in docs/architecture.md.

use std::path::Path;

use anyhow::Context;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use super::db::{self, Stored};
use crate::{
    error::{ApiError, ApiResult},
    pairing::{Controller, parse_sha256},
};

/// Who was chosen on the controller before the log was written.
pub struct Chosen {
    pub pilot_id: Option<Uuid>,
    /// The group chosen with the pilot (still called a team on the
    /// controller).
    pub group_id: Option<Uuid>,
}

/// Checks an uploaded log against the hash the controller computed, and
/// stores it unless the same content is stored already.
pub async fn receive(
    db: &PgPool,
    log_dir: &Path,
    controller: &Controller,
    file_name: &str,
    sha256_hex: &str,
    chosen: Chosen,
    body: &[u8],
) -> ApiResult<Stored> {
    if !is_log_name(file_name) {
        return Err(ApiError::Invalid("Send the log's file name.".to_owned()));
    }
    let claimed = parse_sha256(sha256_hex)
        .ok_or_else(|| ApiError::Invalid("Send the log's SHA-256 in hex.".to_owned()))?;
    let sha256 = Sha256::digest(body);
    if sha256.as_slice() != claimed.as_slice() {
        // Cut off on the way: the controller sends it again.
        return Err(ApiError::Invalid(
            "The log doesn't match its SHA-256.".to_owned(),
        ));
    }
    if db::has_content(db, &sha256).await? {
        return Ok(Stored::Duplicate);
    }
    if !controller.serial.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(ApiError::Internal(anyhow::anyhow!(
            "controller serial {:?} can't be a directory name",
            controller.serial
        )));
    }

    let relative = format!("raw/{}/FlightRecord/{file_name}", controller.serial);
    write(&log_dir.join(&relative), body)
        .await
        .with_context(|| format!("failed to store {relative}"))
        .map_err(ApiError::Internal)?;
    let size = i64::try_from(body.len()).map_err(|error| ApiError::Internal(error.into()))?;
    Ok(db::save_upload(
        db,
        &sha256,
        &relative,
        size,
        chosen.pilot_id,
        chosen.group_id,
    )
    .await?)
}

/// At most this many hashes per question; the controller asks in parts.
pub const MAX_HASHES: usize = 1000;

/// Which of these logs, by SHA-256 in hex, aren't stored yet: the controller
/// asks before uploading, so it only sends what's missing. Answers in the
/// controller's own spelling of each hash.
pub async fn missing(db: &PgPool, sha256s_hex: Vec<String>) -> ApiResult<Vec<String>> {
    if sha256s_hex.len() > MAX_HASHES {
        return Err(ApiError::Invalid(format!(
            "Ask about at most {MAX_HASHES} logs at a time."
        )));
    }
    let sha256s = sha256s_hex
        .iter()
        .map(|hex| parse_sha256(hex))
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| ApiError::Invalid("Send each log's SHA-256 in hex.".to_owned()))?;
    let stored = db::stored_contents(db, &sha256s).await?;
    Ok(sha256s_hex
        .into_iter()
        .zip(sha256s)
        .filter(|(_, sha256)| !stored.contains(sha256))
        .map(|(hex, _)| hex)
        .collect())
}

/// A file name as DJI Pilot 2 writes them, with nothing that could leave the
/// controller's directory.
fn is_log_name(name: &str) -> bool {
    name.len() <= 200
        && name.ends_with(".txt")
        && !name.starts_with('.')
        && !name.contains(['/', '\\'])
}

/// Writes next to the destination first and then renames, so the processing
/// loop never reads a half-written file.
async fn write(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let dir = path.parent().expect("a log path has a directory");
    tokio::fs::create_dir_all(dir).await?;
    let part = dir.join(format!(
        ".{}.part",
        path.file_name()
            .expect("a log path has a file name")
            .to_string_lossy()
    ));
    tokio::fs::write(&part, bytes).await?;
    tokio::fs::rename(&part, path).await
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Only a plain file name: the upload must stay in its controller's folder.
    #[test]
    fn log_names_stay_in_their_folder() {
        assert!(is_log_name("DJIFlightRecord_2026-10-12_[18-32-10].txt"));
        assert!(!is_log_name("../../etc/passwd.txt"));
        assert!(!is_log_name("a/b.txt"));
        assert!(!is_log_name(".hidden.txt"));
        assert!(!is_log_name("log.bin"));
    }
}
