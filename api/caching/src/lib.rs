//! CC Chain API Caching
//!
//! This module provides comprehensive caching functionality for the CC Chain API,
//! including in-memory caches, cache invalidation strategies, and performance optimization.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Cache miss for key: {key}")]
    Miss { key: String },
    #[error("Cache entry expired for key: {key}")]
    Expired { key: String },
    #[error("Cache capacity exceeded")]
    CapacityExceeded,
    #[error("Invalid cache configuration: {reason}")]
    InvalidConfig { reason: String },
    #[error("Serialization error: {0}")]
    Serialization(String),
}

pub type Result<T> = std::result::Result<T, CacheError>;

/// Cache entry with expiration and metadata
#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    pub value: T,
    pub created_at: Instant,
    pub expires_at: Option<Instant>,
    pub access_count: u64,
    pub last_accessed: Instant,
}

impl<T> CacheEntry<T> {
    pub fn new(value: T, ttl: Option<Duration>) -> Self {
        let now = Instant::now();
        Self {
            value,
            created_at: now,
            expires_at: ttl.map(|d| now + d),
            access_count: 0,
            last_accessed: now,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at.map_or(false, |expires| Instant::now() > expires)
    }

    pub fn access(&mut self) -> &T {
        self.access_count += 1;
        self.last_accessed = Instant::now();
        &self.value
    }
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub max_entries: usize,
    pub default_ttl: Option<Duration>,
    pub cleanup_interval: Duration,
    pub eviction_policy: EvictionPolicy,
}

#[derive(Debug, Clone)]
pub enum EvictionPolicy {
    LRU,  // Least Recently Used
    LFU,  // Least Frequently Used
    FIFO, // First In, First Out
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            default_ttl: Some(Duration::from_secs(300)), // 5 minutes
            cleanup_interval: Duration::from_secs(60),   // 1 minute
            eviction_policy: EvictionPolicy::LRU,
        }
    }
}

/// In-memory cache implementation
pub struct ApiCache<K, V> 
where
    K: Hash + Eq + Clone + std::fmt::Debug,
    V: Clone,
{
    entries: HashMap<K, CacheEntry<V>>,
    config: CacheConfig,
    last_cleanup: Instant,
    hit_count: u64,
    miss_count: u64,
}

