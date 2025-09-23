use crate::crypto::{hash, CCPublicKey, CCSignature, Hash};
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Transaction structure optimized for high throughput
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Sender's public key
    pub from: CCPublicKey,
    /// Recipient's public key
    pub to: CCPublicKey,
    /// Amount to transfer (in smallest units)
    pub amount: u64,
    /// Transaction fee
    pub fee: u64,
    /// Nonce to prevent replay attacks
    pub nonce: u64,
    /// Additional data payload (optional)
    pub data: Vec<u8>,
    /// Transaction signature
    pub signature: CCSignature,
}

impl Transaction {
    /// Create a new transaction (without signature)
    pub fn new(
        from: CCPublicKey,
        to: CCPublicKey,
        amount: u64,
        fee: u64,
        nonce: u64,
        data: Vec<u8>,
    ) -> Self {
        Self {
            from,
            to,
            amount,
            fee,
            nonce,
            data,
            signature: CCSignature([0u8; 64]), // Placeholder signature
        }
    }

    /// Get transaction hash (excluding signature)
    pub fn hash(&self) -> Hash {
        let mut tx_copy = self.clone();
        tx_copy.signature = CCSignature([0u8; 64]); // Zero out signature for hashing

        let serialized = bincode::serialize(&tx_copy).expect("Serialization should not fail");
        hash(&serialized)
    }

    /// Sign the transaction
    pub fn sign(&mut self, keypair: &crate::crypto::CCKeypair) {
        let tx_hash = self.hash();
        self.signature = keypair.sign(&tx_hash);
    }

    /// Verify transaction signature
    pub fn verify_signature(&self) -> bool {
        let tx_hash = self.hash();
        self.from.verify(&tx_hash, &self.signature)
    }

    /// Validate transaction (basic checks)
    pub fn validate(&self) -> Result<()> {
        // Check signature
        if !self.verify_signature() {
            return Err(crate::CCError::Transaction("Invalid signature".to_string()));
        }

        // Check amount and fee are not zero (unless it's a data transaction)
        if self.amount == 0 && self.data.is_empty() {
            return Err(crate::CCError::Transaction(
                "Transaction has no value or data".to_string(),
            ));
        }

        // Check data size limit (1KB max for efficiency)
        if self.data.len() > 1024 {
            return Err(crate::CCError::Transaction(
                "Data payload too large".to_string(),
            ));
        }

        Ok(())
    }

    /// Get transaction size in bytes
    pub fn size(&self) -> usize {
        bincode::serialize(self).map(|data| data.len()).unwrap_or(0)
    }

    /// Check if this is a coinbase transaction (from genesis)
    pub fn is_coinbase(&self) -> bool {
        self.from.0 == [0u8; 32]
    }
}

/// Parallel transaction processor for high-throughput processing
pub struct ParallelTransactionProcessor {
    /// Thread pool for parallel processing
    thread_pool: rayon::ThreadPool,
    /// Maximum batch size for parallel processing
    max_batch_size: usize,
}

impl ParallelTransactionProcessor {
    /// Create a new parallel transaction processor
    pub fn new(num_threads: Option<usize>, max_batch_size: usize) -> Self {
        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads.unwrap_or_else(num_cpus::get))
            .build()
            .expect("Failed to create thread pool");

