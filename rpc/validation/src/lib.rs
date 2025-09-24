//! CC Chain RPC Validation
//!
//! This module provides validation utilities for RPC requests and responses.
//! It ensures data integrity and security for all RPC communications.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Invalid parameter: {field} - {reason}")]
    InvalidParameter { field: String, reason: String },
    #[error("Missing required parameter: {0}")]
    MissingParameter(String),
    #[error("Invalid JSON-RPC format: {0}")]
    InvalidFormat(String),
    #[error("Invalid address format: {0}")]
    InvalidAddress(String),
    #[error("Invalid hash format: {0}")]
    InvalidHash(String),
    #[error("Value out of range: {field} must be between {min} and {max}")]
    OutOfRange { field: String, min: i64, max: i64 },
    #[error("Method not allowed: {0}")]
    MethodNotAllowed(String),
}

pub type Result<T> = std::result::Result<T, ValidationError>;

/// JSON-RPC 2.0 request structure for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
    pub id: Option<Value>,
}

/// Validation configuration
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    pub allowed_methods: HashSet<String>,
    pub max_block_height: u64,
    pub min_gas_limit: u64,
    pub max_gas_limit: u64,
    pub max_value: u64,
    pub require_valid_addresses: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        let mut allowed_methods = HashSet::new();
        allowed_methods.insert("cc_getBlockByHeight".to_string());
        allowed_methods.insert("cc_getBlockByHash".to_string());
        allowed_methods.insert("cc_getLatestBlock".to_string());
        allowed_methods.insert("cc_getTransaction".to_string());
        allowed_methods.insert("cc_getAccount".to_string());
        allowed_methods.insert("cc_getBalance".to_string());
        allowed_methods.insert("cc_getNetworkInfo".to_string());
        allowed_methods.insert("cc_getPeerCount".to_string());
        allowed_methods.insert("cc_getSyncStatus".to_string());
        allowed_methods.insert("cc_sendTransaction".to_string());
        allowed_methods.insert("cc_estimateGas".to_string());
        allowed_methods.insert("cc_getTransactionCount".to_string());
        allowed_methods.insert("cc_getVersion".to_string());
        allowed_methods.insert("cc_ping".to_string());

        Self {
            allowed_methods,
            max_block_height: u64::MAX,
            min_gas_limit: 21000,
            max_gas_limit: 10_000_000,
            max_value: u64::MAX,
            require_valid_addresses: true,
        }
    }
}

/// RPC request validator
pub struct RpcValidator {
    config: ValidationConfig,
}

impl RpcValidator {
    /// Create a new validator with default configuration
    pub fn new() -> Self {
        Self::with_config(ValidationConfig::default())
    }

