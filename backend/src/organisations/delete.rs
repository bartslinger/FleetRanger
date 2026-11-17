//! Deleting an organisation with everything in it, at once (user's decisions,
//! 2026-11-13): its data in the database, with its logins, and its raw logs
//! on disk or in S3. The name confirms it, as names aren't unique.
//!
//! The database goes first, in one statement, so the controllers' tokens and
//! the logins stop working right away; the files follow (`files`). Should
//! deleting the files fail, deleting the organisation again, which is gone by
//! then, deletes what is left of them.

use anyhow::{Context, Result, bail};
use uuid::Uuid;

use crate::db::System;
use crate::ingest::store::LogStore;

/// Deletes the organisation `id` from the database, if `name` is its name,
/// with all its data. Returns its name and how many flights went, or None if
/// it was gone already.
pub async fn rows(system: &System, id: Uuid, name: &str) -> Result<Option<(String, i64)>> {
    let mut tx = system.begin().await?;
    let found = sqlx::query_scalar!(
        "select name from organisations where id = $1 for update",
        id
    )
    .fetch_optional(&mut *tx)
    .await?;
    let Some(actual) = found else {
        return Ok(None);
    };
    if actual != name {
        bail!("that isn't the organisation's name, which is {actual:?}");
    }
    let flights = sqlx::query_file_scalar!("src/organisations/delete.sql", id)
        .fetch_one(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Some((actual, flights)))
}

/// Deletes the raw logs of an organisation whose rows are gone. Returns how
/// many files there were.
pub async fn files(store: &LogStore, id: Uuid) -> Result<u64> {
    store.delete_organisation(id).await.with_context(|| {
        format!(
            "failed to delete the raw logs of organisation {id}; \
             `fleetranger delete-organisation {id} --name -` deletes what is left"
        )
    })
}

#[cfg(test)]
mod tests {
    /// Every table with an organisation's data is emptied of it: a table
    /// added later and left out here would keep data that should go, or
    /// make the delete fail on its foreign key.
    #[sqlx::test]
    async fn deleting_covers_every_table(pool: sqlx::PgPool) {
        let tables: Vec<String> = sqlx::query_scalar(
            "select c.table_name::text from information_schema.columns c
             join information_schema.tables t using (table_schema, table_name)
             where c.table_schema = 'public' and c.column_name = 'organisation_id'
               and t.table_type = 'BASE TABLE'",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        let sql = include_str!("delete.sql");
        let missing: Vec<_> = tables
            .iter()
            .filter(|table| {
                !sql.contains(&format!("delete from {table} where organisation_id = $1"))
            })
            .collect();
        assert!(missing.is_empty(), "not deleted: {missing:?}");
    }

    /// An invitation's sender and its login go in the one statement, as the
    /// login's link to the sender is set to null on the way.
    #[sqlx::test]
    async fn deleting_takes_invitations(pool: sqlx::PgPool) {
        let mut db = crate::testing::db(&pool).await;
        let id = crate::testing::second_organisation(&mut db, "Test Org B").await;
        sqlx::query(
            "with admin as (insert into users (organisation_id, name)
                            values ($1, 'Test Admin') returning id),
                  invitee as (insert into users (organisation_id, name)
                              values ($1, 'Test Invitee') returning id)
             insert into logins (organisation_id, user_id, email, oidc_subject, invited_by)
             select $1, admin.id, 'test-admin@example.org', 'test-admin', null from admin
             union all
             select $1, invitee.id, 'test-invitee@example.org', null, admin.id
             from invitee, admin",
        )
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();
        let deleted = super::rows(&crate::db::System(pool.clone()), id, "Test Org B")
            .await
            .unwrap();
        assert!(deleted.is_some());
    }
}
