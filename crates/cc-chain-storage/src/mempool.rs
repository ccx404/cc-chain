use cc_chain_sdk::{Transaction, Result, Hash, CCPublicKey, CCError};
use parking_lot::RwLock;
use dashmap::DashMap;
use std::collections::BTreeMap;

/// Transaction pool for managing pending transactions
#[derive(Debug)]
pub struct TransactionPool {
    /// Pending transactions indexed by hash
    pending: DashMap<Hash, Transaction>,
    /// Transactions indexed by sender for nonce checking
    by_sender: DashMap<CCPublicKey, BTreeMap<u64, Hash>>,
    /// Maximum pool size
    max_size: usize,
}

impl TransactionPool {
    /// Create a new transaction pool
    pub fn new(max_size: usize) -> Self {
        Self {
            pending: DashMap::new(),
            by_sender: DashMap::new(),
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
            return Err(CCError::Transaction(
                "Transaction pool is full".to_string(),
            ));
        }

        // Check for duplicate
        if self.pending.contains_key(&tx_hash) {
            return Err(CCError::Transaction(
                "Transaction already in pool".to_string(),
            ));
        }

        // Add to pending
        self.pending.insert(tx_hash, tx.clone());

        // Index by sender
        self.by_sender
            .entry(tx.from.clone())
            .or_insert_with(BTreeMap::new)
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

/// Memory pool for pending transactions with prioritization
pub struct Mempool {
    /// Transaction pool
    pool: TransactionPool,
    /// Maximum mempool size in bytes
    max_size_bytes: usize,
    /// Current size in bytes
    current_size: parking_lot::RwLock<usize>,
    /// Fee rate cache for quick sorting
    fee_rates: DashMap<Hash, u64>,
}

/// Mempool statistics
#[derive(Debug, Clone)]
pub struct MempoolStats {
    pub transaction_count: usize,
    pub max_transactions: usize,
    pub current_size_bytes: usize,
    pub max_size_bytes: usize,
}

impl MempoolStats {
    /// Get utilization percentage
    pub fn utilization_percent(&self) -> f64 {
        if self.max_transactions > 0 {
            (self.transaction_count as f64 / self.max_transactions as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Get size utilization percentage
    pub fn size_utilization_percent(&self) -> f64 {
        if self.max_size_bytes > 0 {
            (self.current_size_bytes as f64 / self.max_size_bytes as f64) * 100.0
        } else {
            0.0
        }
    }
}

impl Mempool {
    /// Create new mempool
    pub fn new(max_transactions: usize, max_size_bytes: usize) -> Self {
        Self {
            pool: TransactionPool::new(max_transactions),
            max_size_bytes,
            current_size: RwLock::new(0),
            fee_rates: DashMap::new(),
        }
    }

    /// Add transaction to mempool
    pub fn add_transaction(&self, tx: Transaction) -> Result<()> {
        let tx_size = tx.size();

        // Check size limits
        {
            let current_size = *self.current_size.read();
            if current_size + tx_size > self.max_size_bytes {
                return Err(CCError::Transaction(
                    "Mempool size limit exceeded".to_string(),
                ));
            }
        }

        // Calculate fee rate
        let fee_rate = if tx_size > 0 {
            tx.fee * 1000 / tx_size as u64
        } else {
            0
        };
        let tx_hash = tx.hash();

        // Add to pool
        self.pool.add_transaction(tx)?;

        // Update size and fee rate cache
        *self.current_size.write() += tx_size;
        self.fee_rates.insert(tx_hash, fee_rate);

        Ok(())
    }

    /// Remove transaction from mempool
    pub fn remove_transaction(&self, tx_hash: &Hash) -> Option<Transaction> {
        if let Some(tx) = self.pool.remove_transaction(tx_hash) {
            let tx_size = tx.size();

            // Update size
            *self.current_size.write() -= tx_size;
            self.fee_rates.remove(tx_hash);

            Some(tx)
        } else {
            None
        }
    }

    /// Get transactions for block creation (high-priority first)
    pub fn get_transactions_for_block(
        &self,
        max_count: usize,
        max_size: usize,
    ) -> Vec<Transaction> {
        self.pool.get_transactions_for_block(max_count, max_size)
    }

    /// Get mempool statistics
    pub fn stats(&self) -> MempoolStats {
        let (count, max_count) = self.pool.stats();
        let current_size = *self.current_size.read();

        MempoolStats {
            transaction_count: count,
            max_transactions: max_count,
            current_size_bytes: current_size,
            max_size_bytes: self.max_size_bytes,
        }
    }

    /// Clear all transactions
    pub fn clear(&self) {
        self.pool.clear();
        *self.current_size.write() = 0;
        self.fee_rates.clear();
    }

    /// Get transaction by hash
    pub fn get_transaction(&self, _tx_hash: &Hash) -> Option<Transaction> {
        // This is a simplified implementation - in practice we'd need to store transactions
        // in the pool with hash indexing
        None
    }

    /// Validate transaction before adding
    pub fn validate_transaction(&self, tx: &Transaction) -> Result<()> {
        // Basic validation
        tx.validate()?;

        // Check if already in mempool
        let tx_hash = tx.hash();
        if self.fee_rates.contains_key(&tx_hash) {
            return Err(CCError::Transaction(
                "Transaction already in mempool".to_string(),
            ));
        }

        Ok(())
    }
}
