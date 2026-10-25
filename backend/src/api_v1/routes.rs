//! The endpoints. Each answers its base shape, plus what `expand` asks for.
//! What the web app shows on a session or flight comes with it; only
//! expansions that reach another resource or level need a scope of their
//! own (see docs/api-v1.md).

use std::collections::HashMap;

use axum::{Extension, Json, extract::State};
use chrono::NaiveDate;
use serde::Deserialize;
use utoipa::IntoParams;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::{
    Api, Caller, Expand, IdCursor, Params, PathId, TimeCursor, V1Error,
    db::{self, After, FlightRow, InSession, SessionRow},
    decode, encode, limit,
    types::{
        Area, AreaFigures, Battery, Comment, Device, Error, Expandable, Flight, Group, Me,
        Organisation, Page, Person, Session,
    },
};
use crate::{
    batteries,
    equipment::{self, db::Kind},
    flights::db::Filter,
    mcp,
    policy::{self, FlightScope},
};

pub fn router() -> OpenApiRouter<Api> {
    OpenApiRouter::new()
        .routes(routes!(me))
        .routes(routes!(organisation))
        .routes(routes!(list_sessions))
        .routes(routes!(get_session))
        .routes(routes!(list_flights))
        .routes(routes!(get_flight))
        .routes(routes!(list_groups))
        .routes(routes!(list_areas))
        .routes(routes!(list_drones))
        .routes(routes!(list_controllers))
        .routes(routes!(list_batteries))
}

/// What a session's `expand` offers, and the scope each needs beyond the
/// session's own: whatever reaches another resource or level.
const SESSION_EXPANSIONS: &[(&str, Option<&str>)] = &[
    ("group", None),
    ("pilots", None),
    ("areas.area", None),
    ("comments", Some("sessions:read")),
    ("comments.author", Some("sessions:read")),
    ("flights", Some("flights:list")),
    ("flights.pilot", Some("flights:list")),
    ("flights.group", Some("flights:list")),
    ("flights.drone", Some("flights:list")),
    ("flights.controller", Some("flights:list")),
    ("flights.batteries", Some("flights:list")),
    ("flights.areas.area", Some("flights:list")),
];

const FLIGHT_EXPANSIONS: &[(&str, Option<&str>)] = &[
    ("pilot", None),
    ("group", None),
    ("session", Some("sessions:list")),
    ("drone", None),
    ("controller", None),
    ("batteries", None),
    ("areas.area", None),
];

const DEVICE_EXPANSIONS: &[(&str, Option<&str>)] = &[("group", None)];

/// Who the user is
///
/// The user the token acts for.
#[utoipa::path(
    get, path = "/api/v1/me", tag = "Me",
    responses((status = 200, body = Me), (status = 401, body = Error)),
    security(("oauth" = []))
)]
async fn me(Extension(caller): Extension<Caller>) -> Json<Me> {
    Json(Me {
        id: caller.actor.user_id,
        name: caller.actor.name,
    })
}

/// The organisation
///
/// What apps need of it: its name and time zone.
#[utoipa::path(
    get, path = "/api/v1/organisation", tag = "Organisation",
    responses((status = 200, body = Organisation), (status = 401, body = Error)),
    security(("oauth" = []))
)]
async fn organisation(State(api): State<Api>) -> Json<Organisation> {
    Json(Organisation {
        name: api.organisation_name.as_deref().map(str::to_owned),
        time_zone: api.time_zone.to_string(),
    })
}

/// The filters sessions and flights share. Repeat one for several values,
/// which combine with or; different filters combine with and.
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct Filters {
    /// The first day, as YYYY-MM-DD in the organisation's time zone.
    from: Option<String>,
    /// The last day, included; without it, only `from`.
    to: Option<String>,
    /// A group's id, or `none` for no group. For sessions, their own group.
    #[serde(default)]
    group: Vec<String>,
    /// A pilot's id, or `none` for flights without a pilot.
    #[serde(default)]
    pilot: Vec<String>,
    /// An area's id: flights with time in it, or sessions with such a flight.
    #[serde(default)]
    area: Vec<String>,
}

