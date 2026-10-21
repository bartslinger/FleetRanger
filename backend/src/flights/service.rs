use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use super::db::{
    self, AreaOption, Device, Filter, FlightDetail, FlightSummary, Neighbours, PilotChoice,
    PilotOption, Sort, TeamChoice, TeamOption,
};
use crate::{
    auth::Actor,
    error::{ApiError, ApiResult},
    policy::{self, Role, TeamChange},
};

/// One page of the flights the actor may see that pass the filter, and how
/// many there are in all.
pub async fn list(
    db: &PgPool,
    actor: &Actor,
    filter: &Filter,
    sort: &Sort,
    limit: i64,
    offset: i64,
) -> sqlx::Result<(Vec<FlightSummary>, i64)> {
    let scope = policy::flight_scope(actor);
    let flights = db::list(db, &scope, filter, sort, limit, offset).await?;
    let total = db::count(db, &scope, filter).await?;
    Ok((flights, total))
}

/// The ids of all flights the actor may see that pass the filter.
pub async fn ids(db: &PgPool, actor: &Actor, filter: &Filter) -> ApiResult<Vec<Uuid>> {
    let scope = policy::flight_scope(actor);
    Ok(db::ids(db, &scope, filter).await?)
}

/// A flight's neighbours in the list the actor pages through with this filter
/// and order. Not found if the flight isn't in that list.
pub async fn neighbours(
    db: &PgPool,
    actor: &Actor,
    filter: &Filter,
    sort: &Sort,
    id: Uuid,
) -> ApiResult<Neighbours> {
    let scope = policy::flight_scope(actor);
    db::neighbours(db, &scope, filter, sort, id)
        .await?
        .ok_or(ApiError::NotFound)
}

/// The drones and controllers of the flights the actor may see, for the
/// logbook's filters.
#[derive(Serialize)]
pub struct Devices {
    pub aircraft: Vec<Device>,
    pub controllers: Vec<Device>,
    pub teams: Vec<TeamOption>,
    pub pilots: Vec<PilotOption>,
    pub areas: Vec<AreaOption>,
}

pub async fn devices(db: &PgPool, actor: &Actor, filter: &Filter) -> ApiResult<Devices> {
    let scope = policy::flight_scope(actor);
    Ok(Devices {
        aircraft: db::aircraft_of(db, &scope, filter).await?,
        controllers: db::controllers_of(db, &scope, filter).await?,
        teams: db::teams_in(db, &scope, filter).await?,
        pilots: db::pilots_in(db, &scope, filter).await?,
        areas: db::areas_in(db, &scope, filter).await?,
    })
}

/// A flight the actor may see. One they may not see is not found, so its
/// existence isn't revealed.
pub async fn get(db: &PgPool, actor: &Actor, id: Uuid) -> ApiResult<FlightDetail> {
    let scope = policy::flight_scope(actor);
    let mut flight = db::get(db, &scope, id).await?.ok_or(ApiError::NotFound)?;
    link_equipment(db, actor, std::slice::from_mut(&mut flight)).await?;
    Ok(flight)
}

/// Keeps the links to the drone's and controller's pages only where the
/// actor may see them.
pub async fn link_equipment(
    db: &PgPool,
    actor: &Actor,
    flights: &mut [FlightDetail],
) -> sqlx::Result<()> {
    let own_teams = db::own_teams(db, actor.user_id).await?;
    for flight in flights {
        if !policy::see_equipment(actor.role, &own_teams, flight.aircraft_team_id) {
            flight.aircraft_id = None;
        }
        if !policy::see_equipment(actor.role, &own_teams, flight.rc_team_id) {
            flight.rc_id = None;
        }
    }
    Ok(())
}

/// What the actor can change in the logbook, so it offers only that.
#[derive(Serialize)]
pub struct Choices {
    /// The teams whose flights a member can change; none for admins and viewers.
    pub own_team_ids: Vec<Uuid>,
    /// Admins change any flight and can move flights between teams.
    pub admin: bool,
    /// A pilot in several teams is a choice per team: the flights move to it.
    pub pilots: Vec<PilotChoice>,
    /// Teams to move flights to, for admins.
    pub teams: Vec<TeamChoice>,
}

