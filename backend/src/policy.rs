//! Access rules. Every access decision is made here, never in handlers; see
//! "Identity and access" in docs/architecture.md.

use uuid::Uuid;

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

/// Which flights an actor sees, as the parameters of the visibility condition
/// in the flight queries: every flight, or the actor's own flights and those of
/// the actor's team.
pub struct FlightScope {
    pub all: bool,
    pub user_id: Uuid,
}

pub fn flight_scope(actor: &Actor) -> FlightScope {
    FlightScope {
        all: matches!(actor.role, Role::AllFlights | Role::Admin),
        user_id: actor.user_id,
    }
}

/// Whether the actor may split or merge sessions holding flights of these
/// teams: the same rule as setting a flight's pilot.
pub fn change_sessions(
    role: Role,
    own_team: Option<Uuid>,
    flight_teams: &[Option<Uuid>],
) -> ApiResult<()> {
    match role {
        Role::Admin => Ok(()),
        Role::AllFlights => Err(ApiError::Forbidden),
        Role::Member => {
            let Some(own_team) = own_team else {
                return Err(ApiError::Forbidden);
            };
            if flight_teams.iter().any(|team| *team != Some(own_team)) {
                return Err(ApiError::Conflict(
                    "You can only change sessions of your own team.".to_owned(),
                ));
            }
            Ok(())
        }
    }
}

/// Whether the actor sees a drone or controller of this team: admins and the
/// all-flights role every one, members those of their own team.
pub fn see_equipment(role: Role, own_team: Option<Uuid>, item_team: Option<Uuid>) -> bool {
    match role {
        Role::Admin | Role::AllFlights => true,
        Role::Member => own_team.is_some() && item_team == own_team,
    }
}

/// Whether the actor may make this change to a drone or controller they see.
/// Admins change everything. Members change the name and holder of their
/// team's equipment, with a holder from their own team; not its team. The
/// all-flights role only views.
///
/// - `item_team`, `new_team`: the item's team now and as entered.
/// - `holder_team`: for a holder, their team if they're an active user in
///   one; `None` for no holder.
pub fn change_equipment(
    role: Role,
    own_team: Option<Uuid>,
    item_team: Option<Uuid>,
    new_team: Option<Uuid>,
    holder_team: Option<Option<Uuid>>,
) -> ApiResult<()> {
    match role {
        Role::Admin => Ok(()),
        Role::AllFlights => Err(ApiError::Forbidden),
        Role::Member => {
            if !see_equipment(role, own_team, item_team) || new_team != item_team {
                return Err(ApiError::Forbidden);
            }
            if holder_team.is_some_and(|team| team != own_team) {
                return Err(ApiError::Invalid(
                    "Choose someone of your own team.".to_owned(),
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
    /// The flights move to the new pilot's team.
    Follow(Uuid),
}

/// Whether the actor may set the pilot of these flights, and what happens to
/// their team; see "Setting a flight's pilot" in docs/architecture.md.
///
/// - `own_team`: the actor's own team, if any.
/// - `flight_teams`: the current team of each flight.
/// - `new_pilot_team`: the team of the chosen pilot, or `None` for unknown.
pub fn set_pilot(
    role: Role,
    own_team: Option<Uuid>,
    flight_teams: &[Option<Uuid>],
    new_pilot_team: Option<Uuid>,
) -> ApiResult<TeamChange> {
    match role {
        Role::Admin => Ok(match new_pilot_team {
            Some(team) => TeamChange::Follow(team),
            None => TeamChange::Keep,
        }),
        Role::AllFlights => Err(ApiError::Forbidden),
        Role::Member => {
            let Some(own_team) = own_team else {
                return Err(ApiError::Forbidden);
            };
            if flight_teams.iter().any(|team| *team != Some(own_team)) {
                return Err(ApiError::Conflict(
                    "You can only change flights of your own team.".to_owned(),
                ));
            }
            if new_pilot_team.is_some_and(|team| team != own_team) {
                return Err(ApiError::Invalid(
                    "Choose someone of your own team.".to_owned(),
                ));
            }
            Ok(TeamChange::Keep)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: Uuid = Uuid::from_u128(1);
    const B: Uuid = Uuid::from_u128(2);

    #[test]
    fn admin_moves_flights_to_the_new_pilots_team() {
        let (a, b) = (A, B);
        let change = set_pilot(Role::Admin, None, &[Some(a), None], Some(b));
        assert_eq!(change.ok(), Some(TeamChange::Follow(b)));
        let cleared = set_pilot(Role::Admin, None, &[Some(a)], None);
        assert_eq!(cleared.ok(), Some(TeamChange::Keep));
    }

    #[test]
    fn member_changes_name_and_holder_of_own_team_equipment_only() {
        let (own, other) = (A, B);
        let member =
            |item, new, holder| change_equipment(Role::Member, Some(own), item, new, holder);
        assert!(member(Some(own), Some(own), None).is_ok());
        assert!(member(Some(own), Some(own), Some(Some(own))).is_ok());
        // Not its team, not a holder from elsewhere, not other equipment.
        assert!(member(Some(own), None, None).is_err());
        assert!(member(Some(own), Some(own), Some(Some(other))).is_err());
        assert!(member(Some(own), Some(own), Some(None)).is_err());
        assert!(member(Some(other), Some(other), None).is_err());
        // No team and team-less equipment: both None, still not theirs.
        assert!(!see_equipment(Role::Member, None, None));
        assert!(see_equipment(Role::AllFlights, None, Some(other)));
        assert!(change_equipment(Role::AllFlights, None, None, None, None).is_err());
    }

    #[test]
    fn member_changes_only_own_team_flights_to_own_team_pilots() {
        let (own, other) = (A, B);
        let ok = set_pilot(Role::Member, Some(own), &[Some(own)], Some(own));
        assert_eq!(ok.ok(), Some(TeamChange::Keep));
        // "This wasn't me": unknown pilot, and the flight stays in the team.
        let unknown = set_pilot(Role::Member, Some(own), &[Some(own)], None);
        assert_eq!(unknown.ok(), Some(TeamChange::Keep));
        // Their own flight from an earlier team, a queue flight, another team's pilot.
        assert!(set_pilot(Role::Member, Some(own), &[Some(own), Some(other)], None).is_err());
        assert!(set_pilot(Role::Member, Some(own), &[None], None).is_err());
        assert!(set_pilot(Role::Member, Some(own), &[Some(own)], Some(other)).is_err());
    }

    #[test]
    fn viewers_and_members_without_a_team_change_nothing() {
        let own = A;
        assert!(set_pilot(Role::AllFlights, Some(own), &[Some(own)], None).is_err());
        assert!(set_pilot(Role::Member, None, &[None], None).is_err());
    }

    /// A member can't split or merge a session that holds another team's
    /// flight, also when it was merged across teams by an admin.
    #[test]
    fn members_change_only_sessions_of_their_own_team() {
        let (own, other) = (A, B);
        assert!(change_sessions(Role::Member, Some(own), &[Some(own), Some(own)]).is_ok());
        assert!(change_sessions(Role::Member, Some(own), &[Some(own), Some(other)]).is_err());
        assert!(change_sessions(Role::Member, None, &[Some(own)]).is_err());
        assert!(change_sessions(Role::AllFlights, Some(own), &[Some(own)]).is_err());
        assert!(change_sessions(Role::Admin, None, &[Some(own), Some(other)]).is_ok());
    }
}
