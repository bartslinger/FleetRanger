use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{PgConnection, PgPool, types::Json};
use uuid::Uuid;

#[derive(Serialize)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    /// Their roles on the whole organisation, in the order viewer, pilot,
    /// editor, admin.
    pub roles: Vec<String>,
    /// The groups they are a member of, by name, with their roles there.
    pub groups: Json<Vec<Membership>>,
    /// The login's email address; none if the user can't log in.
    pub email: Option<String>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,
    /// Flights with this user as pilot; only a user without flights can be deleted.
    pub flights: i64,
}

#[derive(Serialize, serde::Deserialize)]
pub struct Membership {
    pub id: Uuid,
    pub name: String,
    pub roles: Vec<String>,
}

/// What an admin sets on a user.
pub struct UserFields<'a> {
    pub name: &'a str,
    /// Roles on the whole organisation, as stored.
    pub roles: &'a [String],
    /// Lowercase; None removes the login.
    pub email: Option<&'a str>,
}

/// Why a change was refused.
pub enum Refused {
    NotFound,
    /// It would leave the organisation without an active admin.
    LastAdmin,
}

/// Users by name, the archived ones only if asked for; `id` narrows it to one.
pub async fn list(db: &PgPool, archived: bool, id: Option<Uuid>) -> sqlx::Result<Vec<User>> {
    sqlx::query_as!(
        User,
        r#"select u.id, u.name,
                  array(select r.role from organisation_roles r where r.user_id = u.id
                        order by array_position(array['viewer', 'pilot', 'editor', 'admin'],
                                                r.role)) as "roles!",
                  coalesce((select jsonb_agg(jsonb_build_object('id', g.id, 'name', g.name,
                                                                'roles', m.roles)
                                             order by lower(g.name), g.id)
                            from groups g
                            join (select group_id,
                                         array_agg(role order by array_position(
                                             array['viewer', 'pilot', 'editor'], role)) as roles
                                  from group_members where user_id = u.id
                                  group by group_id) m on m.group_id = g.id),
                           '[]') as "groups!: Json<Vec<Membership>>",
                  l.email as "email?",
                  (select max(created_at) from login_sessions s where s.user_id = u.id)
                      as last_login_at,
                  u.archived_at,
                  (select count(*) from flights f where f.pilot_id = u.id and f.archived_at is null)
                      as "flights!"
           from users u
           left join logins l on l.user_id = u.id
           where ($1 or u.archived_at is null) and ($2::uuid is null or u.id = $2)
           order by lower(u.name), u.id"#,
        archived || id.is_some(),
        id,
    )
    .fetch_all(db)
    .await
}

pub async fn insert(db: &PgPool, fields: &UserFields<'_>) -> sqlx::Result<Uuid> {
    let mut tx = db.begin().await?;
    let id = sqlx::query_scalar!(
        "insert into users (name) values ($1) returning id",
        fields.name,
    )
    .fetch_one(&mut *tx)
    .await?;
    set_roles(&mut tx, id, fields.roles).await?;
    set_login(&mut tx, id, fields.email).await?;
    tx.commit().await?;
    Ok(id)
}

pub async fn update(
    db: &PgPool,
    id: Uuid,
    fields: &UserFields<'_>,
) -> sqlx::Result<Result<(), Refused>> {
    let mut tx = db.begin().await?;
    let admins = lock_active_admins(&mut tx).await?;
    let updated = sqlx::query!("update users set name = $2 where id = $1", id, fields.name)
        .execute(&mut *tx)
        .await?
        .rows_affected()
        == 1;
    if !updated {
        return Ok(Err(Refused::NotFound));
    }
    if admins == [id] && !fields.roles.iter().any(|role| role == "admin") {
        return Ok(Err(Refused::LastAdmin));
    }
    set_roles(&mut tx, id, fields.roles).await?;
    set_login(&mut tx, id, fields.email).await?;
    tx.commit().await?;
    Ok(Ok(()))
}

/// The active admins, locked until the transaction ends, so two changes
/// can't each take away an admin while the other one still counts.
async fn lock_active_admins(db: &mut PgConnection) -> sqlx::Result<Vec<Uuid>> {
    sqlx::query_scalar!(
        "select r.user_id from organisation_roles r
         join users u on u.id = r.user_id
         where r.role = 'admin' and u.archived_at is null
         order by r.user_id
         for update of r, u",
    )
    .fetch_all(db)
    .await
}

/// Makes these the user's roles on the whole organisation.
async fn set_roles(db: &mut PgConnection, id: Uuid, roles: &[String]) -> sqlx::Result<()> {
    sqlx::query!(
        "delete from organisation_roles where user_id = $1 and role <> all($2)",
        id,
        roles,
    )
    .execute(&mut *db)
    .await?;
    sqlx::query!(
        "insert into organisation_roles (user_id, role) select $1, unnest($2::text[])
         on conflict do nothing",
        id,
        roles,
    )
    .execute(db)
    .await?;
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

/// Ends the user's login sessions and disconnects their apps (OAuth grants,
/// and codes an app hasn't exchanged yet).
async fn revoke_sessions(db: &mut PgConnection, id: Uuid) -> sqlx::Result<()> {
    sqlx::query!(
        "update login_sessions set revoked_at = now()
         where user_id = $1 and revoked_at is null",
        id,
    )
    .execute(&mut *db)
    .await?;
    sqlx::query!("delete from oauth_grants where user_id = $1", id)
        .execute(&mut *db)
        .await?;
    sqlx::query!("delete from oauth_codes where user_id = $1", id)
        .execute(db)
        .await?;
    Ok(())
}

/// Archives a user, which also ends their login sessions, or restores them.
/// The last active admin isn't archived.
pub async fn set_archived(
    db: &PgPool,
    id: Uuid,
    archived: bool,
) -> sqlx::Result<Result<(), Refused>> {
    let mut tx = db.begin().await?;
    let admins = lock_active_admins(&mut tx).await?;
    if archived && admins == [id] {
        return Ok(Err(Refused::LastAdmin));
    }
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
    if !updated {
        return Ok(Err(Refused::NotFound));
    }
    if archived {
        revoke_sessions(&mut tx, id).await?;
    }
    tx.commit().await?;
    Ok(Ok(()))
}

/// Deletes a user without flights, with their roles, login and login
/// sessions. False if there is no such user or they have flights. The user's
/// row is locked first, so a flight can't get this user in between: its
/// foreign key check waits for the lock.
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
    // Group memberships and roles go with the user.
    sqlx::query!("delete from login_sessions where user_id = $1", id)
        .execute(&mut *tx)
        .await?;
    sqlx::query!("delete from logins where user_id = $1", id)
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

/// Admin on the whole organisation.
pub async fn make_admin(db: &PgPool, id: Uuid) -> sqlx::Result<()> {
    sqlx::query!(
        "insert into organisation_roles (user_id, role) values ($1, 'admin')
         on conflict do nothing",
        id,
    )
    .execute(db)
    .await?;
    Ok(())
}
