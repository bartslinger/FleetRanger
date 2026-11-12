use std::net::IpAddr;

use sqlx::PgConnection;
use uuid::Uuid;

/// The user a login is for, if they may log in.
pub enum LoginUser {
    Allowed {
        user_id: Uuid,
        organisation_id: Uuid,
    },
    /// No login, or only archived users.
    Refused,
}

/// Finds the user a Google account logs in as, across organisations (with
/// `System`): someone in several organisations has a login in each. A first
/// login in an organisation is matched on its verified email, which ties
/// that login to the account from then on. With several, it is the
/// organisation used last.
pub async fn login_user(
    db: &mut PgConnection,
    subject: &str,
    verified_email: Option<&str>,
) -> sqlx::Result<LoginUser> {
    if let Some(email) = verified_email {
        sqlx::query!(
            "update logins set oidc_subject = $1
             where email = lower($2) and oidc_subject is null
               and organisation_id not in (select organisation_id from logins
                                           where oidc_subject = $1)",
            subject,
            email,
        )
        .execute(&mut *db)
        .await?;
    }
    let found = sqlx::query!(
        "select l.user_id, l.organisation_id
         from logins l join users u on u.id = l.user_id
         where l.oidc_subject = $1 and u.archived_at is null
         order by (select max(s.created_at) from login_sessions s
                   where s.user_id = l.user_id) desc nulls last,
                  l.created_at
         limit 1",
        subject,
    )
    .fetch_optional(&mut *db)
    .await?;
    Ok(match found {
        Some(row) => LoginUser::Allowed {
            user_id: row.user_id,
            organisation_id: row.organisation_id,
        },
        None => LoginUser::Refused,
    })
}

/// Starts a login session for a user, in their organisation (with `System`,
/// as the login comes before one is known).
pub async fn create_session(
    db: &mut PgConnection,
    user_id: Uuid,
    organisation_id: Uuid,
    token_sha256: &[u8],
    idle_days: i32,
    ip: IpAddr,
    user_agent: Option<&str>,
) -> sqlx::Result<()> {
    sqlx::query!(
        "insert into login_sessions (organisation_id, user_id, token_sha256, expires_at, ip, user_agent)
         values ($1, $2, $3, now() + make_interval(days => $4), $5, $6)",
        organisation_id,
        user_id,
        token_sha256,
        idle_days,
        ip as IpAddr,
        user_agent,
    )
    .execute(&mut *db)
    .await?;
    Ok(())
}

pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub organisation_id: Uuid,
    pub email: String,
    pub name: String,
}

/// The valid session with this token hash, extending its expiry because it is
/// in use (with `System`: the session says which organisation the request is
/// in). None if it is unknown, expired or revoked, its user is archived or
/// the user's login is gone.
pub async fn use_session(
    db: &mut PgConnection,
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
         returning s.id, s.user_id, s.organisation_id, l.email, u.name",
        token_sha256,
        idle_days,
    )
    .fetch_optional(&mut *db)
    .await
}

pub struct ImpersonatedUser {
    pub name: String,
    pub email: Option<String>,
}

/// A user an admin impersonates, unless archived: with or without a login.
pub async fn impersonated_user(
    db: &mut PgConnection,
    user_id: Uuid,
) -> sqlx::Result<Option<ImpersonatedUser>> {
    sqlx::query_as!(
        ImpersonatedUser,
        r#"select u.name, l.email as "email?" from users u
           left join logins l on l.user_id = u.id
           where u.id = $1 and u.archived_at is null"#,
        user_id,
    )
    .fetch_optional(&mut *db)
    .await
}

/// The user's roles: per group, and with no group on the whole organisation.
pub async fn roles(
    db: &mut PgConnection,
    user_id: Uuid,
) -> sqlx::Result<Vec<(Option<Uuid>, String)>> {
    let rows = sqlx::query!(
        r#"select group_id as "group_id?", role as "role!" from group_members where user_id = $1
           union all
           select null, role from organisation_roles where user_id = $1"#,
        user_id,
    )
    .fetch_all(&mut *db)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| (row.group_id, row.role))
        .collect())
}

/// Ends a login session (with `System`: also an expired one, whose request
/// has no organisation); its user and their organisation, if it was still open.
pub async fn revoke_session(
    db: &mut PgConnection,
    token_sha256: &[u8],
) -> sqlx::Result<Option<(Uuid, Uuid)>> {
    let row = sqlx::query!(
        "update login_sessions set revoked_at = now()
         where token_sha256 = $1 and revoked_at is null
         returning user_id, organisation_id",
        token_sha256,
    )
    .fetch_optional(&mut *db)
    .await?;
    Ok(row.map(|row| (row.user_id, row.organisation_id)))
}

/// Whether there is more than one group.
pub async fn several_groups(db: &mut PgConnection) -> sqlx::Result<bool> {
    sqlx::query_scalar!(r#"select count(*) > 1 as "several!" from groups"#)
        .fetch_one(&mut *db)
        .await
}
