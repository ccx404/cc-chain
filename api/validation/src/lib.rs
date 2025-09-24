//! CC Chain API Validation
//!
//! This module provides comprehensive request validation functionality for the CC Chain API,
//! including input sanitization, format validation, and business rule enforcement.

use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ValidationError {
    #[error("Required field '{field}' is missing")]
    Required { field: String },
    
    #[error("Field '{field}' has invalid format: {reason}")]
    InvalidFormat { field: String, reason: String },
    
    #[error("Field '{field}' value '{value}' is out of range: {reason}")]
    OutOfRange { field: String, value: String, reason: String },
    
    #[error("Field '{field}' value '{value}' is too short (min: {min})")]
    TooShort { field: String, value: String, min: usize },
    
    #[error("Field '{field}' value '{value}' is too long (max: {max})")]
    TooLong { field: String, value: String, max: usize },
    
    #[error("Field '{field}' contains invalid characters")]
    InvalidCharacters { field: String },
    
    #[error("Business rule violation: {rule}")]
    BusinessRule { rule: String },
}

pub type Result<T> = std::result::Result<T, ValidationError>;

/// Validation result containing all errors
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: ValidationError) {
        self.is_valid = false;
        self.errors.push(error);
    }

    pub fn merge(&mut self, other: ValidationResult) {
        if !other.is_valid {
            self.is_valid = false;
            self.errors.extend(other.errors);
        }
    }
}

/// Transaction validation request
#[derive(Debug, Deserialize)]
pub struct TransactionValidationRequest {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub fee: u64,
    pub data: Option<String>,
    pub signature: String,
    pub nonce: Option<u64>,
    pub gas_limit: Option<u64>,
}

/// Account validation request
#[derive(Debug, Deserialize)]
pub struct AccountValidationRequest {
    pub address: String,
    pub public_key: Option<String>,
}

/// Block validation request
#[derive(Debug, Deserialize)]
pub struct BlockValidationRequest {
    pub hash: Option<String>,
    pub height: Option<u64>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// API key validation request
#[derive(Debug, Deserialize)]
pub struct ApiKeyValidationRequest {
    pub name: String,
    pub permissions: Vec<String>,
    pub rate_limit: Option<u32>,
}

/// CC Chain API Validator
pub struct Validator {
    config: ValidatorConfig,
}

#[derive(Debug, Clone)]
pub struct ValidatorConfig {
    pub max_transaction_amount: u64,
    pub min_fee: u64,
    pub max_data_size: usize,
    pub address_length: usize,
    pub hash_length: usize,
    pub signature_length: usize,
    pub max_query_limit: u32,
    pub max_api_key_name_length: usize,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            max_transaction_amount: 1_000_000_000_000, // 1 trillion base units
            min_fee: 100,
            max_data_size: 1024 * 64, // 64KB
            address_length: 42, // 0x + 40 hex chars
            hash_length: 66, // 0x + 64 hex chars
            signature_length: 132, // 0x + 130 hex chars
            max_query_limit: 1000,
            max_api_key_name_length: 100,
        }
    }
}

impl Validator {
    pub fn new(config: ValidatorConfig) -> Self {
        Self { config }
    }

    pub fn with_default_config() -> Self {
        Self::new(ValidatorConfig::default())
    }

    /// Validate transaction request
    pub fn validate_transaction(&self, request: &TransactionValidationRequest) -> ValidationResult {
        let mut result = ValidationResult::new();

        // Validate from address
        if let Err(error) = self.validate_address(&request.from, "from") {
            result.add_error(error);
        }

        // Validate to address
        if let Err(error) = self.validate_address(&request.to, "to") {
            result.add_error(error);
        }

        // Validate amount
        if request.amount == 0 {
            result.add_error(ValidationError::OutOfRange {
                field: "amount".to_string(),
                value: request.amount.to_string(),
                reason: "Amount must be greater than 0".to_string(),
            });
        }

        if request.amount > self.config.max_transaction_amount {
            result.add_error(ValidationError::OutOfRange {
                field: "amount".to_string(),
                value: request.amount.to_string(),
                reason: format!("Amount exceeds maximum of {}", self.config.max_transaction_amount),
            });
        }

        // Validate fee
        if request.fee < self.config.min_fee {
            result.add_error(ValidationError::OutOfRange {
                field: "fee".to_string(),
                value: request.fee.to_string(),
                reason: format!("Fee must be at least {}", self.config.min_fee),
            });
        }

        // Validate data size
        if let Some(ref data) = request.data {
            if data.len() > self.config.max_data_size {
                result.add_error(ValidationError::TooLong {
                    field: "data".to_string(),
                    value: format!("{}...", &data[..50.min(data.len())]),
                    max: self.config.max_data_size,
                });
            }

            // Validate data is valid hex
            if !data.is_empty() && !self.is_valid_hex(data) {
                result.add_error(ValidationError::InvalidFormat {
                    field: "data".to_string(),
                    reason: "Data must be valid hexadecimal".to_string(),
                });
            }
        }

        // Validate signature
        if let Err(error) = self.validate_signature(&request.signature) {
            result.add_error(error);
        }

        // Validate gas limit if provided
        if let Some(gas_limit) = request.gas_limit {
            if gas_limit == 0 {
                result.add_error(ValidationError::OutOfRange {
                    field: "gas_limit".to_string(),
                    value: gas_limit.to_string(),
                    reason: "Gas limit must be greater than 0".to_string(),
                });
            }
        }

        result
    }

