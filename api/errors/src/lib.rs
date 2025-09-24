//! CC Chain API Error Types
//!
//! This module provides comprehensive error types and error handling utilities
//! for the CC Chain API, including HTTP status code mapping and user-friendly
//! error messages.

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Main API error type
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ApiError {
    #[error("Bad Request: {message}")]
    BadRequest { message: String },
    
    #[error("Unauthorized: {message}")]
    Unauthorized { message: String },
    
    #[error("Forbidden: {message}")]
    Forbidden { message: String },
    
    #[error("Not Found: {message}")]
    NotFound { message: String },
    
    #[error("Method Not Allowed: {method} is not allowed for this endpoint")]
    MethodNotAllowed { method: String },
    
    #[error("Conflict: {message}")]
    Conflict { message: String },
    
    #[error("Unprocessable Entity: {message}")]
    UnprocessableEntity { message: String },
    
    #[error("Rate Limited: {message}")]
    RateLimited { message: String },
    
    #[error("Internal Server Error: {message}")]
    InternalServerError { message: String },
    
    #[error("Service Unavailable: {message}")]
    ServiceUnavailable { message: String },
    
    #[error("Gateway Timeout: {message}")]
    GatewayTimeout { message: String },
}

/// Validation error for request data
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation error in field '{}': {}", self.field, self.message)
    }
}

/// Transaction error types
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum TransactionError {
    #[error("Invalid transaction format")]
    InvalidFormat,
    
    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: u64, available: u64 },
    
    #[error("Invalid signature")]
    InvalidSignature,
    
    #[error("Transaction already exists: {hash}")]
    AlreadyExists { hash: String },
    
    #[error("Transaction not found: {hash}")]
    NotFound { hash: String },
    
    #[error("Transaction failed: {reason}")]
    ExecutionFailed { reason: String },
    
    #[error("Gas limit exceeded: used {used}, limit {limit}")]
    GasLimitExceeded { used: u64, limit: u64 },
    
    #[error("Invalid nonce: expected {expected}, got {actual}")]
    InvalidNonce { expected: u64, actual: u64 },
}

/// Block error types
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum BlockError {
    #[error("Block not found: {identifier}")]
    NotFound { identifier: String },
    
    #[error("Invalid block hash: {hash}")]
    InvalidHash { hash: String },
    
    #[error("Invalid block height: {height}")]
    InvalidHeight { height: u64 },
    
    #[error("Block validation failed: {reason}")]
    ValidationFailed { reason: String },
}

/// Network error types
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum NetworkError {
    #[error("Connection failed: {reason}")]
    ConnectionFailed { reason: String },
    
    #[error("Network timeout")]
    Timeout,
    
    #[error("Peer not found: {peer_id}")]
    PeerNotFound { peer_id: String },
    
    #[error("Network partition detected")]
    PartitionDetected,
    
    #[error("Consensus failure: {reason}")]
    ConsensusFailure { reason: String },
}

/// API error response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub status_code: u16,
    pub timestamp: String,
    pub request_id: Option<String>,
    pub details: Option<serde_json::Value>,
}

/// HTTP status code mapping for API errors
impl ApiError {
    pub fn status_code(&self) -> u16 {
        match self {
            ApiError::BadRequest { .. } => 400,
            ApiError::Unauthorized { .. } => 401,
            ApiError::Forbidden { .. } => 403,
            ApiError::NotFound { .. } => 404,
            ApiError::MethodNotAllowed { .. } => 405,
            ApiError::Conflict { .. } => 409,
            ApiError::UnprocessableEntity { .. } => 422,
            ApiError::RateLimited { .. } => 429,
            ApiError::InternalServerError { .. } => 500,
            ApiError::ServiceUnavailable { .. } => 503,
            ApiError::GatewayTimeout { .. } => 504,
        }
    }

