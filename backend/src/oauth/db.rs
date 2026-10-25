use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

/// An app that registered itself, or a trusted one an admin added.
pub struct Client {
    pub id: Uuid,
    pub name: String,
    pub redirect_uris: Vec<String>,
    /// Added by an admin: skips the consent screen.
    pub trusted: bool,
}

pub async fn register(db: &PgPool, name: &str, redirect_uris: &[String]) -> sqlx::Result<Uuid> {
    sqlx::query_scalar!(
        "insert into oauth_clients (name, redirect_uris) values ($1, $2) returning id",
        name,
        redirect_uris,
    )
    .fetch_one(db)
    .await
}

pub async fn client(db: &PgPool, id: Uuid) -> sqlx::Result<Option<Client>> {
    sqlx::query_as!(
        Client,
        "select id, name, redirect_uris, trusted from oauth_clients where id = $1",
        id
    )
    .fetch_optional(db)
    .await
}

/// What the user allowed, kept with a code until the app exchanges it.
pub struct Code {
    pub client_id: Uuid,
    pub user_id: Uuid,
    pub redirect_uri: String,
    pub code_challenge: String,
    pub scopes: Vec<String>,
    pub resource: String,
    /// The admin who connected the app while impersonating the user.
    pub impersonated_by: Option<Uuid>,
}

pub async fn create_code(
    db: &PgPool,
    code_sha256: &[u8],
    code: &Code,
    valid_seconds: f64,
) -> sqlx::Result<()> {
    sqlx::query!(
        "insert into oauth_codes (code_sha256, client_id, user_id, redirect_uri, code_challenge,
                                  scopes, resource, expires_at, impersonated_by)
         values ($1, $2, $3, $4, $5, $6, $7, now() + make_interval(secs => $8), $9)",
        code_sha256,
        code.client_id,
        code.user_id,
        code.redirect_uri,
        code.code_challenge,
        &code.scopes,
        code.resource,
        valid_seconds,
        code.impersonated_by,
    )
    .execute(db)
    .await?;
    Ok(())
}

/// The code with this hash, used up: a code works once, and only while valid.
pub async fn take_code(db: &PgPool, code_sha256: &[u8]) -> sqlx::Result<Option<Code>> {
    let row = sqlx::query!(
        r#"delete from oauth_codes where code_sha256 = $1
           returning client_id, user_id, redirect_uri, code_challenge, scopes, resource,
                     impersonated_by, expires_at > now() as "valid!""#,
        code_sha256,
    )
    .fetch_optional(db)
    .await?;
    Ok(row.filter(|row| row.valid).map(|row| Code {
        client_id: row.client_id,
        user_id: row.user_id,
        redirect_uri: row.redirect_uri,
        code_challenge: row.code_challenge,
        scopes: row.scopes,
        resource: row.resource,
        impersonated_by: row.impersonated_by,
    }))
}

/// New tokens' hashes and how long the access token lasts.
pub struct Tokens<'a> {
    pub access_sha256: &'a [u8],
    pub refresh_sha256: &'a [u8],
    pub access_seconds: f64,
}

/// Connects an app for a user, as allowed with a code.
pub async fn create_grant(db: &PgPool, code: &Code, tokens: &Tokens<'_>) -> sqlx::Result<()> {
    sqlx::query!(
        "insert into oauth_grants (client_id, user_id, scopes, resource, access_token_sha256,
                                   access_expires_at, refresh_token_sha256, impersonated_by)
         values ($1, $2, $3, $4, $5, now() + make_interval(secs => $6), $7, $8)",
        code.client_id,
        code.user_id,
        &code.scopes,
        code.resource,
        tokens.access_sha256,
        tokens.access_seconds,
        tokens.refresh_sha256,
        code.impersonated_by,
    )
    .execute(db)
    .await?;
    Ok(())
}

