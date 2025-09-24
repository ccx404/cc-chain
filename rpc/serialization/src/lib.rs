//! CC Chain RPC Serialization
//!
//! This module provides serialization and deserialization utilities for RPC
//! communications, supporting multiple formats and efficient data encoding.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SerializationError {
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
    
    #[error("Encoding error: {0}")]
    EncodingError(String),
    
    #[error("Decoding error: {0}")]
    DecodingError(String),
    
    #[error("Schema validation error: {0}")]
    SchemaError(String),
}

pub type Result<T> = std::result::Result<T, SerializationError>;

/// Supported serialization formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerializationFormat {
    Json,
    JsonCompact,
    MessagePack,
    Cbor,
}

impl SerializationFormat {
    /// Get the MIME type for this format
    pub fn mime_type(&self) -> &'static str {
        match self {
            SerializationFormat::Json => "application/json",
            SerializationFormat::JsonCompact => "application/json",
            SerializationFormat::MessagePack => "application/msgpack",
            SerializationFormat::Cbor => "application/cbor",
        }
    }

    /// Get file extension for this format
    pub fn file_extension(&self) -> &'static str {
        match self {
            SerializationFormat::Json => "json",
            SerializationFormat::JsonCompact => "json",
            SerializationFormat::MessagePack => "msgpack",
            SerializationFormat::Cbor => "cbor",
        }
    }

    /// Parse format from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "json" => Some(SerializationFormat::Json),
            "json-compact" => Some(SerializationFormat::JsonCompact),
            "msgpack" | "messagepack" => Some(SerializationFormat::MessagePack),
            "cbor" => Some(SerializationFormat::Cbor),
            _ => None,
        }
    }
}

impl Default for SerializationFormat {
    fn default() -> Self {
        SerializationFormat::Json
    }
}

/// Serialization configuration
#[derive(Debug, Clone)]
pub struct SerializationConfig {
    pub format: SerializationFormat,
    pub pretty_print: bool,
    pub compress: bool,
    pub validate_schema: bool,
    pub max_depth: u32,
    pub max_size: usize,
}

impl Default for SerializationConfig {
    fn default() -> Self {
        Self {
            format: SerializationFormat::Json,
            pretty_print: true,
            compress: false,
            validate_schema: false,
            max_depth: 64,
            max_size: 1024 * 1024, // 1MB
        }
    }
}

/// RPC serializer for handling various data formats
pub struct RpcSerializer {
    config: SerializationConfig,
}

impl RpcSerializer {
    /// Create a new serializer with default configuration
    pub fn new() -> Self {
        Self::with_config(SerializationConfig::default())
    }

    /// Create a new serializer with custom configuration
    pub fn with_config(config: SerializationConfig) -> Self {
        Self { config }
    }

    /// Serialize a value to bytes
    pub fn serialize<T: Serialize>(&self, value: &T) -> Result<Vec<u8>> {
        match self.config.format {
            SerializationFormat::Json => {
                if self.config.pretty_print {
                    let json = serde_json::to_vec_pretty(value)?;
                    Ok(json)
                } else {
                    let json = serde_json::to_vec(value)?;
                    Ok(json)
                }
            }
            SerializationFormat::JsonCompact => {
                let json = serde_json::to_vec(value)?;
                Ok(json)
            }
            SerializationFormat::MessagePack => {
                // Mock implementation - in real code would use rmp-serde
                let json = serde_json::to_vec(value)?;
                Ok(json) // Placeholder
            }
            SerializationFormat::Cbor => {
                // Mock implementation - in real code would use ciborium
                let json = serde_json::to_vec(value)?;
                Ok(json) // Placeholder
            }
        }
    }

    /// Deserialize bytes to a value
    pub fn deserialize<T: for<'de> Deserialize<'de>>(&self, data: &[u8]) -> Result<T> {
        if data.len() > self.config.max_size {
            return Err(SerializationError::DecodingError(
                format!("Data size {} exceeds maximum {}", data.len(), self.config.max_size)
            ));
        }

        match self.config.format {
            SerializationFormat::Json | SerializationFormat::JsonCompact => {
                let value = serde_json::from_slice(data)?;
                Ok(value)
            }
            SerializationFormat::MessagePack => {
                // Mock implementation
                let value = serde_json::from_slice(data)?;
                Ok(value)
            }
            SerializationFormat::Cbor => {
                // Mock implementation
                let value = serde_json::from_slice(data)?;
                Ok(value)
            }
        }
    }

