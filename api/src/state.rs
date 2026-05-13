use axum::extract::FromRef;
use sqlx::PgPool;

use crate::auth::JwtSecret;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: JwtSecret,
}

// Let axum's `FromRequestParts` extractors (like `AuthUser`) pull just the
// `JwtSecret` slice out of `AppState`, instead of demanding the whole state.
impl FromRef<AppState> for JwtSecret {
    fn from_ref(state: &AppState) -> Self {
        state.jwt_secret.clone()
    }
}
