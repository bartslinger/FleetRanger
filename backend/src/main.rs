mod config;
mod health;
mod ingest;

use std::io::IsTerminal;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use axum::{Router, routing::get};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

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

    match std::env::args().nth(1).as_deref() {
        None | Some("serve") => serve(&config, db).await,
        Some("import") => ingest::import::run(&db, &config.log_dir).await,
        Some(other) => bail!("unknown command {other:?}; expected serve or import"),
    }
}

async fn serve(config: &Config, db: PgPool) -> Result<()> {
    tokio::spawn(ingest::process::run(
        db.clone(),
        config.log_dir.clone(),
        config.dji_app_key.clone(),
    ));

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
