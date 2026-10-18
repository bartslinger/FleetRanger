//! Sessions: a pilot's consecutive flights, grouped for reporting. See
//! "Sessions and operations" in docs/architecture.md.

pub mod db;
pub mod routes;
mod service;

/// The longest break within a session, in minutes.
#[derive(Clone, Copy)]
pub struct SessionGap(pub i32);
