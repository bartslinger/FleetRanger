use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::{Context, Result};
use openidconnect::url::Url;

/// Runtime configuration, read from environment variables.
pub struct Config {
    /// Address the HTTP server listens on (`FLEETRANGER_BIND`).
    pub bind: SocketAddr,
    /// PostgreSQL connection string (`DATABASE_URL`), as the account that
    /// created the tables: for database changes and work across organisations.
    pub database_url: String,
    /// The same database as `fleetranger_app`, which row-level security
    /// limits to one organisation (`DATABASE_APP_URL`), for requests. Its
    /// password is set from here on every start.
    pub database_app_url: String,
    /// Directory holding the raw flight logs (`FLEETRANGER_LOG_DIR`).
    pub log_dir: PathBuf,
    /// Raw logs go to this S3 bucket instead of the log directory.
    pub s3_bucket: Option<String>,
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
    /// Whether the install offers MCP at `/mcp` (`FLEETRANGER_MCP=on`); off
    /// by default. Each organisation then switches it on for itself.
    pub mcp: bool,
    /// Whether apps can use the public API at `/api/v1`
    /// (`FLEETRANGER_API=on`); off by default.
    pub api: bool,
    /// Whether the install holds several organisations, as the hosted
    /// service does (`FLEETRANGER_MULTI_TENANT=on`); off by default: one.
    pub multi_tenant: bool,
    /// Whether anyone can create an organisation by signing up
    /// (`FLEETRANGER_SIGNUP=on`), as on the hosted service; off by default.
    /// Needs `FLEETRANGER_MULTI_TENANT=on`.
    pub signup: Option<Signup>,
    /// The Sentry DSN the controller app sends its error reports to
    /// (`FLEETRANGER_ERROR_REPORTS`); none by default.
    pub error_reports: Option<Url>,
}

/// Self-service sign-up.
#[derive(Clone)]
pub struct Signup {
    /// At most this many organisations that aren't archived
    /// (`FLEETRANGER_SIGNUP_CAP`, 200 by default); after that, people go on
    /// the waiting list.
    pub cap: i64,
    /// The terms of use and the privacy policy people accept when signing up
    /// (`FLEETRANGER_TERMS_URL`, `FLEETRANGER_PRIVACY_URL`).
    pub terms_url: Url,
    pub privacy_url: Url,
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
        let database_app_url =
            std::env::var("DATABASE_APP_URL").context("DATABASE_APP_URL is not set")?;
        let log_dir = std::env::var("FLEETRANGER_LOG_DIR")
            .unwrap_or_else(|_| "/data/flightlogs".to_owned())
            .into();
        let s3_bucket = non_empty_var("FLEETRANGER_S3_BUCKET");
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
        let error_reports = non_empty_var("FLEETRANGER_ERROR_REPORTS")
            .map(|dsn| Url::parse(&dsn))
            .transpose()
            .context("FLEETRANGER_ERROR_REPORTS is not a valid DSN")?;
        let mcp = switch("FLEETRANGER_MCP")?;
        let api = switch("FLEETRANGER_API")?;
        let multi_tenant = switch("FLEETRANGER_MULTI_TENANT")?;
        let signup = if switch("FLEETRANGER_SIGNUP")? {
            anyhow::ensure!(
                multi_tenant,
                "FLEETRANGER_SIGNUP=on needs FLEETRANGER_MULTI_TENANT=on"
            );
            let url = |name: &str| -> Result<Url> {
                non_empty_var(name)
                    .with_context(|| format!("FLEETRANGER_SIGNUP=on needs {name}"))?
                    .parse()
                    .with_context(|| format!("{name} is not a valid URL"))
            };
            Some(Signup {
                cap: match non_empty_var("FLEETRANGER_SIGNUP_CAP") {
                    Some(cap) => cap
                        .parse()
                        .ok()
                        .filter(|cap| *cap >= 0)
                        .context("FLEETRANGER_SIGNUP_CAP is not a number of organisations")?,
                    None => 200,
                },
                terms_url: url("FLEETRANGER_TERMS_URL")?,
                privacy_url: url("FLEETRANGER_PRIVACY_URL")?,
            })
        } else {
            None
        };

        Ok(Self {
            bind,
            database_url,
            database_app_url,
            log_dir,
            s3_bucket,
            dji_app_key,
            public_url,
            oidc,
            android_apk,
            mcp,
            api,
            multi_tenant,
            signup,
            error_reports,
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
