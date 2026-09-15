//! Aboriginal Art Gallery API - library crate.
//!
//! The binary in `main.rs` wires environment + tracing, then hands a
//! configured [`AppState`] to [`build_router`]. Everything else (handlers,
//! repos, models, the OpenAPI surface) lives in the submodules below.
//!
//! Bounded contexts:
//! - [`artists`] - biographies, lifespan, tribe affiliation
//! - [`artifacts`] - works, attributed to an artist
//! - [`tribes`] - peoples, language groups, PostGIS territory polygons
//! - [`users`] + [`auth`] - registration, login, JWT, role-based authorisation

use std::net::SocketAddr;

use anyhow::Context;
use axum::{Router, routing::get};
use axum_prometheus::{EndpointLabel, PrometheusMetricLayerBuilder};
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
/// `/api-docs/openapi.json` - the FE's `openapi-typescript` generator
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

/// Default listen address when `BIND_ADDR` is unset. Loopback, so a dev
/// `cargo run` is not exposed on the LAN; containers set `0.0.0.0:8080`.
pub const DEFAULT_BIND_ADDR: &str = "127.0.0.1:8080";

/// Resolves the listen address from `BIND_ADDR`, falling back to
/// [`DEFAULT_BIND_ADDR`]. Shared by the server and the `healthcheck` binary
/// so both always agree on the port.
pub fn bind_addr() -> anyhow::Result<SocketAddr> {
    let raw = std::env::var("BIND_ADDR").unwrap_or_else(|_| DEFAULT_BIND_ADDR.to_string());
    raw.parse()
        .with_context(|| format!("BIND_ADDR `{raw}` is not a valid socket address"))
}

/// `endpoint` label for requests that matched no route.
pub const UNMATCHED_ENDPOINT: &str = "unmatched";

/// Wraps `router` in Prometheus HTTP metrics and mounts the `/metrics`
/// scrape endpoint.
///
/// Emits `axum_http_requests_total`, `axum_http_requests_duration_seconds`
/// and `axum_http_requests_pending`, labelled by matched route, method and
/// status. Requests that match no route share the [`UNMATCHED_ENDPOINT`]
/// label, so probing random URLs cannot create unbounded label values.
/// `/metrics` and `/health` are excluded so scrapes and probes do not inflate
/// request rate or skew latency.
///
/// Installs the process-wide `metrics` recorder, which can only happen once
/// per process. Call it from `main`, never from [`build_router`], because
/// the integration tests build a router per test.
pub fn with_metrics(router: Router) -> Router {
    let (layer, handle) = PrometheusMetricLayerBuilder::new()
        .with_endpoint_label_type(EndpointLabel::MatchedPathWithFallbackFn(|_| {
            UNMATCHED_ENDPOINT.to_string()
        }))
        .with_ignore_patterns(&["/metrics", "/health"])
        .with_default_metrics()
        .build_pair();

    router
        .route("/metrics", get(move || async move { handle.render() }))
        .layer(layer)
}
