mod auth;
mod config;
mod download;
mod equipment;
mod error;
mod events;
mod flights;
mod health;
mod ingest;
mod pairing;
mod policy;
mod sessions;
mod teams;
mod users;

use std::io::IsTerminal;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use axum::{Router, extract::FromRef, middleware, routing::get};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

use crate::auth::Auth;
use crate::config::Config;
use crate::download::AndroidApk;
use crate::events::Changes;
use crate::ingest::LogDir;
use crate::ingest::activity::Activity;
use crate::sessions::SessionGap;

#[tokio::main]
async fn main() -> Result<()> {
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
    let db = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(3))
        .connect(&config.database_url)
        .await
        .context("failed to connect to the database")?;
    sqlx::migrate!()
        .run(&db)
        .await
        .context("failed to run database migrations")?;

    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.iter().map(String::as_str).collect::<Vec<_>>()[..] {
        [] | ["serve"] => serve(&config, db).await,
        ["import"] => ingest::import::run(&db, &config.log_dir).await,
        ["add-user", email, name] => users::add(&db, email, name, false).await,
        ["add-user", "--admin", email, name] => users::add(&db, email, name, true).await,
        _ => bail!(
            "unknown command {args:?}; expected serve, import or add-user [--admin] <email> <name>"
        ),
    }
}

/// What request handlers can extract with `State`.
#[derive(Clone)]
struct AppState {
    db: PgPool,
    auth: Arc<Auth>,
    activity: Activity,
    changes: Changes,
    session_gap: SessionGap,
    log_dir: LogDir,
    android_apk: AndroidApk,
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

impl FromRef<AppState> for LogDir {
    fn from_ref(state: &AppState) -> Self {
        state.log_dir.clone()
    }
}

impl FromRef<AppState> for SessionGap {
    fn from_ref(state: &AppState) -> Self {
        state.session_gap
    }
}

impl FromRef<AppState> for Activity {
    fn from_ref(state: &AppState) -> Self {
        state.activity.clone()
    }
}

impl FromRef<AppState> for PgPool {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}

impl FromRef<AppState> for Arc<Auth> {
    fn from_ref(state: &AppState) -> Self {
        state.auth.clone()
    }
}

async fn serve(config: &Config, db: PgPool) -> Result<()> {
    let session_gap = SessionGap(config.session_gap_minutes);
    let changes = Changes::default();
    let activity = Activity::new(changes.clone());
    tokio::spawn(ingest::process::run(
        db.clone(),
        config.log_dir.clone(),
        config.dji_app_key.clone(),
        activity.clone(),
        session_gap,
    ));

    let provider = match &config.oidc {
        Some(oidc) => Some(auth::Provider::new(oidc, &config.public_url)?),
        None => {
            tracing::warn!("no login provider configured (OIDC_CLIENT_ID); nobody can log in");
            None
        }
    };
    let state = AppState {
        db,
        auth: Arc::new(Auth {
            public_url: config.public_url.clone(),
            provider,
        }),
        activity,
        changes: changes.clone(),
        session_gap,
        log_dir: LogDir(config.log_dir.clone()),
        android_apk: AndroidApk {
            path: config.android_apk.clone(),
            server: config.public_url.as_str().trim_end_matches('/').to_owned(),
        },
    };

    let app = Router::new()
        .route("/healthz", get(health::check))
        .merge(auth::routes::router())
        .merge(download::router())
        .merge(events::router())
        .merge(flights::routes::router())
        .merge(ingest::routes::router())
        .merge(pairing::routes::router())
        .merge(sessions::routes::router())
        .merge(teams::routes::router())
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
