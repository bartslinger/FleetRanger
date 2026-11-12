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
