//! Core transaction processing functionality
//!
//! This module provides comprehensive transaction processing capabilities including
//! validation, execution, mempool management, fee calculation, and batch processing.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BinaryHeap};
use std::cmp::Ordering;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Transaction processing errors
#[derive(Error, Debug, Clone, PartialEq)]
pub enum TransactionProcessingError {
    #[error("Transaction validation failed: {0}")]
    ValidationFailed(String),
    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: u64, available: u64 },
    #[error("Invalid transaction fee: {0}")]
    InvalidFee(String),
    #[error("Transaction already exists: {0}")]
    TransactionExists(String),
    #[error("Transaction not found: {0}")]
    TransactionNotFound(String),
    #[error("Invalid nonce: expected {expected}, got {actual}")]
    InvalidNonce { expected: u64, actual: u64 },
    #[error("Transaction expired")]
    TransactionExpired,
    #[error("Mempool full")]
    MempoolFull,
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Invalid signature")]
    InvalidSignature,
}

pub type Result<T> = std::result::Result<T, TransactionProcessingError>;

/// Transaction hash (32 bytes)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TxHash(pub [u8; 32]);

impl TxHash {
    pub fn new(data: [u8; 32]) -> Self {
        TxHash(data)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
    }
}

impl std::fmt::Display for TxHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Account address type
pub type Address = String;

/// Basic transaction structure for processing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub hash: TxHash,
    pub from: Address,
    pub to: Address,
    pub amount: u64,
    pub fee: u64,
    pub nonce: u64,
    pub gas_limit: u64,
    pub gas_price: u64,
    pub data: Vec<u8>,
    pub signature: Vec<u8>,
    pub timestamp: u64,
}

impl Transaction {
    /// Create new transaction
    pub fn new(
        from: Address,
        to: Address,
        amount: u64,
        fee: u64,
        nonce: u64,
        gas_limit: u64,
        gas_price: u64,
        data: Vec<u8>,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut tx = Transaction {
            hash: TxHash::new([0; 32]),
            from,
            to,
            amount,
            fee,
            nonce,
            gas_limit,
            gas_price,
            data,
            signature: Vec::new(),
            timestamp,
        };

        tx.hash = tx.calculate_hash();
        tx
    }

    /// Calculate transaction hash
    pub fn calculate_hash(&self) -> TxHash {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        
        hasher.update(self.from.as_bytes());
        hasher.update(self.to.as_bytes());
        hasher.update(self.amount.to_le_bytes());
        hasher.update(self.fee.to_le_bytes());
        hasher.update(self.nonce.to_le_bytes());
        hasher.update(self.gas_limit.to_le_bytes());
        hasher.update(self.gas_price.to_le_bytes());
        hasher.update(&self.data);
        hasher.update(self.timestamp.to_le_bytes());
        
        TxHash::new(hasher.finalize().into())
    }

    /// Get total cost (amount + fee + gas cost)
    pub fn total_cost(&self) -> u64 {
        self.amount + self.fee + (self.gas_limit * self.gas_price)
    }

    /// Check if transaction has expired
    pub fn is_expired(&self, max_age: Duration) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        now.saturating_sub(self.timestamp) > max_age.as_secs()
    }

    /// Basic validation
    pub fn validate(&self) -> Result<()> {
        if self.from.is_empty() {
            return Err(TransactionProcessingError::ValidationFailed(
                "Empty sender address".to_string()
            ));
        }

        if self.to.is_empty() {
            return Err(TransactionProcessingError::ValidationFailed(
                "Empty recipient address".to_string()
            ));
        }

        if self.amount == 0 && self.data.is_empty() {
            return Err(TransactionProcessingError::ValidationFailed(
                "Transaction must transfer amount or contain data".to_string()
            ));
        }

        if self.gas_limit == 0 {
            return Err(TransactionProcessingError::ValidationFailed(
                "Gas limit must be greater than zero".to_string()
            ));
        }

        if self.gas_price == 0 {
            return Err(TransactionProcessingError::ValidationFailed(
                "Gas price must be greater than zero".to_string()
            ));
        }

        // Verify hash
        let calculated_hash = self.calculate_hash();
        if calculated_hash != self.hash {
            return Err(TransactionProcessingError::ValidationFailed(
                "Transaction hash mismatch".to_string()
            ));
        }

        Ok(())
    }

    /// Set transaction signature
    pub fn sign(&mut self, signature: Vec<u8>) {
        self.signature = signature;
    }
}