/// Paging, for every list.
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct Paging {
    /// How many at most: 1 to 100, 50 if not given.
    limit: Option<i64>,
    /// The `next_cursor` of the page before; not given for the first page.
    cursor: Option<String>,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct FlightFilters {
    /// A session's id, or `none` for flights in no session.
    session: Option<String>,
    /// A drone's id.
    #[serde(default)]
    drone: Vec<String>,
    /// A controller's id.
    #[serde(default)]
    controller: Vec<String>,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct SessionExpand {
    /// Fields to expand, comma-separated; a path expands what leads to it
    /// too.
    ///
    /// - `group`, `pilots`, `areas.area`: come with the session
    /// - `comments`: needs `sessions:read`
    /// - `comments.author`: each comment's author
    /// - `flights`: needs `flights:list`
    /// - a flight's expansions after `flights.`, such as `flights.batteries`
    ///   or `flights.areas.area`
    expand: Option<String>,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct FlightExpand {
    /// Fields to expand, comma-separated.
    ///
    /// - `pilot`, `group`, `drone`, `controller`, `batteries`,
    ///   `areas.area`: come with the flight
    /// - `session`: needs `sessions:list`
    expand: Option<String>,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct DeviceExpand {
    /// Fields to expand: `group`, which comes with the drone or controller.
    expand: Option<String>,
}

/// List sessions
///
/// Sessions with a flight the user sees, newest first. Filters decide which
/// sessions are listed, by their flights; a listed session shows the
/// figures of all the flights the user sees in it.
#[utoipa::path(
    get, path = "/api/v1/sessions", tag = "Sessions",
    params(Filters, SessionExpand, Paging),
    responses(
        (status = 200, body = Page<Session>), (status = 400, body = Error),
        (status = 401, body = Error), (status = 403, body = Error)
    ),
    security(("oauth" = ["sessions:list"]))
)]
async fn list_sessions(
    State(api): State<Api>,
    Extension(caller): Extension<Caller>,
    Params(filters): Params<Filters>,
    Params(expansion): Params<SessionExpand>,
    Params(paging): Params<Paging>,
) -> Result<Json<Page<Session>>, V1Error> {
    caller.require("sessions:list")?;
    let expand = Expand::parse(expansion.expand.as_deref(), SESSION_EXPANSIONS, &caller)?;
    let filter = filter(&api, &filters).await?;
    let limit = limit(paging.limit)?;
    let after = decode::<TimeCursor>(paging.cursor.as_deref())?;
    let scope = policy::flight_scope(&caller.actor);
    let mut rows = db::sessions(
        &api.db,
        &scope,
        &filter,
        None,
        after.map(|cursor| After {
            at: cursor.at,
            id: cursor.id,
        }),
        Some(limit + 1),
    )
    .await?;
    let next_cursor = next(&mut rows, limit, |row| {
        encode(&TimeCursor {
            at: row.first_takeoff_at,
            id: row.id,
        })
    });
    let items = sessions(&api, &scope, rows, &expand).await?;
    Ok(Json(Page { items, next_cursor }))
}

/// Get a session
#[utoipa::path(
    get, path = "/api/v1/sessions/{id}", tag = "Sessions",
    params(("id" = Uuid, Path), SessionExpand),
    responses(
        (status = 200, body = Session), (status = 401, body = Error),
        (status = 403, body = Error), (status = 404, body = Error)
    ),
    security(("oauth" = ["sessions:list"]))
)]
async fn get_session(
    State(api): State<Api>,
    Extension(caller): Extension<Caller>,
    PathId(id): PathId,
    Params(query): Params<SessionExpand>,
) -> Result<Json<Session>, V1Error> {
    caller.require("sessions:list")?;
    let expand = Expand::parse(query.expand.as_deref(), SESSION_EXPANSIONS, &caller)?;
    let scope = policy::flight_scope(&caller.actor);
    let rows = db::sessions(
        &api.db,
        &scope,
        &Filter::default(),
        Some(&[id]),
        None,
        Some(1),
    )
    .await?;
    sessions(&api, &scope, rows, &expand)
        .await?
        .pop()
        .map(Json)
        .ok_or(V1Error::NotFound)
}

