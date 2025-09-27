//! Core utilities functionality
//!
//! This module provides essential utility functions and helpers used throughout
//! the CC Chain system including formatting, conversion, validation, and common operations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Utility-related errors
#[derive(Error, Debug, Clone, PartialEq)]
pub enum UtilityError {
    #[error("Invalid format: {input}")]
    InvalidFormat { input: String },
    #[error("Conversion failed: {from} to {to}")]
    ConversionFailed { from: String, to: String },
    #[error("Validation failed: {field} = {value}")]
    ValidationFailed { field: String, value: String },
    #[error("Encoding error: {message}")]
    EncodingError { message: String },
    #[error("Configuration error: {parameter}")]
    ConfigurationError { parameter: String },
}

pub type Result<T> = std::result::Result<T, UtilityError>;

/// Common formatting utilities
pub struct Formatter;

impl Formatter {
    /// Format bytes as human-readable string
    pub fn format_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
        const THRESHOLD: u64 = 1024;

        if bytes == 0 {
            return "0 B".to_string();
        }

        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= THRESHOLD as f64 && unit_index < UNITS.len() - 1 {
            size /= THRESHOLD as f64;
            unit_index += 1;
        }

        format!("{:.2} {}", size, UNITS[unit_index])
    }

    /// Format duration as human-readable string
    pub fn format_duration(duration: Duration) -> String {
        let total_seconds = duration.as_secs();
        
        if total_seconds == 0 {
            return format!("{}ms", duration.as_millis());
        }

        let days = total_seconds / 86400;
        let hours = (total_seconds % 86400) / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        let mut parts = Vec::new();

        if days > 0 {
            parts.push(format!("{}d", days));
        }
        if hours > 0 {
            parts.push(format!("{}h", hours));
        }
        if minutes > 0 {
            parts.push(format!("{}m", minutes));
        }
        if seconds > 0 || parts.is_empty() {
            parts.push(format!("{}s", seconds));
        }

        parts.join(" ")
    }

    /// Format timestamp as ISO 8601 string
    pub fn format_timestamp(timestamp: u64) -> String {
        let datetime = SystemTime::UNIX_EPOCH + Duration::from_secs(timestamp);
        // Simple formatting - in production would use chrono
        format!("{:?}", datetime)
    }

    /// Format hash as short string with prefix/suffix
    pub fn format_hash_short(hash: &str, prefix_len: usize, suffix_len: usize) -> String {
        if hash.len() <= prefix_len + suffix_len {
            return hash.to_string();
        }

        format!("{}...{}", 
                &hash[..prefix_len], 
                &hash[hash.len() - suffix_len..])
    }

    /// Format number with thousand separators
    pub fn format_number(num: u64) -> String {
        let num_str = num.to_string();
        let chars: Vec<char> = num_str.chars().collect();
        let mut result = String::new();

        for (i, ch) in chars.iter().enumerate() {
            if i > 0 && (chars.len() - i) % 3 == 0 {
                result.push(',');
            }
            result.push(*ch);
        }

        result
    }

    /// Format percentage
    pub fn format_percentage(numerator: f64, denominator: f64, decimal_places: usize) -> String {
        if denominator == 0.0 {
            return "0.00%".to_string();
        }
        
        let percentage = (numerator / denominator) * 100.0;
        format!("{:.precision$}%", percentage, precision = decimal_places)
    }
}

/// Validation utilities
pub struct Validator;

impl Validator {
    /// Validate hexadecimal string
    pub fn is_valid_hex(input: &str) -> bool {
        if input.is_empty() {
            return false;
        }
        
        // Remove optional 0x prefix
        let hex_str = input.strip_prefix("0x").unwrap_or(input);
        
        hex_str.chars().all(|c| c.is_ascii_hexdigit())
    }

    /// Validate hex string with specific length
    pub fn is_valid_hex_with_length(input: &str, expected_length: usize) -> bool {
        Self::is_valid_hex(input) && {
            let hex_str = input.strip_prefix("0x").unwrap_or(input);
            hex_str.len() == expected_length
        }
    }

    /// Validate address format (simplified)
    pub fn is_valid_address(address: &str) -> bool {
        !address.is_empty() && 
        address.len() >= 3 && 
        address.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    }

    /// Validate numeric string
    pub fn is_valid_number(input: &str) -> bool {
        input.parse::<u64>().is_ok()
    }

    /// Validate amount (must be positive)
    pub fn is_valid_amount(amount: u64) -> bool {
        amount > 0
    }

