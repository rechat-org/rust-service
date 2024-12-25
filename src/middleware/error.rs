use crate::utils::ServerResponse;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Organization ID is missing")]
    MissingOrgId,
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
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        match self {
            AuthError::OrgNotFound => ServerResponse::unauthorized("Organization not found"),
            AuthError::MissingOrgId => ServerResponse::unauthorized("Organization ID is missing"),
            AuthError::InvalidToken(msg) => ServerResponse::unauthorized(msg),
            AuthError::MissingToken => ServerResponse::unauthorized("Authentication token is missing"),
            AuthError::ExpiredToken => ServerResponse::unauthorized("Authentication token has expired"),
            AuthError::InsufficientPermissions => ServerResponse::forbidden("Insufficient permissions to access this resource"),
            AuthError::DatabaseError(msg) => ServerResponse::server_error(msg, "Database error occurred"),
        }
    }
}
