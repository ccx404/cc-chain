use crate::core::crypto::{hash, CCPublicKey, Hash, MerkleTree};
use crate::core::error::Result;
use crate::core::transaction::Transaction;
use serde::{Deserialize, Serialize};

/// Block header containing metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    /// Previous block hash
    pub prev_hash: Hash,
    /// Merkle root of transactions
    pub tx_root: Hash,
    /// State root after applying transactions
    pub state_root: Hash,
    /// Block height
    pub height: u64,
    /// Timestamp (Unix timestamp in milliseconds)
    pub timestamp: u64,
    /// Block proposer (validator)
    pub proposer: CCPublicKey,
    /// Gas limit for the block
    pub gas_limit: u64,
    /// Gas used in the block
    pub gas_used: u64,
    /// Extra data (for future extensions)
    pub extra_data: Vec<u8>,
}

impl BlockHeader {
    /// Calculate the hash of this block header
    pub fn hash(&self) -> Hash {
        let serialized = bincode::serialize(self).expect("Serialization should not fail");
        hash(&serialized)
    }
}

/// Complete block containing header and transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// Block header
    pub header: BlockHeader,
    /// Transactions in this block
    pub transactions: Vec<Transaction>,
}

impl Block {
    /// Create a new block
    pub fn new(
        prev_hash: Hash,
        height: u64,
        timestamp: u64,
        proposer: CCPublicKey,
        transactions: Vec<Transaction>,
        state_root: Hash,
        gas_limit: u64,
    ) -> Self {
        // Calculate transaction merkle root
        let tx_hashes: Vec<Hash> = transactions.iter().map(|tx| tx.hash()).collect();
        let merkle_tree = MerkleTree::build(&tx_hashes);
        let tx_root = merkle_tree.root();

        // Calculate gas used
        let gas_used = transactions.len() as u64 * 1000; // Simple gas model: 1000 gas per tx

        let header = BlockHeader {
            prev_hash,
            tx_root,
            state_root,
            height,
            timestamp,
            proposer,
            gas_limit,
            gas_used,
            extra_data: Vec::new(),
        };

        Self {
            header,
            transactions,
        }
    }

    /// Get the hash of this block
    pub fn hash(&self) -> Hash {
        self.header.hash()
    }

    /// Validate the block structure
    pub fn validate(&self) -> Result<()> {
        // Check timestamp is reasonable (not too far in future/past)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| crate::CCError::Block(format!("Time error: {}", e)))?
            .as_millis() as u64;

        if self.header.timestamp > now + 30000 {
            // 30 seconds in future max
            return Err(crate::CCError::Block(
                "Block timestamp too far in future".to_string(),
            ));
        }

        // Validate merkle root
        let tx_hashes: Vec<Hash> = self.transactions.iter().map(|tx| tx.hash()).collect();
        let merkle_tree = MerkleTree::build(&tx_hashes);
        if merkle_tree.root() != self.header.tx_root {
            return Err(crate::CCError::Block(
                "Invalid transaction merkle root".to_string(),
            ));
        }

        // Validate all transactions
        for tx in &self.transactions {
            tx.validate()?;
        }

        // Check gas limit
        if self.header.gas_used > self.header.gas_limit {
            return Err(crate::CCError::Block(
                "Gas used exceeds gas limit".to_string(),
            ));
        }

        Ok(())
    }

    /// Get block size in bytes
    pub fn size(&self) -> usize {
        bincode::serialize(self).map(|data| data.len()).unwrap_or(0)
    }

    /// Check if this is the genesis block
    pub fn is_genesis(&self) -> bool {
        self.header.height == 0 && self.header.prev_hash == [0u8; 32]
    }

    /// Create genesis block
    pub fn genesis(genesis_validator: CCPublicKey, initial_state_root: Hash) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Self::new(
            [0u8; 32], // No previous block
            0,         // Genesis height
            timestamp,
            genesis_validator,
            Vec::new(), // No transactions in genesis
            initial_state_root,
            1_000_000, // Genesis gas limit
        )
    }
}

/// Blockchain state maintaining blocks and chain metadata
#[derive(Debug)]
pub struct Blockchain {
    /// All blocks indexed by hash
    blocks: dashmap::DashMap<Hash, Block>,
    /// Block hashes indexed by height
    heights: dashmap::DashMap<u64, Hash>,
    /// Current chain head
    head: parking_lot::RwLock<Option<Hash>>,
    /// Genesis block hash
    genesis_hash: Hash,
}

impl Blockchain {
    /// Create new blockchain with genesis block
    pub fn new(genesis_block: Block) -> Result<Self> {
        let genesis_hash = genesis_block.hash();

        if !genesis_block.is_genesis() {
            return Err(crate::CCError::Block("Invalid genesis block".to_string()));
        }

        let blockchain = Self {
            blocks: dashmap::DashMap::new(),
            heights: dashmap::DashMap::new(),
            head: parking_lot::RwLock::new(Some(genesis_hash)),
            genesis_hash,
        };

        // Add genesis block
        blockchain.blocks.insert(genesis_hash, genesis_block);
        blockchain.heights.insert(0, genesis_hash);

        Ok(blockchain)
    }

    /// Add a new block to the chain
    pub fn add_block(&self, block: Block) -> Result<()> {
        // Validate block
        block.validate()?;

        let block_hash = block.hash();

        // Check if block already exists
        if self.blocks.contains_key(&block_hash) {
            return Ok(()); // Already have this block
        }

        // Check if parent exists
        if !self.blocks.contains_key(&block.header.prev_hash) && block.header.height > 0 {
            return Err(crate::CCError::Block("Parent block not found".to_string()));
        }

        // Check height is correct
        if block.header.height > 0 {
            if let Some(parent) = self.blocks.get(&block.header.prev_hash) {
                if block.header.height != parent.header.height + 1 {
                    return Err(crate::CCError::Block("Invalid block height".to_string()));
                }
            }
        }

        // Add block
        self.blocks.insert(block_hash, block.clone());
        self.heights.insert(block.header.height, block_hash);

        // Update head if this extends the main chain
        {
            let mut head = self.head.write();
            if let Some(current_head_hash) = *head {
                if let Some(current_head) = self.blocks.get(&current_head_hash) {
                    if block.header.height > current_head.header.height {
                        *head = Some(block_hash);
                    }
                }
            } else {
                *head = Some(block_hash);
            }
        }

        Ok(())
    }

    /// Get block by hash
    pub fn get_block(&self, hash: &Hash) -> Option<Block> {
        self.blocks.get(hash).map(|entry| entry.value().clone())
    }

    /// Get block by height
    pub fn get_block_by_height(&self, height: u64) -> Option<Block> {
        self.heights
            .get(&height)
            .and_then(|hash_entry| self.blocks.get(hash_entry.value()))
            .map(|block_entry| block_entry.value().clone())
    }

    /// Get current head block
    pub fn get_head_block(&self) -> Option<Block> {
        let head = self.head.read();
        head.as_ref()
            .and_then(|hash| self.blocks.get(hash))
            .map(|entry| entry.value().clone())
    }

    /// Get current chain height
    pub fn get_height(&self) -> u64 {
        self.get_head_block()
            .map(|block| block.header.height)
            .unwrap_or(0)
    }

    /// Get genesis block
    pub fn get_genesis_block(&self) -> Option<Block> {
        self.blocks
            .get(&self.genesis_hash)
            .map(|entry| entry.value().clone())
    }

    /// Check if block exists
    pub fn has_block(&self, hash: &Hash) -> bool {
        self.blocks.contains_key(hash)
    }
}
