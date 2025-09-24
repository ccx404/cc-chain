//! CC Chain Testing Framework
//! 
//! This module provides comprehensive testing functionality for the CC Chain blockchain.
//! It includes utilities for unit testing, integration testing, performance testing,
//! stress testing, benchmarking, mocking, fixtures, and more.

// Re-export all testing submodules for easy access
pub use testing_benchmarks as benchmarks;
pub use testing_fixtures as fixtures;
pub use testing_helpers as helpers;
pub use testing_integration as integration;
pub use testing_mocks as mocks;
pub use testing_performance as performance;
pub use testing_stress as stress;
pub use testing_unit as unit;
pub use testing_utilities as utilities;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Main testing framework facade
pub struct TestingFramework {
    pub fixtures: fixtures::FixtureProvider,
    pub mocks: mocks::MockBlockchain,
}

impl TestingFramework {
    /// Create a new testing framework instance
    pub fn new() -> Self {
        TestingFramework {
            fixtures: fixtures::FixtureProvider::new(),
            mocks: mocks::MockBlockchain::new(),
        }
    }
}

impl Default for TestingFramework {
    fn default() -> Self {
        Self::new()
    }
}

/// Common testing utilities available at the root level
pub mod common {
    /// Generate a random test address
    pub fn random_address() -> String {
        crate::helpers::generators::random_address()
    }
    
    /// Get Alice's test account
    pub fn alice_account() -> &'static crate::fixtures::AccountFixture {
        crate::fixtures::common::alice()
    }
    
    /// Get Bob's test account  
    pub fn bob_account() -> &'static crate::fixtures::AccountFixture {
        crate::fixtures::common::bob()
    }
    
    /// Create a new mock blockchain
    pub fn mock_blockchain() -> crate::mocks::MockBlockchain {
        crate::mocks::MockBlockchain::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }
    
    #[test]
    fn test_framework_creation() {
        let _framework = TestingFramework::new();
        // Just verify it can be created without errors
    }
    
    #[test] 
    fn test_common_utilities() {
        let addr = common::random_address();
        assert!(addr.starts_with("cc1"));
        
        let alice = common::alice_account();
        assert!(alice.address.contains("alice"));
        
        let bob = common::bob_account();
        assert!(bob.address.contains("bob"));
        
        let _blockchain = common::mock_blockchain();
        // Just verify it can be created
    }
}
