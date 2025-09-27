//! Core error handling functionality
//!
//! This module provides comprehensive error handling capabilities including
//! error types, error context, error recovery, logging integration, and debugging utilities.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::error::Error as StdError;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Core CC Chain error types
#[derive(Error, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CCChainError {
    // Blockchain errors
    #[error("Block validation failed: {message}")]
    BlockValidation { message: String },
    
    #[error("Transaction validation failed: {message}")]
    TransactionValidation { message: String },
    
    #[error("Chain validation failed: {message}")]
    ChainValidation { message: String },
    
    #[error("Invalid block hash: {hash}")]
    InvalidBlockHash { hash: String },
    
    #[error("Block not found: {hash}")]
    BlockNotFound { hash: String },

    // Cryptography errors
    #[error("Cryptographic operation failed: {operation}")]
    CryptographyFailure { operation: String },
    
    #[error("Invalid signature: {message}")]
    InvalidSignature { message: String },
    
    #[error("Key generation failed: {reason}")]
    KeyGenerationFailure { reason: String },
    
    #[error("Hash computation failed")]
    HashFailure,

    // Transaction processing errors
    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: u64, available: u64 },
    
    #[error("Invalid transaction fee: {fee}")]
    InvalidFee { fee: u64 },
    
    #[error("Transaction expired at block {block_number}")]
    TransactionExpired { block_number: u64 },
    
    #[error("Nonce mismatch: expected {expected}, got {actual}")]
    NonceMismatch { expected: u64, actual: u64 },
    
    #[error("Mempool full: capacity {capacity}")]
    MempoolFull { capacity: usize },

    // State management errors
    #[error("State key not found: {key}")]
    StateKeyNotFound { key: String },
    
    #[error("Invalid state version: {version}")]
    InvalidStateVersion { version: u64 },
    
    #[error("State corruption detected: {details}")]
    StateCorruption { details: String },
    
    #[error("Snapshot not found: {snapshot_id}")]
    SnapshotNotFound { snapshot_id: String },
    
    #[error("Rollback failed: {reason}")]
    RollbackFailure { reason: String },

    // Network errors
    #[error("Network connection failed to {peer}")]
    NetworkConnection { peer: String },
    
    #[error("Message serialization failed: {message}")]
    MessageSerialization { message: String },
    
    #[error("Protocol version mismatch: expected {expected}, got {actual}")]
    ProtocolMismatch { expected: String, actual: String },
    
    #[error("Peer disconnected: {peer_id}")]
    PeerDisconnected { peer_id: String },

    // Consensus errors
    #[error("Consensus failed: {reason}")]
    ConsensusFailure { reason: String },
    
    #[error("Invalid proposal: {details}")]
    InvalidProposal { details: String },
    
    #[error("Vote validation failed: {message}")]
    VoteValidationFailure { message: String },
    
    #[error("View change failed: from {from_view} to {to_view}")]
    ViewChangeFailure { from_view: u64, to_view: u64 },

    // Storage errors
    #[error("Database operation failed: {operation}")]
    DatabaseFailure { operation: String },
    
    #[error("Disk full: {path}")]
    DiskFull { path: String },
    
    #[error("Serialization failed: {data_type}")]
    SerializationFailure { data_type: String },
    
    #[error("Data corruption detected in {location}")]
    DataCorruption { location: String },

    // System errors
    #[error("Configuration error: {parameter}")]
    Configuration { parameter: String },
    
    #[error("Resource exhausted: {resource}")]
    ResourceExhaustion { resource: String },
    
    #[error("Timeout occurred: {operation} after {duration_ms}ms")]
    Timeout { operation: String, duration_ms: u64 },
    
    #[error("Internal error: {message}")]
    Internal { message: String },

    // Generic errors
    #[error("Operation not supported: {operation}")]
    UnsupportedOperation { operation: String },
    
    #[error("Invalid input: {field} = {value}")]
    InvalidInput { field: String, value: String },
    
    #[error("Parsing failed: {input}")]
    ParsingFailure { input: String },
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Critical system error requiring immediate attention
    Critical,
    /// Error that prevents normal operation
    Error,
    /// Warning that doesn't prevent operation but should be addressed
    Warning,
    /// Informational error for debugging
    Info,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSeverity::Critical => write!(f, "CRITICAL"),
            ErrorSeverity::Error => write!(f, "ERROR"),
            ErrorSeverity::Warning => write!(f, "WARNING"),
            ErrorSeverity::Info => write!(f, "INFO"),
        }
    }
}

/// Error context with additional debugging information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// Error timestamp
    pub timestamp: u64,
    /// Component where error occurred
    pub component: String,
    /// Function or method where error occurred
    pub function: String,
    /// Error severity
    pub severity: ErrorSeverity,
    /// Additional context data
    pub context_data: std::collections::HashMap<String, String>,
    /// Call stack trace (if available)
    pub stack_trace: Vec<String>,
}

