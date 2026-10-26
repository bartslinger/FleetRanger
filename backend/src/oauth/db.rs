use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, PgPool, types::Json};
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
    /// The app the token belongs to.
    pub client_id: Uuid,
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
           returning g.id as grant_id, g.client_id, g.user_id, l.email, u.name, g.scopes, g.resource,
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
    /// Its roles, by name.
    pub roles: Json<Vec<AppRole>>,
    /// How many people use it, as connected apps.
    pub users: i64,
    pub created_at: DateTime<Utc>,
}

/// A role an app declared, as an admin typed it in.
#[derive(Serialize, Deserialize)]
pub struct AppRole {
    pub id: Uuid,
    /// What the app sees.
    pub key: String,
    /// What admins see.
    pub name: String,
    /// How many active users hold it, on a group or the organisation.
    pub holders: i64,
}

/// A role as an admin sets it on an app: without an id, a new one.
pub struct RoleFields<'a> {
    pub id: Option<Uuid>,
    pub key: &'a str,
    pub name: &'a str,
}

pub async fn trusted_apps(db: &PgPool) -> sqlx::Result<Vec<TrustedApp>> {
    sqlx::query_as!(
        TrustedApp,
        r#"select c.id, c.name, c.redirect_uris, c.created_at,
                  coalesce((select jsonb_agg(jsonb_build_object(
                                'id', r.id, 'key', r.key, 'name', r.name,
                                'holders', (select count(distinct h.user_id)
                                            from (select user_id from group_app_roles
                                                  where app_role_id = r.id
                                                  union all
                                                  select user_id from organisation_app_roles
                                                  where app_role_id = r.id) h
                                            join users u on u.id = h.user_id
                                            where u.archived_at is null))
                                order by lower(r.name), r.id)
                            from app_roles r where r.client_id = c.id),
                           '[]') as "roles!: Json<Vec<AppRole>>",
                  (select count(distinct g.user_id) from oauth_grants g where g.client_id = c.id)
                      as "users!"
           from oauth_clients c
           where c.trusted
           order by lower(c.name), c.id"#
    )
    .fetch_all(db)
    .await
}

/// Adds a trusted app with its roles. None if a role's id isn't new.
pub async fn add_trusted(
    db: &PgPool,
    name: &str,
    redirect_uris: &[String],
    roles: &[RoleFields<'_>],
) -> sqlx::Result<Option<Uuid>> {
    let mut tx = db.begin().await?;
    let id = sqlx::query_scalar!(
        "insert into oauth_clients (name, redirect_uris, trusted) values ($1, $2, true)
         returning id",
        name,
        redirect_uris,
    )
    .fetch_one(&mut *tx)
    .await?;
    if !set_app_roles(&mut tx, id, roles).await? {
        return Ok(None);
    }
    tx.commit().await?;
    Ok(Some(id))
}

/// False if there is no such trusted app, or a role's id isn't one of its
/// roles. Roles left out (None) stay as they are.
pub async fn update_trusted(
    db: &PgPool,
    id: Uuid,
    name: &str,
    redirect_uris: &[String],
    roles: Option<&[RoleFields<'_>]>,
) -> sqlx::Result<bool> {
    let mut tx = db.begin().await?;
    let result = sqlx::query!(
        "update oauth_clients set name = $2, redirect_uris = $3 where id = $1 and trusted",
        id,
        name,
        redirect_uris,
    )
    .execute(&mut *tx)
    .await?;
    if result.rows_affected() != 1 {
        return Ok(false);
    }
    if let Some(roles) = roles
        && !set_app_roles(&mut tx, id, roles).await?
    {
        return Ok(false);
    }
    tx.commit().await?;
    Ok(true)
}

/// Makes these the app's roles: roles left out are removed, with everyone's
/// grants of them. False if an id isn't one of the app's roles.
async fn set_app_roles(
    db: &mut PgConnection,
    client_id: Uuid,
    roles: &[RoleFields<'_>],
) -> sqlx::Result<bool> {
    let kept: Vec<Uuid> = roles.iter().filter_map(|role| role.id).collect();
    sqlx::query!(
        "delete from app_roles where client_id = $1 and id <> all($2)",
        client_id,
        &kept,
    )
    .execute(&mut *db)
    .await?;
    for role in roles {
        let done = match role.id {
            Some(id) => sqlx::query!(
                "update app_roles set key = $3, name = $4 where id = $1 and client_id = $2",
                id,
                client_id,
                role.key,
                role.name,
            ),
            None => sqlx::query!(
                "insert into app_roles (client_id, key, name) values ($1, $2, $3)",
                client_id,
                role.key,
                role.name,
            ),
        }
        .execute(&mut *db)
        .await?
        .rows_affected()
            == 1;
        if !done {
            return Ok(false);
        }
    }
    Ok(true)
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

/// A user's roles of one app: on the organisation, and per group.
#[derive(Default)]
pub struct HeldAppRoles {
    pub organisation: Vec<String>,
    pub groups: Vec<(Uuid, Vec<String>)>,
}

pub async fn held_app_roles(
    db: &PgPool,
    client_id: Uuid,
    user_id: Uuid,
) -> sqlx::Result<HeldAppRoles> {
    let organisation = sqlx::query_scalar!(
        "select r.key from organisation_app_roles o
         join app_roles r on r.id = o.app_role_id
         where r.client_id = $1 and o.user_id = $2
         order by r.key",
        client_id,
        user_id,
    )
    .fetch_all(db)
    .await?;
    let groups = sqlx::query!(
        r#"select g.group_id, array_agg(r.key order by r.key) as "keys!"
           from group_app_roles g
           join app_roles r on r.id = g.app_role_id
           where r.client_id = $1 and g.user_id = $2
           group by g.group_id
           order by g.group_id"#,
        client_id,
        user_id,
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|row| (row.group_id, row.keys))
    .collect();
    Ok(HeldAppRoles {
        organisation,
        groups,
    })
}

/// An app role to offer in the roles dialogs, with its app.
#[derive(Serialize)]
pub struct OfferedAppRole {
    pub id: Uuid,
    pub name: String,
    pub app_id: Uuid,
    pub app: String,
}

/// Every trusted app's roles, by app and then name.
pub async fn offered_app_roles(db: &PgPool) -> sqlx::Result<Vec<OfferedAppRole>> {
    sqlx::query_as!(
        OfferedAppRole,
        "select r.id, r.name, c.id as app_id, c.name as app
         from app_roles r
         join oauth_clients c on c.id = r.client_id
         where c.trusted
         order by lower(c.name), c.id, lower(r.name), r.id",
    )
    .fetch_all(db)
    .await
}
