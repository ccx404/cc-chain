//! CC Chain RPC Protocol
//!
//! This module defines the RPC protocol specifications, message formats,
//! and communication patterns for CC Chain RPC interactions.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Protocol version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: String, actual: String },
    
    #[error("Invalid message format: {0}")]
    InvalidMessageFormat(String),
    
    #[error("Unsupported method: {0}")]
    UnsupportedMethod(String),
    
    #[error("Protocol negotiation failed: {0}")]
    NegotiationFailed(String),
    
    #[error("Authentication required")]
    AuthenticationRequired,
    
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
}

pub type Result<T> = std::result::Result<T, ProtocolError>;

/// RPC protocol version
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl ProtocolVersion {
    /// Create a new protocol version
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }

    /// Current CC Chain RPC protocol version
    pub const CURRENT: ProtocolVersion = ProtocolVersion::new(1, 0, 0);

    /// Check if this version is compatible with another
    pub fn is_compatible_with(&self, other: &ProtocolVersion) -> bool {
        self.major == other.major && self.minor <= other.minor
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }

    /// Parse from string
    pub fn from_string(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return None;
        }

        let major = parts[0].parse().ok()?;
        let minor = parts[1].parse().ok()?;
        let patch = parts[2].parse().ok()?;

        Some(ProtocolVersion::new(major, minor, patch))
    }
}

/// Transport layer types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportType {
    Http,
    WebSocket,
    Tcp,
    Ipc,
}

impl TransportType {
    /// Get default port for this transport
    pub fn default_port(&self) -> u16 {
        match self {
            TransportType::Http => 8545,
            TransportType::WebSocket => 8546,
            TransportType::Tcp => 8547,
            TransportType::Ipc => 0, // Not applicable
        }
    }

    /// Check if transport supports batching
    pub fn supports_batching(&self) -> bool {
        match self {
            TransportType::Http => true,
            TransportType::WebSocket => true,
            TransportType::Tcp => true,
            TransportType::Ipc => true,
        }
    }

    /// Check if transport supports streaming
    pub fn supports_streaming(&self) -> bool {
        match self {
            TransportType::Http => false,
            TransportType::WebSocket => true,
            TransportType::Tcp => true,
            TransportType::Ipc => true,
        }
    }
}

/// RPC message envelope for transport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcEnvelope {
    pub protocol_version: ProtocolVersion,
    pub transport: String,
    pub content_type: String,
    pub content_encoding: Option<String>,
    pub authentication: Option<AuthenticationInfo>,
    pub metadata: HashMap<String, Value>,
    pub payload: Value,
}

impl RpcEnvelope {
    /// Create a new RPC envelope
    pub fn new(payload: Value) -> Self {
        Self {
            protocol_version: ProtocolVersion::CURRENT,
            transport: "http".to_string(),
            content_type: "application/json".to_string(),
            content_encoding: None,
            authentication: None,
            metadata: HashMap::new(),
            payload,
        }
    }

    /// Add authentication information
    pub fn with_auth(mut self, auth: AuthenticationInfo) -> Self {
        self.authentication = Some(auth);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Set content encoding
    pub fn with_encoding(mut self, encoding: String) -> Self {
        self.content_encoding = Some(encoding);
        self
    }

    /// Validate the envelope
    pub fn validate(&self) -> Result<()> {
        // Check protocol version compatibility
        if !self.protocol_version.is_compatible_with(&ProtocolVersion::CURRENT) {
            return Err(ProtocolError::VersionMismatch {
                expected: ProtocolVersion::CURRENT.to_string(),
                actual: self.protocol_version.to_string(),
            });
        }

        // Validate content type
        if !["application/json", "application/cbor", "application/msgpack"].contains(&self.content_type.as_str()) {
            return Err(ProtocolError::InvalidMessageFormat(
                format!("Unsupported content type: {}", self.content_type)
            ));
        }

        Ok(())
    }
}

/// Authentication information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationInfo {
    pub auth_type: AuthenticationType,
    pub credentials: HashMap<String, String>,
    pub timestamp: Option<u64>,
    pub nonce: Option<String>,
}