/// List flights
///
/// Flights the user sees, newest first. Archived flights aren't listed.
#[utoipa::path(
    get, path = "/api/v1/flights", tag = "Flights",
    params(Filters, FlightFilters, FlightExpand, Paging),
    responses(
        (status = 200, body = Page<Flight>), (status = 400, body = Error),
        (status = 401, body = Error), (status = 403, body = Error)
    ),
    security(("oauth" = ["flights:list"]))
)]
async fn list_flights(
    State(api): State<Api>,
    Extension(caller): Extension<Caller>,
    Params(filters): Params<Filters>,
    Params(flight_filters): Params<FlightFilters>,
    Params(expansion): Params<FlightExpand>,
    Params(paging): Params<Paging>,
) -> Result<Json<Page<Flight>>, V1Error> {
    caller.require("flights:list")?;
    let expand = Expand::parse(expansion.expand.as_deref(), FLIGHT_EXPANSIONS, &caller)?;
    let mut filter = filter(&api, &filters).await?;
    let (drones, _) = ids(&flight_filters.drone, "drone", false)?;
    let (controllers, _) = ids(&flight_filters.controller, "controller", false)?;
    if let Some(drones) = drones {
        filter.aircraft_serials = Some(db::drone_serials(&api.db, &drones).await?);
    }
    if let Some(controllers) = controllers {
        filter.rc_serials = Some(db::controller_serials(&api.db, &controllers).await?);
    }
    let (session_ids, no_session) = ids(flight_filters.session.as_slice(), "session", true)?;
    let limit = limit(paging.limit)?;
    let after = decode::<TimeCursor>(paging.cursor.as_deref())?;
    let scope = policy::flight_scope(&caller.actor);
    let mut rows = db::flights(
        &api.db,
        &scope,
        &filter,
        InSession {
            ids: session_ids.as_deref(),
            none: no_session,
        },
        None,
        after.map(|cursor| After {
            at: cursor.at,
            id: cursor.id,
        }),
        Some(limit + 1),
    )
    .await?;
    let next_cursor = next(&mut rows, limit, |row| {
        encode(&TimeCursor {
            at: row.takeoff_at,
            id: row.id,
        })
    });
    let items = flights(&api, &scope, rows, &expand).await?;
    Ok(Json(Page { items, next_cursor }))
}

/// Get a flight
#[utoipa::path(
    get, path = "/api/v1/flights/{id}", tag = "Flights",
    params(("id" = Uuid, Path), FlightExpand),
    responses(
        (status = 200, body = Flight), (status = 401, body = Error),
        (status = 403, body = Error), (status = 404, body = Error)
    ),
    security(("oauth" = ["flights:list"]))
)]
async fn get_flight(
    State(api): State<Api>,
    Extension(caller): Extension<Caller>,
    PathId(id): PathId,
    Params(query): Params<FlightExpand>,
) -> Result<Json<Flight>, V1Error> {
    caller.require("flights:list")?;
    let expand = Expand::parse(query.expand.as_deref(), FLIGHT_EXPANSIONS, &caller)?;
    let scope = policy::flight_scope(&caller.actor);
    let rows = db::flights(
        &api.db,
        &scope,
        &Filter::default(),
        InSession::default(),
        Some(&[id]),
        None,
        Some(1),
    )
    .await?;
    flights(&api, &scope, rows, &expand)
        .await?
        .pop()
        .map(Json)
        .ok_or(V1Error::NotFound)
}

/// List groups
///
/// Groups where the user holds a role, or that a flight they see belongs to,
/// in the order they were added.
#[utoipa::path(
    get, path = "/api/v1/groups", tag = "Groups",
    params(Paging),
    responses((status = 200, body = Page<Group>), (status = 400, body = Error), (status = 401, body = Error), (status = 403, body = Error)),
    security(("oauth" = ["groups:list"]))
)]
async fn list_groups(
    State(api): State<Api>,
    Extension(caller): Extension<Caller>,
    Params(paging): Params<Paging>,
) -> Result<Json<Page<Group>>, V1Error> {
    caller.require("groups:list")?;
    let limit = limit(paging.limit)?;
    let after = decode::<IdCursor>(paging.cursor.as_deref())?.map(|cursor| cursor.id);
    let scope = policy::flight_scope(&caller.actor);
    let mut rows = db::groups(&api.db, &scope, after, limit + 1).await?;
    let next_cursor = next(&mut rows, limit, |row| encode(&IdCursor { id: row.id }));
    let items = rows
        .into_iter()
        .map(|row| Group {
            id: row.id,
            name: row.name,
        })
        .collect();
    Ok(Json(Page { items, next_cursor }))
}

