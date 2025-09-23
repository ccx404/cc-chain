//! CC Chain Core Algorithms
//!
//! This crate implements fundamental algorithms used throughout CC Chain,
//! including cryptographic primitives, data structures, and optimization algorithms.

use std::collections::{HashMap, VecDeque};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AlgorithmError {
    #[error("Merkle tree error: {0}")]
    MerkleTree(String),
    #[error("Bloom filter error: {0}")]
    BloomFilter(String),
    #[error("Sorting algorithm error: {0}")]
    Sorting(String),
    #[error("Compression error: {0}")]
    Compression(String),
}

pub type Result<T> = std::result::Result<T, AlgorithmError>;

/// Merkle Tree implementation for efficient data verification
#[derive(Debug, Clone)]
pub struct MerkleTree {
    nodes: Vec<Vec<[u8; 32]>>,
    leaf_count: usize,
}

/// Bloom Filter for probabilistic membership testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BloomFilter {
    bit_array: Vec<bool>,
    hash_functions: u32,
    size: usize,
    item_count: usize,
}

/// LRU Cache implementation for efficient caching
#[derive(Debug)]
pub struct LRUCache<K, V> {
    capacity: usize,
    map: HashMap<K, usize>,
    entries: VecDeque<(K, V)>,
}

/// Skip List for fast searching in ordered data
#[derive(Debug)]
pub struct SkipList<T> {
    levels: Vec<Vec<SkipListNode<T>>>,
    max_level: usize,
    size: usize,
}

#[derive(Debug, Clone)]
struct SkipListNode<T> {
    value: T,
    next: Vec<Option<usize>>,
}

/// Consistent Hashing for distributed data placement
#[derive(Debug)]
pub struct ConsistentHash {
    ring: std::collections::BTreeMap<u64, String>,
    virtual_nodes: u32,
}

/// Trie data structure for efficient prefix matching
#[derive(Debug)]
pub struct Trie {
    root: TrieNode,
    size: usize,
}

#[derive(Debug)]
struct TrieNode {
    children: HashMap<char, Box<TrieNode>>,
    is_end: bool,
    value: Option<String>,
}

impl MerkleTree {
    /// Create a new Merkle tree from data items
    pub fn new(data: &[&[u8]]) -> Result<Self> {
        if data.is_empty() {
            return Err(AlgorithmError::MerkleTree("Cannot create tree from empty data".to_string()));
        }

        let leaf_count = data.len();
        let mut nodes = Vec::new();

        // Create leaf level (level 0)
        let mut leaves = Vec::new();
        for item in data {
            leaves.push(Self::hash(item));
        }
        nodes.push(leaves);

        // Build tree bottom-up
        let mut current_level = 0;
        while nodes[current_level].len() > 1 {
            let mut next_level = Vec::new();
            let current_nodes = &nodes[current_level];
            
            for chunk in current_nodes.chunks(2) {
                let hash = if chunk.len() == 2 {
                    Self::hash_pair(&chunk[0], &chunk[1])
                } else {
                    chunk[0] // Odd number, promote single node
                };
                next_level.push(hash);
            }
            
            nodes.push(next_level);
            current_level += 1;
        }

        Ok(Self { nodes, leaf_count })
    }

    /// Get the root hash of the tree
    pub fn root(&self) -> Option<[u8; 32]> {
        self.nodes.last().and_then(|level| level.first()).copied()
    }

    /// Generate inclusion proof for a given index
    pub fn prove(&self, index: usize) -> Result<Vec<[u8; 32]>> {
        if index >= self.leaf_count {
            return Err(AlgorithmError::MerkleTree("Index out of bounds".to_string()));
        }

        let mut proof = Vec::new();
        let mut current_index = index;

        for level in 0..self.nodes.len() - 1 {
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };

            if sibling_index < self.nodes[level].len() {
                proof.push(self.nodes[level][sibling_index]);
            }

            current_index /= 2;
        }

