//! Binary entry point.
//!
//! Loads `.env`, initialises tracing, connects to Postgres, runs pending
//! migrations, derives the JWT signing keys, and serves the assembled
//! [`gallery_api::build_router`] (wrapped in [`gallery_api::with_metrics`])
//! on [`gallery_api::bind_addr`] until SIGINT or SIGTERM.

use std::sync::Arc;

use gallery_api::{
    artifacts::PgArtifactStore, artists::PgArtistStore, auth::JwtSecret, bind_addr, build_router,
    state::AppState, tribes::PgTribeStore, users::PgUserStore, with_metrics,
};
use sqlx::postgres::PgPoolOptions;
use tokio::signal::unix::{SignalKind, signal};
use tower_http::cors::CorsLayer;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set (see .env.example)");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let jwt_secret = JwtSecret::from_env()?;

    let artists = Arc::new(PgArtistStore::new(pool.clone()));
    let artifacts = Arc::new(PgArtifactStore::new(pool.clone()));
    let tribes = Arc::new(PgTribeStore::new(pool.clone()));
    let users = Arc::new(PgUserStore::new(pool.clone()));

    let cors = CorsLayer::permissive();

    let app = with_metrics(build_router(AppState {
        pool,
        jwt_secret,
        artists,
        artifacts,
        tribes,
        users,
    }))
    .layer(cors);

    let addr = bind_addr()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("listening on http://{addr}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

/// Resolves on SIGINT (Ctrl+C) or SIGTERM (`docker stop`), letting in-flight
/// requests finish instead of being killed after Docker's grace period.
async fn shutdown_signal() {
    let mut terminate = signal(SignalKind::terminate()).expect("install SIGTERM handler");
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = terminate.recv() => {}
    }
    tracing::info!("shutdown signal received, draining connections");
}