/// List areas
///
/// Areas the flights the user sees passed through, in the order they were
/// added. Names only, no boundaries.
#[utoipa::path(
    get, path = "/api/v1/areas", tag = "Areas",
    params(Paging),
    responses((status = 200, body = Page<Area>), (status = 400, body = Error), (status = 401, body = Error), (status = 403, body = Error)),
    security(("oauth" = ["areas:list"]))
)]
async fn list_areas(
    State(api): State<Api>,
    Extension(caller): Extension<Caller>,
    Params(paging): Params<Paging>,
) -> Result<Json<Page<Area>>, V1Error> {
    caller.require("areas:list")?;
    let limit = limit(paging.limit)?;
    let after = decode::<IdCursor>(paging.cursor.as_deref())?.map(|cursor| cursor.id);
    let scope = policy::flight_scope(&caller.actor);
    let mut rows = db::areas(&api.db, &scope, after, limit + 1).await?;
    let next_cursor = next(&mut rows, limit, |row| encode(&IdCursor { id: row.id }));
    let items = rows
        .into_iter()
        .map(|row| Area {
            id: row.id,
            name: row.name,
        })
        .collect();
    Ok(Json(Page { items, next_cursor }))
}

/// List drones
///
/// Drones the user sees, as in Equipment, in the order they were added.
#[utoipa::path(
    get, path = "/api/v1/drones", tag = "Drones",
    params(DeviceExpand, Paging),
    responses((status = 200, body = Page<Device>), (status = 400, body = Error), (status = 401, body = Error), (status = 403, body = Error)),
    security(("oauth" = ["equipment:list"]))
)]
async fn list_drones(
    State(api): State<Api>,
    Extension(caller): Extension<Caller>,
    Params(expansion): Params<DeviceExpand>,
    Params(paging): Params<Paging>,
) -> Result<Json<Page<Device>>, V1Error> {
    devices(&api, &caller, Kind::Aircraft, &expansion, &paging)
        .await
        .map(Json)
}

/// List controllers
///
/// Remote controllers the user sees, as in Equipment, in the order they
/// were added.
#[utoipa::path(
    get, path = "/api/v1/controllers", tag = "Controllers",
    params(DeviceExpand, Paging),
    responses((status = 200, body = Page<Device>), (status = 400, body = Error), (status = 401, body = Error), (status = 403, body = Error)),
    security(("oauth" = ["equipment:list"]))
)]
async fn list_controllers(
    State(api): State<Api>,
    Extension(caller): Extension<Caller>,
    Params(expansion): Params<DeviceExpand>,
    Params(paging): Params<Paging>,
) -> Result<Json<Page<Device>>, V1Error> {
    devices(&api, &caller, Kind::RemoteController, &expansion, &paging)
        .await
        .map(Json)
}

/// List batteries
///
/// Batteries the user sees through their flights, in the order they were
/// added.
#[utoipa::path(
    get, path = "/api/v1/batteries", tag = "Batteries",
    params(Paging),
    responses((status = 200, body = Page<Battery>), (status = 400, body = Error), (status = 401, body = Error), (status = 403, body = Error)),
    security(("oauth" = ["equipment:list"]))
)]
async fn list_batteries(
    State(api): State<Api>,
    Extension(caller): Extension<Caller>,
    Params(paging): Params<Paging>,
) -> Result<Json<Page<Battery>>, V1Error> {
    caller.require("equipment:list")?;
    let limit = limit(paging.limit)? as usize;
    let after = decode::<IdCursor>(paging.cursor.as_deref())?.map(|cursor| cursor.id);
    let scope = policy::flight_scope(&caller.actor);
    let mut all = batteries::db::list(&api.db, &scope, None).await?;
    all.sort_by_key(|battery| battery.id);
    let mut rows: Vec<_> = all
        .into_iter()
        .filter(|battery| after.is_none_or(|after| battery.id > after))
        .take(limit + 1)
        .collect();
    let next_cursor = next(&mut rows, limit as i64, |row| {
        encode(&IdCursor { id: row.id })
    });
    let items = rows
        .into_iter()
        .map(|row| Battery {
            id: row.id,
            name: row.name,
            serial: row.serial,
        })
        .collect();
    Ok(Json(Page { items, next_cursor }))
}

