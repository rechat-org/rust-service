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
            Router::new()

                .route("/participants", post(handlers::create_participant))
                .route("/participants/count", get(handlers::get_participants_count))
                .route("/channels", get(handlers::get_channels))
                .route("/channels", post(handlers::create_channel))
                .route("/channels/:channel_id", get(handlers::get_channel_by_id))
                .route("/messages/:channel_id", get(handlers::get_messages_by_channel_id))
                .route("/messages", post(handlers::create_message))

                .route("/organization_accounts/create", post(handlers::create_user_and_organization)),
        )
}
