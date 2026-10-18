//! Access rules. Every access decision is made here, never in handlers; see
//! "Identity and access" in docs/architecture.md.

use uuid::Uuid;

use crate::{
    auth::Actor,
    error::{ApiError, ApiResult},
};

/// An account's role, stored in `accounts.role`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Member,
    /// Sees every flight, without managing anything.
    AllFlights,
    /// Sees every flight and manages accounts, pilots, teams and devices.
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

/// Managing accounts, pilots, teams and equipment is for admins only.
pub fn require_admin(actor: &Actor) -> ApiResult<()> {
    match actor.role {
        Role::Admin => Ok(()),
        Role::Member | Role::AllFlights => Err(ApiError::Forbidden),
    }
}

/// Which flights an actor sees, as the parameters of the visibility condition
/// in the flight queries: every flight, or the flights of the actor's own pilot
/// and of that pilot's team.
pub struct FlightScope {
    pub all: bool,
    pub account_id: Uuid,
}

pub fn flight_scope(actor: &Actor) -> FlightScope {
    FlightScope {
        all: matches!(actor.role, Role::AllFlights | Role::Admin),
        account_id: actor.account_id,
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
/// - `own_team`: the team of the actor's own pilot, if any.
/// - `flight_teams`: the current team of each flight.
/// - `new_pilot_team`: the team of the chosen pilot, or `None` for nobody.
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
                    "Choose a pilot of your own team.".to_owned(),
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
    fn member_changes_only_own_team_flights_to_own_team_pilots() {
        let (own, other) = (A, B);
        let ok = set_pilot(Role::Member, Some(own), &[Some(own)], Some(own));
        assert_eq!(ok.ok(), Some(TeamChange::Keep));
        // "This wasn't me": nobody, and the flight stays in the team.
        let nobody = set_pilot(Role::Member, Some(own), &[Some(own)], None);
        assert_eq!(nobody.ok(), Some(TeamChange::Keep));
        // Their own flight from an earlier team, a queue flight, another team's pilot.
        assert!(set_pilot(Role::Member, Some(own), &[Some(own), Some(other)], None).is_err());
        assert!(set_pilot(Role::Member, Some(own), &[None], None).is_err());
        assert!(set_pilot(Role::Member, Some(own), &[Some(own)], Some(other)).is_err());
    }

    #[test]
    fn viewers_and_members_without_a_pilot_change_nothing() {
        let own = A;
        assert!(set_pilot(Role::AllFlights, Some(own), &[Some(own)], None).is_err());
        assert!(set_pilot(Role::Member, None, &[None], None).is_err());
    }
}