/// Replaces a grant's tokens, given its current refresh token, if the app is
/// the grant's, the grant was used within `idle_days` and its user may still
/// log in. Given the refresh token it had before, the grant ends: the token
/// was copied, and either holder may be the thief (RFC 9700, 4.14.2).
pub async fn refresh(
    db: &PgPool,
    refresh_sha256: &[u8],
    client_id: Uuid,
    tokens: &Tokens<'_>,
    idle_days: i32,
    resource: Option<&str>,
) -> sqlx::Result<Refreshed> {
    let refreshed = sqlx::query!(
        "update oauth_grants g
         set access_token_sha256 = $3, access_expires_at = now() + make_interval(secs => $4),
             previous_refresh_sha256 = g.refresh_token_sha256, refresh_token_sha256 = $5,
             last_used_at = now()
         from users u, logins l
         where g.refresh_token_sha256 = $1 and g.client_id = $2
           and g.last_used_at > now() - make_interval(days => $6)
           and ($7::text is null or g.resource = $7)
           and u.id = g.user_id and u.archived_at is null and l.user_id = g.user_id
         returning g.scopes, g.resource",
        refresh_sha256,
        client_id,
        tokens.access_sha256,
        tokens.access_seconds,
        tokens.refresh_sha256,
        idle_days,
        resource,
    )
    .fetch_optional(db)
    .await?;
    if let Some(grant) = refreshed {
        return Ok(Refreshed::Renewed(grant.scopes));
    }
    // A refresh token that was already replaced: someone else has a copy.
    let reused = sqlx::query!(
        "delete from oauth_grants where previous_refresh_sha256 = $1",
        refresh_sha256
    )
    .execute(db)
    .await?
    .rows_affected()
        > 0;
    Ok(if reused {
        Refreshed::Reused
    } else {
        Refreshed::Refused
    })
}

pub enum Refreshed {
    /// New tokens, with the grant's scopes.
    Renewed(Vec<String>),
    /// An old refresh token came back, so the grant was ended.
    Reused,
    /// Unknown, idle too long, another app's or resource's, or its user may
    /// no longer log in.
    Refused,
}

/// Who an access token acts for, and what for.
pub struct Access {
    pub grant_id: Uuid,
    pub user_id: Uuid,
    pub email: String,
    pub name: String,
    pub scopes: Vec<String>,
    pub resource: String,
    /// The admin impersonating the user, with their name, when they connected
    /// the app.
    pub impersonated_by: Option<Uuid>,
    pub impersonator_name: Option<String>,
}

/// The grant of a valid access token, marked as used. None if it is unknown
/// or expired, or its user may no longer log in.
pub async fn use_access(db: &PgPool, access_sha256: &[u8]) -> sqlx::Result<Option<Access>> {
    sqlx::query_as!(
        Access,
        r#"update oauth_grants g set last_used_at = now()
           from users u, logins l
           where g.access_token_sha256 = $1 and g.access_expires_at > now()
             and u.id = g.user_id and u.archived_at is null and l.user_id = g.user_id
           returning g.id as grant_id, g.user_id, l.email, u.name, g.scopes, g.resource,
                     g.impersonated_by,
                     (select a.name from users a where a.id = g.impersonated_by)
                         as impersonator_name"#,
        access_sha256,
    )
    .fetch_optional(db)
    .await
}

/// An app the user connected, for the list where they can disconnect it.
#[derive(Serialize)]
pub struct Connection {
    pub id: Uuid,
    pub app: String,
    /// The host its first return address goes to: who the app really is,
    /// as the name is only what it says.
    pub host: String,
    pub scopes: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
}

pub async fn connections(db: &PgPool, user_id: Uuid) -> sqlx::Result<Vec<Connection>> {
    sqlx::query_as!(
        Connection,
        r#"select g.id, c.name as app,
                  coalesce(substring(c.redirect_uris[1] from '^[a-z]+://([^/:?#]+)'), '')
                      as "host!",
                  g.scopes, g.created_at, g.last_used_at
           from oauth_grants g join oauth_clients c on c.id = g.client_id
           where g.user_id = $1 and g.impersonated_by is null
           order by g.created_at desc"#,
        user_id,
    )
    .fetch_all(db)
    .await
}

