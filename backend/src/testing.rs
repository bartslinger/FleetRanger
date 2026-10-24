//! Fixtures shared by the database tests.

use sqlx::PgPool;
use uuid::Uuid;

use crate::{auth::Actor, policy::FlightScope};

/// Adds a group.
pub async fn group(db: &PgPool, name: &str) -> Uuid {
    sqlx::query_scalar("insert into groups (name) values ($1) returning id")
        .bind(name)
        .fetch_one(db)
        .await
        .unwrap()
}

/// Adds a user to a group with Pilot and Editor, as migration 0018 did for
/// team members.
pub async fn join(db: &PgPool, user: Uuid, group: Uuid) {
    sqlx::query(
        "insert into group_members (group_id, user_id, role)
         select $2, $1, unnest(array['pilot', 'editor'])",
    )
    .bind(user)
    .bind(group)
    .execute(db)
    .await
    .unwrap();
}

/// A user acting with their stored roles.
pub async fn actor(db: &PgPool, user_id: Uuid) -> Actor {
    Actor {
        user_id,
        grants: crate::auth::grants(db, user_id).await.unwrap(),
        login_session_id: Uuid::nil(),
        ip: [127, 0, 0, 1].into(),
        email: None,
        impersonated_by: None,
        name: String::new(),
    }
}

/// A user with these roles on the whole organisation.
pub async fn user(db: &PgPool, name: &str, roles: &[&str]) -> Uuid {
    let id = sqlx::query_scalar("insert into users (name) values ($1) returning id")
        .bind(name)
        .fetch_one(db)
        .await
        .unwrap();
    sqlx::query("insert into organisation_roles (user_id, role) select $1, unnest($2::text[])")
        .bind(id)
        .bind(roles)
        .execute(db)
        .await
        .unwrap();
    id
}

/// The flights a user sees, from their stored roles.
pub async fn scope(db: &PgPool, user: Uuid) -> FlightScope {
    FlightScope::new(user, &crate::auth::grants(db, user).await.unwrap())
}
