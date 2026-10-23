//! Access rules. Every access decision is made here, never in handlers; see
//! "Identity and access" in docs/architecture.md.

mod grants;

use uuid::Uuid;

pub use grants::{Grants, Permission, roles_granting};

use crate::{
    auth::Actor,
    error::{ApiError, ApiResult},
};

/// A user's role, stored in `users.role`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Member,
    /// Sees every flight, without managing anything.
    AllFlights,
    /// Sees every flight and manages users, teams and equipment.
    Admin,
}

impl Role {
    pub fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "member" => Self::Member,
            "all_flights" => Self::AllFlights,
            "admin" => Self::Admin,
            _ => return None,
        })
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Member => "member",
            Self::AllFlights => "all_flights",
            Self::Admin => "admin",
        }
    }
}

/// Managing users, teams and equipment is for admins only.
pub fn require_admin(actor: &Actor) -> ApiResult<()> {
    match actor.role {
        Role::Admin => Ok(()),
        Role::Member | Role::AllFlights => Err(ApiError::Forbidden),
    }
}

/// Whether the actor sees everyone's open pairing codes: admins do; others
/// see their own. Any user may create one; see "Clients" in
/// docs/architecture.md.
pub fn all_pairing_codes(actor: &Actor) -> bool {
    actor.role == Role::Admin
}

