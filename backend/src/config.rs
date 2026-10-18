use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::{Context, Result};
use openidconnect::url::Url;

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
    /// Where browsers reach the install (`FLEETRANGER_PUBLIC_URL`), e.g.
    /// `https://fleet.example.org`. Login redirects back to it, and changes are
    /// only accepted from its origin.
    pub public_url: Url,
    /// The login provider. Without it nobody can log in.
    pub oidc: Option<OidcConfig>,
}

pub struct OidcConfig {
    /// `OIDC_ISSUER`, Google by default.
    pub issuer: String,
    /// `OIDC_CLIENT_ID`.
    pub client_id: String,
    /// `OIDC_CLIENT_SECRET`.
    pub client_secret: String,
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
        let dji_app_key = non_empty_var("DJI_APP_KEY");
        let public_url = std::env::var("FLEETRANGER_PUBLIC_URL")
            .unwrap_or_else(|_| "http://localhost:5173".to_owned())
            .parse()
            .context("FLEETRANGER_PUBLIC_URL is not a valid URL")?;
        let oidc = match non_empty_var("OIDC_CLIENT_ID") {
            Some(client_id) => Some(OidcConfig {
                issuer: non_empty_var("OIDC_ISSUER")
                    .unwrap_or_else(|| "https://accounts.google.com".to_owned()),
                client_id,
                client_secret: non_empty_var("OIDC_CLIENT_SECRET")
                    .context("OIDC_CLIENT_ID is set but OIDC_CLIENT_SECRET is not")?,
            }),
            None => None,
        };

        Ok(Self {
            bind,
            database_url,
            log_dir,
            dji_app_key,
            public_url,
            oidc,
        })
    }
}

fn non_empty_var(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|value| !value.is_empty())
}