impl ErrorContext {
    /// Create new error context
    pub fn new(component: &str, function: &str, severity: ErrorSeverity) -> Self {
        ErrorContext {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            component: component.to_string(),
            function: function.to_string(),
            severity,
            context_data: std::collections::HashMap::new(),
            stack_trace: Vec::new(),
        }
    }

    /// Add context data
    pub fn with_context(mut self, key: &str, value: &str) -> Self {
        self.context_data.insert(key.to_string(), value.to_string());
        self
    }

    /// Add multiple context entries
    pub fn with_contexts(mut self, contexts: &[(&str, &str)]) -> Self {
        for (key, value) in contexts {
            self.context_data.insert(key.to_string(), value.to_string());
        }
        self
    }

    /// Add stack trace
    pub fn with_stack_trace(mut self, trace: Vec<String>) -> Self {
        self.stack_trace = trace;
        self
    }

    /// Get formatted error message
    pub fn format_error(&self, error: &CCChainError) -> String {
        format!(
            "[{}] {} in {}::{} - {}\nContext: {:?}",
            self.severity,
            error,
            self.component,
            self.function,
            self.timestamp,
            self.context_data
        )
    }
}

/// Enhanced error with context
#[derive(Debug, Clone)]
pub struct CCChainErrorWithContext {
    pub error: CCChainError,
    pub context: ErrorContext,
    pub source_error: Option<String>,
}

impl CCChainErrorWithContext {
    /// Create new error with context
    pub fn new(error: CCChainError, context: ErrorContext) -> Self {
        CCChainErrorWithContext {
            error,
            context,
            source_error: None,
        }
    }

    /// Create from source error
    pub fn from_source<E: StdError>(
        source: E,
        context: ErrorContext,
        conversion: impl FnOnce(String) -> CCChainError,
    ) -> Self {
        let source_msg = source.to_string();
        let error = conversion(source_msg.clone());
        
        CCChainErrorWithContext {
            error,
            context,
            source_error: Some(source_msg),
        }
    }

    /// Get error severity
    pub fn severity(&self) -> ErrorSeverity {
        self.context.severity
    }

    /// Check if error is critical
    pub fn is_critical(&self) -> bool {
        self.context.severity == ErrorSeverity::Critical
    }

    /// Check if error is recoverable based on type
    pub fn is_recoverable(&self) -> bool {
        match &self.error {
            // Critical system errors are not recoverable
            CCChainError::StateCorruption { .. } |
            CCChainError::DataCorruption { .. } |
            CCChainError::DiskFull { .. } => false,
            
            // Network and temporary errors are usually recoverable
            CCChainError::NetworkConnection { .. } |
            CCChainError::PeerDisconnected { .. } |
            CCChainError::Timeout { .. } |
            CCChainError::MempoolFull { .. } => true,
            
            // Most other errors are recoverable with proper handling
            _ => true,
        }
    }

    /// Get suggested recovery action
    pub fn recovery_action(&self) -> String {
        match &self.error {
            CCChainError::InsufficientBalance { required, available } => {
                format!("Wait for balance to increase or reduce transaction amount by {}", 
                       required.saturating_sub(*available))
            }
            CCChainError::MempoolFull { .. } => {
                "Wait for mempool to clear or increase gas price for priority".to_string()
            }
            CCChainError::NetworkConnection { peer } => {
                format!("Retry connection to peer {} or find alternative peers", peer)
            }
            CCChainError::Timeout { operation, .. } => {
                format!("Retry {} operation with increased timeout", operation)
            }
            CCChainError::InvalidFee { .. } => {
                "Adjust transaction fee to meet minimum requirements".to_string()
            }
            CCChainError::NonceMismatch { expected, .. } => {
                format!("Use correct nonce value: {}", expected)
            }
            _ => "Review error details and retry operation".to_string(),
        }
    }
}

impl fmt::Display for CCChainErrorWithContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.context.format_error(&self.error))?;
        
        if let Some(ref source) = self.source_error {
            write!(f, "\nCaused by: {}", source)?;
        }
        
        if !self.context.stack_trace.is_empty() {
            write!(f, "\nStack trace:")?;
            for frame in &self.context.stack_trace {
                write!(f, "\n  {}", frame)?;
            }
        }
        
        Ok(())
    }
}

impl StdError for CCChainErrorWithContext {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None // We store source as string, not as error type
    }
}

/// Result type with enhanced error context
pub type Result<T> = std::result::Result<T, CCChainErrorWithContext>;

