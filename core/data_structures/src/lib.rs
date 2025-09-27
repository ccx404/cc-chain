//! Core data structures functionality
//!
//! This module provides fundamental data structures used throughout the CC Chain,
//! including specialized collections, trees, caches, and blockchain-specific structures.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque, BinaryHeap};
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant, SystemTime};
use thiserror::Error;

/// Data structure-related errors
#[derive(Error, Debug, Clone, PartialEq)]
pub enum DataStructureError {
    #[error("Index out of bounds: {0}")]
    IndexOutOfBounds(usize),
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    #[error("Collection is empty")]
    EmptyCollection,
    #[error("Operation not supported: {0}")]
    UnsupportedOperation(String),
    #[error("Invalid capacity: {0}")]
    InvalidCapacity(usize),
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),
}

pub type Result<T> = std::result::Result<T, DataStructureError>;

/// Priority item for priority queue
#[derive(Debug, Clone)]
pub struct PriorityItem<T> {
    pub item: T,
    pub priority: u64,
    pub timestamp: Instant,
}

impl<T> PartialEq for PriorityItem<T> {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.timestamp == other.timestamp
    }
}

impl<T> Eq for PriorityItem<T> {}

impl<T> PriorityItem<T> {
    pub fn new(item: T, priority: u64) -> Self {
        PriorityItem {
            item,
            priority,
            timestamp: Instant::now(),
        }
    }
}

impl<T> PartialOrd for PriorityItem<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for PriorityItem<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
            .then_with(|| other.timestamp.cmp(&self.timestamp)) // Older items first for same priority
    }
}

/// A priority queue with capacity limits and expiration
#[derive(Debug)]
pub struct PriorityQueue<T> {
    heap: BinaryHeap<PriorityItem<T>>,
    max_capacity: Option<usize>,
    expiration_duration: Option<Duration>,
}

impl<T> PriorityQueue<T> {
    /// Create a new priority queue
    pub fn new() -> Self {
        PriorityQueue {
            heap: BinaryHeap::new(),
            max_capacity: None,
            expiration_duration: None,
        }
    }

    /// Create with maximum capacity
    pub fn with_capacity(max_capacity: usize) -> Self {
        PriorityQueue {
            heap: BinaryHeap::with_capacity(max_capacity),
            max_capacity: Some(max_capacity),
            expiration_duration: None,
        }
    }

    /// Set expiration duration for items
    pub fn with_expiration(mut self, duration: Duration) -> Self {
        self.expiration_duration = Some(duration);
        self
    }

    /// Push item with priority
    pub fn push(&mut self, item: T, priority: u64) -> Result<()> {
        // Check capacity
        if let Some(max_cap) = self.max_capacity {
            if self.heap.len() >= max_cap {
                // Remove lowest priority item to make space
                self.remove_lowest_priority();
            }
        }

        self.heap.push(PriorityItem::new(item, priority));
        Ok(())
    }

    /// Pop highest priority item
    pub fn pop(&mut self) -> Option<T> {
        self.cleanup_expired();
        self.heap.pop().map(|item| item.item)
    }

    /// Peek at highest priority item without removing
    pub fn peek(&mut self) -> Option<&T> {
        self.cleanup_expired();
        self.heap.peek().map(|item| &item.item)
    }

    /// Get current size
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    /// Remove expired items
    fn cleanup_expired(&mut self) {
        if let Some(expiration) = self.expiration_duration {
            let now = Instant::now();
            let mut valid_items = Vec::new();

            while let Some(item) = self.heap.pop() {
                if now.duration_since(item.timestamp) < expiration {
                    valid_items.push(item);
                }
            }

            self.heap = valid_items.into();
        }
    }

    /// Remove lowest priority item
    fn remove_lowest_priority(&mut self) {
        let mut items: Vec<_> = self.heap.drain().collect();
        items.sort_by(|a, b| b.priority.cmp(&a.priority)); // Sort by priority desc
        
        if !items.is_empty() {
            items.pop(); // Remove lowest priority
        }
        
        self.heap = items.into();
    }
}

impl<T> Default for PriorityQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// LRU Cache with time-based expiration
#[derive(Debug)]
pub struct LRUCache<K, V> 
where
    K: Clone + Hash + Eq,
    V: Clone,
{
    map: HashMap<K, (V, Instant)>,
    order: VecDeque<K>,
    capacity: usize,
    expiration: Option<Duration>,
}

