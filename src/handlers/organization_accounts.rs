use crate::entities::sea_orm_active_enums::{OrganizationRole, OrganizationTier};
use crate::entities::{
    organization_members, organization_tiers, organizations, prelude::*, users,
};
use crate::state::AppState;
use crate::utils::ServerResponse;
use crate::utils::{hash_password_and_salt, verify_password};
use axum::Json;
use axum::{extract::State, response::IntoResponse};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct CreateUserResponse {
    id: Uuid,
    organization_id: Uuid,
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
    organization_id: Uuid, // org ID that the user belongs to
    user_id: Uuid,         // User ID
    sub: Uuid,             // User ID
    email: String,
    exp: i64, // Expiration time
    iat: i64, // Issued at
}

const TOKEN_EXPIRATION_HOURS: i64 = 120;

pub struct UserContext {
    pub user_id: Uuid,
    pub organization_id: Uuid,
    pub email: String,
    pub role: OrganizationRole,
}

// Helper function to get user context
pub async fn get_user_context(
    db: &DatabaseConnection,
    user_id: Uuid,
) -> Result<UserContext, DbErr> {
    // Find user and their organization in one query using joins
    let user_with_org = Users::find_by_id(user_id)
        .find_also_related(OrganizationMembers)
        .one(db)
        .await?;

    match user_with_org {
        Some((user, Some(org_member))) => Ok(UserContext {
            user_id: user.id,
            organization_id: org_member.organization_id,
            email: user.email,
            role: org_member.role,
        }),
        _ => Err(DbErr::RecordNotFound(
            "User or organization not found".to_string(),
        )),
    }
}

fn generate_jwt(
    user_id: Uuid,
    organization_id: Uuid,
    email: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let now = Utc::now();
    let expires_at = now + Duration::hours(TOKEN_EXPIRATION_HOURS);

    let claims = Claims {
        user_id,
        organization_id,
        sub: user_id,
        email: email.to_string(),
        exp: expires_at.timestamp(),
        iat: now.timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub async fn create_user_and_organization(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> impl IntoResponse {
    tracing::info!("executes: create_user");

    let db = &state.db.connection;
    let stripe = &state.stripe;
    let email = payload.email;
    let organization_name = payload.organization_name;
    let password_hash = match hash_password_and_salt(&payload.password) {
        Ok(hash) => hash,
        Err(err) => return ServerResponse::server_error(err, "Failed to hash password"),
    };

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

    let (customer_id, subscription_id, subscription_item_id) = match stripe
        .create_customer_with_subscription(&email, &organization_name)
        .await
    {
        Ok(ids) => ids,
        Err(err) => {
            let _ = txn.rollback().await;
            return ServerResponse::server_error(err, "Failed to create Stripe customer");
        }
    };

    // Create organization
    let new_org = organizations::ActiveModel {
        id: Set(org_id),
        name: Set(organization_name),
        created_at: Set(now),
        updated_at: Set(now),
        stripe_customer_id: Set(Some(customer_id)),
        stripe_subscription_id: Set(Some(subscription_id)),
        stripe_subscription_item_id: Set(Some(subscription_item_id)),
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

    // Create organization tier (Free by default)
    let new_org_tier = organization_tiers::ActiveModel {
        organization_id: Set(org_id),
        created_at: Set(now),
        updated_at: Set(now),
        tier: Default::default(),
        monthly_request_limit: Default::default(),
        current_month_usage: Default::default(),
        last_reset_at: Default::default(),
    };

    if let Err(err) = OrganizationTiers::insert(new_org_tier).exec(&txn).await {
        let _ = txn.rollback().await;
        return ServerResponse::server_error(err, "Failed to create organization tier");
    }

    // Generate JWT token
    let token = match generate_jwt(user_id, org_id.clone(), &email) {
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
        organization_id: org_id,
        email,
        token,
    };
    ServerResponse::created(response)
}

pub async fn sign_in(
    State(state): State<AppState>,
    Json(payload): Json<CreateSignInRequest>,
) -> impl IntoResponse {
    let db = &state.db.connection;

    // First find the user by email
    let user = match Users::find()
        .filter(users::Column::Email.eq(&payload.email))
        .one(db)
        .await
    {
        Ok(Some(user)) => user,
        Ok(None) => return ServerResponse::bad_request("Wrong credentials"),
        Err(err) => return ServerResponse::server_error(err, "Failed to check if user exists"),
    };

    // Verify password
    match verify_password(&payload.password, &user.password_hash) {
        Ok(_) => {}
        Err(_) => return ServerResponse::bad_request("Wrong credentials"),
    }

    // Get user context
    let context = match get_user_context(db, user.id).await {
        Ok(context) => context,
        Err(err) => return ServerResponse::server_error(err, "Failed to get user context"),
    };

    // Generate JWT with context info
    let token = match generate_jwt(context.user_id, context.organization_id, &context.email) {
        Ok(token) => token,
        Err(err) => return ServerResponse::server_error(err, "Failed to generate JWT token"),
    };

    ServerResponse::ok(CreateUserResponse {
        id: context.user_id,
        organization_id: context.organization_id,
        email: context.email,
        token,
    })
}
