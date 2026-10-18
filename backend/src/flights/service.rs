use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use super::db::{self, FlightDetail, FlightSummary, PilotChoice, Sort, TeamChoice};
use crate::{
    auth::Actor,
    error::{ApiError, ApiResult},
    policy::{self, Role, TeamChange},
    sessions::{self, SessionGap},
};

/// One page of the flights the actor may see, and how many there are in all.
pub async fn list(
    db: &PgPool,
    actor: &Actor,
    sort: &Sort,
    limit: i64,
    offset: i64,
) -> sqlx::Result<(Vec<FlightSummary>, i64)> {
    let scope = policy::flight_scope(actor);
    let flights = db::list(db, &scope, sort, limit, offset).await?;
    let total = db::count(db, &scope).await?;
    Ok((flights, total))
}

/// A flight the actor may see. One they may not see is not found, so its
/// existence isn't revealed.
pub async fn get(db: &PgPool, actor: &Actor, id: Uuid) -> ApiResult<FlightDetail> {
    let scope = policy::flight_scope(actor);
    db::get(db, &scope, id).await?.ok_or(ApiError::NotFound)
}

/// What the actor can change in the logbook, so it offers only that.
#[derive(Serialize)]
pub struct Choices {
    /// The team whose flights a member can change; None for admins and viewers.
    pub own_team_id: Option<Uuid>,
    /// Admins change any flight and can move flights between teams.
    pub admin: bool,
    pub pilots: Vec<PilotChoice>,
    /// Teams to move flights to, for admins.
    pub teams: Vec<TeamChoice>,
}

pub async fn choices(db: &PgPool, actor: &Actor) -> ApiResult<Choices> {
    Ok(match actor.role {
        Role::Admin => Choices {
            own_team_id: None,
            admin: true,
            pilots: db::pilot_choices(db, None).await?,
            teams: db::team_choices(db).await?,
        },
        Role::Member => match db::own_team(db, actor.user_id).await? {
            Some(team) => Choices {
                own_team_id: Some(team),
                admin: false,
                pilots: db::pilot_choices(db, Some(team)).await?,
                teams: Vec::new(),
            },
            None => Choices::none(),
        },
        Role::AllFlights => Choices::none(),
    })
}

impl Choices {
    fn none() -> Self {
        Self {
            own_team_id: None,
            admin: false,
            pilots: Vec::new(),
            teams: Vec::new(),
        }
    }
}

/// Sets the pilot of a batch of flights, all or none. Returns how many changed.
pub async fn set_pilot(
    db: &PgPool,
    gap: SessionGap,
    actor: &Actor,
    ids: &[Uuid],
    pilot_id: Option<Uuid>,
) -> ApiResult<u64> {
    let ids = distinct(ids)?;
    let flight_teams = db::teams_of(db, &ids).await?.ok_or(ApiError::NotFound)?;
    let new_pilot_team = match pilot_id {
        Some(pilot) => Some(
            db::pilot_team(db, pilot)
                .await?
                .ok_or_else(|| ApiError::Invalid("Choose an active pilot.".to_owned()))?,
        ),
        None => None,
    };
    let own_team = db::own_team(db, actor.user_id).await?;
    let team = match policy::set_pilot(actor.role, own_team, &flight_teams, new_pilot_team)? {
        TeamChange::Keep => None,
        TeamChange::Follow(team) => Some(team),
    };
    let changed = db::set_pilot(db, &ids, pilot_id, team).await?;
    sessions::db::group(db, gap).await?;
    Ok(changed)
}

/// Moves a batch of flights to a team, or to the queue with None. Admins only.
pub async fn set_team(
    db: &PgPool,
    gap: SessionGap,
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
    let changed = db::set_team(db, &ids, team_id).await?;
    sessions::db::group(db, gap).await?;
    Ok(changed)
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
