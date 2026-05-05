use axum::Router;

pub mod artists;
pub mod error;
pub mod health;
pub mod state;

use state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .merge(health::router())
        .merge(artists::router())
        .with_state(state)
}
