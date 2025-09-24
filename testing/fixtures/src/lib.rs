//! CC Chain Testing Fixtures
//!
//! This crate provides pre-defined test data fixtures for CC Chain components.
//! Fixtures include sample blocks, transactions, network messages, and other
//! test data that can be reused across different test suites.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FixtureError {
    #[error("Fixture loading error: {0}")]
    Loading(String),
    #[error("Invalid fixture data: {0}")]
    InvalidData(String),
}

pub type Result<T> = std::result::Result<T, FixtureError>;

/// Block fixture data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockFixture {
    pub height: u64,
    pub hash: String,
    pub parent_hash: String,
    pub timestamp: u64,
    pub transactions: Vec<TransactionFixture>,
    pub validator_signatures: Vec<String>,
    pub merkle_root: String,
}

/// Transaction fixture data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionFixture {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub gas_price: u64,
    pub gas_limit: u64,
    pub nonce: u64,
    pub signature: String,
}

/// Consensus message fixtures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusMessageFixture {
    pub message_type: String,
    pub round: u64,
    pub height: u64,
    pub validator_id: String,
    pub timestamp: u64,
    pub signature: String,
    pub data: String,
}

/// Network message fixtures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMessageFixture {
    pub message_id: String,
    pub peer_id: String,
    pub message_type: String,
    pub payload: Vec<u8>,
    pub timestamp: u64,
}

/// Account/Wallet fixtures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountFixture {
    pub address: String,
    pub private_key: String,
    pub public_key: String,
    pub balance: u64,
    pub nonce: u64,
}

/// Smart contract fixtures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractFixture {
    pub address: String,
    pub code: Vec<u8>,
    pub abi: String,
    pub storage: HashMap<String, String>,
    pub creator: String,
}

/// Main fixture provider
pub struct FixtureProvider {
    blocks: HashMap<String, BlockFixture>,
    transactions: HashMap<String, TransactionFixture>,
    accounts: HashMap<String, AccountFixture>,
    consensus_messages: HashMap<String, ConsensusMessageFixture>,
    network_messages: HashMap<String, NetworkMessageFixture>,
    contracts: HashMap<String, ContractFixture>,
}

impl FixtureProvider {
    /// Create a new fixture provider with default test data
    pub fn new() -> Self {
        let mut provider = FixtureProvider {
            blocks: HashMap::new(),
            transactions: HashMap::new(),
            accounts: HashMap::new(),
            consensus_messages: HashMap::new(),
            network_messages: HashMap::new(),
            contracts: HashMap::new(),
        };
        
        provider.load_default_fixtures();
        provider
    }
    
    /// Load default fixtures into the provider
    fn load_default_fixtures(&mut self) {
        // Load default accounts
        self.accounts.insert("alice".to_string(), AccountFixture {
            address: "cc1alice123456789abcdef0123456789abcdef01".to_string(),
            private_key: "alice_private_key_0123456789abcdef".to_string(),
            public_key: "alice_public_key_fedcba9876543210".to_string(),
            balance: 1000000,
            nonce: 0,
        });
        
        self.accounts.insert("bob".to_string(), AccountFixture {
            address: "cc1bob123456789abcdef0123456789abcdef012".to_string(),
            private_key: "bob_private_key_0123456789abcdef0".to_string(),
            public_key: "bob_public_key_fedcba98765432100".to_string(),
            balance: 500000,
            nonce: 0,
        });
        
        // Load default transactions
        self.transactions.insert("tx1".to_string(), TransactionFixture {
            hash: "tx1_hash_0123456789abcdef0123456789abcdef01234567".to_string(),
            from: "cc1alice123456789abcdef0123456789abcdef01".to_string(),
            to: "cc1bob123456789abcdef0123456789abcdef012".to_string(),
            amount: 100000,
            gas_price: 20,
            gas_limit: 21000,
            nonce: 1,
            signature: "tx1_signature_0123456789abcdef".to_string(),
        });
        
        // Load default blocks
        self.blocks.insert("genesis".to_string(), BlockFixture {
            height: 0,
            hash: "genesis_hash_0123456789abcdef0123456789abcdef".to_string(),
            parent_hash: "0000000000000000000000000000000000000000".to_string(),
            timestamp: 1640995200, // 2022-01-01 00:00:00 UTC
            transactions: vec![],
            validator_signatures: vec![],
            merkle_root: "genesis_merkle_root_0123456789abcdef".to_string(),
        });
        
        self.blocks.insert("block1".to_string(), BlockFixture {
            height: 1,
            hash: "block1_hash_0123456789abcdef0123456789abcdef".to_string(),
            parent_hash: "genesis_hash_0123456789abcdef0123456789abcdef".to_string(),
            timestamp: 1640995260, // 1 minute later
            transactions: vec![self.transactions["tx1"].clone()],
            validator_signatures: vec!["validator1_sig".to_string()],
            merkle_root: "block1_merkle_root_0123456789abcdef".to_string(),
        });
        
        // Load consensus message fixtures
        self.consensus_messages.insert("prepare1".to_string(), ConsensusMessageFixture {
            message_type: "PREPARE".to_string(),
            round: 1,
            height: 1,
            validator_id: "validator1".to_string(),
            timestamp: 1640995260,
            signature: "prepare_sig_0123456789abcdef".to_string(),
            data: "block1_hash_0123456789abcdef0123456789abcdef".to_string(),
        });
        
        // Load network message fixtures
        self.network_messages.insert("ping1".to_string(), NetworkMessageFixture {
            message_id: "msg_ping_001".to_string(),
            peer_id: "peer_001".to_string(),
            message_type: "PING".to_string(),
            payload: b"ping_data".to_vec(),
            timestamp: 1640995260,
        });
        
        // Load contract fixtures
        self.contracts.insert("token_contract".to_string(), ContractFixture {
            address: "cc1contract123456789abcdef0123456789abcdef".to_string(),
            code: b"contract_bytecode_placeholder".to_vec(),
            abi: r#"[{"name":"transfer","inputs":[{"name":"to","type":"address"},{"name":"amount","type":"uint256"}]}]"#.to_string(),
            storage: [("total_supply".to_string(), "1000000".to_string())].into(),
            creator: "cc1alice123456789abcdef0123456789abcdef01".to_string(),
        });
    }
    