/// Transaction status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Processing,
    Confirmed,
    Failed(String),
    Rejected(String),
}

/// Transaction with metadata for processing
#[derive(Debug, Clone)]
pub struct PendingTransaction {
    pub transaction: Transaction,
    pub status: TransactionStatus,
    pub received_at: Instant,
    pub attempts: u32,
    pub priority_score: u64,
}

impl PendingTransaction {
    pub fn new(transaction: Transaction) -> Self {
        let priority_score = transaction.gas_price * transaction.gas_limit + transaction.fee;
        
        PendingTransaction {
            transaction,
            status: TransactionStatus::Pending,
            received_at: Instant::now(),
            attempts: 0,
            priority_score,
        }
    }

    pub fn age(&self) -> Duration {
        self.received_at.elapsed()
    }
}

impl PartialEq for PendingTransaction {
    fn eq(&self, other: &Self) -> bool {
        self.transaction.hash == other.transaction.hash
    }
}

impl Eq for PendingTransaction {}

impl PartialOrd for PendingTransaction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PendingTransaction {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first
        self.priority_score.cmp(&other.priority_score)
            .then_with(|| other.received_at.cmp(&self.received_at)) // Older first for same priority
    }
}

/// Account state for transaction processing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountState {
    pub address: Address,
    pub balance: u64,
    pub nonce: u64,
    pub storage_root: Vec<u8>,
    pub code_hash: Vec<u8>,
}

impl AccountState {
    pub fn new(address: Address, balance: u64) -> Self {
        AccountState {
            address,
            balance,
            nonce: 0,
            storage_root: vec![0; 32],
            code_hash: vec![0; 32],
        }
    }

    pub fn can_pay(&self, amount: u64) -> bool {
        self.balance >= amount
    }

    pub fn increment_nonce(&mut self) {
        self.nonce += 1;
    }

    pub fn debit(&mut self, amount: u64) -> Result<()> {
        if self.balance < amount {
            return Err(TransactionProcessingError::InsufficientBalance {
                required: amount,
                available: self.balance,
            });
        }
        self.balance -= amount;
        Ok(())
    }

    pub fn credit(&mut self, amount: u64) {
        self.balance = self.balance.saturating_add(amount);
    }
}

/// Transaction execution result
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionResult {
    pub success: bool,
    pub gas_used: u64,
    pub return_data: Vec<u8>,
    pub error: Option<String>,
    pub state_changes: Vec<StateChange>,
}

/// State change from transaction execution
#[derive(Debug, Clone, PartialEq)]
pub struct StateChange {
    pub address: Address,
    pub old_balance: u64,
    pub new_balance: u64,
    pub old_nonce: u64,
    pub new_nonce: u64,
}

/// Transaction mempool with priority ordering
#[derive(Debug)]
pub struct Mempool {
    transactions: BinaryHeap<PendingTransaction>,
    tx_lookup: HashMap<TxHash, PendingTransaction>,
    pub account_nonces: HashMap<Address, u64>,
    max_size: usize,
    max_age: Duration,
    min_gas_price: u64,
}

impl Mempool {
    /// Create new mempool
    pub fn new(max_size: usize, max_age: Duration, min_gas_price: u64) -> Self {
        Mempool {
            transactions: BinaryHeap::with_capacity(max_size),
            tx_lookup: HashMap::with_capacity(max_size),
            account_nonces: HashMap::new(),
            max_size,
            max_age,
            min_gas_price,
        }
    }