        Ok(proof)
    }

    /// Verify an inclusion proof
    pub fn verify(root: &[u8; 32], proof: &[[u8; 32]], index: usize, data: &[u8]) -> bool {
        let mut current_hash = Self::hash(data);
        let mut current_index = index;

        for &sibling_hash in proof {
            current_hash = if current_index % 2 == 0 {
                Self::hash_pair(&current_hash, &sibling_hash)
            } else {
                Self::hash_pair(&sibling_hash, &current_hash)
            };
            current_index /= 2;
        }

        current_hash == *root
    }

    fn hash(data: &[u8]) -> [u8; 32] {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        let hash = hasher.finish();
        
        let mut result = [0u8; 32];
        result[..8].copy_from_slice(&hash.to_le_bytes());
        result
    }

    fn hash_pair(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
        let mut combined = Vec::with_capacity(64);
        combined.extend_from_slice(left);
        combined.extend_from_slice(right);
        Self::hash(&combined)
    }
}

impl BloomFilter {
    /// Create a new Bloom filter with specified capacity and false positive rate
    pub fn new(capacity: usize, false_positive_rate: f64) -> Self {
        let size = Self::optimal_size(capacity, false_positive_rate);
        let hash_functions = Self::optimal_hash_functions(capacity, size);

        Self {
            bit_array: vec![false; size],
            hash_functions,
            size,
            item_count: 0,
        }
    }

    /// Insert an item into the filter
    pub fn insert(&mut self, item: &[u8]) {
        for i in 0..self.hash_functions {
            let hash = self.hash(item, i);
            let index = (hash as usize) % self.size;
            self.bit_array[index] = true;
        }
        self.item_count += 1;
    }

    /// Check if an item might be in the set
    pub fn contains(&self, item: &[u8]) -> bool {
        for i in 0..self.hash_functions {
            let hash = self.hash(item, i);
            let index = (hash as usize) % self.size;
            if !self.bit_array[index] {
                return false;
            }
        }
        true
    }

    /// Get current false positive probability
    pub fn false_positive_probability(&self) -> f64 {
        if self.item_count == 0 {
            return 0.0;
        }

        let ratio = self.bit_array.iter().filter(|&&bit| bit).count() as f64 / self.size as f64;
        ratio.powi(self.hash_functions as i32)
    }

    fn optimal_size(capacity: usize, fp_rate: f64) -> usize {
        let ln2 = std::f64::consts::LN_2;
        (-(capacity as f64 * fp_rate.ln()) / (ln2 * ln2)).ceil() as usize
    }

    fn optimal_hash_functions(capacity: usize, size: usize) -> u32 {
        let ln2 = std::f64::consts::LN_2;
        ((size as f64 / capacity as f64) * ln2).round() as u32
    }

    fn hash(&self, item: &[u8], seed: u32) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        item.hash(&mut hasher);
        hasher.finish()
    }
}

impl<K: Clone + std::hash::Hash + Eq, V: Clone> LRUCache<K, V> {
    /// Create a new LRU cache with specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            map: HashMap::new(),
            entries: VecDeque::new(),
        }
    }

    /// Get a value from the cache
    pub fn get(&mut self, key: &K) -> Option<V> {
        if let Some(&index) = self.map.get(key) {
            // Move to front (most recently used)
            let entry = self.entries.remove(index).unwrap();
            self.entries.push_front(entry.clone());
            
            // Update indices in map
            self.update_indices();
            
            Some(entry.1)
        } else {
            None
        }
    }

    /// Insert a key-value pair into the cache
    pub fn put(&mut self, key: K, value: V) {
        if self.map.contains_key(&key) {
            // Update existing entry
            if let Some(&index) = self.map.get(&key) {
                self.entries.remove(index);
                self.entries.push_front((key.clone(), value));
                self.update_indices();
            }
        } else {
            // Insert new entry
            if self.entries.len() >= self.capacity {
                // Remove least recently used
                if let Some((old_key, _)) = self.entries.pop_back() {
                    self.map.remove(&old_key);
                }
            }
            
            self.entries.push_front((key.clone(), value));
            self.map.insert(key, 0);
            self.update_indices();
        }
    }

    fn update_indices(&mut self) {
        self.map.clear();
        for (index, (key, _)) in self.entries.iter().enumerate() {
            self.map.insert(key.clone(), index);
        }
    }
}

impl<T: Clone + PartialOrd> SkipList<T> {
    /// Create a new skip list
    pub fn new() -> Self {
        Self {
            levels: vec![Vec::new()],
            max_level: 16,
            size: 0,
        }
    }