    /// Get a block fixture by name
    pub fn block(&self, name: &str) -> Result<&BlockFixture> {
        self.blocks.get(name)
            .ok_or_else(|| FixtureError::Loading(format!("Block fixture '{}' not found", name)))
    }
    
    /// Get a transaction fixture by name
    pub fn transaction(&self, name: &str) -> Result<&TransactionFixture> {
        self.transactions.get(name)
            .ok_or_else(|| FixtureError::Loading(format!("Transaction fixture '{}' not found", name)))
    }
    
    /// Get an account fixture by name
    pub fn account(&self, name: &str) -> Result<&AccountFixture> {
        self.accounts.get(name)
            .ok_or_else(|| FixtureError::Loading(format!("Account fixture '{}' not found", name)))
    }
    
    /// Get a consensus message fixture by name
    pub fn consensus_message(&self, name: &str) -> Result<&ConsensusMessageFixture> {
        self.consensus_messages.get(name)
            .ok_or_else(|| FixtureError::Loading(format!("Consensus message fixture '{}' not found", name)))
    }
    
    /// Get a network message fixture by name
    pub fn network_message(&self, name: &str) -> Result<&NetworkMessageFixture> {
        self.network_messages.get(name)
            .ok_or_else(|| FixtureError::Loading(format!("Network message fixture '{}' not found", name)))
    }
    
    /// Get a contract fixture by name
    pub fn contract(&self, name: &str) -> Result<&ContractFixture> {
        self.contracts.get(name)
            .ok_or_else(|| FixtureError::Loading(format!("Contract fixture '{}' not found", name)))
    }
    
    /// Get all available fixture names by category
    pub fn available_fixtures(&self) -> FixtureInventory {
        FixtureInventory {
            blocks: self.blocks.keys().cloned().collect(),
            transactions: self.transactions.keys().cloned().collect(),
            accounts: self.accounts.keys().cloned().collect(),
            consensus_messages: self.consensus_messages.keys().cloned().collect(),
            network_messages: self.network_messages.keys().cloned().collect(),
            contracts: self.contracts.keys().cloned().collect(),
        }
    }
    
    /// Create a chain of connected blocks for testing
    pub fn create_block_chain(&self, length: usize) -> Result<Vec<BlockFixture>> {
        let mut chain = vec![];
        let genesis = self.block("genesis")?.clone();
        chain.push(genesis);
        
        for i in 1..length {
            let parent_hash = chain[i-1].hash.clone();
            let block = BlockFixture {
                height: i as u64,
                hash: format!("block{}_hash_0123456789abcdef0123456789abcdef", i),
                parent_hash,
                timestamp: 1640995200 + (i as u64 * 60), // 1 minute intervals
                transactions: vec![],
                validator_signatures: vec![format!("validator_sig_{}", i)],
                merkle_root: format!("merkle_root_{}_0123456789abcdef", i),
            };
            chain.push(block);
        }
        
        Ok(chain)
    }
}

