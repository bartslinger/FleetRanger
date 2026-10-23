use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    policy::{self, FlightScope, Permission},
    sessions,
};

/// A flight as listed in the logbook.
#[derive(Serialize)]
pub struct FlightSummary {
    pub id: Uuid,
    pub takeoff_at: DateTime<Utc>,
    pub landed_at: DateTime<Utc>,
    pub air_time_s: i32,
    pub distance_m: f64,
    pub aircraft_serial: Option<String>,
    pub aircraft_name: Option<String>,
    pub rc_serial: Option<String>,
    pub rc_name: Option<String>,
    pub pilot_id: Option<Uuid>,
    pub pilot_name: Option<String>,
    /// None: in the organisation's queue.
    pub group_id: Option<Uuid>,
    pub group_name: Option<String>,
    pub archived_at: Option<DateTime<Utc>>,
}

/// The logbook's filters: only the flights of this drone and/or controller.
#[derive(Default)]
pub struct Filter {
    /// Only the flights of these drones, if given.
    pub aircraft_serials: Option<Vec<String>>,
    /// Only the flights of these controllers, if given.
    pub rc_serials: Option<Vec<String>>,
    /// Only flights in a session (true), or in none (false): the ones still
    /// to be grouped.
    pub in_session: Option<bool>,
    /// Only flights taking off at or after this instant.
    pub from: Option<DateTime<Utc>>,
    /// Only flights taking off before this instant.
    pub to: Option<DateTime<Utc>>,
    /// Only the flights of these groups, if given, and with `queue` those
    /// without a group. The Sessions list applies it to the sessions' own
    /// groups instead.
    pub group_ids: Option<Vec<Uuid>>,
    pub queue: bool,
    /// Only the flights of these pilots, if given, and with `unknown_pilot`
    /// those without a pilot.
    pub pilot_ids: Option<Vec<Uuid>>,
    pub unknown_pilot: bool,
    /// Only archived flights (true), or else only the others.
    pub archived: bool,
    /// Only flights with time in one of these areas, if given.
    pub area_ids: Option<Vec<Uuid>>,
}

/// Columns the logbook can be sorted by.
#[derive(Clone, Copy)]
pub enum SortKey {
    TakeoffAt,
    AirTime,
    Distance,
    /// The drone as shown: its name, or else its serial.
    AircraftSerial,
    /// The controller as shown: its name, or else its serial.
    RcSerial,
}

impl SortKey {
    /// From the column name used in the API.
    pub fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "takeoff_at" => Self::TakeoffAt,
            "air_time_s" => Self::AirTime,
            "distance_m" => Self::Distance,
            "aircraft_serial" => Self::AircraftSerial,
            "rc_serial" => Self::RcSerial,
            _ => return None,
        })
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::TakeoffAt => "takeoff_at",
            Self::AirTime => "air_time_s",
            Self::Distance => "distance_m",
            Self::AircraftSerial => "aircraft_serial",
            Self::RcSerial => "rc_serial",
        }
    }
}

pub struct Sort {
    pub key: SortKey,
    pub descending: bool,
}

// The visibility condition is the same in every flight query: the scope sees
// every flight ($all), or the user's own flights and those of the groups where
// they hold `flights.read`.
// Keep the copies in step; the tests check them against each other.

/// One page of the flights in the scope that pass the filter, in the given order.
pub async fn list(
    db: &PgPool,
    scope: &FlightScope,
    filter: &Filter,
    sort: &Sort,
    limit: i64,
    offset: i64,
) -> sqlx::Result<Vec<FlightSummary>> {
    // A column can't be a query parameter, so each sortable column has a CASE
    // pair; only the one matching $1 and $2 is non-null. The id makes the order
    // stable for equal values.
    sqlx::query_as!(
        FlightSummary,
        r#"select id, takeoff_at, landed_at, air_time_s, distance_m,
                aircraft_serial,
                (select a.name from aircraft a where a.serial = flights.aircraft_serial) as "aircraft_name?",
                rc_serial,
                (select r.name from remote_controllers r where r.serial = flights.rc_serial) as "rc_name?",
                pilot_id, (select u.name from users u where u.id = flights.pilot_id) as "pilot_name?",
                group_id,
                (select g.name from groups g where g.id = flights.group_id) as "group_name?",
                archived_at
         from flights
         where ($5
                or pilot_id = $6
                or group_id = any($18))
           and ($7::text[] is null or aircraft_serial = any($7))
           and ($8::text[] is null or rc_serial = any($8))
           and ($9::bool is null or (session_id is not null) = $9)
           and ($10::timestamptz is null or takeoff_at >= $10)
           and ($11::timestamptz is null or takeoff_at < $11)
           and ($12::uuid[] is null or group_id = any($12) or ($13 and group_id is null))
           and ($14::uuid[] is null or pilot_id = any($14) or ($15 and pilot_id is null))
           and (archived_at is not null) = $16
           and ($17::uuid[] is null or exists (select 1 from flight_areas fa
                                               where fa.flight_id = flights.id
                                                 and fa.area_id = any($17)))
         order by
             case when $1 = 'takeoff_at' and not $2 then takeoff_at end asc,
             case when $1 = 'takeoff_at' and $2 then takeoff_at end desc,
             case when $1 = 'air_time_s' and not $2 then air_time_s end asc,
             case when $1 = 'air_time_s' and $2 then air_time_s end desc,
             case when $1 = 'distance_m' and not $2 then distance_m end asc,
             case when $1 = 'distance_m' and $2 then distance_m end desc,
             case when $1 = 'aircraft_serial' and not $2 then lower(coalesce(
                 (select a.name from aircraft a where a.serial = flights.aircraft_serial),
                 aircraft_serial)) end asc nulls last,
             case when $1 = 'aircraft_serial' and $2 then lower(coalesce(
                 (select a.name from aircraft a where a.serial = flights.aircraft_serial),
                 aircraft_serial)) end desc nulls last,
             case when $1 = 'rc_serial' and not $2 then lower(coalesce(
                 (select r.name from remote_controllers r where r.serial = flights.rc_serial),
                 rc_serial)) end asc nulls last,
             case when $1 = 'rc_serial' and $2 then lower(coalesce(
                 (select r.name from remote_controllers r where r.serial = flights.rc_serial),
                 rc_serial)) end desc nulls last,
             id
         limit $3 offset $4"#,
        sort.key.as_str(),
        sort.descending,
        limit,
        offset,
        scope.all,
        scope.user_id,
        filter.aircraft_serials.as_deref(),
        filter.rc_serials.as_deref(),
        filter.in_session,
        filter.from,
        filter.to,
        filter.group_ids.as_deref(),
        filter.queue,
        filter.pilot_ids.as_deref(),
        filter.unknown_pilot,
        filter.archived,
        filter.area_ids.as_deref(),
        &scope.group_ids,
    )
    .fetch_all(db)
    .await
}

