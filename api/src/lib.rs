//! Aboriginal Art Gallery API — library crate.
//!
//! The binary in `bin/main.rs` wires environment + tracing, then hands a
//! configured [`AppState`] to [`build_router`]. Everything else (handlers,
//! repos, models, the OpenAPI surface) lives in the submodules below.
//!
//! Bounded contexts:
//! - [`artists`] — biographies, lifespan, tribe affiliation
//! - [`artifacts`] — works, attributed to an artist
//! - [`tribes`] — peoples, language groups, PostGIS territory polygons
//! - [`users`] + [`auth`] — registration, login, JWT, role-based authorisation

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

/// Assembles every BC router, mounts Swagger UI + the raw OpenAPI document,
/// and stamps the shared [`AppState`] onto the tree.
///
/// The Swagger UI lives at `/docs` and the JSON spec at
/// `/api-docs/openapi.json` — the FE's `openapi-typescript` generator
/// consumes that URL.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .merge(health::router())
        .merge(artists::router())
        .merge(artifacts::router())
        .merge(tribes::router())
        .merge(users::router())
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state)
}
