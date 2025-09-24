//! RPC Server Implementation
//!
//! This module provides a comprehensive RPC server for handling blockchain operations,
//! including transaction processing, block queries, and smart contract interactions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;

/// RPC server error types
#[derive(Error, Debug)]
pub enum RpcError {
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    #[error("Method not found: {0}")]
    MethodNotFound(String),
    
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
    
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    
    #[error("Authentication failed")]
    AuthenticationFailed,
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

pub type Result<T> = std::result::Result<T, RpcError>;

/// JSON-RPC 2.0 request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC version (must be "2.0")
    pub jsonrpc: String,
    
    /// Method name to call
    pub method: String,
    
    /// Method parameters (optional)
    pub params: Option<serde_json::Value>,
    
    /// Request ID (optional for notifications)
    pub id: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC version
    pub jsonrpc: String,
    
    /// Result data (mutually exclusive with error)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    
    /// Error data (mutually exclusive with result)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    
    /// Request ID (same as request)
    pub id: Option<serde_json::Value>,
}

/// JSON-RPC error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code
    pub code: i32,
    
    /// Error message
    pub message: String,
    
    /// Additional error data (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// RPC method handler trait
pub trait RpcMethodHandler: Send + Sync {
    /// Handle an RPC method call
    fn handle(&self, params: Option<serde_json::Value>) -> Result<serde_json::Value>;
    
    /// Get method description
    fn description(&self) -> &str;
    
    /// Get parameter schema (optional)
    fn param_schema(&self) -> Option<&str> {
        None
    }
}

/// RPC server configuration
#[derive(Debug, Clone)]
pub struct RpcServerConfig {
    /// Server bind address
    pub bind_address: String,
    
    /// Server port
    pub port: u16,
    
    /// Maximum concurrent connections
    pub max_connections: usize,
    
    /// Request timeout in seconds
    pub request_timeout: u64,
    
    /// Enable CORS
    pub enable_cors: bool,
    
    /// API key for authentication (optional)
    pub api_key: Option<String>,
    
    /// Rate limiting: requests per minute
    pub rate_limit: Option<u64>,
}

/// RPC server instance
pub struct RpcServer {
    /// Server configuration
    config: RpcServerConfig,
    
    /// Registered method handlers
    methods: Arc<Mutex<HashMap<String, Box<dyn RpcMethodHandler>>>>,
    
    /// Server statistics
    stats: Arc<Mutex<ServerStats>>,
}

/// Server statistics
#[derive(Debug, Clone)]
pub struct ServerStats {
    /// Total requests received
    pub total_requests: u64,
    
    /// Successful requests
    pub successful_requests: u64,
    
    /// Failed requests
    pub failed_requests: u64,
    
    /// Current active connections
    pub active_connections: u64,
    
    /// Server start time
    pub start_time: std::time::SystemTime,
    
    /// Methods call counts
    pub method_calls: HashMap<String, u64>,
}

/// Blockchain-specific RPC methods
pub struct BlockchainRpcMethods;