/// A page of drones or controllers. Equipment is a few dozen items, so the
/// page is taken from the whole list the Equipment page shows.
async fn devices(
    api: &Api,
    caller: &Caller,
    kind: Kind,
    expansion: &DeviceExpand,
    paging: &Paging,
) -> Result<Page<Device>, V1Error> {
    caller.require("equipment:list")?;
    let expand = Expand::parse(expansion.expand.as_deref(), DEVICE_EXPANSIONS, caller)?;
    let limit = limit(paging.limit)? as usize;
    let after = decode::<IdCursor>(paging.cursor.as_deref())?.map(|cursor| cursor.id);
    let mut all = equipment::service::list(&api.db, &caller.actor, kind).await?;
    all.sort_by_key(|item| item.id);
    let mut rows: Vec<_> = all
        .into_iter()
        .filter(|item| after.is_none_or(|after| item.id > after))
        .take(limit + 1)
        .collect();
    let next_cursor = next(&mut rows, limit as i64, |row| {
        encode(&IdCursor { id: row.id })
    });
    let groups = if expand.has("group") {
        let ids: Vec<Uuid> = rows.iter().filter_map(|row| row.group_id).collect();
        Some(groups_by_id(api, &ids).await?)
    } else {
        None
    };
    let items = rows
        .into_iter()
        .map(|row| Device {
            id: row.id,
            name: row.name,
            serial: row.serial,
            group: row.group_id.map(|id| pick(id, groups.as_ref())),
        })
        .collect();
    Ok(Page { items, next_cursor })
}

/// Keeps a page of `limit` items from `limit + 1` fetched, and returns the
/// cursor after the last one kept if there were more.
fn next<T>(rows: &mut Vec<T>, limit: i64, cursor: impl Fn(&T) -> String) -> Option<String> {
    let limit = usize::try_from(limit).unwrap_or(usize::MAX);
    if rows.len() > limit {
        rows.truncate(limit);
        rows.last().map(cursor)
    } else {
        None
    }
}

/// The flights' filter: days in the organisation's time zone, ids with
/// `none` for "without one".
async fn filter(api: &Api, filters: &Filters) -> Result<Filter, V1Error> {
    let day = |value: &str| {
        NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .map_err(|_| V1Error::Invalid(format!("`{value}` isn’t a day; give it as YYYY-MM-DD.")))
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
                return Err(V1Error::Invalid("`to` is before `from`.".to_owned()));
            }
            let start = mcp::db::start_of_day(&api.db, first, &api.time_zone).await?;
            let end =
                mcp::db::start_of_day(&api.db, last + chrono::Days::new(1), &api.time_zone).await?;
            (Some(start), Some(end))
        }
        (None, Some(_)) => {
            return Err(V1Error::Invalid("`to` needs `from`.".to_owned()));
        }
    };
    let (group_ids, queue) = ids(&filters.group, "group", true)?;
    let (pilot_ids, unknown_pilot) = ids(&filters.pilot, "pilot", true)?;
    let (area_ids, _) = ids(&filters.area, "area", false)?;
    Ok(Filter {
        from,
        to,
        group_ids,
        queue,
        pilot_ids,
        unknown_pilot,
        area_ids,
        ..Filter::default()
    })
}

/// A filter's ids, None when not given, and whether `none` was among them.
fn ids(
    values: &[String],
    name: &str,
    none_allowed: bool,
) -> Result<(Option<Vec<Uuid>>, bool), V1Error> {
    if values.is_empty() {
        return Ok((None, false));
    }
    let mut ids = Vec::new();
    let mut none = false;
    for value in values {
        if none_allowed && value == "none" {
            none = true;
        } else {
            ids.push(value.parse().map_err(|_| {
                V1Error::Invalid(format!(
                    "`{name}` must be an id{}.",
                    if none_allowed { " or `none`" } else { "" }
                ))
            })?);
        }
    }
    Ok((Some(ids), none))
}

/// The id, or the object if it was expanded (and still exists).
fn pick<T: Clone>(id: Uuid, objects: Option<&HashMap<Uuid, T>>) -> Expandable<T> {
    match objects.and_then(|objects| objects.get(&id)) {
        Some(object) => Expandable::Object(object.clone()),
        None => Expandable::Id(id),
    }
}

async fn groups_by_id(api: &Api, ids: &[Uuid]) -> Result<HashMap<Uuid, Group>, V1Error> {
    Ok(db::groups_by_id(&api.db, ids)
        .await?
        .into_iter()
        .map(|row| {
            (
                row.id,
                Group {
                    id: row.id,
                    name: row.name,
                },
            )
        })
        .collect())
}

async fn people_by_id(api: &Api, ids: &[Uuid]) -> Result<HashMap<Uuid, Person>, V1Error> {
    Ok(db::people_by_id(&api.db, ids)
        .await?
        .into_iter()
        .map(|row| {
            (
                row.id,
                Person {
                    id: row.id,
                    name: row.name,
                    archived: row.archived,
                },
            )
        })
        .collect())
}

