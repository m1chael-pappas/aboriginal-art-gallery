//! Artists bounded context - biographies, lifespan, region, tribe affiliation.

pub mod model;
pub(crate) mod routes;
mod store;

pub use routes::router;
pub use store::{ArtistStore, PgArtistStore};
