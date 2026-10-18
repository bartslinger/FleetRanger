use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Serialize)]
pub struct Pilot {
    pub id: Uuid,
    pub name: String,
    pub team_id: Option<Uuid>,
    pub team_name: Option<String>,
    /// The user linked to this pilot.
    pub account_id: Option<Uuid>,
    pub account_email: Option<String>,
    pub archived_at: Option<DateTime<Utc>>,
}

/// What an admin sets on a pilot.
pub struct PilotFields<'a> {
    pub name: &'a str,
    pub team_id: Uuid,
    pub account_id: Option<Uuid>,
}

/// Pilots by name, the archived ones only if asked for.
pub async fn list(db: &PgPool, archived: bool) -> sqlx::Result<Vec<Pilot>> {
    sqlx::query_as!(
        Pilot,
        r#"select p.id, p.name, m.team_id as "team_id?", t.name as "team_name?",
                  p.account_id, a.email as "account_email?", p.archived_at
           from pilots p
           left join team_members m on m.pilot_id = p.id
           left join teams t on t.id = m.team_id
           left join accounts a on a.id = p.account_id
           where $1 or p.archived_at is null
           order by lower(p.name), p.id"#,
        archived,
    )
    .fetch_all(db)
    .await
}

pub async fn get(db: &PgPool, id: Uuid) -> sqlx::Result<Option<Pilot>> {
    sqlx::query_as!(
        Pilot,
        r#"select p.id, p.name, m.team_id as "team_id?", t.name as "team_name?",
                  p.account_id, a.email as "account_email?", p.archived_at
           from pilots p
           left join team_members m on m.pilot_id = p.id
           left join teams t on t.id = m.team_id
           left join accounts a on a.id = p.account_id
           where p.id = $1"#,
        id,
    )
    .fetch_optional(db)
    .await
}

pub async fn insert(db: &PgPool, fields: &PilotFields<'_>) -> sqlx::Result<Uuid> {
    let mut tx = db.begin().await?;
    let id = sqlx::query_scalar!(
        "insert into pilots (name, account_id) values ($1, $2) returning id",
        fields.name,
        fields.account_id,
    )
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query!(
        "insert into team_members (pilot_id, team_id) values ($1, $2)",
        id,
        fields.team_id,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(id)
}

/// False if there is no such pilot.
pub async fn update(db: &PgPool, id: Uuid, fields: &PilotFields<'_>) -> sqlx::Result<bool> {
    let mut tx = db.begin().await?;
    let updated = sqlx::query!(
        "update pilots set name = $2, account_id = $3 where id = $1",
        id,
        fields.name,
        fields.account_id,
    )
    .execute(&mut *tx)
    .await?
    .rows_affected()
        == 1;
    if updated {
        // Past flights keep their team; only new flights follow the pilot.
        sqlx::query!(
            "insert into team_members (pilot_id, team_id) values ($1, $2)
             on conflict (pilot_id) do update set team_id = excluded.team_id",
            id,
            fields.team_id,
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(updated)
}

/// Archives or restores a pilot. False if there is no such pilot.
pub async fn set_archived(db: &PgPool, id: Uuid, archived: bool) -> sqlx::Result<bool> {
    let result = sqlx::query!(
        "update pilots set archived_at = case when $2 then coalesce(archived_at, now()) end
         where id = $1",
        id,
        archived,
    )
    .execute(db)
    .await?;
    Ok(result.rows_affected() == 1)
}

/// Whether the team exists and isn't archived, so pilots can join it.
pub async fn team_is_active(db: &PgPool, team_id: Uuid) -> sqlx::Result<bool> {
    sqlx::query_scalar!(
        r#"select exists(select 1 from teams where id = $1 and archived_at is null) as "exists!""#,
        team_id,
    )
    .fetch_one(db)
    .await
}

pub async fn account_exists(db: &PgPool, account_id: Uuid) -> sqlx::Result<bool> {
    sqlx::query_scalar!(
        r#"select exists(select 1 from accounts where id = $1) as "exists!""#,
        account_id,
    )
    .fetch_one(db)
    .await
}