    /// Serialize to string (JSON only)
    pub fn serialize_to_string<T: Serialize>(&self, value: &T) -> Result<String> {
        match self.config.format {
            SerializationFormat::Json => {
                if self.config.pretty_print {
                    Ok(serde_json::to_string_pretty(value)?)
                } else {
                    Ok(serde_json::to_string(value)?)
                }
            }
            SerializationFormat::JsonCompact => {
                Ok(serde_json::to_string(value)?)
            }
            _ => Err(SerializationError::UnsupportedFormat(
                "String serialization only supported for JSON formats".to_string()
            )),
        }
    }

    /// Deserialize from string (JSON only)
    pub fn deserialize_from_string<T: for<'de> Deserialize<'de>>(&self, data: &str) -> Result<T> {
        if data.len() > self.config.max_size {
            return Err(SerializationError::DecodingError(
                format!("Data size {} exceeds maximum {}", data.len(), self.config.max_size)
            ));
        }

        match self.config.format {
            SerializationFormat::Json | SerializationFormat::JsonCompact => {
                Ok(serde_json::from_str(data)?)
            }
            _ => Err(SerializationError::UnsupportedFormat(
                "String deserialization only supported for JSON formats".to_string()
            )),
        }
    }

    /// Convert between different serialization formats
    pub fn convert(&self, data: &[u8], from_format: SerializationFormat, to_format: SerializationFormat) -> Result<Vec<u8>> {
        if from_format == to_format {
            return Ok(data.to_vec());
        }

        // Convert through JSON Value as intermediate format
        let temp_serializer = RpcSerializer::with_config(SerializationConfig {
            format: from_format,
            ..Default::default()
        });
        
        let value: Value = temp_serializer.deserialize(data)?;
        
        let output_serializer = RpcSerializer::with_config(SerializationConfig {
            format: to_format,
            ..self.config
        });
        
        output_serializer.serialize(&value)
    }

    /// Get serialization metadata
    pub fn get_metadata<T: Serialize>(&self, value: &T) -> Result<SerializationMetadata> {
        let serialized = self.serialize(value)?;
        let compressed_size = if self.config.compress {
            // Mock compression - in real implementation would use a compression library
            serialized.len() / 2
        } else {
            serialized.len()
        };

        Ok(SerializationMetadata {
            format: self.config.format,
            original_size: serialized.len(),
            compressed_size,
            compression_ratio: serialized.len() as f64 / compressed_size as f64,
            estimated_parse_time_ms: (serialized.len() / 1000) as u64, // Rough estimate
        })
    }

    /// Validate JSON schema (basic implementation)
    pub fn validate_json_schema(&self, data: &Value, schema: &JsonSchema) -> Result<()> {
        if !self.config.validate_schema {
            return Ok(());
        }

        self.validate_value_against_schema(data, schema, 0)
    }

    fn validate_value_against_schema(&self, value: &Value, schema: &JsonSchema, depth: u32) -> Result<()> {
        if depth > self.config.max_depth {
            return Err(SerializationError::SchemaError(
                "Maximum validation depth exceeded".to_string()
            ));
        }

        match schema {
            JsonSchema::Object { required_fields, optional_fields: _ } => {
                if let Some(obj) = value.as_object() {
                    for field in required_fields {
                        if !obj.contains_key(field) {
                            return Err(SerializationError::SchemaError(
                                format!("Required field '{}' is missing", field)
                            ));
                        }
                    }
                    // In a full implementation, would validate field types
                    Ok(())
                } else {
                    Err(SerializationError::SchemaError(
                        "Expected object type".to_string()
                    ))
                }
            }
            JsonSchema::Array { item_schema } => {
                if let Some(arr) = value.as_array() {
                    for item in arr {
                        self.validate_value_against_schema(item, item_schema, depth + 1)?;
                    }
                    Ok(())
                } else {
                    Err(SerializationError::SchemaError(
                        "Expected array type".to_string()
                    ))
                }
            }
            JsonSchema::String => {
                if value.is_string() {
                    Ok(())
                } else {
                    Err(SerializationError::SchemaError(
                        "Expected string type".to_string()
                    ))
                }
            }
            JsonSchema::Number => {
                if value.is_number() {
                    Ok(())
                } else {
                    Err(SerializationError::SchemaError(
                        "Expected number type".to_string()
                    ))
                }
            }
            JsonSchema::Boolean => {
                if value.is_boolean() {
                    Ok(())
                } else {
                    Err(SerializationError::SchemaError(
                        "Expected boolean type".to_string()
                    ))
                }
            }
            JsonSchema::Any => Ok(()),
        }
    }
}

