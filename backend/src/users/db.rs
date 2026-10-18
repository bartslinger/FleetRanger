use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

#[derive(Serialize)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub role: String,
    /// The team the user flies for; none for users who don't fly.
    pub team_id: Option<Uuid>,
    pub team_name: Option<String>,
    /// The login's email address; none if the user can't log in.
    pub email: Option<String>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,
    /// Flights with this user as pilot; only a user without flights can be deleted.
    pub flights: i64,
}

/// What an admin sets on a user.
pub struct UserFields<'a> {
    pub name: &'a str,
    pub role: &'a str,
    pub team_id: Option<Uuid>,
    /// Lowercase; None removes the login.
    pub email: Option<&'a str>,
}

/// Users by name, the archived ones only if asked for.
pub async fn list(db: &PgPool, archived: bool) -> sqlx::Result<Vec<User>> {
    sqlx::query_as!(
        User,
        r#"select u.id, u.name, u.role, m.team_id as "team_id?", t.name as "team_name?",
                  l.email as "email?",
                  (select max(created_at) from login_sessions s where s.user_id = u.id)
                      as last_login_at,
                  u.archived_at,
                  (select count(*) from flights f where f.pilot_id = u.id) as "flights!"
           from users u
           left join team_members m on m.user_id = u.id
           left join teams t on t.id = m.team_id
           left join logins l on l.user_id = u.id
           where $1 or u.archived_at is null
           order by lower(u.name), u.id"#,
        archived,
    )
    .fetch_all(db)
    .await
}

pub async fn get(db: &PgPool, id: Uuid) -> sqlx::Result<Option<User>> {
    sqlx::query_as!(
        User,
        r#"select u.id, u.name, u.role, m.team_id as "team_id?", t.name as "team_name?",
                  l.email as "email?",
                  (select max(created_at) from login_sessions s where s.user_id = u.id)
                      as last_login_at,
                  u.archived_at,
                  (select count(*) from flights f where f.pilot_id = u.id) as "flights!"
           from users u
           left join team_members m on m.user_id = u.id
           left join teams t on t.id = m.team_id
           left join logins l on l.user_id = u.id
           where u.id = $1"#,
        id,
    )
    .fetch_optional(db)
    .await
}

pub async fn insert(db: &PgPool, fields: &UserFields<'_>) -> sqlx::Result<Uuid> {
    let mut tx = db.begin().await?;
    let id = sqlx::query_scalar!(
        "insert into users (name, role) values ($1, $2) returning id",
        fields.name,
        fields.role,
    )
    .fetch_one(&mut *tx)
    .await?;
    set_team(&mut tx, id, fields.team_id).await?;
    set_login(&mut tx, id, fields.email).await?;
    tx.commit().await?;
    Ok(id)
}

/// False if there is no such user.
pub async fn update(db: &PgPool, id: Uuid, fields: &UserFields<'_>) -> sqlx::Result<bool> {
    let mut tx = db.begin().await?;
    let updated = sqlx::query!(
        "update users set name = $2, role = $3 where id = $1",
        id,
        fields.name,
        fields.role,
    )
    .execute(&mut *tx)
    .await?
    .rows_affected()
        == 1;
    if updated {
        // Past flights keep their team; only new flights follow the user.
        set_team(&mut tx, id, fields.team_id).await?;
        set_login(&mut tx, id, fields.email).await?;
    }
    tx.commit().await?;
    Ok(updated)
}

async fn set_team(db: &mut PgConnection, id: Uuid, team_id: Option<Uuid>) -> sqlx::Result<()> {
    match team_id {
        Some(team_id) => {
            sqlx::query!(
                "insert into team_members (user_id, team_id) values ($1, $2)
                 on conflict (user_id) do update set team_id = excluded.team_id",
                id,
                team_id,
            )
            .execute(db)
            .await?;
        }
        None => {
            sqlx::query!("delete from team_members where user_id = $1", id)
                .execute(db)
                .await?;
        }
    }
    Ok(())
}