    /// Create a new validator with custom configuration
    pub fn with_config(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Validate a complete RPC request
    pub fn validate_request(&self, request: &RpcRequest) -> Result<()> {
        self.validate_jsonrpc_format(request)?;
        self.validate_method(&request.method)?;
        self.validate_method_parameters(&request.method, request.params.as_ref())?;
        Ok(())
    }

    /// Validate JSON-RPC 2.0 format
    fn validate_jsonrpc_format(&self, request: &RpcRequest) -> Result<()> {
        if request.jsonrpc != "2.0" {
            return Err(ValidationError::InvalidFormat(
                "jsonrpc must be '2.0'".to_string()
            ));
        }

        if request.method.is_empty() {
            return Err(ValidationError::InvalidFormat(
                "method cannot be empty".to_string()
            ));
        }

        Ok(())
    }

    /// Validate method name
    fn validate_method(&self, method: &str) -> Result<()> {
        if !self.config.allowed_methods.contains(method) {
            return Err(ValidationError::MethodNotAllowed(method.to_string()));
        }
        Ok(())
    }

    /// Validate method-specific parameters
    fn validate_method_parameters(&self, method: &str, params: Option<&Value>) -> Result<()> {
        match method {
            "cc_getBlockByHeight" => self.validate_get_block_by_height_params(params),
            "cc_getBlockByHash" => self.validate_get_block_by_hash_params(params),
            "cc_getTransaction" => self.validate_get_transaction_params(params),
            "cc_getAccount" => self.validate_get_account_params(params),
            "cc_getBalance" => self.validate_get_balance_params(params),
            "cc_getTransactionCount" => self.validate_get_transaction_count_params(params),
            "cc_sendTransaction" => self.validate_send_transaction_params(params),
            "cc_estimateGas" => self.validate_estimate_gas_params(params),
            // Methods that don't require parameters
            "cc_getLatestBlock" | "cc_getNetworkInfo" | "cc_getPeerCount" | 
            "cc_getSyncStatus" | "cc_getVersion" | "cc_ping" => Ok(()),
            _ => Ok(()), // Unknown methods are handled by method validation
        }
    }

    fn validate_get_block_by_height_params(&self, params: Option<&Value>) -> Result<()> {
        let params = params.ok_or_else(|| {
            ValidationError::MissingParameter("height".to_string())
        })?;

        let height = params.get("height")
            .ok_or_else(|| ValidationError::MissingParameter("height".to_string()))?
            .as_u64()
            .ok_or_else(|| ValidationError::InvalidParameter {
                field: "height".to_string(),
                reason: "must be a valid number".to_string(),
            })?;

        if height > self.config.max_block_height {
            return Err(ValidationError::OutOfRange {
                field: "height".to_string(),
                min: 0,
                max: self.config.max_block_height as i64,
            });
        }

        Ok(())
    }

    fn validate_get_block_by_hash_params(&self, params: Option<&Value>) -> Result<()> {
        let params = params.ok_or_else(|| {
            ValidationError::MissingParameter("hash".to_string())
        })?;

        let hash = params.get("hash")
            .ok_or_else(|| ValidationError::MissingParameter("hash".to_string()))?
            .as_str()
            .ok_or_else(|| ValidationError::InvalidParameter {
                field: "hash".to_string(),
                reason: "must be a string".to_string(),
            })?;

        self.validate_hash(hash, "hash")?;
        Ok(())
    }

    fn validate_get_transaction_params(&self, params: Option<&Value>) -> Result<()> {
        let params = params.ok_or_else(|| {
            ValidationError::MissingParameter("hash".to_string())
        })?;

        let hash = params.get("hash")
            .ok_or_else(|| ValidationError::MissingParameter("hash".to_string()))?
            .as_str()
            .ok_or_else(|| ValidationError::InvalidParameter {
                field: "hash".to_string(),
                reason: "must be a string".to_string(),
            })?;

        self.validate_hash(hash, "hash")?;
        Ok(())
    }

    fn validate_get_account_params(&self, params: Option<&Value>) -> Result<()> {
        let params = params.ok_or_else(|| {
            ValidationError::MissingParameter("address".to_string())
        })?;

        let address = params.get("address")
            .ok_or_else(|| ValidationError::MissingParameter("address".to_string()))?
            .as_str()
            .ok_or_else(|| ValidationError::InvalidParameter {
                field: "address".to_string(),
                reason: "must be a string".to_string(),
            })?;

        self.validate_address(address, "address")?;
        Ok(())
    }

    fn validate_get_balance_params(&self, params: Option<&Value>) -> Result<()> {
        self.validate_get_account_params(params)
    }

    fn validate_get_transaction_count_params(&self, params: Option<&Value>) -> Result<()> {
        self.validate_get_account_params(params)
    }

    fn validate_send_transaction_params(&self, params: Option<&Value>) -> Result<()> {
        let params = params.ok_or_else(|| {
            ValidationError::MissingParameter("transaction".to_string())
        })?;

        let tx = params.get("transaction")
            .ok_or_else(|| ValidationError::MissingParameter("transaction".to_string()))?;

        // Validate required transaction fields
        let from = tx.get("from")
            .ok_or_else(|| ValidationError::MissingParameter("transaction.from".to_string()))?
            .as_str()
            .ok_or_else(|| ValidationError::InvalidParameter {
                field: "transaction.from".to_string(),
                reason: "must be a string".to_string(),
            })?;

        self.validate_address(from, "transaction.from")?;

        // Validate optional 'to' field
        if let Some(to_value) = tx.get("to") {
            if !to_value.is_null() {
                let to = to_value.as_str()
                    .ok_or_else(|| ValidationError::InvalidParameter {
                        field: "transaction.to".to_string(),
                        reason: "must be a string or null".to_string(),
                    })?;
                self.validate_address(to, "transaction.to")?;
            }
        }

        // Validate value
        if let Some(value_str) = tx.get("value").and_then(|v| v.as_str()) {
            let value: u64 = value_str.parse()
                .map_err(|_| ValidationError::InvalidParameter {
                    field: "transaction.value".to_string(),
                    reason: "must be a valid number".to_string(),
                })?;

            if value > self.config.max_value {
                return Err(ValidationError::OutOfRange {
                    field: "transaction.value".to_string(),
                    min: 0,
                    max: self.config.max_value as i64,
                });
            }
        }

        // Validate gas limit
        if let Some(gas_limit_str) = tx.get("gas_limit").and_then(|v| v.as_str()) {
            let gas_limit: u64 = gas_limit_str.parse()
                .map_err(|_| ValidationError::InvalidParameter {
                    field: "transaction.gas_limit".to_string(),
                    reason: "must be a valid number".to_string(),
                })?;

            if gas_limit < self.config.min_gas_limit || gas_limit > self.config.max_gas_limit {
                return Err(ValidationError::OutOfRange {
                    field: "transaction.gas_limit".to_string(),
                    min: self.config.min_gas_limit as i64,
                    max: self.config.max_gas_limit as i64,
                });
            }
        }

        Ok(())
    }

    fn validate_estimate_gas_params(&self, params: Option<&Value>) -> Result<()> {
        // Similar to send_transaction but less strict since it's just an estimate
        let params = params.ok_or_else(|| {
            ValidationError::MissingParameter("transaction".to_string())
        })?;

        let tx = params.get("transaction")
            .ok_or_else(|| ValidationError::MissingParameter("transaction".to_string()))?;

        // Only validate that it's a valid transaction object
        if !tx.is_object() {
            return Err(ValidationError::InvalidParameter {
                field: "transaction".to_string(),
                reason: "must be an object".to_string(),
            });
        }

        Ok(())
    }

    /// Validate address format
    pub fn validate_address(&self, address: &str, field_name: &str) -> Result<()> {
        if !self.config.require_valid_addresses {
            return Ok(());
        }

        if address.is_empty() {
            return Err(ValidationError::InvalidAddress(
                format!("{}: address cannot be empty", field_name)
            ));
        }

        // Basic address format validation (starts with 0x, has reasonable length)
        if !address.starts_with("0x") {
            return Err(ValidationError::InvalidAddress(
                format!("{}: address must start with '0x'", field_name)
            ));
        }

        if address.len() < 10 || address.len() > 66 {
            return Err(ValidationError::InvalidAddress(
                format!("{}: address has invalid length", field_name)
            ));
        }

        // Check if the rest are valid hex characters
        let hex_part = &address[2..];
        if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ValidationError::InvalidAddress(
                format!("{}: address contains invalid characters", field_name)
            ));
        }

