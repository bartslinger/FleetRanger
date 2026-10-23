//! Access rules. Every access decision is made here, never in handlers; see
//! "Groups and roles" in docs/architecture.md. The code asks whether the
//! actor holds a permission on a group or the organisation, never whether
//! they're an admin.

mod grants;

use uuid::Uuid;

pub use grants::{Grants, Permission, Scope, roles_granting};

use crate::{
    auth::Actor,
    error::{ApiError, ApiResult},
};

/// The check, refusing with Forbidden: the permission on a group, or with
/// None on the whole organisation (for what has no group, such as users,
/// areas, batteries and logs).
pub fn require(actor: &Actor, permission: Permission, group: Option<Uuid>) -> ApiResult<()> {
    if actor.grants.may(permission, group) {
        Ok(())
    } else {
        Err(ApiError::Forbidden)
    }
}

/// Whether the actor may see and revoke a pairing code: its creator, and
/// whoever holds `controllers.manage` on the group the controller joins.
/// Any user may create one; see "Clients" in docs/architecture.md.
pub fn manage_pairing_code(actor: &Actor, created_by: Uuid, group: Option<Uuid>) -> bool {
    actor.user_id == created_by || actor.grants.may(Permission::ControllersManage, group)
}

/// Which flights an actor sees, as the parameters of the visibility condition
/// in the flight queries: those of the groups where they hold `flights.read`,
/// every flight when they hold it on the organisation, and, as a built-in
/// rule, the flights they flew themselves.
pub struct FlightScope {
    pub all: bool,
    pub user_id: Uuid,
    pub group_ids: Vec<Uuid>,
}

impl FlightScope {
    pub fn new(user_id: Uuid, grants: &Grants) -> Self {
        let read = grants.scope(Permission::FlightsRead);
        Self {
            all: read.all,
            user_id,
            group_ids: read.groups,
        }
    }
}

pub fn flight_scope(actor: &Actor) -> FlightScope {
    FlightScope::new(actor.user_id, &actor.grants)
}

/// The check on every group involved. Someone who holds the permission
/// nowhere is forbidden; someone who holds it elsewhere gets `refusal`.
fn on_all(
    grants: &Grants,
    permission: Permission,
    groups: &[Option<Uuid>],
    refusal: &str,
) -> ApiResult<()> {
    if grants.may_on_all(permission, groups) {
        Ok(())
    } else if grants.anywhere(permission) {
        Err(ApiError::Conflict(refusal.to_owned()))
    } else {
        Err(ApiError::Forbidden)
    }
}

/// Whether the actor may change which flights are in sessions of these
/// groups: `sessions.edit` on every session that changes, the ones flights
/// join and the ones they leave. The flights' own groups don't count: a
/// session never changes them.
pub fn change_sessions(grants: &Grants, session_groups: &[Option<Uuid>]) -> ApiResult<()> {
    on_all(
        grants,
        Permission::SessionsEdit,
        session_groups,
        "You can only change sessions of your own teams.",
    )
}

/// Whether the actor may move a session from one group to another, or with
/// None to the organisation: `sessions.move` on both, and to move its flights
/// along, `flights.move` on their groups and the new one.
///
/// - `flight_groups`: the group of each of its flights, if they move along.
pub fn move_session(
    grants: &Grants,
    from: Option<Uuid>,
    to: Option<Uuid>,
    flight_groups: Option<&[Option<Uuid>]>,
) -> ApiResult<()> {
    let session = grants.may_on_all(Permission::SessionsMove, &[from, to]);
    let flights = flight_groups.is_none_or(|groups| move_flights(grants, groups, to).is_ok());
    if session && flights {
        Ok(())
    } else {
        Err(ApiError::Forbidden)
    }
}

/// Whether the actor may make this change to a drone or controller they see:
/// `equipment.rename` on its group for the name and holder, and
/// `equipment.move` on both groups to move it. A holder from outside the
/// item's group also needs `equipment.move` (my choice, as holders are
/// dropped in package 5).
///
/// - `item_group`, `new_group`: the item's group now and as entered.
/// - `holder_groups`: for a holder, the groups they fly for; `None` for no
///   holder.
pub fn change_equipment(
    grants: &Grants,
    item_group: Option<Uuid>,
    new_group: Option<Uuid>,
    holder_groups: Option<&[Uuid]>,
) -> ApiResult<()> {
    let moves = grants.may(Permission::EquipmentMove, item_group);
    if !grants.may(Permission::EquipmentRename, item_group)
        || (new_group != item_group && !(moves && grants.may(Permission::EquipmentMove, new_group)))
    {
        return Err(ApiError::Forbidden);
    }
    let from_its_group = |groups: &[Uuid]| item_group.is_some_and(|group| groups.contains(&group));
    if !moves && holder_groups.is_some_and(|groups| !from_its_group(groups)) {
        return Err(ApiError::Invalid(
            "Choose someone of the equipment’s team.".to_owned(),
        ));
    }
    Ok(())
}

