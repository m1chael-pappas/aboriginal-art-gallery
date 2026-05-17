//! Tribes bounded context - peoples, language groups, and (optional)
//! PostGIS territory polygons.

pub mod model;
mod repo;
pub(crate) mod routes;

pub use routes::router;