async fn areas_by_id(api: &Api, ids: &[Uuid]) -> Result<HashMap<Uuid, Area>, V1Error> {
    Ok(db::areas_by_id(&api.db, ids)
        .await?
        .into_iter()
        .map(|row| {
            (
                row.id,
                Area {
                    id: row.id,
                    name: row.name,
                },
            )
        })
        .collect())
}

/// Per-area figures from a stored list, the areas expanded if asked for.
fn area_figures(
    areas: &serde_json::Value,
    objects: Option<&HashMap<Uuid, Area>>,
) -> Vec<AreaFigures> {
    Vec::<db::AreaRow>::deserialize(areas)
        .unwrap_or_default()
        .into_iter()
        .map(|row| AreaFigures {
            area: pick(row.area, objects),
            air_time_s: row.air_time_s,
            distance_m: row.distance_m.round() as i64,
        })
        .collect()
}

/// The area ids in stored per-area lists, for expanding them.
fn area_ids<'a>(lists: impl Iterator<Item = &'a serde_json::Value>) -> Vec<Uuid> {
    lists
        .flat_map(|areas| Vec::<db::AreaRow>::deserialize(areas).unwrap_or_default())
        .map(|row| row.area)
        .collect()
}

/// Sessions as answered, with what `expand` asks for.
async fn sessions(
    api: &Api,
    scope: &FlightScope,
    rows: Vec<SessionRow>,
    expand: &Expand,
) -> Result<Vec<Session>, V1Error> {
    let session_ids: Vec<Uuid> = rows.iter().map(|row| row.id).collect();
    let groups = if expand.has("group") {
        let ids: Vec<Uuid> = rows.iter().filter_map(|row| row.group_id).collect();
        Some(groups_by_id(api, &ids).await?)
    } else {
        None
    };
    let pilots = if expand.has("pilots") {
        let ids: Vec<Uuid> = rows.iter().flat_map(|row| row.pilot_ids.clone()).collect();
        Some(people_by_id(api, &ids).await?)
    } else {
        None
    };
    let areas = if expand.has("areas.area") {
        Some(areas_by_id(api, &area_ids(rows.iter().map(|row| &row.areas))).await?)
    } else {
        None
    };
    let mut comments: Option<HashMap<Uuid, Vec<Comment>>> = None;
    if expand.has("comments") {
        let stored = db::comments(&api.db, &session_ids).await?;
        let authors = if expand.has("comments.author") {
            let ids: Vec<Uuid> = stored.iter().map(|comment| comment.author_id).collect();
            Some(people_by_id(api, &ids).await?)
        } else {
            None
        };
        let mut by_session: HashMap<Uuid, Vec<Comment>> = HashMap::new();
        for comment in stored {
            by_session
                .entry(comment.session_id)
                .or_default()
                .push(Comment {
                    id: comment.id,
                    author: pick(comment.author_id, authors.as_ref()),
                    text: comment.body,
                    written_at: comment.created_at,
                    edited_at: comment.edited_at,
                });
        }
        comments = Some(by_session);
    }
    let mut flights_by_session: Option<HashMap<Uuid, Vec<Flight>>> = None;
    if expand.has("flights") {
        let rows = db::flights(
            &api.db,
            scope,
            &Filter::default(),
            InSession {
                ids: Some(&session_ids),
                none: false,
            },
            None,
            None,
            None,
        )
        .await?;
        let mut by_session: HashMap<Uuid, Vec<Flight>> = HashMap::new();
        for flight in flights(api, scope, rows, &expand.under("flights")).await? {
            if let Some(Expandable::Id(session)) = &flight.session {
                by_session.entry(*session).or_default().push(flight);
            }
        }
        flights_by_session = Some(by_session);
    }
    Ok(rows
        .into_iter()
        .map(|row| Session {
            id: row.id,
            group: row.group_id.map(|id| pick(id, groups.as_ref())),
            first_takeoff_at: row.first_takeoff_at,
            last_landed_at: row.last_landed_at,
            air_time_s: row.air_time_s,
            distance_m: row.distance_m.round() as i64,
            flight_count: row.flight_count,
            pilots: row
                .pilot_ids
                .iter()
                .map(|id| pick(*id, pilots.as_ref()))
                .collect(),
            areas: area_figures(&row.areas, areas.as_ref()),
            comments: comments
                .as_mut()
                .map(|comments| comments.remove(&row.id).unwrap_or_default()),
            flights: flights_by_session
                .as_mut()
                .map(|flights| flights.remove(&row.id).unwrap_or_default()),
        })
        .collect())
}

