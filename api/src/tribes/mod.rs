//! Tribes bounded context - peoples, language groups, and (optional)
//! PostGIS territory polygons.

pub mod model;
pub(crate) mod routes;
mod store;

pub use routes::router;
pub use store::{PgTribeStore, TribeStore};