/// One flight with its track, for the flight's page.
#[derive(Serialize)]
pub struct FlightDetail {
    pub id: Uuid,
    pub takeoff_at: DateTime<Utc>,
    pub landed_at: DateTime<Utc>,
    pub air_time_s: i32,
    pub distance_m: f64,
    pub aircraft_serial: Option<String>,
    pub aircraft_name: Option<String>,
    pub rc_serial: Option<String>,
    pub rc_name: Option<String>,
    pub pilot_id: Option<Uuid>,
    pub pilot_name: Option<String>,
    /// None: in the organisation's queue.
    pub group_id: Option<Uuid>,
    pub group_name: Option<String>,
    /// GeoJSON LineString: [longitude, latitude, height above takeoff].
    pub track: Value,
    pub metrics: Value,
    /// The drone's and controller's pages, for linking; None where the
    /// policy doesn't let the actor see them (see `flights::service`).
    pub aircraft_id: Option<Uuid>,
    #[serde(skip)]
    pub aircraft_group_id: Option<Uuid>,
    pub rc_id: Option<Uuid>,
    #[serde(skip)]
    pub rc_group_id: Option<Uuid>,
    /// The battery's page, for linking: whoever sees the flight sees its
    /// battery, but only through flights that aren't archived (see
    /// batteries/db.rs), so None for an archived flight.
    pub battery_id: Option<Uuid>,
    pub battery_name: Option<String>,
    pub archived_at: Option<DateTime<Utc>>,
    /// The areas the track passes through, with the time (`air_time_s`) and
    /// distance (`distance_m`) in each, most time first. The rest of the
    /// flight is outside all areas.
    pub areas: Value,
}

/// A flight in the scope, or None if it doesn't exist or isn't in the scope.
pub async fn get(db: &PgPool, scope: &FlightScope, id: Uuid) -> sqlx::Result<Option<FlightDetail>> {
    sqlx::query_as!(
        FlightDetail,
        r#"select id, takeoff_at, landed_at, air_time_s, distance_m,
                aircraft_serial,
                (select a.name from aircraft a where a.serial = flights.aircraft_serial) as "aircraft_name?",
                rc_serial,
                (select r.name from remote_controllers r where r.serial = flights.rc_serial) as "rc_name?",
                pilot_id, (select u.name from users u where u.id = flights.pilot_id) as "pilot_name?",
                group_id,
                (select g.name from groups g where g.id = flights.group_id) as "group_name?",
                track, metrics,
                (select a.id from aircraft a where a.serial = flights.aircraft_serial) as "aircraft_id?",
                (select a.group_id from aircraft a where a.serial = flights.aircraft_serial) as "aircraft_group_id?",
                (select r.id from remote_controllers r where r.serial = flights.rc_serial) as "rc_id?",
                (select r.group_id from remote_controllers r where r.serial = flights.rc_serial) as "rc_group_id?",
                (select b.id from flight_batteries fb join batteries b on b.id = fb.battery_id
                 where fb.flight_id = flights.id and fb.slot = 0 and flights.archived_at is null) as "battery_id?",
                (select b.name from flight_batteries fb join batteries b on b.id = fb.battery_id
                 where fb.flight_id = flights.id and fb.slot = 0) as "battery_name?",
                archived_at,
                (select coalesce(jsonb_agg(jsonb_build_object(
                            'id', a.id, 'name', a.name,
                            'air_time_s', fa.air_time_s, 'distance_m', fa.distance_m)
                            order by fa.air_time_s desc, lower(a.name), a.id), '[]')
                 from flight_areas fa join areas a on a.id = fa.area_id
                 where fa.flight_id = flights.id) as "areas!"
         from flights
         where id = $1
           and ($2
                or pilot_id = $3
                or group_id = any($4))"#,
        id,
        scope.all,
        scope.user_id,
        &scope.group_ids,
    )
    .fetch_optional(db)
    .await
}

/// The flights of a session that are in the scope, in takeoff order; empty if
/// the session doesn't exist or none of its flights is in the scope.
pub async fn of_session(
    db: &PgPool,
    scope: &FlightScope,
    session_id: Uuid,
) -> sqlx::Result<Vec<FlightDetail>> {
    sqlx::query_as!(
        FlightDetail,
        r#"select id, takeoff_at, landed_at, air_time_s, distance_m,
                aircraft_serial,
                (select a.name from aircraft a where a.serial = flights.aircraft_serial) as "aircraft_name?",
                rc_serial,
                (select r.name from remote_controllers r where r.serial = flights.rc_serial) as "rc_name?",
                pilot_id, (select u.name from users u where u.id = flights.pilot_id) as "pilot_name?",
                group_id,
                (select g.name from groups g where g.id = flights.group_id) as "group_name?",
                track, metrics,
                (select a.id from aircraft a where a.serial = flights.aircraft_serial) as "aircraft_id?",
                (select a.group_id from aircraft a where a.serial = flights.aircraft_serial) as "aircraft_group_id?",
                (select r.id from remote_controllers r where r.serial = flights.rc_serial) as "rc_id?",
                (select r.group_id from remote_controllers r where r.serial = flights.rc_serial) as "rc_group_id?",
                (select b.id from flight_batteries fb join batteries b on b.id = fb.battery_id
                 where fb.flight_id = flights.id and fb.slot = 0 and flights.archived_at is null) as "battery_id?",
                (select b.name from flight_batteries fb join batteries b on b.id = fb.battery_id
                 where fb.flight_id = flights.id and fb.slot = 0) as "battery_name?",
                archived_at,
                (select coalesce(jsonb_agg(jsonb_build_object(
                            'id', a.id, 'name', a.name,
                            'air_time_s', fa.air_time_s, 'distance_m', fa.distance_m)
                            order by fa.air_time_s desc, lower(a.name), a.id), '[]')
                 from flight_areas fa join areas a on a.id = fa.area_id
                 where fa.flight_id = flights.id) as "areas!"
         from flights
         where session_id = $1
           and ($2
                or pilot_id = $3
                or group_id = any($4))
         order by takeoff_at, id"#,
        session_id,
        scope.all,
        scope.user_id,
        &scope.group_ids,
    )
    .fetch_all(db)
    .await
}

