//! Artists bounded context — biographies, lifespan, region, tribe affiliation.

pub mod model;
mod repo;
pub(crate) mod routes;

pub use routes::router;
