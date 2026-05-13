//! Shared request-scope state — handed to every handler via `State<AppState>`.

use axum::extract::FromRef;
use sqlx::PgPool;

use crate::auth::JwtSecret;

/// Application state cloned into each request's extractor context.
///
/// `pool` is a `PgPool`, which is internally `Arc`-shaped, so cloning is
/// cheap. `jwt_secret` likewise wraps the encoding + decoding keys in an
/// `Arc`-style handle. The whole struct is `#[derive(Clone)]` so axum can
/// thread it through the router.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: JwtSecret,
}

/// Lets axum's [`FromRequestParts`](axum::extract::FromRequestParts)
/// extractors (like [`AuthUser`](crate::auth::AuthUser)) pull just the
/// [`JwtSecret`] sub-state out of [`AppState`] instead of demanding the
/// entire struct.
impl FromRef<AppState> for JwtSecret {
    fn from_ref(state: &AppState) -> Self {
        state.jwt_secret.clone()
    }
}
