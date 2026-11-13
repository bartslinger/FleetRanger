//! Getting raw flight logs into the system and processing them into flights.

use std::path::PathBuf;

pub mod activity;
mod db;
mod dji;
mod flight;
pub mod import;
pub mod parser;
pub mod process;
pub mod routes;
pub mod service;
mod upload;

/// The directory holding the raw flight logs (`FLEETRANGER_LOG_DIR`).
#[derive(Clone)]
pub struct LogDir(pub PathBuf);

/// Where an organisation's raw logs are: a directory of its own under the
/// log directory, named by its id, so they go with it (user's decision,
/// 2026-11-12). Paths stored with the logs are relative to it.
pub fn organisation_dir(log_dir: &std::path::Path, organisation_id: uuid::Uuid) -> PathBuf {
    log_dir.join(organisation_id.to_string())
}
