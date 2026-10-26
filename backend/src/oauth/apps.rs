//! Trusted apps, which an admin adds on the Apps page: they skip the consent
//! screen, but still act only as their user. See "Apps" in
//! docs/architecture.md.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use super::{
    db::{self, OfferedAppRole, RoleFields, TrustedApp},
    routes::redirect_uri_allowed,
};
use crate::{
    auth::Actor,
    error::{self, ApiError, ApiResult},
    policy::{self, Permission},
};

#[derive(Deserialize)]
pub(super) struct AppInput {
    name: String,
    redirect_uris: Vec<String>,
    #[serde(default)]
    roles: Vec<RoleInput>,
}

/// One of the app's roles; without an id, a new one.
#[derive(Deserialize)]
struct RoleInput {
    id: Option<Uuid>,
    key: String,
    name: String,
}

/// An app's input as checked: name, return addresses and roles.
type Checked<'a> = (&'a str, Vec<String>, Vec<RoleFields<'a>>);

impl AppInput {
    /// The name trimmed, return addresses as apps may register them, and
    /// roles with keys the app can rely on.
    fn checked(&self) -> ApiResult<Checked<'_>> {
        let name = self.name.trim();
        if name.is_empty() || name.chars().count() > 100 {
            return Err(ApiError::Invalid(
                "Enter a name of at most 100 characters.".to_owned(),
            ));
        }
        let uris: Vec<String> = self
            .redirect_uris
            .iter()
            .map(|uri| uri.trim().to_owned())
            .filter(|uri| !uri.is_empty())
            .collect();
        if uris.is_empty() {
            return Err(ApiError::Invalid(
                "Enter at least one return address.".to_owned(),
            ));
        }
        if uris.len() > 10 {
            return Err(ApiError::Invalid(
                "Enter no more than 10 return addresses.".to_owned(),
            ));
        }
        // HTTPS only: a trusted app isn't asked about, so it can't use the
        // user's own computer, where any program could listen (RFC 6749,
        // 10.2), as self-registered apps may.
        if let Some(uri) = uris
            .iter()
            .find(|uri| !uri.starts_with("https://") || !redirect_uri_allowed(uri))
        {
            return Err(ApiError::Invalid(format!(
                "{uri} isn’t a valid return address. Use the full address, starting with https://."
            )));
        }
        if let Some((_, uri)) = uris
            .iter()
            .enumerate()
            .find(|(index, uri)| uris[..*index].contains(uri))
        {
            return Err(ApiError::Invalid(format!(
                "{uri} is there twice. Enter each return address once."
            )));
        }
        let roles = self
            .roles
            .iter()
            .map(RoleInput::checked)
            .collect::<ApiResult<Vec<_>>>()?;
        if roles.len() > 50 {
            return Err(ApiError::Invalid("Enter no more than 50 roles.".to_owned()));
        }
        for (index, role) in roles.iter().enumerate() {
            let before = &roles[..index];
            if before.iter().any(|other| other.key == role.key) {
                return Err(ApiError::Invalid(format!(
                    "{} is there twice. Give each role its own key.",
                    role.key
                )));
            }
            if before
                .iter()
                .any(|other| other.name.to_lowercase() == role.name.to_lowercase())
            {
                return Err(ApiError::Invalid(format!(
                    "{} is there twice. Give each role its own name.",
                    role.name
                )));
            }
        }
        Ok((name, uris, roles))
    }
}

impl RoleInput {
    /// The key as the app will see it, and the name trimmed.
    fn checked(&self) -> ApiResult<RoleFields<'_>> {
        let key = self.key.trim();
        let mut chars = key.chars();
        let valid = chars.next().is_some_and(|c| c.is_ascii_lowercase())
            && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
            && key.len() <= 64;
        if !valid {
            return Err(ApiError::Invalid(format!(
                "{key} isn’t a valid key. Use lowercase letters, digits and underscores, starting with a letter, at most 64."
            )));
        }
        let name = self.name.trim();
        if name.is_empty() || name.chars().count() > 100 {
            return Err(ApiError::Invalid(format!(
                "Enter a name of at most 100 characters for {key}."
            )));
        }
        Ok(RoleFields {
            id: self.id,
            key,
            name,
        })
    }
}

pub(super) async fn list(
    actor: Actor,
    State(db): State<PgPool>,
) -> ApiResult<Json<Vec<TrustedApp>>> {
    policy::require(&actor, Permission::AppsManage, None)?;
    Ok(Json(db::trusted_apps(&db).await?))
}

/// The apps' roles, for the dialogs that give roles to users and members.
pub(super) async fn roles(
    actor: Actor,
    State(db): State<PgPool>,
) -> ApiResult<Json<Vec<OfferedAppRole>>> {
    if !actor.grants.may(Permission::UsersManage, None) {
        policy::require(&actor, Permission::GroupsManage, None)?;
    }
    Ok(Json(db::offered_app_roles(&db).await?))
}
#[derive(Serialize)]
pub(super) struct Added {
    id: Uuid,
}

pub(super) async fn add(
    actor: Actor,
    State(db): State<PgPool>,
    Json(input): Json<AppInput>,
) -> ApiResult<(StatusCode, Json<Added>)> {
    policy::require(&actor, Permission::AppsManage, None)?;
    let (name, uris, roles) = input.checked()?;
    let id = db::add_trusted(&db, name, &uris, &roles)
        .await
        .map_err(roles_taken)?
        .ok_or_else(|| ApiError::Invalid("A new app has only new roles.".to_owned()))?;
    tracing::info!(%id, name, by = %actor.user_id, "trusted app added");
    Ok((StatusCode::CREATED, Json(Added { id })))
}

