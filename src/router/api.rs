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
                .route("/participants", post(handlers::create_participant))
                .route("/participants/count", post(handlers::get_participants_count))
                .route("/channels", get(handlers::get_channels))
                .route("/channels", post(handlers::create_channel))
                .route("/channels/:channel_id", get(handlers::get_channel_by_id))
                .route("/messages/", get(handlers::get_channel_by_id))
                .route("/messages/", post(handlers::get_channel_by_id)),
        )
}
