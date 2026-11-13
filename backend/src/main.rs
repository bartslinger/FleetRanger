mod api_v1;
mod areas;
mod auth;
mod batteries;
mod config;
mod db;
mod download;
mod equipment;
mod error;
mod events;
mod flights;
mod groups;
mod health;
mod ingest;
mod mcp;
mod oauth;
mod organisations;
mod pairing;
mod policy;
mod sessions;
#[cfg(test)]
mod testing;
mod users;

use std::io::IsTerminal;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use axum::{Router, extract::FromRef, middleware, routing::get};
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

use crate::auth::Auth;
use crate::config::Config;
use crate::download::AndroidApk;
use crate::events::Changes;
use crate::ingest::activity::Activity;
use crate::ingest::store::LogStore;
use crate::oauth::OAuth;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // The log parser's child process needs none of what follows: no
    // runtime, no log lines on its output, no database.
    if let Some(code) = ingest::parser::run_child(&args) {
        std::process::exit(code);
    }
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("failed to start the runtime")?
        .block_on(start(args))
}

async fn start(args: Vec<String>) -> Result<()> {
    // TLS for S3 (reqwest, through object_store) uses ring, as ureq does.
    if rustls::crypto::ring::default_provider()
        .install_default()
        .is_err()
    {
        anyhow::bail!("failed to set up TLS");
    }
    tracing_subscriber::fmt()
        // Postgres notices ("relation already exists, skipping" on every start) are noise.
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx::postgres::notice=warn".into()),
        )
        .with_ansi(std::io::stdout().is_terminal())
        .init();

    let config = Config::from_env()?;

    // A short acquire timeout makes health checks fail fast when the database is down.
    let system = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(3))
        .connect(&config.database_url)
        .await
        .context("failed to connect to the database")?;
    sqlx::migrate!()
        .run(&system)
        .await
        .context("failed to run database migrations")?;
    db::allow_app_login(&system, &config.database_app_url).await?;
    let tenants = db::Tenants::open(
        PgPoolOptions::new()
            .acquire_timeout(Duration::from_secs(3))
            .max_connections(20),
        &config.database_app_url,
    )
    .await
    .context("failed to connect to the database as fleetranger_app")?;
    let system = db::System(system);
    let store = LogStore::open(config.log_dir.clone(), config.s3_bucket.as_deref())?;

    match args.iter().map(String::as_str).collect::<Vec<_>>()[..] {
        [] | ["serve"] => serve(&config, system, tenants, store).await,
        _ => {
            let multi_tenant = organisations::MultiTenant(config.multi_tenant);
            organisations::cli::run(&args, &system, &tenants, &store, multi_tenant).await
        }
    }
}

/// What request handlers can extract with `State`.
#[derive(Clone)]
struct AppState {
    system: db::System,
    tenants: db::Tenants,
    auth: Arc<Auth>,
    activity: Activity,
    changes: Changes,
    apps: oauth::AppsOn,
    multi_tenant: organisations::MultiTenant,
    store: LogStore,
    android_apk: AndroidApk,
    oauth: OAuth,
}

impl FromRef<AppState> for OAuth {
    fn from_ref(state: &AppState) -> Self {
        state.oauth.clone()
    }
}

impl FromRef<AppState> for Changes {
    fn from_ref(state: &AppState) -> Self {
        state.changes.clone()
    }
}

impl FromRef<AppState> for AndroidApk {
    fn from_ref(state: &AppState) -> Self {
        state.android_apk.clone()
    }
}

impl FromRef<AppState> for LogStore {
    fn from_ref(state: &AppState) -> Self {
        state.store.clone()
    }
}

impl FromRef<AppState> for organisations::MultiTenant {
    fn from_ref(state: &AppState) -> Self {
        state.multi_tenant
    }
}

impl FromRef<AppState> for oauth::AppsOn {
    fn from_ref(state: &AppState) -> Self {
        state.apps
    }
}

impl FromRef<AppState> for Activity {
    fn from_ref(state: &AppState) -> Self {
        state.activity.clone()
    }
}

impl FromRef<AppState> for db::System {
    fn from_ref(state: &AppState) -> Self {
        state.system.clone()
    }
}

impl FromRef<AppState> for db::Tenants {
    fn from_ref(state: &AppState) -> Self {
        state.tenants.clone()
    }
}

