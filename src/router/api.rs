use crate::{handlers, state::AppState};
use axum::routing::delete;
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
                .nest(
                    "/organizations/:organization_id",
                    Router::new()
                        .route("/participants", post(handlers::create_participant))
                        .route("/participants/count", get(handlers::get_participants_count))

                        .route("/channels", post(handlers::create_channel))
                        .route("/channels", get(handlers::get_channels))
                        .route("/channels/:channel_id", get(handlers::get_channel_by_id))

                        .route("/messages/count", get(handlers::get_messages_count_for_current_month))
                        .route("/messages/:channel_id", get(handlers::get_messages_by_channel_id))
                        .route("/messages", post(handlers::create_message))

                        // admin routes they all need
                        .route("/users", get(handlers::get_users_in_org))
                        .route("/users/active", get(handlers::get_active_users))
                        .route("/users/count", get(handlers::get_users_in_org_count))
                        .route("/keys", get(handlers::get_api_keys))
                        .route("/keys/:key_id", delete(handlers::delete_api_key))
                        .route("/keys/count", get(handlers::get_api_key_count))
                        .route("/keys", post(handlers::create_api_key)),
                )
                .route(
                    "/organization_accounts/create",
                    post(handlers::create_user_and_organization),
                )
                .route("/organization_accounts/sign-in", post(handlers::sign_in)),
        )
}