    /// Validate account request
    pub fn validate_account(&self, request: &AccountValidationRequest) -> ValidationResult {
        let mut result = ValidationResult::new();

        // Validate address
        if let Err(error) = self.validate_address(&request.address, "address") {
            result.add_error(error);
        }

        // Validate public key if provided
        if let Some(ref public_key) = request.public_key {
            if !self.is_valid_hex(public_key) || public_key.len() != 66 {
                result.add_error(ValidationError::InvalidFormat {
                    field: "public_key".to_string(),
                    reason: "Public key must be 32 bytes hex string (0x + 64 chars)".to_string(),
                });
            }
        }

        result
    }

    /// Validate block query request
    pub fn validate_block_query(&self, request: &BlockValidationRequest) -> ValidationResult {
        let mut result = ValidationResult::new();

        // Validate hash if provided
        if let Some(ref hash) = request.hash {
            if let Err(error) = self.validate_hash(hash, "hash") {
                result.add_error(error);
            }
        }

        // Validate height if provided
        if let Some(height) = request.height {
            if height > u64::MAX / 2 { // Reasonable upper bound
                result.add_error(ValidationError::OutOfRange {
                    field: "height".to_string(),
                    value: height.to_string(),
                    reason: "Block height is unreasonably large".to_string(),
                });
            }
        }

        // Validate pagination
        if let Some(limit) = request.limit {
            if limit == 0 {
                result.add_error(ValidationError::OutOfRange {
                    field: "limit".to_string(),
                    value: limit.to_string(),
                    reason: "Limit must be greater than 0".to_string(),
                });
            } else if limit > self.config.max_query_limit {
                result.add_error(ValidationError::OutOfRange {
                    field: "limit".to_string(),
                    value: limit.to_string(),
                    reason: format!("Limit cannot exceed {}", self.config.max_query_limit),
                });
            }
        }

        result
    }

    /// Validate API key request
    pub fn validate_api_key(&self, request: &ApiKeyValidationRequest) -> ValidationResult {
        let mut result = ValidationResult::new();

        // Validate name
        if request.name.is_empty() {
            result.add_error(ValidationError::Required {
                field: "name".to_string(),
            });
        } else if request.name.len() > self.config.max_api_key_name_length {
            result.add_error(ValidationError::TooLong {
                field: "name".to_string(),
                value: request.name.clone(),
                max: self.config.max_api_key_name_length,
            });
        } else if !self.is_valid_api_key_name(&request.name) {
            result.add_error(ValidationError::InvalidCharacters {
                field: "name".to_string(),
            });
        }

        // Validate permissions
        for permission in &request.permissions {
            if !self.is_valid_permission(permission) {
                result.add_error(ValidationError::InvalidFormat {
                    field: "permissions".to_string(),
                    reason: format!("Unknown permission: {}", permission),
                });
            }
        }

        // Validate rate limit
        if let Some(rate_limit) = request.rate_limit {
            if rate_limit == 0 {
                result.add_error(ValidationError::OutOfRange {
                    field: "rate_limit".to_string(),
                    value: rate_limit.to_string(),
                    reason: "Rate limit must be greater than 0".to_string(),
                });
            } else if rate_limit > 10000 {
                result.add_error(ValidationError::OutOfRange {
                    field: "rate_limit".to_string(),
                    value: rate_limit.to_string(),
                    reason: "Rate limit cannot exceed 10000 requests per minute".to_string(),
                });
            }
        }

        result
    }