    /// Insert a value into the skip list
    pub fn insert(&mut self, value: T) {
        let level = self.random_level();
        
        // Ensure we have enough levels
        while self.levels.len() <= level {
            self.levels.push(Vec::new());
        }

        let node = SkipListNode {
            value: value.clone(),
            next: vec![None; level + 1],
        };

        // Insert at each level
        for i in 0..=level {
            self.levels[i].push(node.clone());
        }

        self.size += 1;
    }

    /// Search for a value in the skip list
    pub fn contains(&self, value: &T) -> bool {
        if self.levels.is_empty() || self.levels[0].is_empty() {
            return false;
        }

        for level in self.levels.iter().rev() {
            for node in level {
                if &node.value == value {
                    return true;
                } else if node.value > *value {
                    break;
                }
            }
        }

        false
    }

    fn random_level(&self) -> usize {
        let mut level = 0;
        while level < self.max_level && rand::random::<f64>() < 0.5 {
            level += 1;
        }
        level
    }
}

impl ConsistentHash {
    /// Create a new consistent hash ring
    pub fn new(virtual_nodes: u32) -> Self {
        Self {
            ring: std::collections::BTreeMap::new(),
            virtual_nodes,
        }
    }

    /// Add a node to the ring
    pub fn add_node(&mut self, node_id: String) {
        for i in 0..self.virtual_nodes {
            let virtual_node = format!("{}:{}", node_id, i);
            let hash = self.hash(&virtual_node);
            self.ring.insert(hash, node_id.clone());
        }
    }

    /// Remove a node from the ring
    pub fn remove_node(&mut self, node_id: &str) {
        let keys_to_remove: Vec<u64> = self.ring
            .iter()
            .filter(|(_, v)| *v == node_id)
            .map(|(k, _)| *k)
            .collect();

        for key in keys_to_remove {
            self.ring.remove(&key);
        }
    }

    /// Get the node responsible for a given key
    pub fn get_node(&self, key: &str) -> Option<String> {
        if self.ring.is_empty() {
            return None;
        }

        let hash = self.hash(key);
        
        // Find the first node with hash >= key hash
        if let Some((_, node)) = self.ring.range(hash..).next() {
            Some(node.clone())
        } else {
            // Wrap around to the first node
            self.ring.values().next().cloned()
        }
    }

    fn hash(&self, data: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }
}

impl Trie {
    /// Create a new trie
    pub fn new() -> Self {
        Self {
            root: TrieNode {
                children: HashMap::new(),
                is_end: false,
                value: None,
            },
            size: 0,
        }
    }

    /// Insert a word with associated value
    pub fn insert(&mut self, word: &str, value: String) {
        let mut current = &mut self.root;
        
        for ch in word.chars() {
            current = current.children
                .entry(ch)
                .or_insert_with(|| Box::new(TrieNode {
                    children: HashMap::new(),
                    is_end: false,
                    value: None,
                }));
        }
        
        if !current.is_end {
            self.size += 1;
        }
        current.is_end = true;
        current.value = Some(value);
    }

    /// Search for a word in the trie
    pub fn search(&self, word: &str) -> Option<&String> {
        let mut current = &self.root;
        
        for ch in word.chars() {
            if let Some(node) = current.children.get(&ch) {
                current = node;
            } else {
                return None;
            }
        }
        
        if current.is_end {
            current.value.as_ref()
        } else {
            None
        }
    }

    /// Find all words with given prefix
    pub fn words_with_prefix(&self, prefix: &str) -> Vec<String> {
        let mut result = Vec::new();
        let mut current = &self.root;
        
        // Navigate to prefix
        for ch in prefix.chars() {
            if let Some(node) = current.children.get(&ch) {
                current = node;
            } else {
                return result; // Prefix not found
            }
        }
        
        // Collect all words from this point
        self.collect_words(current, prefix, &mut result);
        result
    }

    fn collect_words(&self, node: &TrieNode, prefix: &str, result: &mut Vec<String>) {
        if node.is_end {
            if let Some(value) = &node.value {
                result.push(value.clone());
            }
        }
        
        for (ch, child) in &node.children {
            let new_prefix = format!("{}{}", prefix, ch);
            self.collect_words(child, &new_prefix, result);
        }
    }
}

/// Sorting algorithms
pub mod sorting {
    /// Quick sort implementation
    pub fn quick_sort<T: PartialOrd + Clone>(arr: &mut [T]) {
        if arr.len() <= 1 {
            return;
        }
        
        let pivot_index = partition(arr);
        quick_sort(&mut arr[0..pivot_index]);
        quick_sort(&mut arr[pivot_index + 1..]);
    }