        Self {
            thread_pool,
            max_batch_size,
        }
    }

    /// Process transactions in parallel batches
    pub fn process_batch(&self, transactions: &[Transaction]) -> Vec<Result<()>> {
        use rayon::prelude::*;

        self.thread_pool.install(|| {
            transactions
                .par_chunks(self.max_batch_size)
                .flat_map(|chunk| {
                    chunk.par_iter().map(|tx| tx.validate()).collect::<Vec<_>>()
                })
                .collect()
        })
    }

    /// Verify signatures in parallel
    pub fn verify_signatures_parallel(&self, transactions: &[Transaction]) -> Vec<bool> {
        use rayon::prelude::*;

        self.thread_pool.install(|| {
            transactions
                .par_iter()
                .map(|tx| tx.verify_signature())
                .collect()
        })
    }

    /// Execute transactions in parallel (for independent transactions)
    pub fn execute_parallel<F, R>(&self, transactions: &[Transaction], executor: F) -> Vec<R>
    where
        F: Fn(&Transaction) -> R + Send + Sync,
        R: Send,
    {
        use rayon::prelude::*;

        self.thread_pool.install(|| {
            transactions.par_iter().map(&executor).collect()
        })
    }

    /// Batch verify transactions with dependency checking
    pub fn batch_verify_with_dependencies(
        &self,
        transactions: &[Transaction],
        state: &crate::state::StateManager,
    ) -> Vec<Result<()>> {
        use rayon::prelude::*;
        use std::sync::Arc;

        let state = Arc::new(state);
        
        self.thread_pool.install(|| {
            // Group transactions by sender to handle dependencies
            let mut sender_groups: std::collections::HashMap<crate::crypto::CCPublicKey, Vec<&Transaction>> = 
                std::collections::HashMap::new();

            for tx in transactions {
                sender_groups.entry(tx.from).or_default().push(tx);
            }

            // Sort each group by nonce to handle dependencies
            for group in sender_groups.values_mut() {
                group.sort_by_key(|tx| tx.nonce);
            }

            // Process each sender group in parallel
            sender_groups
                .par_iter()
                .flat_map(|(_sender, txs)| {
                    // Within each sender group, process sequentially to maintain order
                    txs.iter().map(|tx| state.validate_transaction(tx)).collect::<Vec<_>>()
                })
                .collect()
        })
    }
}

/// High-performance transaction batch for optimized processing
#[derive(Debug, Clone)]
pub struct TransactionBatch {
    /// Transactions in the batch
    pub transactions: Vec<Transaction>,
    /// Batch metadata
    pub metadata: BatchMetadata,
}

#[derive(Debug, Clone)]
pub struct BatchMetadata {
    /// Batch creation timestamp
    pub created_at: u64,
    /// Batch size in bytes
    pub size_bytes: usize,
    /// Number of transactions
    pub tx_count: usize,
    /// Average transaction fee
    pub avg_fee: u64,
    /// Batch priority score
    pub priority_score: f64,
}

impl TransactionBatch {
    /// Create a new transaction batch
    pub fn new(transactions: Vec<Transaction>) -> Self {
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let size_bytes = transactions.iter().map(|tx| tx.size()).sum();
        let tx_count = transactions.len();
        let avg_fee = if tx_count > 0 {
            transactions.iter().map(|tx| tx.fee).sum::<u64>() / tx_count as u64
        } else {
            0
        };
        
        // Calculate priority score based on fees and urgency
        let priority_score = avg_fee as f64 * tx_count as f64;

        let metadata = BatchMetadata {
            created_at,
            size_bytes,
            tx_count,
            avg_fee,
            priority_score,
        };

        Self {
            transactions,
            metadata,
        }
    }

    /// Validate all transactions in the batch
    pub fn validate_all(&self) -> Vec<Result<()>> {
        self.transactions.iter().map(|tx| tx.validate()).collect()
    }

    /// Get batch hash for identification
    pub fn batch_hash(&self) -> crate::crypto::Hash {
        let tx_hashes: Vec<crate::crypto::Hash> = self.transactions.iter().map(|tx| tx.hash()).collect();
        let merkle_tree = crate::crypto::MerkleTree::build(&tx_hashes);
        merkle_tree.root()
    }

    /// Split batch into smaller chunks
    pub fn split_into_chunks(&self, chunk_size: usize) -> Vec<TransactionBatch> {
        self.transactions
            .chunks(chunk_size)
            .map(|chunk| TransactionBatch::new(chunk.to_vec()))
            .collect()
    }
}

/// Smart transaction batching system for optimal throughput
pub struct SmartBatcher {
    /// Maximum batch size in transactions
    max_batch_size: usize,
    /// Maximum batch size in bytes
    max_batch_bytes: usize,
    /// Maximum batching delay
    max_delay: std::time::Duration,
    /// Pending transactions for batching
    pending: Vec<Transaction>,
    /// Last batch creation time
    last_batch_time: std::time::Instant,
}

impl SmartBatcher {
    pub fn new(max_batch_size: usize, max_batch_bytes: usize, max_delay: std::time::Duration) -> Self {
        Self {
            max_batch_size,
            max_batch_bytes,
            max_delay,
            pending: Vec::new(),
            last_batch_time: std::time::Instant::now(),
        }
    }

    /// Add transaction to batcher
    pub fn add_transaction(&mut self, tx: Transaction) -> Option<TransactionBatch> {
        self.pending.push(tx);
        self.try_create_batch()
    }