impl RpcServer {
    /// Create a new RPC server
    pub fn new(config: RpcServerConfig) -> Self {
        Self {
            config,
            methods: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(ServerStats::default())),
        }
    }
    
    /// Register an RPC method handler
    pub fn register_method<H>(&self, method_name: &str, handler: H) -> Result<()>
    where
        H: RpcMethodHandler + 'static,
    {
        let mut methods = self.methods.lock().unwrap();
        if methods.contains_key(method_name) {
            return Err(RpcError::InvalidRequest(format!(
                "Method '{}' already registered",
                method_name
            )));
        }
        
        methods.insert(method_name.to_string(), Box::new(handler));
        Ok(())
    }
    
    /// Handle a JSON-RPC request
    pub fn handle_request(&self, request: &str) -> String {
        // Parse the request
        let parsed_request: JsonRpcRequest = match serde_json::from_str(request) {
            Ok(req) => req,
            Err(_) => {
                return self.create_error_response(
                    None,
                    -32700,
                    "Parse error".to_string(),
                    None,
                );
            }
        };
        
        // Validate JSON-RPC version
        if parsed_request.jsonrpc != "2.0" {
            return self.create_error_response(
                parsed_request.id,
                -32600,
                "Invalid Request".to_string(),
                Some(serde_json::json!({"reason": "JSON-RPC version must be 2.0"})),
            );
        }
        
        // Update statistics
        {
            let mut stats = self.stats.lock().unwrap();
            stats.total_requests += 1;
            *stats.method_calls.entry(parsed_request.method.clone()).or_insert(0) += 1;
        }
        
        // Find and execute the method handler
        let methods = self.methods.lock().unwrap();
        match methods.get(&parsed_request.method) {
            Some(handler) => {
                match handler.handle(parsed_request.params) {
                    Ok(result) => {
                        // Update success statistics
                        {
                            let mut stats = self.stats.lock().unwrap();
                            stats.successful_requests += 1;
                        }
                        
                        self.create_success_response(parsed_request.id, result)
                    }
                    Err(error) => {
                        // Update failure statistics
                        {
                            let mut stats = self.stats.lock().unwrap();
                            stats.failed_requests += 1;
                        }
                        
                        let (code, message) = match error {
                            RpcError::InvalidParams(msg) => (-32602, msg),
                            RpcError::MethodNotFound(msg) => (-32601, msg),
                            RpcError::InternalError(msg) => (-32603, msg),
                            RpcError::ServiceUnavailable(msg) => (-32000, msg),
                            RpcError::AuthenticationFailed => (-32001, "Authentication failed".to_string()),
                            RpcError::RateLimitExceeded => (-32002, "Rate limit exceeded".to_string()),
                            _ => (-32603, "Internal error".to_string()),
                        };
                        
                        self.create_error_response(parsed_request.id, code, message, None)
                    }
                }
            }
            None => {
                // Update failure statistics
                {
                    let mut stats = self.stats.lock().unwrap();
                    stats.failed_requests += 1;
                }
                
                self.create_error_response(
                    parsed_request.id,
                    -32601,
                    format!("Method not found: {}", parsed_request.method),
                    None,
                )
            }
        }
    }
    
    /// Get list of registered methods
    pub fn get_registered_methods(&self) -> Vec<String> {
        let methods = self.methods.lock().unwrap();
        methods.keys().cloned().collect()
    }
    
    /// Get server statistics
    pub fn get_stats(&self) -> ServerStats {
        let stats = self.stats.lock().unwrap();
        stats.clone()
    }
    
    /// Reset server statistics
    pub fn reset_stats(&self) {
        let mut stats = self.stats.lock().unwrap();
        *stats = ServerStats::default();
    }
    
    /// Create a success response
    fn create_success_response(
        &self,
        id: Option<serde_json::Value>,
        result: serde_json::Value,
    ) -> String {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id: id.clone(),
        };
        
        serde_json::to_string(&response).unwrap_or_else(|_| {
            self.create_error_response(id, -32603, "Internal error serializing response".to_string(), None)
        })
    }
    
    /// Create an error response
    fn create_error_response(
        &self,
        id: Option<serde_json::Value>,
        code: i32,
        message: String,
        data: Option<serde_json::Value>,
    ) -> String {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError { code, message, data }),
            id,
        };
        
        serde_json::to_string(&response).unwrap_or_else(|_| {
            r#"{"jsonrpc": "2.0", "error": {"code": -32603, "message": "Internal error"}, "id": null}"#.to_string()
        })
    }
}

impl Default for RpcServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1".to_string(),
            port: 8545,
            max_connections: 100,
            request_timeout: 30,
            enable_cors: true,
            api_key: None,
            rate_limit: Some(1000), // 1000 requests per minute
        }
    }
}

impl Default for ServerStats {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            active_connections: 0,
            start_time: std::time::SystemTime::now(),
            method_calls: HashMap::new(),
        }
    }
}

// Example method handlers for common blockchain operations
impl BlockchainRpcMethods {
    /// Create a ping handler
    pub fn ping_handler() -> impl RpcMethodHandler {
        PingHandler
    }
    
    /// Create a get block handler
    pub fn get_block_handler() -> impl RpcMethodHandler {
        GetBlockHandler
    }
    
    /// Create a send transaction handler
    pub fn send_transaction_handler() -> impl RpcMethodHandler {
        SendTransactionHandler
    }
    
    /// Create a get balance handler
    pub fn get_balance_handler() -> impl RpcMethodHandler {
        GetBalanceHandler
    }
}

/// Simple ping handler
struct PingHandler;

impl RpcMethodHandler for PingHandler {
    fn handle(&self, _params: Option<serde_json::Value>) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "status": "ok",
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        }))
    }
    
    fn description(&self) -> &str {
        "Ping the server to check if it's alive"
    }
}

/// Get block handler
struct GetBlockHandler;

impl RpcMethodHandler for GetBlockHandler {
    fn handle(&self, params: Option<serde_json::Value>) -> Result<serde_json::Value> {
        let params = params.ok_or_else(|| RpcError::InvalidParams("Missing parameters".to_string()))?;
        
        let block_number = params.get("block_number")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| RpcError::InvalidParams("Invalid block_number parameter".to_string()))?;
        
        // Mock block data
        Ok(serde_json::json!({
            "number": block_number,
            "hash": format!("0x{:064x}", block_number),
            "parent_hash": format!("0x{:064x}", block_number.saturating_sub(1)),
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            "transactions": [],
            "size": 1024
        }))
    }
    
    fn description(&self) -> &str {
        "Get block information by block number"
    }
    
    fn param_schema(&self) -> Option<&str> {
        Some(r#"{"block_number": "integer"}"#)
    }
}