/// These flights, as far as they're in the scope, in takeoff order.
pub async fn details(
    db: &PgPool,
    scope: &FlightScope,
    ids: &[Uuid],
) -> sqlx::Result<Vec<FlightDetail>> {
    sqlx::query_as!(
        FlightDetail,
        r#"select id, takeoff_at, landed_at, air_time_s, distance_m,
                aircraft_serial,
                (select a.name from aircraft a where a.serial = flights.aircraft_serial) as "aircraft_name?",
                rc_serial,
                (select r.name from remote_controllers r where r.serial = flights.rc_serial) as "rc_name?",
                pilot_id, (select u.name from users u where u.id = flights.pilot_id) as "pilot_name?",
                group_id,
                (select g.name from groups g where g.id = flights.group_id) as "group_name?",
                track, metrics,
                (select a.id from aircraft a where a.serial = flights.aircraft_serial) as "aircraft_id?",
                (select a.group_id from aircraft a where a.serial = flights.aircraft_serial) as "aircraft_group_id?",
                (select r.id from remote_controllers r where r.serial = flights.rc_serial) as "rc_id?",
                (select r.group_id from remote_controllers r where r.serial = flights.rc_serial) as "rc_group_id?",
                (select b.id from flight_batteries fb join batteries b on b.id = fb.battery_id
                 where fb.flight_id = flights.id and fb.slot = 0 and flights.archived_at is null) as "battery_id?",
                (select b.name from flight_batteries fb join batteries b on b.id = fb.battery_id
                 where fb.flight_id = flights.id and fb.slot = 0) as "battery_name?",
                archived_at,
                (select coalesce(jsonb_agg(jsonb_build_object(
                            'id', a.id, 'name', a.name,
                            'air_time_s', fa.air_time_s, 'distance_m', fa.distance_m)
                            order by fa.air_time_s desc, lower(a.name), a.id), '[]')
                 from flight_areas fa join areas a on a.id = fa.area_id
                 where fa.flight_id = flights.id) as "areas!"
         from flights
         where id = any($1)
           and ($2
                or pilot_id = $3
                or group_id = any($4))
         order by takeoff_at, id"#,
        ids,
        scope.all,
        scope.user_id,
        &scope.group_ids,
    )
    .fetch_all(db)
    .await
}

/// The number of flights in the scope that pass the filter.
pub async fn count(db: &PgPool, scope: &FlightScope, filter: &Filter) -> sqlx::Result<i64> {
    sqlx::query_scalar!(
        r#"select count(*) as "count!"
           from flights
           where ($1
                  or pilot_id = $2
                  or group_id = any($14))
             and ($3::text[] is null or aircraft_serial = any($3))
             and ($4::text[] is null or rc_serial = any($4))
             and ($5::bool is null or (session_id is not null) = $5)
             and ($6::timestamptz is null or takeoff_at >= $6)
             and ($7::timestamptz is null or takeoff_at < $7)
             and ($8::uuid[] is null or group_id = any($8) or ($9 and group_id is null))
             and ($10::uuid[] is null or pilot_id = any($10) or ($11 and pilot_id is null))
             and (archived_at is not null) = $12
             and ($13::uuid[] is null or exists (select 1 from flight_areas fa
                                                 where fa.flight_id = flights.id
                                                   and fa.area_id = any($13)))"#,
        scope.all,
        scope.user_id,
        filter.aircraft_serials.as_deref(),
        filter.rc_serials.as_deref(),
        filter.in_session,
        filter.from,
        filter.to,
        filter.group_ids.as_deref(),
        filter.queue,
        filter.pilot_ids.as_deref(),
        filter.unknown_pilot,
        filter.archived,
        filter.area_ids.as_deref(),
        &scope.group_ids,
    )
    .fetch_one(db)
    .await
}

/// The ids of all flights in the scope that pass the filter, for a batch
/// action on everything a filtered list shows.
pub async fn ids(db: &PgPool, scope: &FlightScope, filter: &Filter) -> sqlx::Result<Vec<Uuid>> {
    sqlx::query_scalar!(
        r#"select id
           from flights
           where ($1
                  or pilot_id = $2
                  or group_id = any($14))
             and ($3::text[] is null or aircraft_serial = any($3))
             and ($4::text[] is null or rc_serial = any($4))
             and ($5::bool is null or (session_id is not null) = $5)
             and ($6::timestamptz is null or takeoff_at >= $6)
             and ($7::timestamptz is null or takeoff_at < $7)
             and ($8::uuid[] is null or group_id = any($8) or ($9 and group_id is null))
             and ($10::uuid[] is null or pilot_id = any($10) or ($11 and pilot_id is null))
             and (archived_at is not null) = $12
             and ($13::uuid[] is null or exists (select 1 from flight_areas fa
                                                 where fa.flight_id = flights.id
                                                   and fa.area_id = any($13)))
           order by takeoff_at, id"#,
        scope.all,
        scope.user_id,
        filter.aircraft_serials.as_deref(),
        filter.rc_serials.as_deref(),
        filter.in_session,
        filter.from,
        filter.to,
        filter.group_ids.as_deref(),
        filter.queue,
        filter.pilot_ids.as_deref(),
        filter.unknown_pilot,
        filter.archived,
        filter.area_ids.as_deref(),
        &scope.group_ids,
    )
    .fetch_all(db)
    .await
}

/// Where a flight is in the list that `list` pages through: its neighbours,
/// its position (from 1) and the list's length.
#[derive(Serialize)]
pub struct Neighbours {
    pub previous: Option<Uuid>,
    pub next: Option<Uuid>,
    pub position: i64,
    pub total: i64,
}

/// The flight's place in the scope's flights that pass the filter, in the
/// given order; None if it isn't in that list. The order must stay the one
/// `list` uses.
pub async fn neighbours(
    db: &PgPool,
    scope: &FlightScope,
    filter: &Filter,
    sort: &Sort,
    id: Uuid,
) -> sqlx::Result<Option<Neighbours>> {
    sqlx::query_as!(
        Neighbours,
        r#"with listed as (
               select id,
                      lag(id) over w as previous,
                      lead(id) over w as next,
                      row_number() over w as position,
                      count(*) over () as total
               from flights
               where ($3
                      or pilot_id = $4
                      or group_id = any($17))
                 and ($5::text[] is null or aircraft_serial = any($5))
                 and ($6::text[] is null or rc_serial = any($6))
                 and ($7::bool is null or (session_id is not null) = $7)
                 and ($9::timestamptz is null or takeoff_at >= $9)
                 and ($10::timestamptz is null or takeoff_at < $10)
                 and ($11::uuid[] is null or group_id = any($11) or ($12 and group_id is null))
                 and ($13::uuid[] is null or pilot_id = any($13) or ($14 and pilot_id is null))
                 and (archived_at is not null) = $15
                 and ($16::uuid[] is null or exists (select 1 from flight_areas fa
                                                     where fa.flight_id = flights.id
                                                       and fa.area_id = any($16)))
               window w as (order by
                   case when $1 = 'takeoff_at' and not $2 then takeoff_at end asc,
                   case when $1 = 'takeoff_at' and $2 then takeoff_at end desc,
                   case when $1 = 'air_time_s' and not $2 then air_time_s end asc,
                   case when $1 = 'air_time_s' and $2 then air_time_s end desc,
                   case when $1 = 'distance_m' and not $2 then distance_m end asc,
                   case when $1 = 'distance_m' and $2 then distance_m end desc,
                   case when $1 = 'aircraft_serial' and not $2 then lower(coalesce(
                       (select a.name from aircraft a where a.serial = flights.aircraft_serial),
                       aircraft_serial)) end asc nulls last,
                   case when $1 = 'aircraft_serial' and $2 then lower(coalesce(
                       (select a.name from aircraft a where a.serial = flights.aircraft_serial),
                       aircraft_serial)) end desc nulls last,
                   case when $1 = 'rc_serial' and not $2 then lower(coalesce(
                       (select r.name from remote_controllers r where r.serial = flights.rc_serial),
                       rc_serial)) end asc nulls last,
                   case when $1 = 'rc_serial' and $2 then lower(coalesce(
                       (select r.name from remote_controllers r where r.serial = flights.rc_serial),
                       rc_serial)) end desc nulls last,
                   id)
           )
           select previous, next, position as "position!", total as "total!"
           from listed where id = $8"#,
        sort.key.as_str(),
        sort.descending,
        scope.all,
        scope.user_id,
        filter.aircraft_serials.as_deref(),
        filter.rc_serials.as_deref(),
        filter.in_session,
        id,
        filter.from,
        filter.to,
        filter.group_ids.as_deref(),
        filter.queue,
        filter.pilot_ids.as_deref(),
        filter.unknown_pilot,
        filter.archived,
        filter.area_ids.as_deref(),
        &scope.group_ids,
    )
    .fetch_optional(db)
    .await
}

