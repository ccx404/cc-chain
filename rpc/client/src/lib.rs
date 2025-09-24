//! CC Chain RPC Client
//!
//! This module provides a client for interacting with CC Chain RPC servers.
//! It handles connection management, request serialization, and response parsing.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RpcClientError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Request timeout: {0}")]
    TimeoutError(String),
    #[error("Server error: {code} - {message}")]
    ServerError { code: i32, message: String },
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

pub type Result<T> = std::result::Result<T, RpcClientError>;

/// RPC client configuration
#[derive(Debug, Clone)]
pub struct RpcClientConfig {
    pub endpoint: String,
    pub timeout: Duration,
    pub max_retries: u32,
    pub retry_delay: Duration,
}

impl Default for RpcClientConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:8545".to_string(),
            timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_delay: Duration::from_millis(1000),
        }
    }
}

/// JSON-RPC 2.0 request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
    pub id: Value,
}

/// JSON-RPC 2.0 response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
    pub id: Value,
}

/// JSON-RPC 2.0 error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Block information received from RPC
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

/// Transaction information received from RPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInfo {
    pub hash: String,
    pub from: String,
    pub to: Option<String>,
    pub value: u64,
    pub gas_limit: u64,
    pub gas_used: Option<u64>,
    pub status: String,
    pub block_height: Option<u64>,
    pub block_hash: Option<String>,
    pub transaction_index: Option<u32>,
}

/// Account information received from RPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub address: String,
    pub balance: u64,
    pub nonce: u64,
    pub code_hash: Option<String>,
}

/// Network information received from RPC
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

/// RPC client for communicating with CC Chain nodes
pub struct RpcClient {
    config: RpcClientConfig,
    id_counter: AtomicU64,
}

impl RpcClient {
    /// Create a new RPC client with default configuration
    pub fn new() -> Self {
        Self::with_config(RpcClientConfig::default())
    }

    /// Create a new RPC client with custom configuration
    pub fn with_config(config: RpcClientConfig) -> Self {
        Self {
            config,
            id_counter: AtomicU64::new(1),
        }
    }

    /// Create a new RPC client with custom endpoint
    pub fn with_endpoint(endpoint: &str) -> Self {
        let mut config = RpcClientConfig::default();
        config.endpoint = endpoint.to_string();
        Self::with_config(config)
    }

    /// Generate a unique request ID
    fn next_id(&self) -> u64 {
        self.id_counter.fetch_add(1, Ordering::SeqCst)
    }

    /// Make a raw RPC call
    pub async fn call(&self, method: &str, params: Option<Value>) -> Result<Value> {
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: json!(self.next_id()),
        };

        let response = self.send_request(&request).await?;
        
        if let Some(error) = response.error {
            return Err(RpcClientError::ServerError {
                code: error.code,
                message: error.message,
            });
        }