/// Authentication types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthenticationType {
    None,
    ApiKey,
    Bearer,
    Signature,
    Mutual,
}

/// RPC method metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodMetadata {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ParameterSpec>,
    pub returns: Option<ReturnSpec>,
    pub deprecated: bool,
    pub since_version: ProtocolVersion,
    pub rate_limit: Option<RateLimit>,
    pub auth_required: bool,
}

/// Parameter specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSpec {
    pub name: String,
    pub parameter_type: String,
    pub required: bool,
    pub description: String,
    pub default_value: Option<Value>,
    pub validation: Option<ValidationRule>,
}

/// Return value specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnSpec {
    pub return_type: String,
    pub description: String,
    pub example: Option<Value>,
}

/// Validation rules for parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub allowed_values: Option<Vec<Value>>,
}

/// Rate limiting specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub window_seconds: u32,
}

/// Protocol capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolCapabilities {
    pub version: ProtocolVersion,
    pub supported_transports: Vec<TransportType>,
    pub supported_encodings: Vec<String>,
    pub supports_batching: bool,
    pub supports_notifications: bool,
    pub supports_streaming: bool,
    pub max_request_size: usize,
    pub max_response_size: usize,
    pub timeout_seconds: u32,
}

impl Default for ProtocolCapabilities {
    fn default() -> Self {
        Self {
            version: ProtocolVersion::CURRENT,
            supported_transports: vec![
                TransportType::Http,
                TransportType::WebSocket,
            ],
            supported_encodings: vec![
                "identity".to_string(),
                "gzip".to_string(),
                "deflate".to_string(),
            ],
            supports_batching: true,
            supports_notifications: true,
            supports_streaming: false,
            max_request_size: 1024 * 1024, // 1MB
            max_response_size: 1024 * 1024, // 1MB
            timeout_seconds: 30,
        }
    }
}

/// RPC protocol handler
pub struct RpcProtocol {
    capabilities: ProtocolCapabilities,
    methods: HashMap<String, MethodMetadata>,
}

impl RpcProtocol {
    /// Create a new protocol handler
    pub fn new() -> Self {
        let mut protocol = Self {
            capabilities: ProtocolCapabilities::default(),
            methods: HashMap::new(),
        };
        
        protocol.register_standard_methods();
        protocol
    }

    /// Create protocol with custom capabilities
    pub fn with_capabilities(capabilities: ProtocolCapabilities) -> Self {
        let mut protocol = Self {
            capabilities,
            methods: HashMap::new(),
        };
        
        protocol.register_standard_methods();
        protocol
    }

    /// Register standard CC Chain RPC methods
    fn register_standard_methods(&mut self) {
        // Blockchain query methods
        self.register_method(MethodMetadata {
            name: "cc_getBlockByHeight".to_string(),
            description: "Get block information by height".to_string(),
            parameters: vec![
                ParameterSpec {
                    name: "height".to_string(),
                    parameter_type: "integer".to_string(),
                    required: true,
                    description: "Block height".to_string(),
                    default_value: None,
                    validation: Some(ValidationRule {
                        min_value: Some(0.0),
                        max_value: None,
                        ..Default::default()
                    }),
                },
            ],
            returns: Some(ReturnSpec {
                return_type: "object".to_string(),
                description: "Block information".to_string(),
                example: None,
            }),
            deprecated: false,
            since_version: ProtocolVersion::new(1, 0, 0),
            rate_limit: Some(RateLimit {
                requests_per_minute: 60,
                burst_size: 10,
                window_seconds: 60,
            }),
            auth_required: false,
        });

        self.register_method(MethodMetadata {
            name: "cc_sendTransaction".to_string(),
            description: "Submit a transaction to the network".to_string(),
            parameters: vec![
                ParameterSpec {
                    name: "transaction".to_string(),
                    parameter_type: "object".to_string(),
                    required: true,
                    description: "Transaction object".to_string(),
                    default_value: None,
                    validation: None,
                },
            ],
            returns: Some(ReturnSpec {
                return_type: "string".to_string(),
                description: "Transaction hash".to_string(),
                example: None,
            }),
            deprecated: false,
            since_version: ProtocolVersion::new(1, 0, 0),
            rate_limit: Some(RateLimit {
                requests_per_minute: 30,
                burst_size: 5,
                window_seconds: 60,
            }),
            auth_required: false,
        });

        // Add more standard methods...
        self.register_ping_method();
        self.register_version_method();
    }

