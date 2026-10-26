//! What the public API answers: v1's own types, so the web app's can change
//! without changing them. References are ids unless expanded (see
//! docs/api-v1.md).

use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

/// An id, or with `expand` the object it refers to, in the same field, as
/// Stripe does.
#[derive(Clone, Serialize, ToSchema)]
#[serde(untagged)]
pub enum Expandable<T> {
    Id(Uuid),
    Object(T),
}

/// A page of a list. Pass `next_cursor` back as `cursor` for the next page;
/// it is null on the last one.
#[derive(Clone, Serialize, ToSchema)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
}

/// A sentence for a person.
#[derive(Clone, Serialize, ToSchema)]
pub struct Error {
    pub message: String,
}

#[derive(Clone, Serialize, ToSchema)]
pub struct Me {
    pub id: Uuid,
    pub name: String,
}

#[derive(Clone, Serialize, ToSchema)]
pub struct Organisation {
    /// Null when the install hasn't set one.
    pub name: Option<String>,
    /// An IANA time zone name, such as `Europe/Amsterdam`: days in filters
    /// are days there.
    pub time_zone: String,
}

#[derive(Clone, Serialize, ToSchema)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
}

#[derive(Clone, Serialize, ToSchema)]
pub struct Area {
    pub id: Uuid,
    pub name: String,
    /// Only with `expand=geometry` (scope `areas:read`): the area's boundary
    /// as a GeoJSON geometry, a `Polygon` or a `MultiPolygon`, in WGS 84
    /// longitude and latitude. Neighbouring areas share their border exactly.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<Object>)]
    pub geometry: Option<serde_json::Value>,
}

/// A pilot or a comment's author.
#[derive(Clone, Serialize, ToSchema)]
pub struct Person {
    pub id: Uuid,
    pub name: String,
    pub archived: bool,
}

/// A drone or a controller.
#[derive(Clone, Serialize, ToSchema)]
pub struct Device {
    pub id: Uuid,
    /// Null when none is set.
    pub name: Option<String>,
    pub serial: String,
    /// Null: directly in the organisation. Expandable.
    pub group: Option<Expandable<Group>>,
}

#[derive(Clone, Serialize, ToSchema)]
pub struct Battery {
    pub id: Uuid,
    pub name: Option<String>,
    pub serial: String,
}

/// Time and distance in one area.
#[derive(Clone, Serialize, ToSchema)]
pub struct AreaFigures {
    /// Expandable, as `areas.area`.
    pub area: Expandable<Area>,
    pub air_time_s: i64,
    pub distance_m: i64,
}

#[derive(Clone, Serialize, ToSchema)]
pub struct Comment {
    pub id: Uuid,
    /// Expandable, as `comments.author`.
    pub author: Expandable<Person>,
    pub text: String,
    pub written_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Serialize, ToSchema)]
pub struct Session {
    pub id: Uuid,
    /// The session's own group; null for the organisation. Expandable.
    pub group: Option<Expandable<Group>>,
    pub first_takeoff_at: DateTime<Utc>,
    pub last_landed_at: DateTime<Utc>,
    pub air_time_s: i64,
    pub distance_m: i64,
    pub flight_count: i64,
    /// Expandable.
    pub pilots: Vec<Expandable<Person>>,
    /// Most time first; time outside all areas is the session's total
    /// minus these.
    pub areas: Vec<AreaFigures>,
    /// Only with `expand=comments`, oldest first.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<Vec<Comment>>,
    /// Only with `expand=flights`, newest first.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(no_recursion)]
    pub flights: Option<Vec<Flight>>,
}

#[derive(Clone, Serialize, ToSchema)]
pub struct Flight {
    pub id: Uuid,
    pub takeoff_at: DateTime<Utc>,
    pub landed_at: DateTime<Utc>,
    pub air_time_s: i64,
    pub distance_m: i64,
    /// The highest point above the takeoff point.
    pub max_height_m: Option<i64>,
    /// Expandable.
    pub pilot: Option<Expandable<Person>>,
    /// Null: directly in the organisation. Expandable.
    pub group: Option<Expandable<Group>>,
    /// Null: in no session. Expandable.
    #[schema(no_recursion)]
    pub session: Option<Expandable<Session>>,
    /// Expandable.
    pub drone: Option<Expandable<Device>>,
    /// Expandable.
    pub controller: Option<Expandable<Device>>,
    /// Drones with two batteries fly with both. Expandable.
    pub batteries: Vec<Expandable<Battery>>,
    /// Most time first.
    pub areas: Vec<AreaFigures>,
}