        Ok(())
    }

    /// Validate hash format
    pub fn validate_hash(&self, hash: &str, field_name: &str) -> Result<()> {
        if hash.is_empty() {
            return Err(ValidationError::InvalidHash(
                format!("{}: hash cannot be empty", field_name)
            ));
        }

        if !hash.starts_with("0x") {
            return Err(ValidationError::InvalidHash(
                format!("{}: hash must start with '0x'", field_name)
            ));
        }

        // Standard hash should be 66 characters (0x + 64 hex chars)
        if hash.len() != 66 {
            return Err(ValidationError::InvalidHash(
                format!("{}: hash must be 66 characters long", field_name)
            ));
        }

        let hex_part = &hash[2..];
        if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ValidationError::InvalidHash(
                format!("{}: hash contains invalid characters", field_name)
            ));
        }

        Ok(())
    }

    /// Validate hex string
    pub fn validate_hex_string(&self, hex: &str, field_name: &str) -> Result<()> {
        if hex.starts_with("0x") {
            let hex_part = &hex[2..];
            if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(ValidationError::InvalidParameter {
                    field: field_name.to_string(),
                    reason: "contains invalid hex characters".to_string(),
                });
            }
        } else if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ValidationError::InvalidParameter {
                field: field_name.to_string(),
                reason: "contains invalid hex characters".to_string(),
            });
        }

        Ok(())
    }

    /// Validate positive integer
    pub fn validate_positive_integer(&self, value: i64, field_name: &str) -> Result<()> {
        if value < 0 {
            return Err(ValidationError::InvalidParameter {
                field: field_name.to_string(),
                reason: "must be positive".to_string(),
            });
        }
        Ok(())
    }
}