impl<K, V> LRUCache<K, V>
where
    K: Clone + Hash + Eq,
    V: Clone,
{
    /// Create new LRU cache with capacity
    pub fn new(capacity: usize) -> Result<Self> {
        if capacity == 0 {
            return Err(DataStructureError::InvalidCapacity(capacity));
        }

        Ok(LRUCache {
            map: HashMap::with_capacity(capacity),
            order: VecDeque::with_capacity(capacity),
            capacity,
            expiration: None,
        })
    }

    /// Set expiration time for cache entries
    pub fn with_expiration(mut self, expiration: Duration) -> Self {
        self.expiration = Some(expiration);
        self
    }

    /// Insert or update value
    pub fn put(&mut self, key: K, value: V) -> Option<V> {
        self.cleanup_expired();

        let now = Instant::now();

        if let Some((old_value, _)) = self.map.remove(&key) {
            // Key exists, update and move to front
            self.remove_from_order(&key);
            self.order.push_front(key.clone());
            self.map.insert(key, (value, now));
            Some(old_value)
        } else {
            // New key
            if self.map.len() >= self.capacity {
                // Remove oldest entry
                if let Some(old_key) = self.order.pop_back() {
                    self.map.remove(&old_key);
                }
            }

            self.order.push_front(key.clone());
            self.map.insert(key, (value, now));
            None
        }
    }

    /// Get value by key
    pub fn get(&mut self, key: &K) -> Option<&V> {
        self.cleanup_expired();

        // Check if key exists and not expired
        let should_remove = if let Some((_, timestamp)) = self.map.get(key) {
            if let Some(expiration) = self.expiration {
                timestamp.elapsed() > expiration
            } else {
                false
            }
        } else {
            return None;
        };

        if should_remove {
            self.remove(key);
            return None;
        }

        // Move to front and return value
        if self.map.contains_key(key) {
            self.remove_from_order(key);
            self.order.push_front(key.clone());
            self.map.get(key).map(|(v, _)| v)
        } else {
            None
        }
    }

    /// Remove entry
    pub fn remove(&mut self, key: &K) -> Option<V> {
        if let Some((value, _)) = self.map.remove(key) {
            self.remove_from_order(key);
            Some(value)
        } else {
            None
        }
    }

    /// Check if key exists
    pub fn contains_key(&mut self, key: &K) -> bool {
        self.cleanup_expired();
        self.map.contains_key(key)
    }

    /// Get current size
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.map.clear();
        self.order.clear();
    }

    /// Remove key from order queue
    fn remove_from_order(&mut self, key: &K) {
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            self.order.remove(pos);
        }
    }

    /// Remove expired entries
    fn cleanup_expired(&mut self) {
        if let Some(expiration) = self.expiration {
            let now = Instant::now();
            let expired_keys: Vec<_> = self.map.iter()
                .filter(|(_, (_, timestamp))| now.duration_since(*timestamp) > expiration)
                .map(|(k, _)| k.clone())
                .collect();

            for key in expired_keys {
                self.remove(&key);
            }
        }
    }
}

/// Ring buffer with fixed capacity
#[derive(Debug, Clone)]
pub struct RingBuffer<T> 
where 
    T: Clone,
{
    buffer: Vec<Option<T>>,
    capacity: usize,
    head: usize,
    tail: usize,
    size: usize,
}