    fn register_ping_method(&mut self) {
        self.register_method(MethodMetadata {
            name: "cc_ping".to_string(),
            description: "Ping the server".to_string(),
            parameters: vec![],
            returns: Some(ReturnSpec {
                return_type: "string".to_string(),
                description: "Pong response".to_string(),
                example: Some(serde_json::json!("pong")),
            }),
            deprecated: false,
            since_version: ProtocolVersion::new(1, 0, 0),
            rate_limit: Some(RateLimit {
                requests_per_minute: 120,
                burst_size: 20,
                window_seconds: 60,
            }),
            auth_required: false,
        });
    }

    fn register_version_method(&mut self) {
        self.register_method(MethodMetadata {
            name: "cc_getVersion".to_string(),
            description: "Get server version information".to_string(),
            parameters: vec![],
            returns: Some(ReturnSpec {
                return_type: "object".to_string(),
                description: "Version information".to_string(),
                example: Some(serde_json::json!({
                    "version": "1.0.0",
                    "build": "abc123",
                    "protocol": "1.0.0"
                })),
            }),
            deprecated: false,
            since_version: ProtocolVersion::new(1, 0, 0),
            rate_limit: Some(RateLimit {
                requests_per_minute: 60,
                burst_size: 10,
                window_seconds: 60,
            }),
            auth_required: false,
        });
    }

    /// Register a new method
    pub fn register_method(&mut self, method: MethodMetadata) {
        self.methods.insert(method.name.clone(), method);
    }

    /// Get method metadata
    pub fn get_method(&self, name: &str) -> Option<&MethodMetadata> {
        self.methods.get(name)
    }

    /// Get all supported methods
    pub fn get_supported_methods(&self) -> Vec<String> {
        self.methods.keys().cloned().collect()
    }

    /// Get protocol capabilities
    pub fn get_capabilities(&self) -> &ProtocolCapabilities {
        &self.capabilities
    }

