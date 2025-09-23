use crate::crypto::{hash, CCPublicKey, Hash};
use crate::error::Result;
use crate::transaction::Transaction;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Account state in the blockchain
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Account {
    /// Account balance
    pub balance: u64,
    /// Transaction nonce (to prevent replay attacks)
    pub nonce: u64,
    /// Storage root for smart contract data (future extension)
    pub storage_root: Hash,
    /// Code hash for smart contracts (future extension)
    pub code_hash: Hash,
}

impl Default for Account {
    fn default() -> Self {
        Self {
            balance: 0,
            nonce: 0,
            storage_root: [0u8; 32],
            code_hash: [0u8; 32],
        }
    }
}

impl Account {
    /// Create a new account with initial balance
    pub fn new(balance: u64) -> Self {
        Self {
            balance,
            nonce: 0,
            storage_root: [0u8; 32],
            code_hash: [0u8; 32],
        }
    }

    /// Check if account can afford a transaction
    pub fn can_afford(&self, amount: u64, fee: u64) -> bool {
        self.balance >= amount.saturating_add(fee)
    }

    /// Update account after transaction
    pub fn apply_transaction(&mut self, tx: &Transaction, is_sender: bool) -> Result<()> {
        if is_sender {
            // Sender: deduct amount + fee, increment nonce
            let total_cost = tx.amount.saturating_add(tx.fee);
            if self.balance < total_cost {
                return Err(crate::CCError::State("Insufficient balance".to_string()));
            }

            if tx.nonce != self.nonce {
                return Err(crate::CCError::State("Invalid nonce".to_string()));
            }

            self.balance -= total_cost;
            self.nonce += 1;
        } else {
            // Recipient: add amount
            self.balance = self.balance.saturating_add(tx.amount);
        }

        Ok(())
    }
}

/// State manager for the blockchain
#[derive(Debug)]
pub struct StateManager {
    /// Current state (accounts)
    accounts: dashmap::DashMap<CCPublicKey, Account>,
    /// State cache for faster access
    #[allow(dead_code)]
    cache: lru::LruCache<Hash, HashMap<CCPublicKey, Account>>,
    /// Validators and their stakes
    validators: dashmap::DashMap<CCPublicKey, u64>,
    /// Total supply of tokens
    total_supply: parking_lot::RwLock<u64>,
}

impl StateManager {
    /// Create new state manager
    pub fn new() -> Self {
        Self {
            accounts: dashmap::DashMap::new(),
            cache: lru::LruCache::new(std::num::NonZeroUsize::new(1000).unwrap()),
            validators: dashmap::DashMap::new(),
            total_supply: parking_lot::RwLock::new(0),
        }
    }

    /// Initialize genesis state
    pub fn initialize_genesis(&self, genesis_accounts: Vec<(CCPublicKey, u64)>) -> Result<Hash> {
        let mut total = 0u64;

        for (pubkey, balance) in genesis_accounts {
            let account = Account::new(balance);
            self.accounts.insert(pubkey, account);
            total = total.saturating_add(balance);
        }

        *self.total_supply.write() = total;

        Ok(self.compute_state_root())
    }

    /// Get account state
    pub fn get_account(&self, pubkey: &CCPublicKey) -> Account {
        self.accounts
            .get(pubkey)
            .map(|entry| entry.value().clone())
            .unwrap_or_default()
    }

    /// Set account state
    pub fn set_account(&self, pubkey: CCPublicKey, account: Account) {
        self.accounts.insert(pubkey, account);
    }

    /// Apply a single transaction to the state
    pub fn apply_transaction(&self, tx: &Transaction) -> Result<()> {
        // Skip coinbase transactions (they mint new tokens)
        if tx.is_coinbase() {
            let mut recipient_account = self.get_account(&tx.to);
            recipient_account.balance = recipient_account.balance.saturating_add(tx.amount);
            self.set_account(tx.to.clone(), recipient_account);

            // Update total supply
            *self.total_supply.write() = self.total_supply.read().saturating_add(tx.amount);
            return Ok(());
        }

        // Get sender and recipient accounts
        let mut sender_account = self.get_account(&tx.from);
        let mut recipient_account = self.get_account(&tx.to);

        // Apply transaction to sender
        sender_account.apply_transaction(tx, true)?;

        // Apply transaction to recipient
        recipient_account.apply_transaction(tx, false)?;

        // Update accounts
        self.set_account(tx.from.clone(), sender_account);
        self.set_account(tx.to.clone(), recipient_account);

        Ok(())
    }

