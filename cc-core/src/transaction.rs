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
