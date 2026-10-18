use std::net::IpAddr;

use sqlx::PgPool;
use uuid::Uuid;

/// The account a login is for, if it may log in.
pub enum LoginAccount {
    Allowed(Uuid),
    /// No account, a disabled one, or the email belongs to another login.
    Refused,
}

/// Finds the account for a login by its subject ID. A first login is matched on
/// its verified email instead, which ties the account to the subject from then
/// on. Updates the name shown for the account.
pub async fn login_account(
    db: &PgPool,
    subject: &str,
    verified_email: Option<&str>,
    name: Option<&str>,
) -> sqlx::Result<LoginAccount> {
    let mut found = sqlx::query!(
        r#"update accounts set name = $2 where oidc_subject = $1
           returning id, disabled_at is not null as "disabled!""#,
        subject,
        name,
    )
    .fetch_optional(db)
    .await?
    .map(|row| (row.id, row.disabled));

    if found.is_none()
        && let Some(email) = verified_email
    {
        found = sqlx::query!(
            r#"update accounts set oidc_subject = $1, name = $3
               where email = lower($2) and oidc_subject is null
               returning id, disabled_at is not null as "disabled!""#,
            subject,
            email,
            name,
        )
        .fetch_optional(db)
        .await?
        .map(|row| (row.id, row.disabled));
    }

    Ok(match found {
        Some((id, false)) => LoginAccount::Allowed(id),
        _ => LoginAccount::Refused,
    })
}

pub async fn create_session(
    db: &PgPool,
    account_id: Uuid,
    token_sha256: &[u8],
    idle_days: i32,
    ip: IpAddr,
    user_agent: Option<&str>,
) -> sqlx::Result<()> {
    sqlx::query!(
        "insert into login_sessions (account_id, token_sha256, expires_at, ip, user_agent)
         values ($1, $2, now() + make_interval(days => $3), $4, $5)",
        account_id,
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
    pub account_id: Uuid,
    pub email: String,
    pub name: Option<String>,
}

/// The valid session with this token hash, extending its expiry because it is
/// in use. None if it is unknown, expired or revoked, or its account is disabled.
pub async fn use_session(
    db: &PgPool,
    token_sha256: &[u8],
    idle_days: i32,
) -> sqlx::Result<Option<Session>> {
    sqlx::query_as!(
        Session,
        "update login_sessions s
         set last_seen_at = now(), expires_at = now() + make_interval(days => $2)
         from accounts a
         where s.token_sha256 = $1 and a.id = s.account_id
           and s.revoked_at is null and s.expires_at > now() and a.disabled_at is null
         returning s.id, s.account_id, a.email, a.name",
        token_sha256,
        idle_days,
    )
    .fetch_optional(db)
    .await
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
