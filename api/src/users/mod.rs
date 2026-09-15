//! Users bounded context - accounts, role-based authorisation, and the
//! Argon2 + JWT auth flow built on top of [`crate::auth`].

pub mod model;
pub(crate) mod routes;
mod store;

pub use routes::router;
pub use store::{PgUserStore, UserStore};
