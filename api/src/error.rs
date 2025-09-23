//! API error handling

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use crate::models::ErrorResponse;
use thiserror::Error;

/// API error types
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Internal server error: {0}")]
    Internal(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Rate limit exceeded")]
    RateLimited,
    
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

impl ApiError {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Validation(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApiError::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            ApiError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
        }
    }
    
    /// Get the error code string
    pub fn error_code(&self) -> &'static str {
        match self {
            ApiError::BadRequest(_) => "BAD_REQUEST",
            ApiError::NotFound(_) => "NOT_FOUND",
            ApiError::Internal(_) => "INTERNAL_ERROR",
            ApiError::Validation(_) => "VALIDATION_ERROR",
            ApiError::Unauthorized(_) => "UNAUTHORIZED",
            ApiError::RateLimited => "RATE_LIMITED",
            ApiError::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE",
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_response = ErrorResponse {
            code: self.error_code().to_string(),
            message: self.to_string(),
            details: None,
            timestamp: chrono::Utc::now(),
        };

        (status, Json(error_response)).into_response()
    }
}

/// Convert from various error types
impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError::Internal(err.to_string())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::BadRequest(format!("JSON parsing error: {}", err))
    }
}

impl From<hex::FromHexError> for ApiError {
    fn from(err: hex::FromHexError) -> Self {
        ApiError::BadRequest(format!("Invalid hex format: {}", err))
    }
}