impl Default for RpcValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validator_creation() {
        let validator = RpcValidator::new();
        assert!(!validator.config.allowed_methods.is_empty());
    }

    #[test]
    fn test_validate_jsonrpc_format() {
        let validator = RpcValidator::new();
        
        let valid_request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "cc_ping".to_string(),
            params: None,
            id: Some(json!(1)),
        };
        
        assert!(validator.validate_request(&valid_request).is_ok());
        
        let invalid_request = RpcRequest {
            jsonrpc: "1.0".to_string(),
            method: "cc_ping".to_string(),
            params: None,
            id: Some(json!(1)),
        };
        
        assert!(validator.validate_request(&invalid_request).is_err());
    }

    #[test]
    fn test_validate_method() {
        let validator = RpcValidator::new();
        
        assert!(validator.validate_method("cc_ping").is_ok());
        assert!(validator.validate_method("cc_getLatestBlock").is_ok());
        assert!(validator.validate_method("invalid_method").is_err());
    }

    #[test]
    fn test_validate_address() {
        let validator = RpcValidator::new();
        
        // Valid addresses
        assert!(validator.validate_address("0x1234567890abcdef", "test").is_ok());
        assert!(validator.validate_address("0x1234567890abcdef1234567890abcdef12345678", "test").is_ok());
        
        // Invalid addresses
        assert!(validator.validate_address("", "test").is_err());
        assert!(validator.validate_address("1234567890abcdef", "test").is_err());
        assert!(validator.validate_address("0x", "test").is_err());
        assert!(validator.validate_address("0xnothex", "test").is_err());
    }

    #[test]
    fn test_validate_hash() {
        let validator = RpcValidator::new();
        
        // Valid hash
        let valid_hash = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        assert!(validator.validate_hash(valid_hash, "test").is_ok());
        
        // Invalid hashes
        assert!(validator.validate_hash("", "test").is_err());
        assert!(validator.validate_hash("0x123", "test").is_err());
        assert!(validator.validate_hash("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef", "test").is_err());
        assert!(validator.validate_hash("0xnothexadecimal123456789012345678901234567890123456789012345678", "test").is_err());
    }

    #[test]
    fn test_validate_get_block_by_height() {
        let validator = RpcValidator::new();
        
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "cc_getBlockByHeight".to_string(),
            params: Some(json!({"height": 12345})),
            id: Some(json!(1)),
        };
        
        assert!(validator.validate_request(&request).is_ok());
        
        let invalid_request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "cc_getBlockByHeight".to_string(),
            params: Some(json!({"height": "invalid"})),
            id: Some(json!(1)),
        };
        
        assert!(validator.validate_request(&invalid_request).is_err());
    }

    #[test]
    fn test_validate_get_balance() {
        let validator = RpcValidator::new();
        
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "cc_getBalance".to_string(),  
            params: Some(json!({"address": "0x1234567890abcdef"})),
            id: Some(json!(1)),
        };
        
        assert!(validator.validate_request(&request).is_ok());
    }

    #[test]
    fn test_validate_send_transaction() {
        let validator = RpcValidator::new();
        
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "cc_sendTransaction".to_string(),
            params: Some(json!({
                "transaction": {
                    "from": "0x1234567890abcdef",
                    "to": "0xfedcba0987654321",
                    "value": "1000000",
                    "gas_limit": "21000"
                }
            })),
            id: Some(json!(1)),
        };
        
        assert!(validator.validate_request(&request).is_ok());
        
        // Test missing required field
        let invalid_request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "cc_sendTransaction".to_string(),
            params: Some(json!({
                "transaction": {
                    "to": "0xfedcba0987654321",
                    "value": "1000000"
                    // missing 'from'
                }
            })),
            id: Some(json!(1)),
        };
        
        assert!(validator.validate_request(&invalid_request).is_err());
    }

    #[test]
    fn test_hex_string_validation() {
        let validator = RpcValidator::new();
        
        assert!(validator.validate_hex_string("0x1234abcd", "test").is_ok());
        assert!(validator.validate_hex_string("1234abcd", "test").is_ok());
        assert!(validator.validate_hex_string("0x123xyz", "test").is_err());
        assert!(validator.validate_hex_string("123xyz", "test").is_err());
    }

    #[test]
    fn test_positive_integer_validation() {
        let validator = RpcValidator::new();
        
        assert!(validator.validate_positive_integer(42, "test").is_ok());
        assert!(validator.validate_positive_integer(0, "test").is_ok());
        assert!(validator.validate_positive_integer(-1, "test").is_err());
    }
}