    /// Apply multiple transactions (for block processing)
    pub fn apply_transactions(&self, transactions: &[Transaction]) -> Result<Hash> {
        for tx in transactions {
            self.apply_transaction(tx)?;
        }

        Ok(self.compute_state_root())
    }

    /// Compute merkle root of current state
    pub fn compute_state_root(&self) -> Hash {
        let mut account_hashes = Vec::new();

        for entry in self.accounts.iter() {
            let pubkey = entry.key();
            let account = entry.value();

            // Create deterministic hash for this account
            let account_data =
                bincode::serialize(&(pubkey, account)).expect("Serialization should not fail");
            account_hashes.push(hash(&account_data));
        }

        // Sort for deterministic ordering
        account_hashes.sort();

        // Build merkle tree
        if account_hashes.is_empty() {
            [0u8; 32]
        } else {
            let merkle_tree = crate::crypto::MerkleTree::build(&account_hashes);
            merkle_tree.root()
        }
    }

    /// Get current total supply
    pub fn get_total_supply(&self) -> u64 {
        *self.total_supply.read()
    }

    /// Add validator
    pub fn add_validator(&self, pubkey: CCPublicKey, stake: u64) {
        self.validators.insert(pubkey, stake);
    }

    /// Remove validator
    pub fn remove_validator(&self, pubkey: &CCPublicKey) {
        self.validators.remove(pubkey);
    }

    /// Get validator stake
    pub fn get_validator_stake(&self, pubkey: &CCPublicKey) -> Option<u64> {
        self.validators.get(pubkey).map(|entry| *entry.value())
    }

    /// Get all validators
    pub fn get_validators(&self) -> Vec<(CCPublicKey, u64)> {
        self.validators
            .iter()
            .map(|entry| (entry.key().clone(), *entry.value()))
            .collect()
    }

    /// Get total validator stake
    pub fn get_total_validator_stake(&self) -> u64 {
        self.validators.iter().map(|entry| *entry.value()).sum()
    }

    /// Check if public key is a validator
    pub fn is_validator(&self, pubkey: &CCPublicKey) -> bool {
        self.validators.contains_key(pubkey)
    }

    /// Validate transaction against current state
    pub fn validate_transaction(&self, tx: &Transaction) -> Result<()> {
        // Basic transaction validation
        tx.validate()?;

        // Skip further validation for coinbase transactions
        if tx.is_coinbase() {
            return Ok(());
        }

        // Check sender account
        let sender_account = self.get_account(&tx.from);

        // Check nonce
        if tx.nonce != sender_account.nonce {
            return Err(crate::CCError::Transaction(format!(
                "Invalid nonce: expected {}, got {}",
                sender_account.nonce, tx.nonce
            )));
        }

        // Check balance
        if !sender_account.can_afford(tx.amount, tx.fee) {
            return Err(crate::CCError::Transaction(
                "Insufficient balance".to_string(),
            ));
        }

        Ok(())
    }

    /// Create a snapshot of current state for rollback
    pub fn create_snapshot(&self) -> StateSnapshot {
        let accounts: HashMap<CCPublicKey, Account> = self
            .accounts
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();

        let validators: HashMap<CCPublicKey, u64> = self
            .validators
            .iter()
            .map(|entry| (entry.key().clone(), *entry.value()))
            .collect();

        StateSnapshot::new(
            accounts,
            validators,
            *self.total_supply.read(),
            0, // Block height would come from blockchain context
        )
    }

