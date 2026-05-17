//! Tribes bounded context - peoples, language groups, and (optional)
//! PostGIS territory polygons.

pub mod model;
mod store;
pub(crate) mod routes;

pub use routes::router;
pub use store::{PgTribeStore, TribeStore};
