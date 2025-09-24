//! CC Chain Testing Utilities
//!
//! This crate provides general-purpose testing utilities and convenience functions
//! for CC Chain testing. It includes test runners, comparison utilities, logging
//! helpers, and other common testing patterns.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UtilityError {
    #[error("Test execution error: {0}")]
    Execution(String),
    #[error("Comparison error: {0}")]
    Comparison(String),
    #[error("Configuration error: {0}")]
    Configuration(String),
}

pub type Result<T> = std::result::Result<T, UtilityError>;

/// Test execution context and runner
pub struct TestRunner {
    name: String,
    setup_functions: Vec<Box<dyn Fn() -> Result<()>>>,
    cleanup_functions: Vec<Box<dyn Fn() -> Result<()>>>,
    config: TestConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    pub timeout: Duration,
    pub retry_count: u32,
    pub parallel: bool,
    pub verbose: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        TestConfig {
            timeout: Duration::from_secs(30),
            retry_count: 0,
            parallel: false,
            verbose: false,
        }
    }
}

/// Test result with detailed information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub name: String,
    pub success: bool,
    pub duration: Duration,
    pub error_message: Option<String>,
    pub retry_count: u32,
    pub metrics: HashMap<String, f64>,
}

/// Comparison utilities for testing
pub mod comparison {
    use super::*;
    
