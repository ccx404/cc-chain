//! CC Chain RPC Documentation
//!
//! This module provides automatic documentation generation for RPC APIs,
//! supporting multiple formats including OpenRPC, Swagger/OpenAPI, and custom formats.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DocumentationError {
    #[error("Template error: {0}")]
    TemplateError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    IoError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Format not supported: {0}")]
    UnsupportedFormat(String),
}

pub type Result<T> = std::result::Result<T, DocumentationError>;

/// Documentation generation configuration
#[derive(Debug, Clone)]
pub struct DocumentationConfig {
    pub title: String,
    pub version: String,
    pub description: String,
    pub contact: Option<ContactInfo>,
    pub license: Option<LicenseInfo>,
    pub servers: Vec<ServerInfo>,
    pub include_examples: bool,
    pub include_schemas: bool,
    pub generate_types: bool,
    pub output_format: DocumentationFormat,
}

impl Default for DocumentationConfig {
    fn default() -> Self {
        Self {
            title: "CC Chain RPC API".to_string(),
            version: "1.0.0".to_string(),
            description: "CC Chain blockchain RPC API documentation".to_string(),
            contact: Some(ContactInfo {
                name: "CC Chain Team".to_string(),
                url: Some("https://github.com/ccx404/cc-chain".to_string()),
                email: None,
            }),
            license: Some(LicenseInfo {
                name: "MIT".to_string(),
                url: Some("https://opensource.org/licenses/MIT".to_string()),
            }),
            servers: vec![
                ServerInfo {
                    url: "http://localhost:8545".to_string(),
                    description: "Local development server".to_string(),
                },
            ],
            include_examples: true,
            include_schemas: true,
            generate_types: true,
            output_format: DocumentationFormat::OpenRpc,
        }
    }
}

/// Contact information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactInfo {
    pub name: String,
    pub url: Option<String>,
    pub email: Option<String>,
}

/// License information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseInfo {
    pub name: String,
    pub url: Option<String>,
}

/// Server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub url: String,
    pub description: String,
}

/// Documentation output formats
#[derive(Debug, Clone, PartialEq)]
pub enum DocumentationFormat {
    OpenRpc,
    OpenApi,
    Markdown,
    Html,
    Json,
}

/// RPC method documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodDocumentation {
    pub name: String,
    pub summary: String,
    pub description: String,
    pub parameters: Vec<ParameterDoc>,
    pub result: Option<ResultDoc>,
    pub errors: Vec<ErrorDoc>,
    pub examples: Vec<ExampleDoc>,
    pub tags: Vec<String>,
    pub deprecated: bool,
    pub since_version: String,
}

/// Parameter documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDoc {
    pub name: String,
    pub description: String,
    pub schema: SchemaDoc,
    pub required: bool,
    pub example: Option<Value>,
}

/// Result documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultDoc {
    pub name: String,
    pub description: String,
    pub schema: SchemaDoc,
    pub example: Option<Value>,
}

/// Error documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDoc {
    pub code: i32,
    pub message: String,
    pub description: String,
    pub data_schema: Option<SchemaDoc>,
}

/// Example documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleDoc {
    pub name: String,
    pub summary: String,
    pub description: String,
    pub params: Option<Value>,
    pub result: Option<Value>,
}

/// Schema documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDoc {
    pub schema_type: String,
    pub format: Option<String>,
    pub description: Option<String>,
    pub properties: Option<HashMap<String, SchemaDoc>>,
    pub items: Option<Box<SchemaDoc>>,
    pub required: Option<Vec<String>>,
    pub example: Option<Value>,
    pub enum_values: Option<Vec<Value>>,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
}

/// Documentation generator
pub struct DocumentationGenerator {
    config: DocumentationConfig,
    methods: HashMap<String, MethodDocumentation>,
    schemas: HashMap<String, SchemaDoc>,
}

