
use axum::{
    routing::{get, post},
    Router,
};
use crate::{handlers, state::AppState};

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(handlers::health_check))
        .nest("/api", api_v1_router())
}

// v1 API routes
fn api_v1_router() -> Router<AppState> {
    Router::new()
        .nest("/v1", Router::new()
            // User routes
            // .route("/users", get(handlers::get_users))
            // .route("/users", post(handlers::create_user))
              // You can add more route groups here
        )
}