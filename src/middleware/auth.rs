use crate::middleware::error::AuthError;
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts, HeaderMap},
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: Uuid,
    pub email: String,
    pub exp: usize,
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub email: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Getting headers from parts
        let headers: HeaderMap = parts.headers.clone();

        // Extracting the token from Authorization header
        let auth_header = headers
            .get(AUTHORIZATION)
            .ok_or(AuthError::MissingToken)?
            .to_str()
            .map_err(|_| AuthError::InvalidToken("Invalid header value".to_string()))?;
        
        // Checking if it's a bearer token and extract the token part
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AuthError::InvalidToken("Invalid token format".to_string()))?;

        // Getting the secret from environment variable
        let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

        // Decoding and validate the token
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

        Ok(AuthUser {
            user_id: token_data.claims.user_id,
            email: token_data.claims.email,
        })
    }
}

pub fn validate_token(token: &str) -> Result<Claims, AuthError> {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| AuthError::InvalidToken(e.to_string()))
}