/// A drone or controller to filter the logbook by.
#[derive(Serialize)]
pub struct Device {
    pub serial: String,
    pub name: Option<String>,
}

/// The drones to offer in the logbook's filter, by name or else serial: those
/// with flights in the scope that pass the other filters (not the drone filter
/// itself, so more can be added), and the chosen ones, also when they have no
/// such flights, so a choice never disappears.
pub async fn aircraft_of(
    db: &PgPool,
    scope: &FlightScope,
    filter: &Filter,
) -> sqlx::Result<Vec<Device>> {
    sqlx::query_as!(
        Device,
        r#"select f.aircraft_serial as "serial!", a.name as "name?"
           from flights f
           left join aircraft a on a.serial = f.aircraft_serial
           where f.aircraft_serial is not null
             and ($1
                  or f.pilot_id = $2
                  or f.group_id = any($14))
             and (f.aircraft_serial = any(coalesce($3::text[], '{}'))
                  or (($4::text[] is null or f.rc_serial = any($4))
                      and ($5::bool is null or (f.session_id is not null) = $5)
                      and ($6::timestamptz is null or f.takeoff_at >= $6)
                      and ($7::timestamptz is null or f.takeoff_at < $7)
                      and ($8::uuid[] is null or f.group_id = any($8)
                           or ($9 and f.group_id is null))
                      and ($10::uuid[] is null or f.pilot_id = any($10)
                           or ($11 and f.pilot_id is null))
                      and (f.archived_at is not null) = $12
                      and ($13::uuid[] is null or exists (select 1 from flight_areas fa
                                                          where fa.flight_id = f.id
                                                            and fa.area_id = any($13)))))
           group by f.aircraft_serial, a.name
           order by lower(coalesce(a.name, f.aircraft_serial))"#,
        scope.all,
        scope.user_id,
        filter.aircraft_serials.as_deref(),
        filter.rc_serials.as_deref(),
        filter.in_session,
        filter.from,
        filter.to,
        filter.group_ids.as_deref(),
        filter.queue,
        filter.pilot_ids.as_deref(),
        filter.unknown_pilot,
        filter.archived,
        filter.area_ids.as_deref(),
        &scope.group_ids,
    )
    .fetch_all(db)
    .await
}

/// The controllers to offer in the logbook's filter, as `aircraft_of` does
/// for drones.
pub async fn controllers_of(
    db: &PgPool,
    scope: &FlightScope,
    filter: &Filter,
) -> sqlx::Result<Vec<Device>> {
    sqlx::query_as!(
        Device,
        r#"select f.rc_serial as "serial!", r.name as "name?"
           from flights f
           left join remote_controllers r on r.serial = f.rc_serial
           where f.rc_serial is not null
             and ($1
                  or f.pilot_id = $2
                  or f.group_id = any($14))
             and (f.rc_serial = any(coalesce($3::text[], '{}'))
                  or (($4::text[] is null or f.aircraft_serial = any($4))
                      and ($5::bool is null or (f.session_id is not null) = $5)
                      and ($6::timestamptz is null or f.takeoff_at >= $6)
                      and ($7::timestamptz is null or f.takeoff_at < $7)
                      and ($8::uuid[] is null or f.group_id = any($8)
                           or ($9 and f.group_id is null))
                      and ($10::uuid[] is null or f.pilot_id = any($10)
                           or ($11 and f.pilot_id is null))
                      and (f.archived_at is not null) = $12
                      and ($13::uuid[] is null or exists (select 1 from flight_areas fa
                                                          where fa.flight_id = f.id
                                                            and fa.area_id = any($13)))))
           group by f.rc_serial, r.name
           order by lower(coalesce(r.name, f.rc_serial))"#,
        scope.all,
        scope.user_id,
        filter.rc_serials.as_deref(),
        filter.aircraft_serials.as_deref(),
        filter.in_session,
        filter.from,
        filter.to,
        filter.group_ids.as_deref(),
        filter.queue,
        filter.pilot_ids.as_deref(),
        filter.unknown_pilot,
        filter.archived,
        filter.area_ids.as_deref(),
        &scope.group_ids,
    )
    .fetch_all(db)
    .await
}

/// A group to offer in the lists' filter; no id for the flights without a
/// group (the organisation queue) or the sessions in the organisation.
#[derive(Serialize)]
pub struct GroupOption {
    pub id: Option<Uuid>,
    pub name: Option<String>,
}

/// The groups to offer in the logbook's filter, as `aircraft_of` does for
/// drones: those with flights passing the other filters, and the chosen ones.
pub async fn groups_in(
    db: &PgPool,
    scope: &FlightScope,
    filter: &Filter,
) -> sqlx::Result<Vec<GroupOption>> {
    sqlx::query_as!(
        GroupOption,
        r#"select f.group_id as "id?", g.name as "name?"
           from flights f
           left join groups g on g.id = f.group_id
           where ($1
                  or f.pilot_id = $2
                  or f.group_id = any($14))
             and (f.group_id = any(coalesce($3::uuid[], '{}'))
                  or ($4 and f.group_id is null)
                  or (($5::text[] is null or f.aircraft_serial = any($5))
                      and ($6::text[] is null or f.rc_serial = any($6))
                      and ($7::bool is null or (f.session_id is not null) = $7)
                      and ($8::timestamptz is null or f.takeoff_at >= $8)
                      and ($9::timestamptz is null or f.takeoff_at < $9)
                      and ($10::uuid[] is null or f.pilot_id = any($10)
                           or ($11 and f.pilot_id is null))
                      and (f.archived_at is not null) = $12
                      and ($13::uuid[] is null or exists (select 1 from flight_areas fa
                                                          where fa.flight_id = f.id
                                                            and fa.area_id = any($13)))))
           group by f.group_id, g.name
           order by lower(g.name) nulls last"#,
        scope.all,
        scope.user_id,
        filter.group_ids.as_deref(),
        filter.queue,
        filter.aircraft_serials.as_deref(),
        filter.rc_serials.as_deref(),
        filter.in_session,
        filter.from,
        filter.to,
        filter.pilot_ids.as_deref(),
        filter.unknown_pilot,
        filter.archived,
        filter.area_ids.as_deref(),
        &scope.group_ids,
    )
    .fetch_all(db)
    .await
}