    /// Validate blockchain address
    fn validate_address(&self, address: &str, field: &str) -> Result<()> {
        if address.is_empty() {
            return Err(ValidationError::Required {
                field: field.to_string(),
            });
        }

        if address.len() != self.config.address_length {
            return Err(ValidationError::InvalidFormat {
                field: field.to_string(),
                reason: format!("Address must be {} characters long", self.config.address_length),
            });
        }

        if !address.starts_with("0x") {
            return Err(ValidationError::InvalidFormat {
                field: field.to_string(),
                reason: "Address must start with '0x'".to_string(),
            });
        }

        if !self.is_valid_hex(address) {
            return Err(ValidationError::InvalidFormat {
                field: field.to_string(),
                reason: "Address must be valid hexadecimal".to_string(),
            });
        }

        Ok(())
    }

    /// Validate hash
    fn validate_hash(&self, hash: &str, field: &str) -> Result<()> {
        if hash.is_empty() {
            return Err(ValidationError::Required {
                field: field.to_string(),
            });
        }

        if hash.len() != self.config.hash_length {
            return Err(ValidationError::InvalidFormat {
                field: field.to_string(),
                reason: format!("Hash must be {} characters long", self.config.hash_length),
            });
        }

        if !hash.starts_with("0x") {
            return Err(ValidationError::InvalidFormat {
                field: field.to_string(),
                reason: "Hash must start with '0x'".to_string(),
            });
        }

        if !self.is_valid_hex(hash) {
            return Err(ValidationError::InvalidFormat {
                field: field.to_string(),
                reason: "Hash must be valid hexadecimal".to_string(),
            });
        }

        Ok(())
    }

    /// Validate signature
    fn validate_signature(&self, signature: &str) -> Result<()> {
        if signature.is_empty() {
            return Err(ValidationError::Required {
                field: "signature".to_string(),
            });
        }

        if signature.len() != self.config.signature_length {
            return Err(ValidationError::InvalidFormat {
                field: "signature".to_string(),
                reason: format!("Signature must be {} characters long", self.config.signature_length),
            });
        }

        if !signature.starts_with("0x") {
            return Err(ValidationError::InvalidFormat {
                field: "signature".to_string(),
                reason: "Signature must start with '0x'".to_string(),
            });
        }

        if !self.is_valid_hex(signature) {
            return Err(ValidationError::InvalidFormat {
                field: "signature".to_string(),
                reason: "Signature must be valid hexadecimal".to_string(),
            });
        }

        Ok(())
    }

    /// Check if string is valid hexadecimal
    fn is_valid_hex(&self, s: &str) -> bool {
        if !s.starts_with("0x") || s.len() <= 2 {
            return false;
        }
        
        s[2..].chars().all(|c| c.is_ascii_hexdigit())
    }

    /// Check if API key name is valid
    fn is_valid_api_key_name(&self, name: &str) -> bool {
        name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    }

    /// Check if permission is valid
    fn is_valid_permission(&self, permission: &str) -> bool {
        matches!(permission, 
            "read" | "write" | "admin" | "validate" | "consensus" | 
            "deploy" | "manage_users" | "manage_keys"
        )
    }

    /// Sanitize input string
    pub fn sanitize_string(&self, input: &str, max_length: usize) -> String {
        input
            .chars()
            .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
            .take(max_length)
            .collect()
    }

