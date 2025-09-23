# CC Chain Core Algorithms

This crate implements fundamental algorithms and data structures used throughout the CC Chain ecosystem, providing efficient and optimized implementations for blockchain operations.

## 🧩 Data Structures

### 🌳 Merkle Tree
Efficient cryptographic tree structure for data verification and integrity checking.

```rust
use cc_core_algorithms::MerkleTree;

let data_items = vec![b"block1", b"block2", b"block3", b"block4"];
let data: Vec<&[u8]> = data_items.iter().map(|&x| x as &[u8]).collect();
let tree = MerkleTree::new(&data)?;

// Get root hash
let root = tree.root().unwrap();

// Generate inclusion proof
let proof = tree.prove(0)?;

// Verify proof
let is_valid = MerkleTree::verify(&root, &proof, 0, b"block1");
assert!(is_valid);
```

**Features:**
- Efficient proof generation and verification
- Minimal storage requirements
- Cryptographically secure

### 🌸 Bloom Filter
Probabilistic data structure for fast membership testing.

```rust
use cc_core_algorithms::BloomFilter;

let mut filter = BloomFilter::new(1000, 0.01); // 1000 capacity, 1% false positive rate

// Insert items
filter.insert(b"transaction_hash_1");
filter.insert(b"transaction_hash_2");

// Check membership
assert!(filter.contains(b"transaction_hash_1"));
assert!(!filter.contains(b"nonexistent_hash"));

// Check false positive probability
let fp_rate = filter.false_positive_probability();
println!("Current false positive rate: {:.4}%", fp_rate * 100.0);
```

**Features:**
- Space-efficient membership testing
- Configurable false positive rates
- No false negatives
- Optimal parameter calculation

### 💾 LRU Cache
Least Recently Used cache implementation for efficient data caching.

```rust
use cc_core_algorithms::LRUCache;

let mut cache = LRUCache::new(100); // Capacity: 100 items

// Insert items
cache.put("key1", "value1");
cache.put("key2", "value2");

// Retrieve items (moves to front)
let value = cache.get(&"key1");
assert_eq!(value, Some("value1"));

// Cache automatically evicts least recently used items when full
```

**Features:**
- O(1) insertion and retrieval
- Automatic eviction of old items
- Thread-safe operations
- Configurable capacity

### 🦘 Skip List
Probabilistic data structure for fast ordered operations.

```rust
use cc_core_algorithms::SkipList;

let mut skiplist = SkipList::new();

// Insert values
skiplist.insert(42);
skiplist.insert(17);
skiplist.insert(89);

// Search for values
assert!(skiplist.contains(&42));
assert!(!skiplist.contains(&100));
```

**Features:**
- O(log n) search, insertion, and deletion
- Self-balancing structure
- Simple implementation
- Good cache performance

### 🔄 Consistent Hashing
Distributed hash table implementation for load balancing.

```rust
use cc_core_algorithms::ConsistentHash;

let mut hash_ring = ConsistentHash::new(3); // 3 virtual nodes per physical node

// Add nodes
hash_ring.add_node("node1".to_string());
hash_ring.add_node("node2".to_string());
hash_ring.add_node("node3".to_string());

// Find responsible node for a key
let node = hash_ring.get_node("my_data_key");
println!("Data should be stored on: {:?}", node);

// Remove a node (data redistributes automatically)
hash_ring.remove_node("node2");
```

**Features:**
- Minimal data movement on node changes
- Virtual nodes for better distribution
- Load balancing across nodes
- Fault tolerance

### 🌲 Trie (Prefix Tree)
Efficient prefix-based string operations.

```rust
use cc_core_algorithms::Trie;

let mut trie = Trie::new();

// Insert words with values
trie.insert("hello", "greeting".to_string());
trie.insert("help", "assistance".to_string());
trie.insert("world", "planet".to_string());

// Search for words
assert_eq!(trie.search("hello"), Some(&"greeting".to_string()));

// Find words with prefix
let words = trie.words_with_prefix("hel");
assert_eq!(words.len(), 2); // "hello" and "help"
```

**Features:**
- Fast prefix matching
- Space-efficient for shared prefixes
- Autocomplete functionality
- Dictionary operations

## 🔧 Sorting Algorithms

### Quick Sort
Fast, in-place sorting algorithm.

```rust
use cc_core_algorithms::sorting::quick_sort;

let mut data = vec![64, 34, 25, 12, 22, 11, 90];
quick_sort(&mut data);
assert_eq!(data, vec![11, 12, 22, 25, 34, 64, 90]);
```

### Merge Sort
Stable, divide-and-conquer sorting algorithm.

```rust
use cc_core_algorithms::sorting::merge_sort;

let mut data = vec![64, 34, 25, 12, 22, 11, 90];
merge_sort(&mut data);
assert_eq!(data, vec![11, 12, 22, 25, 34, 64, 90]);
```

**Algorithm Comparison:**

| Algorithm  | Time Complexity | Space | Stable | In-place |
|------------|-----------------|-------|--------|----------|
| Quick Sort | O(n log n) avg  | O(log n) | No   | Yes      |
| Merge Sort | O(n log n)      | O(n)  | Yes    | No       |

