use crate::middleware::error::MiddlewareError;
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

#[derive(Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub email: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = MiddlewareError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Getting headers from parts
        let headers: HeaderMap = parts.headers.clone();

        // Extracting the token from Authorization header
        let auth_header = headers
            .get(AUTHORIZATION)
            .ok_or(MiddlewareError::MissingToken)?
            .to_str()
            .map_err(|_| MiddlewareError::InvalidToken("Invalid header value".to_string()))?;

        // Checking if it's a bearer token and extract the token part
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(MiddlewareError::InvalidToken("Invalid token format".to_string()))?;

        // Getting the secret from environment variable
        let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

        // Decoding and validate the token
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| MiddlewareError::InvalidToken(e.to_string()))?;

        Ok(AuthUser {
            user_id: token_data.claims.user_id,
            email: token_data.claims.email,
        })
    }
}
