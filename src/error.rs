use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::fmt;
use tracing::error;

#[derive(Debug)]
pub enum ApiError {
    SystemNotFound(String),
    #[allow(dead_code)]
    InvalidInput(String),
    DatabaseError(sqlx::Error),
    InternalError(anyhow::Error),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_id: Option<String>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match self {
            ApiError::SystemNotFound(name) => (
                StatusCode::NOT_FOUND,
                "system_not_found",
                format!("System '{}' was not found", name),
            ),
            ApiError::InvalidInput(msg) => (
                StatusCode::BAD_REQUEST,
                "invalid_input",
                msg,
            ),
            ApiError::DatabaseError(ref e) => {
                error!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "database_error",
                    "A database error occurred".to_string(),
                )
            },
            ApiError::InternalError(ref e) => {
                error!("Internal error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "An internal server error occurred".to_string(),
                )
            },
        };

        let response = ErrorResponse {
            error: error_type.to_string(),
            message,
            details: None,
            request_id: None, // TODO: Extract from request context if needed
        };

        (status, Json(response)).into_response()
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::SystemNotFound(name) => write!(f, "System not found: {}", name),
            ApiError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            ApiError::DatabaseError(e) => write!(f, "Database error: {}", e),
            ApiError::InternalError(e) => write!(f, "Internal error: {}", e),
        }
    }
}

impl std::error::Error for ApiError {}

// Conversion implementations
impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        ApiError::DatabaseError(err)
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError::InternalError(err)
    }
}

// Result type alias for convenience
pub type ApiResult<T> = Result<T, ApiError>; 