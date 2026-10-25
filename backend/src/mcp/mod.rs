//! MCP server: AI assistants read summaries of flights and sessions, as the
//! user they act for, under the same access rules as the web app. Read-only,
//! without tracks or locations, and only when the install switches it on. See
//! "Clients" in docs/architecture.md.

pub mod db;

use std::sync::Arc;

use axum::{
    Router,
    extract::{Request, State},
    http::{HeaderValue, StatusCode, header, request::Parts},
    middleware::{self, Next},
    response::{IntoResponse, Response},
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
    oauth::{self, Bearer},
    policy, sessions,
};

const DEFAULT_PAGE_SIZE: u32 = 50;
const MAX_PAGE_SIZE: u32 = 100;

/// The scope an app needs for these tools.
const SCOPE: &str = "flights:read";

/// Who may use `/mcp`, checked before the tools run.
#[derive(Clone)]
struct Gate {
    db: PgPool,
    /// This resource's address, which access tokens must be for.
    resource: String,
    /// Where apps learn how to get such a token.
    metadata: String,
}

/// The address of `/mcp` at an install, which its tokens are for.
pub fn resource(issuer: &str) -> String {
    format!("{issuer}/mcp")
}

/// `/mcp`, for apps with an access token for it from `issuer` (the install's
/// address). `hosts` are the names the server is reached by, which the
/// transport checks against DNS rebinding.
pub fn router<S>(db: PgPool, time_zone: String, issuer: &str, hosts: Vec<String>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
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
        .layer(middleware::from_fn_with_state(
            Gate {
                db,
                resource: resource(issuer),
                metadata: format!("{issuer}/.well-known/oauth-protected-resource/mcp"),
            },
            authenticate,
        ))
}

