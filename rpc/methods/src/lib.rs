//! CC Chain RPC Methods
//!
//! This module implements the core RPC methods for interacting with the CC Chain blockchain.
//! It provides a standardized interface for querying blockchain state, submitting transactions,
//! and retrieving various blockchain information.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RpcMethodError {
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
    #[error("Method not found: {0}")]
    MethodNotFound(String),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
}

pub type Result<T> = std::result::Result<T, RpcMethodError>;

/// Standard JSON-RPC 2.0 request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
    pub id: Option<Value>,
}

/// Standard JSON-RPC 2.0 response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
    pub id: Option<Value>,
}

/// JSON-RPC 2.0 error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Block information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInfo {
    pub height: u64,
    pub hash: String,
    pub parent_hash: String,
    pub timestamp: u64,
    pub transaction_count: u32,
    pub size: u64,
    pub validator: String,
}

/// Transaction information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInfo {
    pub hash: String,
    pub from: String,
    pub to: Option<String>,
    pub value: u64,
    pub gas_limit: u64,
    pub gas_used: Option<u64>,
    pub status: TransactionStatus,
    pub block_height: Option<u64>,
    pub block_hash: Option<String>,
    pub transaction_index: Option<u32>,
}

/// Transaction status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
}

/// Account balance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub address: String,
    pub balance: u64,
    pub nonce: u64,
    pub code_hash: Option<String>,
}

/// Network information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub chain_id: String,
    pub network_name: String,
    pub latest_block_height: u64,
    pub latest_block_hash: String,
    pub peer_count: u32,
    pub is_syncing: bool,
    pub sync_progress: Option<f64>,
}

/// Core RPC methods implementation
pub struct RpcMethods {
    handlers: HashMap<String, Box<dyn Fn(&Value) -> Result<Value> + Send + Sync>>,
}

impl RpcMethods {
    /// Create a new RPC methods handler with default methods
    pub fn new() -> Self {
        let mut methods = Self {
            handlers: HashMap::new(),
        };
        
        methods.register_default_methods();
        methods
    }

    /// Register all default RPC methods
    fn register_default_methods(&mut self) {
        // Blockchain query methods
        self.register("cc_getBlockByHeight", Box::new(Self::get_block_by_height));
        self.register("cc_getBlockByHash", Box::new(Self::get_block_by_hash));
        self.register("cc_getLatestBlock", Box::new(Self::get_latest_block));
        self.register("cc_getTransaction", Box::new(Self::get_transaction));
        self.register("cc_getAccount", Box::new(Self::get_account));
        self.register("cc_getBalance", Box::new(Self::get_balance));
        
        // Network information methods
        self.register("cc_getNetworkInfo", Box::new(Self::get_network_info));
        self.register("cc_getPeerCount", Box::new(Self::get_peer_count));
        self.register("cc_getSyncStatus", Box::new(Self::get_sync_status));
        
        // Transaction methods
        self.register("cc_sendTransaction", Box::new(Self::send_transaction));
        self.register("cc_estimateGas", Box::new(Self::estimate_gas));
        self.register("cc_getTransactionCount", Box::new(Self::get_transaction_count));
        
        // Utility methods
        self.register("cc_getVersion", Box::new(Self::get_version));
        self.register("cc_ping", Box::new(Self::ping));
    }

    /// Register a new RPC method
    pub fn register(&mut self, method: &str, handler: Box<dyn Fn(&Value) -> Result<Value> + Send + Sync>) {
        self.handlers.insert(method.to_string(), handler);
    }

