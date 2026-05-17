//! Authentication primitives: password hashing, JWT issue / verify, and the
//! axum extractors that gate protected routes.
//!
//! The HTTP surface that *uses* these primitives - register, login, the
//! current-user endpoint, admin user management - lives in the [`crate::users`]
//! module.

pub mod extractor;
pub mod jwt;
pub mod password;

pub use extractor::{AdminUser, AuthUser};
pub use jwt::{Claims, JwtSecret, Role};
