//! Users bounded context - accounts, role-based authorisation, and the
//! Argon2 + JWT auth flow built on top of [`crate::auth`].

pub mod model;
mod store;
pub(crate) mod routes;

pub use routes::router;
pub use store::{PgUserStore, UserStore};