/// Disconnects one of the user's apps; false if the user has no such grant.
pub async fn disconnect(db: &PgPool, user_id: Uuid, id: Uuid) -> sqlx::Result<bool> {
    let result = sqlx::query!(
        "delete from oauth_grants where id = $1 and user_id = $2",
        id,
        user_id
    )
    .execute(db)
    .await?;
    Ok(result.rows_affected() == 1)
}

/// Removes what's no longer of use: expired codes, grants idle for longer
/// than `idle_days` (a day for those made while impersonating), and apps that registered themselves without a grant or code a
/// day later, so open registration can't fill the table.
pub async fn clean_up(db: &PgPool, idle_days: i32) -> sqlx::Result<()> {
    sqlx::query!("delete from oauth_codes where expires_at < now()")
        .execute(db)
        .await?;
    sqlx::query!(
        "delete from oauth_grants where last_used_at < now() - make_interval(days => $1)",
        idle_days
    )
    .execute(db)
    .await?;
    // An admin who closed the browser without stopping the impersonation.
    sqlx::query!(
        "delete from oauth_grants
         where impersonated_by is not null and last_used_at < now() - interval '1 day'"
    )
    .execute(db)
    .await?;
    sqlx::query!(
        "delete from oauth_clients c
         where not c.trusted and c.created_at < now() - interval '1 day'
           and not exists (select 1 from oauth_grants g where g.client_id = c.id)
           and not exists (select 1 from oauth_codes o where o.client_id = c.id)"
    )
    .execute(db)
    .await?;
    Ok(())
}

/// A trusted app, for the Apps page.
#[derive(Serialize)]
pub struct TrustedApp {
    /// Its client_id, which the app is configured with.
    pub id: Uuid,
    pub name: String,
    pub redirect_uris: Vec<String>,
    /// How many people use it, as connected apps.
    pub users: i64,
    pub created_at: DateTime<Utc>,
}

pub async fn trusted_apps(db: &PgPool) -> sqlx::Result<Vec<TrustedApp>> {
    sqlx::query_as!(
        TrustedApp,
        r#"select c.id, c.name, c.redirect_uris, c.created_at,
                  (select count(distinct g.user_id) from oauth_grants g where g.client_id = c.id)
                      as "users!"
           from oauth_clients c
           where c.trusted
           order by lower(c.name), c.id"#
    )
    .fetch_all(db)
    .await
}

pub async fn add_trusted(db: &PgPool, name: &str, redirect_uris: &[String]) -> sqlx::Result<Uuid> {
    sqlx::query_scalar!(
        "insert into oauth_clients (name, redirect_uris, trusted) values ($1, $2, true)
         returning id",
        name,
        redirect_uris,
    )
    .fetch_one(db)
    .await
}

/// False if there is no such trusted app.
pub async fn update_trusted(
    db: &PgPool,
    id: Uuid,
    name: &str,
    redirect_uris: &[String],
) -> sqlx::Result<bool> {
    let result = sqlx::query!(
        "update oauth_clients set name = $2, redirect_uris = $3 where id = $1 and trusted",
        id,
        name,
        redirect_uris,
    )
    .execute(db)
    .await?;
    Ok(result.rows_affected() == 1)
}

/// Removes a trusted app; its users' codes and tokens go with it. False if
/// there is no such trusted app.
pub async fn remove_trusted(db: &PgPool, id: Uuid) -> sqlx::Result<bool> {
    let result = sqlx::query!("delete from oauth_clients where id = $1 and trusted", id)
        .execute(db)
        .await?;
    Ok(result.rows_affected() == 1)
}

/// Ends the app connections an admin made while impersonating someone, with
/// codes not yet exchanged.
pub async fn end_impersonation(db: &PgPool, admin: Uuid) -> sqlx::Result<()> {
    sqlx::query!("delete from oauth_codes where impersonated_by = $1", admin)
        .execute(db)
        .await?;
    sqlx::query!("delete from oauth_grants where impersonated_by = $1", admin)
        .execute(db)
        .await?;
    Ok(())
}
