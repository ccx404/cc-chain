//! Core blockchain functionality
//! 
//! This module provides fundamental blockchain data structures and operations
//! including Block, Transaction, Blockchain chain management, and validation logic.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Blockchain-related errors
#[derive(Error, Debug, Clone, PartialEq)]
pub enum BlockchainError {
    #[error("Invalid block hash")]
    InvalidBlockHash,
    #[error("Invalid block signature")]
    InvalidBlockSignature,
    #[error("Block validation failed: {0}")]
    BlockValidationFailed(String),
    #[error("Transaction validation failed: {0}")]
    TransactionValidationFailed(String),
    #[error("Chain validation failed: {0}")]
    ChainValidationFailed(String),
    #[error("Genesis block mismatch")]
    GenesisBlockMismatch,
    #[error("Block not found: {0}")]
    BlockNotFound(String),
}

pub type Result<T> = std::result::Result<T, BlockchainError>;

/// A 32-byte hash
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hash([u8; 32]);

impl Hash {
    /// Create a new hash
    pub fn new(data: [u8; 32]) -> Self {
        Hash(data)
    }

    /// Create hash from slice
    pub fn from_slice(data: &[u8]) -> Result<Self> {
        if data.len() != 32 {
            return Err(BlockchainError::InvalidBlockHash);
        }
        let mut hash = [0u8; 32];
        hash.copy_from_slice(data);
        Ok(Hash(hash))
    }

    /// Hash some data using SHA256
    pub fn hash(data: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        Hash(hasher.finalize().into())
    }

    /// Get the hash as bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Zero hash
    pub fn zero() -> Self {
        Hash([0u8; 32])
    }
}

impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

/// Transaction structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    /// Transaction ID (hash)
    pub id: Hash,
    /// Sender address
    pub from: String,
    /// Recipient address  
    pub to: String,
    /// Amount to transfer
    pub amount: u64,
    /// Transaction fee
    pub fee: u64,
    /// Transaction nonce
    pub nonce: u64,
    /// Transaction timestamp
    pub timestamp: u64,
    /// Transaction signature
    pub signature: Vec<u8>,
    /// Optional transaction data
    pub data: Vec<u8>,
}

impl Transaction {
    /// Create a new transaction
    pub fn new(from: String, to: String, amount: u64, fee: u64, nonce: u64, data: Vec<u8>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut tx = Transaction {
            id: Hash::zero(),
            from,
            to,
            amount,
            fee,
            nonce,
            timestamp,
            signature: Vec::new(),
            data,
        };

        // Calculate transaction ID
        tx.id = tx.calculate_hash();
        tx
    }

    /// Calculate transaction hash
    pub fn calculate_hash(&self) -> Hash {
        let mut data = Vec::new();
        data.extend(self.from.as_bytes());
        data.extend(self.to.as_bytes());
        data.extend(self.amount.to_le_bytes());
        data.extend(self.fee.to_le_bytes());
        data.extend(self.nonce.to_le_bytes());
        data.extend(self.timestamp.to_le_bytes());
        data.extend(&self.data);
        Hash::hash(&data)
    }

    /// Validate transaction basic fields
    pub fn validate(&self) -> Result<()> {
        if self.from.is_empty() {
            return Err(BlockchainError::TransactionValidationFailed(
                "Empty sender address".to_string()
            ));
        }

        if self.to.is_empty() {
            return Err(BlockchainError::TransactionValidationFailed(
                "Empty recipient address".to_string()
            ));
        }

        if self.amount == 0 && self.data.is_empty() {
            return Err(BlockchainError::TransactionValidationFailed(
                "Transaction must transfer amount or contain data".to_string()
            ));
        }

        // Verify hash
        let calculated_hash = self.calculate_hash();
        if calculated_hash != self.id {
            return Err(BlockchainError::TransactionValidationFailed(
                "Transaction hash mismatch".to_string()
            ));
        }

        Ok(())
    }

    /// Set transaction signature
    pub fn set_signature(&mut self, signature: Vec<u8>) {
        self.signature = signature;
    }

    /// Get transaction size in bytes (approximate)
    pub fn size(&self) -> usize {
        32 + // id
        self.from.len() +
        self.to.len() +
        8 + // amount
        8 + // fee  
        8 + // nonce
        8 + // timestamp
        self.signature.len() +
        self.data.len()
    }
}