impl Default for RpcSerializer {
    fn default() -> Self {
        Self::new()
    }
}

/// Serialization metadata
#[derive(Debug, Clone)]
pub struct SerializationMetadata {
    pub format: SerializationFormat,
    pub original_size: usize,
    pub compressed_size: usize,
    pub compression_ratio: f64,
    pub estimated_parse_time_ms: u64,
}

/// Basic JSON schema definition
#[derive(Debug, Clone)]
pub enum JsonSchema {
    Object {
        required_fields: Vec<String>,
        optional_fields: Vec<String>,
    },
    Array {
        item_schema: Box<JsonSchema>,
    },
    String,
    Number,
    Boolean,
    Any,
}

/// Utility functions for common serialization tasks
pub struct SerializationUtils;

impl SerializationUtils {
    /// Create a schema for JSON-RPC 2.0 request
    pub fn jsonrpc_request_schema() -> JsonSchema {
        JsonSchema::Object {
            required_fields: vec![
                "jsonrpc".to_string(),
                "method".to_string(),
            ],
            optional_fields: vec![
                "params".to_string(),
                "id".to_string(),
            ],
        }
    }

    /// Create a schema for JSON-RPC 2.0 response
    pub fn jsonrpc_response_schema() -> JsonSchema {
        JsonSchema::Object {
            required_fields: vec![
                "jsonrpc".to_string(),
            ],
            optional_fields: vec![
                "result".to_string(),
                "error".to_string(),
                "id".to_string(),
            ],
        }
    }

    /// Pretty print a JSON value
    pub fn pretty_print_json(value: &Value) -> Result<String> {
        Ok(serde_json::to_string_pretty(value)?)
    }

    /// Minify a JSON string
    pub fn minify_json(json_str: &str) -> Result<String> {
        let value: Value = serde_json::from_str(json_str)?;
        Ok(serde_json::to_string(&value)?)
    }

    /// Calculate approximate serialization overhead
    pub fn calculate_overhead(original_data_size: usize, format: SerializationFormat) -> usize {
        match format {
            SerializationFormat::Json => original_data_size / 4, // Rough estimate
            SerializationFormat::JsonCompact => original_data_size / 6,
            SerializationFormat::MessagePack => original_data_size / 8,
            SerializationFormat::Cbor => original_data_size / 10,
        }
    }