        response.result.ok_or_else(|| {
            RpcClientError::InvalidResponse("No result or error in response".to_string())
        })
    }

    /// Send a request and handle retries
    async fn send_request(&self, request: &RpcRequest) -> Result<RpcResponse> {
        let mut last_error = None;
        
        for attempt in 0..=self.config.max_retries {
            match self.send_request_once(request).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.config.max_retries {
                        tokio::time::sleep(self.config.retry_delay).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap())
    }

    /// Send a single request (mock implementation for now)
    async fn send_request_once(&self, request: &RpcRequest) -> Result<RpcResponse> {
        // Mock implementation - in a real client this would use HTTP/WebSocket
        tokio::time::sleep(Duration::from_millis(10)).await; // Simulate network delay
        
        // Simulate successful responses for known methods
        let result = match request.method.as_str() {
            "cc_ping" => Some(json!("pong")),
            "cc_getVersion" => Some(json!({
                "version": "1.0.0",
                "build": "cc-chain-1.0.0",
                "commit": "abc123def"
            })),
            "cc_getLatestBlock" => Some(json!({
                "height": 12345,
                "hash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
                "parent_hash": "0x0987654321fedcba0987654321fedcba0987654321fedcba0987654321fedcba",
                "timestamp": 1640000000,
                "transaction_count": 42,
                "size": 2048,
                "validator": "validator_0"
            })),
            "cc_getNetworkInfo" => Some(json!({
                "chain_id": "cc-chain-1",
                "network_name": "CC Chain Mainnet",
                "latest_block_height": 12345,
                "latest_block_hash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
                "peer_count": 23,
                "is_syncing": false,
                "sync_progress": null
            })),
            "cc_getBalance" => {
                if let Some(params) = &request.params {
                    if let Some(address) = params.get("address").and_then(|v| v.as_str()) {
                        if !address.is_empty() {
                            Some(json!("5000000000"))
                        } else {
                            return Ok(RpcResponse {
                                jsonrpc: "2.0".to_string(),
                                result: None,
                                error: Some(RpcError {
                                    code: -32602,
                                    message: "Invalid parameters: address cannot be empty".to_string(),
                                    data: None,
                                }),
                                id: request.id.clone(),
                            });
                        }
                    } else {
                        return Ok(RpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: None,
                            error: Some(RpcError {
                                code: -32602,
                                message: "Invalid parameters".to_string(),
                                data: None,
                            }),
                            id: request.id.clone(),
                        });
                    }
                } else {
                    return Ok(RpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: Some(RpcError {
                            code: -32602,
                            message: "Invalid parameters".to_string(),
                            data: None,
                        }),
                        id: request.id.clone(),
                    });
                }
            },
            _ => return Ok(RpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(RpcError {
                    code: -32601,
                    message: format!("Method '{}' not found", request.method),
                    data: None,
                }),
                id: request.id.clone(),
            }),
        };

        Ok(RpcResponse {
            jsonrpc: "2.0".to_string(),
            result,
            error: None,
            id: request.id.clone(),
        })
    }

    // High-level API methods

    /// Ping the server
    pub async fn ping(&self) -> Result<String> {
        let result = self.call("cc_ping", None).await?;
        result.as_str()
            .ok_or_else(|| RpcClientError::InvalidResponse("Expected string response".to_string()))
            .map(|s| s.to_string())
    }

    /// Get server version
    pub async fn get_version(&self) -> Result<Value> {
        self.call("cc_getVersion", None).await
    }

    /// Get latest block information
    pub async fn get_latest_block(&self) -> Result<BlockInfo> {
        let result = self.call("cc_getLatestBlock", None).await?;
        serde_json::from_value(result)
            .map_err(|e| RpcClientError::SerializationError(e.to_string()))
    }

    /// Get block by height
    pub async fn get_block_by_height(&self, height: u64) -> Result<BlockInfo> {
        let params = json!({"height": height});
        let result = self.call("cc_getBlockByHeight", Some(params)).await?;
        serde_json::from_value(result)
            .map_err(|e| RpcClientError::SerializationError(e.to_string()))
    }

    /// Get block by hash
    pub async fn get_block_by_hash(&self, hash: &str) -> Result<BlockInfo> {
        let params = json!({"hash": hash});
        let result = self.call("cc_getBlockByHash", Some(params)).await?;
        serde_json::from_value(result)
            .map_err(|e| RpcClientError::SerializationError(e.to_string()))
    }

    /// Get transaction information
    pub async fn get_transaction(&self, hash: &str) -> Result<TransactionInfo> {
        let params = json!({"hash": hash});
        let result = self.call("cc_getTransaction", Some(params)).await?;
        serde_json::from_value(result)
            .map_err(|e| RpcClientError::SerializationError(e.to_string()))
    }

    /// Get account information
    pub async fn get_account(&self, address: &str) -> Result<AccountInfo> {
        let params = json!({"address": address});
        let result = self.call("cc_getAccount", Some(params)).await?;
        serde_json::from_value(result)
            .map_err(|e| RpcClientError::SerializationError(e.to_string()))
    }

    /// Get account balance
    pub async fn get_balance(&self, address: &str) -> Result<u64> {
        let params = json!({"address": address});
        let result = self.call("cc_getBalance", Some(params)).await?;
        result.as_str()
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| RpcClientError::InvalidResponse("Invalid balance format".to_string()))
    }

    /// Get network information
    pub async fn get_network_info(&self) -> Result<NetworkInfo> {
        let result = self.call("cc_getNetworkInfo", None).await?;
        serde_json::from_value(result)
            .map_err(|e| RpcClientError::SerializationError(e.to_string()))
    }

    /// Get peer count
    pub async fn get_peer_count(&self) -> Result<u32> {
        let result = self.call("cc_getPeerCount", None).await?;
        result.as_u64()
            .map(|n| n as u32)
            .ok_or_else(|| RpcClientError::InvalidResponse("Invalid peer count format".to_string()))
    }

    /// Send a transaction
    pub async fn send_transaction(&self, transaction: Value) -> Result<String> {
        let params = json!({"transaction": transaction});
        let result = self.call("cc_sendTransaction", Some(params)).await?;
        result.as_str()
            .ok_or_else(|| RpcClientError::InvalidResponse("Expected transaction hash".to_string()))
            .map(|s| s.to_string())
    }

    /// Estimate gas for a transaction
    pub async fn estimate_gas(&self, transaction: Value) -> Result<u64> {
        let params = json!({"transaction": transaction});
        let result = self.call("cc_estimateGas", Some(params)).await?;
        result.as_str()
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| RpcClientError::InvalidResponse("Invalid gas estimate format".to_string()))
    }

    /// Get transaction count for an address
    pub async fn get_transaction_count(&self, address: &str) -> Result<u64> {
        let params = json!({"address": address});
        let result = self.call("cc_getTransactionCount", Some(params)).await?;
        result.as_u64()
            .ok_or_else(|| RpcClientError::InvalidResponse("Invalid transaction count format".to_string()))
    }
}

