use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod artifacts;
pub mod artists;
pub mod auth;
pub mod error;
pub mod health;
pub mod openapi;
pub mod state;
pub mod tribes;
pub mod users;

use openapi::ApiDoc;
use state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .merge(health::router())
        .merge(artists::router())
        .merge(artifacts::router())
        .merge(tribes::router())
        .merge(users::router())
        // Browsable docs at /docs, raw spec at /api-docs/openapi.json (the
        // openapi-typescript generator on the FE consumes the JSON URL).
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state)
}
