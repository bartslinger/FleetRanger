mod config;
mod health;

use std::io::IsTerminal;
use std::time::Duration;

use anyhow::{Context, Result};
use axum::{Router, routing::get};
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

use crate::config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
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

    let app = Router::new()
        .route("/healthz", get(health::check))
        .with_state(db);

    let listener = TcpListener::bind(config.bind)
        .await
        .with_context(|| format!("failed to listen on {}", config.bind))?;
    tracing::info!("listening on {}", config.bind);
    axum::serve(listener, app)
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