/// Error metrics and statistics
#[derive(Debug, Clone, Default)]
pub struct ErrorMetrics {
    pub total_errors: u64,
    pub critical_errors: u64,
    pub error_count_by_type: std::collections::HashMap<String, u64>,
    pub error_count_by_component: std::collections::HashMap<String, u64>,
    pub recoverable_errors: u64,
    pub unrecoverable_errors: u64,
}

impl ErrorMetrics {
    /// Create new error metrics
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an error
    pub fn record_error(&mut self, error: &CCChainErrorWithContext) {
        self.total_errors += 1;
        
        if error.is_critical() {
            self.critical_errors += 1;
        }
        
        if error.is_recoverable() {
            self.recoverable_errors += 1;
        } else {
            self.unrecoverable_errors += 1;
        }
        
        // Count by error type
        let error_type = format!("{:?}", std::mem::discriminant(&error.error));
        *self.error_count_by_type.entry(error_type).or_insert(0) += 1;
        
        // Count by component
        *self.error_count_by_component.entry(error.context.component.clone()).or_insert(0) += 1;
    }

    /// Get error rate for component
    pub fn error_rate_for_component(&self, component: &str) -> f64 {
        let component_errors = self.error_count_by_component.get(component).unwrap_or(&0);
        if self.total_errors > 0 {
            *component_errors as f64 / self.total_errors as f64
        } else {
            0.0
        }
    }

    /// Get most common error type
    pub fn most_common_error_type(&self) -> Option<String> {
        self.error_count_by_type
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(error_type, _)| error_type.clone())
    }

    /// Get critical error rate
    pub fn critical_error_rate(&self) -> f64 {
        if self.total_errors > 0 {
            self.critical_errors as f64 / self.total_errors as f64
        } else {
            0.0
        }
    }

    /// Get recovery rate
    pub fn recovery_rate(&self) -> f64 {
        if self.total_errors > 0 {
            self.recoverable_errors as f64 / self.total_errors as f64
        } else {
            0.0
        }
    }
}

/// Error handler with recovery strategies
#[derive(Debug)]
pub struct ErrorHandler {
    metrics: ErrorMetrics,
    max_retry_attempts: u32,
    enable_recovery: bool,
}

impl ErrorHandler {
    /// Create new error handler
    pub fn new(max_retry_attempts: u32, enable_recovery: bool) -> Self {
        ErrorHandler {
            metrics: ErrorMetrics::new(),
            max_retry_attempts,
            enable_recovery,
        }
    }

    /// Handle error and record metrics
    pub fn handle_error(&mut self, error: CCChainErrorWithContext) -> CCChainErrorWithContext {
        self.metrics.record_error(&error);
        
        // Log error based on severity
        match error.severity() {
            ErrorSeverity::Critical => {
                tracing::error!("CRITICAL ERROR: {}", error);
            }
            ErrorSeverity::Error => {
                tracing::error!("ERROR: {}", error);
            }
            ErrorSeverity::Warning => {
                tracing::warn!("WARNING: {}", error);
            }
            ErrorSeverity::Info => {
                tracing::info!("INFO: {}", error);
            }
        }
        
        error
    }

    /// Try to recover from error with retry logic
    pub fn try_recover<T, F>(&mut self, operation: F, context: ErrorContext) -> Result<T>
    where
        F: Fn() -> std::result::Result<T, CCChainError>,
    {
        let mut attempts = 0;
        
        loop {
            attempts += 1;
            
            match operation() {
                Ok(result) => return Ok(result),
                Err(error) => {
                    let error_with_context = CCChainErrorWithContext::new(error, context.clone());
                    
                    if !self.enable_recovery || !error_with_context.is_recoverable() || attempts >= self.max_retry_attempts {
                        return Err(self.handle_error(error_with_context));
                    }
                    
                    // Wait before retry (exponential backoff)
                    let delay_ms = (2_u64.pow(attempts - 1) * 100).min(5000);
                    std::thread::sleep(std::time::Duration::from_millis(delay_ms));
                }
            }
        }
    }

    /// Get error metrics
    pub fn get_metrics(&self) -> &ErrorMetrics {
        &self.metrics
    }

    /// Reset metrics
    pub fn reset_metrics(&mut self) {
        self.metrics = ErrorMetrics::new();
    }
}

/// Helper macros for error handling
#[macro_export]
macro_rules! chain_error {
    ($error:expr, $component:expr, $function:expr) => {
        CCChainErrorWithContext::new(
            $error,
            ErrorContext::new($component, $function, ErrorSeverity::Error)
        )
    };
    
    ($error:expr, $component:expr, $function:expr, $severity:expr) => {
        CCChainErrorWithContext::new(
            $error,
            ErrorContext::new($component, $function, $severity)
        )
    };
    
    ($error:expr, $component:expr, $function:expr, $severity:expr, $($key:expr => $value:expr),+) => {
        CCChainErrorWithContext::new(
            $error,
            ErrorContext::new($component, $function, $severity)
                $(.with_context($key, $value))+
        )
    };
}