/// An area to offer in the lists' filter.
#[derive(Serialize)]
pub struct AreaOption {
    pub id: Uuid,
    pub name: String,
}

/// The areas to offer in the lists' filter, as `aircraft_of` does for
/// drones: those with time of flights passing the other filters, and the
/// chosen ones.
pub async fn areas_in(
    db: &PgPool,
    scope: &FlightScope,
    filter: &Filter,
) -> sqlx::Result<Vec<AreaOption>> {
    sqlx::query_as!(
        AreaOption,
        r#"select a.id, a.name
           from areas a
           where a.id = any(coalesce($3::uuid[], '{}'))
              or exists (
                  select 1 from flight_areas fa join flights f on f.id = fa.flight_id
                  where fa.area_id = a.id
                    and ($1
                         or f.pilot_id = $2
                         or f.group_id = any($14))
                    and ($4::text[] is null or f.aircraft_serial = any($4))
                    and ($5::text[] is null or f.rc_serial = any($5))
                    and ($6::bool is null or (f.session_id is not null) = $6)
                    and ($7::timestamptz is null or f.takeoff_at >= $7)
                    and ($8::timestamptz is null or f.takeoff_at < $8)
                    and ($9::uuid[] is null or f.group_id = any($9) or ($10 and f.group_id is null))
                    and ($11::uuid[] is null or f.pilot_id = any($11)
                         or ($12 and f.pilot_id is null))
                    and (f.archived_at is not null) = $13)
           order by lower(a.name), a.id"#,
        scope.all,
        scope.user_id,
        filter.area_ids.as_deref(),
        filter.aircraft_serials.as_deref(),
        filter.rc_serials.as_deref(),
        filter.in_session,
        filter.from,
        filter.to,
        filter.group_ids.as_deref(),
        filter.queue,
        filter.pilot_ids.as_deref(),
        filter.unknown_pilot,
        filter.archived,
        &scope.group_ids,
    )
    .fetch_all(db)
    .await
}

/// A pilot to offer in the lists' filter; no id for the flights without a
/// pilot.
#[derive(Serialize)]
pub struct PilotOption {
    pub id: Option<Uuid>,
    pub name: Option<String>,
}

/// The pilots to offer in the lists' filter, as `aircraft_of` does for
/// drones: those with flights passing the other filters, and the chosen ones.
pub async fn pilots_in(
    db: &PgPool,
    scope: &FlightScope,
    filter: &Filter,
) -> sqlx::Result<Vec<PilotOption>> {
    sqlx::query_as!(
        PilotOption,
        r#"select f.pilot_id as "id?", u.name as "name?"
           from flights f
           left join users u on u.id = f.pilot_id
           where ($1
                  or f.pilot_id = $2
                  or f.group_id = any($14))
             and (f.pilot_id = any(coalesce($3::uuid[], '{}'))
                  or ($4 and f.pilot_id is null)
                  or (($5::text[] is null or f.aircraft_serial = any($5))
                      and ($6::text[] is null or f.rc_serial = any($6))
                      and ($7::bool is null or (f.session_id is not null) = $7)
                      and ($8::timestamptz is null or f.takeoff_at >= $8)
                      and ($9::timestamptz is null or f.takeoff_at < $9)
                      and ($10::uuid[] is null or f.group_id = any($10)
                           or ($11 and f.group_id is null))
                      and (f.archived_at is not null) = $12
                      and ($13::uuid[] is null or exists (select 1 from flight_areas fa
                                                          where fa.flight_id = f.id
                                                            and fa.area_id = any($13)))))
           group by f.pilot_id, u.name
           order by lower(u.name) nulls last"#,
        scope.all,
        scope.user_id,
        filter.pilot_ids.as_deref(),
        filter.unknown_pilot,
        filter.aircraft_serials.as_deref(),
        filter.rc_serials.as_deref(),
        filter.in_session,
        filter.from,
        filter.to,
        filter.group_ids.as_deref(),
        filter.queue,
        filter.archived,
        filter.area_ids.as_deref(),
        &scope.group_ids,
    )
    .fetch_all(db)
    .await
}

/// Whether all these flights exist and are in the scope.
pub async fn all_visible(db: &PgPool, scope: &FlightScope, ids: &[Uuid]) -> sqlx::Result<bool> {
    let visible = sqlx::query_scalar!(
        r#"select count(*) as "count!" from flights
           where id = any($1)
             and ($2
                  or pilot_id = $3
                  or group_id = any($4))"#,
        ids,
        scope.all,
        scope.user_id,
        &scope.group_ids,
    )
    .fetch_one(db)
    .await?;
    Ok(usize::try_from(visible).is_ok_and(|visible| visible == ids.len()))
}

/// The group of each of these flights, or None if one of them doesn't exist.
/// The visibility check is the policy's job, not this query's.
pub async fn groups_of(db: &PgPool, ids: &[Uuid]) -> sqlx::Result<Option<Vec<Option<Uuid>>>> {
    let groups = sqlx::query_scalar!("select group_id from flights where id = any($1)", ids)
        .fetch_all(db)
        .await?;
    Ok((groups.len() == ids.len()).then_some(groups))
}

/// Whether this is an active user who holds `flights.fly_for` anywhere, on a
/// group or the organisation: someone who can be a flight's pilot.
pub async fn flies(db: &PgPool, user_id: Uuid) -> sqlx::Result<bool> {
    sqlx::query_scalar!(
        r#"select exists(
               select 1 from users u
               where u.id = $1 and u.archived_at is null
                 and (exists (select 1 from group_members m
                              where m.user_id = u.id and m.role = any($2))
                      or exists (select 1 from organisation_roles r
                                 where r.user_id = u.id and r.role = any($2)))
           ) as "flies!""#,
        user_id,
        &policy::roles_granting(Permission::FlightsFlyFor),
    )
    .fetch_one(db)
    .await
}

