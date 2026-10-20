//! MCP server: AI assistants read summaries of flights and sessions, as the
//! user they act for, under the same access rules as the web app. Read-only,
//! without tracks or locations, and only when the install switches it on. See
//! "Clients" in docs/architecture.md.

pub mod db;

use std::sync::Arc;

use axum::{
    Router,
    extract::{FromRef, Request},
    http::request::Parts,
    middleware::{self, Next},
    response::Response,
};
use chrono::{DateTime, NaiveDate, Utc};
use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::Extension, wrapper::Parameters},
    model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerConfig},
    schemars::{self, JsonSchema},
    tool, tool_handler, tool_router,
    transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::never::NeverSessionManager,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use self::db::GroupBy;
use crate::{
    auth::Actor,
    equipment::{self, db::Kind},
    error::ApiError,
    flights::{
        self,
        db::{Filter, FlightDetail, Sort, SortKey},
    },
    policy, sessions,
};

const DEFAULT_PAGE_SIZE: u32 = 50;
const MAX_PAGE_SIZE: u32 = 100;

/// `/mcp`, for logged-in users only. `hosts` are the names the server is
/// reached by, which the transport checks against DNS rebinding.
pub fn router<S>(db: PgPool, time_zone: String, hosts: Vec<String>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    PgPool: FromRef<S>,
{
    let config = StreamableHttpServerConfig::default()
        .with_legacy_session_mode(false)
        .with_json_response(true)
        .with_allowed_hosts(hosts);
    let time_zone: Arc<str> = time_zone.into();
    let tools_db = db.clone();
    let service = StreamableHttpService::new(
        move || Ok(FleetRanger::new(tools_db.clone(), time_zone.clone())),
        Arc::new(NeverSessionManager::default()),
        config,
    );
    Router::new()
        .route_service("/mcp", service)
        .layer(middleware::from_fn_with_state(db, authenticate))
}

/// Lets only logged-in users through, and hands the tools who they are.
async fn authenticate(actor: Actor, mut request: Request, next: Next) -> Response {
    request.extensions_mut().insert(actor);
    next.run(request).await
}

#[derive(Clone)]
struct FleetRanger {
    db: PgPool,
    time_zone: Arc<str>,
    tool_router: ToolRouter<Self>,
}

/// The filters the list and totals tools share.
#[derive(Deserialize, JsonSchema, Default)]
struct Filters {
    /// The first day, as YYYY-MM-DD in the organisation's time zone.
    from: Option<String>,
    /// The last day, included; without it, only `from`.
    to: Option<String>,
    /// Team ids from `directory`; "none" for flights without a team.
    teams: Option<Vec<String>>,
    /// Pilots' ids from `directory`; "none" for flights without a pilot.
    pilots: Option<Vec<String>>,
    /// Drone serials from `directory`.
    drones: Option<Vec<String>>,
    /// Controller serials from `directory`.
    controllers: Option<Vec<String>>,
}

#[derive(Deserialize, JsonSchema)]
struct TotalsParams {
    #[serde(flatten)]
    filters: Filters,
    /// What to total per: nothing (one row), team, pilot, drone,
    /// controller, day, week or month.
    group_by: Option<GroupBy>,
}

#[derive(Deserialize, JsonSchema)]
struct ListParams {
    #[serde(flatten)]
    filters: Filters,
    /// From 1.
    page: Option<u32>,
    /// At most 100; 50 by default.
    size: Option<u32>,
}

#[derive(Deserialize, JsonSchema)]
struct FlightListParams {
    #[serde(flatten)]
    filters: Filters,
    /// Only flights without a pilot.
    without_pilot: Option<bool>,
    /// Only flights that aren't in a session yet.
    without_session: Option<bool>,
    /// Oldest first instead of newest first.
    oldest_first: Option<bool>,
    /// From 1.
    page: Option<u32>,
    /// At most 100; 50 by default.
    size: Option<u32>,
}

#[derive(Deserialize, JsonSchema)]
struct IdParams {
    id: String,
}

/// A flight's facts, without its track.
#[derive(Serialize)]
struct FlightFacts {
    id: Uuid,
    takeoff_at: DateTime<Utc>,
    landed_at: DateTime<Utc>,
    air_time_s: i32,
    distance_m: f64,
    pilot_id: Option<Uuid>,
    pilot_name: Option<String>,
    team_id: Option<Uuid>,
    team_name: Option<String>,
    drone_serial: Option<String>,
    drone_name: Option<String>,
    controller_serial: Option<String>,
    controller_name: Option<String>,
    /// The highest point above takeoff, the battery's and camera's serials.
    metrics: Value,
}

impl From<FlightDetail> for FlightFacts {
    fn from(flight: FlightDetail) -> Self {
        Self {
            id: flight.id,
            takeoff_at: flight.takeoff_at,
            landed_at: flight.landed_at,
            air_time_s: flight.air_time_s,
            distance_m: flight.distance_m,
            pilot_id: flight.pilot_id,
            pilot_name: flight.pilot_name,
            team_id: flight.team_id,
            team_name: flight.team_name,
            drone_serial: flight.aircraft_serial,
            drone_name: flight.aircraft_name,
            controller_serial: flight.rc_serial,
            controller_name: flight.rc_name,
            metrics: flight.metrics,
        }
    }
}

#[tool_router]
impl FleetRanger {
    fn new(db: PgPool, time_zone: Arc<str>) -> Self {
        Self {
            db,
            time_zone,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "Start here. The teams, pilots, drones and controllers whose flights the user can see, with the ids and serials the other tools filter on, and the organisation's time zone and today's date in it."
    )]
    async fn directory(
        &self,
        Extension(parts): Extension<Parts>,
    ) -> Result<CallToolResult, McpError> {
        let scope = policy::flight_scope(actor(&parts)?);
        let filter = Filter::default();
        let today = db::today(&self.db, &self.time_zone)
            .await
            .map_err(internal)?;
        let directory = serde_json::json!({
            "time_zone": &*self.time_zone,
            "today": today,
            "teams": flights::db::teams_in(&self.db, &scope, &filter).await.map_err(internal)?,
            "pilots": flights::db::pilots_in(&self.db, &scope, &filter).await.map_err(internal)?,
            "drones": flights::db::aircraft_of(&self.db, &scope, &filter).await.map_err(internal)?,
            "controllers": flights::db::controllers_of(&self.db, &scope, &filter).await.map_err(internal)?,
        });
        json(directory)
    }

    #[tool(
        description = "Totals of the flights the user can see: number of flights and sessions, air time in seconds, distance in metres, first takeoff and last landing. For a period and filters, in one row or per team, pilot, drone, controller, day, week or month, e.g. air time per pilot in a month. Archived flights don't count."
    )]
    async fn flight_totals(
        &self,
        Extension(parts): Extension<Parts>,
        Parameters(params): Parameters<TotalsParams>,
    ) -> Result<CallToolResult, McpError> {
        let scope = policy::flight_scope(actor(&parts)?);
        let filter = self.filter(&params.filters).await?;
        let group_by = params.group_by.unwrap_or(GroupBy::None);
        json(
            db::totals(&self.db, &scope, &filter, group_by, &self.time_zone)
                .await
                .map_err(internal)?,
        )
    }

    #[tool(
        description = "The sessions the user can see (groups of flights flown together), newest first: start, end, number of flights, air time, distance, pilots and teams. A session is listed when one of its flights passes the filters."
    )]
    async fn list_sessions(
        &self,
        Extension(parts): Extension<Parts>,
        Parameters(params): Parameters<ListParams>,
    ) -> Result<CallToolResult, McpError> {
        let actor = actor(&parts)?;
        let filter = self.filter(&params.filters).await?;
        let (limit, offset) = page(params.page, params.size)?;
        let (sessions, total) =
            sessions::service::list(&self.db, actor, &filter, None, limit, offset)
                .await
                .map_err(api)?;
        json(serde_json::json!({ "sessions": sessions, "total": total }))
    }

    #[tool(description = "One session with its flights, in takeoff order.")]
    async fn get_session(
        &self,
        Extension(parts): Extension<Parts>,
        Parameters(params): Parameters<IdParams>,
    ) -> Result<CallToolResult, McpError> {
        let session = sessions::service::get(&self.db, actor(&parts)?, uuid(&params.id)?)
            .await
            .map_err(api)?;
        let flights: Vec<FlightFacts> = session.flights.into_iter().map(Into::into).collect();
        json(serde_json::json!({ "id": session.id, "flights": flights }))
    }

    #[tool(
        description = "The flights the user can see, newest first: takeoff, landing, air time, distance, pilot, team, drone and controller. Also the flights still without a pilot or without a session."
    )]
    async fn list_flights(
        &self,
        Extension(parts): Extension<Parts>,
        Parameters(params): Parameters<FlightListParams>,
    ) -> Result<CallToolResult, McpError> {
        let actor = actor(&parts)?;
        let mut filter = self.filter(&params.filters).await?;
        if params.without_pilot == Some(true) {
            filter.pilot_ids = Some(Vec::new());
            filter.unknown_pilot = true;
        }
        if params.without_session == Some(true) {
            filter.in_session = Some(false);
        }
        let sort = Sort {
            key: SortKey::TakeoffAt,
            descending: params.oldest_first != Some(true),
        };
        let (limit, offset) = page(params.page, params.size)?;
        let (flights, total) =
            flights::service::list(&self.db, actor, &filter, &sort, limit, offset)
                .await
                .map_err(internal)?;
        json(serde_json::json!({ "flights": flights, "total": total }))
    }

    #[tool(
        description = "One flight's facts: times, air time, distance, highest point above takeoff, pilot, team and the serials of drone, controller, battery and camera."
    )]
    async fn get_flight(
        &self,
        Extension(parts): Extension<Parts>,
        Parameters(params): Parameters<IdParams>,
    ) -> Result<CallToolResult, McpError> {
        let flight = flights::service::get(&self.db, actor(&parts)?, uuid(&params.id)?)
            .await
            .map_err(api)?;
        json(FlightFacts::from(flight))
    }

    #[tool(
        description = "The drones and controllers the user can see, with team, holder and last flight; controllers also with whether they're paired, when they last synced and their app version."
    )]
    async fn list_equipment(
        &self,
        Extension(parts): Extension<Parts>,
    ) -> Result<CallToolResult, McpError> {
        let actor = actor(&parts)?;
        let drones = equipment::service::list(&self.db, actor, Kind::Aircraft)
            .await
            .map_err(api)?;
        let controllers = equipment::service::list(&self.db, actor, Kind::RemoteController)
            .await
            .map_err(api)?;
        json(serde_json::json!({ "drones": drones, "controllers": controllers }))
    }
}

