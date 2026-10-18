use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Serialize)]
pub struct Account {
    pub id: Uuid,
    pub email: String,
    /// From the login provider, after the first login.
    pub name: Option<String>,
    pub role: String,
    pub disabled_at: Option<DateTime<Utc>>,
    pub pilot_id: Option<Uuid>,
    pub pilot_name: Option<String>,
    pub last_login_at: Option<DateTime<Utc>>,
}

/// Adds an account for a lowercase email address. Returns its id, or None if
/// one exists.
pub async fn insert(db: &PgPool, email: &str, role: &str) -> sqlx::Result<Option<Uuid>> {
    sqlx::query_scalar!(
        "insert into accounts (email, role) values ($1, $2)
         on conflict (email) do nothing
         returning id",
        email,
        role,
    )
    .fetch_optional(db)
    .await
}

pub async fn set_role_by_email(db: &PgPool, email: &str, role: &str) -> sqlx::Result<()> {
    sqlx::query!(
        "update accounts set role = $2 where email = $1",
        email,
        role
    )
    .execute(db)
    .await?;
    Ok(())
}

/// Accounts by email.
pub async fn list(db: &PgPool) -> sqlx::Result<Vec<Account>> {
    sqlx::query_as!(
        Account,
        r#"select a.id, a.email, a.name, a.role, a.disabled_at,
                  p.id as "pilot_id?", p.name as "pilot_name?",
                  (select max(created_at) from login_sessions s
                   where s.account_id = a.id) as last_login_at
           from accounts a
           left join pilots p on p.account_id = a.id
           order by a.email"#,
    )
    .fetch_all(db)
    .await
}

pub async fn get(db: &PgPool, id: Uuid) -> sqlx::Result<Option<Account>> {
    sqlx::query_as!(
        Account,
        r#"select a.id, a.email, a.name, a.role, a.disabled_at,
                  p.id as "pilot_id?", p.name as "pilot_name?",
                  (select max(created_at) from login_sessions s
                   where s.account_id = a.id) as last_login_at
           from accounts a
           left join pilots p on p.account_id = a.id
           where a.id = $1"#,
        id,
    )
    .fetch_optional(db)
    .await
}

/// False if there is no such account.
pub async fn set_role(db: &PgPool, id: Uuid, role: &str) -> sqlx::Result<bool> {
    let result = sqlx::query!("update accounts set role = $2 where id = $1", id, role)
        .execute(db)
        .await?;
    Ok(result.rows_affected() == 1)
}

/// Disables an account and revokes its login sessions, or enables it again.
/// False if there is no such account.
pub async fn set_disabled(db: &PgPool, id: Uuid, disabled: bool) -> sqlx::Result<bool> {
    let mut tx = db.begin().await?;
    let updated = sqlx::query!(
        "update accounts set disabled_at = case when $2 then coalesce(disabled_at, now()) end
         where id = $1",
        id,
        disabled,
    )
    .execute(&mut *tx)
    .await?
    .rows_affected()
        == 1;
    if updated && disabled {
        sqlx::query!(
            "update login_sessions set revoked_at = now()
             where account_id = $1 and revoked_at is null",
            id,
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(updated)
}
