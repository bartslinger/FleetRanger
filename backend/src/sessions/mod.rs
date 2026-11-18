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
