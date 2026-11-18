//! Sessions: a pilot's consecutive flights, grouped for reporting. See
//! "Sessions" in docs/architecture.md.

pub mod comments;
pub mod db;
pub mod routes;
pub mod service;
pub mod suggest;

/// The longest break within a session, in minutes.
#[derive(Clone, Copy)]
pub struct SessionGap(pub i32);

/// An uploaded flight joins a session by itself only if the session's flights
/// came in less than this many hours before it: the same night's sync. A later
/// one is left for someone to add (user's decision, 2026-10-19).
pub const LATE_UPLOAD_HOURS: i32 = 12;

/// Sessions a new flight bridges become one only if all came in less than
/// this many minutes before it, in one sync, so a session someone changed
/// after an earlier sync stays as it is (my choice, 2026-11-18).
pub const MERGE_WITHIN_MINUTES: i64 = 60;