    fn partition<T: PartialOrd + Clone>(arr: &mut [T]) -> usize {
        let pivot_index = arr.len() - 1;
        let mut i = 0;
        
        for j in 0..pivot_index {
            if arr[j] <= arr[pivot_index] {
                arr.swap(i, j);
                i += 1;
            }
        }
        
        arr.swap(i, pivot_index);
        i
    }

    /// Merge sort implementation
    pub fn merge_sort<T: PartialOrd + Clone>(arr: &mut [T]) {
        if arr.len() <= 1 {
            return;
        }
        
        let mid = arr.len() / 2;
        let mut left = arr[0..mid].to_vec();
        let mut right = arr[mid..].to_vec();
        
        merge_sort(&mut left);
        merge_sort(&mut right);
        
        merge(&left, &right, arr);
    }

    fn merge<T: PartialOrd + Clone>(left: &[T], right: &[T], result: &mut [T]) {
        let mut i = 0;
        let mut j = 0;
        let mut k = 0;
        
        while i < left.len() && j < right.len() {
            if left[i] <= right[j] {
                result[k] = left[i].clone();
                i += 1;
            } else {
                result[k] = right[j].clone();
                j += 1;
            }
            k += 1;
        }
        
        while i < left.len() {
            result[k] = left[i].clone();
            i += 1;
            k += 1;
        }
        
        while j < right.len() {
            result[k] = right[j].clone();
            j += 1;
            k += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_tree() {
        let data_items = vec![b"block1", b"block2", b"block3", b"block4"];
        let data: Vec<&[u8]> = data_items.iter().map(|&x| x as &[u8]).collect();
        let tree = MerkleTree::new(&data).unwrap();
        
        assert!(tree.root().is_some());
        
        let proof = tree.prove(0).unwrap();
        let root = tree.root().unwrap();
        assert!(MerkleTree::verify(&root, &proof, 0, b"block1"));
    }

    #[test]
    fn test_bloom_filter() {
        let mut filter = BloomFilter::new(1000, 0.01);
        
        filter.insert(b"hello");
        filter.insert(b"world");
        
        assert!(filter.contains(b"hello"));
        assert!(filter.contains(b"world"));
        assert!(!filter.contains(b"foo"));
    }

    #[test]
    fn test_lru_cache() {
        let mut cache = LRUCache::new(2);
        
        cache.put("a", 1);
        cache.put("b", 2);
        
        assert_eq!(cache.get(&"a"), Some(1));
        
        cache.put("c", 3); // Should evict "b"
        assert_eq!(cache.get(&"b"), None);
        assert_eq!(cache.get(&"a"), Some(1));
        assert_eq!(cache.get(&"c"), Some(3));
    }

    #[test]
    fn test_consistent_hash() {
        let mut hash_ring = ConsistentHash::new(3);
        
        hash_ring.add_node("node1".to_string());
        hash_ring.add_node("node2".to_string());
        hash_ring.add_node("node3".to_string());
        
        let node = hash_ring.get_node("test_key");
        assert!(node.is_some());
        
        hash_ring.remove_node("node2");
        let new_node = hash_ring.get_node("test_key");
        assert!(new_node.is_some());
    }

    #[test]
    fn test_trie() {
        let mut trie = Trie::new();
        
        trie.insert("hello", "greeting".to_string());
        trie.insert("help", "assistance".to_string());
        trie.insert("world", "planet".to_string());
        
        assert_eq!(trie.search("hello"), Some(&"greeting".to_string()));
        assert_eq!(trie.search("help"), Some(&"assistance".to_string()));
        assert_eq!(trie.search("hi"), None);
        
        let words = trie.words_with_prefix("hel");
        assert_eq!(words.len(), 2);
    }

    #[test]
    fn test_sorting_algorithms() {
        let mut arr1 = vec![64, 34, 25, 12, 22, 11, 90];
        let mut arr2 = arr1.clone();
        
        sorting::quick_sort(&mut arr1);
        sorting::merge_sort(&mut arr2);
        
        let expected = vec![11, 12, 22, 25, 34, 64, 90];
        assert_eq!(arr1, expected);
        assert_eq!(arr2, expected);
    }
}