/// Block header containing metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockHeader {
    /// Block hash
    pub hash: Hash,
    /// Previous block hash
    pub prev_hash: Hash,
    /// Merkle root of transactions
    pub merkle_root: Hash,
    /// Block timestamp
    pub timestamp: u64,
    /// Block height/number
    pub height: u64,
    /// Number of transactions
    pub transaction_count: u32,
    /// Block version
    pub version: u32,
    /// Mining difficulty (for PoW chains)
    pub difficulty: u64,
    /// Mining nonce (for PoW chains)
    pub nonce: u64,
    /// Block proposer/validator
    pub proposer: String,
}

impl BlockHeader {
    /// Calculate merkle root of transactions
    pub fn calculate_merkle_root(transactions: &[Transaction]) -> Hash {
        if transactions.is_empty() {
            return Hash::zero();
        }

        let mut hashes: Vec<Hash> = transactions.iter()
            .map(|tx| tx.id.clone())
            .collect();

        while hashes.len() > 1 {
            let mut new_hashes = Vec::new();
            
            for chunk in hashes.chunks(2) {
                let combined = if chunk.len() == 2 {
                    let mut data = Vec::new();
                    data.extend(chunk[0].as_bytes());
                    data.extend(chunk[1].as_bytes());
                    Hash::hash(&data)
                } else {
                    chunk[0].clone()
                };
                new_hashes.push(combined);
            }
            
            hashes = new_hashes;
        }

        hashes[0].clone()
    }

    /// Calculate block header hash
    pub fn calculate_hash(&self) -> Hash {
        let mut data = Vec::new();
        data.extend(self.prev_hash.as_bytes());
        data.extend(self.merkle_root.as_bytes());
        data.extend(self.timestamp.to_le_bytes());
        data.extend(self.height.to_le_bytes());
        data.extend(self.transaction_count.to_le_bytes());
        data.extend(self.version.to_le_bytes());
        data.extend(self.difficulty.to_le_bytes());
        data.extend(self.nonce.to_le_bytes());
        data.extend(self.proposer.as_bytes());
        Hash::hash(&data)
    }
}

/// Complete block structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    /// Block header
    pub header: BlockHeader,
    /// Block transactions
    pub transactions: Vec<Transaction>,
    /// Block signature
    pub signature: Vec<u8>,
}

impl Block {
    /// Create a new block
    pub fn new(
        prev_hash: Hash,
        transactions: Vec<Transaction>,
        height: u64,
        proposer: String,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let merkle_root = BlockHeader::calculate_merkle_root(&transactions);

        let mut header = BlockHeader {
            hash: Hash::zero(),
            prev_hash,
            merkle_root,
            timestamp,
            height,
            transaction_count: transactions.len() as u32,
            version: 1,
            difficulty: 0,
            nonce: 0,
            proposer,
        };

        header.hash = header.calculate_hash();

        Block {
            header,
            transactions,
            signature: Vec::new(),
        }
    }

    /// Create genesis block
    pub fn genesis(proposer: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let merkle_root = BlockHeader::calculate_merkle_root(&[]);

        let mut header = BlockHeader {
            hash: Hash::zero(),
            prev_hash: Hash::zero(),
            merkle_root,
            timestamp,
            height: 0,
            transaction_count: 0,
            version: 1,
            difficulty: 0,
            nonce: 0,
            proposer,
        };

        // Calculate hash first
        header.hash = header.calculate_hash();
        // Genesis block points to itself
        header.prev_hash = header.hash.clone();
        // Recalculate hash after setting prev_hash
        header.hash = header.calculate_hash();
        
        Block {
            header,
            transactions: Vec::new(),
            signature: Vec::new(),
        }
    }

    /// Validate block structure and transactions
    pub fn validate(&self) -> Result<()> {
        // Validate header hash
        let calculated_hash = self.header.calculate_hash();
        if calculated_hash != self.header.hash {
            return Err(BlockchainError::BlockValidationFailed(
                "Block hash mismatch".to_string()
            ));
        }

        // Validate merkle root
        let calculated_merkle = BlockHeader::calculate_merkle_root(&self.transactions);
        if calculated_merkle != self.header.merkle_root {
            return Err(BlockchainError::BlockValidationFailed(
                "Merkle root mismatch".to_string()
            ));
        }

        // Validate transaction count
        if self.transactions.len() != self.header.transaction_count as usize {
            return Err(BlockchainError::BlockValidationFailed(
                "Transaction count mismatch".to_string()
            ));
        }

        // Validate each transaction
        for tx in &self.transactions {
            tx.validate()?;
        }

        Ok(())
    }