/// Send transaction handler
struct SendTransactionHandler;

impl RpcMethodHandler for SendTransactionHandler {
    fn handle(&self, params: Option<serde_json::Value>) -> Result<serde_json::Value> {
        let params = params.ok_or_else(|| RpcError::InvalidParams("Missing parameters".to_string()))?;
        
        let _from = params.get("from")
            .and_then(|v| v.as_str())
            .ok_or_else(|| RpcError::InvalidParams("Missing 'from' parameter".to_string()))?;
            
        let _to = params.get("to")
            .and_then(|v| v.as_str())
            .ok_or_else(|| RpcError::InvalidParams("Missing 'to' parameter".to_string()))?;
            
        let _value = params.get("value")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| RpcError::InvalidParams("Missing 'value' parameter".to_string()))?;
        
        // Mock transaction hash
        let tx_hash = format!("0x{:064x}", rand::random::<u64>());
        
        Ok(serde_json::json!({
            "transaction_hash": tx_hash,
            "status": "pending"
        }))
    }
    
    fn description(&self) -> &str {
        "Send a transaction to the network"
    }
    
    fn param_schema(&self) -> Option<&str> {
        Some(r#"{"from": "string", "to": "string", "value": "integer"}"#)
    }
}

/// Get balance handler
struct GetBalanceHandler;

impl RpcMethodHandler for GetBalanceHandler {
    fn handle(&self, params: Option<serde_json::Value>) -> Result<serde_json::Value> {
        let params = params.ok_or_else(|| RpcError::InvalidParams("Missing parameters".to_string()))?;
        
        let address = params.get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| RpcError::InvalidParams("Missing 'address' parameter".to_string()))?;
        
        // Mock balance based on address hash
        let balance = address.len() as u64 * 1000;
        
        Ok(serde_json::json!({
            "address": address,
            "balance": balance,
            "unit": "CC"
        }))
    }
    
    fn description(&self) -> &str {
        "Get account balance for an address"
    }
    
    fn param_schema(&self) -> Option<&str> {
        Some(r#"{"address": "string"}"#)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rpc_server_creation() {
        let config = RpcServerConfig::default();
        let server = RpcServer::new(config);
        
        assert_eq!(server.get_registered_methods().len(), 0);
        assert_eq!(server.get_stats().total_requests, 0);
    }
    
    #[test]
    fn test_method_registration() {
        let config = RpcServerConfig::default();
        let server = RpcServer::new(config);
        
        let result = server.register_method("ping", BlockchainRpcMethods::ping_handler());
        assert!(result.is_ok());
        
        assert_eq!(server.get_registered_methods().len(), 1);
        assert!(server.get_registered_methods().contains(&"ping".to_string()));
    }
    
    #[test]
    fn test_ping_request() {
        let config = RpcServerConfig::default();
        let server = RpcServer::new(config);
        
        server.register_method("ping", BlockchainRpcMethods::ping_handler()).unwrap();
        
        let request = r#"{"jsonrpc": "2.0", "method": "ping", "id": 1}"#;
        let response = server.handle_request(request);
        
        assert!(response.contains("\"result\""));
        assert!(response.contains("\"status\":\"ok\""));
        
        let stats = server.get_stats();
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.successful_requests, 1);
    }
    
    #[test]
    fn test_invalid_method() {
        let config = RpcServerConfig::default();
        let server = RpcServer::new(config);
        
        let request = r#"{"jsonrpc": "2.0", "method": "unknown_method", "id": 1}"#;
        let response = server.handle_request(request);
        
        assert!(response.contains("\"error\""));
        assert!(response.contains("-32601"));
        
        let stats = server.get_stats();
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.failed_requests, 1);
    }
    
    #[test]
    fn test_invalid_json() {
        let config = RpcServerConfig::default();
        let server = RpcServer::new(config);
        
        let request = r#"{"invalid": "json""#;
        let response = server.handle_request(request);
        
        assert!(response.contains("\"error\""));
        assert!(response.contains("-32700"));
    }
    
    #[test] 
    fn test_get_block_with_params() {
        let config = RpcServerConfig::default();
        let server = RpcServer::new(config);
        
        server.register_method("get_block", BlockchainRpcMethods::get_block_handler()).unwrap();
        
        let request = r#"{"jsonrpc": "2.0", "method": "get_block", "params": {"block_number": 123}, "id": 1}"#;
        let response = server.handle_request(request);
        
        assert!(response.contains("\"result\""));
        assert!(response.contains("\"number\":123"));
        
        let stats = server.get_stats();
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.successful_requests, 1);
    }
}
