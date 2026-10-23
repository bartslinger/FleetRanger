use sqlx::PgPool;
use uuid::Uuid;

use super::db::{self, User, UserFields};
use crate::{
    auth::Actor,
    error::{self, ApiError, ApiResult},
    policy::{self, Permission},
};

/// The roles stored in `users.role` and set on the Users page. Access comes
/// from `organisation_roles` instead; the Users page moves to those in
/// package 6.
const ROLES: [&str; 3] = ["member", "all_flights", "admin"];

/// What an admin enters for a user.
pub struct UserInput {
    pub name: String,
    pub role: String,
    /// The teams they fly for; none for someone who doesn't fly.
    pub team_ids: Vec<Uuid>,
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
    policy::require(actor, Permission::UsersManage, None)?;
    Ok(db::list(db, archived).await?)
}

pub async fn get(db: &PgPool, actor: &Actor, id: Uuid) -> ApiResult<User> {
    policy::require(actor, Permission::UsersManage, None)?;
    db::get(db, id).await?.ok_or(ApiError::NotFound)
}

pub async fn create(db: &PgPool, actor: &Actor, input: &UserInput) -> ApiResult<User> {
    policy::require(actor, Permission::UsersManage, None)?;
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
        if checked.role != current.role {
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
    policy::require(actor, Permission::UsersManage, None)?;
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

/// A user's input, checked: a name, a known role, active teams if any, and a
/// valid email address if any.
struct Checked {
    name: String,
    role: &'static str,
    team_ids: Vec<Uuid>,
    email: Option<String>,
}

impl Checked {
    async fn new(db: &PgPool, input: &UserInput) -> ApiResult<Self> {
        let name = error::name(&input.name)?;
        let role = ROLES
            .into_iter()
            .find(|role| *role == input.role)
            .ok_or_else(|| ApiError::Invalid("Choose a role.".to_owned()))?;
        let mut team_ids = input.team_ids.clone();
        team_ids.sort_unstable();
        team_ids.dedup();
        for team in &team_ids {
            if !db::team_is_active(db, *team).await? {
                return Err(ApiError::Invalid("Choose active teams.".to_owned()));
            }
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
            team_ids,
            email,
        })
    }

    fn fields(&self) -> UserFields<'_> {
        UserFields {
            name: &self.name,
            role: self.role,
            team_ids: &self.team_ids,
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