/// Flights as answered, with what `expand` asks for. An expanded session is
/// its base shape.
async fn flights(
    api: &Api,
    scope: &FlightScope,
    rows: Vec<FlightRow>,
    expand: &Expand,
) -> Result<Vec<Flight>, V1Error> {
    let pilots = if expand.has("pilot") {
        let ids: Vec<Uuid> = rows.iter().filter_map(|row| row.pilot_id).collect();
        Some(people_by_id(api, &ids).await?)
    } else {
        None
    };
    let groups = if expand.has("group") {
        let ids: Vec<Uuid> = rows.iter().filter_map(|row| row.group_id).collect();
        Some(groups_by_id(api, &ids).await?)
    } else {
        None
    };
    let sessions = if expand.has("session") {
        let ids: Vec<Uuid> = rows.iter().filter_map(|row| row.session_id).collect();
        let stored =
            db::sessions(&api.db, scope, &Filter::default(), Some(&ids), None, None).await?;
        let base = Box::pin(self::sessions(
            api,
            scope,
            stored,
            &Expand(Default::default()),
        ))
        .await?;
        Some(
            base.into_iter()
                .map(|session| (session.id, session))
                .collect::<HashMap<_, _>>(),
        )
    } else {
        None
    };
    let devices = async |wanted: bool, ids: Vec<Uuid>, drones: bool| -> Result<_, V1Error> {
        if !wanted {
            return Ok(None);
        }
        let stored = if drones {
            db::drones_by_id(&api.db, &ids).await?
        } else {
            db::controllers_by_id(&api.db, &ids).await?
        };
        Ok(Some(
            stored
                .into_iter()
                .map(|row| (row.id, row))
                .collect::<HashMap<_, _>>(),
        ))
    };
    let drones = devices(
        expand.has("drone"),
        rows.iter().filter_map(|row| row.drone_id).collect(),
        true,
    )
    .await?;
    let controllers = devices(
        expand.has("controller"),
        rows.iter().filter_map(|row| row.controller_id).collect(),
        false,
    )
    .await?;
    let batteries = if expand.has("batteries") {
        let ids: Vec<Uuid> = rows
            .iter()
            .flat_map(|row| row.battery_ids.clone())
            .collect();
        Some(
            db::batteries_by_id(&api.db, &ids)
                .await?
                .into_iter()
                .map(|row| {
                    (
                        row.id,
                        Battery {
                            id: row.id,
                            name: row.name,
                            serial: row.serial,
                        },
                    )
                })
                .collect::<HashMap<_, _>>(),
        )
    } else {
        None
    };
    let areas = if expand.has("areas.area") {
        Some(areas_by_id(api, &area_ids(rows.iter().map(|row| &row.areas))).await?)
    } else {
        None
    };
    let device = |id: Uuid, stored: Option<&HashMap<Uuid, db::DeviceRow>>| match stored
        .and_then(|stored| stored.get(&id))
    {
        Some(row) => Expandable::Object(Device {
            id: row.id,
            name: row.name.clone(),
            serial: row.serial.clone(),
            group: row.group_id.map(Expandable::Id),
        }),
        None => Expandable::Id(id),
    };
    Ok(rows
        .into_iter()
        .map(|row| Flight {
            id: row.id,
            takeoff_at: row.takeoff_at,
            landed_at: row.landed_at,
            air_time_s: i64::from(row.air_time_s),
            distance_m: row.distance_m.round() as i64,
            max_height_m: row.max_height_m.map(|height| height.round() as i64),
            pilot: row.pilot_id.map(|id| pick(id, pilots.as_ref())),
            group: row.group_id.map(|id| pick(id, groups.as_ref())),
            session: row.session_id.map(|id| pick(id, sessions.as_ref())),
            drone: row.drone_id.map(|id| device(id, drones.as_ref())),
            controller: row.controller_id.map(|id| device(id, controllers.as_ref())),
            batteries: row
                .battery_ids
                .iter()
                .map(|id| pick(*id, batteries.as_ref()))
                .collect(),
            areas: area_figures(&row.areas, areas.as_ref()),
        })
        .collect())
}
