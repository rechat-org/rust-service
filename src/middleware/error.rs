use crate::utils::ServerResponse;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MiddlewareError {
    #[error("Organization not found")]
    OrgNotFound,   
    #[error("Not found")]
    NotFound(String),
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

impl IntoResponse for MiddlewareError {
    fn into_response(self) -> Response {
        match self {
            MiddlewareError::NotFound(msg) => ServerResponse::not_found(msg),
            MiddlewareError::OrgNotFound => ServerResponse::unauthorized("Organization not found"),
            MiddlewareError::InvalidToken(msg) => ServerResponse::unauthorized(msg),
            MiddlewareError::MissingToken => ServerResponse::unauthorized("Authentication token is missing"),
            MiddlewareError::ExpiredToken => ServerResponse::unauthorized("Authentication token has expired"),
            MiddlewareError::InsufficientPermissions => ServerResponse::forbidden("Insufficient permissions to access this resource"),
            MiddlewareError::DatabaseError(msg) => ServerResponse::server_error(msg, "Database error occurred"),
            MiddlewareError::CacheError(msg) => ServerResponse::server_error(msg, "Cache error occurred"),
            MiddlewareError::StripeError(msg) => ServerResponse::server_error(msg, "Stripe error occurred"),
            MiddlewareError::ConfigError(msg) => ServerResponse::server_error(msg, "Stripe metadata parsing error occurred"),
            MiddlewareError::UsageLimitExceeded(msg) => ServerResponse::forbidden(msg),
        }
    }
}