    /// Restore state from snapshot
    pub fn restore_snapshot(&self, snapshot: StateSnapshot) {
        self.accounts.clear();
        self.validators.clear();

        for (pubkey, account) in snapshot.accounts {
            self.accounts.insert(pubkey, account);
        }

        for (pubkey, stake) in snapshot.validators {
            self.validators.insert(pubkey, stake);
        }

        *self.total_supply.write() = snapshot.total_supply;
    }
}

/// State snapshot for rollback functionality
#[derive(Debug, Clone)]
pub struct StateSnapshot {
    accounts: HashMap<CCPublicKey, Account>,
    validators: HashMap<CCPublicKey, u64>,
    total_supply: u64,
    timestamp: u64,
    block_height: u64,
}

impl StateSnapshot {
    /// Create a new state snapshot
    pub fn new(
        accounts: HashMap<CCPublicKey, Account>,
        validators: HashMap<CCPublicKey, u64>,
        total_supply: u64,
        block_height: u64,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            accounts,
            validators,
            total_supply,
            timestamp,
            block_height,
        }
    }

    /// Get snapshot metadata
    pub fn metadata(&self) -> (u64, u64, usize, usize) {
        (
            self.timestamp,
            self.block_height,
            self.accounts.len(),
            self.validators.len(),
        )
    }
}

/// Enhanced state manager with advanced features
impl StateManager {
    /// Restore state from snapshot
    pub fn restore_from_snapshot(&self, snapshot: &StateSnapshot) {
        // Clear current state
        self.accounts.clear();
        self.validators.clear();

        // Restore accounts
        for (pubkey, account) in &snapshot.accounts {
            self.accounts.insert(pubkey.clone(), account.clone());
        }

        // Restore validators
        for (pubkey, stake) in &snapshot.validators {
            self.validators.insert(pubkey.clone(), *stake);
        }

        // Restore total supply
        *self.total_supply.write() = snapshot.total_supply;
    }

    /// Apply transactions with atomic rollback on failure
    pub fn apply_transactions_atomic(&self, transactions: &[crate::transaction::Transaction]) -> Result<Hash> {
        // Create snapshot before applying transactions
        let snapshot = self.create_snapshot();

        match self.apply_transactions(transactions) {
            Ok(state_root) => Ok(state_root),
            Err(err) => {
                // Rollback on error
                self.restore_from_snapshot(&snapshot);
                Err(err)
            }
        }
    }

    /// Parallel state validation for multiple transactions
    pub fn validate_transactions_parallel(&self, transactions: &[crate::transaction::Transaction]) -> Vec<Result<()>> {
        use rayon::prelude::*;
        use std::sync::Arc;

        let state_ref = Arc::new(self);
        
        transactions
            .par_iter()
            .map(|tx| state_ref.validate_transaction(tx))
            .collect()
    }

    /// Optimized account batch update
    pub fn batch_update_accounts(&self, updates: &[(CCPublicKey, Account)]) {
        for (pubkey, account) in updates {
            self.accounts.insert(pubkey.clone(), account.clone());
        }
    }

    /// Get state statistics
    pub fn get_state_stats(&self) -> StateStatistics {
        let account_count = self.accounts.len();
        let validator_count = self.validators.len();
        let total_supply = *self.total_supply.read();
        
        let total_balance: u64 = self
            .accounts
            .iter()
            .map(|entry| entry.value().balance)
            .sum();

        let total_validator_stake = self.get_total_validator_stake();

        StateStatistics {
            account_count,
            validator_count,
            total_supply,
            total_balance,
            total_validator_stake,
        }
    }

    /// Efficient state diff computation
    pub fn compute_state_diff(&self, other_snapshot: &StateSnapshot) -> StateDiff {
        let mut added_accounts = Vec::new();
        let mut modified_accounts = Vec::new();
        let mut removed_accounts = Vec::new();

        // Check for added and modified accounts
        for entry in self.accounts.iter() {
            let pubkey = entry.key();
            let account = entry.value();
            
            match other_snapshot.accounts.get(pubkey) {
                Some(old_account) => {
                    if account != old_account {
                        modified_accounts.push((pubkey.clone(), account.clone(), old_account.clone()));
                    }
                }
                None => {
                    added_accounts.push((pubkey.clone(), account.clone()));
                }
            }
        }

        // Check for removed accounts
        for pubkey in other_snapshot.accounts.keys() {
            if !self.accounts.contains_key(pubkey) {
                removed_accounts.push(pubkey.clone());
            }
        }

        StateDiff {
            added_accounts,
            modified_accounts,
            removed_accounts,
        }
    }
}

