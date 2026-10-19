//! Sessions: a pilot's consecutive flights, grouped for reporting. See
//! "Sessions and operations" in docs/architecture.md.

pub mod db;
pub mod routes;
mod service;

/// The longest break within a session, in minutes.
#[derive(Clone, Copy)]
pub struct SessionGap(pub i32);

/// An uploaded flight joins a session by itself only if the session's flights
/// came in less than this many hours before it: the same night's sync. A later
/// one is left for someone to add (user's decision, 2026-10-19).
pub const LATE_UPLOAD_HOURS: i32 = 12;
