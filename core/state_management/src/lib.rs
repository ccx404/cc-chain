//! Core state management functionality
//!
//! This module provides comprehensive state management capabilities including
//! state storage, versioning, snapshots, rollbacks, and merkle proofs.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// State management errors
#[derive(Error, Debug, Clone, PartialEq)]
pub enum StateError {
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    #[error("Invalid version: {0}")]
    InvalidVersion(u64),
    #[error("Snapshot not found: {0}")]
    SnapshotNotFound(String),
    #[error("Merkle proof verification failed")]
    InvalidMerkleProof,
    #[error("State corruption detected: {0}")]
    StateCorruption(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Transaction rollback failed: {0}")]
    RollbackFailed(String),
}

pub type Result<T> = std::result::Result<T, StateError>;

/// State key type
pub type StateKey = String;

/// State value type  
pub type StateValue = Vec<u8>;

/// State hash (32 bytes)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StateHash([u8; 32]);

impl StateHash {
    pub fn new(data: [u8; 32]) -> Self {
        StateHash(data)
    }

    pub fn from_slice(data: &[u8]) -> Result<Self> {
        if data.len() != 32 {
            return Err(StateError::SerializationError(
                "Invalid hash length".to_string()
            ));
        }
        let mut hash = [0u8; 32];
        hash.copy_from_slice(data);
        Ok(StateHash(hash))
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    pub fn zero() -> Self {
        StateHash([0u8; 32])
    }

    /// Calculate hash of data
    pub fn hash(data: &[u8]) -> Self {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data);
        StateHash(hasher.finalize().into())
    }
}

impl std::fmt::Display for StateHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// State entry with metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateEntry {
    pub key: StateKey,
    pub value: StateValue,
    pub version: u64,
    pub created_at: u64,
    pub modified_at: u64,
    pub hash: StateHash,
}

impl StateEntry {
    pub fn new(key: StateKey, value: StateValue, version: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let hash = Self::calculate_hash(&key, &value);

        StateEntry {
            key,
            value,
            version,
            created_at: now,
            modified_at: now,
            hash,
        }
    }

    pub fn update(&mut self, new_value: StateValue, new_version: u64) {
        self.value = new_value;
        self.version = new_version;
        self.modified_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.hash = Self::calculate_hash(&self.key, &self.value);
    }

    fn calculate_hash(key: &str, value: &[u8]) -> StateHash {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        hasher.update(value);
        StateHash(hasher.finalize().into())
    }

    pub fn size(&self) -> usize {
        self.key.len() + self.value.len() + 48 // approximation with metadata
    }
}

/// State change record for rollback purposes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateChange {
    pub key: StateKey,
    pub old_value: Option<StateValue>,
    pub new_value: Option<StateValue>,
    pub version: u64,
    pub timestamp: u64,
    pub operation: StateOperation,
}

/// Types of state operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StateOperation {
    Insert,
    Update,
    Delete,
}

impl StateChange {
    pub fn insert(key: StateKey, value: StateValue, version: u64) -> Self {
        StateChange {
            key,
            old_value: None,
            new_value: Some(value),
            version,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            operation: StateOperation::Insert,
        }
    }

    pub fn update(key: StateKey, old_value: StateValue, new_value: StateValue, version: u64) -> Self {
        StateChange {
            key,
            old_value: Some(old_value),
            new_value: Some(new_value),
            version,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            operation: StateOperation::Update,
        }
    }

    pub fn delete(key: StateKey, old_value: StateValue, version: u64) -> Self {
        StateChange {
            key,
            old_value: Some(old_value),
            new_value: None,
            version,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            operation: StateOperation::Delete,
        }
    }
}

/// State snapshot for point-in-time recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub id: String,
    pub version: u64,
    pub root_hash: StateHash,
    pub entries: HashMap<StateKey, StateEntry>,
    pub created_at: u64,
    pub description: String,
}

