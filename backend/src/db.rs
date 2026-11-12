//! The database, kept apart per organisation ("Several organisations per
//! install" in docs/architecture.md). Requests use [`Db`]: a connection of the
//! database account `fleetranger_app` with the request's organisation set,
//! which row-level security limits to that organisation's rows. The few jobs
//! that look across organisations use [`System`], the account that created the
//! tables, which isn't limited.

use std::ops::{Deref, DerefMut};

use sqlx::pool::PoolConnection;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgConnection, PgPool, Postgres};
use uuid::Uuid;

/// Connections that are limited to one organisation once it is set.
#[derive(Clone)]
pub struct Tenants(PgPool);

impl Tenants {
    /// Connects as `fleetranger_app`. Each connection forgets its
    /// organisation when it goes back to the pool, so the next user sets
    /// their own; one that can't is closed instead.
    pub async fn open(options: PgPoolOptions, url: &str) -> sqlx::Result<Self> {
        let pool = options
            .after_release(|connection, _| {
                Box::pin(async move { Ok(set_organisation(connection, None).await.is_ok()) })
            })
            .connect(url)
            .await?;
        Ok(Self(pool))
    }

    /// Any pool, such as a test's, whose connections then act as if limited:
    /// only `fleetranger_app` is really limited by row-level security.
    #[cfg(test)]
    pub fn of(pool: PgPool) -> Self {
        Self(pool)
    }

    /// A connection limited to this organisation.
    pub async fn connect(&self, organisation_id: Uuid) -> sqlx::Result<Db> {
        let mut connection = self.0.acquire().await?;
        set_organisation(&mut connection, Some(organisation_id)).await?;
        Ok(Db {
            connection,
            organisation_id,
        })
    }
}

/// Sets the organisation that row-level security and the `organisation_id`
/// columns' defaults use, or clears it: then every query on an
/// organisation's table fails.
pub async fn set_organisation(
    connection: &mut PgConnection,
    organisation_id: Option<Uuid>,
) -> sqlx::Result<()> {
    let value = organisation_id.map(|id| id.to_string()).unwrap_or_default();
    sqlx::query("select set_config('app.organisation_id', $1, false)")
        .bind(value)
        .execute(connection)
        .await?;
    Ok(())
}

/// A connection limited to one organisation, for one request or one job.
pub struct Db {
    connection: PoolConnection<Postgres>,
    pub organisation_id: Uuid,
}

impl Deref for Db {
    type Target = PgConnection;

    fn deref(&self) -> &PgConnection {
        &self.connection
    }
}

impl DerefMut for Db {
    fn deref_mut(&mut self) -> &mut PgConnection {
        &mut self.connection
    }
}

/// The database without limits, for the few jobs that look across
/// organisations: finding which organisation a token or login belongs to,
/// taking the next log to process, clean-ups and the command-line tool. Using
/// it anywhere else needs a reason.
#[derive(Clone)]
pub struct System(pub PgPool);

impl Deref for System {
    type Target = PgPool;

    fn deref(&self) -> &PgPool {
        &self.0
    }
}

/// Lets `fleetranger_app` log in with the password in its connection string
/// (`DATABASE_APP_URL`), so that string is the only place it is kept: the
/// deploy generates it, and the account that created the tables sets it on
/// every start.
pub async fn allow_app_login(system: &PgPool, app_url: &str) -> anyhow::Result<()> {
    use anyhow::Context;
    let url: openidconnect::url::Url = app_url.parse().context("DATABASE_APP_URL is not a URL")?;
    anyhow::ensure!(
        url.username() == "fleetranger_app",
        "DATABASE_APP_URL must connect as fleetranger_app"
    );
    let password = url.password().context("DATABASE_APP_URL has no password")?;
    let password = percent_encoding::percent_decode_str(password)
        .decode_utf8()
        .context("DATABASE_APP_URL's password is not UTF-8")?;
    // ALTER ROLE takes no parameters: PostgreSQL quotes the password itself.
    let statement: String = sqlx::query_scalar(
        "select format('alter role fleetranger_app login password %L', $1::text)",
    )
    .bind(password.as_ref())
    .fetch_one(system)
    .await?;
    sqlx::raw_sql(sqlx::AssertSqlSafe(statement))
        .execute(system)
        .await?;
    Ok(())
}

/// The organisation a request is in, left on it by whoever authenticated
/// it (a user's login session, a controller's or an app's token), for its
/// [`Db`].
#[derive(Clone, Copy)]
pub struct RequestOrganisation(pub Uuid);

/// The database keeps organisations apart ("Several organisations per
/// install" in docs/architecture.md), whatever the code asks of it.
#[cfg(test)]
mod tests {
    use sqlx::{AssertSqlSafe, PgPool};
    use uuid::Uuid;

    use crate::testing;

