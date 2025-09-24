//! CC Chain RPC Error Handling
//!
//! This module provides comprehensive error handling for RPC operations,
//! including standard JSON-RPC 2.0 error codes and CC Chain-specific errors.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use thiserror::Error;

/// Standard JSON-RPC 2.0 error codes
pub mod error_codes {
    /// Parse error - Invalid JSON was received by the server
    pub const PARSE_ERROR: i32 = -32700;
    
    /// Invalid Request - The JSON sent is not a valid Request object
    pub const INVALID_REQUEST: i32 = -32600;
    
    /// Method not found - The method does not exist / is not available
    pub const METHOD_NOT_FOUND: i32 = -32601;
    
    /// Invalid params - Invalid method parameter(s)
    pub const INVALID_PARAMS: i32 = -32602;
    
    /// Internal error - Internal JSON-RPC error
    pub const INTERNAL_ERROR: i32 = -32603;
    
    /// Server errors - Reserved for implementation-defined server-errors
    pub const SERVER_ERROR_START: i32 = -32099;
    pub const SERVER_ERROR_END: i32 = -32000;
    
    // CC Chain specific error codes
    /// Transaction pool full
    pub const TRANSACTION_POOL_FULL: i32 = -32001;
    
    /// Insufficient funds
    pub const INSUFFICIENT_FUNDS: i32 = -32002;
    
    /// Gas limit exceeded
    pub const GAS_LIMIT_EXCEEDED: i32 = -32003;
    
    /// Nonce too low
    pub const NONCE_TOO_LOW: i32 = -32004;
    
    /// Nonce too high
    pub const NONCE_TOO_HIGH: i32 = -32005;
    
    /// Account does not exist
    pub const ACCOUNT_NOT_FOUND: i32 = -32006;
    
    /// Block not found
    pub const BLOCK_NOT_FOUND: i32 = -32007;
    
    /// Transaction not found
    pub const TRANSACTION_NOT_FOUND: i32 = -32008;
    
    /// Network not synced
    pub const NETWORK_NOT_SYNCED: i32 = -32009;
    
    /// Rate limit exceeded
    pub const RATE_LIMIT_EXCEEDED: i32 = -32010;
    
    /// Unauthorized access
    pub const UNAUTHORIZED: i32 = -32011;
    
    /// Service unavailable
    pub const SERVICE_UNAVAILABLE: i32 = -32012;
}

