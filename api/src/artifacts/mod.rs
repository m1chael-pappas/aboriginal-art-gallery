//! Artifacts bounded context - works, attributed to an artist.

pub mod model;
mod store;
pub(crate) mod routes;

pub use routes::router;
pub use store::{ArtifactStore, PgArtifactStore};
