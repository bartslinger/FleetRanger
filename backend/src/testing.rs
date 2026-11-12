//! Fixtures shared by the database tests.

use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::{
    auth::Actor,
    db::{Db, Tenants},
    policy::FlightScope,
};

/// A test's connection, in the organisation migration 0026 adds to every
/// install. Tests connect as the account that created the tables, which
/// row-level security doesn't limit; the separation tests use
/// `fleetranger_app`.
pub async fn db(pool: &PgPool) -> Db {
    let organisation: Uuid =
        sqlx::query_scalar("select id from organisations order by created_at limit 1")
            .fetch_one(pool)
            .await
            .unwrap();
    Tenants::of(pool.clone())
        .connect(organisation)
        .await
        .unwrap()
}

/// Adds a group.
pub async fn group(db: &mut PgConnection, name: &str) -> Uuid {
    sqlx::query_scalar("insert into groups (name) values ($1) returning id")
        .bind(name)
        .fetch_one(&mut *db)
        .await
        .unwrap()
}

/// Adds a user to a group with Pilot and Editor, as migration 0018 did for
/// team members.
pub async fn join(db: &mut PgConnection, user: Uuid, group: Uuid) {
    sqlx::query(
        "insert into group_members (group_id, user_id, role)
         select $2, $1, unnest(array['pilot', 'editor'])",
    )
    .bind(user)
    .bind(group)
    .execute(&mut *db)
    .await
    .unwrap();
}

/// A user acting with their stored roles.
pub async fn actor(db: &mut PgConnection, user_id: Uuid) -> Actor {
    Actor {
        user_id,
        organisation_id: organisation(db).await,
        grants: crate::auth::grants(db, user_id).await.unwrap(),
        login_session_id: Uuid::nil(),
        ip: [127, 0, 0, 1].into(),
        email: None,
        impersonated_by: None,
        name: String::new(),
    }
}

/// A user with these roles on the whole organisation: Admin, or none.
pub async fn user(db: &mut PgConnection, name: &str, roles: &[&str]) -> Uuid {
    let id = sqlx::query_scalar("insert into users (name) values ($1) returning id")
        .bind(name)
        .fetch_one(&mut *db)
        .await
        .unwrap();
    sqlx::query("insert into organisation_roles (user_id, role) select $1, unnest($2::text[])")
        .bind(id)
        .bind(roles)
        .execute(&mut *db)
        .await
        .unwrap();
    id
}

/// The flights a user sees, from their stored roles.
pub async fn scope(db: &mut PgConnection, user: Uuid) -> FlightScope {
    FlightScope::new(user, &crate::auth::grants(db, user).await.unwrap())
}

/// The group migration 0025 adds to every install, where items go when a
/// test doesn't care about their group.
pub async fn general(db: &mut PgConnection) -> Uuid {
    sqlx::query_scalar("select id from groups where name = 'General'")
        .fetch_one(&mut *db)
        .await
        .unwrap()
}

/// A user with these roles in a group, such as Viewer, Pilot or Editor.
pub async fn member(db: &mut PgConnection, name: &str, group: Uuid, roles: &[&str]) -> Uuid {
    let id = user(db, name, &[]).await;
    sqlx::query(
        "insert into group_members (group_id, user_id, role)
         select $2, $1, unnest($3::text[])",
    )
    .bind(id)
    .bind(group)
    .bind(roles)
    .execute(&mut *db)
    .await
    .unwrap();
    id
}

/// The organisation a test's connection is in.
pub async fn organisation(db: &mut PgConnection) -> Uuid {
    sqlx::query_scalar("select current_setting('app.organisation_id')::uuid")
        .fetch_one(db)
        .await
        .unwrap()
}

/// A second organisation, made as `add-organisation` makes one: its
/// settings and its group "General".
pub async fn second_organisation(db: &mut PgConnection, name: &str) -> Uuid {
    let id: Uuid = sqlx::query_scalar("insert into organisations (name) values ($1) returning id")
        .bind(name)
        .fetch_one(&mut *db)
        .await
        .unwrap();
    for sql in [
        "insert into organisation_settings (organisation_id) values ($1)",
        "insert into groups (organisation_id, name) values ($1, 'General')",
    ] {
        sqlx::query(sql).bind(id).execute(&mut *db).await.unwrap();
    }
    id
}

/// A connection that acts as `fleetranger_app`, as requests do, so
/// row-level security applies, in this organisation or, with None, in none.
/// (Tests can't log in as it; the account that created the tables may
/// take its role.)
pub async fn restricted(
    pool: &PgPool,
    organisation: Option<Uuid>,
) -> sqlx::pool::PoolConnection<sqlx::Postgres> {
    let mut connection = pool.acquire().await.unwrap();
    sqlx::query("set role fleetranger_app")
        .execute(&mut *connection)
        .await
        .unwrap();
    crate::db::set_organisation(&mut connection, organisation)
        .await
        .unwrap();
    connection
}
