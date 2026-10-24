//! Groups and their members' roles; see "Groups and roles" in
//! docs/architecture.md. A group holds flights, sessions, drones and
//! controllers.

pub mod db;
#[cfg(test)]
mod fill;
pub mod routes;
mod service;