    /// Get block size in bytes (approximate)
    pub fn size(&self) -> usize {
        std::mem::size_of::<BlockHeader>() +
        self.transactions.iter().map(|tx| tx.size()).sum::<usize>() +
        self.signature.len()
    }

    /// Set block signature
    pub fn set_signature(&mut self, signature: Vec<u8>) {
        self.signature = signature;
    }
}

/// Blockchain state management
#[derive(Debug, Clone)]
pub struct Blockchain {
    /// Chain of blocks indexed by hash
    blocks: HashMap<Hash, Block>,
    /// Current chain tip (latest block hash)
    tip: Option<Hash>,
    /// Genesis block hash
    genesis_hash: Option<Hash>,
    /// Chain height
    height: u64,
    /// Total difficulty
    total_difficulty: u64,
}

impl Blockchain {
    /// Create new blockchain
    pub fn new() -> Self {
        Blockchain {
            blocks: HashMap::new(),
            tip: None,
            genesis_hash: None,
            height: 0,
            total_difficulty: 0,
        }
    }

    /// Initialize blockchain with genesis block
    pub fn initialize_with_genesis(&mut self, genesis: Block) -> Result<()> {
        if genesis.header.height != 0 {
            return Err(BlockchainError::GenesisBlockMismatch);
        }

        genesis.validate()?;

        let genesis_hash = genesis.header.hash.clone();
        self.blocks.insert(genesis_hash.clone(), genesis);
        self.tip = Some(genesis_hash.clone());
        self.genesis_hash = Some(genesis_hash);
        self.height = 0;
        self.total_difficulty = 0;

        Ok(())
    }

    /// Add a new block to the chain
    pub fn add_block(&mut self, block: Block) -> Result<()> {
        // Validate block
        block.validate()?;

        // Check if previous block exists (except for genesis)
        if block.header.height > 0 {
            if !self.blocks.contains_key(&block.header.prev_hash) {
                return Err(BlockchainError::ChainValidationFailed(
                    "Previous block not found".to_string()
                ));
            }

            // Validate height
            if let Some(prev_block) = self.blocks.get(&block.header.prev_hash) {
                if block.header.height != prev_block.header.height + 1 {
                    return Err(BlockchainError::ChainValidationFailed(
                        "Invalid block height".to_string()
                    ));
                }
            }
        }

        let block_hash = block.header.hash.clone();
        let block_height = block.header.height;

        // Add block to chain
        self.blocks.insert(block_hash.clone(), block);

        // Update tip if this is the new longest chain
        if block_height > self.height {
            self.tip = Some(block_hash);
            self.height = block_height;
            self.total_difficulty += 1; // Simplified difficulty calculation
        }

        Ok(())
    }

    /// Get block by hash
    pub fn get_block(&self, hash: &Hash) -> Option<&Block> {
        self.blocks.get(hash)
    }

    /// Get current tip block
    pub fn get_tip(&self) -> Option<&Block> {
        self.tip.as_ref().and_then(|hash| self.blocks.get(hash))
    }

    /// Get genesis block
    pub fn get_genesis(&self) -> Option<&Block> {
        self.genesis_hash.as_ref().and_then(|hash| self.blocks.get(hash))
    }

    /// Get blockchain height
    pub fn get_height(&self) -> u64 {
        self.height
    }

    /// Get total number of blocks
    pub fn get_block_count(&self) -> usize {
        self.blocks.len()
    }

    /// Validate entire blockchain
    pub fn validate_chain(&self) -> Result<()> {
        if self.blocks.is_empty() {
            return Ok(());
        }

        // Validate genesis block
        if let Some(genesis) = self.get_genesis() {
            genesis.validate()?;
        }

        // Validate chain continuity
        let mut current_hash = self.genesis_hash.clone();
        let mut height = 0;

        while let Some(hash) = current_hash {
            if let Some(block) = self.blocks.get(&hash) {
                block.validate()?;

                if block.header.height != height {
                    return Err(BlockchainError::ChainValidationFailed(
                        format!("Height mismatch at block {}", height)
                    ));
                }

                // Find next block
                let next_height = height + 1;
                current_hash = self.blocks.iter()
                    .find(|(_, b)| b.header.height == next_height && b.header.prev_hash == hash)
                    .map(|(h, _)| h.clone());

                height += 1;
            } else {
                break;
            }
        }

        Ok(())
    }
}