    /// Validate a method call
    pub fn validate_method_call(&self, method: &str, params: Option<&Value>) -> Result<()> {
        let method_meta = self.methods.get(method)
            .ok_or_else(|| ProtocolError::UnsupportedMethod(method.to_string()))?;

        if method_meta.deprecated {
            // Log deprecation warning but don't fail
        }

        // Validate parameters if provided
        if let Some(params_obj) = params {
            self.validate_parameters(&method_meta.parameters, params_obj)?;
        } else {
            // Check if any required parameters
            let required_params: Vec<_> = method_meta.parameters.iter()
                .filter(|p| p.required)
                .collect();
            
            if !required_params.is_empty() {
                return Err(ProtocolError::InvalidMessageFormat(
                    format!("Method {} requires parameters: {}", 
                        method, 
                        required_params.iter()
                            .map(|p| &p.name)
                            .cloned()
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                ));
            }
        }

        Ok(())
    }

    fn validate_parameters(&self, param_specs: &[ParameterSpec], params: &Value) -> Result<()> {
        // Basic parameter validation
        if let Some(params_obj) = params.as_object() {
            for spec in param_specs {
                if spec.required && !params_obj.contains_key(&spec.name) {
                    return Err(ProtocolError::InvalidMessageFormat(
                        format!("Required parameter '{}' is missing", spec.name)
                    ));
                }

                if let Some(param_value) = params_obj.get(&spec.name) {
                    self.validate_parameter_value(spec, param_value)?;
                }
            }
        }

        Ok(())
    }

    fn validate_parameter_value(&self, spec: &ParameterSpec, value: &Value) -> Result<()> {
        if let Some(validation) = &spec.validation {
            // Validate string length
            if let Some(s) = value.as_str() {
                if let Some(min_len) = validation.min_length {
                    if s.len() < min_len {
                        return Err(ProtocolError::InvalidMessageFormat(
                            format!("Parameter '{}' is too short", spec.name)
                        ));
                    }
                }
                if let Some(max_len) = validation.max_length {
                    if s.len() > max_len {
                        return Err(ProtocolError::InvalidMessageFormat(
                            format!("Parameter '{}' is too long", spec.name)
                        ));
                    }
                }
            }

            // Validate numeric range
            if let Some(n) = value.as_f64() {
                if let Some(min_val) = validation.min_value {
                    if n < min_val {
                        return Err(ProtocolError::InvalidMessageFormat(
                            format!("Parameter '{}' is too small", spec.name)
                        ));
                    }
                }
                if let Some(max_val) = validation.max_value {
                    if n > max_val {
                        return Err(ProtocolError::InvalidMessageFormat(
                            format!("Parameter '{}' is too large", spec.name)
                        ));
                    }
                }
            }

            // Validate allowed values
            if let Some(allowed) = &validation.allowed_values {
                if !allowed.contains(value) {
                    return Err(ProtocolError::InvalidMessageFormat(
                        format!("Parameter '{}' has invalid value", spec.name)
                    ));
                }
            }
        }

        Ok(())
    }

    /// Generate OpenRPC specification
    pub fn generate_openrpc_spec(&self) -> Value {
        serde_json::json!({
            "openrpc": "1.2.6",
            "info": {
                "title": "CC Chain RPC API",
                "version": self.capabilities.version.to_string(),
                "description": "CC Chain blockchain RPC API"
            },
            "methods": self.methods.values().map(|m| {
                serde_json::json!({
                    "name": m.name,
                    "description": m.description,
                    "params": m.parameters.iter().map(|p| {
                        serde_json::json!({
                            "name": p.name,
                            "required": p.required,
                            "schema": {
                                "type": p.parameter_type
                            },
                            "description": p.description
                        })
                    }).collect::<Vec<_>>(),
                    "result": m.returns.as_ref().map(|r| {
                        serde_json::json!({
                            "name": "result",
                            "schema": {
                                "type": r.return_type
                            },
                            "description": r.description
                        })
                    })
                })
            }).collect::<Vec<_>>()
        })
    }
}

impl Default for RpcProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ValidationRule {
    fn default() -> Self {
        Self {
            min_length: None,
            max_length: None,
            pattern: None,
            min_value: None,
            max_value: None,
            allowed_values: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_version() {
        let version = ProtocolVersion::new(1, 2, 3);
        assert_eq!(version.to_string(), "1.2.3");
        
        let parsed = ProtocolVersion::from_string("1.2.3").unwrap();
        assert_eq!(version, parsed);
        
        assert!(version.is_compatible_with(&ProtocolVersion::new(1, 3, 0)));
        assert!(!version.is_compatible_with(&ProtocolVersion::new(2, 0, 0)));
    }

    #[test]
    fn test_transport_type() {
        assert_eq!(TransportType::Http.default_port(), 8545);
        assert!(TransportType::WebSocket.supports_streaming());
        assert!(!TransportType::Http.supports_streaming());
    }

    #[test]
    fn test_rpc_envelope() {
        let payload = serde_json::json!({"method": "test"});
        let envelope = RpcEnvelope::new(payload.clone());
        
        assert_eq!(envelope.protocol_version, ProtocolVersion::CURRENT);
        assert_eq!(envelope.payload, payload);
        assert!(envelope.validate().is_ok());
    }

    #[test]
    fn test_rpc_envelope_with_auth() {
        let auth = AuthenticationInfo {
            auth_type: AuthenticationType::ApiKey,
            credentials: {
                let mut creds = HashMap::new();
                creds.insert("api_key".to_string(), "secret123".to_string());
                creds
            },
            timestamp: Some(1640000000),
            nonce: None,
        };
        
        let envelope = RpcEnvelope::new(serde_json::json!({}))
            .with_auth(auth);
        
        assert!(envelope.authentication.is_some());
    }

    #[test]
    fn test_protocol_creation() {
        let protocol = RpcProtocol::new();
        let methods = protocol.get_supported_methods();
        
        assert!(!methods.is_empty());
        assert!(methods.contains(&"cc_ping".to_string()));
        assert!(methods.contains(&"cc_getVersion".to_string()));
    }

    #[test]
    fn test_method_validation() {
        let protocol = RpcProtocol::new();
        
        // Valid method call
        let params = serde_json::json!({"height": 12345});
        assert!(protocol.validate_method_call("cc_getBlockByHeight", Some(&params)).is_ok());
        
        // Invalid method
        assert!(protocol.validate_method_call("invalid_method", None).is_err());
        
        // Missing required parameter
        assert!(protocol.validate_method_call("cc_getBlockByHeight", None).is_err());
    }

    #[test]
    fn test_parameter_validation() {
        let protocol = RpcProtocol::new();
        
        // Valid parameters
        let valid_params = serde_json::json!({"height": 100});
        assert!(protocol.validate_method_call("cc_getBlockByHeight", Some(&valid_params)).is_ok());
        
        // Invalid parameter type (negative number where positive expected)
        let invalid_params = serde_json::json!({"height": -1});
        assert!(protocol.validate_method_call("cc_getBlockByHeight", Some(&invalid_params)).is_err());
    }

    #[test]
    fn test_capabilities() {
        let capabilities = ProtocolCapabilities::default();
        assert!(capabilities.supports_batching);
        assert!(capabilities.supports_notifications);
        assert_eq!(capabilities.timeout_seconds, 30);
    }

    #[test]
    fn test_method_metadata() {
        let protocol = RpcProtocol::new();
        let ping_method = protocol.get_method("cc_ping").unwrap();
        
        assert_eq!(ping_method.name, "cc_ping");
        assert!(!ping_method.deprecated);
        assert!(!ping_method.auth_required);
        assert!(ping_method.rate_limit.is_some());
    }

    #[test]
    fn test_openrpc_spec_generation() {
        let protocol = RpcProtocol::new();
        let spec = protocol.generate_openrpc_spec();
        
        assert!(spec.get("openrpc").is_some());
        assert!(spec.get("info").is_some());
        assert!(spec.get("methods").is_some());
        
        let methods = spec["methods"].as_array().unwrap();
        assert!(!methods.is_empty());
    }

    #[test]
    fn test_envelope_validation() {
        let mut envelope = RpcEnvelope::new(serde_json::json!({}));
        
        // Valid envelope
        assert!(envelope.validate().is_ok());
        
        // Invalid content type
        envelope.content_type = "text/plain".to_string();
        assert!(envelope.validate().is_err());
        
        // Invalid protocol version
        envelope.content_type = "application/json".to_string();
        envelope.protocol_version = ProtocolVersion::new(99, 0, 0);
        assert!(envelope.validate().is_err());
    }

    #[test]
    fn test_validation_rules() {
        let rule = ValidationRule {
            min_length: Some(5),
            max_length: Some(20),
            min_value: Some(0.0),
            max_value: Some(100.0),
            ..Default::default()
        };
        
        assert!(rule.min_length.is_some());
        assert!(rule.max_value.is_some());
    }

    #[test]
    fn test_authentication_types() {
        let auth = AuthenticationInfo {
            auth_type: AuthenticationType::Bearer,
            credentials: HashMap::new(),
            timestamp: None,
            nonce: None,
        };
        
        assert!(matches!(auth.auth_type, AuthenticationType::Bearer));
    }

    #[test]
    fn test_rate_limit() {
        let rate_limit = RateLimit {
            requests_per_minute: 60,
            burst_size: 10,
            window_seconds: 60,
        };
        
        assert_eq!(rate_limit.requests_per_minute, 60);
        assert_eq!(rate_limit.burst_size, 10);
    }
}