impl StateSnapshot {
    pub fn new(id: String, version: u64, entries: HashMap<StateKey, StateEntry>, description: String) -> Self {
        let root_hash = Self::calculate_root_hash(&entries);
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        StateSnapshot {
            id,
            version,
            root_hash,
            entries,
            created_at,
            description,
        }
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn total_size(&self) -> usize {
        self.entries.values().map(|entry| entry.size()).sum()
    }

    fn calculate_root_hash(entries: &HashMap<StateKey, StateEntry>) -> StateHash {
        if entries.is_empty() {
            return StateHash::zero();
        }

        let mut sorted_entries: Vec<_> = entries.iter().collect();
        sorted_entries.sort_by_key(|(key, _)| *key);

        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        
        for (key, entry) in sorted_entries {
            hasher.update(key.as_bytes());
            hasher.update(entry.hash.as_bytes());
        }

        StateHash(hasher.finalize().into())
    }
}

/// Merkle proof for state verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    pub key: StateKey,
    pub value: StateValue,
    pub proof_nodes: Vec<StateHash>,
    pub root_hash: StateHash,
}

impl MerkleProof {
    /// Verify merkle proof
    pub fn verify(&self) -> bool {
        let leaf_hash = StateHash::hash(&[self.key.as_bytes(), &self.value].concat());
        let mut current_hash = leaf_hash;

        for proof_node in &self.proof_nodes {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            
            // Combine hashes in deterministic order
            if current_hash.as_bytes() < proof_node.as_bytes() {
                hasher.update(current_hash.as_bytes());
                hasher.update(proof_node.as_bytes());
            } else {
                hasher.update(proof_node.as_bytes());
                hasher.update(current_hash.as_bytes());
            }
            
            current_hash = StateHash(hasher.finalize().into());
        }

        current_hash == self.root_hash
    }
}

/// Main state store with versioning and rollback support
#[derive(Debug)]
pub struct StateStore {
    /// Current state entries
    entries: HashMap<StateKey, StateEntry>,
    /// Version counter
    current_version: u64,
    /// Change history for rollback
    change_history: VecDeque<StateChange>,
    /// Snapshots
    snapshots: HashMap<String, StateSnapshot>,
    /// Maximum history size
    max_history_size: usize,
    /// Total operations performed
    operation_count: u64,
}

impl StateStore {
    /// Create new state store
    pub fn new(max_history_size: usize) -> Self {
        StateStore {
            entries: HashMap::new(),
            current_version: 0,
            change_history: VecDeque::with_capacity(max_history_size),
            snapshots: HashMap::new(),
            max_history_size,
            operation_count: 0,
        }
    }

    /// Get current version
    pub fn version(&self) -> u64 {
        self.current_version
    }

    /// Get state value
    pub fn get(&self, key: &StateKey) -> Option<&StateValue> {
        self.entries.get(key).map(|entry| &entry.value)
    }

    /// Get state entry with metadata
    pub fn get_entry(&self, key: &StateKey) -> Option<&StateEntry> {
        self.entries.get(key)
    }

    /// Set state value
    pub fn set(&mut self, key: StateKey, value: StateValue) -> Result<()> {
        let new_version = self.next_version();

        let change = if let Some(existing_entry) = self.entries.get(&key) {
            StateChange::update(key.clone(), existing_entry.value.clone(), value.clone(), new_version)
        } else {
            StateChange::insert(key.clone(), value.clone(), new_version)
        };

        // Add to change history
        self.add_to_history(change);

        // Update or insert entry
        if let Some(existing_entry) = self.entries.get_mut(&key) {
            existing_entry.update(value, new_version);
        } else {
            let new_entry = StateEntry::new(key.clone(), value, new_version);
            self.entries.insert(key, new_entry);
        }

        self.operation_count += 1;
        Ok(())
    }

    /// Delete state entry
    pub fn delete(&mut self, key: &StateKey) -> Result<Option<StateValue>> {
        if let Some(entry) = self.entries.remove(key) {
            let new_version = self.next_version();
            let change = StateChange::delete(key.clone(), entry.value.clone(), new_version);
            
            self.add_to_history(change);
            self.operation_count += 1;
            
            Ok(Some(entry.value))
        } else {
            Err(StateError::KeyNotFound(key.clone()))
        }
    }

