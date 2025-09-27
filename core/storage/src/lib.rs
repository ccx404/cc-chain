//! Core storage functionality
//!
//! This module provides storage interfaces and abstractions for the CC Chain system
//! including key-value storage, blob storage, metadata management, and caching.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Storage-related errors
#[derive(Error, Debug, Clone, PartialEq)]
pub enum StorageError {
    #[error("Key not found: {key}")]
    KeyNotFound { key: String },
    #[error("Storage operation failed: {operation}")]
    OperationFailed { operation: String },
    #[error("Serialization failed: {message}")]
    SerializationFailed { message: String },
    #[error("Storage is read-only")]
    ReadOnly,
    #[error("Storage is full")]
    StorageFull,
    #[error("Invalid key: {key}")]
    InvalidKey { key: String },
    #[error("Connection failed: {details}")]
    ConnectionFailed { details: String },
    #[error("Transaction failed: {reason}")]
    TransactionFailed { reason: String },
}

pub type Result<T> = std::result::Result<T, StorageError>;

/// Storage key type
pub type StorageKey = Vec<u8>;

/// Storage value type
pub type StorageValue = Vec<u8>;

/// Storage metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetadata {
    pub key: StorageKey,
    pub size: usize,
    pub created_at: u64,
    pub modified_at: u64,
    pub access_count: u64,
    pub version: u64,
    pub content_type: Option<String>,
    pub checksum: Option<Vec<u8>>,
}

impl StorageMetadata {
    pub fn new(key: StorageKey, size: usize) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        StorageMetadata {
            key,
            size,
            created_at: now,
            modified_at: now,
            access_count: 0,
            version: 1,
            content_type: None,
            checksum: None,
        }
    }

    pub fn update(&mut self, new_size: usize) {
        self.size = new_size;
        self.modified_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.version += 1;
    }

    pub fn increment_access(&mut self) {
        self.access_count += 1;
    }

    pub fn age(&self) -> Duration {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Duration::from_secs(now.saturating_sub(self.created_at))
    }

    pub fn time_since_modification(&self) -> Duration {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Duration::from_secs(now.saturating_sub(self.modified_at))
    }
}

/// Core storage trait
pub trait Storage: Send + Sync {
    /// Get value by key
    fn get(&self, key: &StorageKey) -> Result<Option<StorageValue>>;

    /// Put key-value pair
    fn put(&mut self, key: StorageKey, value: StorageValue) -> Result<()>;

    /// Delete key
    fn delete(&mut self, key: &StorageKey) -> Result<bool>;

    /// Check if key exists
    fn contains(&self, key: &StorageKey) -> Result<bool>;

    /// List all keys
    fn keys(&self) -> Result<Vec<StorageKey>>;

    /// Get storage size in bytes
    fn size(&self) -> Result<usize>;

    /// Clear all data
    fn clear(&mut self) -> Result<()>;
}

/// Batch operation for atomic writes
#[derive(Debug, Clone)]
pub enum BatchOperation {
    Put { key: StorageKey, value: StorageValue },
    Delete { key: StorageKey },
}

/// Batch storage trait for atomic operations
pub trait BatchStorage: Storage {
    /// Execute batch of operations atomically
    fn batch(&mut self, operations: Vec<BatchOperation>) -> Result<()>;
}

/// Metadata storage trait
pub trait MetadataStorage: Storage {
    /// Get metadata for key
    fn get_metadata(&self, key: &StorageKey) -> Result<Option<StorageMetadata>>;

    /// Set metadata for key
    fn set_metadata(&mut self, metadata: StorageMetadata) -> Result<()>;

    /// List keys with metadata
    fn list_with_metadata(&self) -> Result<Vec<StorageMetadata>>;
}