impl<T> RingBuffer<T> 
where 
    T: Clone,
{
    /// Create new ring buffer with capacity
    pub fn new(capacity: usize) -> Result<Self> {
        if capacity == 0 {
            return Err(DataStructureError::InvalidCapacity(capacity));
        }

        Ok(RingBuffer {
            buffer: vec![None; capacity],
            capacity,
            head: 0,
            tail: 0,
            size: 0,
        })
    }

    /// Push item to buffer (overwrites oldest if full)
    pub fn push(&mut self, item: T) -> Option<T> {
        let old_item = if self.is_full() {
            self.buffer[self.tail].take()
        } else {
            None
        };

        self.buffer[self.head] = Some(item);
        self.head = (self.head + 1) % self.capacity;

        if self.is_full() {
            self.tail = (self.tail + 1) % self.capacity;
        } else {
            self.size += 1;
        }

        old_item
    }

    /// Pop from front (oldest item)
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let item = self.buffer[self.tail].take();
        self.tail = (self.tail + 1) % self.capacity;
        self.size -= 1;

        item
    }

    /// Peek at front item without removing
    pub fn front(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            self.buffer[self.tail].as_ref()
        }
    }

    /// Peek at back item without removing
    pub fn back(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            let back_idx = if self.head == 0 { 
                self.capacity - 1 
            } else { 
                self.head - 1 
            };
            self.buffer[back_idx].as_ref()
        }
    }

    /// Get item at index (0 = front/oldest)
    pub fn get(&self, index: usize) -> Result<Option<&T>> {
        if index >= self.size {
            return Err(DataStructureError::IndexOutOfBounds(index));
        }

        let actual_index = (self.tail + index) % self.capacity;
        Ok(self.buffer[actual_index].as_ref())
    }

    /// Current size
    pub fn len(&self) -> usize {
        self.size
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Check if full
    pub fn is_full(&self) -> bool {
        self.size == self.capacity
    }

    /// Get capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Clear buffer
    pub fn clear(&mut self) {
        for item in &mut self.buffer {
            *item = None;
        }
        self.head = 0;
        self.tail = 0;
        self.size = 0;
    }

    /// Convert to vector (oldest first)
    pub fn to_vec(&self) -> Vec<T> 
    where 
        T: Clone,
    {
        let mut result = Vec::with_capacity(self.size);
        for i in 0..self.size {
            if let Ok(Some(item)) = self.get(i) {
                result.push(item.clone());
            }
        }
        result
    }
}

/// Merkle tree for efficient integrity verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleTree {
    nodes: Vec<Vec<u8>>,
    leaf_count: usize,
}

impl MerkleTree {
    /// Build Merkle tree from data
    pub fn new<T: AsRef<[u8]>>(data: Vec<T>) -> Self {
        let leaf_count = data.len();
        
        if leaf_count == 0 {
            return MerkleTree {
                nodes: vec![vec![0u8; 32]], // Empty root
                leaf_count: 0,
            };
        }

        // Hash all leaf data
        let mut nodes: Vec<Vec<u8>> = data.iter()
            .map(|item| {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(item.as_ref());
                hasher.finalize().to_vec()
            })
            .collect();

        // Build tree bottom-up
        while nodes.len() > 1 {
            let mut next_level = Vec::new();
            
            for chunk in nodes.chunks(2) {
                let combined_hash = if chunk.len() == 2 {
                    use sha2::{Digest, Sha256};
                    let mut hasher = Sha256::new();
                    hasher.update(&chunk[0]);
                    hasher.update(&chunk[1]);
                    hasher.finalize().to_vec()
                } else {
                    chunk[0].clone()
                };
                next_level.push(combined_hash);
            }
            
            nodes = next_level;
        }

        MerkleTree { nodes, leaf_count }
    }

    /// Get root hash
    pub fn root(&self) -> Option<&[u8]> {
        self.nodes.first().map(|v| v.as_slice())
    }

    /// Get leaf count
    pub fn leaf_count(&self) -> usize {
        self.leaf_count
    }

    /// Check if tree is empty
    pub fn is_empty(&self) -> bool {
        self.leaf_count == 0
    }
}

/// Time-series data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint<T> {
    pub timestamp: SystemTime,
    pub value: T,
}

impl<T> TimeSeriesPoint<T> {
    pub fn new(value: T) -> Self {
        TimeSeriesPoint {
            timestamp: SystemTime::now(),
            value,
        }
    }

    pub fn with_timestamp(value: T, timestamp: SystemTime) -> Self {
        TimeSeriesPoint { timestamp, value }
    }

    pub fn age(&self) -> Duration {
        SystemTime::now().duration_since(self.timestamp)
            .unwrap_or_else(|_| Duration::from_secs(0))
    }
}

/// Time-series data collection
#[derive(Debug, Clone)]
pub struct TimeSeries<T> {
    data: VecDeque<TimeSeriesPoint<T>>,
    max_age: Option<Duration>,
    max_points: Option<usize>,
}

impl<T> TimeSeries<T> {
    /// Create new time series
    pub fn new() -> Self {
        TimeSeries {
            data: VecDeque::new(),
            max_age: None,
            max_points: None,
        }
    }

    /// Set maximum age for data points
    pub fn with_max_age(mut self, max_age: Duration) -> Self {
        self.max_age = Some(max_age);
        self
    }

    /// Set maximum number of data points
    pub fn with_max_points(mut self, max_points: usize) -> Self {
        self.max_points = Some(max_points);
        self
    }

    /// Add data point
    pub fn add_point(&mut self, value: T) {
        let point = TimeSeriesPoint::new(value);
        self.data.push_back(point);
        self.cleanup();
    }