    /// Execute an RPC method
    pub fn execute(&self, request: &RpcRequest) -> RpcResponse {
        let response_id = request.id.clone();
        
        match self.handlers.get(&request.method) {
            Some(handler) => {
                match handler(request.params.as_ref().unwrap_or(&Value::Null)) {
                    Ok(result) => RpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: Some(result),
                        error: None,
                        id: response_id,
                    },
                    Err(e) => RpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: Some(RpcError {
                            code: -32603,
                            message: e.to_string(),
                            data: None,
                        }),
                        id: response_id,
                    },
                }
            }
            None => RpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(RpcError {
                    code: -32601,
                    message: format!("Method '{}' not found", request.method),
                    data: None,
                }),
                id: response_id,
            },
        }
    }

    /// Get available RPC methods
    pub fn get_available_methods(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }

    // Default method implementations (mock implementations for now)
    
    fn get_block_by_height(params: &Value) -> Result<Value> {
        let height = params.get("height")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| RpcMethodError::InvalidParameters("Missing or invalid 'height' parameter".to_string()))?;
            
        let block = BlockInfo {
            height,
            hash: format!("0x{:064x}", height * 12345),
            parent_hash: format!("0x{:064x}", (height - 1) * 12345),
            timestamp: 1640000000 + height * 10,
            transaction_count: (height % 100) as u32,
            size: 1024 + height * 100,
            validator: format!("validator_{}", height % 10),
        };
        
        Ok(serde_json::to_value(block).unwrap())
    }

    fn get_block_by_hash(params: &Value) -> Result<Value> {
        let hash = params.get("hash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| RpcMethodError::InvalidParameters("Missing or invalid 'hash' parameter".to_string()))?;
            
        // Mock block based on hash
        let height = hash.len() as u64; // Simple mock
        let block = BlockInfo {
            height,
            hash: hash.to_string(),
            parent_hash: format!("0x{:064x}", (height - 1) * 12345),
            timestamp: 1640000000 + height * 10,
            transaction_count: (height % 100) as u32,
            size: 1024 + height * 100,
            validator: format!("validator_{}", height % 10),
        };
        
        Ok(serde_json::to_value(block).unwrap())
    }

    fn get_latest_block(_params: &Value) -> Result<Value> {
        let block = BlockInfo {
            height: 12345,
            hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            parent_hash: "0x0987654321fedcba0987654321fedcba0987654321fedcba0987654321fedcba".to_string(),
            timestamp: 1640000000,
            transaction_count: 42,
            size: 2048,
            validator: "validator_0".to_string(),
        };
        
        Ok(serde_json::to_value(block).unwrap())
    }

    fn get_transaction(params: &Value) -> Result<Value> {
        let hash = params.get("hash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| RpcMethodError::InvalidParameters("Missing or invalid 'hash' parameter".to_string()))?;
            
        let tx = TransactionInfo {
            hash: hash.to_string(),
            from: "0xsender123456789abcdef".to_string(),
            to: Some("0xrecipient987654321".to_string()),
            value: 1000000,
            gas_limit: 21000,
            gas_used: Some(21000),
            status: TransactionStatus::Confirmed,
            block_height: Some(12344),
            block_hash: Some("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string()),
            transaction_index: Some(0),
        };
        
        Ok(serde_json::to_value(tx).unwrap())
    }

    fn get_account(params: &Value) -> Result<Value> {
        let address = params.get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| RpcMethodError::InvalidParameters("Missing or invalid 'address' parameter".to_string()))?;
            
        let account = AccountInfo {
            address: address.to_string(),
            balance: 5000000000, // 5 billion units
            nonce: 42,
            code_hash: None,
        };
        
        Ok(serde_json::to_value(account).unwrap())
    }

    fn get_balance(params: &Value) -> Result<Value> {
        let _address = params.get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| RpcMethodError::InvalidParameters("Missing or invalid 'address' parameter".to_string()))?;
            
        Ok(json!("5000000000"))
    }

    fn get_network_info(_params: &Value) -> Result<Value> {
        let info = NetworkInfo {
            chain_id: "cc-chain-1".to_string(),
            network_name: "CC Chain Mainnet".to_string(),
            latest_block_height: 12345,
            latest_block_hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            peer_count: 23,
            is_syncing: false,
            sync_progress: None,
        };
        
        Ok(serde_json::to_value(info).unwrap())
    }

    fn get_peer_count(_params: &Value) -> Result<Value> {
        Ok(json!(23))
    }

    fn get_sync_status(_params: &Value) -> Result<Value> {
        Ok(json!({
            "is_syncing": false,
            "progress": null,
            "current_height": 12345,
            "target_height": 12345
        }))
    }

    fn send_transaction(params: &Value) -> Result<Value> {
        let _tx_data = params.get("transaction")
            .ok_or_else(|| RpcMethodError::InvalidParameters("Missing 'transaction' parameter".to_string()))?;
            
        // Mock transaction hash
        let tx_hash = "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";
        Ok(json!(tx_hash))
    }

    fn estimate_gas(params: &Value) -> Result<Value> {
        let _tx_data = params.get("transaction")
            .ok_or_else(|| RpcMethodError::InvalidParameters("Missing 'transaction' parameter".to_string()))?;
            
        Ok(json!("21000"))
    }

    fn get_transaction_count(params: &Value) -> Result<Value> {
        let _address = params.get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| RpcMethodError::InvalidParameters("Missing or invalid 'address' parameter".to_string()))?;
            
        Ok(json!(42))
    }

    fn get_version(_params: &Value) -> Result<Value> {
        Ok(json!({
            "version": "1.0.0",
            "build": "cc-chain-1.0.0",
            "commit": "abc123def"
        }))
    }

    fn ping(_params: &Value) -> Result<Value> {
        Ok(json!("pong"))
    }
}

