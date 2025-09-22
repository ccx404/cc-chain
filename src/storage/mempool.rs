use crate::transaction::{Transaction, TransactionPool};
use crate::Result;

/// Memory pool for pending transactions with prioritization
pub struct Mempool {
    /// Transaction pool
    pool: TransactionPool,
    /// Maximum mempool size in bytes
    max_size_bytes: usize,
    /// Current size in bytes
    current_size: parking_lot::RwLock<usize>,
    /// Fee rate cache for quick sorting
    fee_rates: dashmap::DashMap<crate::crypto::Hash, u64>,
}

impl Mempool {
    /// Create new mempool
    pub fn new(max_transactions: usize, max_size_bytes: usize) -> Self {
        Self {
            pool: TransactionPool::new(max_transactions),
            max_size_bytes,
            current_size: parking_lot::RwLock::new(0),
            fee_rates: dashmap::DashMap::new(),
        }
    }

    /// Add transaction to mempool
    pub fn add_transaction(&self, tx: Transaction) -> Result<()> {
        let tx_size = tx.size();

        // Check size limits
        {
            let current_size = *self.current_size.read();
            if current_size + tx_size > self.max_size_bytes {
                return Err(crate::CCError::Transaction(
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
    pub fn remove_transaction(&self, tx_hash: &crate::crypto::Hash) -> Option<Transaction> {
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
    pub fn get_transaction(&self, _tx_hash: &crate::crypto::Hash) -> Option<Transaction> {
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
            return Err(crate::CCError::Transaction(
                "Transaction already in mempool".to_string(),
            ));
        }

        Ok(())
    }
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