    /// Add data point with specific timestamp
    pub fn add_point_with_timestamp(&mut self, value: T, timestamp: SystemTime) {
        // Insert in correct chronological order
        let mut insert_pos = None;
        for (i, existing) in self.data.iter().enumerate() {
            if timestamp <= existing.timestamp {
                insert_pos = Some(i);
                break;
            }
        }
        
        let point = TimeSeriesPoint::with_timestamp(value, timestamp);
        
        if let Some(pos) = insert_pos {
            self.data.insert(pos, point);
        } else {
            self.data.push_back(point);
        }
        
        self.cleanup();
    }

    /// Get all points
    pub fn get_points(&self) -> &VecDeque<TimeSeriesPoint<T>> {
        &self.data
    }

    /// Get points within time range
    pub fn get_points_in_range(&self, start: SystemTime, end: SystemTime) -> Vec<&TimeSeriesPoint<T>> {
        self.data.iter()
            .filter(|point| point.timestamp >= start && point.timestamp <= end)
            .collect()
    }

    /// Get latest point
    pub fn latest(&self) -> Option<&TimeSeriesPoint<T>> {
        self.data.back()
    }

    /// Get oldest point
    pub fn oldest(&self) -> Option<&TimeSeriesPoint<T>> {
        self.data.front()
    }

    /// Get point count
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Clear all points
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Remove expired and excess points
    fn cleanup(&mut self) {
        let now = SystemTime::now();

        // Remove expired points
        if let Some(max_age) = self.max_age {
            while let Some(front) = self.data.front() {
                if now.duration_since(front.timestamp).unwrap_or_else(|_| Duration::from_secs(0)) > max_age {
                    self.data.pop_front();
                } else {
                    break;
                }
            }
        }

        // Remove excess points
        if let Some(max_points) = self.max_points {
            while self.data.len() > max_points {
                self.data.pop_front();
            }
        }
    }
}

impl<T> Default for TimeSeries<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Bloom filter for membership testing
#[derive(Debug, Clone)]
pub struct BloomFilter {
    bits: Vec<bool>,
    hash_count: usize,
    size: usize,
}

impl BloomFilter {
    /// Create bloom filter with expected items and false positive rate
    pub fn new(expected_items: usize, false_positive_rate: f64) -> Result<Self> {
        if expected_items == 0 {
            return Err(DataStructureError::InvalidCapacity(expected_items));
        }

        let size = Self::optimal_size(expected_items, false_positive_rate);
        let hash_count = Self::optimal_hash_count(expected_items, size);

        Ok(BloomFilter {
            bits: vec![false; size],
            hash_count,
            size,
        })
    }

    /// Add item to filter
    pub fn add<T: Hash>(&mut self, item: &T) {
        let hashes = self.hash_item(item);
        for hash_val in hashes {
            self.bits[hash_val % self.size] = true;
        }
    }

    /// Test if item might be in set
    pub fn contains<T: Hash>(&self, item: &T) -> bool {
        let hashes = self.hash_item(item);
        hashes.iter().all(|&hash_val| self.bits[hash_val % self.size])
    }

    /// Clear all bits
    pub fn clear(&mut self) {
        self.bits.fill(false);
    }

    /// Get filter statistics
    pub fn stats(&self) -> BloomFilterStats {
        let set_bits = self.bits.iter().filter(|&&bit| bit).count();
        let load_factor = set_bits as f64 / self.size as f64;
        
        BloomFilterStats {
            size: self.size,
            hash_count: self.hash_count,
            set_bits,
            load_factor,
        }
    }

    /// Generate hash values for item
    fn hash_item<T: Hash>(&self, item: &T) -> Vec<usize> {
        let mut hashes = Vec::with_capacity(self.hash_count);
        
        for i in 0..self.hash_count {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            item.hash(&mut hasher);
            i.hash(&mut hasher); // Different hash for each function
            hashes.push(hasher.finish() as usize);
        }
        
        hashes
    }

    /// Calculate optimal filter size
    fn optimal_size(expected_items: usize, false_positive_rate: f64) -> usize {
        let ln2 = std::f64::consts::LN_2;
        (-(expected_items as f64) * false_positive_rate.ln() / (ln2 * ln2)).ceil() as usize
    }

    /// Calculate optimal hash function count
    fn optimal_hash_count(expected_items: usize, size: usize) -> usize {
        let ln2 = std::f64::consts::LN_2;
        ((size as f64 / expected_items as f64) * ln2).round() as usize
    }
}