    /// Try to create a batch if conditions are met
    pub fn try_create_batch(&mut self) -> Option<TransactionBatch> {
        let current_size = self.pending.iter().map(|tx| tx.size()).sum::<usize>();
        let elapsed = self.last_batch_time.elapsed();

        if self.pending.len() >= self.max_batch_size
            || current_size >= self.max_batch_bytes
            || elapsed >= self.max_delay && !self.pending.is_empty()
        {
            let batch = TransactionBatch::new(std::mem::take(&mut self.pending));
            self.last_batch_time = std::time::Instant::now();
            Some(batch)
        } else {
            None
        }
    }

    /// Force create batch with pending transactions
    pub fn force_batch(&mut self) -> Option<TransactionBatch> {
        if self.pending.is_empty() {
            None
        } else {
            let batch = TransactionBatch::new(std::mem::take(&mut self.pending));
            self.last_batch_time = std::time::Instant::now();
            Some(batch)
        }
    }

    /// Get pending transaction count
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Get pending transactions size in bytes
    pub fn pending_size(&self) -> usize {
        self.pending.iter().map(|tx| tx.size()).sum()
    }
}

/// Transaction pool for managing pending transactions
#[derive(Debug)]
pub struct TransactionPool {
    /// Pending transactions indexed by hash
    pending: dashmap::DashMap<Hash, Transaction>,
    /// Transactions indexed by sender for nonce checking
    by_sender: dashmap::DashMap<CCPublicKey, std::collections::BTreeMap<u64, Hash>>,
    /// Maximum pool size
    max_size: usize,
}

impl TransactionPool {
    /// Create a new transaction pool
    pub fn new(max_size: usize) -> Self {
        Self {
            pending: dashmap::DashMap::new(),
            by_sender: dashmap::DashMap::new(),
            max_size,
        }
    }

    /// Add transaction to pool
    pub fn add_transaction(&self, tx: Transaction) -> Result<()> {
        // Validate transaction
        tx.validate()?;

        let tx_hash = tx.hash();

        // Check if pool is full
        if self.pending.len() >= self.max_size {
            return Err(crate::CCError::Transaction(
                "Transaction pool is full".to_string(),
            ));
        }

        // Check for duplicate
        if self.pending.contains_key(&tx_hash) {
            return Err(crate::CCError::Transaction(
                "Transaction already in pool".to_string(),
            ));
        }

        // Add to pending
        self.pending.insert(tx_hash, tx.clone());

        // Index by sender
        self.by_sender
            .entry(tx.from.clone())
            .or_insert_with(std::collections::BTreeMap::new)
            .insert(tx.nonce, tx_hash);

        Ok(())
    }

    /// Remove transaction from pool
    pub fn remove_transaction(&self, tx_hash: &Hash) -> Option<Transaction> {
        if let Some((_, tx)) = self.pending.remove(tx_hash) {
            // Remove from sender index
            if let Some(mut sender_txs) = self.by_sender.get_mut(&tx.from) {
                sender_txs.remove(&tx.nonce);
                if sender_txs.is_empty() {
                    drop(sender_txs);
                    self.by_sender.remove(&tx.from);
                }
            }
            Some(tx)
        } else {
            None
        }
    }

    /// Get transactions for block creation (sorted by fee)
    pub fn get_transactions_for_block(
        &self,
        max_count: usize,
        max_size: usize,
    ) -> Vec<Transaction> {
        let mut transactions: Vec<_> = self
            .pending
            .iter()
            .map(|entry| entry.value().clone())
            .collect();

        // Sort by fee (descending) then by nonce (ascending)
        transactions.sort_by(|a, b| b.fee.cmp(&a.fee).then_with(|| a.nonce.cmp(&b.nonce)));

        let mut selected = Vec::new();
        let mut total_size = 0;

        for tx in transactions {
            if selected.len() >= max_count {
                break;
            }

            let tx_size = tx.size();
            if total_size + tx_size > max_size {
                break;
            }

            selected.push(tx);
            total_size += tx_size;
        }

        selected
    }

    /// Get pool statistics
    pub fn stats(&self) -> (usize, usize) {
        (self.pending.len(), self.max_size)
    }

    /// Clear all transactions
    pub fn clear(&self) {
        self.pending.clear();
        self.by_sender.clear();
    }
}
