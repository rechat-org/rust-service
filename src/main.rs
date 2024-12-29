#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]

mod config;
mod entities;
mod handlers;
mod middleware;
mod router;
mod state;
mod utils;

use crate::config::StripeClient;
use config::{AppConfig, Database, RedisConfig, RedisStore};
use router::api_router;
use state::AppState;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    // Load .env file
    dotenvy::dotenv().ok();

    // Initialize logging
    tracing_subscriber::fmt::init();

    let app_config = AppConfig::new();

    let database = Database::new()
        .await
        .expect("Failed to connect to database");
    // Verify database connection
    database.ping().await.expect("Could not ping database");

    // Initialize Redis
    let redis_config = RedisConfig::new();
    let redis_store = RedisStore::new(redis_config).expect("Failed to create Redis client.");
    // Verify Redis connection
    redis_store.ping().await.expect("Could not ping Redis");

    // Initialize Stripe
    let stripe = StripeClient::new().expect("Failed to create Stripe client.");

    // Create app state
    let state = AppState::new(database, redis_store, stripe);

    // Build our application with routes
    let app = api_router().with_state(state);

    let addr = app_config.addr();
    println!("🚀🚀 Server running on http://{}", addr);

    // Create and start the server
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    // Start the server
    axum::serve(listener, app).await.unwrap();
}
