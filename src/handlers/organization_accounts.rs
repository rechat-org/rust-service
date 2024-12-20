use crate::entities::sea_orm_active_enums::OrganizationRole;
use crate::entities::{organization_members, organizations, prelude::*, users};
use crate::utils::{ServerResponse};
use axum::Json;
use axum::{extract::State, response::IntoResponse};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AppState;
use crate::utils::hash_password_and_salt;

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
    organization_name: String,
}
#[derive(Debug, Deserialize)]
pub struct CreateSignInRequest {
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

pub async fn create_user_and_organization(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> impl IntoResponse {
    tracing::info!("executes: create_user");

    let db = &state.db.connection;
    let email = payload.email;
    let password_hash = hash_password_and_salt(&payload.password).unwrap();

    // Start a transaction
    let txn = match db.begin().await {
        Ok(txn) => txn,
        Err(err) => return ServerResponse::server_error(err, "Failed to start transaction"),
    };

    // Check if user exists
    match Users::find()
        .filter(users::Column::Email.eq(&email))
        .one(&txn)
        .await
    {
        Ok(None) => {}
        Ok(Some(_)) => return ServerResponse::bad_request("User already exists"),
        Err(err) => return ServerResponse::server_error(err, "Failed to check if user exists"),
    };

    let user_id = Uuid::new_v4();
    let org_id = Uuid::new_v4();
    let now = Utc::now().naive_utc();

    // Create organization
    let new_org = organizations::ActiveModel {
        id: Set(org_id),
        name: Set(payload.organization_name),
        created_at: Set(now),
        updated_at: Set(now),
    };

    if let Err(err) = Organizations::insert(new_org).exec(&txn).await {
        let _ = txn.rollback().await;
        return ServerResponse::server_error(err, "Failed to create organization");
    }

    // Create user
    let new_user = users::ActiveModel {
        id: Set(user_id),
        email: Set(email.clone()),
        password_hash: Set(password_hash),
        created_at: Set(now),
        updated_at: Set(now),
    };

    if let Err(err) = Users::insert(new_user).exec(&txn).await {
        let _ = txn.rollback().await;
        return ServerResponse::server_error(err, "Failed to create user");
    }

    // Create organization member
    let new_org_member = organization_members::ActiveModel {
        user_id: Set(user_id),
        organization_id: Set(org_id),
        role: Set(OrganizationRole::Owner),
        created_at: Set(now),
        updated_at: Set(now),
    };

    if let Err(err) = OrganizationMembers::insert(new_org_member).exec(&txn).await {
        let _ = txn.rollback().await;
        return ServerResponse::server_error(err, "Failed to create organization member");
    }

    // Generate JWT token
    let token = match generate_jwt(user_id, &email) {
        Ok(token) => token,
        Err(err) => {
            let _ = txn.rollback().await;
            return ServerResponse::server_error(err, "Failed to generate JWT token");
        }
    };

    // Commit the transaction: by doing this,
    // we ensure that the of operations are grouped -
    // either all operations succeed, or none of them do.
    if let Err(err) = txn.commit().await {
        return ServerResponse::server_error(err, "Failed to commit transaction");
    }

    let response = CreateUserResponse {
        id: user_id,
        email,
        token,
    };
    ServerResponse::created(response)
}

pub async fn sign_in(
    State(state): State<AppState>,
    Json(payload): Json<CreateSignInRequest>,
) -> impl IntoResponse {
    tracing::info!("executes: create_user");

    let db = &state.db.connection;
    let email = payload.email;
    let password_hash = hash_password_and_salt(&payload.password).unwrap();

    // Check if user exists
    match Users::find()
        .filter(users::Column::Email.eq(&email))
        .filter(users::Column::PasswordHash.eq(password_hash))
        .one(db)
        .await
    {
        Ok(None) => ServerResponse::bad_request("Wrong credentials"),
        Ok(Some(user)) => {
            let token = match generate_jwt(user.id, &email) {
                Ok(token) => token,
                Err(err) => {
                    return ServerResponse::server_error(err, "Failed to generate JWT token");
                }
            };
            ServerResponse::ok({
                CreateUserResponse {
                    id: user.id,
                    email,
                    token,
                }
            })
        }
        Err(err) => ServerResponse::server_error(err, "Failed to check if user exists"),
    }
}