pub async fn choices(db: &PgPool, actor: &Actor) -> ApiResult<Choices> {
    Ok(match actor.role {
        Role::Admin => Choices {
            own_team_ids: Vec::new(),
            admin: true,
            pilots: db::pilot_choices(db, None).await?,
            teams: db::team_choices(db).await?,
        },
        Role::Member => {
            let teams = db::own_teams(db, actor.user_id).await?;
            if teams.is_empty() {
                return Ok(Choices::none());
            }
            Choices {
                pilots: db::pilot_choices(db, Some(&teams)).await?,
                own_team_ids: teams,
                admin: false,
                teams: Vec::new(),
            }
        }
        Role::AllFlights => Choices::none(),
    })
}

impl Choices {
    fn none() -> Self {
        Self {
            own_team_ids: Vec::new(),
            admin: false,
            pilots: Vec::new(),
            teams: Vec::new(),
        }
    }
}

/// Sets the pilot of a batch of flights, all or none, chosen with one of
/// their teams; without one, a pilot in a single team brings that team.
/// Returns how many changed.
pub async fn set_pilot(
    db: &PgPool,
    actor: &Actor,
    ids: &[Uuid],
    pilot_id: Option<Uuid>,
    team_id: Option<Uuid>,
) -> ApiResult<u64> {
    let ids = distinct(ids)?;
    let flight_teams = db::teams_of(db, &ids).await?.ok_or(ApiError::NotFound)?;
    let new_pilot_team = match pilot_id {
        Some(pilot) => {
            let teams = db::pilot_teams(db, pilot).await?;
            Some(match (team_id, teams.as_slice()) {
                (_, []) => return Err(ApiError::Invalid("Choose an active pilot.".to_owned())),
                (Some(team), teams) if teams.contains(&team) => team,
                (Some(_), _) => {
                    return Err(ApiError::Invalid(
                        "The pilot isn’t in that team.".to_owned(),
                    ));
                }
                (None, [team]) => *team,
                (None, _) => {
                    return Err(ApiError::Invalid(
                        "Choose the team the pilot flew for.".to_owned(),
                    ));
                }
            })
        }
        None => None,
    };
    let own_teams = db::own_teams(db, actor.user_id).await?;
    let team = match policy::set_pilot(actor.role, &own_teams, &flight_teams, new_pilot_team)? {
        TeamChange::Keep => None,
        TeamChange::Follow(team) => Some(team),
    };
    Ok(db::set_pilot(db, &ids, pilot_id, team).await?)
}

/// Moves a batch of flights to a team, or to the queue with None. Admins only.
pub async fn set_team(
    db: &PgPool,
    actor: &Actor,
    ids: &[Uuid],
    team_id: Option<Uuid>,
) -> ApiResult<u64> {
    policy::require_admin(actor)?;
    let ids = distinct(ids)?;
    if db::teams_of(db, &ids).await?.is_none() {
        return Err(ApiError::NotFound);
    }
    if let Some(team) = team_id
        && !db::team_is_active(db, team).await?
    {
        return Err(ApiError::Invalid("Choose an active team.".to_owned()));
    }
    Ok(db::set_team(db, &ids, team_id).await?)
}

/// Archives a batch of flights, all or none: they leave their sessions and
/// every list and total. Returns how many weren't archived yet.
pub async fn archive(db: &PgPool, actor: &Actor, ids: &[Uuid]) -> ApiResult<u64> {
    let ids = changeable(db, actor, ids).await?;
    Ok(db::archive(db, &ids).await?)
}

/// Restores a batch of archived flights, without a session. Returns how many
/// were archived.
pub async fn restore(db: &PgPool, actor: &Actor, ids: &[Uuid]) -> ApiResult<u64> {
    let ids = changeable(db, actor, ids).await?;
    Ok(db::restore(db, &ids).await?)
}

/// These flights, once, if they all exist and the actor may change them.
async fn changeable(db: &PgPool, actor: &Actor, ids: &[Uuid]) -> ApiResult<Vec<Uuid>> {
    let ids = distinct(ids)?;
    let flight_teams = db::teams_of(db, &ids).await?.ok_or(ApiError::NotFound)?;
    let own_teams = db::own_teams(db, actor.user_id).await?;
    policy::change_flights(actor.role, &own_teams, &flight_teams)?;
    Ok(ids)
}

fn distinct(ids: &[Uuid]) -> ApiResult<Vec<Uuid>> {
    let mut ids = ids.to_vec();
    ids.sort_unstable();
    ids.dedup();
    if ids.is_empty() {
        return Err(ApiError::Invalid("Select one or more flights.".to_owned()));
    }
    Ok(ids)
}
