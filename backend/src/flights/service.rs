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
    policy::{self, Permission},
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
    link_equipment(actor, std::slice::from_mut(&mut flight));
    Ok(flight)
}

/// Keeps the links to the drone's and controller's pages only where the
/// actor may see them.
pub fn link_equipment(actor: &Actor, flights: &mut [FlightDetail]) {
    let see = |group| actor.grants.may(Permission::EquipmentRead, group);
    for flight in flights {
        if !see(flight.aircraft_group_id) {
            flight.aircraft_id = None;
        }
        if !see(flight.rc_group_id) {
            flight.rc_id = None;
        }
    }
}

/// What the actor can change in the logbook, so it offers only that.
#[derive(Serialize)]
pub struct Choices {
    /// The groups whose flights the actor can change, when that isn't all of
    /// them.
    pub own_team_ids: Vec<Uuid>,
    /// Changes any flight and can move flights between groups.
    pub admin: bool,
    /// Everyone who flies, the group's own pilots first if a group was given;
    /// empty if the actor sets no pilots.
    pub pilots: Vec<PilotChoice>,
    /// Groups to move flights to.
    pub teams: Vec<TeamChoice>,
}

/// - `group`: the group of the flights whose pilot is chosen, if they share
///   one, so its own pilots come first.
pub async fn choices(db: &PgPool, actor: &Actor, group: Option<Uuid>) -> ApiResult<Choices> {
    let set_pilot = actor.grants.scope(Permission::FlightsSetPilot);
    let moves = actor.grants.may(Permission::FlightsMove, None);
    let pilots = if set_pilot.all || !set_pilot.groups.is_empty() {
        db::pilot_choices(db, group).await?
    } else {
        Vec::new()
    };
    Ok(Choices {
        own_team_ids: if set_pilot.all {
            Vec::new()
        } else {
            set_pilot.groups
        },
        admin: moves,
        pilots,
        teams: if moves {
            db::group_choices(db).await?
        } else {
            Vec::new()
        },
    })
}

/// Sets the pilot of a batch of flights, all or none: someone who flies
/// (anywhere), or unknown. The flights stay in their group. Returns how many
/// changed.
pub async fn set_pilot(
    db: &PgPool,
    actor: &Actor,
    ids: &[Uuid],
    pilot_id: Option<Uuid>,
) -> ApiResult<u64> {
    let ids = distinct(ids)?;
    let flight_groups = db::groups_of(db, &ids).await?.ok_or(ApiError::NotFound)?;
    policy::change_flights(&actor.grants, Permission::FlightsSetPilot, &flight_groups)?;
    if let Some(pilot) = pilot_id
        && !db::flies(db, pilot).await?
    {
        return Err(ApiError::Invalid("Choose an active pilot.".to_owned()));
    }
    Ok(db::set_pilot(db, &ids, pilot_id).await?)
}

/// Moves a batch of flights to a group, or to the queue with None. They keep
/// their pilot.
pub async fn set_team(
    db: &PgPool,
    actor: &Actor,
    ids: &[Uuid],
    group_id: Option<Uuid>,
) -> ApiResult<u64> {
    let ids = distinct(ids)?;
    // Refused before not found, so this doesn't tell whether flights exist.
    let flight_groups = db::groups_of(db, &ids).await?;
    policy::move_flights(
        &actor.grants,
        flight_groups.as_deref().unwrap_or_default(),
        group_id,
    )?;
    if flight_groups.is_none() {
        return Err(ApiError::NotFound);
    }
    if let Some(group) = group_id
        && !db::group_exists(db, group).await?
    {
        return Err(ApiError::Invalid("Choose an active team.".to_owned()));
    }
    Ok(db::set_team(db, &ids, group_id).await?)
}

/// Archives a batch of flights, all or none: they leave their sessions and
/// every list and total. Returns how many weren't archived yet.
pub async fn archive(db: &PgPool, actor: &Actor, ids: &[Uuid]) -> ApiResult<u64> {
    let ids = archivable(db, actor, ids).await?;
    Ok(db::archive(db, &ids).await?)
}

/// Restores a batch of archived flights, without a session. Returns how many
/// were archived.
pub async fn restore(db: &PgPool, actor: &Actor, ids: &[Uuid]) -> ApiResult<u64> {
    let ids = archivable(db, actor, ids).await?;
    Ok(db::restore(db, &ids).await?)
}

/// These flights, once, if they all exist and the actor may archive and
/// restore them.
async fn archivable(db: &PgPool, actor: &Actor, ids: &[Uuid]) -> ApiResult<Vec<Uuid>> {
    let ids = distinct(ids)?;
    let flight_groups = db::groups_of(db, &ids).await?.ok_or(ApiError::NotFound)?;
    policy::change_flights(&actor.grants, Permission::FlightsArchive, &flight_groups)?;
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
