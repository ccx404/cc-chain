//! CC Chain REST API Server
//! 
//! This module provides a comprehensive REST API for interacting with the CC Chain blockchain.
//! It includes endpoints for transactions, blocks, accounts, and network information.

pub mod server;
pub mod models;
pub mod error;

// Re-export important types
pub use server::{ApiServer, NodeApi};
pub use models::*;
pub use error::ApiError;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Mock node implementation for testing
#[cfg(test)]
pub struct MockNode {
    height: u64,
    balances: std::collections::HashMap<String, u64>,
}

#[cfg(test)]
impl MockNode {
    pub fn new() -> Self {
        let mut balances = std::collections::HashMap::new();
        balances.insert("test_address_1".to_string(), 1000000);
        balances.insert("test_address_2".to_string(), 500000);
        
        Self {
            height: 100,
            balances,
        }
    }
}

#[cfg(test)]
impl NodeApi for MockNode {
    fn get_height(&self) -> u64 {
        self.height
    }
    
    fn get_balance(&self, address: &str) -> Result<u64, ApiError> {
        Ok(self.balances.get(address).copied().unwrap_or(0))
    }
    
    fn submit_transaction(&self, _tx_data: TransactionRequest) -> Result<String, ApiError> {
        Ok("0x1234567890abcdef".to_string())
    }
    
    fn get_block(&self, height: u64) -> Result<Option<BlockResponse>, ApiError> {
        if height > self.height {
            return Ok(None);
        }
        
        Ok(Some(BlockResponse {
            hash: format!("0x{:064x}", height),
            height,
            parent_hash: format!("0x{:064x}", height.saturating_sub(1)),
            timestamp: chrono::Utc::now(),
            proposer: "validator_1".to_string(),
            transactions_root: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            state_root: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            transactions: vec![],
            transaction_count: 0,
            size: 1024,
            gas_limit: 10000000,
            gas_used: 0,
        }))
    }
    
    fn get_transaction(&self, hash: &str) -> Result<Option<TransactionResponse>, ApiError> {
        if hash == "0x1234567890abcdef" {
            Ok(Some(TransactionResponse {
                hash: hash.to_string(),
                block_height: Some(self.height),
                block_hash: Some(format!("0x{:064x}", self.height)),
                transaction_index: Some(0),
                from: "test_address_1".to_string(),
                to: "test_address_2".to_string(),
                amount: 1000,
                fee: 100,
                data: None,
                status: TransactionStatus::Confirmed,
                gas_used: Some(21000),
                timestamp: chrono::Utc::now(),
            }))
        } else {
            Ok(None)
        }
    }
    
    fn get_chain_info(&self) -> Result<ChainInfo, ApiError> {
        Ok(ChainInfo {
            chain_id: "cc-chain-testnet".to_string(),
            name: "CC Chain Testnet".to_string(),
            height: self.height,
            latest_block_hash: format!("0x{:064x}", self.height),
            genesis_hash: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            avg_block_time: 2.5,
            total_transactions: 12345,
            version: "1.0.0".to_string(),
        })
    }
    
    fn get_mempool_status(&self) -> Result<MempoolStatus, ApiError> {
        Ok(MempoolStatus {
            pending_count: 5,
            pending_size: 2048,
            max_size: 1000000,
            min_fee_rate: 100,
        })
    }
    
    fn get_peers(&self) -> Result<Vec<PeerInfo>, ApiError> {
        Ok(vec![
            PeerInfo {
                id: "peer_1".to_string(),
                address: "192.168.1.100:8001".to_string(),
                version: "1.0.0".to_string(),
                uptime: 7200, // 2 hours
                last_seen: chrono::Utc::now(),
                direction: "outbound".to_string(),
            },
            PeerInfo {
                id: "peer_2".to_string(),
                address: "192.168.1.101:8001".to_string(),
                version: "1.0.0".to_string(),
                uptime: 3600, // 1 hour
                last_seen: chrono::Utc::now(),
                direction: "inbound".to_string(),
            },
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }
    
    #[test]
    fn test_mock_node_height() {
        let node = MockNode::new();
        assert_eq!(node.get_height(), 100);
    }
    
    #[test]
    fn test_mock_node_balance() {
        let node = MockNode::new();
        assert_eq!(node.get_balance("test_address_1").unwrap(), 1000000);
        assert_eq!(node.get_balance("unknown_address").unwrap(), 0);
    }
    
    #[test]
    fn test_mock_node_submit_transaction() {
        let node = MockNode::new();
        let tx_request = TransactionRequest {
            from: "test_address_1".to_string(),
            to: "test_address_2".to_string(),
            amount: 1000,
            fee: 100,
            data: None,
            signature: "signature".to_string(),
        };
        
        let result = node.submit_transaction(tx_request);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "0x1234567890abcdef");
    }
    
    #[test]
    fn test_mock_node_get_block() {
        let node = MockNode::new();
        
        // Test existing block
        let block = node.get_block(50).unwrap();
        assert!(block.is_some());
        assert_eq!(block.unwrap().height, 50);
        
        // Test non-existing block
        let block = node.get_block(200).unwrap();
        assert!(block.is_none());
    }
    
    #[test]
    fn test_api_server_creation() {
        let node = Arc::new(MockNode::new());
        let _server = ApiServer::new(node);
        // If we reach here without panic, the server was created successfully
    }
}
