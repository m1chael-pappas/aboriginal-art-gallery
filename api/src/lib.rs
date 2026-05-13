use axum::Router;

pub mod artifacts;
pub mod artists;
pub mod auth;
pub mod error;
pub mod health;
pub mod state;
pub mod tribes;
pub mod users;

use state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .merge(health::router())
        .merge(artists::router())
        .merge(artifacts::router())
        .merge(tribes::router())
        .merge(users::router())
        .with_state(state)
}