impl FleetRanger {
    /// The flights' filter from the tools' filters; days in the
    /// organisation's time zone.
    async fn filter(&self, filters: &Filters) -> Result<Filter, McpError> {
        let day = |value: &str| {
            NaiveDate::parse_from_str(value, "%Y-%m-%d")
                .map_err(|_| invalid(format!("{value} isn't a day as YYYY-MM-DD")))
        };
        let (from, to) = match (&filters.from, &filters.to) {
            (None, None) => (None, None),
            (Some(from), to) => {
                let first = day(from)?;
                let last = match to {
                    Some(to) => day(to)?,
                    None => first,
                };
                if last < first {
                    return Err(invalid("to is before from".to_owned()));
                }
                let start = db::start_of_day(&self.db, first, &self.time_zone);
                let end = db::start_of_day(&self.db, last + chrono::Days::new(1), &self.time_zone);
                (
                    Some(start.await.map_err(internal)?),
                    Some(end.await.map_err(internal)?),
                )
            }
            (None, Some(_)) => return Err(invalid("to needs from".to_owned())),
        };
        let (team_ids, queue) = ids(filters.teams.as_deref(), "teams")?;
        let (pilot_ids, unknown_pilot) = ids(filters.pilots.as_deref(), "pilots")?;
        Ok(Filter {
            aircraft_serials: filters.drones.clone(),
            rc_serials: filters.controllers.clone(),
            in_session: None,
            from,
            to,
            team_ids,
            queue,
            pilot_ids,
            unknown_pilot,
            archived: false,
        })
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for FleetRanger {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("FleetRanger", env!("CARGO_PKG_VERSION")))
            .with_instructions(
                "FleetRanger keeps an organisation's drone flight logs. These tools read summaries of \
                 the flights and sessions the user may see. Call `directory` first for ids, the time \
                 zone and today's date. Air time is in seconds, distance in metres, times in UTC.",
            )
    }
}

