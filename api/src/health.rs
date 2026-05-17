//! Health check endpoint.
//!
//! Exposes `GET /health`, used by container orchestrators / probes and by
//! local smoke-testing. Pings the DB so the check is meaningful - a process
//! that can't reach Postgres reports unhealthy even if it's still listening.

use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;

use crate::{error::AppResult, state::AppState};

/// Health probe response payload - flat `{"status": "ok", "db": "ok"}`.
#[derive(Serialize)]
pub struct Health {
    status: &'static str,
    db: &'static str,
}

/// Build the `/health` sub-router. Merged into the main router by
/// [`crate::build_router`].
pub fn router() -> Router<AppState> {
    Router::new().route("/health", get(health))
}

async fn health(State(state): State<AppState>) -> AppResult<Json<Health>> {
    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(Health {
        status: "ok",
        db: "ok",
    }))
}
