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

        Ok(Self {
            bind,
            database_url,
            log_dir,
        })
    }
}