/// The logged-in user, put there by `authenticate`.
fn actor(parts: &Parts) -> Result<&Actor, McpError> {
    parts
        .extensions
        .get::<Actor>()
        .ok_or_else(|| McpError::internal_error("no user", None))
}

/// Ids, and whether "none" was among them; nothing if not given.
fn ids(values: Option<&[String]>, name: &str) -> Result<(Option<Vec<Uuid>>, bool), McpError> {
    let Some(values) = values else {
        return Ok((None, false));
    };
    let mut ids = Vec::new();
    let mut none = false;
    for value in values {
        if value == "none" {
            none = true;
        } else {
            ids.push(uuid(value).map_err(|_| invalid(format!("{name} must be ids or none")))?);
        }
    }
    Ok((Some(ids), none))
}

fn uuid(value: &str) -> Result<Uuid, McpError> {
    value
        .parse()
        .map_err(|_| invalid(format!("{value} isn't an id")))
}

/// The limit and offset of a page.
fn page(page: Option<u32>, size: Option<u32>) -> Result<(i64, i64), McpError> {
    let size = size.unwrap_or(DEFAULT_PAGE_SIZE);
    if !(1..=MAX_PAGE_SIZE).contains(&size) {
        return Err(invalid(format!(
            "size must be between 1 and {MAX_PAGE_SIZE}"
        )));
    }
    let page = page.unwrap_or(1).max(1);
    Ok((i64::from(size), i64::from(page - 1) * i64::from(size)))
}

fn json(value: impl Serialize) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![ContentBlock::json(value)?]))
}

fn invalid(message: String) -> McpError {
    McpError::invalid_params(message, None)
}

fn internal(error: sqlx::Error) -> McpError {
    api(error.into())
}

fn api(error: ApiError) -> McpError {
    match error {
        ApiError::NotFound => invalid("Not found, or not visible to this user.".to_owned()),
        ApiError::Invalid(message) | ApiError::Conflict(message) => invalid(message),
        ApiError::Unauthorized | ApiError::Forbidden => {
            McpError::invalid_request("Not allowed for this user.", None)
        }
        ApiError::Internal(error) => {
            tracing::error!(error = format!("{error:#}"), "MCP tool failed");
            McpError::internal_error("Something went wrong in FleetRanger.", None)
        }
    }
}
