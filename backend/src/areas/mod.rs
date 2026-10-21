//! Areas: named parts of the land that flights are reported by. A flight's
//! time and distance per area come from its track; see "Areas" in
//! docs/architecture.md.

pub mod db;
mod geometry;
pub mod routes;
mod service;