    /// Add transaction to mempool
    pub fn add_transaction(&mut self, transaction: Transaction) -> Result<()> {
        // Validate transaction
        transaction.validate()?;

        // Check if already exists
        if self.tx_lookup.contains_key(&transaction.hash) {
            return Err(TransactionProcessingError::TransactionExists(
                transaction.hash.to_hex()
            ));
        }

        // Check minimum gas price
        if transaction.gas_price < self.min_gas_price {
            return Err(TransactionProcessingError::InvalidFee(
                format!("Gas price {} below minimum {}", transaction.gas_price, self.min_gas_price)
            ));
        }

        // Check if expired
        if transaction.is_expired(self.max_age) {
            return Err(TransactionProcessingError::TransactionExpired);
        }

        // Check nonce (should be current nonce for account or next expected)
        let expected_nonce = self.account_nonces.get(&transaction.from).copied().unwrap_or(0);
        if transaction.nonce < expected_nonce {
            return Err(TransactionProcessingError::InvalidNonce {
                expected: expected_nonce,
                actual: transaction.nonce,
            });
        }

        // Check mempool capacity
        if self.transactions.len() >= self.max_size {
            // Try to remove lowest priority transaction
            if let Some(lowest) = self.find_lowest_priority() {
                if lowest.priority_score >= PendingTransaction::new(transaction.clone()).priority_score {
                    return Err(TransactionProcessingError::MempoolFull);
                }
                self.remove_transaction(&lowest.transaction.hash);
            } else {
                return Err(TransactionProcessingError::MempoolFull);
            }
        }

        // Add to mempool
        let pending = PendingTransaction::new(transaction);
        self.transactions.push(pending.clone());
        self.tx_lookup.insert(pending.transaction.hash.clone(), pending.clone());
        
        // Update expected nonce to allow for sequential transactions
        let new_expected_nonce = pending.transaction.nonce + 1;
        self.account_nonces.insert(pending.transaction.from.clone(), new_expected_nonce);

        Ok(())
    }

    /// Remove transaction from mempool
    pub fn remove_transaction(&mut self, hash: &TxHash) -> Option<PendingTransaction> {
        if let Some(pending) = self.tx_lookup.remove(hash) {
            // Remove from heap (expensive operation)
            let mut temp_vec: Vec<_> = self.transactions.drain().collect();
            temp_vec.retain(|tx| tx.transaction.hash != *hash);
            self.transactions = temp_vec.into();

            // Update account nonce
            if let Some(nonce) = self.account_nonces.get_mut(&pending.transaction.from) {
                if *nonce == pending.transaction.nonce + 1 {
                    *nonce = pending.transaction.nonce;
                }
            }

            Some(pending)
        } else {
            None
        }
    }

    /// Get next transaction for processing (highest priority)
    pub fn pop_transaction(&mut self) -> Option<PendingTransaction> {
        self.cleanup_expired();
        
        if let Some(pending) = self.transactions.pop() {
            self.tx_lookup.remove(&pending.transaction.hash);
            Some(pending)
        } else {
            None
        }
    }

    /// Get transactions for a specific account
    pub fn get_account_transactions(&self, address: &Address) -> Vec<&PendingTransaction> {
        self.tx_lookup
            .values()
            .filter(|pending| pending.transaction.from == *address)
            .collect()
    }

    /// Get mempool statistics
    pub fn stats(&self) -> MempoolStats {
        let total_fees: u64 = self.tx_lookup
            .values()
            .map(|pending| pending.transaction.fee)
            .sum();
        
        let avg_gas_price = if self.tx_lookup.is_empty() {
            0
        } else {
            self.tx_lookup
                .values()
                .map(|pending| pending.transaction.gas_price)
                .sum::<u64>() / self.tx_lookup.len() as u64
        };

        MempoolStats {
            size: self.tx_lookup.len(),
            max_size: self.max_size,
            total_fees,
            avg_gas_price,
            min_gas_price: self.min_gas_price,
        }
    }

    /// Get pending transaction count
    pub fn len(&self) -> usize {
        self.tx_lookup.len()
    }

    /// Check if mempool is empty
    pub fn is_empty(&self) -> bool {
        self.tx_lookup.is_empty()
    }

