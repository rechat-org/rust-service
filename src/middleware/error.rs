use crate::utils::ServerResponse;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
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

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::OrgNotFound => ServerResponse::unauthorized("Organization not found"),
            AppError::InvalidToken(msg) => ServerResponse::unauthorized(msg),
            AppError::MissingToken => ServerResponse::unauthorized("Authentication token is missing"),
            AppError::ExpiredToken => ServerResponse::unauthorized("Authentication token has expired"),
            AppError::InsufficientPermissions => ServerResponse::forbidden("Insufficient permissions to access this resource"),
            AppError::DatabaseError(msg) => ServerResponse::server_error(msg, "Database error occurred"),
            AppError::CacheError(msg) => ServerResponse::server_error(msg, "Cache error occurred"),
            AppError::StripeError(msg) => ServerResponse::server_error(msg, "Stripe error occurred"),
            AppError::ConfigError(msg) => ServerResponse::server_error(msg, "Stripe metadata parsing error occurred"),
            AppError::UsageLimitExceeded(msg) => ServerResponse::forbidden(msg),
        }
    }
}