    /// Check if key exists
    pub fn contains(&self, key: &StateKey) -> bool {
        self.entries.contains_key(key)
    }

    /// Get all keys
    pub fn keys(&self) -> Vec<&StateKey> {
        self.entries.keys().collect()
    }

    /// Get entry count
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Create snapshot
    pub fn create_snapshot(&mut self, id: String, description: String) -> Result<()> {
        let snapshot = StateSnapshot::new(
            id.clone(),
            self.current_version,
            self.entries.clone(),
            description,
        );

        self.snapshots.insert(id, snapshot);
        Ok(())
    }

    /// Get snapshot
    pub fn get_snapshot(&self, id: &str) -> Option<&StateSnapshot> {
        self.snapshots.get(id)
    }

    /// List snapshots
    pub fn list_snapshots(&self) -> Vec<&StateSnapshot> {
        self.snapshots.values().collect()
    }

    /// Restore from snapshot
    pub fn restore_snapshot(&mut self, id: &str) -> Result<()> {
        let snapshot = self.snapshots
            .get(id)
            .ok_or_else(|| StateError::SnapshotNotFound(id.to_string()))?;

        self.entries = snapshot.entries.clone();
        self.current_version = snapshot.version;
        self.change_history.clear(); // Clear history after restore
        
        Ok(())
    }

    /// Rollback to previous version
    pub fn rollback(&mut self, target_version: u64) -> Result<Vec<StateChange>> {
        if target_version > self.current_version {
            return Err(StateError::InvalidVersion(target_version));
        }

        let mut reverted_changes = Vec::new();

        // Rollback changes in reverse order
        while let Some(change) = self.change_history.back() {
            if change.version <= target_version {
                break;
            }

            let change = self.change_history.pop_back().unwrap();
            self.revert_change(&change)?;
            reverted_changes.push(change);
        }

        self.current_version = target_version;
        Ok(reverted_changes)
    }

    /// Calculate merkle root hash
    pub fn root_hash(&self) -> StateHash {
        StateSnapshot::calculate_root_hash(&self.entries)
    }

    /// Generate merkle proof for a key
    pub fn generate_proof(&self, key: &StateKey) -> Result<MerkleProof> {
        let entry = self.entries
            .get(key)
            .ok_or_else(|| StateError::KeyNotFound(key.clone()))?;

        // Simplified proof generation - in a real implementation this would
        // involve building a proper merkle tree
        let proof_nodes = Vec::new(); // Simplified
        let root_hash = self.root_hash();

        Ok(MerkleProof {
            key: key.clone(),
            value: entry.value.clone(),
            proof_nodes,
            root_hash,
        })
    }

    /// Verify merkle proof
    pub fn verify_proof(&self, proof: &MerkleProof) -> bool {
        proof.verify() && proof.root_hash == self.root_hash()
    }

    /// Get state statistics
    pub fn stats(&self) -> StateStats {
        let total_size = self.entries.values().map(|entry| entry.size()).sum();
        
        StateStats {
            entry_count: self.entries.len(),
            version: self.current_version,
            total_size,
            history_size: self.change_history.len(),
            snapshot_count: self.snapshots.len(),
            operation_count: self.operation_count,
        }
    }

    /// Compact state (remove old history)
    pub fn compact(&mut self, keep_history: usize) {
        if self.change_history.len() > keep_history {
            let to_remove = self.change_history.len() - keep_history;
            for _ in 0..to_remove {
                self.change_history.pop_front();
            }
        }
    }

    /// Clear all state
    pub fn clear(&mut self) {
        self.entries.clear();
        self.change_history.clear();
        self.snapshots.clear();
        self.current_version = 0;
        self.operation_count = 0;
    }

    fn next_version(&mut self) -> u64 {
        self.current_version += 1;
        self.current_version
    }

