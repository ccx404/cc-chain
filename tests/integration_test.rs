//! Integration tests for the enhanced CLI, API, and Bridge modules

use api::{ApiServer, NodeApi, models::*};
use bridge::{CrossChainBridge, SupportedChain, BridgeConfig};
use cli::node::{CCNode, NodeConfig, NodeType};
use std::sync::Arc;
use std::collections::HashMap;

/// Mock node implementation for testing API integration
pub struct MockNode {
    height: u64,
    balances: HashMap<String, u64>,
}

impl MockNode {
    pub fn new() -> Self {
        let mut balances = HashMap::new();
        balances.insert("test_address".to_string(), 1000000);
        
        Self {
            height: 100,
            balances,
        }
    }
}

impl NodeApi for MockNode {
    fn get_height(&self) -> u64 {
        self.height
    }
    
    fn get_balance(&self, address: &str) -> Result<u64, api::ApiError> {
        Ok(self.balances.get(address).copied().unwrap_or(0))
    }
    
    fn submit_transaction(&self, _tx_data: TransactionRequest) -> Result<String, api::ApiError> {
        Ok("test_tx_hash".to_string())
    }
    
    fn get_block(&self, height: u64) -> Result<Option<BlockResponse>, api::ApiError> {
        if height <= self.height {
            Ok(Some(BlockResponse {
                hash: format!("block_hash_{}", height),
                height,
                parent_hash: format!("parent_hash_{}", height.saturating_sub(1)),
                timestamp: chrono::Utc::now(),
                proposer: "test_proposer".to_string(),
                transactions_root: "test_tx_root".to_string(),
                state_root: "test_state_root".to_string(),
                transactions: vec!["test_tx_1".to_string(), "test_tx_2".to_string()],
                transaction_count: 2,
                size: 1024,
                gas_limit: 10_000_000,
                gas_used: 5_000_000,
            }))
        } else {
            Ok(None)
        }
    }
    
    fn get_transaction(&self, _hash: &str) -> Result<Option<TransactionResponse>, api::ApiError> {
        Ok(Some(TransactionResponse {
            hash: "test_tx_hash".to_string(),
            block_height: Some(self.height),
            block_hash: Some(format!("block_hash_{}", self.height)),
            transaction_index: Some(0),
            from: "sender_address".to_string(),
            to: "recipient_address".to_string(),
            amount: 1000,
            fee: 100,
            data: None,
            status: TransactionStatus::Confirmed,
            gas_used: Some(21000),
            timestamp: chrono::Utc::now(),
        }))
    }
    
    fn get_chain_info(&self) -> Result<ChainInfo, api::ApiError> {
        Ok(ChainInfo {
            chain_id: "cc-chain-testnet".to_string(),
            name: "CC Chain Testnet".to_string(),
            height: self.height,
            latest_block_hash: format!("block_hash_{}", self.height),
            genesis_hash: "genesis_hash".to_string(),
            avg_block_time: 2.0,
            total_transactions: 1000,
            version: "1.0.0".to_string(),
        })
    }
    
    fn get_mempool_status(&self) -> Result<MempoolStatus, api::ApiError> {
        Ok(MempoolStatus {
            pending_count: 10,
            pending_size: 10240,
            max_size: 100_000_000,
            min_fee_rate: 1000,
        })
    }
    
    fn get_peers(&self) -> Result<Vec<PeerInfo>, api::ApiError> {
        Ok(vec![
            PeerInfo {
                id: "peer_1".to_string(),
                address: "127.0.0.1:8001".to_string(),
                direction: "outbound".to_string(),
                version: "1.0.0".to_string(),
                uptime: 3600,
                last_seen: chrono::Utc::now(),
            }
        ])
    }
}

#[tokio::test]
async fn test_api_server_creation() {
    let mock_node = Arc::new(MockNode::new());
    let _api_server = ApiServer::new(mock_node);
    // Test that API server can be created successfully
}

#[tokio::test]
async fn test_bridge_initialization() {
    let mut bridge = CrossChainBridge::new();
    
    // Add bridge configuration
    let config = BridgeConfig {
        source_chain: SupportedChain::CcChain,
        destination_chain: SupportedChain::Ethereum,
        min_validators: 3,
        confirmation_blocks: 12,
        max_transfer_amount: 1_000_000,
        daily_limit: 10_000_000,
        fee_rate: 30, // 0.3%
        paused: false,
    };
    
    assert!(bridge.add_bridge_config(config).is_ok());
    
    // Test bridge statistics
    let stats = bridge.get_bridge_stats();
    assert_eq!(stats.total_transfers, 0);
    assert_eq!(stats.active_validators, 0);
    assert_eq!(stats.supported_chains, 0);
}

#[tokio::test]
async fn test_node_configuration() {
    use std::net::SocketAddr;
    use std::path::PathBuf;
    
    let config = NodeConfig {
        node_type: NodeType::LightCompute,
        listen_addr: "127.0.0.1:8000".parse::<SocketAddr>().unwrap(),
        validator_keypair: None,
        bootstrap_peers: vec![],
        data_dir: "./test_data".to_string(),
        max_mempool_size: 10000,
        enable_metrics: true,
    };
    
    // Test that node configuration can be created
    assert_eq!(config.max_mempool_size, 10000);
    assert!(config.enable_metrics);
}

#[test]
fn test_bridge_transfer_initiation() {
    let mut bridge = CrossChainBridge::new();
    
    // Add bridge configuration first
    let config = BridgeConfig {
        source_chain: SupportedChain::CcChain,
        destination_chain: SupportedChain::Ethereum,
        min_validators: 1, // Lower for testing
        confirmation_blocks: 1,
        max_transfer_amount: 1_000_000,
        daily_limit: 10_000_000,
        fee_rate: 30,
        paused: false,
    };
    
    bridge.add_bridge_config(config).unwrap();
    
    // Add a test validator
    use bridge::BridgeValidator;
    let validator = BridgeValidator::new(
        "test_validator".to_string(),
        "test_pubkey".to_string(),
        "127.0.0.1:9000".to_string(),
        1000,
    );
    bridge.add_validator(validator).unwrap();
    
    // Add a test asset
    use bridge::CrossChainAsset;
    let mut contract_addresses = HashMap::new();
    contract_addresses.insert(SupportedChain::CcChain, "native".to_string());
    contract_addresses.insert(SupportedChain::Ethereum, "0x123...".to_string());
    
    let asset = CrossChainAsset {
        symbol: "CC".to_string(),
        name: "CC Token".to_string(),
        native_chain: SupportedChain::CcChain,
        contract_addresses,
        decimals: 18,
        min_transfer_amount: 1000,
        max_transfer_amount: 1_000_000,
    };
    bridge.add_asset(asset);
    
    // Test transfer initiation
    let result = bridge.initiate_transfer(
        SupportedChain::CcChain,
        SupportedChain::Ethereum,
        "CC".to_string(),
        10000,
        "cc_sender".to_string(),
        "eth_recipient".to_string(),
    );
    
    assert!(result.is_ok());
    let transfer_id = result.unwrap();
    assert!(!transfer_id.is_empty());
    
    // Verify transfer was created
    let transfer = bridge.get_transfer(&transfer_id);
    assert!(transfer.is_some());
    
    let transfer = transfer.unwrap();
    assert_eq!(transfer.amount, 10000);
    assert_eq!(transfer.asset_symbol, "CC");
}