    /// Clear all transactions
    pub fn clear(&mut self) {
        self.transactions.clear();
        self.tx_lookup.clear();
        self.account_nonces.clear();
    }

    /// Find lowest priority transaction
    fn find_lowest_priority(&self) -> Option<PendingTransaction> {
        self.tx_lookup
            .values()
            .min_by(|a, b| a.priority_score.cmp(&b.priority_score))
            .cloned()
    }

    /// Remove expired transactions
    fn cleanup_expired(&mut self) {
        let expired_hashes: Vec<_> = self.tx_lookup
            .values()
            .filter(|pending| pending.transaction.is_expired(self.max_age))
            .map(|pending| pending.transaction.hash.clone())
            .collect();

        for hash in expired_hashes {
            self.remove_transaction(&hash);
        }
    }
}

/// Mempool statistics
#[derive(Debug, Clone)]
pub struct MempoolStats {
    pub size: usize,
    pub max_size: usize,
    pub total_fees: u64,
    pub avg_gas_price: u64,
    pub min_gas_price: u64,
}

/// Transaction processor with state management
#[derive(Debug)]
pub struct TransactionProcessor {
    accounts: HashMap<Address, AccountState>,
    mempool: Mempool,
    processed_count: u64,
    failed_count: u64,
}

impl TransactionProcessor {
    /// Create new transaction processor
    pub fn new(mempool_size: usize, max_tx_age: Duration, min_gas_price: u64) -> Self {
        TransactionProcessor {
            accounts: HashMap::new(),
            mempool: Mempool::new(mempool_size, max_tx_age, min_gas_price),
            processed_count: 0,
            failed_count: 0,
        }
    }

    /// Create account with initial balance
    pub fn create_account(&mut self, address: Address, balance: u64) {
        self.accounts.insert(address.clone(), AccountState::new(address, balance));
    }

    /// Get account state
    pub fn get_account(&self, address: &Address) -> Option<&AccountState> {
        self.accounts.get(address)
    }

    /// Submit transaction to mempool
    pub fn submit_transaction(&mut self, transaction: Transaction) -> Result<()> {
        // Validate sender account exists and has sufficient balance
        let account = self.accounts.get(&transaction.from)
            .ok_or_else(|| TransactionProcessingError::ValidationFailed(
                format!("Account {} does not exist", transaction.from)
            ))?;

        if !account.can_pay(transaction.total_cost()) {
            return Err(TransactionProcessingError::InsufficientBalance {
                required: transaction.total_cost(),
                available: account.balance,
            });
        }

        // Check nonce (use mempool's nonce tracking for submitted transactions)
        let expected_nonce = self.mempool.account_nonces.get(&transaction.from).copied().unwrap_or(account.nonce);
        if transaction.nonce != expected_nonce {
            return Err(TransactionProcessingError::InvalidNonce {
                expected: expected_nonce,
                actual: transaction.nonce,
            });
        }

        self.mempool.add_transaction(transaction)
    }

    /// Process next transaction from mempool
    pub fn process_next_transaction(&mut self) -> Option<ExecutionResult> {
        if let Some(pending) = self.mempool.pop_transaction() {
            Some(self.execute_transaction(pending.transaction))
        } else {
            None
        }
    }

    /// Process batch of transactions
    pub fn process_batch(&mut self, batch_size: usize) -> Vec<ExecutionResult> {
        let mut results = Vec::with_capacity(batch_size);
        
        for _ in 0..batch_size {
            if let Some(result) = self.process_next_transaction() {
                results.push(result);
            } else {
                break;
            }
        }
        
        results
    }