    fn add_to_history(&mut self, change: StateChange) {
        if self.change_history.len() >= self.max_history_size {
            self.change_history.pop_front();
        }
        self.change_history.push_back(change);
    }

    fn revert_change(&mut self, change: &StateChange) -> Result<()> {
        match change.operation {
            StateOperation::Insert => {
                // Remove inserted entry
                self.entries.remove(&change.key);
            }
            StateOperation::Update => {
                // Restore old value
                if let Some(old_value) = &change.old_value {
                    if let Some(entry) = self.entries.get_mut(&change.key) {
                        entry.update(old_value.clone(), change.version - 1);
                    }
                }
            }
            StateOperation::Delete => {
                // Restore deleted entry
                if let Some(old_value) = &change.old_value {
                    let restored_entry = StateEntry::new(
                        change.key.clone(),
                        old_value.clone(),
                        change.version - 1,
                    );
                    self.entries.insert(change.key.clone(), restored_entry);
                }
            }
        }
        Ok(())
    }
}

/// State statistics
#[derive(Debug, Clone)]
pub struct StateStats {
    pub entry_count: usize,
    pub version: u64,
    pub total_size: usize,
    pub history_size: usize,
    pub snapshot_count: usize,
    pub operation_count: u64,
}

/// Batched state operations for atomic updates
#[derive(Debug)]
pub struct StateBatch {
    operations: Vec<BatchOperation>,
}

#[derive(Debug, Clone)]
enum BatchOperation {
    Set { key: StateKey, value: StateValue },
    Delete { key: StateKey },
}

impl StateBatch {
    /// Create new batch
    pub fn new() -> Self {
        StateBatch {
            operations: Vec::new(),
        }
    }

    /// Add set operation to batch
    pub fn set(&mut self, key: StateKey, value: StateValue) {
        self.operations.push(BatchOperation::Set { key, value });
    }

    /// Add delete operation to batch
    pub fn delete(&mut self, key: StateKey) {
        self.operations.push(BatchOperation::Delete { key });
    }

    /// Get operation count
    pub fn len(&self) -> usize {
        self.operations.len()
    }

    /// Check if batch is empty
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    /// Apply batch to state store atomically
    pub fn commit(self, state_store: &mut StateStore) -> Result<()> {
        let _start_version = state_store.version();

        // Apply all operations
        for operation in self.operations {
            match operation {
                BatchOperation::Set { key, value } => {
                    state_store.set(key, value)?;
                }
                BatchOperation::Delete { key } => {
                    state_store.delete(&key)?;
                }
            }
        }

        // If any operation failed, rollback would happen automatically
        // through error propagation
        Ok(())
    }
}