/// Storage statistics
#[derive(Debug, Clone, Default)]
pub struct StorageStats {
    pub total_keys: usize,
    pub total_size_bytes: usize,
    pub get_operations: u64,
    pub put_operations: u64,
    pub delete_operations: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl StorageStats {
    pub fn cache_hit_rate(&self) -> f64 {
        let total_gets = self.cache_hits + self.cache_misses;
        if total_gets > 0 {
            self.cache_hits as f64 / total_gets as f64
        } else {
            0.0
        }
    }

    pub fn average_value_size(&self) -> f64 {
        if self.total_keys > 0 {
            self.total_size_bytes as f64 / self.total_keys as f64
        } else {
            0.0
        }
    }
}

/// In-memory storage implementation
#[derive(Debug)]
pub struct InMemoryStorage {
    data: RwLock<HashMap<StorageKey, StorageValue>>,
    metadata: RwLock<HashMap<StorageKey, StorageMetadata>>,
    stats: RwLock<StorageStats>,
    read_only: bool,
}

impl InMemoryStorage {
    /// Create new in-memory storage
    pub fn new() -> Self {
        InMemoryStorage {
            data: RwLock::new(HashMap::new()),
            metadata: RwLock::new(HashMap::new()),
            stats: RwLock::new(StorageStats::default()),
            read_only: false,
        }
    }

    /// Create read-only storage
    pub fn new_read_only(data: HashMap<StorageKey, StorageValue>) -> Self {
        let mut metadata = HashMap::new();
        for (key, value) in &data {
            metadata.insert(key.clone(), StorageMetadata::new(key.clone(), value.len()));
        }

        InMemoryStorage {
            data: RwLock::new(data),
            metadata: RwLock::new(metadata),
            stats: RwLock::new(StorageStats::default()),
            read_only: true,
        }
    }

    /// Get storage statistics
    pub fn get_stats(&self) -> StorageStats {
        self.stats.read().unwrap().clone()
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        *self.stats.write().unwrap() = StorageStats::default();
    }

    fn check_writable(&self) -> Result<()> {
        if self.read_only {
            Err(StorageError::ReadOnly)
        } else {
            Ok(())
        }
    }
}

impl Storage for InMemoryStorage {
    fn get(&self, key: &StorageKey) -> Result<Option<StorageValue>> {
        let data = self.data.read().unwrap();
        let result = data.get(key).cloned();
        
        // Update stats
        let mut stats = self.stats.write().unwrap();
        stats.get_operations += 1;
        if result.is_some() {
            stats.cache_hits += 1;
        } else {
            stats.cache_misses += 1;
        }

        // Update metadata access count
        if result.is_some() {
            if let Ok(mut metadata) = self.metadata.write() {
                if let Some(meta) = metadata.get_mut(key) {
                    meta.increment_access();
                }
            }
        }

        Ok(result)
    }

    fn put(&mut self, key: StorageKey, value: StorageValue) -> Result<()> {
        self.check_writable()?;

        let value_size = value.len();
        let mut data = self.data.write().unwrap();
        let mut metadata = self.metadata.write().unwrap();
        let mut stats = self.stats.write().unwrap();

        let _is_update = data.contains_key(&key);
        
        if let Some(old_meta) = metadata.get(&key) {
            stats.total_size_bytes = stats.total_size_bytes
                .saturating_sub(old_meta.size)
                .saturating_add(value_size);
        } else {
            stats.total_keys += 1;
            stats.total_size_bytes += value_size;
        }

        data.insert(key.clone(), value);
        
        if let Some(existing_meta) = metadata.get_mut(&key) {
            existing_meta.update(value_size);
        } else {
            metadata.insert(key.clone(), StorageMetadata::new(key, value_size));
        }

        stats.put_operations += 1;

        Ok(())
    }

    fn delete(&mut self, key: &StorageKey) -> Result<bool> {
        self.check_writable()?;

        let mut data = self.data.write().unwrap();
        let mut metadata = self.metadata.write().unwrap();
        let mut stats = self.stats.write().unwrap();

        let existed = if let Some(meta) = metadata.remove(key) {
            data.remove(key);
            stats.total_keys = stats.total_keys.saturating_sub(1);
            stats.total_size_bytes = stats.total_size_bytes.saturating_sub(meta.size);
            true
        } else {
            false
        };

        stats.delete_operations += 1;
        Ok(existed)
    }

    fn contains(&self, key: &StorageKey) -> Result<bool> {
        let data = self.data.read().unwrap();
        Ok(data.contains_key(key))
    }

    fn keys(&self) -> Result<Vec<StorageKey>> {
        let data = self.data.read().unwrap();
        Ok(data.keys().cloned().collect())
    }

    fn size(&self) -> Result<usize> {
        let stats = self.stats.read().unwrap();
        Ok(stats.total_size_bytes)
    }