/// Gives the user a login with this email, changes it, or removes it. A new or
/// changed address is tied to its login provider again on first use, and ends
/// the user's login sessions, as does removing the login.
async fn set_login(db: &mut PgConnection, id: Uuid, email: Option<&str>) -> sqlx::Result<()> {
    let current = sqlx::query_scalar!("select email from logins where user_id = $1", id)
        .fetch_optional(&mut *db)
        .await?;
    if current.as_deref() == email {
        return Ok(());
    }
    match email {
        Some(email) => {
            sqlx::query!(
                "insert into logins (user_id, email) values ($1, $2)
                 on conflict (user_id) do update set email = excluded.email, oidc_subject = null",
                id,
                email,
            )
            .execute(&mut *db)
            .await?;
        }
        None => {
            sqlx::query!("delete from logins where user_id = $1", id)
                .execute(&mut *db)
                .await?;
        }
    }
    revoke_sessions(db, id).await
}

async fn revoke_sessions(db: &mut PgConnection, id: Uuid) -> sqlx::Result<()> {
    sqlx::query!(
        "update login_sessions set revoked_at = now()
         where user_id = $1 and revoked_at is null",
        id,
    )
    .execute(db)
    .await?;
    Ok(())
}

/// Archives a user, which also ends their login sessions, or restores them.
/// False if there is no such user.
pub async fn set_archived(db: &PgPool, id: Uuid, archived: bool) -> sqlx::Result<bool> {
    let mut tx = db.begin().await?;
    let updated = sqlx::query!(
        "update users set archived_at = case when $2 then coalesce(archived_at, now()) end
         where id = $1",
        id,
        archived,
    )
    .execute(&mut *tx)
    .await?
    .rows_affected()
        == 1;
    if updated && archived {
        revoke_sessions(&mut tx, id).await?;
    }
    tx.commit().await?;
    Ok(updated)
}

/// Deletes a user without flights, with their team membership, login and login
/// sessions, and clears them as holder of equipment. False if there is no such
/// user or they have flights. The user's row is locked first, so a flight can't
/// get this user in between: its foreign key check waits for the lock.
pub async fn delete(db: &PgPool, id: Uuid) -> sqlx::Result<bool> {
    let mut tx = db.begin().await?;
    let deletable = sqlx::query_scalar!(
        r#"select not exists(select 1 from flights where pilot_id = $1) as "deletable!"
           from users where id = $1 for update"#,
        id,
    )
    .fetch_optional(&mut *tx)
    .await?;
    if deletable != Some(true) {
        return Ok(false);
    }
    sqlx::query!("delete from team_members where user_id = $1", id)
        .execute(&mut *tx)
        .await?;
    sqlx::query!("delete from login_sessions where user_id = $1", id)
        .execute(&mut *tx)
        .await?;
    sqlx::query!("delete from logins where user_id = $1", id)
        .execute(&mut *tx)
        .await?;
    sqlx::query!(
        "update aircraft set holder_user_id = null where holder_user_id = $1",
        id
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "update remote_controllers set holder_user_id = null where holder_user_id = $1",
        id
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!("delete from users where id = $1", id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(true)
}

/// For the `add-user` command: the user with a login for this email, if any.
pub async fn find_by_email(db: &PgPool, email: &str) -> sqlx::Result<Option<Uuid>> {
    sqlx::query_scalar!("select user_id from logins where email = $1", email)
        .fetch_optional(db)
        .await
}

pub async fn make_admin(db: &PgPool, id: Uuid) -> sqlx::Result<()> {
    sqlx::query!("update users set role = 'admin' where id = $1", id)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn team_is_active(db: &PgPool, team_id: Uuid) -> sqlx::Result<bool> {
    sqlx::query_scalar!(
        r#"select exists(select 1 from teams where id = $1 and archived_at is null) as "exists!""#,
        team_id,
    )
    .fetch_one(db)
    .await
}
