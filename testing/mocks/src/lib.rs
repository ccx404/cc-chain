//! CC Chain Testing Mocks
//!
//! This crate provides mock implementations of CC Chain components for testing.
//! Mocks allow for controlled testing environments where external dependencies
//! and complex interactions can be simulated predictably.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MockError {
    #[error("Mock configuration error: {0}")]
    Configuration(String),
    #[error("Mock state error: {0}")]
    State(String),
    #[error("Mock verification failed: {0}")]
    Verification(String),
}

pub type Result<T> = std::result::Result<T, MockError>;

/// Mock blockchain state for testing
#[derive(Debug, Clone)]
pub struct MockBlockchain {
    blocks: Arc<Mutex<Vec<MockBlock>>>,
    transactions: Arc<Mutex<HashMap<String, MockTransaction>>>,
    accounts: Arc<Mutex<HashMap<String, MockAccount>>>,
    config: MockConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockConfig {
    pub auto_mine: bool,
    pub block_time: u64, // seconds
    pub initial_balance: u64,
    pub gas_limit: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockBlock {
    pub height: u64,
    pub hash: String,
    pub parent_hash: String,
    pub timestamp: u64,
    pub transactions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockTransaction {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub gas_price: u64,
    pub status: TransactionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockAccount {
    pub address: String,
    pub balance: u64,
    pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
}

impl MockBlockchain {
    /// Create a new mock blockchain with default configuration
    pub fn new() -> Self {
        let config = MockConfig {
            auto_mine: true,
            block_time: 10,
            initial_balance: 1000000,
            gas_limit: 21000,
        };
        
        Self::with_config(config)
    }
    
    /// Create a mock blockchain with custom configuration
    pub fn with_config(config: MockConfig) -> Self {
        let mut blockchain = MockBlockchain {
            blocks: Arc::new(Mutex::new(Vec::new())),
            transactions: Arc::new(Mutex::new(HashMap::new())),
            accounts: Arc::new(Mutex::new(HashMap::new())),
            config,
        };
        
        blockchain.initialize();
        blockchain
    }
    
    /// Initialize the mock blockchain with genesis block
    fn initialize(&mut self) {
        let genesis = MockBlock {
            height: 0,
            hash: "genesis_hash".to_string(),
            parent_hash: "0".to_string(),
            timestamp: 1640995200,
            transactions: vec![],
        };
        
        self.blocks.lock().unwrap().push(genesis);
    }
    
    /// Create a new mock account
    pub fn create_account(&self, address: &str) -> Result<()> {
        let account = MockAccount {
            address: address.to_string(),
            balance: self.config.initial_balance,
            nonce: 0,
        };
        
        self.accounts.lock().unwrap().insert(address.to_string(), account);
        Ok(())
    }
    
    /// Get account balance
    pub fn get_balance(&self, address: &str) -> Result<u64> {
        let accounts = self.accounts.lock().unwrap();
        let account = accounts.get(address)
            .ok_or_else(|| MockError::State(format!("Account {} not found", address)))?;
        Ok(account.balance)
    }
    
    /// Submit a transaction
    pub fn submit_transaction(&self, from: &str, to: &str, amount: u64) -> Result<String> {
        let tx_hash = format!("tx_{}_{}_{}_{}", from, to, amount, self.get_current_time());
        
        // Check sender balance
        {
            let mut accounts = self.accounts.lock().unwrap();
            let sender = accounts.get_mut(from)
                .ok_or_else(|| MockError::State(format!("Sender account {} not found", from)))?;
                
            if sender.balance < amount {
                return Err(MockError::State("Insufficient balance".to_string()));
            }
            
            sender.balance -= amount;
            sender.nonce += 1;
            
            // Credit recipient
            let recipient = accounts.get_mut(to)
                .ok_or_else(|| MockError::State(format!("Recipient account {} not found", to)))?;
            recipient.balance += amount;
        }
        
        let transaction = MockTransaction {
            hash: tx_hash.clone(),
            from: from.to_string(),
            to: to.to_string(),
            amount,
            gas_price: 20,
            status: if self.config.auto_mine {
                TransactionStatus::Confirmed
            } else {
                TransactionStatus::Pending
            },
        };
        
        self.transactions.lock().unwrap().insert(tx_hash.clone(), transaction);
        
        if self.config.auto_mine {
            self.mine_block(vec![tx_hash.clone()])?;
        }
        
        Ok(tx_hash)
    }
    
    /// Mine a new block with given transactions
    pub fn mine_block(&self, tx_hashes: Vec<String>) -> Result<String> {
        let mut blocks = self.blocks.lock().unwrap();
        let height = blocks.len() as u64;
        let parent_hash = blocks.last().unwrap().hash.clone();
        
        let block = MockBlock {
            height,
            hash: format!("block_hash_{}", height),
            parent_hash,
            timestamp: self.get_current_time(),
            transactions: tx_hashes,
        };
        
        let block_hash = block.hash.clone();
        blocks.push(block);
        
        Ok(block_hash)
    }
    
    /// Get transaction by hash
    pub fn get_transaction(&self, tx_hash: &str) -> Result<MockTransaction> {
        let transactions = self.transactions.lock().unwrap();
        let transaction = transactions.get(tx_hash)
            .ok_or_else(|| MockError::State(format!("Transaction {} not found", tx_hash)))?;
        Ok(transaction.clone())
    }
    
    /// Get block by height
    pub fn get_block(&self, height: u64) -> Result<MockBlock> {
        let blocks = self.blocks.lock().unwrap();
        let block = blocks.get(height as usize)
            .ok_or_else(|| MockError::State(format!("Block at height {} not found", height)))?;
        Ok(block.clone())
    }
    
    /// Get current block height
    pub fn get_height(&self) -> u64 {
        let blocks = self.blocks.lock().unwrap();
        blocks.len() as u64 - 1
    }
    
    fn get_current_time(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

/// Mock network for testing network components
#[derive(Debug)]
pub struct MockNetwork {
    peers: Arc<Mutex<HashMap<String, MockPeer>>>,
    messages: Arc<Mutex<Vec<MockMessage>>>,
    config: NetworkConfig,
}

#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub max_peers: usize,
    pub message_delay: u64, // milliseconds
    pub drop_rate: f64, // percentage of messages to drop
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockPeer {
    pub id: String,
    pub address: String,
    pub connected: bool,
    pub last_seen: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockMessage {
    pub id: String,
    pub from: String,
    pub to: String,
    pub message_type: String,
    pub payload: Vec<u8>,
    pub timestamp: u64,
}

impl MockNetwork {
    /// Create a new mock network
    pub fn new() -> Self {
        let config = NetworkConfig {
            max_peers: 100,
            message_delay: 100,
            drop_rate: 0.0,
        };
        
        MockNetwork {
            peers: Arc::new(Mutex::new(HashMap::new())),
            messages: Arc::new(Mutex::new(Vec::new())),
            config,
        }
    }
    
    /// Add a peer to the network
    pub fn add_peer(&self, id: &str, address: &str) -> Result<()> {
        let peer = MockPeer {
            id: id.to_string(),
            address: address.to_string(),
            connected: true,
            last_seen: self.get_current_time(),
        };
        
        self.peers.lock().unwrap().insert(id.to_string(), peer);
        Ok(())
    }
    
    /// Send a message between peers
    pub fn send_message(&self, from: &str, to: &str, message_type: &str, payload: Vec<u8>) -> Result<String> {
        // Check if peers exist
        let peers = self.peers.lock().unwrap();
        if !peers.contains_key(from) {
            return Err(MockError::State(format!("Sender peer {} not found", from)));
        }
        if !peers.contains_key(to) {
            return Err(MockError::State(format!("Recipient peer {} not found", to)));
        }
        drop(peers);
        
        let message_id = format!("msg_{}_{}_{}_{}", from, to, message_type, self.get_current_time());
        
        let message = MockMessage {
            id: message_id.clone(),
            from: from.to_string(),
            to: to.to_string(),
            message_type: message_type.to_string(),
            payload,
            timestamp: self.get_current_time(),
        };
        
        self.messages.lock().unwrap().push(message);
        Ok(message_id)
    }
    
    /// Get messages for a peer
    pub fn get_messages_for_peer(&self, peer_id: &str) -> Vec<MockMessage> {
        let messages = self.messages.lock().unwrap();
        messages.iter()
            .filter(|msg| msg.to == peer_id)
            .cloned()
            .collect()
    }
    
    /// Get all connected peers
    pub fn get_connected_peers(&self) -> Vec<MockPeer> {
        let peers = self.peers.lock().unwrap();
        peers.values()
            .filter(|peer| peer.connected)
            .cloned()
            .collect()
    }
    
    fn get_current_time(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

/// Mock consensus for testing consensus algorithms
#[derive(Debug)]
pub struct MockConsensus {
    validators: Arc<Mutex<HashMap<String, MockValidator>>>,
    rounds: Arc<Mutex<Vec<MockConsensusRound>>>,
    config: ConsensusConfig,
}

#[derive(Debug, Clone)]
pub struct ConsensusConfig {
    pub min_validators: usize,
    pub round_timeout: u64, // seconds
    pub byzantine_tolerance: f64, // fraction of validators that can be Byzantine
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockValidator {
    pub id: String,
    pub voting_power: u64,
    pub online: bool,
    pub byzantine: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockConsensusRound {
    pub height: u64,
    pub round: u64,
    pub proposed_block: Option<String>,
    pub votes: HashMap<String, Vote>,
    pub status: RoundStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub validator: String,
    pub vote_type: VoteType,
    pub block_hash: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VoteType {
    Prevote,
    Precommit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoundStatus {
    Proposing,
    Voting,
    Committed,
    Failed,
}

impl MockConsensus {
    /// Create a new mock consensus
    pub fn new() -> Self {
        let config = ConsensusConfig {
            min_validators: 3,
            round_timeout: 30,
            byzantine_tolerance: 0.33,
        };
        
        MockConsensus {
            validators: Arc::new(Mutex::new(HashMap::new())),
            rounds: Arc::new(Mutex::new(Vec::new())),
            config,
        }
    }
    
    /// Add a validator
    pub fn add_validator(&self, id: &str, voting_power: u64) -> Result<()> {
        let validator = MockValidator {
            id: id.to_string(),
            voting_power,
            online: true,
            byzantine: false,
        };
        
        self.validators.lock().unwrap().insert(id.to_string(), validator);
        Ok(())
    }
    
    /// Start a new consensus round
    pub fn start_round(&self, height: u64, round: u64, proposed_block: &str) -> Result<()> {
        let consensus_round = MockConsensusRound {
            height,
            round,
            proposed_block: Some(proposed_block.to_string()),
            votes: HashMap::new(),
            status: RoundStatus::Proposing,
        };
        
        self.rounds.lock().unwrap().push(consensus_round);
        Ok(())
    }
    
    /// Cast a vote in the current round
    pub fn vote(&self, validator_id: &str, vote_type: VoteType, block_hash: &str) -> Result<()> {
        let mut rounds = self.rounds.lock().unwrap();
        let current_round = rounds.last_mut()
            .ok_or_else(|| MockError::State("No active round".to_string()))?;
        
        let vote = Vote {
            validator: validator_id.to_string(),
            vote_type,
            block_hash: block_hash.to_string(),
            timestamp: self.get_current_time(),
        };
        
        current_round.votes.insert(validator_id.to_string(), vote);
        Ok(())
    }
    
    /// Check if consensus is reached
    pub fn check_consensus(&self) -> Result<bool> {
        let rounds = self.rounds.lock().unwrap();
        let current_round = rounds.last()
            .ok_or_else(|| MockError::State("No active round".to_string()))?;
        
        let validators = self.validators.lock().unwrap();
        let total_power: u64 = validators.values().map(|v| v.voting_power).sum();
        let required_power = (total_power * 2) / 3 + 1; // 2/3+ majority
        
        let voting_power: u64 = current_round.votes.values()
            .filter_map(|vote| validators.get(&vote.validator).map(|v| v.voting_power))
            .sum();
        
        Ok(voting_power >= required_power)
    }
    
    fn get_current_time(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

impl Default for MockBlockchain {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for MockNetwork {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for MockConsensus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mock_blockchain() {
        let blockchain = MockBlockchain::new();
        
        // Create accounts
        blockchain.create_account("alice").unwrap();
        blockchain.create_account("bob").unwrap();
        
        assert_eq!(blockchain.get_balance("alice").unwrap(), 1000000);
        assert_eq!(blockchain.get_balance("bob").unwrap(), 1000000);
        
        // Submit transaction
        let tx_hash = blockchain.submit_transaction("alice", "bob", 100000).unwrap();
        
        assert_eq!(blockchain.get_balance("alice").unwrap(), 900000);
        assert_eq!(blockchain.get_balance("bob").unwrap(), 1100000);
        
        let tx = blockchain.get_transaction(&tx_hash).unwrap();
        assert_eq!(tx.amount, 100000);
        assert_eq!(tx.from, "alice");
        assert_eq!(tx.to, "bob");
    }
    
    #[test]
    fn test_mock_network() {
        let network = MockNetwork::new();
        
        // Add peers
        network.add_peer("peer1", "127.0.0.1:8001").unwrap();
        network.add_peer("peer2", "127.0.0.1:8002").unwrap();
        
        let peers = network.get_connected_peers();
        assert_eq!(peers.len(), 2);
        
        // Send message
        let msg_id = network.send_message("peer1", "peer2", "PING", b"hello".to_vec()).unwrap();
        
        let messages = network.get_messages_for_peer("peer2");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id, msg_id);
        assert_eq!(messages[0].from, "peer1");
        assert_eq!(messages[0].payload, b"hello");
    }
    
    #[test]
    fn test_mock_consensus() {
        let consensus = MockConsensus::new();
        
        // Add validators
        consensus.add_validator("val1", 100).unwrap();
        consensus.add_validator("val2", 100).unwrap();
        consensus.add_validator("val3", 100).unwrap();
        consensus.add_validator("val4", 100).unwrap();
        
        // Start round
        consensus.start_round(1, 0, "block_hash_1").unwrap();
        
        // Cast votes (3 out of 4 validators)
        consensus.vote("val1", VoteType::Prevote, "block_hash_1").unwrap();
        consensus.vote("val2", VoteType::Prevote, "block_hash_1").unwrap();
        consensus.vote("val3", VoteType::Prevote, "block_hash_1").unwrap();
        
        // Should reach consensus with 3/4 validators (75% > 66%)
        assert!(consensus.check_consensus().unwrap());
    }
    
    #[test]
    fn test_insufficient_balance() {
        let blockchain = MockBlockchain::new();
        blockchain.create_account("poor_alice").unwrap();
        blockchain.create_account("bob").unwrap();
        
        // Try to send more than balance
        let result = blockchain.submit_transaction("poor_alice", "bob", 2000000);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_nonexistent_account() {
        let blockchain = MockBlockchain::new();
        let result = blockchain.get_balance("nonexistent");
        assert!(result.is_err());
    }
}
