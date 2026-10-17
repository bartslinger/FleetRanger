use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::{Context, Result};

/// Runtime configuration, read from environment variables.
pub struct Config {
    /// Address the HTTP server listens on (`FLEETRANGER_BIND`).
    pub bind: SocketAddr,
    /// PostgreSQL connection string (`DATABASE_URL`).
    pub database_url: String,
    /// Directory holding the raw flight logs (`FLEETRANGER_LOG_DIR`).
    pub log_dir: PathBuf,
    /// DJI app key for requesting log decryption keys (`DJI_APP_KEY`). Logs from
    /// format version 13 on can't be processed without it.
    pub dji_app_key: Option<String>,
    /// Serves the flight API without login (`FLEETRANGER_INSECURE_DEV_API=true`).
    /// For local development only, until login exists.
    pub insecure_dev_api: bool,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let bind = std::env::var("FLEETRANGER_BIND")
            .unwrap_or_else(|_| "0.0.0.0:8080".to_owned())
            .parse()
            .context("FLEETRANGER_BIND is not a valid socket address")?;
        let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL is not set")?;
        let log_dir = std::env::var("FLEETRANGER_LOG_DIR")
            .unwrap_or_else(|_| "/data/flightlogs".to_owned())
            .into();
        let dji_app_key = std::env::var("DJI_APP_KEY")
            .ok()
            .filter(|key| !key.is_empty());
        let insecure_dev_api =
            std::env::var("FLEETRANGER_INSECURE_DEV_API").is_ok_and(|value| value == "true");

        Ok(Self {
            bind,
            database_url,
            log_dir,
            dji_app_key,
            insecure_dev_api,
        })
    }
}
