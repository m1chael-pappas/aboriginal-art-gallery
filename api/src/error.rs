//! Unified error type for every handler.
//!
//! Every fallible code path returns [`AppResult<T>`]. The [`IntoResponse`]
//! impl below picks the right HTTP status and serialises a uniform
//! `{"error": "..."}` body, so the FE never has to special-case "is this an
//! axum error or a domain error".

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

/// Every error the API can surface, tagged with the status it should map to.
///
/// `Database` and `Internal` are intentionally lumped together as 500s — they
/// indicate a bug or infrastructure failure rather than something the caller
/// did wrong, and we log them before returning a generic message so we don't
/// leak schema/internals.
#[derive(Debug, Error)]
pub enum AppError {
    /// Resource doesn't exist (or has been deleted) — HTTP 404.
    #[error("not found")]
    NotFound,

    /// Input failed semantic validation (empty string, bad lifespan, unknown
    /// FK, malformed GeoJSON, …) — HTTP 400.
    #[error("validation failed: {0}")]
    Validation(String),

    /// State conflict — duplicate unique key, FK still referenced, last-admin
    /// self-delete — HTTP 409.
    #[error("conflict: {0}")]
    Conflict(String),

    /// No bearer token, expired token, or tampered token — HTTP 401.
    #[error("unauthorized: {0}")]
    Unauthorized(String),

    /// Token is valid but the caller's role doesn't grant access — HTTP 403.
    #[error("forbidden: {0}")]
    Forbidden(String),

    /// Anything sqlx returns. Mapped to 500 except for `RowNotFound` which is
    /// 404 (a missing row is rarely a server bug).
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Anything wrapped via `anyhow!` — programmer errors, panics that were
    /// caught upstream, etc. Always 500.
    #[error("internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Validation(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::Conflict(_) => (StatusCode::CONFLICT, self.to_string()),
            AppError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::Forbidden(_) => (StatusCode::FORBIDDEN, self.to_string()),
            AppError::Database(sqlx::Error::RowNotFound) => {
                (StatusCode::NOT_FOUND, "not found".to_string())
            }
            AppError::Database(_) | AppError::Internal(_) => {
                tracing::error!(error = %self, "internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                )
            }
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

/// Result alias used throughout the crate so handlers can `?` straight to the
/// wire format.
pub type AppResult<T> = Result<T, AppError>;
