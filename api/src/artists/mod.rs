//! Artists bounded context - biographies, lifespan, region, tribe affiliation.

pub mod model;
mod store;
pub(crate) mod routes;

pub use routes::router;
pub use store::{ArtistStore, PgArtistStore};
