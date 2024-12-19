use crate::entities::{prelude::*, users};
use crate::utils::ServerResponse;
use axum::Json;
use axum::{extract::State, response::IntoResponse};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use sea_orm::*;
use serde::{Serialize,Deserialize};
use uuid::Uuid;

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct CreateUserResponse {
    id: Uuid,
    email: String,
    token: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct Claims {
    sub: Uuid, // User ID
    email: String,
    exp: i64, // Expiration time
    iat: i64, // Issued at
}

const JWT_SECRET: &[u8] = b"your-secret-key"; // In production, this should come from environment variables
const TOKEN_EXPIRATION_HOURS: i64 = 120;

fn generate_jwt(user_id: Uuid, email: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let expires_at = now + Duration::hours(TOKEN_EXPIRATION_HOURS);

    let claims = Claims {
        sub: user_id,
        email: email.to_string(),
        exp: expires_at.timestamp(),
        iat: now.timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> impl IntoResponse {
    tracing::info!("executes: create_user");

    let db = &state.db.connection;
    let email = payload.email;
    let password_hash = payload.password;

    match Users::find()
        .filter(users::Column::Email.eq(&email))
        .one(db)
        .await
    {
        Ok(None) => {}
        Ok(Some(_)) => return ServerResponse::bad_request("User already exists"),
        Err(err) => return ServerResponse::server_error(err, "Failed to check if user exists"),
    };

    let user_id = Uuid::new_v4();
    let now = chrono::Utc::now().naive_utc();
    let new_user = users::ActiveModel {
        id: Set(user_id),
        email: Set(email.clone()),
        password_hash: Set(password_hash),
        created_at: Set(now),
        updated_at: Set(now),
    };

    match Users::insert(new_user).exec(db).await {
        Ok(_) => match generate_jwt(user_id, &email) {
            Ok(token) => {
                let response = CreateUserResponse {
                    id: user_id,
                    email,
                    token,
                };
                ServerResponse::created(response)
            }
            Err(err) => ServerResponse::server_error(err, "Failed to generate JWT token"),
        },
        Err(err) => ServerResponse::server_error(err, "Failed to create user"),
    }
}
