use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::flights::db::GroupChoice;
use crate::policy::{self, Permission, Scope};

/// Adds a pairing code, and deletes expired ones. Returns its id, or None if
/// the code is already taken, so the caller can try another.
pub async fn add_code(
    db: &PgPool,
    code: &str,
    created_by: Uuid,
    group_id: Option<Uuid>,
    expires_at: DateTime<Utc>,
) -> sqlx::Result<Option<Uuid>> {
    let mut tx = db.begin().await?;
    sqlx::query!("delete from pairing_codes where expires_at < now()")
        .execute(&mut *tx)
        .await?;
    let id = sqlx::query_scalar!(
        "insert into pairing_codes (code, created_by, group_id, expires_at)
         values ($1, $2, $3, $4)
         on conflict (code) do nothing
         returning id",
        code,
        created_by,
        group_id,
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
    /// The group the controller joins; None for the organisation.
    pub group: Option<String>,
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
        r#"select c.id, c.code, u.name as created_by, g.name as "group?", c.created_at, c.expires_at
           from pairing_codes c
           join users u on u.id = c.created_by
           left join groups g on g.id = c.group_id
           where c.expires_at > now()
             and ($2 or c.created_by = $1 or c.group_id = any($3))
           order by c.expires_at, c.id"#,
        created_by,
        scope.all,
        &scope.groups,
    )
    .fetch_all(db)
    .await
}

/// A code's creator and the group the controller joins.
pub struct CodeOwner {
    pub created_by: Uuid,
    pub group_id: Option<Uuid>,
}

