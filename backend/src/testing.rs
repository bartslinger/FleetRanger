//! Fixtures shared by the database tests.

use sqlx::PgPool;
use uuid::Uuid;

use crate::policy::FlightScope;

/// Adds a team and its group with the same id, as migration 0018 made them.
pub async fn team(db: &PgPool, name: &str) -> Uuid {
    let id = sqlx::query_scalar("insert into teams (name) values ($1) returning id")
        .bind(name)
        .fetch_one(db)
        .await
        .unwrap();
    sqlx::query("insert into groups (id, name) values ($1, $2)")
        .bind(id)
        .bind(name)
        .execute(db)
        .await
        .unwrap();
    id
}

/// Adds a user to a team, and to its group with Pilot and Editor, as
/// migration 0018 did.
pub async fn join(db: &PgPool, user: Uuid, team: Uuid) {
    sqlx::query("insert into team_members (user_id, team_id) values ($1, $2)")
        .bind(user)
        .bind(team)
        .execute(db)
        .await
        .unwrap();
    sqlx::query(
        "insert into group_members (group_id, user_id, role)
         select $2, $1, unnest(array['pilot', 'editor'])",
    )
    .bind(user)
    .bind(team)
    .execute(db)
    .await
    .unwrap();
}

/// The flights a user sees, from their stored roles.
pub async fn scope(db: &PgPool, user: Uuid) -> FlightScope {
    FlightScope::new(user, &crate::auth::grants(db, user).await.unwrap())
}