/// Bloom filter statistics
#[derive(Debug, Clone)]
pub struct BloomFilterStats {
    pub size: usize,
    pub hash_count: usize,
    pub set_bits: usize,
    pub load_factor: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_queue() {
        let mut pq = PriorityQueue::new();
        
        pq.push("low", 1).unwrap();
        pq.push("high", 10).unwrap();
        pq.push("medium", 5).unwrap();
        
        assert_eq!(pq.pop(), Some("high"));
        assert_eq!(pq.pop(), Some("medium"));
        assert_eq!(pq.pop(), Some("low"));
        assert_eq!(pq.pop(), None);
    }

    #[test]
    fn test_priority_queue_with_capacity() {
        let mut pq = PriorityQueue::with_capacity(2);
        
        pq.push("first", 1).unwrap();
        pq.push("second", 2).unwrap();
        pq.push("third", 3).unwrap(); // Should remove lowest priority
        
        assert_eq!(pq.len(), 2);
        assert_eq!(pq.pop(), Some("third"));
        assert_eq!(pq.pop(), Some("second"));
    }

    #[test]
    fn test_lru_cache() {
        let mut cache = LRUCache::new(2).unwrap();
        
        assert_eq!(cache.put("a", 1), None);
        assert_eq!(cache.put("b", 2), None);
        assert_eq!(*cache.get(&"a").unwrap(), 1);
        
        // This should evict "b"
        assert_eq!(cache.put("c", 3), None);
        assert!(cache.get(&"b").is_none());
        assert_eq!(*cache.get(&"a").unwrap(), 1);
        assert_eq!(*cache.get(&"c").unwrap(), 3);
    }

    #[test]
    fn test_ring_buffer() {
        let mut rb = RingBuffer::new(3).unwrap();
        
        assert_eq!(rb.push(1), None);
        assert_eq!(rb.push(2), None);
        assert_eq!(rb.push(3), None);
        assert_eq!(rb.push(4), Some(1)); // Should overwrite oldest
        
        assert_eq!(rb.front(), Some(&2));
        assert_eq!(rb.back(), Some(&4));
        assert_eq!(rb.pop(), Some(2));
        assert_eq!(rb.len(), 2);
    }

    #[test]
    fn test_merkle_tree() {
        let data = vec![b"block1", b"block2", b"block3"];
        let tree = MerkleTree::new(data);
        
        assert_eq!(tree.leaf_count(), 3);
        assert!(tree.root().is_some());
        assert_eq!(tree.root().unwrap().len(), 32); // SHA256 hash
    }

    #[test]
    fn test_merkle_tree_empty() {
        let tree = MerkleTree::new(Vec::<&[u8]>::new());
        assert_eq!(tree.leaf_count(), 0);
        assert!(tree.is_empty());
    }

    #[test]
    fn test_time_series() {
        let mut ts = TimeSeries::new();
        
        ts.add_point(10);
        ts.add_point(20);
        ts.add_point(30);
        
        assert_eq!(ts.len(), 3);
        assert_eq!(ts.latest().unwrap().value, 30);
        assert_eq!(ts.oldest().unwrap().value, 10);
    }

    #[test]
    fn test_time_series_with_max_points() {
        let mut ts = TimeSeries::new().with_max_points(2);
        
        ts.add_point(10);
        ts.add_point(20);
        ts.add_point(30); // Should remove oldest
        
        assert_eq!(ts.len(), 2);
        assert_eq!(ts.oldest().unwrap().value, 20);
        assert_eq!(ts.latest().unwrap().value, 30);
    }

    #[test]
    fn test_bloom_filter() {
        let mut bf = BloomFilter::new(100, 0.01).unwrap();
        
        bf.add(&"hello");
        bf.add(&"world");
        
        assert!(bf.contains(&"hello"));
        assert!(bf.contains(&"world"));
        assert!(!bf.contains(&"missing")); // Might have false positives but not false negatives
    }

    #[test]
    fn test_bloom_filter_stats() {
        let mut bf = BloomFilter::new(10, 0.1).unwrap();
        
        bf.add(&"test1");
        bf.add(&"test2");
        
        let stats = bf.stats();
        assert!(stats.set_bits > 0);
        assert!(stats.load_factor > 0.0);
    }

    #[test]
    fn test_time_series_point() {
        let point = TimeSeriesPoint::new(42);
        assert_eq!(point.value, 42);
        assert!(point.age() < Duration::from_secs(1)); // Just created
    }
}

