use crate::{handlers, state::AppState};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tungstenite::protocol::frame::coding::CloseCode;
use tungstenite::protocol::CloseFrame;

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(handlers::health_check))
        .route("/ws/chat/:room_id", get(handlers::chat_ws_handler))
        // .nest("/api", api_v1_router())
}

// // v1 API routes
// fn api_v1_router() -> Router<AppState> {
//     Router::new().nest(
//         "/v1",
//         Router::new(), // User routes
//                        // .route("/users", get(handlers::get_users))
//                        // .route("/users", post(handlers::create_user))
//                        // You can add more route groups here
//     )
// }