    /// Validate fee (must be non-negative)
    pub fn is_valid_fee(_fee: u64) -> bool {
        true // All u64 values are valid fees
    }

    /// Validate nonce (must be non-negative)
    pub fn is_valid_nonce(_nonce: u64) -> bool {
        true // All u64 values are valid nonces
    }

    /// Validate gas limit
    pub fn is_valid_gas_limit(gas_limit: u64) -> bool {
        gas_limit > 0 && gas_limit <= 10_000_000 // Reasonable upper bound
    }

    /// Validate gas price
    pub fn is_valid_gas_price(gas_price: u64) -> bool {
        gas_price > 0
    }
}

/// Conversion utilities
pub struct Converter;

impl Converter {
    /// Convert hex string to bytes
    pub fn hex_to_bytes(hex_str: &str) -> Result<Vec<u8>> {
        // Remove optional 0x prefix
        let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
        
        if !Validator::is_valid_hex(hex_str) {
            return Err(UtilityError::InvalidFormat {
                input: hex_str.to_string(),
            });
        }

        hex::decode(hex_str).map_err(|_| UtilityError::ConversionFailed {
            from: "hex".to_string(),
            to: "bytes".to_string(),
        })
    }

    /// Convert bytes to hex string
    pub fn bytes_to_hex(bytes: &[u8]) -> String {
        format!("0x{}", hex::encode(bytes))
    }

    /// Convert string to u64
    pub fn str_to_u64(s: &str) -> Result<u64> {
        s.parse().map_err(|_| UtilityError::ConversionFailed {
            from: "string".to_string(),
            to: "u64".to_string(),
        })
    }

    /// Convert timestamp to duration since epoch
    pub fn timestamp_to_duration(timestamp: u64) -> Duration {
        Duration::from_secs(timestamp)
    }

    /// Convert duration to milliseconds
    pub fn duration_to_ms(duration: Duration) -> u64 {
        duration.as_millis() as u64
    }

    /// Convert milliseconds to duration
    pub fn ms_to_duration(ms: u64) -> Duration {
        Duration::from_millis(ms)
    }
}

/// Configuration utilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    /// Create new config
    pub fn new() -> Self {
        Config {
            values: HashMap::new(),
        }
    }

    /// Set configuration value
    pub fn set(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }

    /// Get configuration value
    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    /// Get configuration value with default
    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).cloned().unwrap_or_else(|| default.to_string())
    }

    /// Get configuration value as u64
    pub fn get_u64(&self, key: &str) -> Result<u64> {
        let value = self.values.get(key)
            .ok_or_else(|| UtilityError::ConfigurationError {
                parameter: key.to_string(),
            })?;
        
        Converter::str_to_u64(value)
    }

    /// Get configuration value as u64 with default
    pub fn get_u64_or_default(&self, key: &str, default: u64) -> u64 {
        self.get_u64(key).unwrap_or(default)
    }

    /// Get configuration value as bool
    pub fn get_bool(&self, key: &str) -> Result<bool> {
        let value = self.values.get(key)
            .ok_or_else(|| UtilityError::ConfigurationError {
                parameter: key.to_string(),
            })?;
        
        match value.to_lowercase().as_str() {
            "true" | "yes" | "1" | "on" => Ok(true),
            "false" | "no" | "0" | "off" => Ok(false),
            _ => Err(UtilityError::ConversionFailed {
                from: "string".to_string(),
                to: "bool".to_string(),
            })
        }
    }

    /// Get configuration value as bool with default
    pub fn get_bool_or_default(&self, key: &str, default: bool) -> bool {
        self.get_bool(key).unwrap_or(default)
    }

    /// Check if configuration key exists
    pub fn has(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    /// Remove configuration key
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.values.remove(key)
    }

    /// Get all keys
    pub fn keys(&self) -> Vec<&String> {
        self.values.keys().collect()
    }

    /// Get number of configuration entries
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Check if config is empty
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Clear all configuration
    pub fn clear(&mut self) {
        self.values.clear();
    }

    /// Merge with another config
    pub fn merge(&mut self, other: &Config) {
        for (key, value) in &other.values {
            self.values.insert(key.clone(), value.clone());
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// Rate limiter utility
#[derive(Debug)]
pub struct RateLimiter {
    max_requests: u32,
    time_window: Duration,
    requests: Vec<SystemTime>,
}

impl RateLimiter {
    /// Create new rate limiter
    pub fn new(max_requests: u32, time_window: Duration) -> Self {
        RateLimiter {
            max_requests,
            time_window,
            requests: Vec::with_capacity(max_requests as usize),
        }
    }

    /// Check if request is allowed
    pub fn is_allowed(&mut self) -> bool {
        let now = SystemTime::now();
        
        // Remove old requests outside the time window
        self.requests.retain(|&request_time| {
            now.duration_since(request_time).unwrap_or(Duration::MAX) <= self.time_window
        });

        // Check if we can add another request
        if self.requests.len() < self.max_requests as usize {
            self.requests.push(now);
            true
        } else {
            false
        }
    }

    /// Get current request count in window
    pub fn current_count(&self) -> usize {
        self.requests.len()
    }

    /// Get time until next request is allowed
    pub fn time_until_next_allowed(&self) -> Option<Duration> {
        if self.requests.len() < self.max_requests as usize {
            return None;
        }

        let oldest_request = self.requests.first()?;
        let time_since_oldest = SystemTime::now()
            .duration_since(*oldest_request)
            .unwrap_or(Duration::ZERO);

        if time_since_oldest < self.time_window {
            Some(self.time_window - time_since_oldest)
        } else {
            None
        }
    }

    /// Reset rate limiter
    pub fn reset(&mut self) {
        self.requests.clear();
    }
}

/// ID generation utilities
pub struct IdGenerator;

impl IdGenerator {
    /// Generate random ID
    pub fn generate_id() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let id: u64 = rng.gen();
        format!("{:016x}", id)
    }

    /// Generate UUID v4 (simplified)
    pub fn generate_uuid() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    /// Generate session ID
    pub fn generate_session_id() -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        format!("{}_{}", timestamp, Self::generate_id())
    }

    /// Generate transaction ID from data
    pub fn generate_tx_id(from: &str, to: &str, amount: u64, nonce: u64) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(from.as_bytes());
        hasher.update(to.as_bytes());
        hasher.update(amount.to_le_bytes());
        hasher.update(nonce.to_le_bytes());
        
        let hash = hasher.finalize();
        hex::encode(hash)
    }
}