/// Whether the actor may make this change, such as setting the pilot or
/// archiving, to flights of these groups. Setting the pilot never moves a
/// flight, so only the flights' groups count; that the pilot flies is a rule
/// of the data, not of the actor.
pub fn change_flights(
    grants: &Grants,
    permission: Permission,
    flight_groups: &[Option<Uuid>],
) -> ApiResult<()> {
    on_all(
        grants,
        permission,
        flight_groups,
        "You can only change flights of your own teams.",
    )
}

/// Whether the actor may move flights of these groups to another group, or
/// with None to the queue: `flights.move` on every group involved.
pub fn move_flights(
    grants: &Grants,
    flight_groups: &[Option<Uuid>],
    to: Option<Uuid>,
) -> ApiResult<()> {
    if grants.may_on_all(Permission::FlightsMove, flight_groups)
        && grants.may(Permission::FlightsMove, to)
    {
        Ok(())
    } else {
        Err(ApiError::Forbidden)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: Uuid = Uuid::from_u128(1);
    const B: Uuid = Uuid::from_u128(2);
    const C: Uuid = Uuid::from_u128(3);

    // Today's roles as migration 0018 filled them.

    fn admin() -> Grants {
        Grants::from_stored([(None, "admin".to_owned())]).unwrap()
    }

    fn all_flights() -> Grants {
        Grants::from_stored([(None, "viewer".to_owned())]).unwrap()
    }

    /// A member of these teams: Pilot and Editor in each.
    fn member(teams: &[Uuid]) -> Grants {
        Grants::from_stored(
            teams
                .iter()
                .flat_map(|team| ["pilot", "editor"].map(|role| (Some(*team), role.to_owned()))),
        )
        .unwrap()
    }

    #[test]
    fn member_changes_name_and_holder_of_own_teams_equipment_only() {
        let (own, second, other) = (A, C, B);
        let mine = member(&[own, second]);
        let change =
            |item, new, holder: Option<&[Uuid]>| change_equipment(&mine, item, new, holder);
        assert!(change(Some(own), Some(own), None).is_ok());
        assert!(change(Some(second), Some(second), Some(&[own, second])).is_ok());
        // Not its team, not a holder from elsewhere, not other equipment.
        assert!(change(Some(own), None, None).is_err());
        assert!(change(Some(own), Some(own), Some(&[other])).is_err());
        assert!(change(Some(own), Some(own), Some(&[second])).is_err());
        assert!(change(Some(own), Some(own), Some(&[])).is_err());
        assert!(change(Some(other), Some(other), None).is_err());
        // No team and team-less equipment: still not theirs.
        let see = Permission::EquipmentRead;
        assert!(!member(&[]).may(see, None));
        assert!(all_flights().may(see, Some(other)));
        assert!(change_equipment(&all_flights(), None, None, None).is_err());
    }

    /// A member of two groups sets the pilot of both groups' flights, but
    /// not of another group's or the queue's; an admin of any.
    #[test]
    fn member_sets_pilots_of_own_groups_flights_only() {
        let (own, second, other) = (A, C, B);
        let set = |grants: &Grants, groups: &[Option<Uuid>]| {
            change_flights(grants, Permission::FlightsSetPilot, groups)
        };
        let mine = member(&[own, second]);
        assert!(set(&mine, &[Some(own), Some(second)]).is_ok());
        assert!(set(&mine, &[Some(own), Some(other)]).is_err());
        assert!(set(&mine, &[None]).is_err());
        assert!(set(&admin(), &[Some(other), None]).is_ok());
        assert!(set(&all_flights(), &[Some(own)]).is_err());
        assert!(set(&member(&[]), &[None]).is_err());
    }

    /// A member changes only their own groups' sessions: a change that
    /// touches another group's session, or one in the organisation, is
    /// refused as a whole.
    #[test]
    fn members_change_only_sessions_of_their_own_groups() {
        let (own, second, other) = (A, C, B);
        let mine = member(&[own, second]);
        assert!(change_sessions(&mine, &[Some(own), Some(second)]).is_ok());
        assert!(change_sessions(&mine, &[Some(own), Some(other)]).is_err());
        assert!(change_sessions(&mine, &[Some(own), None]).is_err());
        assert!(change_sessions(&member(&[]), &[Some(own)]).is_err());
        assert!(change_sessions(&all_flights(), &[Some(own)]).is_err());
        assert!(change_sessions(&admin(), &[Some(own), Some(other)]).is_ok());
    }

    /// Changing a session's group is admin work, also for an editor of both
    /// groups.
    #[test]
    fn only_admins_move_sessions() {
        let (own, other) = (A, B);
        let mine = member(&[own, other]);
        assert!(move_session(&mine, Some(own), Some(other), None).is_err());
        assert!(move_session(&admin(), Some(own), None, Some(&[Some(other)])).is_ok());
    }
}