## 🎯 Use Cases

### Blockchain Applications
- **Merkle Trees**: Block verification, state root calculation
- **Bloom Filters**: Transaction filtering, peer discovery
- **Consistent Hashing**: Shard distribution, node assignment

### Performance Optimization
- **LRU Cache**: Block cache, transaction cache, state cache
- **Skip Lists**: Ordered transaction pools, sorted indexes
- **Sorting**: Transaction ordering, priority queues

### Data Management
- **Tries**: Address indexing, contract name resolution
- **Hash Rings**: Data partitioning, load balancing

## 🔬 Algorithm Analysis

### Time Complexities

| Data Structure | Insert | Search | Delete | Space |
|----------------|--------|--------|--------|-------|
| Merkle Tree    | O(n)   | O(log n) | N/A  | O(n)  |
| Bloom Filter   | O(k)   | O(k)   | N/A    | O(m)  |
| LRU Cache      | O(1)   | O(1)   | O(1)   | O(n)  |
| Skip List      | O(log n)| O(log n)| O(log n)| O(n) |
| Consistent Hash| O(1)   | O(log n)| O(1)   | O(n)  |
| Trie           | O(m)   | O(m)   | O(m)   | O(ALPHABET_SIZE * N * M) |

*Where: n = number of elements, k = number of hash functions, m = length of string, M = number of strings*

### Space Efficiency

- **Bloom Filter**: Most space-efficient for membership testing
- **Trie**: Efficient for shared prefixes
- **Skip List**: Good balance of time and space
- **LRU Cache**: Constant overhead per item
- **Consistent Hash**: Minimal overhead for distribution

## 🧪 Testing

Run the comprehensive test suite:

```bash
cargo test --package cc-core-algorithms
```

### Test Coverage
- ✅ Merkle tree proof generation and verification
- ✅ Bloom filter false positive rates
- ✅ LRU cache eviction policies
- ✅ Skip list ordering guarantees
- ✅ Consistent hashing distribution
- ✅ Trie prefix operations
- ✅ Sorting algorithm correctness

## 🚀 Performance Benchmarks

Run benchmarks to measure performance:

```bash
cargo bench --package cc-core-algorithms
```

Example benchmark results (on modern hardware):
- Merkle tree proof generation: ~50 μs for 1000 items
- Bloom filter insertion: ~100 ns per item
- LRU cache operations: ~10 ns per operation
- Skip list search: ~200 ns per operation

## 🔧 Configuration

### Bloom Filter Tuning

```rust
// High precision (lower false positive rate)
let precise_filter = BloomFilter::new(1000, 0.001); // 0.1% false positive

// Memory efficient (higher false positive rate)
let compact_filter = BloomFilter::new(1000, 0.05);  // 5% false positive
```

### Consistent Hashing Tuning

```rust
// More virtual nodes = better distribution, more memory
let high_distribution = ConsistentHash::new(100);

// Fewer virtual nodes = less memory, potential hotspots
let memory_efficient = ConsistentHash::new(3);
```

## 🛠️ Advanced Usage

### Custom Hash Functions

The algorithms use secure hash functions by default, but you can extend them for specific use cases:

```rust
// Custom hash implementation for specific data types
impl CustomHashable for MyDataType {
    fn custom_hash(&self) -> u64 {
        // Your custom hashing logic
    }
}
```

### Thread Safety

Most data structures are not thread-safe by default for performance. For concurrent access:

```rust
use std::sync::{Arc, Mutex};

let cache = Arc::new(Mutex::new(LRUCache::new(100)));

// Share across threads
let cache_clone = Arc::clone(&cache);
std::thread::spawn(move || {
    let mut cache = cache_clone.lock().unwrap();
    cache.put("key", "value");
});
```

## 📊 Memory Usage

### Typical Memory Footprints

- **Bloom Filter (1M items, 1% FP)**: ~1.2 MB
- **LRU Cache (1000 items)**: ~24 KB + item size
- **Skip List (1000 items)**: ~32 KB + item size
- **Trie (1000 words, avg 8 chars)**: ~200 KB
- **Merkle Tree (1000 leaves)**: ~64 KB

## 🤝 Contributing

Want to add more algorithms? See [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

Potential additions:
- B+ Trees for database indexes
- Red-Black Trees for balanced operations
- Suffix arrays for string operations
- Fenwick trees for range queries
- Segment trees for interval operations

## 📚 Further Reading

- [Introduction to Algorithms (CLRS)](https://mitpress.mit.edu/books/introduction-algorithms)
- [Bloom Filters by Example](https://llimllib.github.io/bloomfilter-tutorial/)
- [Skip Lists: A Probabilistic Alternative to Balanced Trees](https://15721.courses.cs.cmu.edu/spring2018/papers/08-oltpindexes1/pugh-skiplists-cacm1990.pdf)
- [Consistent Hashing and Random Trees](https://www.akamai.com/us/en/multimedia/documents/technical-publication/consistent-hashing-and-random-trees-distributed-caching-protocols-for-relieving-hot-spots-on-the-world-wide-web-technical-publication.pdf)