use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;

use crate::{error::AppResult, state::AppState};

#[derive(Serialize)]
pub struct Health {
    status: &'static str,
    db: &'static str,
}

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
