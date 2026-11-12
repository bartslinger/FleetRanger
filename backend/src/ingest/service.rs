use chrono::Utc;
use serde::Serialize;
use sqlx::PgConnection;

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

pub async fn status(
    db: &mut PgConnection,
    actor: &Actor,
    activity: &Activity,
) -> ApiResult<Status> {
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
        // The log being processed, and why processing paused, only for the
        // organisation whose log it is: its path names a controller and a
        // flight. The counts are the organisation's own.
        current: now
            .current
            .filter(|current| current.organisation_id == actor.organisation_id)
            .filter(|_| counts.pending > 0),
        paused: now
            .paused
            .filter(|paused| {
                paused
                    .organisation_id
                    .is_none_or(|id| id == actor.organisation_id)
            })
            .filter(|_| counts.pending > 0),
        latest: db::latest_processed(db, since, LATEST).await?,
    })
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;
    use uuid::Uuid;

    use super::status;
    use crate::{events::Changes, ingest::activity::Activity, testing};

    /// The log being processed names a controller and a file, so only the
    /// admins of its own organisation see it, also while another
    /// organisation has logs waiting.
    #[sqlx::test]
    async fn only_its_organisation_sees_the_log_being_processed(pool: PgPool) {
        let mut a = testing::db(&pool).await;
        let a_id = testing::organisation(&mut a).await;
        let admin_a = testing::user(&mut a, "Test Admin A", &["admin"]).await;
        let b_id = testing::second_organisation(&mut a, "Test Org B").await;
        let admin_b: Uuid = sqlx::query_scalar(
            "insert into users (organisation_id, name) values ($1, 'Test Admin B') returning id",
        )
        .bind(b_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        sqlx::query(
            "insert into organisation_roles (organisation_id, user_id, role) values ($1, $2, 'admin')",
        )
        .bind(b_id)
        .bind(admin_b)
        .execute(&pool)
        .await
        .unwrap();
        // A log waiting in each.
        for organisation in [a_id, b_id] {
            sqlx::query(
                "insert into raw_logs (organisation_id, sha256, path, size_bytes)
                 values ($1, sha256(gen_random_uuid()::text::bytea), 'raw/TESTRC/FlightRecord/test.txt', 0)",
            )
            .bind(organisation)
            .execute(&pool)
            .await
            .unwrap();
        }

        let activity = Activity::new(Changes::default());
        activity.start(a_id, "raw/TESTRC/FlightRecord/test.txt");
        let mut as_a = testing::restricted(&pool, Some(a_id)).await;
        let actor_a = testing::actor(&mut as_a, admin_a).await;
        let seen_by_a = status(&mut as_a, &actor_a, &activity).await.ok().unwrap();
        assert!(seen_by_a.current.is_some());
        let mut as_b = testing::restricted(&pool, Some(b_id)).await;
        let actor_b = testing::actor(&mut as_b, admin_b).await;
        let seen_by_b = status(&mut as_b, &actor_b, &activity).await.ok().unwrap();
        assert_eq!(seen_by_b.pending, 1, "B's own waiting log");
        assert!(seen_by_b.current.is_none());
    }
}
