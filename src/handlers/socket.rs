use crate::state::AppState;
use axum::extract::ws::{Message, WebSocket};
use axum::{
    extract::{Path, State, WebSocketUpgrade},
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    user_id: String,
    room_id: String,
    content: String,
    timestamp: u64,
}

pub async fn chat_ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(room_id): Path<String>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket_connection(socket, state, room_id))
}

async fn handle_socket_connection(socket: WebSocket, state: AppState, room_id: String) {
    let (mut sender, mut receiver) = socket.split();

    // Get Redis connection
    let redis = state.redis.clone();
    let mut redis_conn = match redis.client.get_async_connection().await {
        Ok(conn) => conn,
        Err(e) => {
            error!("Redis connection failed: {}", e);
            return;
        }
    };

    // Create Redis PubSub
    let mut pubsub = redis_conn.into_pubsub();
    let channel = format!("chat:{}", room_id);

    // Subscribe to room channel
    if let Err(e) = pubsub.subscribe(&channel).await {
        error!("Redis subscribe failed: {}", e);
        return;
    }

    info!("Client connected to room: {}", room_id);

    // Handle incoming WebSocket messages
    let redis_publisher = redis.client.clone();
    let ws_room_id = room_id.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(result) = receiver.next().await {
            match result {
                Ok(Message::Text(text)) => {
                    // Parse the incoming message
                    match serde_json::from_str::<ChatMessage>(&text) {
                        Ok(mut msg) => {
                            // Add server-side timestamp
                            msg.timestamp = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs();
                            msg.room_id = ws_room_id.clone();

                            // Publish to Redis
                            if let Ok(json) = serde_json::to_string(&msg) {
                                println!("Publishing message: {}", json);
                                let mut conn = redis_publisher
                                    .get_async_connection()
                                    .await
                                    .expect("Failed to get Redis connection");

                                // Publish to the channel with explicit type annotation
                                match conn.publish::<_, _, ()>(&channel, json).await {
                                    Ok(_) => println!("Successfully published to {}", channel),
                                    Err(e) => error!("Failed to publish to Redis: {}", e),
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to parse message: {}", e);
                        }
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    // Handle messages from Redis and send them to WebSocket
    let mut send_task = tokio::spawn(async move {
        let mut pubsub_stream = pubsub.on_message();

        while let Some(msg) = pubsub_stream.next().await {
            let payload: String = msg.get_payload().unwrap_or_default();

            if let Err(e) = sender.send(Message::Text(payload)).await {
                error!("Failed to send WebSocket message: {}", e);
                break;
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    info!("Client disconnected from room: {}", room_id);
}
