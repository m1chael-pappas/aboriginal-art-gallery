//! Artifacts bounded context - works, attributed to an artist.

pub mod model;
pub(crate) mod routes;
mod store;

pub use routes::router;
pub use store::{ArtifactStore, PgArtifactStore};