/// Inventory of available fixtures
#[derive(Debug)]
pub struct FixtureInventory {
    pub blocks: Vec<String>,
    pub transactions: Vec<String>,
    pub accounts: Vec<String>,
    pub consensus_messages: Vec<String>,
    pub network_messages: Vec<String>,
    pub contracts: Vec<String>,
}

impl Default for FixtureProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience functions for common fixtures
pub mod common {
    use super::*;
    
    lazy_static::lazy_static! {
        static ref PROVIDER: FixtureProvider = FixtureProvider::new();
    }
    
    /// Get Alice's account fixture
    pub fn alice() -> &'static AccountFixture {
        PROVIDER.account("alice").unwrap()
    }
    
    /// Get Bob's account fixture
    pub fn bob() -> &'static AccountFixture {
        PROVIDER.account("bob").unwrap()
    }
    
    /// Get genesis block fixture
    pub fn genesis_block() -> &'static BlockFixture {
        PROVIDER.block("genesis").unwrap()
    }
    
    /// Get first block fixture
    pub fn block1() -> &'static BlockFixture {
        PROVIDER.block("block1").unwrap()
    }
    
    /// Get sample transaction fixture
    pub fn sample_transaction() -> &'static TransactionFixture {
        PROVIDER.transaction("tx1").unwrap()
    }
    
    /// Get token contract fixture
    pub fn token_contract() -> &'static ContractFixture {
        PROVIDER.contract("token_contract").unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fixture_provider_creation() {
        let provider = FixtureProvider::new();
        let inventory = provider.available_fixtures();
        
        assert!(!inventory.blocks.is_empty());
        assert!(!inventory.accounts.is_empty());
        assert!(!inventory.transactions.is_empty());
    }
    
    #[test]
    fn test_account_fixtures() {
        let provider = FixtureProvider::new();
        
        let alice = provider.account("alice").unwrap();
        assert!(alice.address.contains("alice"));
        assert_eq!(alice.balance, 1000000);
        
        let bob = provider.account("bob").unwrap();
        assert!(bob.address.contains("bob"));
        assert_eq!(bob.balance, 500000);
    }
    
    #[test]
    fn test_block_fixtures() {
        let provider = FixtureProvider::new();
        
        let genesis = provider.block("genesis").unwrap();
        assert_eq!(genesis.height, 0);
        assert!(genesis.transactions.is_empty());
        
        let block1 = provider.block("block1").unwrap();
        assert_eq!(block1.height, 1);
        assert_eq!(block1.parent_hash, genesis.hash);
        assert_eq!(block1.transactions.len(), 1);
    }
    
    #[test]
    fn test_transaction_fixtures() {
        let provider = FixtureProvider::new();
        let tx = provider.transaction("tx1").unwrap();
        
        assert!(tx.hash.contains("tx1"));
        assert_eq!(tx.amount, 100000);
        assert!(tx.from.contains("alice"));
        assert!(tx.to.contains("bob"));
    }
    
    #[test]
    fn test_consensus_message_fixtures() {
        let provider = FixtureProvider::new();
        let msg = provider.consensus_message("prepare1").unwrap();
        
        assert_eq!(msg.message_type, "PREPARE");
        assert_eq!(msg.round, 1);
        assert_eq!(msg.height, 1);
    }
    
    #[test]
    fn test_contract_fixtures() {
        let provider = FixtureProvider::new();
        let contract = provider.contract("token_contract").unwrap();
        
        assert!(contract.address.contains("contract"));
        assert!(!contract.code.is_empty());
        assert!(contract.abi.contains("transfer"));
    }
    
    #[test]
    fn test_block_chain_creation() {
        let provider = FixtureProvider::new();
        let chain = provider.create_block_chain(5).unwrap();
        
        assert_eq!(chain.len(), 5);
        assert_eq!(chain[0].height, 0); // Genesis
        assert_eq!(chain[4].height, 4); // Last block
        
        // Verify chain integrity
        for i in 1..chain.len() {
            assert_eq!(chain[i].parent_hash, chain[i-1].hash);
        }
    }
    
    #[test]
    fn test_common_fixtures() {
        let alice = common::alice();
        assert!(alice.address.contains("alice"));
        
        let _bob = common::bob();
        assert!(alice.address.contains("alice"));
        
        let genesis = common::genesis_block();
        assert_eq!(genesis.height, 0);
        
        let tx = common::sample_transaction();
        assert!(tx.hash.contains("tx1"));
    }
    
    #[test]
    fn test_fixture_error_handling() {
        let provider = FixtureProvider::new();
        
        let result = provider.account("nonexistent");
        assert!(result.is_err());
        
        let result = provider.block("nonexistent");
        assert!(result.is_err());
    }
}