    /// Execute transaction and update state
    fn execute_transaction(&mut self, transaction: Transaction) -> ExecutionResult {
        // Get sender account state first
        let sender_state = if let Some(account) = self.accounts.get(&transaction.from) {
            (account.balance, account.nonce)
        } else {
            self.failed_count += 1;
            return ExecutionResult {
                success: false,
                gas_used: 0,
                return_data: vec![],
                error: Some("Sender account not found".to_string()),
                state_changes: vec![],
            };
        };

        let (sender_old_balance, sender_old_nonce) = sender_state;

        // Check if recipient exists
        let recipient_exists = self.accounts.contains_key(&transaction.to);

        // Create recipient account if needed
        if !recipient_exists {
            self.create_account(transaction.to.clone(), 0);
        }

        // Now perform the transaction operations
        let sender = self.accounts.get_mut(&transaction.from).unwrap();

        // Validate and debit sender
        if let Err(e) = sender.debit(transaction.total_cost()) {
            self.failed_count += 1;
            return ExecutionResult {
                success: false,
                gas_used: 0,
                return_data: vec![],
                error: Some(e.to_string()),
                state_changes: vec![],
            };
        }

        sender.increment_nonce();
        let new_sender_balance = sender.balance;
        let new_sender_nonce = sender.nonce;

        // Credit recipient
        let recipient = self.accounts.get_mut(&transaction.to).unwrap();
        recipient.credit(transaction.amount);

        let state_changes = vec![
            StateChange {
                address: transaction.from.clone(),
                old_balance: sender_old_balance,
                new_balance: new_sender_balance,
                old_nonce: sender_old_nonce,
                new_nonce: new_sender_nonce,
            }
        ];

        self.processed_count += 1;

        ExecutionResult {
            success: true,
            gas_used: transaction.gas_limit, // Simplified: assume all gas is used
            return_data: vec![],
            error: None,
            state_changes,
        }
    }

    /// Get processor statistics
    pub fn stats(&self) -> ProcessorStats {
        ProcessorStats {
            processed_count: self.processed_count,
            failed_count: self.failed_count,
            success_rate: if self.processed_count + self.failed_count > 0 {
                self.processed_count as f64 / (self.processed_count + self.failed_count) as f64
            } else {
                0.0
            },
            mempool_stats: self.mempool.stats(),
            account_count: self.accounts.len(),
        }
    }
}

/// Transaction processor statistics
#[derive(Debug, Clone)]
pub struct ProcessorStats {
    pub processed_count: u64,
    pub failed_count: u64,
    pub success_rate: f64,
    pub mempool_stats: MempoolStats,
    pub account_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_creation() {
        let tx = Transaction::new(
            "alice".to_string(),
            "bob".to_string(),
            100,
            5,
            0,
            21000,
            20,
            vec![],
        );

        assert_eq!(tx.from, "alice");
        assert_eq!(tx.to, "bob");
        assert_eq!(tx.amount, 100);
        assert_eq!(tx.fee, 5);
        assert_eq!(tx.total_cost(), 100 + 5 + (21000 * 20));
        assert!(tx.validate().is_ok());
    }

    #[test]
    fn test_transaction_validation() {
        let mut tx = Transaction::new(
            "".to_string(), // Empty sender
            "bob".to_string(),
            100,
            5,
            0,
            21000,
            20,
            vec![],
        );

        assert!(tx.validate().is_err());

        tx.from = "alice".to_string();
        tx.hash = tx.calculate_hash(); // Recalculate hash
        assert!(tx.validate().is_ok());
    }

    #[test]
    fn test_account_state() {
        let mut account = AccountState::new("alice".to_string(), 1000);

        assert!(account.can_pay(500));
        assert!(!account.can_pay(1500));

        account.debit(300).unwrap();
        assert_eq!(account.balance, 700);

        account.credit(200);
        assert_eq!(account.balance, 900);

        account.increment_nonce();
        assert_eq!(account.nonce, 1);
    }

    #[test]
    fn test_mempool_basic() {
        let mut mempool = Mempool::new(10, Duration::from_secs(300), 1);

        let tx = Transaction::new(
            "alice".to_string(),
            "bob".to_string(),
            100,
            5,
            0,
            21000,
            20,
            vec![],
        );

        assert!(mempool.add_transaction(tx).is_ok());
        assert_eq!(mempool.len(), 1);

        let pending = mempool.pop_transaction().unwrap();
        assert_eq!(pending.transaction.amount, 100);
        assert_eq!(mempool.len(), 0);
    }

