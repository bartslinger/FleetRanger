use std::net::IpAddr;

use sqlx::PgPool;
use uuid::Uuid;

/// The user a login is for, if they may log in.
pub enum LoginUser {
    Allowed(Uuid),
    /// No login, an archived user, or the email belongs to another login.
    Refused,
}

/// Finds the user for a login by its subject ID. A first login is matched on
/// its verified email instead, which ties the login to the subject from then on.
pub async fn login_user(
    db: &PgPool,
    subject: &str,
    verified_email: Option<&str>,
) -> sqlx::Result<LoginUser> {
    let mut found = sqlx::query!(
        r#"select l.user_id, u.archived_at is not null as "archived!"
           from logins l join users u on u.id = l.user_id
           where l.oidc_subject = $1"#,
        subject,
    )
    .fetch_optional(db)
    .await?
    .map(|row| (row.user_id, row.archived));

    if found.is_none()
        && let Some(email) = verified_email
    {
        found = sqlx::query!(
            r#"update logins l set oidc_subject = $1
               from users u
               where u.id = l.user_id and l.email = lower($2) and l.oidc_subject is null
               returning l.user_id, u.archived_at is not null as "archived!""#,
            subject,
            email,
        )
        .fetch_optional(db)
        .await?
        .map(|row| (row.user_id, row.archived));
    }

    Ok(match found {
        Some((id, false)) => LoginUser::Allowed(id),
        _ => LoginUser::Refused,
    })
}

pub async fn create_session(
    db: &PgPool,
    user_id: Uuid,
    token_sha256: &[u8],
    idle_days: i32,
    ip: IpAddr,
    user_agent: Option<&str>,
) -> sqlx::Result<()> {
    sqlx::query!(
        "insert into login_sessions (user_id, token_sha256, expires_at, ip, user_agent)
         values ($1, $2, now() + make_interval(days => $3), $4, $5)",
        user_id,
        token_sha256,
        idle_days,
        ip as IpAddr,
        user_agent,
    )
    .execute(db)
    .await?;
    Ok(())
}

pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub email: String,
    pub name: String,
}

/// The valid session with this token hash, extending its expiry because it is
/// in use. None if it is unknown, expired or revoked, its user is archived or
/// the user's login is gone.
pub async fn use_session(
    db: &PgPool,
    token_sha256: &[u8],
    idle_days: i32,
) -> sqlx::Result<Option<Session>> {
    sqlx::query_as!(
        Session,
        "update login_sessions s
         set last_seen_at = now(), expires_at = now() + make_interval(days => $2)
         from users u, logins l
         where s.token_sha256 = $1 and u.id = s.user_id and l.user_id = s.user_id
           and s.revoked_at is null and s.expires_at > now() and u.archived_at is null
         returning s.id, s.user_id, l.email, u.name",
        token_sha256,
        idle_days,
    )
    .fetch_optional(db)
    .await
}

/// The user's roles: per group, and with no group on the whole organisation.
pub async fn roles(db: &PgPool, user_id: Uuid) -> sqlx::Result<Vec<(Option<Uuid>, String)>> {
    let rows = sqlx::query!(
        r#"select group_id as "group_id?", role as "role!" from group_members where user_id = $1
           union all
           select null, role from organisation_roles where user_id = $1"#,
        user_id,
    )
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| (row.group_id, row.role))
        .collect())
}

pub async fn revoke_session(db: &PgPool, token_sha256: &[u8]) -> sqlx::Result<()> {
    sqlx::query!(
        "update login_sessions set revoked_at = now()
         where token_sha256 = $1 and revoked_at is null",
        token_sha256,
    )
    .execute(db)
    .await?;
    Ok(())
}