impl DocumentationGenerator {
    /// Create a new documentation generator
    pub fn new() -> Self {
        Self::with_config(DocumentationConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: DocumentationConfig) -> Self {
        let mut generator = Self {
            config,
            methods: HashMap::new(),
            schemas: HashMap::new(),
        };
        
        generator.register_standard_methods();
        generator.register_standard_schemas();
        generator
    }

    /// Register standard CC Chain RPC methods
    fn register_standard_methods(&mut self) {
        // Ping method
        self.add_method(MethodDocumentation {
            name: "cc_ping".to_string(),
            summary: "Ping the server".to_string(),
            description: "Returns a simple pong response to verify server connectivity".to_string(),
            parameters: vec![],
            result: Some(ResultDoc {
                name: "result".to_string(),
                description: "Pong response".to_string(),
                schema: SchemaDoc {
                    schema_type: "string".to_string(),
                    format: None,
                    description: Some("Always returns 'pong'".to_string()),
                    example: Some(json!("pong")),
                    ..Default::default()
                },
                example: Some(json!("pong")),
            }),
            errors: vec![],
            examples: vec![
                ExampleDoc {
                    name: "Basic ping".to_string(),
                    summary: "Simple ping request".to_string(),
                    description: "Test server connectivity".to_string(),
                    params: None,
                    result: Some(json!("pong")),
                },
            ],
            tags: vec!["utility".to_string()],
            deprecated: false,
            since_version: "1.0.0".to_string(),
        });

        // Get latest block method
        self.add_method(MethodDocumentation {
            name: "cc_getLatestBlock".to_string(),
            summary: "Get the latest block".to_string(),
            description: "Returns information about the most recent block in the blockchain".to_string(),
            parameters: vec![],
            result: Some(ResultDoc {
                name: "block".to_string(),
                description: "Latest block information".to_string(),
                schema: self.create_block_schema(),
                example: Some(json!({
                    "height": 12345,
                    "hash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
                    "parent_hash": "0x0987654321fedcba0987654321fedcba0987654321fedcba0987654321fedcba",
                    "timestamp": 1640000000,
                    "transaction_count": 42,
                    "size": 2048,
                    "validator": "validator_0"
                })),
            }),
            errors: vec![
                ErrorDoc {
                    code: -32603,
                    message: "Internal error".to_string(),
                    description: "Server internal error occurred".to_string(),
                    data_schema: None,
                },
            ],
            examples: vec![
                ExampleDoc {
                    name: "Get latest block".to_string(),
                    summary: "Retrieve the most recent block".to_string(),
                    description: "Returns the latest block with all its properties".to_string(),
                    params: None,
                    result: Some(json!({
                        "height": 12345,
                        "hash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
                        "timestamp": 1640000000,
                        "transaction_count": 42
                    })),
                },
            ],
            tags: vec!["blockchain".to_string(), "blocks".to_string()],
            deprecated: false,
            since_version: "1.0.0".to_string(),
        });

        // Get block by height method
        self.add_method(MethodDocumentation {
            name: "cc_getBlockByHeight".to_string(),
            summary: "Get block by height".to_string(),
            description: "Returns block information for the specified block height".to_string(),
            parameters: vec![
                ParameterDoc {
                    name: "height".to_string(),
                    description: "Block height to retrieve".to_string(),
                    schema: SchemaDoc {
                        schema_type: "integer".to_string(),
                        format: Some("uint64".to_string()),
                        description: Some("Block height (0 or greater)".to_string()),
                        minimum: Some(0.0),
                        example: Some(json!(12345)),
                        ..Default::default()
                    },
                    required: true,
                    example: Some(json!(12345)),
                },
            ],
            result: Some(ResultDoc {
                name: "block".to_string(),
                description: "Block information".to_string(),
                schema: self.create_block_schema(),
                example: Some(json!({
                    "height": 12345,
                    "hash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
                })),
            }),
            errors: vec![
                ErrorDoc {
                    code: -32602,
                    message: "Invalid params".to_string(),
                    description: "Invalid block height parameter".to_string(),
                    data_schema: None,
                },
                ErrorDoc {
                    code: -32007,
                    message: "Block not found".to_string(),
                    description: "Block with specified height does not exist".to_string(),
                    data_schema: Some(SchemaDoc {
                        schema_type: "object".to_string(),
                        properties: Some({
                            let mut props = HashMap::new();
                            props.insert("height".to_string(), SchemaDoc {
                                schema_type: "integer".to_string(),
                                description: Some("Requested block height".to_string()),
                                ..Default::default()
                            });
                            props
                        }),
                        ..Default::default()
                    }),
                },
            ],
            examples: vec![
                ExampleDoc {
                    name: "Get block by height".to_string(),
                    summary: "Retrieve a specific block".to_string(),
                    description: "Get block information using block height".to_string(),
                    params: Some(json!({"height": 12345})),
                    result: Some(json!({
                        "height": 12345,
                        "hash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
                    })),
                },
            ],
            tags: vec!["blockchain".to_string(), "blocks".to_string()],
            deprecated: false,
            since_version: "1.0.0".to_string(),
        });
    }

    /// Register standard schemas
    fn register_standard_schemas(&mut self) {
        self.schemas.insert("Block".to_string(), self.create_block_schema());
        self.schemas.insert("Transaction".to_string(), self.create_transaction_schema());
        self.schemas.insert("Account".to_string(), self.create_account_schema());
    }

    fn create_block_schema(&self) -> SchemaDoc {
        let mut properties = HashMap::new();
        
        properties.insert("height".to_string(), SchemaDoc {
            schema_type: "integer".to_string(),
            format: Some("uint64".to_string()),
            description: Some("Block height".to_string()),
            example: Some(json!(12345)),
            ..Default::default()
        });
        
        properties.insert("hash".to_string(), SchemaDoc {
            schema_type: "string".to_string(),
            format: Some("hex".to_string()),
            description: Some("Block hash".to_string()),
            min_length: Some(66),
            max_length: Some(66),
            example: Some(json!("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef")),
            ..Default::default()
        });
        
        properties.insert("timestamp".to_string(), SchemaDoc {
            schema_type: "integer".to_string(),
            format: Some("uint64".to_string()),
            description: Some("Block timestamp (Unix epoch)".to_string()),
            example: Some(json!(1640000000)),
            ..Default::default()
        });

        SchemaDoc {
            schema_type: "object".to_string(),
            description: Some("Block information".to_string()),
            properties: Some(properties),
            required: Some(vec!["height".to_string(), "hash".to_string(), "timestamp".to_string()]),
            ..Default::default()
        }
    }

    fn create_transaction_schema(&self) -> SchemaDoc {
        let mut properties = HashMap::new();
        
        properties.insert("hash".to_string(), SchemaDoc {
            schema_type: "string".to_string(),
            format: Some("hex".to_string()),
            description: Some("Transaction hash".to_string()),
            example: Some(json!("0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890")),
            ..Default::default()
        });
        
        properties.insert("from".to_string(), SchemaDoc {
            schema_type: "string".to_string(),
            format: Some("address".to_string()),
            description: Some("Sender address".to_string()),
            example: Some(json!("0x1234567890abcdef1234567890abcdef12345678")),
            ..Default::default()
        });
        
        properties.insert("value".to_string(), SchemaDoc {
            schema_type: "string".to_string(),
            format: Some("uint256".to_string()),
            description: Some("Transaction value".to_string()),
            example: Some(json!("1000000000000000000")),
            ..Default::default()
        });

        SchemaDoc {
            schema_type: "object".to_string(),
            description: Some("Transaction information".to_string()),
            properties: Some(properties),
            required: Some(vec!["hash".to_string(), "from".to_string()]),
            ..Default::default()
        }
    }

    fn create_account_schema(&self) -> SchemaDoc {
        let mut properties = HashMap::new();
        
        properties.insert("address".to_string(), SchemaDoc {
            schema_type: "string".to_string(),
            format: Some("address".to_string()),
            description: Some("Account address".to_string()),
            example: Some(json!("0x1234567890abcdef1234567890abcdef12345678")),
            ..Default::default()
        });
        
        properties.insert("balance".to_string(), SchemaDoc {
            schema_type: "string".to_string(),
            format: Some("uint256".to_string()),
            description: Some("Account balance".to_string()),
            example: Some(json!("5000000000000000000")),
            ..Default::default()
        });

        SchemaDoc {
            schema_type: "object".to_string(),
            description: Some("Account information".to_string()),
            properties: Some(properties),
            required: Some(vec!["address".to_string(), "balance".to_string()]),
            ..Default::default()
        }
    }

    /// Add a method to the documentation
    pub fn add_method(&mut self, method: MethodDocumentation) {
        self.methods.insert(method.name.clone(), method);
    }

    /// Add a schema to the documentation
    pub fn add_schema(&mut self, name: String, schema: SchemaDoc) {
        self.schemas.insert(name, schema);
    }

    /// Generate documentation in the specified format
    pub fn generate(&self) -> Result<String> {
        match self.config.output_format {
            DocumentationFormat::OpenRpc => self.generate_openrpc(),
            DocumentationFormat::OpenApi => self.generate_openapi(),
            DocumentationFormat::Markdown => self.generate_markdown(),
            DocumentationFormat::Html => self.generate_html(),
            DocumentationFormat::Json => self.generate_json(),
        }
    }

    /// Generate OpenRPC specification
    fn generate_openrpc(&self) -> Result<String> {
        let spec = json!({
            "openrpc": "1.2.6",
            "info": {
                "title": self.config.title,
                "version": self.config.version,
                "description": self.config.description,
                "contact": self.config.contact,
                "license": self.config.license
            },
            "servers": self.config.servers,
            "methods": self.methods.values().map(|method| {
                json!({
                    "name": method.name,
                    "summary": method.summary,
                    "description": method.description,
                    "params": method.parameters.iter().map(|param| {
                        json!({
                            "name": param.name,
                            "description": param.description,
                            "required": param.required,
                            "schema": param.schema
                        })
                    }).collect::<Vec<_>>(),
                    "result": method.result.as_ref().map(|result| {
                        json!({
                            "name": result.name,
                            "description": result.description,
                            "schema": result.schema
                        })
                    }),
                    "errors": method.errors.iter().map(|error| {
                        json!({
                            "code": error.code,
                            "message": error.message,
                            "description": error.description,
                            "data": error.data_schema
                        })
                    }).collect::<Vec<_>>(),
                    "examples": if self.config.include_examples {
                        method.examples.iter().map(|example| {
                            json!({
                                "name": example.name,
                                "summary": example.summary,
                                "description": example.description,
                                "params": example.params,
                                "result": example.result
                            })
                        }).collect::<Vec<_>>()
                    } else {
                        vec![]
                    },
                    "tags": method.tags,
                    "deprecated": method.deprecated
                })
            }).collect::<Vec<_>>(),
            "components": if self.config.include_schemas {
                json!({
                    "schemas": self.schemas
                })
            } else {
                json!({})
            }
        });

        Ok(serde_json::to_string_pretty(&spec)?)
    }

    /// Generate OpenAPI specification
    fn generate_openapi(&self) -> Result<String> {
        let spec = json!({
            "openapi": "3.0.3",
            "info": {
                "title": self.config.title,
                "version": self.config.version,
                "description": self.config.description,
                "contact": self.config.contact,
                "license": self.config.license
            },
            "servers": self.config.servers,
            "paths": {
                "/": {
                    "post": {
                        "summary": "JSON-RPC 2.0 Endpoint",
                        "description": "All RPC methods are accessed via POST to this endpoint",
                        "requestBody": {
                            "required": true,
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/JsonRpcRequest"
                                    }
                                }
                            }
                        },
                        "responses": {
                            "200": {
                                "description": "JSON-RPC response",
                                "content": {
                                    "application/json": {
                                        "schema": {
                                            "$ref": "#/components/schemas/JsonRpcResponse"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "components": {
                "schemas": self.generate_openapi_schemas()?
            }
        });

        Ok(serde_json::to_string_pretty(&spec)?)
    }

    fn generate_openapi_schemas(&self) -> Result<Value> {
        let mut schemas = HashMap::new();
        
        // Basic JSON-RPC schemas
        schemas.insert("JsonRpcRequest", json!({
            "type": "object",
            "required": ["jsonrpc", "method"],
            "properties": {
                "jsonrpc": {
                    "type": "string",
                    "enum": ["2.0"]
                },
                "method": {
                    "type": "string"
                },
                "params": {
                    "oneOf": [
                        {"type": "array"},
                        {"type": "object"}
                    ]
                },
                "id": {
                    "oneOf": [
                        {"type": "string"},
                        {"type": "number"},
                        {"type": "null"}
                    ]
                }
            }
        }));
        
        schemas.insert("JsonRpcResponse", json!({
            "type": "object",
            "required": ["jsonrpc"],
            "properties": {
                "jsonrpc": {
                    "type": "string",
                    "enum": ["2.0"]
                },
                "result": {},
                "error": {
                    "$ref": "#/components/schemas/JsonRpcError"
                },
                "id": {
                    "oneOf": [
                        {"type": "string"},
                        {"type": "number"},
                        {"type": "null"}
                    ]
                }
            }
        }));
        
        schemas.insert("JsonRpcError", json!({
            "type": "object",
            "required": ["code", "message"],
            "properties": {
                "code": {
                    "type": "integer"
                },
                "message": {
                    "type": "string"
                },
                "data": {}
            }
        }));

        // Add custom schemas
        for (name, schema) in &self.schemas {
            schemas.insert(name.as_str(), serde_json::to_value(schema)?);
        }

        Ok(json!(schemas))
    }

    /// Generate Markdown documentation
    fn generate_markdown(&self) -> Result<String> {
        let mut markdown = String::new();
        
        markdown.push_str(&format!("# {}\n\n", self.config.title));
        markdown.push_str(&format!("{}\n\n", self.config.description));
        markdown.push_str(&format!("**Version**: {}\n\n", self.config.version));
        
        if let Some(contact) = &self.config.contact {
            markdown.push_str(&format!("**Contact**: {}", contact.name));
            if let Some(url) = &contact.url {
                markdown.push_str(&format!(" - {}", url));
            }
            markdown.push_str("\n\n");
        }

        markdown.push_str("## Servers\n\n");
        for server in &self.config.servers {
            markdown.push_str(&format!("- **{}**: {}\n", server.url, server.description));
        }
        markdown.push_str("\n");

        markdown.push_str("## Methods\n\n");
        
        let mut methods: Vec<_> = self.methods.values().collect();
        methods.sort_by(|a, b| a.name.cmp(&b.name));
        
        for method in methods {
            markdown.push_str(&format!("### {}\n\n", method.name));
            markdown.push_str(&format!("{}\n\n", method.description));
            
            if method.deprecated {
                markdown.push_str("**⚠️ Deprecated**\n\n");
            }
            
            if !method.parameters.is_empty() {
                markdown.push_str("**Parameters:**\n\n");
                for param in &method.parameters {
                    let required = if param.required { " (required)" } else { " (optional)" };
                    markdown.push_str(&format!("- `{}` ({}){}: {}\n", 
                        param.name, param.schema.schema_type, required, param.description));
                }
                markdown.push_str("\n");
            }
            
            if let Some(result) = &method.result {
                markdown.push_str(&format!("**Returns:** {} - {}\n\n", 
                    result.schema.schema_type, result.description));
            }
            
            if !method.errors.is_empty() {
                markdown.push_str("**Errors:**\n\n");
                for error in &method.errors {
                    markdown.push_str(&format!("- `{}`: {} - {}\n", 
                        error.code, error.message, error.description));
                }
                markdown.push_str("\n");
            }
            
            if self.config.include_examples && !method.examples.is_empty() {
                markdown.push_str("**Examples:**\n\n");
                for example in &method.examples {
                    markdown.push_str(&format!("*{}*\n\n", example.summary));
                    
                    if let Some(params) = &example.params {
                        markdown.push_str("Request:\n```json\n");
                        markdown.push_str(&serde_json::to_string_pretty(&json!({
                            "jsonrpc": "2.0",
                            "method": method.name,
                            "params": params,
                            "id": 1
                        }))?);
                        markdown.push_str("\n```\n\n");
                    }
                    
                    if let Some(result) = &example.result {
                        markdown.push_str("Response:\n```json\n");
                        markdown.push_str(&serde_json::to_string_pretty(&json!({
                            "jsonrpc": "2.0",
                            "result": result,
                            "id": 1
                        }))?);
                        markdown.push_str("\n```\n\n");
                    }
                }
            }
            
            markdown.push_str("---\n\n");
        }

        Ok(markdown)
    }

    /// Generate HTML documentation
    fn generate_html(&self) -> Result<String> {
        let mut html = String::new();
        
        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str(&format!("<title>{}</title>\n", self.config.title));
        html.push_str("<style>\n");
        html.push_str(include_str!("../templates/docs.css"));
        html.push_str("</style>\n");
        html.push_str("</head>\n<body>\n");
        
        html.push_str(&format!("<h1>{}</h1>\n", self.config.title));
        html.push_str(&format!("<p>{}</p>\n", self.config.description));
        html.push_str(&format!("<p><strong>Version:</strong> {}</p>\n", self.config.version));
        
        html.push_str("<h2>Methods</h2>\n");
        
        let mut methods: Vec<_> = self.methods.values().collect();
        methods.sort_by(|a, b| a.name.cmp(&b.name));
        
        for method in methods {
            html.push_str(&format!("<div class=\"method\">\n"));
            html.push_str(&format!("<h3>{}</h3>\n", method.name));
            html.push_str(&format!("<p>{}</p>\n", method.description));
            
            if !method.parameters.is_empty() {
                html.push_str("<h4>Parameters</h4>\n<ul>\n");
                for param in &method.parameters {
                    let required = if param.required { " (required)" } else { " (optional)" };
                    html.push_str(&format!("<li><code>{}</code> ({}){}: {}</li>\n", 
                        param.name, param.schema.schema_type, required, param.description));
                }
                html.push_str("</ul>\n");
            }
            
            html.push_str("</div>\n");
        }
        
        html.push_str("</body>\n</html>");
        
        Ok(html)
    }

    /// Generate JSON documentation
    fn generate_json(&self) -> Result<String> {
        let doc = json!({
            "title": self.config.title,
            "version": self.config.version,
            "description": self.config.description,
            "contact": self.config.contact,
            "license": self.config.license,
            "servers": self.config.servers,
            "methods": self.methods,
            "schemas": if self.config.include_schemas { Some(&self.schemas) } else { None }
        });

        Ok(serde_json::to_string_pretty(&doc)?)
    }

    /// Get method documentation
    pub fn get_method(&self, name: &str) -> Option<&MethodDocumentation> {
        self.methods.get(name)
    }

    /// Get all method names
    pub fn get_method_names(&self) -> Vec<String> {
        self.methods.keys().cloned().collect()
    }

    /// Get schema documentation
    pub fn get_schema(&self, name: &str) -> Option<&SchemaDoc> {
        self.schemas.get(name)
    }

    /// Export to file (mock implementation)
    pub fn export_to_file(&self, _filename: &str) -> Result<()> {
        // In a real implementation, this would write to a file
        let _content = self.generate()?;
        Ok(())
    }
}

impl Default for DocumentationGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SchemaDoc {
    fn default() -> Self {
        Self {
            schema_type: "object".to_string(),
            format: None,
            description: None,
            properties: None,
            items: None,
            required: None,
            example: None,
            enum_values: None,
            minimum: None,
            maximum: None,
            min_length: None,
            max_length: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_documentation_generator_creation() {
        let generator = DocumentationGenerator::new();
        assert_eq!(generator.config.title, "CC Chain RPC API");
        assert!(!generator.methods.is_empty());
    }

    #[test]
    fn test_custom_config() {
        let config = DocumentationConfig {
            title: "Custom API".to_string(),
            version: "2.0.0".to_string(),
            include_examples: false,
            ..Default::default()
        };
        
        let generator = DocumentationGenerator::with_config(config);
        assert_eq!(generator.config.title, "Custom API");
        assert_eq!(generator.config.version, "2.0.0");
        assert!(!generator.config.include_examples);
    }

    #[test]
    fn test_add_method() {
        let mut generator = DocumentationGenerator::new();
        let initial_count = generator.methods.len();
        
        let method = MethodDocumentation {
            name: "test_method".to_string(),
            summary: "Test method".to_string(),
            description: "A test method".to_string(),
            parameters: vec![],
            result: None,
            errors: vec![],
            examples: vec![],
            tags: vec![],
            deprecated: false,
            since_version: "1.0.0".to_string(),
        };
        
        generator.add_method(method);
        assert_eq!(generator.methods.len(), initial_count + 1);
        assert!(generator.methods.contains_key("test_method"));
    }

    #[test]
    fn test_openrpc_generation() {
        let generator = DocumentationGenerator::new();
        let openrpc = generator.generate_openrpc().unwrap();
        
        assert!(openrpc.contains("openrpc"));
        assert!(openrpc.contains("1.2.6"));
        assert!(openrpc.contains("CC Chain RPC API"));
        assert!(openrpc.contains("cc_ping"));
    }

    #[test]
    fn test_markdown_generation() {
        let mut config = DocumentationConfig::default();
        config.output_format = DocumentationFormat::Markdown;
        
        let generator = DocumentationGenerator::with_config(config);
        let markdown = generator.generate().unwrap();
        
        assert!(markdown.contains("# CC Chain RPC API"));
        assert!(markdown.contains("## Methods"));
        assert!(markdown.contains("### cc_ping"));
    }

    #[test]
    fn test_json_generation() {
        let mut config = DocumentationConfig::default();
        config.output_format = DocumentationFormat::Json;
        
        let generator = DocumentationGenerator::with_config(config);
        let json_doc = generator.generate().unwrap();
        
        assert!(json_doc.contains("CC Chain RPC API"));
        let parsed: Value = serde_json::from_str(&json_doc).unwrap();
        assert!(parsed.get("methods").is_some());
    }

    #[test]
    fn test_html_generation() {
        let mut config = DocumentationConfig::default();
        config.output_format = DocumentationFormat::Html;
        
        let generator = DocumentationGenerator::with_config(config);
        let html = generator.generate().unwrap();
        
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<title>CC Chain RPC API</title>"));
        assert!(html.contains("<h1>CC Chain RPC API</h1>"));
    }

    #[test]
    fn test_schema_creation() {
        let generator = DocumentationGenerator::new();
        let block_schema = generator.create_block_schema();
        
        assert_eq!(block_schema.schema_type, "object");
        assert!(block_schema.properties.is_some());
        
        let properties = block_schema.properties.unwrap();
        assert!(properties.contains_key("height"));
        assert!(properties.contains_key("hash"));
        assert!(properties.contains_key("timestamp"));
    }

    #[test]
    fn test_method_retrieval() {
        let generator = DocumentationGenerator::new();
        
        let ping_method = generator.get_method("cc_ping");
        assert!(ping_method.is_some());
        assert_eq!(ping_method.unwrap().name, "cc_ping");
        
        let nonexistent = generator.get_method("nonexistent");
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_method_names() {
        let generator = DocumentationGenerator::new();
        let method_names = generator.get_method_names();
        
        assert!(!method_names.is_empty());
        assert!(method_names.contains(&"cc_ping".to_string()));
        assert!(method_names.contains(&"cc_getLatestBlock".to_string()));
    }

    #[test]
    fn test_schema_retrieval() {
        let generator = DocumentationGenerator::new();
        
        let block_schema = generator.get_schema("Block");
        assert!(block_schema.is_some());
        
        let nonexistent = generator.get_schema("NonexistentSchema");
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_contact_info() {
        let contact = ContactInfo {
            name: "Test Team".to_string(),
            url: Some("https://example.com".to_string()),
            email: Some("test@example.com".to_string()),
        };
        
        assert_eq!(contact.name, "Test Team");
        assert_eq!(contact.url, Some("https://example.com".to_string()));
        assert_eq!(contact.email, Some("test@example.com".to_string()));
    }

    #[test]
    fn test_license_info() {
        let license = LicenseInfo {
            name: "MIT".to_string(),
            url: Some("https://opensource.org/licenses/MIT".to_string()),
        };
        
        assert_eq!(license.name, "MIT");
        assert!(license.url.is_some());
    }

    #[test]
    fn test_documentation_formats() {
        assert_eq!(DocumentationFormat::OpenRpc, DocumentationFormat::OpenRpc);
        assert_ne!(DocumentationFormat::OpenRpc, DocumentationFormat::Markdown);
    }

    #[test]
    fn test_export_to_file() {
        let generator = DocumentationGenerator::new();
        let result = generator.export_to_file("test.json");
        assert!(result.is_ok());
    }
}