/// State statistics for monitoring
#[derive(Debug, Clone)]
pub struct StateStatistics {
    pub account_count: usize,
    pub validator_count: usize,
    pub total_supply: u64,
    pub total_balance: u64,
    pub total_validator_stake: u64,
}

/// State difference for efficient updates
#[derive(Debug, Clone)]
pub struct StateDiff {
    pub added_accounts: Vec<(CCPublicKey, Account)>,
    pub modified_accounts: Vec<(CCPublicKey, Account, Account)>, // (pubkey, new, old)
    pub removed_accounts: Vec<CCPublicKey>,
}

/// Advanced state caching layer
pub struct StateCache {
    /// LRU cache for frequently accessed accounts
    account_cache: parking_lot::Mutex<lru::LruCache<CCPublicKey, Account>>,
    /// Cache for state roots
    state_root_cache: parking_lot::Mutex<lru::LruCache<u64, Hash>>, // height -> state_root
    /// Cache statistics
    cache_stats: parking_lot::RwLock<CacheStatistics>,
}

impl StateCache {
    pub fn new(max_accounts: usize, max_state_roots: usize) -> Self {
        Self {
            account_cache: parking_lot::Mutex::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(max_accounts).unwrap()
            )),
            state_root_cache: parking_lot::Mutex::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(max_state_roots).unwrap()
            )),
            cache_stats: parking_lot::RwLock::new(CacheStatistics::default()),
        }
    }

    /// Get account from cache
    pub fn get_account(&self, pubkey: &CCPublicKey) -> Option<Account> {
        let mut cache = self.account_cache.lock();
        let result = cache.get(pubkey).cloned();
        
        let mut stats = self.cache_stats.write();
        stats.account_requests += 1;
        if result.is_some() {
            stats.account_hits += 1;
        }
        
        result
    }

    /// Put account in cache
    pub fn put_account(&self, pubkey: CCPublicKey, account: Account) {
        let mut cache = self.account_cache.lock();
        cache.put(pubkey, account);
    }

    /// Get state root from cache
    pub fn get_state_root(&self, height: u64) -> Option<Hash> {
        let mut cache = self.state_root_cache.lock();
        let result = cache.get(&height).cloned();
        
        let mut stats = self.cache_stats.write();
        stats.state_root_requests += 1;
        if result.is_some() {
            stats.state_root_hits += 1;
        }
        
        result
    }

    /// Put state root in cache
    pub fn put_state_root(&self, height: u64, state_root: Hash) {
        let mut cache = self.state_root_cache.lock();
        cache.put(height, state_root);
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStatistics {
        self.cache_stats.read().clone()
    }

    /// Clear all caches
    pub fn clear_all(&self) {
        self.account_cache.lock().clear();
        self.state_root_cache.lock().clear();
        *self.cache_stats.write() = CacheStatistics::default();
    }
}

/// Cache performance statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStatistics {
    pub account_requests: u64,
    pub account_hits: u64,
    pub state_root_requests: u64,
    pub state_root_hits: u64,
}

impl CacheStatistics {
    /// Calculate account cache hit rate
    pub fn account_hit_rate(&self) -> f64 {
        if self.account_requests == 0 {
            0.0
        } else {
            self.account_hits as f64 / self.account_requests as f64
        }
    }

    /// Calculate state root cache hit rate
    pub fn state_root_hit_rate(&self) -> f64 {
        if self.state_root_requests == 0 {
            0.0
        } else {
            self.state_root_hits as f64 / self.state_root_requests as f64
        }
    }
}
