use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Serialize)]
pub struct Team {
    pub id: Uuid,
    pub name: String,
    pub archived_at: Option<DateTime<Utc>>,
    /// Users in the team that aren't archived.
    pub members: i64,
}

/// Teams by name, the archived ones only if asked for.
pub async fn list(db: &PgPool, archived: bool) -> sqlx::Result<Vec<Team>> {
    sqlx::query_as!(
        Team,
        r#"select t.id, t.name, t.archived_at,
                  (select count(*) from team_members m join users u on u.id = m.user_id
                   where m.team_id = t.id and u.archived_at is null) as "members!"
           from teams t
           where $1 or t.archived_at is null
           order by lower(t.name), t.id"#,
        archived,
    )
    .fetch_all(db)
    .await
}

pub async fn get(db: &PgPool, id: Uuid) -> sqlx::Result<Option<Team>> {
    sqlx::query_as!(
        Team,
        r#"select t.id, t.name, t.archived_at,
                  (select count(*) from team_members m join users u on u.id = m.user_id
                   where m.team_id = t.id and u.archived_at is null) as "members!"
           from teams t
           where t.id = $1"#,
        id,
    )
    .fetch_optional(db)
    .await
}

pub async fn insert(db: &PgPool, name: &str) -> sqlx::Result<Uuid> {
    sqlx::query_scalar!("insert into teams (name) values ($1) returning id", name)
        .fetch_one(db)
        .await
}

/// False if there is no such team.
pub async fn rename(db: &PgPool, id: Uuid, name: &str) -> sqlx::Result<bool> {
    let result = sqlx::query!("update teams set name = $2 where id = $1", id, name)
        .execute(db)
        .await?;
    Ok(result.rows_affected() == 1)
}

/// Archives or restores a team. False if there is no such team.
pub async fn set_archived(db: &PgPool, id: Uuid, archived: bool) -> sqlx::Result<bool> {
    let result = sqlx::query!(
        "update teams set archived_at = case when $2 then coalesce(archived_at, now()) end
         where id = $1",
        id,
        archived,
    )
    .execute(db)
    .await?;
    Ok(result.rows_affected() == 1)
}