    /// Every table but the migrations' own has row-level security with a
    /// policy for `fleetranger_app`, which owns nothing and can't bypass it:
    /// a table added without one would show every organisation's rows.
    #[sqlx::test]
    async fn every_table_has_row_level_security(pool: PgPool) {
        let unguarded: Vec<String> = sqlx::query_scalar(
            "select c.relname::text from pg_class c
             where c.relnamespace = 'public'::regnamespace and c.relkind = 'r'
               and c.relname <> '_sqlx_migrations'
               and not (c.relrowsecurity and exists (
                   select 1 from pg_policies p
                   where p.schemaname = 'public' and p.tablename = c.relname
                     and 'fleetranger_app' = any(p.roles)))
             order by 1",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(unguarded, Vec::<String>::new());

        let (bypasses, owns): (bool, i64) = sqlx::query_as(
            "select r.rolbypassrls or r.rolsuper,
                    (select count(*) from pg_class c where c.relowner = r.oid)
             from pg_roles r where r.rolname = 'fleetranger_app'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(!bypasses);
        assert_eq!(owns, 0);
    }

    /// As `fleetranger_app` in one organisation, nothing of another's shows,
    /// nothing of it can be changed, and nothing can be written into it;
    /// without an organisation, queries fail.
    #[sqlx::test]
    async fn organisations_see_and_change_only_their_own(pool: PgPool) {
        let mut a = testing::db(&pool).await;
        let general = testing::general(&mut a).await;
        let pilot = testing::member(&mut a, "Test Pilot", general, &["pilot"]).await;
        let raw_log: Uuid = sqlx::query_scalar(
            "insert into raw_logs (sha256, path, size_bytes)
             values (sha256('test'), 'raw/TESTRC/FlightRecord/test.txt', 0) returning id",
        )
        .fetch_one(&mut *a)
        .await
        .unwrap();
        let session: Uuid =
            sqlx::query_scalar("insert into sessions (group_id) values ($1) returning id")
                .bind(general)
                .fetch_one(&mut *a)
                .await
                .unwrap();
        for sql in [
            "insert into logins (user_id, email) values ($1, 'test-pilot@example.org')",
            "insert into flights (raw_log_id, takeoff_at, landed_at, air_time_s, distance_m,
                                  track, pilot_id, group_id, session_id)
             select $2, now(), now(), 60, 100, '{}', $1, $3, $4",
            "insert into session_comments (session_id, author_id, body) values ($4, $1, 'Test')",
            "insert into aircraft (serial, group_id) values ('TESTDRONE', $3)",
            "insert into batteries (serial) values ('TESTBATTERY')",
            "insert into oauth_clients (organisation_id, name, redirect_uris)
             values (current_setting('app.organisation_id')::uuid, 'Test app', '{}')",
            // Self-registered: in no organisation, so every organisation sees it.
            "insert into oauth_clients (name, redirect_uris) values ('Test assistant', '{}')",
        ] {
            sqlx::query(sql)
                .bind(pilot)
                .bind(raw_log)
                .bind(general)
                .bind(session)
                .execute(&mut *a)
                .await
                .unwrap();
        }
        let a_id = testing::organisation(&mut a).await;
        let b_id = testing::second_organisation(&mut a, "Test Org B").await;

        // Each table, as B, shows only B's own rows (and apps of no one).
        let tables: Vec<String> = sqlx::query_scalar(
            "select table_name::text from information_schema.columns
             where table_schema = 'public' and column_name = 'organisation_id'
             order by 1",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        assert!(tables.len() > 20, "tables found: {tables:?}");
        let mut b = testing::restricted(&pool, Some(b_id)).await;
        for table in &tables {
            let own = |rows: &str| {
                format!(
                    "select count(*) from {table} where organisation_id = $1 {}",
                    if table == "oauth_clients" { rows } else { "" }
                )
            };
            let expected: i64 =
                sqlx::query_scalar(AssertSqlSafe(own("or organisation_id is null")))
                    .bind(b_id)
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            let seen: i64 =
                sqlx::query_scalar(AssertSqlSafe(format!("select count(*) from {table}")))
                    .fetch_one(&mut *b)
                    .await
                    .unwrap();
            assert_eq!(seen, expected, "{table} as organisation B");
        }
        let a_flights: i64 =
            sqlx::query_scalar("select count(*) from flights where organisation_id = $1")
                .bind(a_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(a_flights, 1, "A's data is there to hide");

        // A's rows can't be changed or removed from B, apps of no one
        // included, nor B's moved into A, nor rows written into A.
        for sql in [
            "update flights set air_time_s = 1",
            "delete from session_comments",
            "update oauth_clients set name = 'Test taken over'",
            "delete from oauth_clients",
        ] {
            let changed = sqlx::query(sql).execute(&mut *b).await.unwrap();
            assert_eq!(changed.rows_affected(), 0, "{sql}");
        }
        for sql in [
            "update groups set organisation_id = $1",
            "insert into groups (organisation_id, name) values ($1, 'Test sneaked in')",
            "insert into oauth_clients (organisation_id, name, redirect_uris)
             values ($1, 'Test sneaked in', '{}')",
        ] {
            let refused = sqlx::query(sql).bind(a_id).execute(&mut *b).await;
            assert!(refused.is_err(), "{sql}");
        }
        // B's own rows land in B by themselves.
        let landed: Uuid = sqlx::query_scalar(
            "insert into groups (name) values ('Test North') returning organisation_id",
        )
        .fetch_one(&mut *b)
        .await
        .unwrap();
        assert_eq!(landed, b_id);

        let mut nowhere = testing::restricted(&pool, None).await;
        let refused = sqlx::query("select count(*) from flights")
            .execute(&mut *nowhere)
            .await;
        assert!(refused.is_err(), "no organisation, no rows");
    }
}
