use chrono::Utc;
use serde::Serialize;
use sqlx::PgPool;

use super::activity::{Activity, Current, Paused};
use super::db::{self, Processed};
use crate::{
    auth::Actor,
    error::ApiResult,
    policy::{self, Permission},
};

/// How many processed logs the status lists.
const LATEST: i64 = 5;

/// Progress through the current batch of logs, for admins. When no log is
/// pending, the counts and latest logs are the last batch's result (empty if
/// none ran since the backend started).
#[derive(Serialize)]
pub struct Status {
    pub pending: i64,
    /// Processed in this batch, failed ones included.
    pub processed: i64,
    pub failed: i64,
    /// The log being processed now.
    pub current: Option<Current>,
    pub paused: Option<Paused>,
    /// Newest first.
    pub latest: Vec<Processed>,
}

pub async fn status(db: &PgPool, actor: &Actor, activity: &Activity) -> ApiResult<Status> {
    policy::require(actor, Permission::LogsManage, None)?;
    let now = activity.snapshot();
    let empty = |pending| Status {
        pending,
        processed: 0,
        failed: 0,
        current: None,
        paused: None,
        latest: Vec::new(),
    };
    let Some(since) = now.batch_started_at.or(now.finished_batch_started_at) else {
        return Ok(empty(db::batch_counts(db, Utc::now()).await?.pending));
    };
    let counts = db::batch_counts(db, since).await?;
    // New logs before the loop picks up the first: nothing of them is processed yet.
    if now.batch_started_at.is_none() && counts.pending > 0 {
        return Ok(empty(counts.pending));
    }
    Ok(Status {
        pending: counts.pending,
        processed: counts.processed,
        failed: counts.failed,
        // A finished batch: the last log may still show as current for a moment.
        current: now.current.filter(|_| counts.pending > 0),
        paused: now.paused.filter(|_| counts.pending > 0),
        latest: db::latest_processed(db, since, LATEST).await?,
    })
}