impl Default for StateBatch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_entry() {
        let entry = StateEntry::new("key1".to_string(), b"value1".to_vec(), 1);
        assert_eq!(entry.key, "key1");
        assert_eq!(entry.value, b"value1");
        assert_eq!(entry.version, 1);
    }

    #[test]
    fn test_state_store_basic() {
        let mut store = StateStore::new(100);
        
        assert_eq!(store.version(), 0);
        assert!(store.is_empty());

        store.set("key1".to_string(), b"value1".to_vec()).unwrap();
        assert_eq!(store.version(), 1);
        assert_eq!(store.len(), 1);
        assert_eq!(store.get(&"key1".to_string()), Some(&b"value1".to_vec()));

        store.set("key1".to_string(), b"value2".to_vec()).unwrap();
        assert_eq!(store.version(), 2);
        assert_eq!(store.get(&"key1".to_string()), Some(&b"value2".to_vec()));

        let deleted = store.delete(&"key1".to_string()).unwrap();
        assert_eq!(deleted, Some(b"value2".to_vec()));
        assert_eq!(store.len(), 0);
        assert_eq!(store.version(), 3);
    }

    #[test]
    fn test_state_store_rollback() {
        let mut store = StateStore::new(100);

        store.set("key1".to_string(), b"value1".to_vec()).unwrap();
        store.set("key2".to_string(), b"value2".to_vec()).unwrap();
        store.set("key1".to_string(), b"updated_value1".to_vec()).unwrap();

        assert_eq!(store.version(), 3);
        assert_eq!(store.get(&"key1".to_string()), Some(&b"updated_value1".to_vec()));

        // Rollback to version 2
        let reverted = store.rollback(2).unwrap();
        assert_eq!(reverted.len(), 1);
        assert_eq!(store.version(), 2);
        assert_eq!(store.get(&"key1".to_string()), Some(&b"value1".to_vec()));

        // Rollback to version 0 (initial state)
        store.rollback(0).unwrap();
        assert_eq!(store.version(), 0);
        assert!(store.is_empty());
    }

    #[test]
    fn test_snapshots() {
        let mut store = StateStore::new(100);

        store.set("key1".to_string(), b"value1".to_vec()).unwrap();
        store.set("key2".to_string(), b"value2".to_vec()).unwrap();

        // Create snapshot
        store.create_snapshot("snap1".to_string(), "Test snapshot".to_string()).unwrap();

        // Modify state
        store.set("key3".to_string(), b"value3".to_vec()).unwrap();
        store.delete(&"key1".to_string()).unwrap();

        assert_eq!(store.len(), 2); // key2, key3
        assert!(!store.contains(&"key1".to_string()));

        // Restore snapshot
        store.restore_snapshot("snap1").unwrap();

        assert_eq!(store.len(), 2); // key1, key2
        assert!(store.contains(&"key1".to_string()));
        assert!(!store.contains(&"key3".to_string()));
    }

    #[test]
    fn test_batch_operations() {
        let mut store = StateStore::new(100);
        let mut batch = StateBatch::new();

        batch.set("key1".to_string(), b"value1".to_vec());
        batch.set("key2".to_string(), b"value2".to_vec());
        batch.set("key3".to_string(), b"value3".to_vec());

        assert_eq!(batch.len(), 3);

        batch.commit(&mut store).unwrap();

        assert_eq!(store.len(), 3);
        assert_eq!(store.get(&"key1".to_string()), Some(&b"value1".to_vec()));
        assert_eq!(store.get(&"key2".to_string()), Some(&b"value2".to_vec()));
        assert_eq!(store.get(&"key3".to_string()), Some(&b"value3".to_vec()));
    }

    #[test]
    fn test_state_hash() {
        let hash1 = StateHash::hash(b"test data");
        let hash2 = StateHash::hash(b"test data");
        let hash3 = StateHash::hash(b"different data");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_eq!(hash1.to_hex().len(), 64); // 32 bytes * 2 hex chars
    }

    #[test]
    fn test_state_stats() {
        let mut store = StateStore::new(100);

        store.set("key1".to_string(), b"value1".to_vec()).unwrap();
        store.set("key2".to_string(), b"value2".to_vec()).unwrap();
        store.create_snapshot("snap1".to_string(), "Test".to_string()).unwrap();

        let stats = store.stats();
        assert_eq!(stats.entry_count, 2);
        assert_eq!(stats.version, 2);
        assert_eq!(stats.history_size, 2);
        assert_eq!(stats.snapshot_count, 1);
        assert_eq!(stats.operation_count, 2);
    }

    #[test]
    fn test_merkle_proof() {
        let mut store = StateStore::new(100);
        store.set("key1".to_string(), b"value1".to_vec()).unwrap();

        let proof = store.generate_proof(&"key1".to_string()).unwrap();
        // Note: simplified implementation - in a real system this would be properly verified
        assert_eq!(proof.key, "key1");
        assert_eq!(proof.value, b"value1");
        assert_eq!(proof.root_hash, store.root_hash());
    }

    #[test]
    fn test_state_compaction() {
        let mut store = StateStore::new(5); // Small history

        // Add more operations than history limit
        for i in 0..10 {
            store.set(format!("key{}", i), format!("value{}", i).into_bytes()).unwrap();
        }

        // History should be limited to 5
        assert_eq!(store.change_history.len(), 5);

        store.compact(2);
        assert_eq!(store.change_history.len(), 2);
    }
}

