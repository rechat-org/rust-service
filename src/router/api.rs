use crate::{handlers, state::AppState};
use axum::{
    routing::{get, post},
    Router,
};

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(handlers::health_check))
        .route("/ws/chat/:room_id", get(handlers::chat_ws_handler))
        .nest(
            "/api",
            Router::new() // User routes
                .route("/participant", post(handlers::create_participant))
                .route("/channel", post(handlers::create_channel))
                .route("/channel/:channel_id", get(handlers::get_channel_by_id)),
        )
}
