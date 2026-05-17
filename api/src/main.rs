//! Binary entry point.
//!
//! Loads `.env`, initialises tracing, connects to Postgres, runs pending
//! migrations, derives the JWT signing keys, and serves the assembled
//! [`gallery_api::build_router`] on `127.0.0.1:8080`.

use std::net::SocketAddr;
use std::sync::Arc;

use gallery_api::{artists::PgArtistStore, auth::JwtSecret, build_router, state::AppState};
use sqlx::postgres::PgPoolOptions;
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

    let cors = CorsLayer::permissive();

    let app = build_router(AppState {
        pool,
        jwt_secret,
        artists,
    })
    .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("listening on http://{addr}");

    axum::serve(listener, app).await?;

    Ok(())
}
