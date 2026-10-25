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
    /// The controller app the download page offers (`FLEETRANGER_ANDROID_APK`,
    /// `/data/android/FleetRanger.apk` by default); without it the page says so.
    pub android_apk: PathBuf,
    /// The longest break within a session, in minutes
    /// (`FLEETRANGER_SESSION_GAP_MINUTES`, 120 by default).
    pub session_gap_minutes: i32,
    /// The organisation's time zone (`TIME_ZONE`, UTC by default), e.g.
    /// `Europe/Amsterdam`: days in MCP requests are days there.
    pub time_zone: String,
    /// Whether AI assistants can query the install over MCP
    /// (`FLEETRANGER_MCP=on`); off by default.
    pub mcp: bool,
    /// Whether apps can use the public API at `/api/v1`
    /// (`FLEETRANGER_API=on`); off by default.
    pub api: bool,
    /// The organisation's name, for apps (`FLEETRANGER_ORGANISATION_NAME`),
    /// until organisations have settings of their own.
    pub organisation_name: Option<String>,
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
        let android_apk = non_empty_var("FLEETRANGER_ANDROID_APK")
            .unwrap_or_else(|| "/data/android/FleetRanger.apk".to_owned())
            .into();
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
        let session_gap_minutes =
            match non_empty_var("FLEETRANGER_SESSION_GAP_MINUTES") {
                Some(value) => value.parse().ok().filter(|minutes| *minutes > 0).context(
                    "FLEETRANGER_SESSION_GAP_MINUTES is not a positive number of minutes",
                )?,
                None => 120,
            };
        let time_zone = non_empty_var("TIME_ZONE").unwrap_or_else(|| "UTC".to_owned());
        let mcp = switch("FLEETRANGER_MCP")?;
        let api = switch("FLEETRANGER_API")?;
        let organisation_name = non_empty_var("FLEETRANGER_ORGANISATION_NAME");

        Ok(Self {
            bind,
            database_url,
            log_dir,
            dji_app_key,
            public_url,
            oidc,
            android_apk,
            session_gap_minutes,
            time_zone,
            mcp,
            api,
            organisation_name,
        })
    }
}

fn non_empty_var(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|value| !value.is_empty())
}

/// A feature an install switches on: `on`, or `off` (the default).
fn switch(name: &str) -> Result<bool> {
    match non_empty_var(name).as_deref() {
        None | Some("off") => Ok(false),
        Some("on") => Ok(true),
        Some(_) => anyhow::bail!("{name} must be on or off"),
    }
}
