use serde::Serialize;
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

#[derive(Serialize)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    /// Members who aren't archived.
    pub members: i64,
    /// What it holds; only a group without any of these can be deleted.
    /// Archived flights count too.
    pub flights: i64,
    pub sessions: i64,
    pub drones: i64,
    pub controllers: i64,
}

/// A member who isn't archived, with their roles in the group.
#[derive(Serialize)]
pub struct Member {
    pub user_id: Uuid,
    pub name: String,
    pub email: Option<String>,
    /// Sorted: viewer, pilot, editor.
    pub roles: Vec<String>,
    /// Apps' roles, by id.
    pub app_roles: Vec<Uuid>,
}

/// Groups by name; `id` narrows it to one.
pub async fn list(db: &PgPool, id: Option<Uuid>) -> sqlx::Result<Vec<Group>> {
    sqlx::query_as!(
        Group,
        r#"select g.id, g.name,
                  (select count(*) from users u
                   where u.archived_at is null
                     and (exists (select 1 from group_members m
                                  where m.group_id = g.id and m.user_id = u.id)
                          or exists (select 1 from group_app_roles a
                                     where a.group_id = g.id and a.user_id = u.id)))
                      as "members!",
                  (select count(*) from flights where group_id = g.id) as "flights!",
                  (select count(*) from sessions where group_id = g.id) as "sessions!",
                  (select count(*) from aircraft where group_id = g.id) as "drones!",
                  (select count(*) from remote_controllers where group_id = g.id)
                      as "controllers!"
           from groups g
           where $1::uuid is null or g.id = $1
           order by lower(g.name), g.id"#,
        id,
    )
    .fetch_all(db)
    .await
}

pub async fn members(db: &PgPool, group_id: Uuid) -> sqlx::Result<Vec<Member>> {
    sqlx::query_as!(
        Member,
        r#"select u.id as user_id, u.name, l.email as "email?",
                  array(select m.role from group_members m
                        where m.group_id = $1 and m.user_id = u.id
                        order by array_position(array['viewer', 'pilot', 'editor'], m.role))
                      as "roles!",
                  array(select a.app_role_id from group_app_roles a
                        where a.group_id = $1 and a.user_id = u.id
                        order by a.app_role_id) as "app_roles!"
           from users u
           left join logins l on l.user_id = u.id
           where u.archived_at is null
             and (exists (select 1 from group_members m where m.group_id = $1 and m.user_id = u.id)
                  or exists (select 1 from group_app_roles a
                             where a.group_id = $1 and a.user_id = u.id))
           order by lower(u.name), u.id"#,
        group_id,
    )
    .fetch_all(db)
    .await
}

pub async fn insert(db: &PgPool, name: &str) -> sqlx::Result<Uuid> {
    sqlx::query_scalar!("insert into groups (name) values ($1) returning id", name)
        .fetch_one(db)
        .await
}

/// False if there is no such group.
pub async fn rename(db: &PgPool, id: Uuid, name: &str) -> sqlx::Result<bool> {
    let result = sqlx::query!("update groups set name = $2 where id = $1", id, name)
        .execute(db)
        .await?;
    Ok(result.rows_affected() == 1)
}

/// Makes these a user's roles in the group, FleetRanger's and apps': none
/// removes them from it. False if the group or user doesn't exist, or the
/// user is archived.
pub async fn set_roles(
    db: &PgPool,
    group_id: Uuid,
    user_id: Uuid,
    roles: &[String],
    app_roles: &[Uuid],
) -> sqlx::Result<bool> {
    let mut tx = db.begin().await?;
    let found = sqlx::query_scalar!(
        r#"select exists(select 1 from groups where id = $1)
                  and exists(select 1 from users where id = $2 and archived_at is null)
           as "found!""#,
        group_id,
        user_id,
    )
    .fetch_one(&mut *tx)
    .await?;
    if !found {
        return Ok(false);
    }
    sqlx::query!(
        "delete from group_members where group_id = $1 and user_id = $2 and role <> all($3)",
        group_id,
        user_id,
        roles,
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "insert into group_members (group_id, user_id, role)
         select $1, $2, unnest($3::text[])
         on conflict do nothing",
        group_id,
        user_id,
        roles,
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "delete from group_app_roles
         where group_id = $1 and user_id = $2 and app_role_id <> all($3)",
        group_id,
        user_id,
        app_roles,
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "insert into group_app_roles (group_id, user_id, app_role_id)
         select $1, $2, unnest($3::uuid[])
         on conflict do nothing",
        group_id,
        user_id,
        app_roles,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(true)
}

/// What a deletion did.
pub enum Deleted {
    Done,
    NotFound,
    /// The group still holds something, and nothing was to be moved.
    NotEmpty,
}

/// Deletes a group with its memberships. With `move_to`, everything in it
/// moves first: flights, sessions, drones and controllers, the logs chosen
/// for it on a controller, and open pairing codes for it; `Some(None)` moves
/// them to the organisation (flights to the queue). Without it, only an empty
/// group is deleted; its logs lose their group and its codes go with it.
///
/// The group's row is locked first, so nothing can be put in it in between:
/// a foreign key check on it waits for the lock.
pub async fn delete(db: &PgPool, id: Uuid, move_to: Option<Option<Uuid>>) -> sqlx::Result<Deleted> {
    let mut tx = db.begin().await?;
    let found = sqlx::query_scalar!("select id from groups where id = $1 for update", id)
        .fetch_optional(&mut *tx)
        .await?;
    if found.is_none() {
        return Ok(Deleted::NotFound);
    }
    match move_to {
        Some(target) => move_contents(&mut tx, id, target).await?,
        None => {
            if !is_empty(&mut tx, id).await? {
                return Ok(Deleted::NotEmpty);
            }
        }
    }
    sqlx::query!("delete from groups where id = $1", id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Deleted::Done)
}

async fn is_empty(db: &mut PgConnection, id: Uuid) -> sqlx::Result<bool> {
    sqlx::query_scalar!(
        r#"select not exists(select 1 from flights where group_id = $1)
                  and not exists(select 1 from sessions where group_id = $1)
                  and not exists(select 1 from aircraft where group_id = $1)
                  and not exists(select 1 from remote_controllers where group_id = $1)
           as "empty!""#,
        id,
    )
    .fetch_one(db)
    .await
}

async fn move_contents(db: &mut PgConnection, from: Uuid, to: Option<Uuid>) -> sqlx::Result<()> {
    sqlx::query!(
        "with flights as (update flights set group_id = $2 where group_id = $1),
              sessions as (update sessions set group_id = $2 where group_id = $1),
              aircraft as (update aircraft set group_id = $2 where group_id = $1),
              controllers as (update remote_controllers set group_id = $2 where group_id = $1),
              logs as (update raw_logs set group_id = $2 where group_id = $1)
         update pairing_codes set group_id = $2 where group_id = $1",
        from,
        to,
    )
    .execute(db)
    .await?;
    Ok(())
}

pub async fn exists(db: &PgPool, id: Uuid) -> sqlx::Result<bool> {
    sqlx::query_scalar!(
        r#"select exists(select 1 from groups where id = $1) as "exists!""#,
        id,
    )
    .fetch_one(db)
    .await
}
