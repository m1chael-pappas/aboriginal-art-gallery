//! Axum request extractors that gate protected routes.
//!
//! Handlers accept these as parameters; axum runs them during request
//! parsing, so the handler body only runs once authorisation has passed.

use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};

use super::{Claims, JwtSecret, Role};
use crate::error::AppError;

/// Authenticated request — the caller presented a valid (non-expired,
/// correctly-signed) bearer token. Use this when *any* logged-in user may
/// hit the endpoint; use [`AdminUser`] when the route is restricted
/// further.
pub struct AuthUser {
    pub claims: Claims,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    JwtSecret: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("missing Authorization header".into()))?;

        let token = header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Unauthorized("expected 'Bearer <token>'".into()))?;

        let secret = JwtSecret::from_ref(state);
        let claims = secret.decode(token)?;

        Ok(AuthUser { claims })
    }
}

/// Stricter variant: a valid token *and* `role == Admin`.
///
/// Plain `User` tokens get a 403; missing or invalid tokens still get a 401
/// (via the inner [`AuthUser`] extractor) so clients can tell "you're not
/// logged in" apart from "you're logged in but not authorised".
pub struct AdminUser {
    pub claims: Claims,
}

impl<S> FromRequestParts<S> for AdminUser
where
    S: Send + Sync,
    JwtSecret: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let AuthUser { claims } = AuthUser::from_request_parts(parts, state).await?;
        if claims.role != Role::Admin {
            return Err(AppError::Forbidden("admin role required".into()));
        }
        Ok(AdminUser { claims })
    }
}
