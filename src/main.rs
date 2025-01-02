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
use std::str::FromStr;
use time::macros::format_description;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn setup_logging() -> Result<(), Box<dyn std::error::Error>> {
    // Get log level from environment or default to INFO
    let log_level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

    let env_filter = EnvFilter::from_str(&log_level)?;

    // Set up JSON formatting for structured logging
    let formatting_layer = fmt::layer()
        .with_target(true) // Include target (module path)
        .with_thread_ids(true) // Include thread IDs
        .with_thread_names(true) // Include thread names
        .with_file(true) // Include file name
        .with_line_number(true) // Include line number
        .with_level(true) // Include log level
        .json() // Use JSON formatter
        .with_current_span(true); // Include span context

    // Combine filter and formatting
    tracing_subscriber::registry()
        .with(env_filter)
        .with(formatting_layer)
        .init();

    Ok(())
}

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
    println!("🚀 Server running on http://{}", addr);

    // Create and start the server
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    // Start the server
    axum::serve(listener, app).await.unwrap();
}