#[macro_export]
macro_rules! bail_chain {
    ($error:expr, $component:expr, $function:expr) => {
        return Err(chain_error!($error, $component, $function));
    };
    
    ($error:expr, $component:expr, $function:expr, $severity:expr) => {
        return Err(chain_error!($error, $component, $function, $severity));
    };
    
    ($error:expr, $component:expr, $function:expr, $severity:expr, $($key:expr => $value:expr),+) => {
        return Err(chain_error!($error, $component, $function, $severity, $($key => $value),+));
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = CCChainError::InvalidFee { fee: 100 };
        assert_eq!(error.to_string(), "Invalid transaction fee: 100");
    }

    #[test]
    fn test_error_context() {
        let context = ErrorContext::new("blockchain", "validate_block", ErrorSeverity::Error)
            .with_context("block_hash", "abc123")
            .with_context("block_height", "1000");
        
        assert_eq!(context.component, "blockchain");
        assert_eq!(context.function, "validate_block");
        assert_eq!(context.severity, ErrorSeverity::Error);
        assert_eq!(context.context_data.len(), 2);
    }

    #[test]
    fn test_error_with_context() {
        let error = CCChainError::BlockNotFound { hash: "abc123".to_string() };
        let context = ErrorContext::new("blockchain", "get_block", ErrorSeverity::Error);
        
        let error_with_context = CCChainErrorWithContext::new(error, context);
        
        assert!(!error_with_context.is_critical());
        assert!(error_with_context.is_recoverable());
    }

    #[test]
    fn test_error_recovery() {
        let error = CCChainError::NetworkConnection { peer: "peer1".to_string() };
        let context = ErrorContext::new("network", "connect", ErrorSeverity::Warning);
        let error_with_context = CCChainErrorWithContext::new(error, context);
        
        assert!(error_with_context.is_recoverable());
        assert!(error_with_context.recovery_action().contains("Retry connection"));
    }

    #[test]
    fn test_error_metrics() {
        let mut metrics = ErrorMetrics::new();
        
        let error1 = CCChainErrorWithContext::new(
            CCChainError::InvalidFee { fee: 100 },
            ErrorContext::new("tx", "validate", ErrorSeverity::Error)
        );
        
        let error2 = CCChainErrorWithContext::new(
            CCChainError::StateCorruption { details: "test".to_string() },
            ErrorContext::new("state", "read", ErrorSeverity::Critical)
        );
        
        metrics.record_error(&error1);
        metrics.record_error(&error2);
        
        assert_eq!(metrics.total_errors, 2);
        assert_eq!(metrics.critical_errors, 1);
        assert_eq!(metrics.recoverable_errors, 1);
        assert_eq!(metrics.unrecoverable_errors, 1);
    }

    #[test]
    fn test_error_handler() {
        let mut handler = ErrorHandler::new(3, true);
        
        // Test successful operation
        let result = handler.try_recover(
            || Ok::<i32, CCChainError>(42),
            ErrorContext::new("test", "operation", ErrorSeverity::Info)
        );
        
        assert_eq!(result.unwrap(), 42);
        
        // Test failed operation
        let result: Result<i32> = handler.try_recover(
            || Err(CCChainError::StateCorruption { details: "test".to_string() }),
            ErrorContext::new("test", "operation", ErrorSeverity::Critical)
        );
        
        assert!(result.is_err());
        assert_eq!(handler.get_metrics().total_errors, 1);
    }

    #[test]
    fn test_severity_levels() {
        use ErrorSeverity::*;
        
        assert_eq!(Critical.to_string(), "CRITICAL");
        assert_eq!(Error.to_string(), "ERROR");
        assert_eq!(Warning.to_string(), "WARNING");
        assert_eq!(Info.to_string(), "INFO");
    }

    #[test]
    fn test_error_categorization() {
        // Test recoverable errors
        let recoverable = CCChainErrorWithContext::new(
            CCChainError::MempoolFull { capacity: 1000 },
            ErrorContext::new("mempool", "add_tx", ErrorSeverity::Warning)
        );
        assert!(recoverable.is_recoverable());
        
        // Test non-recoverable errors
        let non_recoverable = CCChainErrorWithContext::new(
            CCChainError::DataCorruption { location: "db".to_string() },
            ErrorContext::new("storage", "read", ErrorSeverity::Critical)
        );
        assert!(!non_recoverable.is_recoverable());
    }

    #[test]
    fn test_error_macros() {
        let error = chain_error!(
            CCChainError::InvalidInput { field: "amount".to_string(), value: "0".to_string() },
            "validator",
            "check_transaction"
        );
        
        assert_eq!(error.context.component, "validator");
        assert_eq!(error.context.function, "check_transaction");
    }
}

