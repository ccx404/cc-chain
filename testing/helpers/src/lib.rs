//! CC Chain Testing Helpers
//!
//! This crate provides common utilities and helpers for testing CC Chain components.
//! It includes assertions, test data generators, and other utilities that make
//! testing more convenient and consistent across the codebase.

use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use serde::{Deserialize, Serialize};

#[derive(Error, Debug)]
pub enum TestHelperError {
    #[error("Test data generation error: {0}")]
    DataGeneration(String),
    #[error("Assertion failed: {0}")]
    AssertionFailed(String),
    #[error("Test environment error: {0}")]
    Environment(String),
}

pub type Result<T> = std::result::Result<T, TestHelperError>;

/// Test data generator utilities
pub mod generators {
    use super::*;
    
    /// Generate a random test address
    pub fn random_address() -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        SystemTime::now().hash(&mut hasher);
        format!("cc1{:016x}", hasher.finish())
    }
    
    /// Generate a random transaction hash
    pub fn random_tx_hash() -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        (SystemTime::now(), "tx").hash(&mut hasher);
        format!("{:064x}", hasher.finish())
    }
    
    /// Generate test keypair (simplified for testing)
    pub fn test_keypair() -> (String, String) {
        let private_key = "test_private_key_0123456789abcdef";
        let public_key = "test_public_key_fedcba9876543210";
        (private_key.to_string(), public_key.to_string())
    }
    
    /// Generate test block data
    pub fn test_block_data(height: u64) -> TestBlockData {
        TestBlockData {
            height,
            hash: format!("block_hash_{:016x}", height),
            parent_hash: if height == 0 {
                "genesis".to_string()
            } else {
                format!("block_hash_{:016x}", height - 1)
            },
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            transactions: vec![random_tx_hash()],
        }
    }
}

/// Assertion helpers for testing
pub mod assertions {
    use super::*;
    
    /// Assert that a result is successful
    pub fn assert_success<T, E: std::fmt::Debug>(result: std::result::Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("Expected success but got error: {:?}", error),
        }
    }
    
    /// Assert that a result is an error
    pub fn assert_error<T: std::fmt::Debug, E>(result: std::result::Result<T, E>) {
        match result {
            Ok(value) => panic!("Expected error but got success: {:?}", value),
            Err(_) => {},
        }
    }
    
    /// Assert that two durations are approximately equal (within tolerance)
    pub fn assert_duration_approx(actual: Duration, expected: Duration, tolerance: Duration) {
        let diff = if actual > expected {
            actual - expected
        } else {
            expected - actual
        };
        
        if diff > tolerance {
            panic!(
                "Duration assertion failed: actual {:?} not within {:?} of expected {:?}",
                actual, tolerance, expected
            );
        }
    }
    
    /// Assert that a collection contains all expected items
    pub fn assert_contains_all<T: PartialEq + std::fmt::Debug>(
        collection: &[T],
        expected: &[T],
    ) {
        for item in expected {
            if !collection.contains(item) {
                panic!("Collection does not contain expected item: {:?}", item);
            }
        }
    }
}

/// Test environment setup utilities
pub mod environment {
    use super::*;
    
    /// Test environment configuration
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TestEnvironment {
        pub temp_dir: String,
        pub log_level: String,
        pub test_data_path: String,
        pub cleanup_on_drop: bool,
    }
    
    impl TestEnvironment {
        /// Create a new test environment with default settings
        pub fn new() -> Result<Self> {
            let temp_dir = std::env::temp_dir()
                .join("cc_chain_test")
                .to_string_lossy()
                .to_string();
                
            Ok(TestEnvironment {
                temp_dir,
                log_level: "debug".to_string(),
                test_data_path: "test_data".to_string(),
                cleanup_on_drop: true,
            })
        }
        
        /// Setup the test environment
        pub fn setup(&self) -> Result<()> {
            // Create temp directory if it doesn't exist
            std::fs::create_dir_all(&self.temp_dir)
                .map_err(|e| TestHelperError::Environment(format!("Failed to create temp dir: {}", e)))?;
            
            // Set environment variables for testing
            std::env::set_var("CC_CHAIN_LOG_LEVEL", &self.log_level);
            std::env::set_var("CC_CHAIN_TEST_MODE", "true");
            
            Ok(())
        }
        