/// JSON-RPC 2.0 error structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl RpcError {
    /// Create a new RPC error
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// Create a new RPC error with additional data
    pub fn with_data(code: i32, message: impl Into<String>, data: Value) -> Self {
        Self {
            code,
            message: message.into(),
            data: Some(data),
        }
    }

    /// Create a parse error
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::new(error_codes::PARSE_ERROR, message)
    }

    /// Create an invalid request error
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new(error_codes::INVALID_REQUEST, message)
    }

    /// Create a method not found error
    pub fn method_not_found(method: impl Into<String>) -> Self {
        Self::new(
            error_codes::METHOD_NOT_FOUND,
            format!("Method '{}' not found", method.into()),
        )
    }

    /// Create an invalid params error
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self::new(error_codes::INVALID_PARAMS, message)
    }

    /// Create an internal error
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(error_codes::INTERNAL_ERROR, message)
    }

    /// Create a transaction pool full error
    pub fn transaction_pool_full() -> Self {
        Self::new(
            error_codes::TRANSACTION_POOL_FULL,
            "Transaction pool is full",
        )
    }

    /// Create an insufficient funds error
    pub fn insufficient_funds(required: u64, available: u64) -> Self {
        Self::with_data(
            error_codes::INSUFFICIENT_FUNDS,
            "Insufficient funds",
            serde_json::json!({
                "required": required,
                "available": available
            }),
        )
    }

    /// Create a gas limit exceeded error
    pub fn gas_limit_exceeded(used: u64, limit: u64) -> Self {
        Self::with_data(
            error_codes::GAS_LIMIT_EXCEEDED,
            "Gas limit exceeded",
            serde_json::json!({
                "used": used,
                "limit": limit
            }),
        )
    }

    /// Create a nonce too low error
    pub fn nonce_too_low(provided: u64, expected: u64) -> Self {
        Self::with_data(
            error_codes::NONCE_TOO_LOW,
            "Nonce too low",
            serde_json::json!({
                "provided": provided,
                "expected": expected
            }),
        )
    }

    /// Create a nonce too high error
    pub fn nonce_too_high(provided: u64, expected: u64) -> Self {
        Self::with_data(
            error_codes::NONCE_TOO_HIGH,
            "Nonce too high",
            serde_json::json!({
                "provided": provided,
                "expected": expected
            }),
        )
    }

    /// Create an account not found error
    pub fn account_not_found(address: impl Into<String>) -> Self {
        Self::with_data(
            error_codes::ACCOUNT_NOT_FOUND,
            "Account not found",
            serde_json::json!({
                "address": address.into()
            }),
        )
    }

    /// Create a block not found error
    pub fn block_not_found(identifier: impl Into<String>) -> Self {
        Self::with_data(
            error_codes::BLOCK_NOT_FOUND,
            "Block not found",
            serde_json::json!({
                "identifier": identifier.into()
            }),
        )
    }

    /// Create a transaction not found error
    pub fn transaction_not_found(hash: impl Into<String>) -> Self {
        Self::with_data(
            error_codes::TRANSACTION_NOT_FOUND,
            "Transaction not found",
            serde_json::json!({
                "hash": hash.into()
            }),
        )
    }

    /// Create a network not synced error
    pub fn network_not_synced(current_height: u64, target_height: u64) -> Self {
        Self::with_data(
            error_codes::NETWORK_NOT_SYNCED,
            "Network not fully synced",
            serde_json::json!({
                "current_height": current_height,
                "target_height": target_height,
                "sync_progress": (current_height as f64 / target_height as f64) * 100.0
            }),
        )
    }

    /// Create a rate limit exceeded error
    pub fn rate_limit_exceeded(retry_after: u64) -> Self {
        Self::with_data(
            error_codes::RATE_LIMIT_EXCEEDED,
            "Rate limit exceeded",
            serde_json::json!({
                "retry_after_seconds": retry_after
            }),
        )
    }

    /// Create an unauthorized error
    pub fn unauthorized(reason: impl Into<String>) -> Self {
        Self::new(error_codes::UNAUTHORIZED, reason)
    }

    /// Create a service unavailable error
    pub fn service_unavailable(reason: impl Into<String>) -> Self {
        Self::new(error_codes::SERVICE_UNAVAILABLE, reason)
    }

    /// Check if this is a client error (4xx equivalent)
    pub fn is_client_error(&self) -> bool {
        matches!(self.code,
            error_codes::PARSE_ERROR |
            error_codes::INVALID_REQUEST |
            error_codes::METHOD_NOT_FOUND |
            error_codes::INVALID_PARAMS |
            error_codes::INSUFFICIENT_FUNDS |
            error_codes::NONCE_TOO_LOW |
            error_codes::NONCE_TOO_HIGH |
            error_codes::UNAUTHORIZED
        )
    }

    /// Check if this is a server error (5xx equivalent)
    pub fn is_server_error(&self) -> bool {
        matches!(self.code,
            error_codes::INTERNAL_ERROR |
            error_codes::TRANSACTION_POOL_FULL |
            error_codes::SERVICE_UNAVAILABLE
        ) || (self.code >= error_codes::SERVER_ERROR_END && self.code <= error_codes::SERVER_ERROR_START)
    }

    /// Check if this is a resource not found error
    pub fn is_not_found_error(&self) -> bool {
        matches!(self.code,
            error_codes::ACCOUNT_NOT_FOUND |
            error_codes::BLOCK_NOT_FOUND |
            error_codes::TRANSACTION_NOT_FOUND
        )
    }

    /// Get the error category as a string
    pub fn category(&self) -> &'static str {
        match self.code {
            error_codes::PARSE_ERROR => "parse",
            error_codes::INVALID_REQUEST => "request",
            error_codes::METHOD_NOT_FOUND => "method",
            error_codes::INVALID_PARAMS => "params",
            error_codes::INTERNAL_ERROR => "internal",
            error_codes::TRANSACTION_POOL_FULL => "pool",
            error_codes::INSUFFICIENT_FUNDS => "funds",
            error_codes::GAS_LIMIT_EXCEEDED => "gas",
            error_codes::NONCE_TOO_LOW | error_codes::NONCE_TOO_HIGH => "nonce",
            error_codes::ACCOUNT_NOT_FOUND | error_codes::BLOCK_NOT_FOUND | 
            error_codes::TRANSACTION_NOT_FOUND => "not_found",
            error_codes::NETWORK_NOT_SYNCED => "sync",
            error_codes::RATE_LIMIT_EXCEEDED => "rate_limit",
            error_codes::UNAUTHORIZED => "auth",
            error_codes::SERVICE_UNAVAILABLE => "service",
            _ => "unknown",
        }
    }
}

impl fmt::Display for RpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RPC Error {}: {}", self.code, self.message)
    }
}

impl std::error::Error for RpcError {}

/// High-level RPC error types that can be converted to RpcError
#[derive(Error, Debug, Clone)]
pub enum RpcErrorType {
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    #[error("Method '{method}' not found")]
    MethodNotFound { method: String },
    
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
    
    #[error("Transaction pool full")]
    TransactionPoolFull,
    