    /// Compare two byte arrays with detailed diff information
    pub fn compare_bytes(expected: &[u8], actual: &[u8]) -> Result<()> {
        if expected.len() != actual.len() {
            return Err(UtilityError::Comparison(
                format!("Length mismatch: expected {}, got {}", expected.len(), actual.len())
            ));
        }
        
        for (i, (exp, act)) in expected.iter().zip(actual.iter()).enumerate() {
            if exp != act {
                return Err(UtilityError::Comparison(
                    format!("Byte mismatch at index {}: expected 0x{:02x}, got 0x{:02x}", i, exp, act)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Compare JSON strings with structural comparison
    pub fn compare_json_strings(expected: &str, actual: &str) -> Result<()> {
        let expected_value: serde_json::Value = serde_json::from_str(expected)
            .map_err(|e| UtilityError::Comparison(format!("Failed to parse expected JSON: {}", e)))?;
        
        let actual_value: serde_json::Value = serde_json::from_str(actual)
            .map_err(|e| UtilityError::Comparison(format!("Failed to parse actual JSON: {}", e)))?;
        
        if expected_value != actual_value {
            return Err(UtilityError::Comparison(
                format!("JSON mismatch:\nExpected: {}\nActual: {}", expected, actual)
            ));
        }
        
        Ok(())
    }
    
    /// Compare floating point numbers with tolerance
    pub fn compare_floats(expected: f64, actual: f64, tolerance: f64) -> Result<()> {
        let diff = (expected - actual).abs();
        if diff > tolerance {
            return Err(UtilityError::Comparison(
                format!("Float comparison failed: expected {}, got {} (diff: {}, tolerance: {})",
                    expected, actual, diff, tolerance)
            ));
        }
        Ok(())
    }
    
    /// Compare collections ignoring order
    pub fn compare_unordered<T: PartialEq + std::fmt::Debug>(expected: &[T], actual: &[T]) -> Result<()> {
        if expected.len() != actual.len() {
            return Err(UtilityError::Comparison(
                format!("Collection length mismatch: expected {}, got {}", expected.len(), actual.len())
            ));
        }
        
        for item in expected {
            if !actual.contains(item) {
                return Err(UtilityError::Comparison(
                    format!("Expected item not found in actual collection: {:?}", item)
                ));
            }
        }
        
        Ok(())
    }
}

/// Retry utilities for flaky tests
pub mod retry {
    use super::*;
    use std::thread;
    
    /// Retry a test function with exponential backoff
    pub fn with_exponential_backoff<F, T, E>(
        mut f: F,
        max_retries: u32,
        initial_delay: Duration,
    ) -> std::result::Result<T, E>
    where
        F: FnMut() -> std::result::Result<T, E>,
    {
        let mut delay = initial_delay;
        
        for attempt in 0..=max_retries {
            match f() {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt == max_retries {
                        return Err(e);
                    }
                    thread::sleep(delay);
                    delay = delay * 2; // Exponential backoff
                }
            }
        }
        
        unreachable!()
    }
    
    /// Retry a test with fixed intervals
    pub fn with_fixed_interval<F, T, E>(
        mut f: F,
        max_retries: u32,
        interval: Duration,
    ) -> std::result::Result<T, E>
    where
        F: FnMut() -> std::result::Result<T, E>,
    {
        for attempt in 0..=max_retries {
            match f() {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt == max_retries {
                        return Err(e);
                    }
                    thread::sleep(interval);
                }
            }
        }
        
        unreachable!()
    }
}

/// Logging utilities for tests
pub mod logging {
    use super::*;
    
    /// Test logger that captures logs for verification
    pub struct TestLogger {
        logs: Vec<LogEntry>,
        level: LogLevel,
    }
    
    #[derive(Debug, Clone)]
    pub struct LogEntry {
        pub level: LogLevel,
        pub message: String,
        pub timestamp: std::time::SystemTime,
    }
    
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub enum LogLevel {
        Trace,
        Debug,
        Info,
        Warn,
        Error,
    }
    
    impl TestLogger {
        /// Create a new test logger
        pub fn new(level: LogLevel) -> Self {
            TestLogger {
                logs: Vec::new(),
                level,
            }
        }
        
        /// Log a message at the specified level
        pub fn log(&mut self, level: LogLevel, message: &str) {
            if level >= self.level {
                self.logs.push(LogEntry {
                    level,
                    message: message.to_string(),
                    timestamp: std::time::SystemTime::now(),
                });
            }
        }
        
        /// Get all captured logs
        pub fn get_logs(&self) -> &[LogEntry] {
            &self.logs
        }
        
        /// Get logs at or above a certain level
        pub fn get_logs_at_level(&self, min_level: LogLevel) -> Vec<&LogEntry> {
            self.logs.iter().filter(|entry| entry.level >= min_level).collect()
        }
        
        /// Clear all captured logs
        pub fn clear(&mut self) {
            self.logs.clear();
        }
        
        /// Check if any error logs were captured
        pub fn has_errors(&self) -> bool {
            self.logs.iter().any(|entry| entry.level == LogLevel::Error)
        }
    }
}

/// Wait and polling utilities for async testing
pub mod wait {
    use super::*;
    use std::thread;
    
    /// Wait for a condition to become true with timeout
    pub fn for_condition<F>(condition: F, timeout: Duration) -> Result<()>
    where
        F: Fn() -> bool,
    {
        let start = Instant::now();
        let check_interval = Duration::from_millis(10);
        
        while start.elapsed() < timeout {
            if condition() {
                return Ok(());
            }
            thread::sleep(check_interval);
        }
        
        Err(UtilityError::Execution(
            format!("Condition not met within timeout of {:?}", timeout)
        ))
    }
    
    /// Poll a function until it succeeds or times out
    pub fn poll_until_success<F, T, E>(mut f: F, timeout: Duration, interval: Duration) -> Result<T>
    where
        F: FnMut() -> std::result::Result<T, E>,
        E: std::fmt::Debug,
    {
        let start = Instant::now();
        
        loop {
            match f() {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if start.elapsed() >= timeout {
                        return Err(UtilityError::Execution(
                            format!("Polling timed out after {:?}. Last error: {:?}", timeout, e)
                        ));
                    }
                    thread::sleep(interval);
                }
            }
        }
    }
}

/// Random test data generation utilities
pub mod random {
    use super::*;
    
    /// Generate random string of specified length
    pub fn string(length: usize) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        std::time::SystemTime::now().hash(&mut hasher);
        
        let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
            .chars()
            .collect();
        
        let hash = hasher.finish();
        let mut result = String::new();
        let mut seed = hash;
        
        for _ in 0..length {
            let index = (seed as usize) % chars.len();
            result.push(chars[index]);
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345); // Linear congruential generator
        }
        
        result
    }
    
    /// Generate random bytes
    pub fn bytes(length: usize) -> Vec<u8> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        std::time::SystemTime::now().hash(&mut hasher);
        
        let mut result = Vec::with_capacity(length);
        let mut seed = hasher.finish();
        
        for _ in 0..length {
            result.push((seed & 0xff) as u8);
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        }
        
        result
    }
    
    /// Generate random number in range
    pub fn number_in_range(min: u64, max: u64) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        std::time::SystemTime::now().hash(&mut hasher);
        
        let hash = hasher.finish();
        min + (hash % (max - min + 1))
    }
}

/// File system utilities for test isolation
pub mod filesystem {
    use super::*;
    use std::path::{Path, PathBuf};
    
    /// Temporary directory that cleans up on drop
    pub struct TempDir {
        path: PathBuf,
        cleanup: bool,
    }
    
