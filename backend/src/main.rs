mod accounts;
mod auth;
mod config;
mod equipment;
mod error;
mod flights;
mod health;
mod ingest;
mod pilots;
mod policy;
mod teams;

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
        ["add-account", email] => accounts::add(&db, email, false).await,
        ["add-account", "--admin", email] => accounts::add(&db, email, true).await,
        _ => bail!(
            "unknown command {args:?}; expected serve, import or add-account [--admin] <email>"
        ),
    }
}

/// What request handlers can extract with `State`.
#[derive(Clone)]
struct AppState {
    db: PgPool,
    auth: Arc<Auth>,
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
    tokio::spawn(ingest::process::run(
        db.clone(),
        config.log_dir.clone(),
        config.dji_app_key.clone(),
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
    };

    let app = Router::new()
        .route("/healthz", get(health::check))
        .merge(auth::routes::router())
        .merge(flights::routes::router())
        .merge(teams::routes::router())
        .merge(pilots::routes::router())
        .merge(equipment::routes::router())
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
    .with_graceful_shutdown(shutdown_signal())
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