    fn clear(&mut self) -> Result<()> {
        self.check_writable()?;

        let mut data = self.data.write().unwrap();
        let mut metadata = self.metadata.write().unwrap();
        let mut stats = self.stats.write().unwrap();

        data.clear();
        metadata.clear();
        stats.total_keys = 0;
        stats.total_size_bytes = 0;

        Ok(())
    }
}

impl BatchStorage for InMemoryStorage {
    fn batch(&mut self, operations: Vec<BatchOperation>) -> Result<()> {
        self.check_writable()?;

        // Execute all operations
        for operation in operations {
            match operation {
                BatchOperation::Put { key, value } => {
                    self.put(key, value)?;
                }
                BatchOperation::Delete { key } => {
                    self.delete(&key)?;
                }
            }
        }

        Ok(())
    }
}

impl MetadataStorage for InMemoryStorage {
    fn get_metadata(&self, key: &StorageKey) -> Result<Option<StorageMetadata>> {
        let metadata = self.metadata.read().unwrap();
        Ok(metadata.get(key).cloned())
    }

    fn set_metadata(&mut self, metadata: StorageMetadata) -> Result<()> {
        self.check_writable()?;

        let mut meta_store = self.metadata.write().unwrap();
        meta_store.insert(metadata.key.clone(), metadata);
        Ok(())
    }

    fn list_with_metadata(&self) -> Result<Vec<StorageMetadata>> {
        let metadata = self.metadata.read().unwrap();
        Ok(metadata.values().cloned().collect())
    }
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// Cached storage wrapper
#[derive(Debug)]
pub struct CachedStorage<T: Storage> {
    inner: T,
    cache: RwLock<HashMap<StorageKey, (StorageValue, SystemTime)>>,
    cache_ttl: Duration,
    max_cache_size: usize,
}

impl<T: Storage> CachedStorage<T> {
    /// Create cached storage wrapper
    pub fn new(inner: T, cache_ttl: Duration, max_cache_size: usize) -> Self {
        CachedStorage {
            inner,
            cache: RwLock::new(HashMap::with_capacity(max_cache_size)),
            cache_ttl,
            max_cache_size,
        }
    }

    /// Clear cache
    pub fn clear_cache(&self) {
        self.cache.write().unwrap().clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        let cache = self.cache.read().unwrap();
        (cache.len(), self.max_cache_size)
    }

    fn cleanup_expired(&self) {
        let now = SystemTime::now();
        let mut cache = self.cache.write().unwrap();
        
        cache.retain(|_, (_, timestamp)| {
            now.duration_since(*timestamp).unwrap_or(Duration::MAX) < self.cache_ttl
        });
    }

    fn evict_lru_if_needed(&self) {
        let mut cache = self.cache.write().unwrap();
        
        if cache.len() >= self.max_cache_size {
            // Simple LRU: remove oldest entry
            let oldest_key = cache.iter()
                .min_by_key(|(_, (_, timestamp))| *timestamp)
                .map(|(k, _)| k.clone());
            
            if let Some(key) = oldest_key {
                cache.remove(&key);
            }
        }
    }
}

impl<T: Storage> Storage for CachedStorage<T> {
    fn get(&self, key: &StorageKey) -> Result<Option<StorageValue>> {
        // Check cache first
        {
            let cache = self.cache.read().unwrap();
            if let Some((value, timestamp)) = cache.get(key) {
                let age = SystemTime::now()
                    .duration_since(*timestamp)
                    .unwrap_or(Duration::MAX);
                
                if age < self.cache_ttl {
                    return Ok(Some(value.clone()));
                }
            }
        }

        // Not in cache or expired, get from storage
        let result = self.inner.get(key)?;
        
        // Cache the result if found
        if let Some(ref value) = result {
            self.cleanup_expired();
            self.evict_lru_if_needed();
            
            let mut cache = self.cache.write().unwrap();
            cache.insert(key.clone(), (value.clone(), SystemTime::now()));
        }

        Ok(result)
    }

    fn put(&mut self, key: StorageKey, value: StorageValue) -> Result<()> {
        let result = self.inner.put(key.clone(), value.clone());
        
        if result.is_ok() {
            // Update cache
            self.cleanup_expired();
            self.evict_lru_if_needed();
            
            let mut cache = self.cache.write().unwrap();
            cache.insert(key, (value, SystemTime::now()));
        }
        
        result
    }