pub async fn group_exists(db: &PgPool, group_id: Uuid) -> sqlx::Result<bool> {
    sqlx::query_scalar!(
        r#"select exists(select 1 from groups where id = $1) as "exists!""#,
        group_id,
    )
    .fetch_one(db)
    .await
}

/// Sets the pilot of these flights. They stay in their group and their
/// session: sessions change only when someone changes them.
pub async fn set_pilot(db: &PgPool, ids: &[Uuid], pilot_id: Option<Uuid>) -> sqlx::Result<u64> {
    let result = sqlx::query!(
        "update flights set pilot_id = $2 where id = any($1)",
        ids,
        pilot_id,
    )
    .execute(db)
    .await?;
    Ok(result.rows_affected())
}

/// Moves these flights to a group, or to the queue. They keep their pilot and
/// their session.
pub async fn set_group(db: &PgPool, ids: &[Uuid], group_id: Option<Uuid>) -> sqlx::Result<u64> {
    let result = sqlx::query!(
        "update flights set group_id = $2 where id = any($1)",
        ids,
        group_id,
    )
    .execute(db)
    .await?;
    Ok(result.rows_affected())
}

/// Archives these flights and returns how many weren't archived yet, or None
/// without archiving any when one of them is in a session: archiving never
/// changes a session.
pub async fn archive(db: &PgPool, ids: &[Uuid]) -> sqlx::Result<Option<u64>> {
    let mut tx = sessions::db::locked(db).await?;
    let in_session = sqlx::query_scalar!(
        r#"select exists(select 1 from flights where id = any($1) and session_id is not null)
               as "exists!""#,
        ids,
    )
    .fetch_one(&mut *tx)
    .await?;
    if in_session {
        return Ok(None);
    }
    let archived = sqlx::query!(
        "update flights set archived_at = now() where id = any($1) and archived_at is null",
        ids,
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();
    tx.commit().await?;
    Ok(Some(archived))
}

/// Restores these archived flights, without a session, and returns how many
/// were archived.
pub async fn restore(db: &PgPool, ids: &[Uuid]) -> sqlx::Result<u64> {
    let result = sqlx::query!(
        "update flights set archived_at = null where id = any($1) and archived_at is not null",
        ids,
    )
    .execute(db)
    .await?;
    Ok(result.rows_affected())
}

/// Whether any of these flights is archived.
pub async fn any_archived(db: &PgPool, ids: &[Uuid]) -> sqlx::Result<bool> {
    sqlx::query_scalar!(
        r#"select exists(select 1 from flights where id = any($1) and archived_at is not null)
               as "exists!""#,
        ids,
    )
    .fetch_one(db)
    .await
}

/// A user who can be chosen as the pilot of flights: active and holding
/// `flights.fly_for` anywhere.
#[derive(Serialize)]
pub struct PilotChoice {
    pub id: Uuid,
    pub name: String,
    /// The groups they are a member of with Pilot, so a list for a group's
    /// flights can put its own pilots first.
    pub group_ids: Vec<Uuid>,
}

/// Everyone who flies, by name.
pub async fn pilot_choices(db: &PgPool) -> sqlx::Result<Vec<PilotChoice>> {
    sqlx::query_as!(
        PilotChoice,
        r#"select u.id, u.name,
                  array(select m.group_id from group_members m
                        where m.user_id = u.id and m.role = any($1)
                        order by m.group_id) as "group_ids!"
           from users u
           where u.archived_at is null
             and (exists (select 1 from group_members m
                          where m.user_id = u.id and m.role = any($1))
                  or exists (select 1 from organisation_roles r
                             where r.user_id = u.id and r.role = any($1)))
           order by lower(u.name), u.id"#,
        &policy::roles_granting(Permission::FlightsFlyFor),
    )
    .fetch_all(db)
    .await
}

/// A group to choose, e.g. to move flights to.
#[derive(Serialize)]
pub struct GroupChoice {
    pub id: Uuid,
    pub name: String,
}

/// These groups, by name.
pub async fn groups_named(db: &PgPool, ids: &[Uuid]) -> sqlx::Result<Vec<GroupChoice>> {
    sqlx::query_as!(
        GroupChoice,
        "select id, name from groups where id = any($1) order by lower(name), id",
        ids,
    )
    .fetch_all(db)
    .await
}

pub async fn group_choices(db: &PgPool) -> sqlx::Result<Vec<GroupChoice>> {
    sqlx::query_as!(
        GroupChoice,
        "select id, name from groups order by lower(name), id",
    )
    .fetch_all(db)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing;

    /// Adds a flight with the given group and pilot, returning its id.
    async fn flight(db: &PgPool, group: Option<Uuid>, pilot: Option<Uuid>) -> Uuid {
        let raw_log: Uuid = sqlx::query_scalar(
            "insert into raw_logs (sha256, path, size_bytes)
             values (sha256(gen_random_uuid()::text::bytea), gen_random_uuid()::text, 0)
             returning id",
        )
        .fetch_one(db)
        .await
        .unwrap();
        sqlx::query_scalar(
            "insert into flights (raw_log_id, takeoff_at, landed_at, air_time_s, distance_m,
                                  track, group_id, pilot_id)
             values ($1, now(), now(), 0, 0, '{}', $2, $3)
             returning id",
        )
        .bind(raw_log)
        .bind(group)
        .bind(pilot)
        .fetch_one(db)
        .await
        .unwrap()
    }

    async fn insert_id(db: &PgPool, sql: &'static str) -> Uuid {
        sqlx::query_scalar(sql).fetch_one(db).await.unwrap()
    }

    async fn visible(db: &PgPool, scope: &FlightScope) -> Vec<Uuid> {
        let sort = Sort {
            key: SortKey::TakeoffAt,
            descending: false,
        };
        let filter = Filter::default();
        let mut ids: Vec<Uuid> = list(db, scope, &filter, &sort, 100, 0)
            .await
            .unwrap()
            .into_iter()
            .map(|flight| flight.id)
            .collect();
        assert_eq!(count(db, scope, &filter).await.unwrap(), ids.len() as i64);
        ids.sort();
        // A single flight is found exactly when it is listed.
        let all: Vec<Uuid> = sqlx::query_scalar("select id from flights")
            .fetch_all(db)
            .await
            .unwrap();
        for id in all {
            let found = get(db, scope, id).await.unwrap().is_some();
            assert_eq!(found, ids.contains(&id), "flight {id}");
        }
        ids
    }

    /// A member sees their own flights, also from an earlier group, and all
    /// flights of their current group; not other groups or the queue.
    #[sqlx::test]
    async fn member_sees_own_and_group_flights(db: PgPool) {
        let north = testing::group(&db, "North").await;
        let south = testing::group(&db, "South").await;
        let me = insert_id(&db, "insert into users (name) values ('A') returning id").await;
        let fellow = insert_id(&db, "insert into users (name) values ('B') returning id").await;
        let other = insert_id(&db, "insert into users (name) values ('C') returning id").await;
        for (pilot, group) in [(me, north), (fellow, north), (other, south)] {
            testing::join(&db, pilot, group).await;
        }

        let mut expected = vec![
            flight(&db, Some(north), Some(me)).await,
            flight(&db, Some(north), Some(fellow)).await,
            flight(&db, Some(north), None).await,
            // Flown for South before moving to North.
            flight(&db, Some(south), Some(me)).await,
        ];
        flight(&db, Some(south), Some(other)).await;
        flight(&db, Some(south), None).await;
        flight(&db, None, None).await;
        expected.sort();

        let member = testing::scope(&db, me).await;
        assert_eq!(visible(&db, &member).await, expected);
        let everything = FlightScope {
            all: true,
            user_id: me,
            group_ids: Vec::new(),
        };
        assert_eq!(visible(&db, &everything).await.len(), 7);
    }

    /// A flight's neighbours and position agree with the list, for every
    /// order (ties and missing drones included) and with a filter; a flight
    /// the filter leaves out has none.
    #[sqlx::test]
    async fn neighbours_follow_the_list(db: PgPool) {
        let session = insert_id(&db, "insert into sessions default values returning id").await;
        let mut grouped = None;
        for (air_time, serial, in_session) in [
            (60, Some("B"), false),
            (60, None, true),
            (30, Some("A"), false),
            (90, Some("B"), false),
            (30, None, false),
        ] {
            let id = flight(&db, None, None).await;
            sqlx::query(
                "update flights set air_time_s = $2, distance_m = $2 * 10, aircraft_serial = $3,
                     session_id = $4, takeoff_at = now() - make_interval(mins => $2)
                 where id = $1",
            )
            .bind(id)
            .bind(air_time)
            .bind(serial)
            .bind(in_session.then_some(session))
            .execute(&db)
            .await
            .unwrap();
            if in_session {
                grouped = Some(id);
            }
        }
        let scope = FlightScope {
            all: true,
            user_id: Uuid::nil(),
            group_ids: Vec::new(),
        };
        let keys = [
            SortKey::TakeoffAt,
            SortKey::AirTime,
            SortKey::Distance,
            SortKey::AircraftSerial,
            SortKey::RcSerial,
        ];
        let filters = || {
            [
                Filter::default(),
                Filter {
                    in_session: Some(false),
                    ..Filter::default()
                },
                Filter {
                    aircraft_serials: Some(vec!["A".to_owned(), "B".to_owned()]),
                    ..Filter::default()
                },
                // Takeoffs 30 to 90 minutes ago: the 60-minute ones and the 90.
                Filter {
                    from: Some(Utc::now() - chrono::Duration::minutes(100)),
                    to: Some(Utc::now() - chrono::Duration::minutes(45)),
                    ..Filter::default()
                },
                // These flights have no group: the queue holds them all, a group none.
                Filter {
                    group_ids: Some(Vec::new()),
                    queue: true,
                    ..Filter::default()
                },
                Filter {
                    group_ids: Some(vec![Uuid::nil()]),
                    ..Filter::default()
                },
                // Nor a pilot: "Unknown" holds them all, a pilot none.
                Filter {
                    pilot_ids: Some(Vec::new()),
                    unknown_pilot: true,
                    ..Filter::default()
                },
                Filter {
                    pilot_ids: Some(vec![Uuid::nil()]),
                    ..Filter::default()
                },
            ]
        };
        let [_, _, by_drone, by_time, queue, a_group, unknown, a_pilot] = filters();
        for (filter, expected) in [(&unknown, 5), (&a_pilot, 0)] {
            assert_eq!(count(&db, &scope, filter).await.unwrap(), expected);
        }
        let none = || Sort {
            key: SortKey::TakeoffAt,
            descending: true,
        };
        assert_eq!(
            list(&db, &scope, &queue, &none(), 100, 0)
                .await
                .unwrap()
                .len(),
            5
        );
        assert!(
            list(&db, &scope, &a_group, &none(), 100, 0)
                .await
                .unwrap()
                .is_empty()
        );
        let all = || Sort {
            key: SortKey::TakeoffAt,
            descending: true,
        };
        let drones = list(&db, &scope, &by_drone, &all(), 100, 0).await.unwrap();
        assert_eq!(drones.len(), 3, "the flights of drones A and B");
        let window = list(&db, &scope, &by_time, &all(), 100, 0).await.unwrap();
        let mut air_times: Vec<i32> = window.iter().map(|flight| flight.air_time_s).collect();
        air_times.sort_unstable();
        assert_eq!(air_times, [60, 60, 90]);

        for filter in filters() {
            let newest_first = Sort {
                key: SortKey::TakeoffAt,
                descending: true,
            };
            let listed = list(&db, &scope, &filter, &newest_first, 100, 0)
                .await
                .unwrap()
                .len() as i64;
            assert_eq!(count(&db, &scope, &filter).await.unwrap(), listed);
            assert_eq!(
                ids(&db, &scope, &filter).await.unwrap().len() as i64,
                listed
            );
            for key in keys {
                for descending in [false, true] {
                    let sort = Sort { key, descending };
                    let ids: Vec<Uuid> = list(&db, &scope, &filter, &sort, 100, 0)
                        .await
                        .unwrap()
                        .into_iter()
                        .map(|flight| flight.id)
                        .collect();
                    for (index, id) in ids.iter().enumerate() {
                        let place = neighbours(&db, &scope, &filter, &sort, *id)
                            .await
                            .unwrap()
                            .unwrap();
                        let at = |i: Option<usize>| i.and_then(|i| ids.get(i)).copied();
                        assert_eq!(place.previous, at(index.checked_sub(1)));
                        assert_eq!(place.next, at(Some(index + 1)));
                        assert_eq!(place.position, index as i64 + 1);
                        assert_eq!(place.total, ids.len() as i64);
                    }
                }
            }
        }
        let filter = Filter {
            in_session: Some(false),
            ..Filter::default()
        };
        let sort = Sort {
            key: SortKey::TakeoffAt,
            descending: true,
        };
        let left_out = neighbours(&db, &scope, &filter, &sort, grouped.unwrap()).await;
        assert!(left_out.unwrap().is_none());
    }

    /// The filters' options narrow by the other filters, not by their own
    /// choice, and keep what was chosen even without flights left.
    #[sqlx::test]
    async fn filter_options_narrow_by_the_other_filters(db: PgPool) {
        for (drone, controller) in [("A", "X"), ("B", "Y")] {
            let id = flight(&db, None, None).await;
            sqlx::query("update flights set aircraft_serial = $2, rc_serial = $3 where id = $1")
                .bind(id)
                .bind(drone)
                .bind(controller)
                .execute(&db)
                .await
                .unwrap();
        }
        let scope = FlightScope {
            all: true,
            user_id: Uuid::nil(),
            group_ids: Vec::new(),
        };
        let serials = |devices: Vec<Device>| -> Vec<String> {
            devices.into_iter().map(|device| device.serial).collect()
        };
        let list = |values: &[&str]| Some(values.iter().map(|v| (*v).to_owned()).collect());

        // Controller X chosen: only its drone is offered; its own list stays whole.
        let x = Filter {
            rc_serials: list(&["X"]),
            ..Filter::default()
        };
        assert_eq!(serials(aircraft_of(&db, &scope, &x).await.unwrap()), ["A"]);
        assert_eq!(
            serials(controllers_of(&db, &scope, &x).await.unwrap()),
            ["X", "Y"]
        );

        // Drone B chosen as well, though X never flew it: B stays offered,
        // and the controllers are X (chosen) and B's controller Y.
        let x_and_b = Filter {
            rc_serials: list(&["X"]),
            aircraft_serials: list(&["B"]),
            ..Filter::default()
        };
        assert_eq!(
            serials(aircraft_of(&db, &scope, &x_and_b).await.unwrap()),
            ["A", "B"]
        );
        assert_eq!(
            serials(controllers_of(&db, &scope, &x_and_b).await.unwrap()),
            ["X", "Y"]
        );
    }

    /// A user without a group or flights sees nothing unless they see all.
    #[sqlx::test]
    async fn user_without_group_sees_nothing(db: PgPool) {
        let user = insert_id(&db, "insert into users (name) values ('A') returning id").await;
        let group = testing::group(&db, "North").await;
        flight(&db, Some(group), None).await;
        flight(&db, None, None).await;
        let member = testing::scope(&db, user).await;
        assert!(visible(&db, &member).await.is_empty());
    }

    /// A flight's pilot and group are independent: moving it, also to a
    /// group its pilot isn't in or to the queue, keeps its pilot and its
    /// session, and setting its pilot keeps its group.
    #[sqlx::test]
    async fn pilot_and_group_change_independently(db: PgPool) {
        let north = testing::group(&db, "North").await;
        let south = testing::group(&db, "South").await;
        let pilot = insert_id(&db, "insert into users (name) values ('A') returning id").await;
        testing::join(&db, pilot, south).await;
        let flight = flight(&db, Some(north), Some(pilot)).await;
        let session = sessions::db::split(&db, &[flight], None).await.unwrap();
        let state = async || -> (Option<Uuid>, Option<Uuid>, Option<Uuid>) {
            sqlx::query_as("select pilot_id, group_id, session_id from flights where id = $1")
                .bind(flight)
                .fetch_one(&db)
                .await
                .unwrap()
        };

        set_group(&db, &[flight], None).await.unwrap();
        assert_eq!(state().await, (Some(pilot), None, Some(session)));
        set_group(&db, &[flight], Some(north)).await.unwrap();
        set_pilot(&db, &[flight], None).await.unwrap();
        assert_eq!(state().await, (None, Some(north), Some(session)));
    }

    /// Only someone active with `flights.fly_for` somewhere can be a pilot:
    /// a member with Pilot in a group, or Pilot on the organisation, but not
    /// an admin or an editor who doesn't fly, nor an archived pilot.
    #[sqlx::test]
    async fn only_who_flies_can_be_a_pilot(db: PgPool) {
        let north = testing::group(&db, "North").await;
        let user = async |name: &str| -> Uuid {
            sqlx::query_scalar("insert into users (name) values ($1) returning id")
                .bind(name)
                .fetch_one(&db)
                .await
                .unwrap()
        };
        let (member, guest, admin, editor, archived) = (
            user("Member").await,
            user("Guest").await,
            user("Admin").await,
            user("Editor").await,
            user("Archived").await,
        );
        testing::join(&db, member, north).await;
        testing::join(&db, archived, north).await;
        sqlx::query(
            "insert into organisation_roles (user_id, role) values ($1, 'pilot'), ($2, 'admin')",
        )
        .bind(guest)
        .bind(admin)
        .execute(&db)
        .await
        .unwrap();
        sqlx::query(
            "insert into group_members (group_id, user_id, role) values ($1, $2, 'editor')",
        )
        .bind(north)
        .bind(editor)
        .execute(&db)
        .await
        .unwrap();
        sqlx::query("update users set archived_at = now() where id = $1")
            .bind(archived)
            .execute(&db)
            .await
            .unwrap();

        for (who, expected) in [
            (member, true),
            (guest, true),
            (admin, false),
            (editor, false),
            (archived, false),
        ] {
            assert_eq!(flies(&db, who).await.unwrap(), expected);
        }
        // The pilot list agrees, with the groups each flies for as a member.
        let listed: Vec<(Uuid, Vec<Uuid>)> = pilot_choices(&db)
            .await
            .unwrap()
            .into_iter()
            .map(|pilot| (pilot.id, pilot.group_ids))
            .collect();
        assert_eq!(listed, [(guest, vec![]), (member, vec![north])]);
    }

    /// Only flights without a session can be archived: a batch with a flight
    /// in a session is refused whole. Archived flights are left out of the
    /// list but for the archived filter; restoring brings them back.
    #[sqlx::test]
    async fn archived_flights_leave_lists_but_never_sessions(db: PgPool) {
        let (a, b, c) = (
            flight(&db, None, None).await,
            flight(&db, None, None).await,
            flight(&db, None, None).await,
        );
        let session = sessions::db::split(&db, &[b], None).await.unwrap();

        assert_eq!(archive(&db, &[a, b]).await.unwrap(), None);
        assert!(!any_archived(&db, &[a, b]).await.unwrap());
        assert_eq!(archive(&db, &[a, c]).await.unwrap(), Some(2));
        assert_eq!(archive(&db, &[a]).await.unwrap(), Some(0));
        let session_of_b: Option<Uuid> =
            sqlx::query_scalar("select session_id from flights where id = $1")
                .bind(b)
                .fetch_one(&db)
                .await
                .unwrap();
        assert_eq!(session_of_b, Some(session));

        let everything = FlightScope {
            all: true,
            user_id: Uuid::nil(),
            group_ids: Vec::new(),
        };
        let listed = async |archived: bool| -> Vec<Uuid> {
            let filter = Filter {
                archived,
                ..Filter::default()
            };
            let sort = Sort {
                key: SortKey::TakeoffAt,
                descending: false,
            };
            let mut ids: Vec<Uuid> = list(&db, &everything, &filter, &sort, 100, 0)
                .await
                .unwrap()
                .into_iter()
                .map(|flight| flight.id)
                .collect();
            ids.sort();
            ids
        };
        assert_eq!(listed(false).await, vec![b]);
        let mut archived = vec![a, c];
        archived.sort();
        assert_eq!(listed(true).await, archived);

        assert_eq!(restore(&db, &[a]).await.unwrap(), 1);
        assert!(!any_archived(&db, &[a, b]).await.unwrap());
    }
}