impl Default for Blockchain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_creation() {
        let data = b"test data";
        let hash = Hash::hash(data);
        assert_eq!(hash.as_bytes().len(), 32);
    }

    #[test]
    fn test_transaction_creation() {
        let tx = Transaction::new(
            "sender".to_string(),
            "recipient".to_string(),
            100,
            10,
            1,
            vec![],
        );
        
        assert_eq!(tx.amount, 100);
        assert_eq!(tx.fee, 10);
        assert_eq!(tx.nonce, 1);
        assert!(tx.validate().is_ok());
    }

    #[test]
    fn test_transaction_validation() {
        let mut tx = Transaction::new(
            "".to_string(), // Empty sender should fail
            "recipient".to_string(),
            100,
            10,
            1,
            vec![],
        );
        
        assert!(tx.validate().is_err());
        
        tx.from = "sender".to_string();
        // Need to recalculate hash after changing from field
        tx.id = tx.calculate_hash();
        assert!(tx.validate().is_ok());
    }

    #[test]
    fn test_block_creation() {
        let tx = Transaction::new(
            "sender".to_string(),
            "recipient".to_string(),
            100,
            10,
            1,
            vec![],
        );
        
        let block = Block::new(
            Hash::zero(),
            vec![tx],
            1,
            "proposer".to_string(),
        );
        
        assert_eq!(block.header.height, 1);
        assert_eq!(block.transactions.len(), 1);
        assert!(block.validate().is_ok());
    }

    #[test]
    fn test_genesis_block() {
        let genesis = Block::genesis("genesis_proposer".to_string());
        assert_eq!(genesis.header.height, 0);
        assert_eq!(genesis.transactions.len(), 0);
        assert!(genesis.validate().is_ok());
    }

    #[test]
    fn test_blockchain_initialization() {
        let mut blockchain = Blockchain::new();
        let genesis = Block::genesis("proposer".to_string());
        
        assert!(blockchain.initialize_with_genesis(genesis).is_ok());
        assert_eq!(blockchain.get_height(), 0);
        assert!(blockchain.get_genesis().is_some());
    }

    #[test]
    fn test_blockchain_add_block() {
        let mut blockchain = Blockchain::new();
        let genesis = Block::genesis("proposer".to_string());
        let genesis_hash = genesis.header.hash.clone();
        
        blockchain.initialize_with_genesis(genesis).unwrap();
        
        let tx = Transaction::new(
            "sender".to_string(),
            "recipient".to_string(),
            100,
            10,
            1,
            vec![],
        );
        
        let block = Block::new(genesis_hash, vec![tx], 1, "proposer".to_string());
        
        assert!(blockchain.add_block(block).is_ok());
        assert_eq!(blockchain.get_height(), 1);
        assert_eq!(blockchain.get_block_count(), 2);
    }

    #[test]
    fn test_merkle_root_calculation() {
        let tx1 = Transaction::new("a".to_string(), "b".to_string(), 1, 0, 1, vec![]);
        let tx2 = Transaction::new("b".to_string(), "c".to_string(), 2, 0, 1, vec![]);
        
        let merkle_root = BlockHeader::calculate_merkle_root(&[tx1, tx2]);
        assert_ne!(merkle_root, Hash::zero());
    }

    #[test]
    fn test_blockchain_validation() {
        let mut blockchain = Blockchain::new();
        let genesis = Block::genesis("proposer".to_string());
        blockchain.initialize_with_genesis(genesis).unwrap();
        
        let tx = Transaction::new(
            "sender".to_string(),
            "recipient".to_string(),
            100,
            10,
            1,
            vec![],
        );
        
        let genesis_hash = blockchain.get_genesis().unwrap().header.hash.clone();
        let block = Block::new(genesis_hash, vec![tx], 1, "proposer".to_string());
        blockchain.add_block(block).unwrap();
        
        assert!(blockchain.validate_chain().is_ok());
    }
}
