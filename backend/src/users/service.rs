use sqlx::PgPool;
use uuid::Uuid;

use super::db::{self, Refused, User, UserFields};
use crate::{
    auth::Actor,
    error::{self, ApiError, ApiResult},
    policy::{self, Permission, Role},
};

/// What an admin enters for a user.
pub struct UserInput {
    pub name: String,
    /// Roles on the whole organisation; none for someone who has roles only
    /// in groups, or none yet.
    pub roles: Vec<String>,
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
    Ok(db::list(db, archived, None).await?)
}

pub async fn get(db: &PgPool, actor: &Actor, id: Uuid) -> ApiResult<User> {
    policy::require(actor, Permission::UsersManage, None)?;
    db::list(db, true, Some(id))
        .await?
        .pop()
        .ok_or(ApiError::NotFound)
}

pub async fn create(db: &PgPool, actor: &Actor, input: &UserInput) -> ApiResult<User> {
    policy::require(actor, Permission::UsersManage, None)?;
    let checked = Checked::new(input)?;
    let id = db::insert(db, &checked.fields())
        .await
        .map_err(email_taken)?;
    get(db, actor, id).await
}

/// Admins can't remove their own login, so they can still log in; the last
/// active admin keeps Admin.
pub async fn update(db: &PgPool, actor: &Actor, id: Uuid, input: &UserInput) -> ApiResult<User> {
    let current = get(db, actor, id).await?;
    let checked = Checked::new(input)?;
    if id == actor.user_id && checked.email.is_none() && current.email.is_some() {
        return Err(ApiError::Conflict(
            "You can't remove your own login.".to_owned(),
        ));
    }
    db::update(db, id, &checked.fields())
        .await
        .map_err(email_taken)?
        .map_err(refusal)?;
    get(db, actor, id).await
}

/// Admins can't archive themselves, and the last active admin isn't archived.
pub async fn set_archived(db: &PgPool, actor: &Actor, id: Uuid, archived: bool) -> ApiResult<User> {
    policy::require(actor, Permission::UsersManage, None)?;
    if archived && id == actor.user_id {
        return Err(ApiError::Conflict("You can't archive yourself.".to_owned()));
    }
    db::set_archived(db, id, archived).await?.map_err(refusal)?;
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

fn refusal(refused: Refused) -> ApiError {
    match refused {
        Refused::NotFound => ApiError::NotFound,
        Refused::LastAdmin => ApiError::Conflict(
            "This is the only active admin. Make someone else an admin first.".to_owned(),
        ),
    }
}

/// A user's input, checked: a name, known roles, and a valid email address
/// if any.
struct Checked {
    name: String,
    roles: Vec<String>,
    email: Option<String>,
}

impl Checked {
    fn new(input: &UserInput) -> ApiResult<Self> {
        let name = error::name(&input.name)?;
        let mut roles = Vec::new();
        for role in &input.roles {
            roles.push(
                Role::parse(role)
                    .ok_or_else(|| ApiError::Invalid("Choose known roles.".to_owned()))?,
            );
        }
        roles.sort_unstable_by_key(|role| *role as u8);
        roles.dedup();
        let email = match input.email.as_deref().map(str::trim) {
            None | Some("") => None,
            Some(value) => Some(
                email(value)
                    .ok_or_else(|| ApiError::Invalid("Enter a valid email address.".to_owned()))?,
            ),
        };
        Ok(Self {
            name,
            roles: roles
                .into_iter()
                .map(|role| role.as_str().to_owned())
                .collect(),
            email,
        })
    }

    fn fields(&self) -> UserFields<'_> {
        UserFields {
            name: &self.name,
            roles: &self.roles,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing;

    fn input(name: &str, roles: &[&str]) -> UserInput {
        UserInput {
            name: name.to_owned(),
            roles: roles.iter().map(|role| (*role).to_owned()).collect(),
            email: None,
        }
    }

    /// The last active admin can't lose Admin or be archived, so nobody locks
    /// the organisation out; an archived admin doesn't count.
    #[sqlx::test]
    async fn the_last_active_admin_stays(db: PgPool) {
        let first = testing::user(&db, "A", &["admin"]).await;
        let second = testing::user(&db, "B", &["admin"]).await;
        let actor = testing::actor(&db, second).await;

        assert!(set_archived(&db, &actor, first, true).await.is_ok());
        assert!(matches!(
            update(&db, &actor, second, &input("B", &["viewer"])).await,
            Err(ApiError::Conflict(_))
        ));
        assert!(matches!(
            db::set_archived(&db, second, true).await,
            Ok(Err(Refused::LastAdmin))
        ));

        assert!(set_archived(&db, &actor, first, false).await.is_ok());
        assert!(
            update(&db, &actor, second, &input("B", &["viewer"]))
                .await
                .is_ok()
        );
    }
}
