//! CC Chain Unit Testing Utilities
//!
//! This crate provides specialized utilities for unit testing CC Chain components.
//! It includes test fixtures, assertion macros, and component-specific test helpers.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UnitTestError {
    #[error("Unit test setup error: {0}")]
    Setup(String),
    #[error("Assertion failed: {0}")]
    AssertionFailed(String),
}

pub type Result<T> = std::result::Result<T, UnitTestError>;

/// Unit test context
#[derive(Debug, Clone)]
pub struct UnitTestContext {
    pub test_name: String,
    pub test_data_dir: String,
    pub cleanup_required: bool,
}

impl UnitTestContext {
    /// Create a new unit test context
    pub fn new(test_name: &str) -> Self {
        UnitTestContext {
            test_name: test_name.to_string(),
            test_data_dir: format!("/tmp/cc_chain_unit_test_{}", test_name),
            cleanup_required: true,
        }
    }
    
    /// Setup the test environment
    pub fn setup(&self) -> Result<()> {
        std::fs::create_dir_all(&self.test_data_dir)
            .map_err(|e| UnitTestError::Setup(format!("Failed to create test dir: {}", e)))?;
        Ok(())
    }
    
    /// Cleanup the test environment
    pub fn cleanup(&self) -> Result<()> {
        if self.cleanup_required {
            std::fs::remove_dir_all(&self.test_data_dir)
                .map_err(|e| UnitTestError::Setup(format!("Failed to cleanup test dir: {}", e)))?;
        }
        Ok(())
    }
}

/// Specialized assertion macros for CC Chain components
pub mod assertions {
    use super::*;
    
    /// Assert that a blockchain operation is successful
    pub fn assert_blockchain_success<T, E: std::fmt::Debug>(
        operation: std::result::Result<T, E>
    ) -> Result<T> {
        operation.map_err(|e| UnitTestError::AssertionFailed(
            format!("Blockchain operation failed: {:?}", e)
        ))
    }
    
    /// Assert that two hash values are equal
    pub fn assert_hash_equal(expected: &str, actual: &str) -> Result<()> {
        if expected != actual {
            return Err(UnitTestError::AssertionFailed(
                format!("Hash mismatch: expected '{}', got '{}'", expected, actual)
            ));
        }
        Ok(())
    }
    
    /// Assert that a transaction is valid
    pub fn assert_transaction_valid(tx_hash: &str, from: &str, to: &str, amount: u64) -> Result<()> {
        // Simplified validation for testing
        if tx_hash.is_empty() || from.is_empty() || to.is_empty() || amount == 0 {
            return Err(UnitTestError::AssertionFailed(
                "Invalid transaction parameters".to_string()
            ));
        }
        Ok(())
    }
}

/// Test data builders for unit tests
pub mod builders {
    use super::*;
    
    /// Mock transaction builder
    #[derive(Debug, Clone)]
    pub struct TransactionBuilder {
        hash: String,
        from: String,
        to: String,
        amount: u64,
    }
    
    impl TransactionBuilder {
        pub fn new() -> Self {
            TransactionBuilder {
                hash: "default_hash".to_string(),
                from: "default_from".to_string(),
                to: "default_to".to_string(),
                amount: 100,
            }
        }
        
        pub fn with_hash(mut self, hash: &str) -> Self {
            self.hash = hash.to_string();
            self
        }
        
        pub fn with_from(mut self, from: &str) -> Self {
            self.from = from.to_string();
            self
        }
        
        pub fn with_to(mut self, to: &str) -> Self {
            self.to = to.to_string();
            self
        }
        
        pub fn with_amount(mut self, amount: u64) -> Self {
            self.amount = amount;
            self
        }
        
        pub fn build(self) -> MockTransaction {
            MockTransaction {
                hash: self.hash,
                from: self.from,
                to: self.to,
                amount: self.amount,
            }
        }
    }
    
    impl Default for TransactionBuilder {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockTransaction {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_unit_test_context() {
        let context = UnitTestContext::new("test_context");
        assert_eq!(context.test_name, "test_context");
        assert!(context.test_data_dir.contains("test_context"));
    }
    
    #[test]
    fn test_transaction_builder() {
        let tx = builders::TransactionBuilder::new()
            .with_hash("tx123")
            .with_from("alice")
            .with_to("bob")
            .with_amount(500)
            .build();
            
        assert_eq!(tx.hash, "tx123");
        assert_eq!(tx.from, "alice");
        assert_eq!(tx.to, "bob");
        assert_eq!(tx.amount, 500);
    }
    
    #[test]
    fn test_assertions() {
        assertions::assert_hash_equal("hash1", "hash1").unwrap();
        
        let result = assertions::assert_hash_equal("hash1", "hash2");
        assert!(result.is_err());
        
        assertions::assert_transaction_valid("tx1", "alice", "bob", 100).unwrap();
        
        let result = assertions::assert_transaction_valid("", "alice", "bob", 100);
        assert!(result.is_err());
    }
}