impl FromRef<AppState> for Arc<Auth> {
    fn from_ref(state: &AppState) -> Self {
        state.auth.clone()
    }
}

async fn serve(
    config: &Config,
    system: db::System,
    tenants: db::Tenants,
    store: LogStore,
) -> Result<()> {
    tracing::info!("raw logs on {}", store.describe());
    store.remove_old_scratch();
    let changes = Changes::default();
    let activity = Activity::new(changes.clone());
    tokio::spawn(ingest::process::run(
        system.clone(),
        tenants.clone(),
        store.clone(),
        config.dji_app_key.clone(),
        activity.clone(),
    ));

    let provider = match &config.oidc {
        Some(oidc) => Some(auth::Provider::new(oidc, &config.public_url)?),
        None => {
            tracing::warn!("no login provider configured (OIDC_CLIENT_ID); nobody can log in");
            None
        }
    };
    let issuer = config.public_url.as_str().trim_end_matches('/').to_owned();
    // The resources OAuth issues tokens for: /mcp and /api/v1, when the
    // install has them on (FLEETRANGER_MCP, FLEETRANGER_API); each
    // organisation then switches MCP on for itself (user's decision,
    // 2026-11-12). OAuth runs only with one of them.
    let mut resources = Vec::new();
    if config.mcp {
        resources.push(oauth::Resource {
            url: mcp::resource(&issuer),
            scopes: oauth::MCP_SCOPES,
        });
    }
    if config.api {
        resources.push(oauth::Resource {
            url: api_v1::resource(&issuer),
            scopes: oauth::API_SCOPES,
        });
    }
    auth::set_https(config.public_url.scheme() == "https");
    let state = AppState {
        system,
        tenants,
        auth: Arc::new(Auth {
            public_url: config.public_url.clone(),
            provider,
        }),
        activity,
        changes: changes.clone(),
        apps: oauth::AppsOn {
            mcp: config.mcp,
            api: config.api,
        },
        multi_tenant: organisations::MultiTenant(config.multi_tenant),
        store,
        android_apk: AndroidApk {
            path: config.android_apk.clone(),
            server: issuer.clone(),
        },
        oauth: OAuth {
            issuer: issuer.clone(),
            resources: resources.into(),
        },
    };

    let mut apps = Router::new();
    if config.mcp {
        let mut hosts = vec!["localhost".to_owned(), "127.0.0.1".to_owned()];
        hosts.extend(config.public_url.host_str().map(str::to_owned));
        tracing::info!("MCP server on at /mcp");
        apps = apps.merge(mcp::router(
            state.system.clone(),
            state.tenants.clone(),
            &issuer,
            hosts,
        ));
    }
    if config.api {
        tracing::info!("public API on at /api/v1");
        apps = apps.merge(api_v1::router(api_v1::Api {
            system: state.system.clone(),
            tenants: state.tenants.clone(),
            resource: api_v1::resource(&issuer).into(),
        }));
    }
    if config.mcp || config.api {
        tokio::spawn(oauth::clean_up_hourly(state.system.clone()));
        apps = apps.merge(oauth::routes::router());
    }

    let app = Router::new()
        .route("/healthz", get(health::check))
        .merge(apps)
        .merge(areas::routes::router())
        .merge(auth::routes::router())
        .merge(batteries::routes::router())
        .merge(download::router())
        .merge(events::router())
        .merge(flights::routes::router())
        .merge(groups::routes::router())
        .merge(organisations::routes::router())
        .merge(ingest::routes::router())
        .merge(pairing::routes::router())
        .merge(sessions::routes::router())
        .merge(equipment::routes::router())
        .merge(users::routes::router())
        .layer(middleware::from_fn_with_state(
            state.auth.clone(),
            auth::same_origin,
        ))
        .with_state(state);

    let listener = TcpListener::bind(config.bind)
        .await
        .with_context(|| format!("failed to listen on {}", config.bind))?;
    tracing::info!("listening on {}", config.bind);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async move {
        shutdown_signal().await;
        // Open event streams would keep the shutdown waiting.
        changes.close();
    })
    .await?;

    Ok(())
}

/// Resolves on Ctrl-C or SIGTERM, which is what `docker stop` sends.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.ok();
    };
    let terminate = async {
        if let Ok(mut signal) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            signal.recv().await;
        }
    };
    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
    tracing::info!("shutting down");
}
