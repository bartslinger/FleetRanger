//! Getting raw flight logs into the system and processing them into flights.

use std::path::PathBuf;

pub mod activity;
mod db;
mod dji;
mod flight;
pub mod import;
pub mod process;
pub mod routes;
pub mod service;
mod upload;

/// The directory holding the raw flight logs (`FLEETRANGER_LOG_DIR`).
#[derive(Clone)]
pub struct LogDir(pub PathBuf);