impl Default for RpcClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rpc_client_creation() {
        let client = RpcClient::new();
        assert_eq!(client.config.endpoint, "http://localhost:8545");
    }

    #[tokio::test]
    async fn test_rpc_client_with_endpoint() {
        let client = RpcClient::with_endpoint("http://example.com:8545");
        assert_eq!(client.config.endpoint, "http://example.com:8545");
    }

    #[tokio::test]
    async fn test_ping() {
        let client = RpcClient::new();
        let result = client.ping().await.unwrap();
        assert_eq!(result, "pong");
    }

    #[tokio::test]
    async fn test_get_version() {
        let client = RpcClient::new();
        let result = client.get_version().await.unwrap();
        assert!(result.get("version").is_some());
    }

    #[tokio::test]
    async fn test_get_latest_block() {
        let client = RpcClient::new();
        let block = client.get_latest_block().await.unwrap();
        assert_eq!(block.height, 12345);
        assert!(!block.hash.is_empty());
    }

    #[tokio::test]
    async fn test_get_balance() {
        let client = RpcClient::new();
        let balance = client.get_balance("0x123456789abcdef").await.unwrap();
        assert_eq!(balance, 5000000000);
    }

    #[tokio::test]
    async fn test_get_network_info() {
        let client = RpcClient::new();
        let info = client.get_network_info().await.unwrap();
        assert_eq!(info.chain_id, "cc-chain-1");
        assert_eq!(info.peer_count, 23);
    }

    #[tokio::test]
    async fn test_id_counter() {
        let client = RpcClient::new();
        let id1 = client.next_id();
        let id2 = client.next_id();
        assert!(id2 > id1);
    }

    #[tokio::test]
    async fn test_invalid_method() {
        let client = RpcClient::new();
        let result = client.call("invalid_method", None).await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            RpcClientError::ServerError { code, .. } => {
                assert_eq!(code, -32601);
            }
            _ => panic!("Expected ServerError"),
        }
    }

    #[tokio::test]
    async fn test_invalid_parameters() {
        let client = RpcClient::new();
        let result = client.get_balance("").await;
        // This should fail due to empty address in our mock implementation
        assert!(result.is_err());
    }
}
