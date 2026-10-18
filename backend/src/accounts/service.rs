use sqlx::PgPool;
use uuid::Uuid;

use super::db::{self, Account};
use crate::{
    auth::Actor,
    error::{ApiError, ApiResult},
    policy::{self, Role},
};

/// A lowercase email address, or None if it isn't one.
pub fn email(value: &str) -> Option<String> {
    let email = value.trim().to_lowercase();
    let (local, domain) = email.split_once('@')?;
    (!local.is_empty() && domain.contains('.') && !email.contains(char::is_whitespace))
        .then_some(email)
}

fn role(value: &str) -> ApiResult<Role> {
    Role::parse(value).ok_or_else(|| ApiError::Invalid("Choose a role.".to_owned()))
}

pub async fn list(db: &PgPool, actor: &Actor) -> ApiResult<Vec<Account>> {
    policy::require_admin(actor)?;
    Ok(db::list(db).await?)
}

pub async fn get(db: &PgPool, actor: &Actor, id: Uuid) -> ApiResult<Account> {
    policy::require_admin(actor)?;
    db::get(db, id).await?.ok_or(ApiError::NotFound)
}

/// Lets someone log in. The first login with this email ties it to their login.
pub async fn create(
    db: &PgPool,
    actor: &Actor,
    email: &str,
    role_name: &str,
) -> ApiResult<Account> {
    policy::require_admin(actor)?;
    let email = self::email(email)
        .ok_or_else(|| ApiError::Invalid("Enter a valid email address.".to_owned()))?;
    role(role_name)?;
    let id = db::insert(db, &email, role_name)
        .await?
        .ok_or_else(|| ApiError::Conflict("A user with this email already exists.".to_owned()))?;
    get(db, actor, id).await
}

/// Admins can't change their own role, so an install always keeps an admin.
pub async fn set_role(db: &PgPool, actor: &Actor, id: Uuid, role_name: &str) -> ApiResult<Account> {
    policy::require_admin(actor)?;
    role(role_name)?;
    if id == actor.account_id {
        return Err(ApiError::Conflict(
            "You can't change your own role.".to_owned(),
        ));
    }
    if !db::set_role(db, id, role_name).await? {
        return Err(ApiError::NotFound);
    }
    get(db, actor, id).await
}

/// Disabling logs the user out everywhere. Admins can't disable themselves.
pub async fn set_disabled(
    db: &PgPool,
    actor: &Actor,
    id: Uuid,
    disabled: bool,
) -> ApiResult<Account> {
    policy::require_admin(actor)?;
    if disabled && id == actor.account_id {
        return Err(ApiError::Conflict(
            "You can't disable your own account.".to_owned(),
        ));
    }
    if !db::set_disabled(db, id, disabled).await? {
        return Err(ApiError::NotFound);
    }
    get(db, actor, id).await
}