    pub fn error_type(&self) -> &'static str {
        match self {
            ApiError::BadRequest { .. } => "BAD_REQUEST",
            ApiError::Unauthorized { .. } => "UNAUTHORIZED",
            ApiError::Forbidden { .. } => "FORBIDDEN",
            ApiError::NotFound { .. } => "NOT_FOUND",
            ApiError::MethodNotAllowed { .. } => "METHOD_NOT_ALLOWED",
            ApiError::Conflict { .. } => "CONFLICT",
            ApiError::UnprocessableEntity { .. } => "UNPROCESSABLE_ENTITY",
            ApiError::RateLimited { .. } => "RATE_LIMITED",
            ApiError::InternalServerError { .. } => "INTERNAL_SERVER_ERROR",
            ApiError::ServiceUnavailable { .. } => "SERVICE_UNAVAILABLE",
            ApiError::GatewayTimeout { .. } => "GATEWAY_TIMEOUT",
        }
    }

    pub fn to_response(&self, request_id: Option<String>) -> ErrorResponse {
        ErrorResponse {
            error: self.error_type().to_string(),
            message: self.to_string(),
            status_code: self.status_code(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            request_id,
            details: None,
        }
    }
}

/// Convert transaction errors to API errors
impl From<TransactionError> for ApiError {
    fn from(err: TransactionError) -> Self {
        match err {
            TransactionError::NotFound { .. } => {
                ApiError::NotFound { message: err.to_string() }
            }
            TransactionError::AlreadyExists { .. } => {
                ApiError::Conflict { message: err.to_string() }
            }
            TransactionError::InvalidFormat 
            | TransactionError::InvalidSignature
            | TransactionError::InvalidNonce { .. } => {
                ApiError::BadRequest { message: err.to_string() }
            }
            TransactionError::InsufficientBalance { .. }
            | TransactionError::GasLimitExceeded { .. } => {
                ApiError::UnprocessableEntity { message: err.to_string() }
            }
            TransactionError::ExecutionFailed { .. } => {
                ApiError::InternalServerError { message: err.to_string() }
            }
        }
    }
}

/// Convert block errors to API errors
impl From<BlockError> for ApiError {
    fn from(err: BlockError) -> Self {
        match err {
            BlockError::NotFound { .. } => {
                ApiError::NotFound { message: err.to_string() }
            }
            BlockError::InvalidHash { .. }
            | BlockError::InvalidHeight { .. } => {
                ApiError::BadRequest { message: err.to_string() }
            }
            BlockError::ValidationFailed { .. } => {
                ApiError::UnprocessableEntity { message: err.to_string() }
            }
        }
    }
}

/// Convert network errors to API errors
impl From<NetworkError> for ApiError {
    fn from(err: NetworkError) -> Self {
        match err {
            NetworkError::ConnectionFailed { .. }
            | NetworkError::Timeout => {
                ApiError::ServiceUnavailable { message: err.to_string() }
            }
            NetworkError::PeerNotFound { .. } => {
                ApiError::NotFound { message: err.to_string() }
            }
            NetworkError::PartitionDetected
            | NetworkError::ConsensusFailure { .. } => {
                ApiError::InternalServerError { message: err.to_string() }
            }
        }
    }
}

/// Utility functions for error handling
pub struct ErrorHandler;

impl ErrorHandler {
    /// Create a bad request error
    pub fn bad_request(message: &str) -> ApiError {
        ApiError::BadRequest {
            message: message.to_string(),
        }
    }

    /// Create an unauthorized error
    pub fn unauthorized(message: &str) -> ApiError {
        ApiError::Unauthorized {
            message: message.to_string(),
        }
    }

    /// Create a forbidden error
    pub fn forbidden(message: &str) -> ApiError {
        ApiError::Forbidden {
            message: message.to_string(),
        }
    }

    /// Create a not found error
    pub fn not_found(message: &str) -> ApiError {
        ApiError::NotFound {
            message: message.to_string(),
        }
    }

    /// Create an internal server error
    pub fn internal_error(message: &str) -> ApiError {
        ApiError::InternalServerError {
            message: message.to_string(),
        }
    }

    /// Create a validation error response
    pub fn validation_error(errors: Vec<ValidationError>) -> ApiError {
        let messages: Vec<String> = errors.iter()
            .map(|e| format!("{}: {}", e.field, e.message))
            .collect();
        
        ApiError::UnprocessableEntity {
            message: format!("Validation failed: {}", messages.join(", ")),
        }
    }

    /// Create a rate limit error
    pub fn rate_limited(retry_after: Option<u64>) -> ApiError {
        let message = match retry_after {
            Some(seconds) => format!("Rate limit exceeded. Retry after {} seconds", seconds),
            None => "Rate limit exceeded".to_string(),
        };
        
        ApiError::RateLimited { message }
    }
}

