//! Organisations and their settings ("Several organisations per install" in
//! docs/architecture.md). The operator creates them and sets their settings
//! with the command-line tool (`cli`); requests read their own organisation's.

pub mod cli;

use sqlx::PgConnection;

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
