use chrono::Utc;
use serde::Serialize;
use sqlx::PgConnection;

use super::activity::{Activity, Current, Paused};
use super::db::{self, Processed};
use crate::{
    auth::Actor,
    db::System,
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

/// `system` reads DJI's outage, which every backend shares.
pub async fn status(
    db: &mut PgConnection,
    system: &System,
    actor: &Actor,
    activity: &Activity,
) -> ApiResult<Status> {
    policy::require(actor, Permission::LogsManage, None)?;
    let now = activity.snapshot();
    let since = now.batch_started_at.or(now.finished_batch_started_at);
    let counts = db::batch_counts(db, since.unwrap_or_else(Utc::now)).await?;
    // Why this backend, or every backend for DJI, holds the logs: only while
    // some are waiting.
    let paused = match now.paused.filter(|paused| paused.until > Utc::now()) {
        Some(paused) => Some(paused),
        None => db::dji_blocked(&mut *system.acquire().await?)
            .await?
            .map(|(until, reason)| Paused { reason, until }),
    }
    .filter(|_| counts.pending > 0);
    // None ran since the backend started, or new logs before the loop picks
    // up the first: nothing of them is processed yet.
    let Some(since) = since.filter(|_| now.batch_started_at.is_some() || counts.pending == 0)
    else {
        return Ok(Status {
            pending: counts.pending,
            processed: 0,
            failed: 0,
            current: None,
            paused,
            latest: Vec::new(),
        });
    };
    Ok(Status {
        pending: counts.pending,
        processed: counts.processed,
        failed: counts.failed,
        // A finished batch: the last log may still show as current for a moment.
        // The log being processed only for the organisation whose log it
        // is: its path names a controller and a flight. The counts are the
        // organisation's own.
        current: now
            .current
            .filter(|current| current.organisation_id == actor.organisation_id)
            .filter(|_| counts.pending > 0),
        paused,
        latest: db::latest_processed(db, since, LATEST).await?,
    })
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;
    use uuid::Uuid;

    use super::{Status, status};
    use crate::{db::System, events::Changes, ingest::activity::Activity, ingest::db, testing};

    /// Two organisations with an admin and a waiting log each; returns how
    /// each admin sees the status.
    struct TwoOrganisations {
        pool: PgPool,
        /// A pool of its own: restricted connections go back to `pool`
        /// still as `fleetranger_app`.
        system: System,
        a: (Uuid, Uuid),
        b: (Uuid, Uuid),
    }

    impl TwoOrganisations {
        async fn status(&self, (organisation, admin): (Uuid, Uuid), activity: &Activity) -> Status {
            let mut db = testing::restricted(&self.pool, Some(organisation)).await;
            let actor = testing::actor(&mut db, admin).await;
            status(&mut db, &self.system, &actor, activity)
                .await
                .ok()
                .unwrap()
        }
    }

    async fn two_organisations(pool: PgPool) -> TwoOrganisations {
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
        let system = PgPool::connect_with((*pool.connect_options()).clone())
            .await
            .unwrap();
        TwoOrganisations {
            pool,
            system: System(system),
            a: (a_id, admin_a),
            b: (b_id, admin_b),
        }
    }

    /// The log being processed names a controller and a file, so only the
    /// admins of its own organisation see it, also while another
    /// organisation has logs waiting.
    #[sqlx::test]
    async fn only_its_organisation_sees_the_log_being_processed(pool: PgPool) {
        let two = two_organisations(pool).await;
        let activity = Activity::new(Changes::default());
        activity.start(two.a.0, "raw/TESTRC/FlightRecord/test.txt");
        assert!(two.status(two.a, &activity).await.current.is_some());
        let seen_by_b = two.status(two.b, &activity).await;
        assert_eq!(seen_by_b.pending, 1, "B's own waiting log");
        assert!(seen_by_b.current.is_none());
    }

    /// While DJI is down every organisation with logs waiting sees why,
    /// also before this backend took any log and while it processes others.
    #[sqlx::test]
    async fn every_organisation_sees_that_dji_is_down(pool: PgPool) {
        let two = two_organisations(pool).await;
        let activity = Activity::new(Changes::default());
        db::dji_failed(&mut two.system.acquire().await.unwrap(), "DJI answered 503")
            .await
            .unwrap();
        for admin in [two.a, two.b] {
            let paused = two.status(admin, &activity).await.paused;
            assert_eq!(
                paused.map(|paused| paused.reason).as_deref(),
                Some("DJI answered 503")
            );
        }
        activity.start(two.a.0, "raw/TESTRC/FlightRecord/test.txt");
        activity.finished();
        assert!(two.status(two.b, &activity).await.paused.is_some());
    }
}