/// Result type for API operations
pub type ApiResult<T> = Result<T, ApiError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_status_codes() {
        assert_eq!(ApiError::BadRequest { message: "test".to_string() }.status_code(), 400);
        assert_eq!(ApiError::Unauthorized { message: "test".to_string() }.status_code(), 401);
        assert_eq!(ApiError::Forbidden { message: "test".to_string() }.status_code(), 403);
        assert_eq!(ApiError::NotFound { message: "test".to_string() }.status_code(), 404);
        assert_eq!(ApiError::InternalServerError { message: "test".to_string() }.status_code(), 500);
    }

    #[test]
    fn test_api_error_types() {
        assert_eq!(ApiError::BadRequest { message: "test".to_string() }.error_type(), "BAD_REQUEST");
        assert_eq!(ApiError::Unauthorized { message: "test".to_string() }.error_type(), "UNAUTHORIZED");
        assert_eq!(ApiError::NotFound { message: "test".to_string() }.error_type(), "NOT_FOUND");
    }

    #[test]
    fn test_error_response_creation() {
        let error = ApiError::BadRequest { message: "Invalid input".to_string() };
        let response = error.to_response(Some("req_123".to_string()));
        
        assert_eq!(response.error, "BAD_REQUEST");
        assert_eq!(response.message, "Bad Request: Invalid input");
        assert_eq!(response.status_code, 400);
        assert_eq!(response.request_id, Some("req_123".to_string()));
    }

    #[test]
    fn test_transaction_error_conversion() {
        let tx_error = TransactionError::NotFound { hash: "0x123".to_string() };
        let api_error: ApiError = tx_error.into();
        
        match api_error {
            ApiError::NotFound { message } => {
                assert!(message.contains("0x123"));
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_block_error_conversion() {
        let block_error = BlockError::InvalidHash { hash: "invalid".to_string() };
        let api_error: ApiError = block_error.into();
        
        match api_error {
            ApiError::BadRequest { message } => {
                assert!(message.contains("invalid"));
            }
            _ => panic!("Expected BadRequest error"),
        }
    }

    #[test]
    fn test_network_error_conversion() {
        let network_error = NetworkError::ConnectionFailed { reason: "timeout".to_string() };
        let api_error: ApiError = network_error.into();
        
        match api_error {
            ApiError::ServiceUnavailable { message } => {
                assert!(message.contains("timeout"));
            }
            _ => panic!("Expected ServiceUnavailable error"),
        }
    }

    #[test]
    fn test_error_handler_utilities() {
        let bad_request = ErrorHandler::bad_request("Invalid data");
        assert_eq!(bad_request.status_code(), 400);
        
        let unauthorized = ErrorHandler::unauthorized("Token expired");
        assert_eq!(unauthorized.status_code(), 401);
        
        let not_found = ErrorHandler::not_found("Resource not found");
        assert_eq!(not_found.status_code(), 404);
    }

    #[test]
    fn test_validation_error() {
        let validation_errors = vec![
            ValidationError {
                field: "email".to_string(),
                message: "Invalid email format".to_string(),
            },
            ValidationError {
                field: "age".to_string(),
                message: "Must be between 18 and 100".to_string(),
            },
        ];
        
        let error = ErrorHandler::validation_error(validation_errors);
        assert_eq!(error.status_code(), 422);
        assert!(error.to_string().contains("email"));
        assert!(error.to_string().contains("age"));
    }

    #[test]
    fn test_rate_limit_error() {
        let rate_limit_error = ErrorHandler::rate_limited(Some(60));
        assert_eq!(rate_limit_error.status_code(), 429);
        assert!(rate_limit_error.to_string().contains("60 seconds"));
        
        let rate_limit_no_retry = ErrorHandler::rate_limited(None);
        assert_eq!(rate_limit_no_retry.status_code(), 429);
    }

    #[test]
    fn test_serialization() {
        let error = ApiError::BadRequest { message: "test".to_string() };
        let json = serde_json::to_string(&error).unwrap();
        let deserialized: ApiError = serde_json::from_str(&json).unwrap();
        
        match deserialized {
            ApiError::BadRequest { message } => assert_eq!(message, "test"),
            _ => panic!("Deserialization failed"),
        }
    }
}