impl<K, V> ApiCache<K, V> 
where
    K: Hash + Eq + Clone + std::fmt::Debug,
    V: Clone,
{
    pub fn new(config: CacheConfig) -> Self {
        Self {
            entries: HashMap::new(),
            config,
            last_cleanup: Instant::now(),
            hit_count: 0,
            miss_count: 0,
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(CacheConfig::default())
    }

    /// Get value from cache
    pub fn get(&mut self, key: &K) -> Result<V> {
        self.cleanup_if_needed();

        if let Some(entry) = self.entries.get_mut(key) {
            if entry.is_expired() {
                self.entries.remove(key);
                self.miss_count += 1;
                return Err(CacheError::Expired { key: format!("{:?}", key) });
            }
            
            self.hit_count += 1;
            Ok(entry.access().clone())
        } else {
            self.miss_count += 1;
            Err(CacheError::Miss { key: format!("{:?}", key) })
        }
    }

    /// Insert value into cache
    pub fn insert(&mut self, key: K, value: V) -> Result<()> {
        self.cleanup_if_needed();
        
        // Check capacity and evict if necessary
        if self.entries.len() >= self.config.max_entries && !self.entries.contains_key(&key) {
            self.evict_one()?;
        }

        let entry = CacheEntry::new(value, self.config.default_ttl);
        self.entries.insert(key, entry);
        Ok(())
    }

    /// Insert with custom TTL
    pub fn insert_with_ttl(&mut self, key: K, value: V, ttl: Duration) -> Result<()> {
        self.cleanup_if_needed();
        
        if self.entries.len() >= self.config.max_entries && !self.entries.contains_key(&key) {
            self.evict_one()?;
        }

        let entry = CacheEntry::new(value, Some(ttl));
        self.entries.insert(key, entry);
        Ok(())
    }

    /// Remove entry from cache
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.entries.remove(key).map(|entry| entry.value)
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
        self.hit_count = 0;
        self.miss_count = 0;
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let total_requests = self.hit_count + self.miss_count;
        let hit_rate = if total_requests > 0 {
            self.hit_count as f64 / total_requests as f64
        } else {
            0.0
        };

        CacheStats {
            entries: self.entries.len(),
            max_entries: self.config.max_entries,
            hit_count: self.hit_count,
            miss_count: self.miss_count,
            hit_rate,
            total_requests,
        }
    }

    /// Check if key exists in cache (doesn't update access statistics)
    pub fn contains_key(&self, key: &K) -> bool {
        if let Some(entry) = self.entries.get(key) {
            !entry.is_expired()
        } else {
            false
        }
    }

    /// Cleanup expired entries
    fn cleanup_expired(&mut self) {
        let keys_to_remove: Vec<K> = self.entries
            .iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(key, _)| key.clone())
            .collect();

        for key in keys_to_remove {
            self.entries.remove(&key);
        }
    }

    /// Cleanup if needed based on interval
    fn cleanup_if_needed(&mut self) {
        if self.last_cleanup.elapsed() >= self.config.cleanup_interval {
            self.cleanup_expired();
            self.last_cleanup = Instant::now();
        }
    }

    /// Evict one entry based on eviction policy
    fn evict_one(&mut self) -> Result<()> {
        if self.entries.is_empty() {
            return Err(CacheError::CapacityExceeded);
        }

        let key_to_remove = match self.config.eviction_policy {
            EvictionPolicy::LRU => {
                // Find least recently used
                self.entries
                    .iter()
                    .min_by_key(|(_, entry)| entry.last_accessed)
                    .map(|(key, _)| key.clone())
            }
            EvictionPolicy::LFU => {
                // Find least frequently used
                self.entries
                    .iter()
                    .min_by_key(|(_, entry)| entry.access_count)
                    .map(|(key, _)| key.clone())
            }
            EvictionPolicy::FIFO => {
                // Find oldest entry
                self.entries
                    .iter()
                    .min_by_key(|(_, entry)| entry.created_at)
                    .map(|(key, _)| key.clone())
            }
        };

        if let Some(key) = key_to_remove {
            self.entries.remove(&key);
            Ok(())
        } else {
            Err(CacheError::CapacityExceeded)
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub entries: usize,
    pub max_entries: usize,
    pub hit_count: u64,
    pub miss_count: u64,
    pub hit_rate: f64,
    pub total_requests: u64,
}

/// Multi-layered cache for different types of data
pub struct LayeredCache {
    pub block_cache: ApiCache<String, String>,      // Block data by hash
    pub transaction_cache: ApiCache<String, String>, // Transaction data by hash
    pub account_cache: ApiCache<String, String>,     // Account data by address
    pub query_cache: ApiCache<String, String>,       // Query results by query hash
}

impl LayeredCache {
    pub fn new() -> Self {
        let block_config = CacheConfig {
            max_entries: 1000,
            default_ttl: Some(Duration::from_secs(600)), // 10 minutes for blocks
            cleanup_interval: Duration::from_secs(60),
            eviction_policy: EvictionPolicy::LRU,
        };

        let transaction_config = CacheConfig {
            max_entries: 5000,
            default_ttl: Some(Duration::from_secs(300)), // 5 minutes for transactions
            cleanup_interval: Duration::from_secs(60),
            eviction_policy: EvictionPolicy::LRU,
        };

        let account_config = CacheConfig {
            max_entries: 2000,
            default_ttl: Some(Duration::from_secs(120)), // 2 minutes for accounts
            cleanup_interval: Duration::from_secs(60),
            eviction_policy: EvictionPolicy::LRU,
        };

        let query_config = CacheConfig {
            max_entries: 1000,
            default_ttl: Some(Duration::from_secs(60)), // 1 minute for queries
            cleanup_interval: Duration::from_secs(30),
            eviction_policy: EvictionPolicy::LRU,
        };

        Self {
            block_cache: ApiCache::new(block_config),
            transaction_cache: ApiCache::new(transaction_config),
            account_cache: ApiCache::new(account_config),
            query_cache: ApiCache::new(query_config),
        }
    }

    /// Get comprehensive cache statistics
    pub fn get_all_stats(&self) -> LayeredCacheStats {
        LayeredCacheStats {
            block_cache: self.block_cache.stats(),
            transaction_cache: self.transaction_cache.stats(),
            account_cache: self.account_cache.stats(),
            query_cache: self.query_cache.stats(),
        }
    }
}

impl Default for LayeredCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for layered cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayeredCacheStats {
    pub block_cache: CacheStats,
    pub transaction_cache: CacheStats,
    pub account_cache: CacheStats,
    pub query_cache: CacheStats,
}

/// Cache key generator for consistent hashing
pub struct CacheKeyGenerator;

impl CacheKeyGenerator {
    /// Generate cache key for block queries
    pub fn block_key(hash: Option<&str>, height: Option<u64>) -> String {
        match (hash, height) {
            (Some(h), _) => format!("block_hash:{}", h),
            (None, Some(h)) => format!("block_height:{}", h),
            _ => "block_latest".to_string(),
        }
    }

    /// Generate cache key for transaction queries
    pub fn transaction_key(hash: &str) -> String {
        format!("tx:{}", hash)
    }

    /// Generate cache key for account queries
    pub fn account_key(address: &str) -> String {
        format!("account:{}", address)
    }

    /// Generate cache key for custom queries
    pub fn query_key(endpoint: &str, params: &str) -> String {
        format!("query:{}:{}", endpoint, Self::hash_string(params))
    }

    /// Simple string hashing for cache keys
    fn hash_string(s: &str) -> String {
        let hash_value = s.len().wrapping_mul(31).wrapping_add(
            s.chars().map(|c| c as u32).sum::<u32>() as usize
        );
        format!("{:x}", hash_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_cache_basic_operations() {
        let mut cache = ApiCache::<String, String>::with_default_config();
        
        // Test insert and get
        cache.insert("key1".to_string(), "value1".to_string()).unwrap();
        assert_eq!(cache.get(&"key1".to_string()).unwrap(), "value1");
        
        // Test cache miss
        assert!(cache.get(&"nonexistent".to_string()).is_err());
    }

    #[test]
    fn test_cache_expiration() {
        let mut cache = ApiCache::<String, String>::with_default_config();
        
        // Insert with very short TTL
        cache.insert_with_ttl("key1".to_string(), "value1".to_string(), Duration::from_millis(50)).unwrap();
        
        // Should be available immediately
        assert!(cache.get(&"key1".to_string()).is_ok());
        
        // Wait for expiration
        thread::sleep(Duration::from_millis(100));
        
        // Should be expired now
        assert!(cache.get(&"key1".to_string()).is_err());
    }

    #[test]
    fn test_cache_capacity_and_eviction() {
        let config = CacheConfig {
            max_entries: 2,
            default_ttl: None,
            cleanup_interval: Duration::from_secs(1),
            eviction_policy: EvictionPolicy::LRU,
        };
        
        let mut cache = ApiCache::<String, String>::new(config);
        
        // Fill cache to capacity
        cache.insert("key1".to_string(), "value1".to_string()).unwrap();
        cache.insert("key2".to_string(), "value2".to_string()).unwrap();
        
        // Access key1 to update its LRU status
        let _ = cache.get(&"key1".to_string());
        
        // Insert third item, should evict key2 (least recently used)
        cache.insert("key3".to_string(), "value3".to_string()).unwrap();
        
        // key1 and key3 should exist, key2 should be evicted
        assert!(cache.get(&"key1".to_string()).is_ok());
        assert!(cache.get(&"key3".to_string()).is_ok());
        assert!(cache.get(&"key2".to_string()).is_err());
    }

    #[test]
    fn test_cache_statistics() {
        let mut cache = ApiCache::<String, String>::with_default_config();
        
        cache.insert("key1".to_string(), "value1".to_string()).unwrap();
        
        // One hit
        let _ = cache.get(&"key1".to_string());
        
        // One miss
        let _ = cache.get(&"nonexistent".to_string());
        
        let stats = cache.stats();
        assert_eq!(stats.hit_count, 1);
        assert_eq!(stats.miss_count, 1);
        assert_eq!(stats.hit_rate, 0.5);
        assert_eq!(stats.total_requests, 2);
    }

    #[test]
    fn test_layered_cache() {
        let mut layered = LayeredCache::new();
        
        // Test different cache layers
        layered.block_cache.insert("block1".to_string(), "block_data".to_string()).unwrap();
        layered.transaction_cache.insert("tx1".to_string(), "tx_data".to_string()).unwrap();
        layered.account_cache.insert("addr1".to_string(), "account_data".to_string()).unwrap();
        
        assert!(layered.block_cache.get(&"block1".to_string()).is_ok());
        assert!(layered.transaction_cache.get(&"tx1".to_string()).is_ok());
        assert!(layered.account_cache.get(&"addr1".to_string()).is_ok());
    }

    #[test]
    fn test_cache_key_generation() {
        assert_eq!(CacheKeyGenerator::block_key(Some("0x123"), None), "block_hash:0x123");
        assert_eq!(CacheKeyGenerator::block_key(None, Some(100)), "block_height:100");
        assert_eq!(CacheKeyGenerator::transaction_key("0xabc"), "tx:0xabc");
        assert_eq!(CacheKeyGenerator::account_key("0xdef"), "account:0xdef");
        
        let query_key = CacheKeyGenerator::query_key("blocks", "limit=10");
        assert!(query_key.starts_with("query:blocks:"));
    }

    #[test]
    fn test_cache_entry_access_tracking() {
        let mut entry = CacheEntry::new("test_value".to_string(), None);
        
        assert_eq!(entry.access_count, 0);
        
        let value = entry.access();
        assert_eq!(value, "test_value");
        assert_eq!(entry.access_count, 1);
        
        entry.access();
        assert_eq!(entry.access_count, 2);
    }

    #[test]
    fn test_cache_contains_key() {
        let mut cache = ApiCache::<String, String>::with_default_config();
        
        cache.insert("key1".to_string(), "value1".to_string()).unwrap();
        
        assert!(cache.contains_key(&"key1".to_string()));
        assert!(!cache.contains_key(&"nonexistent".to_string()));
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = ApiCache::<String, String>::with_default_config();
        
        cache.insert("key1".to_string(), "value1".to_string()).unwrap();
        cache.insert("key2".to_string(), "value2".to_string()).unwrap();
        
        let stats_before = cache.stats();
        assert_eq!(stats_before.entries, 2);
        
        cache.clear();
        
        let stats_after = cache.stats();
        assert_eq!(stats_after.entries, 0);
        assert_eq!(stats_after.hit_count, 0);
        assert_eq!(stats_after.miss_count, 0);
    }
}
