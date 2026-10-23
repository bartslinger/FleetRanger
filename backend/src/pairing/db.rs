use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::policy::Scope;

/// Adds a pairing code, and deletes expired ones. Returns its id, or None if
/// the code is already taken, so the caller can try another.
pub async fn add_code(
    db: &PgPool,
    code: &str,
    created_by: Uuid,
    team_id: Option<Uuid>,
    expires_at: DateTime<Utc>,
) -> sqlx::Result<Option<Uuid>> {
    let mut tx = db.begin().await?;
    sqlx::query!("delete from pairing_codes where expires_at < now()")
        .execute(&mut *tx)
        .await?;
    let id = sqlx::query_scalar!(
        "insert into pairing_codes (code, created_by, team_id, expires_at)
         values ($1, $2, $3, $4)
         on conflict (code) do nothing
         returning id",
        code,
        created_by,
        team_id,
        expires_at,
    )
    .fetch_optional(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(id)
}

/// A code that can still be used.
#[derive(Serialize)]
pub struct OpenCode {
    pub id: Uuid,
    pub code: String,
    pub created_by: String,
    /// The team the controller joins; None for none.
    pub team: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// The codes that can still be used of this creator, and of the groups in
/// the scope; those expiring first come first.
pub async fn open_codes(
    db: &PgPool,
    created_by: Uuid,
    scope: &Scope,
) -> sqlx::Result<Vec<OpenCode>> {
    sqlx::query_as!(
        OpenCode,
        r#"select c.id, c.code, u.name as created_by, t.name as "team?", c.created_at, c.expires_at
           from pairing_codes c
           join users u on u.id = c.created_by
           left join teams t on t.id = c.team_id
           where c.expires_at > now()
             and ($2 or c.created_by = $1 or c.team_id = any($3))
           order by c.expires_at, c.id"#,
        created_by,
        scope.all,
        &scope.groups,
    )
    .fetch_all(db)
    .await
}

/// A code's creator and the group (team) the controller joins.
pub struct CodeOwner {
    pub created_by: Uuid,
    pub group_id: Option<Uuid>,
}

/// None if there is no such code.
pub async fn code_owner(db: &PgPool, id: Uuid) -> sqlx::Result<Option<CodeOwner>> {
    sqlx::query_as!(
        CodeOwner,
        "select created_by, team_id as group_id from pairing_codes where id = $1",
        id,
    )
    .fetch_optional(db)
    .await
}

/// The group a controller is in, `Some(None)` for none; None if there is no
/// such controller.
pub async fn controller_group(db: &PgPool, id: Uuid) -> sqlx::Result<Option<Option<Uuid>>> {
    sqlx::query_scalar!("select group_id from remote_controllers where id = $1", id)
        .fetch_optional(db)
        .await
}

/// Deletes a code. Returns false if there is no such code.
pub async fn revoke(db: &PgPool, id: Uuid) -> sqlx::Result<bool> {
    let result = sqlx::query!("delete from pairing_codes where id = $1", id)
        .execute(db)
        .await?;
    Ok(result.rows_affected() == 1)
}

/// A controller that just paired.
#[derive(Serialize)]
pub struct Paired {
    /// The team it joined; None for none.
    pub team: Option<String>,
}

/// Uses a code: it's deleted, and the controller with this serial (registered
/// now if it's new) gets the token and the code's team. Returns None if the
/// code doesn't exist, has expired, or its creator has been archived since.
pub async fn use_code(
    db: &PgPool,
    code: &str,
    serial: &str,
    token_sha256: &[u8],
) -> sqlx::Result<Option<Paired>> {
    let mut tx = db.begin().await?;
    let Some(team_id) = sqlx::query_scalar!(
        "delete from pairing_codes c using users u
         where c.code = $1 and c.expires_at > now()
           and u.id = c.created_by and u.archived_at is null
         returning c.team_id",
        code,
    )
    .fetch_optional(&mut *tx)
    .await?
    else {
        return Ok(None);
    };
    // A team archived since the code was made isn't joined.
    let team = sqlx::query!(
        "select id, name from teams where id = $1 and archived_at is null",
        team_id,
    )
    .fetch_optional(&mut *tx)
    .await?;
    sqlx::query!(
        "insert into remote_controllers (serial, team_id, group_id, token_sha256)
         values ($1, $2, $2, $3)
         on conflict (serial) do update set
             team_id = excluded.team_id,
             group_id = excluded.group_id,
             token_sha256 = excluded.token_sha256",
        serial,
        team.as_ref().map(|team| team.id),
        token_sha256,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Some(Paired {
        team: team.map(|team| team.name),
    }))
}

/// A paired controller, as it authenticates.
pub struct Authenticated {
    pub id: Uuid,
    pub serial: String,
    pub team_id: Option<Uuid>,
}

pub async fn by_token(db: &PgPool, token_sha256: &[u8]) -> sqlx::Result<Option<Authenticated>> {
    sqlx::query_as!(
        Authenticated,
        "select id, serial, team_id from remote_controllers where token_sha256 = $1",
        token_sha256,
    )
    .fetch_optional(db)
    .await
}

/// A sync: the controller looked for logs on an allowed connection, now.
pub async fn synced(db: &PgPool, id: Uuid, app_version: &str) -> sqlx::Result<()> {
    sqlx::query!(
        "update remote_controllers set synced_at = now(), app_version = $2 where id = $1",
        id,
        app_version,
    )
    .execute(db)
    .await?;
    Ok(())
}

/// Unpairs a controller: its token stops working. Returns false if there is no
/// such controller.
pub async fn unpair(db: &PgPool, id: Uuid) -> sqlx::Result<bool> {
    let result = sqlx::query!(
        "update remote_controllers set token_sha256 = null where id = $1",
        id
    )
    .execute(db)
    .await?;
    Ok(result.rows_affected() == 1)
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    async fn insert(db: &PgPool, sql: &'static str) -> Uuid {
        sqlx::query_scalar(sql).fetch_one(db).await.unwrap()
    }

    /// A code pairs one controller, with the token whose hash it sent, in the
    /// code's team; it can't be used twice, nor once expired, nor once its
    /// creator was archived.
    #[sqlx::test]
    async fn a_code_pairs_one_controller(db: PgPool) {
        let team = crate::testing::team(&db, "North").await;
        let user = insert(
            &db,
            "insert into users (name) values ('Test user') returning id",
        )
        .await;
        let soon = Utc::now() + Duration::hours(1);
        assert!(
            add_code(&db, "AAAA2222", user, Some(team), soon)
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            add_code(&db, "AAAA2222", user, None, soon)
                .await
                .unwrap()
                .is_none()
        );

        let paired = use_code(&db, "AAAA2222", "RC1", b"token-hash")
            .await
            .unwrap();
        assert_eq!(paired.and_then(|p| p.team).as_deref(), Some("North"));
        assert!(
            use_code(&db, "AAAA2222", "RC2", b"other")
                .await
                .unwrap()
                .is_none()
        );
        let controller = by_token(&db, b"token-hash").await.unwrap().unwrap();
        assert_eq!(controller.serial, "RC1");
        assert_eq!(controller.team_id, Some(team));

        // Expired: a code created long ago, as add_code refuses none.
        sqlx::query(
            "insert into pairing_codes (code, created_by, created_at, expires_at)
             values ('BBBB3333', $1, now() - interval '2 hours', now() - interval '1 hour')",
        )
        .bind(user)
        .execute(&db)
        .await
        .unwrap();
        assert!(
            use_code(&db, "BBBB3333", "RC3", b"late")
                .await
                .unwrap()
                .is_none()
        );

        add_code(&db, "CCCC4444", user, None, soon).await.unwrap();
        sqlx::query("update users set archived_at = now() where id = $1")
            .bind(user)
            .execute(&db)
            .await
            .unwrap();
        assert!(
            use_code(&db, "CCCC4444", "RC4", b"gone")
                .await
                .unwrap()
                .is_none()
        );

        assert!(unpair(&db, controller.id).await.unwrap());
        assert!(by_token(&db, b"token-hash").await.unwrap().is_none());
    }
}
