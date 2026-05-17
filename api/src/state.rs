//! Shared request-scope state - handed to every handler via `State<AppState>`.

use std::sync::Arc;

use axum::extract::FromRef;
use sqlx::PgPool;

use crate::artifacts::ArtifactStore;
use crate::artists::ArtistStore;
use crate::auth::JwtSecret;
use crate::tribes::TribeStore;
use crate::users::UserStore;

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
    /// Artists persistence behind a trait object, so the Artists handlers
    /// depend on the [`ArtistStore`] abstraction rather than a concrete
    /// database type. This is the "program to an interface, inject the
    /// implementation" seam: production wires a `PgArtistStore`, unit tests
    /// wire an in-memory fake. (Reference pattern; the other contexts still
    /// call their concrete repos directly.)
    pub artists: Arc<dyn ArtistStore>,
    /// Artifacts persistence behind the [`ArtifactStore`] trait.
    pub artifacts: Arc<dyn ArtifactStore>,
    /// Tribes persistence (CRUD + territory + spatial search) behind the
    /// [`TribeStore`] trait.
    pub tribes: Arc<dyn TribeStore>,
    /// Users persistence behind the [`UserStore`] trait. Password hashing
    /// and JWT issuance stay in the handler, not the store.
    pub users: Arc<dyn UserStore>,
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
