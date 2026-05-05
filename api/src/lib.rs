use axum::Router;

pub mod artifacts;
pub mod artists;
pub mod error;
pub mod health;
pub mod state;
pub mod tribes;

use state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .merge(health::router())
        .merge(artists::router())
        .merge(artifacts::router())
        .merge(tribes::router())
        .with_state(state)
}