/// Math utilities
pub struct MathUtils;

impl MathUtils {
    /// Calculate percentage
    pub fn percentage(part: u64, total: u64) -> f64 {
        if total == 0 {
            0.0
        } else {
            (part as f64 / total as f64) * 100.0
        }
    }

    /// Calculate average
    pub fn average(values: &[u64]) -> f64 {
        if values.is_empty() {
            0.0
        } else {
            values.iter().sum::<u64>() as f64 / values.len() as f64
        }
    }

    /// Find median
    pub fn median(values: &[u64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        let mut sorted = values.to_vec();
        sorted.sort_unstable();

        let len = sorted.len();
        if len % 2 == 0 {
            (sorted[len / 2 - 1] + sorted[len / 2]) as f64 / 2.0
        } else {
            sorted[len / 2] as f64
        }
    }

    /// Calculate standard deviation
    pub fn standard_deviation(values: &[u64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }

        let mean = Self::average(values);
        let variance = values.iter()
            .map(|&x| {
                let diff = x as f64 - mean;
                diff * diff
            })
            .sum::<f64>() / values.len() as f64;

        variance.sqrt()
    }

    /// Safe addition that doesn't overflow
    pub fn safe_add(a: u64, b: u64) -> Option<u64> {
        a.checked_add(b)
    }

    /// Safe subtraction that doesn't underflow
    pub fn safe_sub(a: u64, b: u64) -> Option<u64> {
        a.checked_sub(b)
    }

    /// Safe multiplication that doesn't overflow
    pub fn safe_mul(a: u64, b: u64) -> Option<u64> {
        a.checked_mul(b)
    }

    /// Safe division
    pub fn safe_div(a: u64, b: u64) -> Option<u64> {
        if b == 0 {
            None
        } else {
            Some(a / b)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formatter_bytes() {
        assert_eq!(Formatter::format_bytes(0), "0 B");
        assert_eq!(Formatter::format_bytes(1024), "1.00 KB");
        assert_eq!(Formatter::format_bytes(1048576), "1.00 MB");
        assert_eq!(Formatter::format_bytes(1073741824), "1.00 GB");
    }

    #[test]
    fn test_formatter_duration() {
        assert_eq!(Formatter::format_duration(Duration::from_secs(0)), "0ms");
        assert_eq!(Formatter::format_duration(Duration::from_secs(61)), "1m 1s");
        assert_eq!(Formatter::format_duration(Duration::from_secs(3661)), "1h 1m 1s");
    }

    #[test]
    fn test_formatter_hash_short() {
        let hash = "0123456789abcdef0123456789abcdef01234567";
        assert_eq!(Formatter::format_hash_short(hash, 4, 4), "0123...4567");
        assert_eq!(Formatter::format_hash_short("abc", 4, 4), "abc");
    }

    #[test]
    fn test_formatter_number() {
        assert_eq!(Formatter::format_number(1000), "1,000");
        assert_eq!(Formatter::format_number(1234567), "1,234,567");
    }

    #[test]
    fn test_validator_hex() {
        assert!(Validator::is_valid_hex("abc123"));
        assert!(Validator::is_valid_hex("0xabc123"));
        assert!(!Validator::is_valid_hex("xyz123"));
        assert!(!Validator::is_valid_hex(""));
    }

    #[test]
    fn test_validator_hex_length() {
        assert!(Validator::is_valid_hex_with_length("abc123", 6));
        assert!(Validator::is_valid_hex_with_length("0xabc123", 6));
        assert!(!Validator::is_valid_hex_with_length("abc123", 8));
    }

    #[test]
    fn test_validator_address() {
        assert!(Validator::is_valid_address("alice"));
        assert!(Validator::is_valid_address("user_123"));
        assert!(Validator::is_valid_address("addr-456"));
        assert!(!Validator::is_valid_address(""));
        assert!(!Validator::is_valid_address("ab"));
    }

    #[test]
    fn test_converter_hex_bytes() {
        let bytes = Converter::hex_to_bytes("48656c6c6f").unwrap();
        assert_eq!(bytes, b"Hello");

        let hex = Converter::bytes_to_hex(b"Hello");
        assert_eq!(hex, "0x48656c6c6f");
    }

    #[test]
    fn test_converter_string_u64() {
        assert_eq!(Converter::str_to_u64("123").unwrap(), 123);
        assert!(Converter::str_to_u64("abc").is_err());
    }

    #[test]
    fn test_config() {
        let mut config = Config::new();
        
        config.set("port", "8080");
        config.set("debug", "true");
        config.set("timeout", "30");

        assert_eq!(config.get("port"), Some(&"8080".to_string()));
        assert_eq!(config.get_u64("timeout").unwrap(), 30);
        assert_eq!(config.get_bool("debug").unwrap(), true);
        assert_eq!(config.get_or_default("missing", "default"), "default");
    }

    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(2, Duration::from_secs(60));
        
        assert!(limiter.is_allowed());
        assert!(limiter.is_allowed());
        assert!(!limiter.is_allowed()); // Third request should be denied
        
        assert_eq!(limiter.current_count(), 2);
    }

    #[test]
    fn test_id_generator() {
        let id1 = IdGenerator::generate_id();
        let id2 = IdGenerator::generate_id();
        
        assert_ne!(id1, id2);
        assert_eq!(id1.len(), 16); // 8 bytes as hex
        
        let uuid = IdGenerator::generate_uuid();
        assert_eq!(uuid.len(), 36); // UUID format
        
        let tx_id = IdGenerator::generate_tx_id("alice", "bob", 100, 1);
        assert_eq!(tx_id.len(), 64); // SHA256 hash
    }

    #[test]
    fn test_math_utils() {
        assert_eq!(MathUtils::percentage(25, 100), 25.0);
        assert_eq!(MathUtils::average(&[1, 2, 3, 4, 5]), 3.0);
        assert_eq!(MathUtils::median(&[1, 2, 3, 4, 5]), 3.0);
        assert_eq!(MathUtils::median(&[1, 2, 3, 4]), 2.5);
        
        assert_eq!(MathUtils::safe_add(u64::MAX, 1), None);
        assert_eq!(MathUtils::safe_sub(0, 1), None);
        assert_eq!(MathUtils::safe_div(10, 0), None);
        assert_eq!(MathUtils::safe_mul(u64::MAX, 2), None);
    }

    #[test]
    fn test_format_percentage() {
        assert_eq!(Formatter::format_percentage(25.0, 100.0, 2), "25.00%");
        assert_eq!(Formatter::format_percentage(33.333, 100.0, 1), "33.3%");
        assert_eq!(Formatter::format_percentage(50.0, 0.0, 2), "0.00%");
    }
}