    impl TempDir {
        /// Create a new temporary directory
        pub fn new() -> Result<Self> {
            let path = std::env::temp_dir().join(format!("cc_test_{}", uuid::Uuid::new_v4()));
            
            std::fs::create_dir_all(&path)
                .map_err(|e| UtilityError::Configuration(format!("Failed to create temp dir: {}", e)))?;
            
            Ok(TempDir {
                path,
                cleanup: true,
            })
        }
        
        /// Get the path to the temporary directory
        pub fn path(&self) -> &Path {
            &self.path
        }
        
        /// Create a file in the temporary directory
        pub fn create_file(&self, name: &str, contents: &str) -> Result<PathBuf> {
            let file_path = self.path.join(name);
            std::fs::write(&file_path, contents)
                .map_err(|e| UtilityError::Configuration(format!("Failed to create file: {}", e)))?;
            Ok(file_path)
        }
        
        /// Disable cleanup on drop
        pub fn persist(mut self) -> PathBuf {
            self.cleanup = false;
            self.path.clone()
        }
    }
    
    impl Drop for TempDir {
        fn drop(&mut self) {
            if self.cleanup {
                let _ = std::fs::remove_dir_all(&self.path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compare_bytes() {
        let expected = b"hello world";
        let actual = b"hello world";
        
        comparison::compare_bytes(expected, actual).unwrap();
        
        let actual_different = b"hello world!";
        assert!(comparison::compare_bytes(expected, actual_different).is_err());
    }
    
    #[test]
    fn test_compare_json() {
        let expected = r#"{"name": "test", "value": 42}"#;
        let actual = r#"{"value": 42, "name": "test"}"#; // Different order, should be equal
        
        comparison::compare_json_strings(expected, actual).unwrap();
        
        let actual_different = r#"{"name": "test", "value": 43}"#;
        assert!(comparison::compare_json_strings(expected, actual_different).is_err());
    }
    
    #[test]
    fn test_compare_floats() {
        comparison::compare_floats(1.0, 1.001, 0.01).unwrap();
        assert!(comparison::compare_floats(1.0, 1.1, 0.01).is_err());
    }
    
    #[test]
    fn test_compare_unordered() {
        let expected = vec![1, 2, 3];
        let actual = vec![3, 1, 2];
        
        comparison::compare_unordered(&expected, &actual).unwrap();
        
        let actual_different = vec![1, 2, 4];
        assert!(comparison::compare_unordered(&expected, &actual_different).is_err());
    }
    
    #[test]
    fn test_retry_with_success() {
        let mut attempts = 0;
        let result = retry::with_fixed_interval(
            || {
                attempts += 1;
                if attempts >= 3 {
                    Ok("success")
                } else {
                    Err("failure")
                }
            },
            5,
            Duration::from_millis(1),
        );
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(attempts, 3);
    }
    
    #[test]
    fn test_test_logger() {
        let mut logger = logging::TestLogger::new(logging::LogLevel::Info);
        
        logger.log(logging::LogLevel::Debug, "debug message"); // Should be filtered
        logger.log(logging::LogLevel::Info, "info message");
        logger.log(logging::LogLevel::Error, "error message");
        
        assert_eq!(logger.get_logs().len(), 2); // Debug message filtered
        assert!(logger.has_errors());
        
        let error_logs = logger.get_logs_at_level(logging::LogLevel::Error);
        assert_eq!(error_logs.len(), 1);
    }
    
    #[test]
    fn test_wait_for_condition() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicU32, Ordering};
        
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = Arc::clone(&counter);
        
        let result = wait::for_condition(
            move || {
                counter_clone.fetch_add(1, Ordering::Relaxed);
                counter_clone.load(Ordering::Relaxed) >= 3
            },
            Duration::from_millis(100),
        );
        
        assert!(result.is_ok());
        assert!(counter.load(Ordering::Relaxed) >= 3);
    }
    
    #[test]
    fn test_random_generation() {
        let s1 = random::string(10);
        let s2 = random::string(10);
        
        assert_eq!(s1.len(), 10);
        assert_eq!(s2.len(), 10);
        // Note: There's a small chance they could be equal, but very unlikely
        
        let bytes = random::bytes(5);
        assert_eq!(bytes.len(), 5);
        
        let num = random::number_in_range(10, 20);
        assert!(num >= 10 && num <= 20);
    }
    
    #[test]
    fn test_temp_dir() {
        let temp_dir = filesystem::TempDir::new().unwrap();
        
        let file_path = temp_dir.create_file("test.txt", "hello world").unwrap();
        assert!(file_path.exists());
        
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "hello world");
        
        // Directory should be cleaned up when temp_dir goes out of scope
    }
}