impl Default for RpcMethods {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_methods_creation() {
        let methods = RpcMethods::new();
        let available = methods.get_available_methods();
        assert!(!available.is_empty());
        assert!(available.contains(&"cc_getLatestBlock".to_string()));
        assert!(available.contains(&"cc_ping".to_string()));
    }

    #[test]
    fn test_ping_method() {
        let methods = RpcMethods::new();
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "cc_ping".to_string(),
            params: None,
            id: Some(json!(1)),
        };
        
        let response = methods.execute(&request);
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.error.is_none());
        if let Some(result) = response.result {
            assert_eq!(result, json!("pong"));
        }
    }

    #[test]
    fn test_get_latest_block() {
        let methods = RpcMethods::new();
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "cc_getLatestBlock".to_string(),
            params: None,
            id: Some(json!(2)),
        };
        
        let response = methods.execute(&request);
        assert!(response.error.is_none());
        assert!(response.result.is_some());
    }

    #[test]
    fn test_get_block_by_height() {
        let methods = RpcMethods::new();
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "cc_getBlockByHeight".to_string(),
            params: Some(json!({"height": 12345})),
            id: Some(json!(3)),
        };
        
        let response = methods.execute(&request);
        assert!(response.error.is_none());
        if let Some(result) = response.result {
            let block: BlockInfo = serde_json::from_value(result).unwrap();
            assert_eq!(block.height, 12345);
        }
    }

    #[test]
    fn test_method_not_found() {
        let methods = RpcMethods::new();
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "nonexistent_method".to_string(),
            params: None,
            id: Some(json!(4)),
        };
        
        let response = methods.execute(&request);
        assert!(response.result.is_none());
        assert!(response.error.is_some());
        
        if let Some(error) = response.error {
            assert_eq!(error.code, -32601);
        }
    }

    #[test]
    fn test_invalid_parameters() {
        let methods = RpcMethods::new();
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "cc_getBlockByHeight".to_string(),
            params: Some(json!({"invalid": "param"})),
            id: Some(json!(5)),
        };
        
        let response = methods.execute(&request);
        assert!(response.result.is_none());
        assert!(response.error.is_some());
    }

    #[test]
    fn test_get_balance() {
        let methods = RpcMethods::new();
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "cc_getBalance".to_string(),
            params: Some(json!({"address": "0x123456789abcdef"})),
            id: Some(json!(6)),
        };
        
        let response = methods.execute(&request);
        assert!(response.error.is_none());
        if let Some(result) = response.result {
            assert_eq!(result, json!("5000000000"));
        }
    }

    #[test]
    fn test_send_transaction() {
        let methods = RpcMethods::new();
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "cc_sendTransaction".to_string(),
            params: Some(json!({"transaction": {"from": "0x123", "to": "0x456", "value": "1000"}})),
            id: Some(json!(7)),
        };
        
        let response = methods.execute(&request);
        assert!(response.error.is_none());
        assert!(response.result.is_some());
    }
}
