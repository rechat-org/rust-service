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

                .route("/organization_accounts/create", post(handlers::create_user_and_organization))
                .route("/organization_accounts/sign-in", post(handlers::sign_in))
                
                .route("/organizations/users", get(handlers::get_users_in_org))
                .route("/organizations/users/count", get(handlers::get_users_in_org_count))
                .route("/organizations/generate-api-key", post(handlers::create_api_key))
                .route("/organizations/:organization_id/keys", get(handlers::get_api_keys))
                .route("/organizations/:organization_id/keys/count", get(handlers::get_api_key_count))
                .route("/organizations/:organization_id/keys", post(handlers::create_api_key))

            ,
        )
}