    #[test]
    fn test_mempool_priority() {
        let mut mempool = Mempool::new(10, Duration::from_secs(300), 1);

        let low_priority = Transaction::new(
            "alice".to_string(),
            "bob".to_string(),
            100,
            1, // Low fee
            0,
            21000,
            1, // Low gas price
            vec![],
        );

        let high_priority = Transaction::new(
            "charlie".to_string(),
            "dave".to_string(),
            50,
            10, // High fee
            0,
            21000,
            50, // High gas price
            vec![],
        );

        mempool.add_transaction(low_priority).unwrap();
        mempool.add_transaction(high_priority).unwrap();

        // High priority should come first
        let first = mempool.pop_transaction().unwrap();
        assert_eq!(first.transaction.from, "charlie");

        let second = mempool.pop_transaction().unwrap();
        assert_eq!(second.transaction.from, "alice");
    }

    #[test]
    fn test_transaction_processor() {
        let mut processor = TransactionProcessor::new(100, Duration::from_secs(300), 1);

        // Create accounts with sufficient balance
        processor.create_account("alice".to_string(), 50000);
        processor.create_account("bob".to_string(), 500);

        let tx = Transaction::new(
            "alice".to_string(),
            "bob".to_string(),
            100,
            5,
            0, // Nonce 0
            1000, // Lower gas limit
            2,    // Lower gas price
            vec![],
        );

        // Submit transaction
        assert!(processor.submit_transaction(tx).is_ok());

        // Process transaction
        let result = processor.process_next_transaction().unwrap();
        assert!(result.success);

        // Verify state changes
        let alice = processor.get_account(&"alice".to_string()).unwrap();
        let bob = processor.get_account(&"bob".to_string()).unwrap();

        assert_eq!(alice.nonce, 1);
        assert_eq!(alice.balance, 50000 - 100 - 5 - (1000 * 2)); // amount + fee + gas
        assert_eq!(bob.balance, 500 + 100); // original + received amount
    }

    #[test]
    fn test_processor_batch_processing() {
        let mut processor = TransactionProcessor::new(100, Duration::from_secs(300), 1);

        processor.create_account("alice".to_string(), 50000);
        processor.create_account("bob".to_string(), 0);

        // Submit multiple transactions
        for i in 0..5 {
            let tx = Transaction::new(
                "alice".to_string(),
                "bob".to_string(),
                100,
                1,
                i, // Different nonce for each
                1000, // Lower gas limit
                1,    // Lower gas price
                vec![],
            );
            processor.submit_transaction(tx).unwrap();
        }

        // Process batch
        let results = processor.process_batch(3);
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.success));

        // Verify remaining transactions
        let remaining_results = processor.process_batch(5);
        assert_eq!(remaining_results.len(), 2);
    }

    #[test]
    fn test_insufficient_balance() {
        let mut processor = TransactionProcessor::new(100, Duration::from_secs(300), 1);
        processor.create_account("alice".to_string(), 100);

        let expensive_tx = Transaction::new(
            "alice".to_string(),
            "bob".to_string(),
            50, // Reasonable amount
            5,
            0,
            100, // Lower gas limit
            1,   // Lower gas price 
            vec![],
        );

        assert!(processor.submit_transaction(expensive_tx).is_err());
    }

    #[test]
    fn test_mempool_stats() {
        let mut mempool = Mempool::new(10, Duration::from_secs(300), 5);

        let tx = Transaction::new(
            "alice".to_string(),
            "bob".to_string(),
            100,
            10,
            0,
            21000,
            25,
            vec![],
        );

        mempool.add_transaction(tx).unwrap();

        let stats = mempool.stats();
        assert_eq!(stats.size, 1);
        assert_eq!(stats.max_size, 10);
        assert_eq!(stats.total_fees, 10);
        assert_eq!(stats.avg_gas_price, 25);
        assert_eq!(stats.min_gas_price, 5);
    }
}