    #[error("Insufficient funds: required {required}, available {available}")]
    InsufficientFunds { required: u64, available: u64 },
    
    #[error("Gas limit exceeded: used {used}, limit {limit}")]
    GasLimitExceeded { used: u64, limit: u64 },
    
    #[error("Nonce too low: provided {provided}, expected {expected}")]
    NonceTooLow { provided: u64, expected: u64 },
    
    #[error("Nonce too high: provided {provided}, expected {expected}")]
    NonceTooHigh { provided: u64, expected: u64 },
    
    #[error("Account not found: {address}")]
    AccountNotFound { address: String },
    
    #[error("Block not found: {identifier}")]
    BlockNotFound { identifier: String },
    
    #[error("Transaction not found: {hash}")]
    TransactionNotFound { hash: String },
    
    #[error("Network not synced: {current_height}/{target_height}")]
    NetworkNotSynced { current_height: u64, target_height: u64 },
    
    #[error("Rate limit exceeded: retry after {retry_after} seconds")]
    RateLimitExceeded { retry_after: u64 },
    
    #[error("Unauthorized: {reason}")]
    Unauthorized { reason: String },
    
    #[error("Service unavailable: {reason}")]
    ServiceUnavailable { reason: String },
}

impl From<RpcErrorType> for RpcError {
    fn from(error_type: RpcErrorType) -> Self {
        match error_type {
            RpcErrorType::ParseError(msg) => RpcError::parse_error(msg),
            RpcErrorType::InvalidRequest(msg) => RpcError::invalid_request(msg),
            RpcErrorType::MethodNotFound { method } => RpcError::method_not_found(method),
            RpcErrorType::InvalidParams(msg) => RpcError::invalid_params(msg),
            RpcErrorType::InternalError(msg) => RpcError::internal_error(msg),
            RpcErrorType::TransactionPoolFull => RpcError::transaction_pool_full(),
            RpcErrorType::InsufficientFunds { required, available } => {
                RpcError::insufficient_funds(required, available)
            }
            RpcErrorType::GasLimitExceeded { used, limit } => {
                RpcError::gas_limit_exceeded(used, limit)
            }
            RpcErrorType::NonceTooLow { provided, expected } => {
                RpcError::nonce_too_low(provided, expected)
            }
            RpcErrorType::NonceTooHigh { provided, expected } => {
                RpcError::nonce_too_high(provided, expected)
            }
            RpcErrorType::AccountNotFound { address } => RpcError::account_not_found(address),
            RpcErrorType::BlockNotFound { identifier } => RpcError::block_not_found(identifier),
            RpcErrorType::TransactionNotFound { hash } => RpcError::transaction_not_found(hash),
            RpcErrorType::NetworkNotSynced { current_height, target_height } => {
                RpcError::network_not_synced(current_height, target_height)
            }
            RpcErrorType::RateLimitExceeded { retry_after } => {
                RpcError::rate_limit_exceeded(retry_after)
            }
            RpcErrorType::Unauthorized { reason } => RpcError::unauthorized(reason),
            RpcErrorType::ServiceUnavailable { reason } => RpcError::service_unavailable(reason),
        }
    }
}

/// Error handler that can convert various error types to RpcError
pub struct RpcErrorHandler;

impl RpcErrorHandler {
    /// Handle a generic error and convert it to an appropriate RpcError
    pub fn handle_error(error: &dyn std::error::Error) -> RpcError {
        // For generic errors, create an internal error
        RpcError::internal_error(error.to_string())
    }

    /// Handle JSON parsing errors
    pub fn handle_json_error(error: &serde_json::Error) -> RpcError {
        RpcError::parse_error(format!("JSON parsing failed: {}", error))
    }

    /// Handle validation errors (generic)
    pub fn handle_validation_error(field: &str, reason: &str) -> RpcError {
        RpcError::invalid_params(format!("Validation failed for {}: {}", field, reason))
    }