pub(super) async fn update(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(input): Json<AppInput>,
) -> ApiResult<StatusCode> {
    policy::require(&actor, Permission::AppsManage, None)?;
    let (name, uris, roles) = input.checked()?;
    if !db::update_trusted(&db, id, name, &uris, &roles)
        .await
        .map_err(roles_taken)?
    {
        return Err(ApiError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

/// Two roles that swap keys or names meet the unique constraints halfway.
fn roles_taken(error: sqlx::Error) -> ApiError {
    if error::is_unique_violation(&error) {
        ApiError::Conflict(
            "Roles can’t swap keys or names in one change. Save in two steps.".to_owned(),
        )
    } else {
        error.into()
    }
}

/// Removes an app: its users' tokens stop working at once.
pub(super) async fn remove(
    actor: Actor,
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    policy::require(&actor, Permission::AppsManage, None)?;
    if !db::remove_trusted(&db, id).await? {
        return Err(ApiError::NotFound);
    }
    tracing::info!(%id, by = %actor.user_id, "trusted app removed");
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oauth::{
        db::{Code, Tokens},
        sha256,
    };

    /// Clean-up keeps a trusted app without users, as it removes registered
    /// ones; removing a trusted app ends its users' access at once.
    #[sqlx::test]
    async fn removing_a_trusted_app_ends_its_tokens(db: PgPool) {
        let user: Uuid = sqlx::query_scalar("insert into users (name) values ('A') returning id")
            .fetch_one(&db)
            .await
            .unwrap();
        sqlx::query("insert into logins (user_id, email) values ($1, 'a@example.org')")
            .bind(user)
            .execute(&db)
            .await
            .unwrap();
        let uris = ["https://app.example.org/callback".to_owned()];
        let app = db::add_trusted(&db, "Test app", &uris, &[])
            .await
            .unwrap()
            .unwrap();
        sqlx::query("update oauth_clients set created_at = now() - interval '2 days'")
            .execute(&db)
            .await
            .unwrap();
        db::clean_up(&db, 30).await.unwrap();
        assert!(
            db::client(&db, app)
                .await
                .unwrap()
                .is_some_and(|c| c.trusted)
        );

        let code = Code {
            client_id: app,
            user_id: user,
            redirect_uri: uris[0].clone(),
            code_challenge: String::new(),
            scopes: vec!["sessions:list".to_owned()],
            resource: "https://fleet.example.org/api/v1".to_owned(),
            impersonated_by: None,
        };
        let tokens = Tokens {
            access_sha256: &sha256("access"),
            refresh_sha256: &sha256("refresh"),
            access_seconds: 60.0,
        };
        db::create_grant(&db, &code, &tokens).await.unwrap();
        assert!(
            db::use_access(&db, &sha256("access"))
                .await
                .unwrap()
                .is_some()
        );

        assert!(db::remove_trusted(&db, app).await.unwrap());
        assert!(
            db::use_access(&db, &sha256("access"))
                .await
                .unwrap()
                .is_none()
        );
    }
}

#[cfg(test)]
mod impersonation_tests {
    use super::*;
    use crate::oauth::{
        db::{Code, Tokens},
        sha256,
    };

    async fn user(db: &PgPool, email: &str) -> Uuid {
        let id: Uuid = sqlx::query_scalar("insert into users (name) values ($1) returning id")
            .bind(email)
            .fetch_one(db)
            .await
            .unwrap();
        sqlx::query("insert into logins (user_id, email) values ($1, $2)")
            .bind(id)
            .bind(email)
            .execute(db)
            .await
            .unwrap();
        id
    }

    /// An app an admin opened while impersonating someone isn't in that
    /// person's connected apps, and ends with the impersonation; the
    /// person's own connection stays.
    #[sqlx::test]
    async fn impersonated_connections_end_with_the_impersonation(db: PgPool) {
        let pilot = user(&db, "pilot@example.org").await;
        let admin = user(&db, "admin@example.org").await;
        let uris = ["https://app.example.org/callback".to_owned()];
        let app = db::add_trusted(&db, "Test app", &uris, &[])
            .await
            .unwrap()
            .unwrap();
        let connect = async |impersonated_by: Option<Uuid>, token: &str| {
            let code = Code {
                client_id: app,
                user_id: pilot,
                redirect_uri: uris[0].clone(),
                code_challenge: String::new(),
                scopes: vec!["sessions:list".to_owned()],
                resource: "https://fleet.example.org/api/v1".to_owned(),
                impersonated_by,
            };
            let tokens = Tokens {
                access_sha256: &sha256(token),
                refresh_sha256: &sha256(&format!("{token}-refresh")),
                access_seconds: 60.0,
            };
            db::create_grant(&db, &code, &tokens).await.unwrap();
        };
        connect(None, "own").await;
        connect(Some(admin), "impersonated").await;

        let access = db::use_access(&db, &sha256("impersonated"))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(access.user_id, pilot);
        assert_eq!(access.impersonated_by, Some(admin));
        assert_eq!(db::connections(&db, pilot).await.unwrap().len(), 1);

        db::end_impersonation(&db, admin).await.unwrap();
        assert!(
            db::use_access(&db, &sha256("impersonated"))
                .await
                .unwrap()
                .is_none()
        );
        assert!(db::use_access(&db, &sha256("own")).await.unwrap().is_some());
    }
}
