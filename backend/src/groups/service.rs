use serde::Serialize;
use sqlx::PgConnection;
use uuid::Uuid;

use super::db::{self, Deleted, Group, Member};
use crate::{
    auth::Actor,
    error::{self, ApiError, ApiResult},
    policy::{self, Permission, Role},
};

/// A group with its members, for its page.
#[derive(Serialize)]
pub struct GroupDetail {
    #[serde(flatten)]
    pub group: Group,
    pub member_list: Vec<Member>,
}

/// Every group, for whoever manages groups on the whole organisation.
pub async fn list(db: &mut PgConnection, actor: &Actor) -> ApiResult<Vec<Group>> {
    policy::require(actor, Permission::GroupsManage, None)?;
    Ok(db::list(db, None).await?)
}

pub async fn get(db: &mut PgConnection, actor: &Actor, id: Uuid) -> ApiResult<GroupDetail> {
    policy::require(actor, Permission::GroupsManage, Some(id))?;
    let group = db::list(db, Some(id))
        .await?
        .pop()
        .ok_or(ApiError::NotFound)?;
    Ok(GroupDetail {
        group,
        member_list: db::members(db, id).await?,
    })
}

pub async fn create(db: &mut PgConnection, actor: &Actor, name: &str) -> ApiResult<GroupDetail> {
    policy::require(actor, Permission::GroupsManage, None)?;
    let name = error::name(name)?;
    let id = db::insert(db, &name).await.map_err(name_taken)?;
    get(db, actor, id).await
}

pub async fn rename(
    db: &mut PgConnection,
    actor: &Actor,
    id: Uuid,
    name: &str,
) -> ApiResult<GroupDetail> {
    policy::require(actor, Permission::GroupsManage, Some(id))?;
    let name = error::name(name)?;
    if !db::rename(db, id, &name).await.map_err(name_taken)? {
        return Err(ApiError::NotFound);
    }
    get(db, actor, id).await
}

/// Gives a member these roles in the group, FleetRanger's and apps', adding
/// them if they aren't in it yet: at least one, and only the roles a group
/// has (Admin is on the whole organisation only). Taking someone out of a
/// group is `remove_member`.
pub async fn set_roles(
    db: &mut PgConnection,
    actor: &Actor,
    id: Uuid,
    user_id: Uuid,
    roles: &[String],
    app_roles: &[Uuid],
) -> ApiResult<Vec<Member>> {
    policy::require(actor, Permission::GroupsManage, Some(id))?;
    let roles = group_roles(roles)?;
    if roles.is_empty() && app_roles.is_empty() {
        return Err(ApiError::Invalid("Choose at least one role.".to_owned()));
    }
    if !db::set_roles(db, id, user_id, &roles, app_roles)
        .await
        .map_err(error::unknown_app_role)?
    {
        return Err(ApiError::NotFound);
    }
    Ok(db::members(db, id).await?)
}

pub async fn remove_member(
    db: &mut PgConnection,
    actor: &Actor,
    id: Uuid,
    user_id: Uuid,
) -> ApiResult<Vec<Member>> {
    policy::require(actor, Permission::GroupsManage, Some(id))?;
    if !db::set_roles(db, id, user_id, &[], &[]).await? {
        return Err(ApiError::NotFound);
    }
    Ok(db::members(db, id).await?)
}

/// Deletes an empty group, or with `move_to` moves everything in it to
/// another group first. Moving needs the move permissions on both sides, as
/// moving each item would. The last group stays: everything is in a group.
pub async fn delete(
    db: &mut PgConnection,
    actor: &Actor,
    id: Uuid,
    move_to: Option<Uuid>,
) -> ApiResult<()> {
    policy::require(actor, Permission::GroupsManage, Some(id))?;
    if let Some(target) = move_to {
        if target == id {
            return Err(ApiError::Invalid("Choose another group.".to_owned()));
        }
        if !db::exists(db, target).await? {
            return Err(ApiError::Invalid("Choose an existing group.".to_owned()));
        }
        for permission in [
            Permission::FlightsMove,
            Permission::SessionsMove,
            Permission::EquipmentMove,
        ] {
            if !actor.grants.may_in_all(permission, &[id, target]) {
                return Err(ApiError::Forbidden);
            }
        }
    }
    match db::delete(db, id, move_to).await? {
        Deleted::Done => Ok(()),
        Deleted::NotFound => Err(ApiError::NotFound),
        Deleted::LastGroup => Err(ApiError::Conflict(
            "This is the only group, so it can't be deleted.".to_owned(),
        )),
        Deleted::NotEmpty => Err(ApiError::Conflict(
            "This group still holds flights, sessions or equipment. Move them first.".to_owned(),
        )),
    }
}

