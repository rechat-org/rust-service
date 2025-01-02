use std::str::FromStr;
use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn setup_logging() -> Result<(), Box<dyn std::error::Error>> {
    // Get log level from environment or default to INFO
    // This allows setting log level via RUST_LOG environment variable
    // e.g., RUST_LOG=debug cargo run
    let log_level = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "info".to_string());

    let env_filter = EnvFilter::from_str(&log_level)?;

    // Set up pretty console layer with colors
    // This layer is for human-readable output in the terminal
    // It includes colors and formatting for better readability
    let console_layer = fmt::layer()
        .with_target(true)                // Include target (module path)
        .with_thread_ids(true)            // Include thread IDs for concurrent debugging
        .with_thread_names(true)          // Include thread names if set
        .with_file(true)                   // Show source file name
        .with_line_number(true)           // Show line number in source
        .with_level(true)                 // Show log level (ERROR, WARN, etc.)
        .with_ansi(true)                  // Enable terminal colors
        .pretty();                        // Use pretty printing format

    // Set up JSON layer for structured logging
    // This layer outputs machine-readable JSON format
    // Perfect for log aggregation systems like Loki
    let json_layer = fmt::layer()
        .with_target(true)                // Include target (module path)
        .with_thread_ids(true)            // Include thread IDs
        .with_thread_names(true)          // Include thread names
        .with_file(true)                   // Include file name
        .with_line_number(true)           // Include line number
        .with_level(true)                 // Include log level
        .json()                                      // Format as JSON
        .with_current_span(true);            // Include tracing span context

    // Combine both layers with the environment filter
    // This creates a subscriber that will:
    // 1. Filter logs based on RUST_LOG
    // 2. Output pretty colored logs to the console
    // 3. Output JSON formatted logs for Loki
    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(json_layer)
        .init();

    Ok(())
}