/// None if there is no such code.
pub async fn code_owner(db: &PgPool, id: Uuid) -> sqlx::Result<Option<CodeOwner>> {
    sqlx::query_as!(
        CodeOwner,
        "select created_by, group_id from pairing_codes where id = $1",
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
pub struct Paired {
    /// The group it joined; None for the organisation.
    pub group: Option<GroupChoice>,
}

/// Uses a code: it's deleted, and the controller with this serial (registered
/// now if it's new) gets the token and the code's group. Returns None if the
/// code doesn't exist, has expired, or its creator has been archived since.
/// A deleted group takes its codes with it.
pub async fn use_code(
    db: &PgPool,
    code: &str,
    serial: &str,
    token_sha256: &[u8],
) -> sqlx::Result<Option<Paired>> {
    let mut tx = db.begin().await?;
    let Some(group_id) = sqlx::query_scalar!(
        "delete from pairing_codes c using users u
         where c.code = $1 and c.expires_at > now()
           and u.id = c.created_by and u.archived_at is null
         returning c.group_id",
        code,
    )
    .fetch_optional(&mut *tx)
    .await?
    else {
        return Ok(None);
    };
    let group = sqlx::query_as!(
        GroupChoice,
        "select id, name from groups where id = $1",
        group_id
    )
    .fetch_optional(&mut *tx)
    .await?;
    sqlx::query!(
        "insert into remote_controllers (serial, group_id, token_sha256)
         values ($1, $2, $3)
         on conflict (serial) do update set
             group_id = excluded.group_id,
             token_sha256 = excluded.token_sha256",
        serial,
        group_id,
        token_sha256,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Some(Paired { group }))
}

/// A paired controller, as it authenticates.
pub struct Authenticated {
    pub id: Uuid,
    pub serial: String,
    pub group_id: Option<Uuid>,
}

pub async fn by_token(db: &PgPool, token_sha256: &[u8]) -> sqlx::Result<Option<Authenticated>> {
    sqlx::query_as!(
        Authenticated,
        "select id, serial, group_id from remote_controllers where token_sha256 = $1",
        token_sha256,
    )
    .fetch_optional(db)
    .await
}

/// A controller's status: its app, and when it last looked for logs on an
/// allowed connection (None for never).
pub async fn status(
    db: &PgPool,
    id: Uuid,
    app_version: &str,
    last_sync: Option<DateTime<Utc>>,
) -> sqlx::Result<()> {
    sqlx::query!(
        "update remote_controllers set synced_at = $3, app_version = $2 where id = $1",
        id,
        app_version,
        last_sync,
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

/// A pilot as a controller offers them, from v1 of its API: once, with the
/// groups they're a member of and fly for, and the other groups where they
/// hold `flights.fly_for` (every group, with it on the organisation).
#[derive(Serialize)]
pub struct Pilot {
    pub id: Uuid,
    pub name: String,
    pub groups: Vec<Uuid>,
    pub other_groups: Vec<Uuid>,
}

/// Active users who hold `flights.fly_for` anywhere, by name; their groups by
/// name too.
pub async fn pilots(db: &PgPool) -> sqlx::Result<Vec<Pilot>> {
    sqlx::query_as!(
        Pilot,
        r#"select u.id, u.name,
                  coalesce(array_agg(g.id order by lower(g.name), g.id) filter (where g.member),
                           '{}') as "groups!",
                  coalesce(array_agg(g.id order by lower(g.name), g.id) filter (where not g.member),
                           '{}') as "other_groups!"
           from users u
           left join lateral (
               select g.id, g.name,
                      exists (select 1 from group_members m
                              where m.user_id = u.id and m.group_id = g.id) as member
               from groups g
               where exists (select 1 from group_members m
                             where m.user_id = u.id and m.group_id = g.id and m.role = any($1))
                  or exists (select 1 from organisation_roles r
                             where r.user_id = u.id and r.role = any($1))
           ) g on true
           where u.archived_at is null
             and (exists (select 1 from group_members m
                          where m.user_id = u.id and m.role = any($1))
                  or exists (select 1 from organisation_roles r
                             where r.user_id = u.id and r.role = any($1)))
           group by u.id, u.name
           order by lower(u.name), u.id"#,
        &policy::roles_granting(Permission::FlightsFlyFor),
    )
    .fetch_all(db)
    .await
}

/// A pilot as android-v0.3.0 offers them: once per group they are a pilot in,
/// which it still calls a team. Goes once no controller reports that version.
#[derive(Serialize)]
pub struct PilotChoice {
    pub id: Uuid,
    pub name: String,
    pub team_id: Uuid,
    pub team_name: String,
}

/// Active users by name, once per group they fly for.
pub async fn pilot_choices(db: &PgPool) -> sqlx::Result<Vec<PilotChoice>> {
    sqlx::query_as!(
        PilotChoice,
        "select u.id, u.name, g.id as team_id, g.name as team_name
         from users u
         join groups g on exists (select 1 from group_members m
                                  where m.user_id = u.id and m.group_id = g.id
                                    and m.role = any($1))
         where u.archived_at is null
         order by lower(u.name), u.id, lower(g.name)",
        &policy::roles_granting(Permission::FlightsFlyFor),
    )
    .fetch_all(db)
    .await
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
        let team = crate::testing::group(&db, "North").await;
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
        assert_eq!(
            paired.and_then(|p| p.group).map(|g| g.name).as_deref(),
            Some("North")
        );
        assert!(
            use_code(&db, "AAAA2222", "RC2", b"other")
                .await
                .unwrap()
                .is_none()
        );
        let controller = by_token(&db, b"token-hash").await.unwrap().unwrap();
        assert_eq!(controller.serial, "RC1");
        assert_eq!(controller.group_id, Some(team));

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

    /// The controller's pilot list holds only people who fly, each once: the
    /// groups they're a member of and fly for first, then the others where
    /// they may fly, which with Pilot on the organisation is every group.
    #[sqlx::test]
    async fn the_pilot_list_holds_who_flies_with_their_groups(db: PgPool) {
        use crate::testing::{group, join, user};
        let north = group(&db, "North").await;
        let south = group(&db, "South").await;
        let member = user(&db, "Test member pilot", &[]).await;
        join(&db, member, north).await;
        // Pilot on the organisation, and a viewer in South.
        let everywhere = user(&db, "Test organisation pilot", &["pilot"]).await;
        sqlx::query(
            "insert into group_members (group_id, user_id, role) values ($1, $2, 'viewer')",
        )
        .bind(south)
        .bind(everywhere)
        .execute(&db)
        .await
        .unwrap();
        user(&db, "Test admin", &["admin"]).await;
        let viewer = user(&db, "Test viewer", &[]).await;
        sqlx::query(
            "insert into group_members (group_id, user_id, role) values ($1, $2, 'viewer')",
        )
        .bind(north)
        .bind(viewer)
        .execute(&db)
        .await
        .unwrap();

        let pilots = pilots(&db).await.unwrap();
        let found: Vec<_> = pilots
            .iter()
            .map(|p| (p.id, p.groups.clone(), p.other_groups.clone()))
            .collect();
        assert_eq!(
            found,
            vec![
                (member, vec![north], vec![]),
                (everywhere, vec![south], vec![north]),
            ]
        );
    }
}