/// Group roles as stored, sorted and without repeats; refuses unknown ones
/// and Admin.
fn group_roles(roles: &[String]) -> ApiResult<Vec<String>> {
    let mut parsed = Vec::new();
    for role in roles {
        match Role::parse(role) {
            Some(role) if role != Role::Admin => parsed.push(role),
            _ => {
                return Err(ApiError::Invalid(
                    "Choose Viewer, Pilot or Editor.".to_owned(),
                ));
            }
        }
    }
    parsed.sort_unstable_by_key(|role| *role as u8);
    parsed.dedup();
    Ok(parsed
        .into_iter()
        .map(|role| role.as_str().to_owned())
        .collect())
}

fn name_taken(error: sqlx::Error) -> ApiError {
    if error::is_unique_violation(&error) {
        ApiError::Conflict("A group with this name already exists.".to_owned())
    } else {
        error.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing;

    /// A membership always has a role: adding someone without one is
    /// refused, as is Admin, which is on the whole organisation only.
    #[sqlx::test]
    async fn a_member_needs_a_group_role(pool: sqlx::PgPool) {
        let mut db = crate::testing::db(&pool).await;
        let admin = testing::user(&mut db, "A", &["admin"]).await;
        let admin = testing::actor(&mut db, admin).await;
        let pilot = testing::user(&mut db, "B", &[]).await;
        let north = testing::group(&mut db, "North").await;

        for roles in [vec![], vec!["admin".to_owned()]] {
            assert!(matches!(
                set_roles(&mut db, &admin, north, pilot, &roles, &[]).await,
                Err(ApiError::Invalid(_))
            ));
        }
        let roles = ["editor", "pilot", "pilot"].map(str::to_owned);
        let Ok(members) = set_roles(&mut db, &admin, north, pilot, &roles, &[]).await else {
            panic!("a member with roles is added");
        };
        assert_eq!(members[0].roles, ["pilot", "editor"]);
    }

    /// Only an empty group is deleted, unless everything moves along, which
    /// needs the move permissions on both groups; the last group stays.
    #[sqlx::test]
    async fn a_group_is_deleted_empty_or_with_everything_moved(pool: sqlx::PgPool) {
        let mut db = crate::testing::db(&pool).await;
        let admin = testing::user(&mut db, "A", &["admin"]).await;
        let admin = testing::actor(&mut db, admin).await;
        let north = testing::group(&mut db, "North").await;
        let south = testing::group(&mut db, "South").await;
        let editor = testing::user(&mut db, "B", &[]).await;
        testing::join(&mut db, editor, north).await;
        testing::join(&mut db, editor, south).await;
        let editor = testing::actor(&mut db, editor).await;
        sqlx::query("insert into aircraft (serial, group_id) values ('TESTDRONE', $1)")
            .bind(north)
            .execute(&mut *db)
            .await
            .unwrap();

        assert!(matches!(
            delete(&mut db, &admin, north, None).await,
            Err(ApiError::Conflict(_))
        ));
        assert!(matches!(
            delete(&mut db, &editor, north, Some(south)).await,
            Err(ApiError::Forbidden)
        ));
        assert!(delete(&mut db, &admin, north, Some(south)).await.is_ok());
        let drone: Uuid =
            sqlx::query_scalar("select group_id from aircraft where serial = 'TESTDRONE'")
                .fetch_one(&mut *db)
                .await
                .unwrap();
        assert_eq!(drone, south);
        assert!(!db::exists(&mut db, north).await.unwrap());

        let general = testing::general(&mut db).await;
        assert!(delete(&mut db, &admin, general, None).await.is_ok());
        assert!(matches!(
            delete(&mut db, &admin, south, None).await,
            Err(ApiError::Conflict(_))
        ));
    }
}
