//! Fixtures shared by the database tests.

use sqlx::PgPool;
use uuid::Uuid;

use crate::policy::FlightScope;

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

/// The flights a user sees, from their stored roles.
pub async fn scope(db: &PgPool, user: Uuid) -> FlightScope {
    FlightScope::new(user, &crate::auth::grants(db, user).await.unwrap())
}
