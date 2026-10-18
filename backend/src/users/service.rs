use sqlx::PgPool;
use uuid::Uuid;

use super::db::{self, User, UserFields};
use crate::{
    auth::Actor,
    error::{self, ApiError, ApiResult},
    policy::{self, Role},
};

/// What an admin enters for a user.
pub struct UserInput {
    pub name: String,
    pub role: String,
    pub team_id: Option<Uuid>,
    /// Empty or None: no login.
    pub email: Option<String>,
}

/// A lowercase email address, or None if it isn't one.
pub fn email(value: &str) -> Option<String> {
    let email = value.trim().to_lowercase();
    let (local, domain) = email.split_once('@')?;
    (!local.is_empty() && domain.contains('.') && !email.contains(char::is_whitespace))
        .then_some(email)
}

pub async fn list(db: &PgPool, actor: &Actor, archived: bool) -> ApiResult<Vec<User>> {
    policy::require_admin(actor)?;
    Ok(db::list(db, archived).await?)
}

pub async fn get(db: &PgPool, actor: &Actor, id: Uuid) -> ApiResult<User> {
    policy::require_admin(actor)?;
    db::get(db, id).await?.ok_or(ApiError::NotFound)
}

pub async fn create(db: &PgPool, actor: &Actor, input: &UserInput) -> ApiResult<User> {
    policy::require_admin(actor)?;
    let checked = Checked::new(db, input).await?;
    let id = db::insert(db, &checked.fields())
        .await
        .map_err(email_taken)?;
    get(db, actor, id).await
}

/// Admins can't change their own role or remove their own login, so an install
/// always keeps an admin who can log in.
pub async fn update(db: &PgPool, actor: &Actor, id: Uuid, input: &UserInput) -> ApiResult<User> {
    let current = get(db, actor, id).await?;
    let checked = Checked::new(db, input).await?;
    if id == actor.user_id {
        if checked.role != actor.role {
            return Err(ApiError::Conflict(
                "You can't change your own role.".to_owned(),
            ));
        }
        if checked.email.is_none() && current.email.is_some() {
            return Err(ApiError::Conflict(
                "You can't remove your own login.".to_owned(),
            ));
        }
    }
    if !db::update(db, id, &checked.fields())
        .await
        .map_err(email_taken)?
    {
        return Err(ApiError::NotFound);
    }
    get(db, actor, id).await
}

/// Admins can't archive themselves.
pub async fn set_archived(db: &PgPool, actor: &Actor, id: Uuid, archived: bool) -> ApiResult<User> {
    policy::require_admin(actor)?;
    if archived && id == actor.user_id {
        return Err(ApiError::Conflict("You can't archive yourself.".to_owned()));
    }
    if !db::set_archived(db, id, archived).await? {
        return Err(ApiError::NotFound);
    }
    get(db, actor, id).await
}

/// Only a user without flights can be deleted; others are archived.
pub async fn delete(db: &PgPool, actor: &Actor, id: Uuid) -> ApiResult<()> {
    let user = get(db, actor, id).await?;
    if id == actor.user_id {
        return Err(ApiError::Conflict("You can't delete yourself.".to_owned()));
    }
    if user.flights > 0 || !db::delete(db, id).await? {
        return Err(ApiError::Conflict(
            "This user has flights, so they can't be deleted. Archive them instead.".to_owned(),
        ));
    }
    Ok(())
}

/// A user's input, checked: a name, a known role, an active team if any, and a
/// valid email address if any.
struct Checked {
    name: String,
    role: Role,
    team_id: Option<Uuid>,
    email: Option<String>,
}

impl Checked {
    async fn new(db: &PgPool, input: &UserInput) -> ApiResult<Self> {
        let name = error::name(&input.name)?;
        let role = Role::parse(&input.role)
            .ok_or_else(|| ApiError::Invalid("Choose a role.".to_owned()))?;
        if let Some(team) = input.team_id
            && !db::team_is_active(db, team).await?
        {
            return Err(ApiError::Invalid("Choose an active team.".to_owned()));
        }
        let email = match input.email.as_deref().map(str::trim) {
            None | Some("") => None,
            Some(value) => Some(
                email(value)
                    .ok_or_else(|| ApiError::Invalid("Enter a valid email address.".to_owned()))?,
            ),
        };
        Ok(Self {
            name,
            role,
            team_id: input.team_id,
            email,
        })
    }

    fn fields(&self) -> UserFields<'_> {
        UserFields {
            name: &self.name,
            role: self.role.as_str(),
            team_id: self.team_id,
            email: self.email.as_deref(),
        }
    }
}

fn email_taken(error: sqlx::Error) -> ApiError {
    if error::is_unique_violation(&error) {
        ApiError::Conflict("Another user already logs in with this email address.".to_owned())
    } else {
        error.into()
    }
}