    /// Validate and sanitize pagination parameters
    pub fn validate_pagination(&self, limit: Option<u32>, _offset: Option<u32>) -> ValidationResult {
        let mut result = ValidationResult::new();

        if let Some(limit) = limit {
            if limit == 0 {
                result.add_error(ValidationError::OutOfRange {
                    field: "limit".to_string(),
                    value: limit.to_string(),
                    reason: "Limit must be greater than 0".to_string(),
                });
            } else if limit > self.config.max_query_limit {
                result.add_error(ValidationError::OutOfRange {
                    field: "limit".to_string(),
                    value: limit.to_string(),
                    reason: format!("Limit cannot exceed {}", self.config.max_query_limit),
                });
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_validator() -> Validator {
        Validator::with_default_config()
    }

    #[test]
    fn test_validate_transaction_valid() {
        let validator = create_test_validator();
        let request = TransactionValidationRequest {
            from: "0x1234567890123456789012345678901234567890".to_string(),
            to: "0x1234567890123456789012345678901234567890".to_string(), // Fixed length
            amount: 1000,
            fee: 200,
            data: Some("0x1234".to_string()),
            signature: "0x1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890".to_string(), // Fixed length (130 hex chars + 0x)
            nonce: Some(1),
            gas_limit: Some(21000),
        };

        let result = validator.validate_transaction(&request);
        assert!(result.is_valid, "Expected valid transaction, got errors: {:?}", result.errors);
    }

    #[test]
    fn test_validate_transaction_invalid_address() {
        let validator = create_test_validator();
        let request = TransactionValidationRequest {
            from: "invalid_address".to_string(),
            to: "0xabcdefabcdefabcdefabcdefabcdefabcdefabcdef".to_string(),
            amount: 1000,
            fee: 200,
            data: None,
            signature: "0x123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012".to_string(),
            nonce: None,
            gas_limit: None,
        };

        let result = validator.validate_transaction(&request);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| matches!(e, ValidationError::InvalidFormat { field, .. } if field == "from")));
    }

    #[test]
    fn test_validate_transaction_zero_amount() {
        let validator = create_test_validator();
        let request = TransactionValidationRequest {
            from: "0x1234567890123456789012345678901234567890".to_string(),
            to: "0x1234567890123456789012345678901234567890".to_string(), // Fixed length
            amount: 0,
            fee: 200,
            data: None,
            signature: "0x1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890".to_string(), // Fixed length
            nonce: None,
            gas_limit: None,
        };

        let result = validator.validate_transaction(&request);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| matches!(e, ValidationError::OutOfRange { field, .. } if field == "amount")));
    }

    #[test]
    fn test_validate_account_valid() {
        let validator = create_test_validator();
        let request = AccountValidationRequest {
            address: "0x1234567890123456789012345678901234567890".to_string(),
            public_key: Some("0x1234567890123456789012345678901234567890123456789012345678901234".to_string()),
        };

        let result = validator.validate_account(&request);
        assert!(result.is_valid, "Expected valid account, got errors: {:?}", result.errors);
    }

    #[test]
    fn test_validate_block_query_valid() {
        let validator = create_test_validator();
        let request = BlockValidationRequest {
            hash: Some("0x1234567890123456789012345678901234567890123456789012345678901234".to_string()),
            height: Some(100),
            limit: Some(50),
            offset: Some(0),
        };

        let result = validator.validate_block_query(&request);
        assert!(result.is_valid, "Expected valid block query, got errors: {:?}", result.errors);
    }

    #[test]
    fn test_validate_api_key_valid() {
        let validator = create_test_validator();
        let request = ApiKeyValidationRequest {
            name: "test-key".to_string(),
            permissions: vec!["read".to_string(), "write".to_string()],
            rate_limit: Some(1000),
        };

        let result = validator.validate_api_key(&request);
        assert!(result.is_valid, "Expected valid API key, got errors: {:?}", result.errors);
    }

    #[test]
    fn test_validate_api_key_invalid_permission() {
        let validator = create_test_validator();
        let request = ApiKeyValidationRequest {
            name: "test-key".to_string(),
            permissions: vec!["invalid_permission".to_string()],
            rate_limit: Some(1000),
        };

        let result = validator.validate_api_key(&request);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| matches!(e, ValidationError::InvalidFormat { field, .. } if field == "permissions")));
    }

    #[test]
    fn test_is_valid_hex() {
        let validator = create_test_validator();
        
        assert!(validator.is_valid_hex("0x1234567890abcdef"));
        assert!(validator.is_valid_hex("0x0000000000000000"));
        assert!(!validator.is_valid_hex("1234567890abcdef")); // No 0x prefix
        assert!(!validator.is_valid_hex("0x123g")); // Invalid hex character
        assert!(!validator.is_valid_hex("0x")); // Empty hex
    }

    #[test]
    fn test_sanitize_string() {
        let validator = create_test_validator();
        
        let input = "Hello\x00World\tTest\nLine";
        let sanitized = validator.sanitize_string(input, 100);
        assert_eq!(sanitized, "HelloWorld\tTest\nLine");
        
        let long_input = "a".repeat(200);
        let sanitized_long = validator.sanitize_string(&long_input, 50);
        assert_eq!(sanitized_long.len(), 50);
    }

    #[test]
    fn test_validate_pagination() {
        let validator = create_test_validator();
        
        // Valid pagination
        let result = validator.validate_pagination(Some(50), Some(0));
        assert!(result.is_valid);
        
        // Invalid limit (too high)
        let result = validator.validate_pagination(Some(2000), Some(0));
        assert!(!result.is_valid);
        
        // Invalid limit (zero)
        let result = validator.validate_pagination(Some(0), Some(0));
        assert!(!result.is_valid);
    }
}