/// Whether the actor may revoke a pairing code: its creator and admins.
pub fn revoke_pairing_code(actor: &Actor, created_by: Uuid) -> bool {
    actor.user_id == created_by || all_pairing_codes(actor)
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

// A member can be in several teams (user's decision, 2026-10-19): what's
// theirs is what belongs to any of them. `own_teams` is that list, empty for
// someone in no team.

/// Whether a flight's (or item's) team is one of the member's.
fn among(own_teams: &[Uuid], team: Option<Uuid>) -> bool {
    team.is_some_and(|team| own_teams.contains(&team))
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

/// Whether the actor may split or merge sessions holding flights of these
/// groups: `sessions.edit` on every one of them. A session's own group
/// decides this from package 4.
pub fn change_sessions(grants: &Grants, flight_groups: &[Option<Uuid>]) -> ApiResult<()> {
    on_all(
        grants,
        Permission::SessionsEdit,
        flight_groups,
        "You can only change sessions of your own teams.",
    )
}

/// Whether the actor sees a drone or controller of this team: admins and the
/// all-flights role every one, members those of their own teams.
pub fn see_equipment(role: Role, own_teams: &[Uuid], item_team: Option<Uuid>) -> bool {
    match role {
        Role::Admin | Role::AllFlights => true,
        Role::Member => among(own_teams, item_team),
    }
}

/// Whether the actor may make this change to a drone or controller they see.
/// Admins change everything. Members change the name and holder of their
/// teams' equipment, with a holder from the item's team; not its team. The
/// all-flights role only views.
///
/// - `item_team`, `new_team`: the item's team now and as entered.
/// - `holder_teams`: for a holder, the teams they're an active user in;
///   `None` for no holder.
pub fn change_equipment(
    role: Role,
    own_teams: &[Uuid],
    item_team: Option<Uuid>,
    new_team: Option<Uuid>,
    holder_teams: Option<&[Uuid]>,
) -> ApiResult<()> {
    match role {
        Role::Admin => Ok(()),
        Role::AllFlights => Err(ApiError::Forbidden),
        Role::Member => {
            if !see_equipment(role, own_teams, item_team) || new_team != item_team {
                return Err(ApiError::Forbidden);
            }
            if holder_teams.is_some_and(|teams| !among(teams, item_team)) {
                return Err(ApiError::Invalid(
                    "Choose someone of the equipment’s team.".to_owned(),
                ));
            }
            Ok(())
        }
    }
}

/// What happens to the flights' team when their pilot is set.
#[derive(Debug, PartialEq, Eq)]
pub enum TeamChange {
    /// The flights keep their team.
    Keep,
    /// The flights move to the team the pilot was chosen with.
    Follow(Uuid),
}

/// Whether the actor may set the pilot of these flights, and what happens to
/// their team; see "Setting a flight's pilot" in docs/architecture.md. A
/// pilot is chosen with one of their teams, and the flights move to it, so
/// the actor needs `flights.set_pilot` on the flights' groups and that one.
/// Package 4 makes the pilot and the group independent.
///
/// - `flight_groups`: the current group of each flight.
/// - `new_pilot_group`: the group the pilot was chosen with, or `None` for
///   unknown.
pub fn set_pilot(
    grants: &Grants,
    flight_groups: &[Option<Uuid>],
    new_pilot_group: Option<Uuid>,
) -> ApiResult<TeamChange> {
    let change = match new_pilot_group {
        Some(group) => TeamChange::Follow(group),
        None => TeamChange::Keep,
    };
    change_flights(grants, Permission::FlightsSetPilot, flight_groups)?;
    if new_pilot_group.is_some_and(|group| !grants.may(Permission::FlightsSetPilot, Some(group))) {
        return Err(ApiError::Invalid(
            "Choose someone of your own teams.".to_owned(),
        ));
    }
    Ok(change)
}

/// Whether the actor may make this change, such as archiving, to flights of
/// these groups.
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
    fn admin_moves_flights_to_the_new_pilots_team() {
        let (a, b) = (A, B);
        let change = set_pilot(&admin(), &[Some(a), None], Some(b));
        assert_eq!(change.ok(), Some(TeamChange::Follow(b)));
        let cleared = set_pilot(&admin(), &[Some(a)], None);
        assert_eq!(cleared.ok(), Some(TeamChange::Keep));
    }

    #[test]
    fn member_changes_name_and_holder_of_own_teams_equipment_only() {
        let (own, second, other) = (A, C, B);
        let member = |item, new, holder: Option<&[Uuid]>| {
            change_equipment(Role::Member, &[own, second], item, new, holder)
        };
        assert!(member(Some(own), Some(own), None).is_ok());
        assert!(member(Some(second), Some(second), Some(&[own, second])).is_ok());
        // Not its team, not a holder from elsewhere, not other equipment.
        assert!(member(Some(own), None, None).is_err());
        assert!(member(Some(own), Some(own), Some(&[other])).is_err());
        assert!(member(Some(own), Some(own), Some(&[second])).is_err());
        assert!(member(Some(own), Some(own), Some(&[])).is_err());
        assert!(member(Some(other), Some(other), None).is_err());
        // No team and team-less equipment: still not theirs.
        assert!(!see_equipment(Role::Member, &[], None));
        assert!(see_equipment(Role::AllFlights, &[], Some(other)));
        assert!(change_equipment(Role::AllFlights, &[], None, None, None).is_err());
    }

    /// A member of two teams changes the flights of both, also moving them
    /// from one to the other with a pilot chosen with that team.
    #[test]
    fn member_changes_only_own_teams_flights_to_own_teams_pilots() {
        let (own, second, other) = (A, C, B);
        let mine = member(&[own, second]);
        let ok = set_pilot(&mine, &[Some(own), Some(second)], Some(second));
        assert_eq!(ok.ok(), Some(TeamChange::Follow(second)));
        // "This wasn't me": unknown pilot, and the flight stays in the team.
        let unknown = set_pilot(&mine, &[Some(own)], None);
        assert_eq!(unknown.ok(), Some(TeamChange::Keep));
        // Another team's flight, a queue flight, a pilot of another team.
        assert!(set_pilot(&mine, &[Some(own), Some(other)], None).is_err());
        assert!(set_pilot(&mine, &[None], None).is_err());
        assert!(set_pilot(&mine, &[Some(own)], Some(other)).is_err());
    }

    #[test]
    fn viewers_and_members_without_a_team_change_nothing() {
        let own = A;
        assert!(set_pilot(&all_flights(), &[Some(own)], None).is_err());
        assert!(set_pilot(&member(&[]), &[None], None).is_err());
    }

    /// A member can't split or merge a session that holds another team's
    /// flight, also when it was merged across teams by an admin.
    #[test]
    fn members_change_only_sessions_of_their_own_teams() {
        let (own, second, other) = (A, C, B);
        let mine = member(&[own, second]);
        assert!(change_sessions(&mine, &[Some(own), Some(second)]).is_ok());
        assert!(change_sessions(&mine, &[Some(own), Some(other)]).is_err());
        assert!(change_sessions(&member(&[]), &[Some(own)]).is_err());
        assert!(change_sessions(&all_flights(), &[Some(own)]).is_err());
        assert!(change_sessions(&admin(), &[Some(own), Some(other)]).is_ok());
    }
}