        /// Get path for test file
        pub fn test_file_path(&self, filename: &str) -> String {
            format!("{}/{}", self.temp_dir, filename)
        }
    }
    
    impl Drop for TestEnvironment {
        fn drop(&mut self) {
            if self.cleanup_on_drop {
                let _ = std::fs::remove_dir_all(&self.temp_dir);
            }
        }
    }
}

/// Test data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestBlockData {
    pub height: u64,
    pub hash: String,
    pub parent_hash: String,
    pub timestamp: u64,
    pub transactions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestTransactionData {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub gas_price: u64,
    pub gas_limit: u64,
}

/// Timing utilities for performance tests
pub mod timing {
    use super::*;
    use std::time::Instant;
    
    /// Simple timer for measuring execution time
    pub struct Timer {
        start: Instant,
        name: String,
    }
    
    impl Timer {
        /// Create a new named timer
        pub fn new(name: &str) -> Self {
            Timer {
                start: Instant::now(),
                name: name.to_string(),
            }
        }
        
        /// Get elapsed time since timer creation
        pub fn elapsed(&self) -> Duration {
            self.start.elapsed()
        }
        
        /// Stop the timer and return elapsed time
        pub fn stop(self) -> Duration {
            let elapsed = self.elapsed();
            println!("Timer '{}': {:?}", self.name, elapsed);
            elapsed
        }
    }
    
    /// Measure execution time of a closure
    pub fn measure<F, R>(name: &str, f: F) -> (R, Duration)
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let elapsed = start.elapsed();
        println!("Measured '{}': {:?}", name, elapsed);
        (result, elapsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_random_address_generation() {
        let addr1 = generators::random_address();
        let addr2 = generators::random_address();
        
        assert!(addr1.starts_with("cc1"));
        assert!(addr2.starts_with("cc1"));
        assert_ne!(addr1, addr2); // Should be different
    }
    
    #[test]
    fn test_random_tx_hash_generation() {
        let hash1 = generators::random_tx_hash();
        let hash2 = generators::random_tx_hash();
        
        assert_eq!(hash1.len(), 64); // Expected length for hex hash
        assert_eq!(hash2.len(), 64);
        assert_ne!(hash1, hash2); // Should be different
    }
    
    #[test]
    fn test_keypair_generation() {
        let (private, public) = generators::test_keypair();
        assert!(private.contains("private"));
        assert!(public.contains("public"));
    }
    
    #[test]
    fn test_block_data_generation() {
        let block = generators::test_block_data(42);
        assert_eq!(block.height, 42);
        assert!(block.hash.contains("block_hash"));
        assert!(!block.transactions.is_empty());
    }
    
    #[test]
    fn test_assertion_helpers() {
        let success: std::result::Result<i32, String> = Ok(42);
        let value = assertions::assert_success(success);
        assert_eq!(value, 42);
        
        let error: std::result::Result<i32, String> = Err("test error".to_string());
        assertions::assert_error(error);
    }
    
    #[test]
    fn test_duration_assertion() {
        let duration1 = Duration::from_millis(100);
        let duration2 = Duration::from_millis(105);
        let tolerance = Duration::from_millis(10);
        
        assertions::assert_duration_approx(duration1, duration2, tolerance);
    }
    
    #[test]
    fn test_contains_all() {
        let collection = vec![1, 2, 3, 4, 5];
        let expected = vec![2, 4];
        assertions::assert_contains_all(&collection, &expected);
    }
    
    #[test]
    fn test_environment_setup() {
        let env = environment::TestEnvironment::new().unwrap();
        env.setup().unwrap();
        
        assert!(env.temp_dir.contains("cc_chain_test"));
        assert_eq!(std::env::var("CC_CHAIN_TEST_MODE").unwrap(), "true");
    }
    
    #[test]
    fn test_timer() {
        let timer = timing::Timer::new("test");
        std::thread::sleep(Duration::from_millis(10));
        let elapsed = timer.elapsed();
        assert!(elapsed >= Duration::from_millis(10));
    }
    
    #[test]
    fn test_measure() {
        let (result, duration) = timing::measure("test_computation", || {
            std::thread::sleep(Duration::from_millis(5));
            42
        });
        
        assert_eq!(result, 42);
        assert!(duration >= Duration::from_millis(5));
    }
}
