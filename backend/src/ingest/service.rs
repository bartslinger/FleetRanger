use chrono::Utc;
use serde::Serialize;
use sqlx::PgPool;

use super::activity::{Activity, Current, Paused};
use super::db::{self, Processed};
use crate::{auth::Actor, error::ApiResult, policy};

/// How many processed logs the status lists.
const LATEST: i64 = 5;

/// Progress through the current batch of logs, for admins. Everything is empty
/// when no log is pending.
#[derive(Serialize)]
pub struct Status {
    pub pending: i64,
    /// Processed so far in this batch, failed ones included.
    pub processed: i64,
    pub failed: i64,
    /// The log being processed now.
    pub current: Option<Current>,
    pub paused: Option<Paused>,
    /// Newest first.
    pub latest: Vec<Processed>,
}

pub async fn status(db: &PgPool, actor: &Actor, activity: &Activity) -> ApiResult<Status> {
    policy::require_admin(actor)?;
    let now = activity.snapshot();
    // Before the loop picks up the first log, nothing is processed yet.
    let since = now.batch_started_at.unwrap_or_else(Utc::now);
    let counts = db::batch_counts(db, since).await?;
    if counts.pending == 0 {
        return Ok(Status {
            pending: 0,
            processed: 0,
            failed: 0,
            current: None,
            paused: None,
            latest: Vec::new(),
        });
    }
    Ok(Status {
        pending: counts.pending,
        processed: counts.processed,
        failed: counts.failed,
        current: now.current,
        paused: now.paused,
        latest: db::latest_processed(db, since, LATEST).await?,
    })
}
