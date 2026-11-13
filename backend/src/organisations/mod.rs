//! Organisations and their settings ("Several organisations per install" in
//! docs/architecture.md). The operator creates them with the command-line
//! tool (`cli`); the operator and the organisation's admins (`routes`) change
//! their settings.

pub mod cli;
pub mod routes;

use sqlx::{PgConnection, PgExecutor};

use crate::sessions::SessionGap;

/// An organisation's settings, which were environment variables.
pub struct Settings {
    pub name: String,
    /// All days and times are shown in it, e.g. `Europe/Amsterdam`.
    pub time_zone: String,
    /// The longest break within a session.
    pub session_gap: SessionGap,
    /// Whether AI assistants can query it over MCP.
    pub mcp: bool,
}

/// The settings of the connection's organisation.
pub async fn settings(db: &mut PgConnection) -> sqlx::Result<Settings> {
    let row = sqlx::query!(
        "select o.name, s.time_zone, s.session_gap_minutes, s.mcp
         from organisations o join organisation_settings s on s.organisation_id = o.id
         -- Row-level security shows only this one; tests connect without it.
         where o.id = current_setting('app.organisation_id', true)::uuid"
    )
    .fetch_one(&mut *db)
    .await?;
    Ok(Settings {
        name: row.name,
        time_zone: row.time_zone,
        session_gap: SessionGap(row.session_gap_minutes),
        mcp: row.mcp,
    })
}

/// Whether Postgres knows the time zone, e.g. `Europe/Amsterdam`.
pub async fn known_time_zone<'e>(db: impl PgExecutor<'e>, zone: &str) -> sqlx::Result<bool> {
    sqlx::query_scalar!(
        r#"select exists(select 1 from pg_timezone_names where name = $1) as "known!""#,
        zone
    )
    .fetch_one(db)
    .await
}

/// The time zones to choose from: the regions' names, such as
/// `Europe/Amsterdam`, and UTC, without Postgres's aliases (`posix/…`,
/// `Etc/GMT+1`, `CET`).
pub async fn time_zones(db: &mut PgConnection) -> sqlx::Result<Vec<String>> {
    sqlx::query_scalar!(
        r#"select name as "name!" from pg_timezone_names
           where (name like '%/%' and name !~ '^(posix|Etc|SystemV)/') or name = 'UTC'
           order by name"#
    )
    .fetch_all(&mut *db)
    .await
}