    /// Create an error for unsupported operations
    pub fn unsupported_operation(operation: &str) -> RpcError {
        RpcError::internal_error(format!("Operation '{}' is not supported", operation))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_rpc_error_creation() {
        let error = RpcError::new(-32600, "Invalid request");
        assert_eq!(error.code, -32600);
        assert_eq!(error.message, "Invalid request");
        assert!(error.data.is_none());
    }

    #[test]
    fn test_rpc_error_with_data() {
        let data = json!({"field": "value"});
        let error = RpcError::with_data(-32602, "Invalid params", data.clone());
        assert_eq!(error.code, -32602);
        assert_eq!(error.message, "Invalid params");
        assert_eq!(error.data, Some(data));
    }

    #[test]
    fn test_standard_errors() {
        let parse_error = RpcError::parse_error("Invalid JSON");
        assert_eq!(parse_error.code, error_codes::PARSE_ERROR);

        let invalid_request = RpcError::invalid_request("Missing method");
        assert_eq!(invalid_request.code, error_codes::INVALID_REQUEST);

        let method_not_found = RpcError::method_not_found("unknown_method");
        assert_eq!(method_not_found.code, error_codes::METHOD_NOT_FOUND);
        assert!(method_not_found.message.contains("unknown_method"));

        let invalid_params = RpcError::invalid_params("Missing parameter");
        assert_eq!(invalid_params.code, error_codes::INVALID_PARAMS);

        let internal_error = RpcError::internal_error("Database error");
        assert_eq!(internal_error.code, error_codes::INTERNAL_ERROR);
    }

    #[test]
    fn test_cc_chain_specific_errors() {
        let pool_full = RpcError::transaction_pool_full();
        assert_eq!(pool_full.code, error_codes::TRANSACTION_POOL_FULL);

        let insufficient_funds = RpcError::insufficient_funds(1000, 500);
        assert_eq!(insufficient_funds.code, error_codes::INSUFFICIENT_FUNDS);
        if let Some(data) = &insufficient_funds.data {
            assert_eq!(data["required"], 1000);
            assert_eq!(data["available"], 500);
        }

        let gas_exceeded = RpcError::gas_limit_exceeded(100000, 50000);
        assert_eq!(gas_exceeded.code, error_codes::GAS_LIMIT_EXCEEDED);

        let nonce_low = RpcError::nonce_too_low(5, 10);
        assert_eq!(nonce_low.code, error_codes::NONCE_TOO_LOW);

        let account_not_found = RpcError::account_not_found("0x123");
        assert_eq!(account_not_found.code, error_codes::ACCOUNT_NOT_FOUND);
    }

    #[test]
    fn test_error_categories() {
        let parse_error = RpcError::parse_error("test");
        assert!(parse_error.is_client_error());
        assert!(!parse_error.is_server_error());

        let internal_error = RpcError::internal_error("test");
        assert!(!internal_error.is_client_error());
        assert!(internal_error.is_server_error());

        let not_found = RpcError::account_not_found("0x123");
        assert!(not_found.is_not_found_error());
    }

    #[test]
    fn test_error_categories_string() {
        assert_eq!(RpcError::parse_error("test").category(), "parse");
        assert_eq!(RpcError::method_not_found("test").category(), "method");
        assert_eq!(RpcError::insufficient_funds(100, 50).category(), "funds");
        assert_eq!(RpcError::account_not_found("0x123").category(), "not_found");
    }

    #[test]
    fn test_error_type_conversion() {
        let error_type = RpcErrorType::InsufficientFunds {
            required: 1000,
            available: 500,
        };
        let rpc_error: RpcError = error_type.into();
        assert_eq!(rpc_error.code, error_codes::INSUFFICIENT_FUNDS);
    }

    #[test]
    fn test_error_handler() {
        let json_error = serde_json::from_str::<Value>("invalid json").unwrap_err();
        let rpc_error = RpcErrorHandler::handle_json_error(&json_error);
        assert_eq!(rpc_error.code, error_codes::PARSE_ERROR);

        let validation_error = RpcErrorHandler::handle_validation_error("address", "invalid format");
        assert_eq!(validation_error.code, error_codes::INVALID_PARAMS);
        assert!(validation_error.message.contains("address"));

        let unsupported = RpcErrorHandler::unsupported_operation("debug_trace");
        assert_eq!(unsupported.code, error_codes::INTERNAL_ERROR);
        assert!(unsupported.message.contains("debug_trace"));
    }

    #[test]
    fn test_error_display() {
        let error = RpcError::new(-32602, "Invalid params");
        let display_string = format!("{}", error);
        assert!(display_string.contains("RPC Error -32602"));
        assert!(display_string.contains("Invalid params"));
    }

    #[test]
    fn test_error_serialization() {
        let error = RpcError::insufficient_funds(1000, 500);
        let json = serde_json::to_string(&error).unwrap();
        let deserialized: RpcError = serde_json::from_str(&json).unwrap();
        assert_eq!(error, deserialized);
    }

    #[test]
    fn test_network_not_synced_error() {
        let error = RpcError::network_not_synced(1000, 2000);
        assert_eq!(error.code, error_codes::NETWORK_NOT_SYNCED);
        if let Some(data) = &error.data {
            assert_eq!(data["current_height"], 1000);
            assert_eq!(data["target_height"], 2000);
            assert_eq!(data["sync_progress"], 50.0);
        }
    }

    #[test]
    fn test_rate_limit_error() {
        let error = RpcError::rate_limit_exceeded(60);
        assert_eq!(error.code, error_codes::RATE_LIMIT_EXCEEDED);
        if let Some(data) = &error.data {
            assert_eq!(data["retry_after_seconds"], 60);
        }
    }
}