    /// Estimate parsing performance
    pub fn estimate_parse_time(data_size: usize, format: SerializationFormat) -> u64 {
        let base_time = data_size / 1000; // 1ms per KB baseline
        match format {
            SerializationFormat::Json => base_time as u64,
            SerializationFormat::JsonCompact => (base_time * 8 / 10) as u64,
            SerializationFormat::MessagePack => (base_time * 6 / 10) as u64,
            SerializationFormat::Cbor => (base_time * 7 / 10) as u64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_serialization_format() {
        assert_eq!(SerializationFormat::Json.mime_type(), "application/json");
        assert_eq!(SerializationFormat::MessagePack.file_extension(), "msgpack");
        
        assert_eq!(SerializationFormat::from_str("json"), Some(SerializationFormat::Json));
        assert_eq!(SerializationFormat::from_str("msgpack"), Some(SerializationFormat::MessagePack));
        assert_eq!(SerializationFormat::from_str("invalid"), None);
    }

    #[test]
    fn test_serializer_creation() {
        let serializer = RpcSerializer::new();
        assert_eq!(serializer.config.format, SerializationFormat::Json);
        
        let config = SerializationConfig {
            format: SerializationFormat::JsonCompact,
            pretty_print: false,
            ..Default::default()
        };
        let serializer = RpcSerializer::with_config(config);
        assert_eq!(serializer.config.format, SerializationFormat::JsonCompact);
        assert!(!serializer.config.pretty_print);
    }

    #[test]
    fn test_json_serialization() {
        let serializer = RpcSerializer::new();
        let data = json!({"key": "value", "number": 42});
        
        let serialized = serializer.serialize(&data).unwrap();
        assert!(!serialized.is_empty());
        
        let deserialized: Value = serializer.deserialize(&serialized).unwrap();
        assert_eq!(data, deserialized);
    }

    #[test]
    fn test_string_serialization() {
        let serializer = RpcSerializer::new();
        let data = json!({"message": "hello world"});
        
        let json_string = serializer.serialize_to_string(&data).unwrap();
        assert!(json_string.contains("hello world"));
        
        let deserialized: Value = serializer.deserialize_from_string(&json_string).unwrap();
        assert_eq!(data, deserialized);
    }

    #[test]
    fn test_compact_json() {
        let config = SerializationConfig {
            format: SerializationFormat::JsonCompact,
            pretty_print: false,
            ..Default::default()
        };
        let serializer = RpcSerializer::with_config(config);
        
        let data = json!({"key": "value"});
        let json_string = serializer.serialize_to_string(&data).unwrap();
        
        // Compact JSON should not contain extra whitespace
        assert!(!json_string.contains("  "));
        assert!(!json_string.contains("\n"));
    }

    #[test]
    fn test_size_limits() {
        let config = SerializationConfig {
            max_size: 10, // Very small limit
            ..Default::default()
        };
        let serializer = RpcSerializer::with_config(config);
        
        let large_data = "x".repeat(100);
        let result = serializer.deserialize_from_string::<String>(&large_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_metadata() {
        let serializer = RpcSerializer::new();
        let data = json!({"test": "data", "number": 123});
        
        let metadata = serializer.get_metadata(&data).unwrap();
        assert_eq!(metadata.format, SerializationFormat::Json);
        assert!(metadata.original_size > 0);
        assert!(metadata.compressed_size > 0);
    }

    #[test]
    fn test_json_schema_validation() {
        let serializer = RpcSerializer::with_config(SerializationConfig {
            validate_schema: true,
            ..Default::default()
        });
        
        let schema = JsonSchema::Object {
            required_fields: vec!["name".to_string()],
            optional_fields: vec!["age".to_string()],
        };
        
        let valid_data = json!({"name": "John", "age": 30});
        assert!(serializer.validate_json_schema(&valid_data, &schema).is_ok());
        
        let invalid_data = json!({"age": 30}); // Missing required 'name'
        assert!(serializer.validate_json_schema(&invalid_data, &schema).is_err());
    }

    #[test]
    fn test_schema_utilities() {
        let request_schema = SerializationUtils::jsonrpc_request_schema();
        let response_schema = SerializationUtils::jsonrpc_response_schema();
        
        // Basic schema structure tests
        match request_schema {
            JsonSchema::Object { required_fields, .. } => {
                assert!(required_fields.contains(&"jsonrpc".to_string()));
                assert!(required_fields.contains(&"method".to_string()));
            }
            _ => panic!("Expected object schema"),
        }
        
        match response_schema {
            JsonSchema::Object { required_fields, .. } => {
                assert!(required_fields.contains(&"jsonrpc".to_string()));
            }
            _ => panic!("Expected object schema"),
        }
    }

    #[test]
    fn test_utility_functions() {
        let data = json!({"key": "value"});
        let pretty = SerializationUtils::pretty_print_json(&data).unwrap();
        assert!(pretty.contains("\n"));
        
        let minified = SerializationUtils::minify_json(&pretty).unwrap();
        assert!(!minified.contains("\n"));
        
        let overhead = SerializationUtils::calculate_overhead(1000, SerializationFormat::Json);
        assert!(overhead > 0);
        
        let parse_time = SerializationUtils::estimate_parse_time(1000, SerializationFormat::Json);
        assert!(parse_time >= 1);
    }

    #[test]
    fn test_format_conversion() {
        let serializer = RpcSerializer::new();
        let data = json!({"test": "conversion"});
        
        let json_bytes = serializer.serialize(&data).unwrap();
        
        // Convert JSON to JSON (should be identical)
        let converted = serializer.convert(
            &json_bytes,
            SerializationFormat::Json,
            SerializationFormat::Json
        ).unwrap();
        
        assert_eq!(json_bytes, converted);
    }

    #[test]
    fn test_array_schema_validation() {
        let serializer = RpcSerializer::with_config(SerializationConfig {
            validate_schema: true,
            ..Default::default()
        });
        
        let schema = JsonSchema::Array {
            item_schema: Box::new(JsonSchema::String),
        };
        
        let valid_array = json!(["hello", "world"]);
        assert!(serializer.validate_json_schema(&valid_array, &schema).is_ok());
        
        let invalid_array = json!(["hello", 123]); // Mixed types
        assert!(serializer.validate_json_schema(&invalid_array, &schema).is_err());
    }

    #[test]
    fn test_error_types() {
        let json_error = serde_json::from_str::<Value>("invalid json").unwrap_err();
        let ser_error = SerializationError::from(json_error);
        assert!(matches!(ser_error, SerializationError::JsonError(_)));
        
        let unsupported = SerializationError::UnsupportedFormat("test".to_string());
        assert!(unsupported.to_string().contains("Unsupported format"));
    }
}
