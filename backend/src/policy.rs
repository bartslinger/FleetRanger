//! Access rules. Every access decision is made here, never in handlers; see
//! "Identity and access" in docs/architecture.md.

use uuid::Uuid;

use crate::auth::Actor;

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
