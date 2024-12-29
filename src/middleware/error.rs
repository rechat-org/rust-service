use crate::utils::ServerResponse;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Organization not found")]
    OrgNotFound,
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    #[error("Missing authentication token")]
    MissingToken,
    #[error("Token has expired")]
    ExpiredToken,
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Redis error: {0}")]
    CacheError(String),
    #[error("Stripe error: {0}")]
    StripeError(String),
    #[error("Stripe Metadata config error")]
    ConfigError(String),
    #[error("Usage limit exceeded: {0}")]
    UsageLimitExceeded(String),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        match self {
            AuthError::OrgNotFound => ServerResponse::unauthorized("Organization not found"),
            AuthError::InvalidToken(msg) => ServerResponse::unauthorized(msg),
            AuthError::MissingToken => ServerResponse::unauthorized("Authentication token is missing"),
            AuthError::ExpiredToken => ServerResponse::unauthorized("Authentication token has expired"),
            AuthError::InsufficientPermissions => ServerResponse::forbidden("Insufficient permissions to access this resource"),
            AuthError::DatabaseError(msg) => ServerResponse::server_error(msg, "Database error occurred"),
            AuthError::CacheError(msg) => ServerResponse::server_error(msg, "Cache error occurred"),
            AuthError::StripeError(msg) => ServerResponse::server_error(msg, "Stripe error occurred"),
            AuthError::ConfigError(msg) => ServerResponse::server_error(msg, "Stripe metadata parsing error occurred"),
            AuthError::UsageLimitExceeded(msg) => ServerResponse::forbidden(msg),
        }
    }
}