    fn delete(&mut self, key: &StorageKey) -> Result<bool> {
        let result = self.inner.delete(key);
        
        if result.is_ok() {
            // Remove from cache
            self.cache.write().unwrap().remove(key);
        }
        
        result
    }

    fn contains(&self, key: &StorageKey) -> Result<bool> {
        // Check cache first
        {
            let cache = self.cache.read().unwrap();
            if let Some((_, timestamp)) = cache.get(key) {
                let age = SystemTime::now()
                    .duration_since(*timestamp)
                    .unwrap_or(Duration::MAX);
                
                if age < self.cache_ttl {
                    return Ok(true);
                }
            }
        }

        self.inner.contains(key)
    }

    fn keys(&self) -> Result<Vec<StorageKey>> {
        self.inner.keys()
    }

    fn size(&self) -> Result<usize> {
        self.inner.size()
    }

    fn clear(&mut self) -> Result<()> {
        let result = self.inner.clear();
        if result.is_ok() {
            self.clear_cache();
        }
        result
    }
}

/// Storage interface utilities
pub struct StorageUtils;

impl StorageUtils {
    /// Copy all data from source to destination storage
    pub fn copy_all<S: Storage, D: Storage>(source: &S, destination: &mut D) -> Result<usize> {
        let keys = source.keys()?;
        let mut copied = 0;

        for key in keys {
            if let Some(value) = source.get(&key)? {
                destination.put(key, value)?;
                copied += 1;
            }
        }

        Ok(copied)
    }

    /// Validate storage integrity
    pub fn validate_storage<S: Storage>(storage: &S) -> Result<Vec<String>> {
        let mut issues = Vec::new();
        let keys = storage.keys()?;

        for key in keys {
            match storage.get(&key) {
                Ok(Some(_)) => {
                    // Value exists and is accessible
                }
                Ok(None) => {
                    issues.push(format!("Key exists but value is None: {:?}", 
                                       String::from_utf8_lossy(&key)));
                }
                Err(e) => {
                    issues.push(format!("Error accessing key {:?}: {}", 
                                       String::from_utf8_lossy(&key), e));
                }
            }
        }

        Ok(issues)
    }

    /// Create storage key from string
    pub fn key_from_string(s: &str) -> StorageKey {
        s.as_bytes().to_vec()
    }

    /// Create storage key from multiple parts
    pub fn key_from_parts(parts: &[&str]) -> StorageKey {
        parts.join(":").as_bytes().to_vec()
    }

    /// Convert storage key to string (if valid UTF-8)
    pub fn key_to_string(key: &StorageKey) -> Option<String> {
        String::from_utf8(key.clone()).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_metadata() {
        let key = b"test_key".to_vec();
        let mut meta = StorageMetadata::new(key.clone(), 100);
        
        assert_eq!(meta.key, key);
        assert_eq!(meta.size, 100);
        assert_eq!(meta.version, 1);
        assert_eq!(meta.access_count, 0);

        meta.increment_access();
        assert_eq!(meta.access_count, 1);

        meta.update(200);
        assert_eq!(meta.size, 200);
        assert_eq!(meta.version, 2);
    }

    #[test]
    fn test_in_memory_storage_basic() {
        let mut storage = InMemoryStorage::new();
        let key = b"test_key".to_vec();
        let value = b"test_value".to_vec();

        // Test put and get
        storage.put(key.clone(), value.clone()).unwrap();
        assert_eq!(storage.get(&key).unwrap(), Some(value.clone()));

        // Test contains
        assert!(storage.contains(&key).unwrap());

        // Test keys
        let keys = storage.keys().unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0], key);

        // Test delete
        assert!(storage.delete(&key).unwrap());
        assert_eq!(storage.get(&key).unwrap(), None);
        assert!(!storage.contains(&key).unwrap());
    }

    #[test]
    fn test_in_memory_storage_batch() {
        let mut storage = InMemoryStorage::new();
        
        let operations = vec![
            BatchOperation::Put {
                key: b"key1".to_vec(),
                value: b"value1".to_vec(),
            },
            BatchOperation::Put {
                key: b"key2".to_vec(),
                value: b"value2".to_vec(),
            },
            BatchOperation::Delete {
                key: b"key1".to_vec(),
            },
        ];

        storage.batch(operations).unwrap();

        assert_eq!(storage.get(&b"key1".to_vec()).unwrap(), None);
        assert_eq!(storage.get(&b"key2".to_vec()).unwrap(), Some(b"value2".to_vec()));
    }