/// Lets through apps with an access token for `/mcp` and `flights:read`, and
/// hands the tools who they act for. Others get a 401 that says where to get
/// a token (RFC 9728), or a 403 for a token without the scope (RFC 6750).
/// Debug builds also accept a login session, for trying the tools without
/// OAuth; release builds only access tokens. The tools have the user's own
/// access, read only: an admin's assistant sees what the admin sees.
async fn authenticate(State(gate): State<Gate>, request: Request, next: Next) -> Response {
    let (parts, body) = request.into_parts();
    let bearer = match oauth::bearer(&gate.db, &parts, &gate.resource, Some(SCOPE)).await {
        Ok(bearer) => bearer,
        Err(error) => {
            tracing::error!(%error, "checking an MCP access token");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let refusal = match bearer {
        Bearer::Valid(actor, _) => Ok(*actor),
        Bearer::Missing => Err((StatusCode::UNAUTHORIZED, None)),
        Bearer::Invalid => Err((StatusCode::UNAUTHORIZED, Some("invalid_token"))),
        Bearer::Insufficient => Err((StatusCode::FORBIDDEN, Some("insufficient_scope"))),
    };
    #[cfg(debug_assertions)]
    let refusal = match refusal {
        Err((StatusCode::UNAUTHORIZED, error)) => {
            use axum::extract::FromRequestParts;
            Actor::from_request_parts(&mut parts.clone(), &gate.db)
                .await
                .map_err(|_| (StatusCode::UNAUTHORIZED, error))
        }
        other => other,
    };
    let actor = match refusal {
        Ok(actor) => actor,
        Err((status, error)) => return challenge(&gate.metadata, status, error),
    };
    let mut request = Request::from_parts(parts, body);
    request.extensions_mut().insert(actor);
    next.run(request).await
}

/// A refusal with the `WWW-Authenticate` header: where to get a token, which
/// scope it needs, and what was wrong with the one sent, if any.
fn challenge(metadata: &str, status: StatusCode, error: Option<&str>) -> Response {
    let mut value = format!(r#"Bearer resource_metadata="{metadata}", scope="{SCOPE}""#);
    if let Some(error) = error {
        value.push_str(&format!(r#", error="{error}""#));
    }
    let mut response = status.into_response();
    if let Ok(value) = HeaderValue::from_str(&value) {
        response
            .headers_mut()
            .insert(header::WWW_AUTHENTICATE, value);
    }
    response
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
    /// Group ids from `directory`; "none" for flights in no group (the
    /// organisation's queue).
    groups: Option<Vec<String>>,
    /// Pilots' ids from `directory`; "none" for flights without a pilot.
    pilots: Option<Vec<String>>,
    /// Drone serials from `directory`.
    drones: Option<Vec<String>>,
    /// Controller serials from `directory`.
    controllers: Option<Vec<String>>,
    /// Area ids from `directory`: flights with time in one of them.
    areas: Option<Vec<String>>,
}

#[derive(Deserialize, JsonSchema)]
struct TotalsParams {
    #[serde(flatten)]
    filters: Filters,
    /// What to total per: nothing (one row), group, pilot, drone,
    /// controller, day, week, month or area. Per area, each flight counts
    /// with only its time and distance in that area.
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
    group_id: Option<Uuid>,
    group_name: Option<String>,
    drone_serial: Option<String>,
    drone_name: Option<String>,
    controller_serial: Option<String>,
    controller_name: Option<String>,
    /// The highest point above takeoff, the battery's and camera's serials.
    metrics: Value,
    /// Seconds and metres in each area the flight passed through; the rest
    /// was outside all areas.
    areas: Vec<AreaFacts>,
}

/// Time and distance in one area.
#[derive(Serialize)]
struct AreaFacts {
    id: Uuid,
    name: String,
    air_time_s: i64,
    distance_m: f64,
}

impl AreaFacts {
    /// From a flight's `areas`; distances in whole metres, as everywhere in these tools.
    fn of(areas: &Value) -> Vec<Self> {
        #[derive(Deserialize)]
        struct Stored {
            id: Uuid,
            name: String,
            air_time_s: i64,
            distance_m: f64,
        }
        Vec::<Stored>::deserialize(areas)
            .unwrap_or_default()
            .into_iter()
            .map(|area| Self {
                id: area.id,
                name: area.name,
                air_time_s: area.air_time_s,
                distance_m: area.distance_m.round(),
            })
            .collect()
    }

    /// The totals per area of several flights, most time first.
    fn total(flights: &[FlightFacts]) -> Vec<Self> {
        let mut totals: Vec<Self> = Vec::new();
        for area in flights.iter().flat_map(|flight| &flight.areas) {
            match totals.iter_mut().find(|total| total.id == area.id) {
                Some(total) => {
                    total.air_time_s += area.air_time_s;
                    total.distance_m += area.distance_m;
                }
                None => totals.push(Self {
                    id: area.id,
                    name: area.name.clone(),
                    air_time_s: area.air_time_s,
                    distance_m: area.distance_m,
                }),
            }
        }
        totals.sort_by(|a, b| b.air_time_s.cmp(&a.air_time_s).then(a.name.cmp(&b.name)));
        totals
    }
}

impl From<FlightDetail> for FlightFacts {
    fn from(flight: FlightDetail) -> Self {
        Self {
            id: flight.id,
            takeoff_at: flight.takeoff_at,
            landed_at: flight.landed_at,
            air_time_s: flight.air_time_s,
            distance_m: flight.distance_m.round(),
            pilot_id: flight.pilot_id,
            pilot_name: flight.pilot_name,
            group_id: flight.group_id,
            group_name: flight.group_name,
            drone_serial: flight.aircraft_serial,
            drone_name: flight.aircraft_name,
            controller_serial: flight.rc_serial,
            controller_name: flight.rc_name,
            metrics: flight.metrics,
            areas: AreaFacts::of(&flight.areas),
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
        description = "Start here. Who the user is (`you`: their id, name and the groups they fly for, for questions about \"my flights\" or \"my group\"), the groups, pilots, drones, controllers and areas of the flights the user can see, with the ids and serials the other tools filter on, and the organisation's time zone and today's date in it."
    )]
    async fn directory(
        &self,
        Extension(parts): Extension<Parts>,
    ) -> Result<CallToolResult, McpError> {
        let actor = actor(&parts)?;
        let scope = policy::flight_scope(actor);
        let filter = Filter::default();
        let today = db::today(&self.db, &self.time_zone)
            .await
            .map_err(internal)?;
        let directory = serde_json::json!({
            "you": {
                "id": actor.user_id,
                "name": actor.name,
                "groups": db::own_groups(&self.db, actor.user_id).await.map_err(internal)?,
            },
            "time_zone": &*self.time_zone,
            "today": today,
            "groups": flights::db::groups_in(&self.db, &scope, &filter).await.map_err(internal)?,
            "pilots": flights::db::pilots_in(&self.db, &scope, &filter).await.map_err(internal)?,
            "drones": flights::db::aircraft_of(&self.db, &scope, &filter).await.map_err(internal)?,
            "controllers": flights::db::controllers_of(&self.db, &scope, &filter).await.map_err(internal)?,
            "areas": flights::db::areas_in(&self.db, &scope, &filter).await.map_err(internal)?,
        });
        json(directory)
    }

    #[tool(
        description = "Totals of the flights the user can see: number of flights and sessions, air time in seconds, distance in metres, first takeoff and last landing. For a period and filters, in one row or per group, pilot, drone, controller, day, week, month or area, e.g. air time per pilot in a month. Archived flights don't count."
    )]
    async fn flight_totals(
        &self,
        Extension(parts): Extension<Parts>,
        Parameters(params): Parameters<TotalsParams>,
    ) -> Result<CallToolResult, McpError> {
        let scope = policy::flight_scope(actor(&parts)?);
        let filter = self.filter(&params.filters).await?;
        let totals = match params.group_by.unwrap_or(GroupBy::None) {
            GroupBy::Area => db::area_totals(&self.db, &scope, &filter).await,
            group_by => db::totals(&self.db, &scope, &filter, group_by, &self.time_zone).await,
        };
        json(totals.map_err(internal)?)
    }

    #[tool(
        description = "The sessions the user can see (flights flown together, reported on as one), newest first: start, end, number of flights, air time, distance, pilots and the session's own group, which its flights needn't share. A session is listed when one of its flights passes the filters; the groups filter is on the session's own group."
    )]
    async fn list_sessions(
        &self,
        Extension(parts): Extension<Parts>,
        Parameters(params): Parameters<ListParams>,
    ) -> Result<CallToolResult, McpError> {
        let actor = actor(&parts)?;
        let filter = self.filter(&params.filters).await?;
        let (limit, offset) = page(params.page, params.size)?;
        let (mut sessions, total) =
            sessions::service::list(&self.db, actor, &filter, None, limit, offset, false)
                .await
                .map_err(api)?;
        // Whole metres, as everywhere in these tools; the sum is of the exact
        // distances. Without the day strip's data: apps get no takeoff points.
        for session in &mut sessions {
            session.distance_m = session.distance_m.round();
            session.flight_times = serde_json::Value::Null;
        }
        json(serde_json::json!({ "sessions": sessions, "total": total }))
    }

    #[tool(
        description = "One session with its flights, in takeoff order, its time and distance per area, and its comments, oldest first: what people recorded about what happened."
    )]
    async fn get_session(
        &self,
        Extension(parts): Extension<Parts>,
        Parameters(params): Parameters<IdParams>,
    ) -> Result<CallToolResult, McpError> {
        let session = sessions::service::get(&self.db, actor(&parts)?, uuid(&params.id)?)
            .await
            .map_err(api)?;
        let flights: Vec<FlightFacts> = session.flights.into_iter().map(Into::into).collect();
        let areas = AreaFacts::total(&flights);
        let comments: Vec<_> = session
            .comments
            .into_iter()
            .map(|comment| {
                serde_json::json!({
                    "author": comment.author_name,
                    "written_at": comment.created_at,
                    "edited_at": comment.edited_at,
                    "text": comment.body,
                })
            })
            .collect();
        json(serde_json::json!({
            "id": session.id,
            "flights": flights,
            "areas": areas,
            "comments": comments,
        }))
    }

    #[tool(
        description = "The flights the user can see, newest first: takeoff, landing, air time, distance, pilot, group, drone and controller. Also the flights still without a pilot or without a session."
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
        let (mut flights, total) =
            flights::service::list(&self.db, actor, &filter, &sort, limit, offset)
                .await
                .map_err(internal)?;
        for flight in &mut flights {
            flight.distance_m = flight.distance_m.round();
        }
        json(serde_json::json!({ "flights": flights, "total": total }))
    }

    #[tool(
        description = "One flight's facts: times, air time, distance, highest point above takeoff, pilot, group, the serials of drone, controller, battery and camera, and its time and distance per area."
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
        description = "The drones and controllers the user can see, with group, number of flights, air time in seconds, last flight and who flew it, counted over all their flights; controllers also with whether they're paired, when they last synced and their app version."
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
        let (group_ids, queue) = ids(filters.groups.as_deref(), "groups")?;
        let (pilot_ids, unknown_pilot) = ids(filters.pilots.as_deref(), "pilots")?;
        let (area_ids, _) = ids(filters.areas.as_deref(), "areas")?;
        Ok(Filter {
            aircraft_serials: filters.drones.clone(),
            rc_serials: filters.controllers.clone(),
            in_session: None,
            from,
            to,
            group_ids,
            queue,
            pilot_ids,
            unknown_pilot,
            archived: false,
            area_ids,
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
                 the flights and sessions the user can see in FleetRanger. Call `directory` first \
                 for who the user is, ids, the time zone and today's date. Air time is in seconds, \
                 distance in whole metres, times in UTC.",
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
        ApiError::Forbidden => McpError::invalid_request("Not allowed for this user.", None),
        ApiError::Internal(error) => {
            tracing::error!(error = format!("{error:#}"), "MCP tool failed");
            McpError::internal_error("Something went wrong in FleetRanger.", None)
        }
    }
}
