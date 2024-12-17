use crate::entities::{participant, prelude::*};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct CreateParticipantResponse {
    id: Uuid,
    name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateParticipantRequest {
    name: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    message: String,
}

pub async fn create_participant(
    State(state): State<AppState>,
    Json(payload): Json<CreateParticipantRequest>,
) -> impl IntoResponse {
    tracing::info!("executes: create_participant");

    let db = &state.db.connection;

    let name = payload.name;

    let new_participant = participant::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(name.clone()),
        created_at: Set(chrono::Utc::now().naive_utc()),
        updated_at: Set(chrono::Utc::now().naive_utc()),
    };

    match Participant::insert(new_participant).exec(db).await {
        Ok(participant) => {
            let response = CreateParticipantResponse {
                id: participant.last_insert_id,
                name,
            };
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(err) => {
            tracing::error!("Failed to create participant: {:?}", err);
            let error_response = ErrorResponse {
                message: "Failed to create participant".to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
        }
    }
}