    #[test]
    fn test_in_memory_storage_metadata() {
        let mut storage = InMemoryStorage::new();
        let key = b"test_key".to_vec();
        let value = b"test_value".to_vec();

        storage.put(key.clone(), value).unwrap();
        
        let metadata = storage.get_metadata(&key).unwrap().unwrap();
        assert_eq!(metadata.key, key);
        assert_eq!(metadata.size, 10); // "test_value" length
        assert_eq!(metadata.version, 1);

        // Access the key to update metadata
        storage.get(&key).unwrap();
        
        let updated_metadata = storage.get_metadata(&key).unwrap().unwrap();
        assert_eq!(updated_metadata.access_count, 1);
    }

    #[test]
    fn test_storage_stats() {
        let mut storage = InMemoryStorage::new();
        let key1 = b"key1".to_vec();
        let key2 = b"key2".to_vec();
        let value = b"value".to_vec();

        storage.put(key1.clone(), value.clone()).unwrap();
        storage.put(key2.clone(), value.clone()).unwrap();
        storage.get(&key1).unwrap();
        storage.get(&b"nonexistent".to_vec()).unwrap();

        let stats = storage.get_stats();
        assert_eq!(stats.total_keys, 2);
        assert_eq!(stats.total_size_bytes, 10); // 2 * "value".len()
        assert_eq!(stats.put_operations, 2);
        assert_eq!(stats.get_operations, 2);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.cache_hit_rate(), 0.5);
    }

    #[test]
    fn test_read_only_storage() {
        let mut initial_data = HashMap::new();
        initial_data.insert(b"key1".to_vec(), b"value1".to_vec());
        
        let mut storage = InMemoryStorage::new_read_only(initial_data);

        // Should be able to read
        assert_eq!(storage.get(&b"key1".to_vec()).unwrap(), Some(b"value1".to_vec()));

        // Should not be able to write
        assert!(storage.put(b"key2".to_vec(), b"value2".to_vec()).is_err());
        assert!(storage.delete(&b"key1".to_vec()).is_err());
        assert!(storage.clear().is_err());
    }

    #[test]
    fn test_cached_storage() {
        let inner = InMemoryStorage::new();
        let mut cached = CachedStorage::new(inner, Duration::from_secs(60), 10);
        
        let key = b"test_key".to_vec();
        let value = b"test_value".to_vec();

        // Put and get through cache
        cached.put(key.clone(), value.clone()).unwrap();
        assert_eq!(cached.get(&key).unwrap(), Some(value));

        // Check cache stats
        let (cache_size, cache_capacity) = cached.cache_stats();
        assert_eq!(cache_size, 1);
        assert_eq!(cache_capacity, 10);
    }

    #[test]
    fn test_storage_utils() {
        let mut source = InMemoryStorage::new();
        let mut destination = InMemoryStorage::new();

        source.put(b"key1".to_vec(), b"value1".to_vec()).unwrap();
        source.put(b"key2".to_vec(), b"value2".to_vec()).unwrap();

        let copied = StorageUtils::copy_all(&source, &mut destination).unwrap();
        assert_eq!(copied, 2);
        
        assert_eq!(destination.get(&b"key1".to_vec()).unwrap(), Some(b"value1".to_vec()));
        assert_eq!(destination.get(&b"key2".to_vec()).unwrap(), Some(b"value2".to_vec()));
    }

    #[test]
    fn test_storage_validation() {
        let storage = InMemoryStorage::new();
        let issues = StorageUtils::validate_storage(&storage).unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_key_utilities() {
        let key1 = StorageUtils::key_from_string("test");
        assert_eq!(key1, b"test".to_vec());

        let key2 = StorageUtils::key_from_parts(&["prefix", "middle", "suffix"]);
        assert_eq!(key2, b"prefix:middle:suffix".to_vec());

        let string = StorageUtils::key_to_string(&key1).unwrap();
        assert_eq!(string, "test");
    }
}

