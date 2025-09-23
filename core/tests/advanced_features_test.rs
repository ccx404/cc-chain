use cc_core::*;
use std::time::Duration;

#[test]
fn test_signature_aggregation() {
    let mut aggregator = SignatureAggregator::new();
    
    // Create test keypairs and messages
    let keypair1 = CCKeypair::generate();
    let keypair2 = CCKeypair::generate();
    let keypair3 = CCKeypair::generate();
    
    let msg1 = b"message 1".to_vec();
    let msg2 = b"message 2".to_vec();
    let msg3 = b"message 3".to_vec();
    
    let sig1 = keypair1.sign(&msg1);
    let sig2 = keypair2.sign(&msg2);
    let sig3 = keypair3.sign(&msg3);
    
    // Add signatures to aggregator
    aggregator.add_signature(sig1, keypair1.public_key(), msg1);
    aggregator.add_signature(sig2, keypair2.public_key(), msg2);
    aggregator.add_signature(sig3, keypair3.public_key(), msg3);
    
    assert_eq!(aggregator.len(), 3);
    assert!(!aggregator.is_empty());
    
    // Verify batch
    assert!(aggregator.verify_batch());
    
    // Clear and test empty state
    aggregator.clear();
    assert_eq!(aggregator.len(), 0);
    assert!(aggregator.is_empty());
}

#[test]
fn test_quantum_resistant_signatures() {
    let keypair = CCKeypair::generate();
    let message = b"quantum test message";
    
    let quantum_sig = QuantumResistantSignature::sign_quantum(&keypair, message);
    assert!(quantum_sig.verify_quantum(&keypair.public_key(), message));
}

#[test]
fn test_hash_cache() {
    let mut cache = HashCache::new(100);
    
    let data1 = b"test data 1";
    let data2 = b"test data 2";
    
    // First computation should cache the result
    let hash1 = cache.get_or_compute(data1);
    let hash2 = cache.get_or_compute(data1); // Should be cached
    assert_eq!(hash1, hash2);
    
    // Different data should produce different hash
    let hash3 = cache.get_or_compute(data2);
    assert_ne!(hash1, hash3);
    
    // Check cache stats
    let (size, max_size) = cache.stats();
    assert_eq!(size, 2);
    assert_eq!(max_size, 100);
    
    cache.clear();
    let (size, _) = cache.stats();
    assert_eq!(size, 0);
}

#[test]
fn test_parallel_hash_computation() {
    let data_pieces = vec![
        b"data1".as_slice(),
        b"data2".as_slice(),
        b"data3".as_slice(),
        b"data4".as_slice(),
    ];
    
    let hashes = parallel_hash_multiple(&data_pieces);
    assert_eq!(hashes.len(), 4);
    
    // Verify hashes are correct
    for (i, hash) in hashes.iter().enumerate() {
        let expected = crate::crypto::hash(data_pieces[i]);
        assert_eq!(*hash, expected);
    }
}

#[test]
fn test_multi_hash() {
    let data = b"test data for multi hash";
    let multi_hash = multi_hash(data);
    
    // Verify both hashes are computed
    let expected_blake3 = crate::crypto::hash(data);
    assert_eq!(multi_hash.blake3, expected_blake3);
    
    // Blake3 and SHA256 should produce different results
    assert_ne!(multi_hash.blake3, multi_hash.sha256);
}

#[test]
fn test_merkle_proof_generation_and_verification() {
    let leaves = vec![
        crate::crypto::hash(b"leaf1"),
        crate::crypto::hash(b"leaf2"),
        crate::crypto::hash(b"leaf3"),
        crate::crypto::hash(b"leaf4"),
    ];
    
    let tree = MerkleTree::build(&leaves);
    let root = tree.root();
    
    // Generate and verify proof for each leaf
    for (i, leaf) in leaves.iter().enumerate() {
        if let Some(proof) = tree.proof(i) {
            assert!(MerkleTree::verify_proof(&root, leaf, &proof, i));
        } else {
            panic!("Failed to generate proof for leaf {}", i);
        }
    }
}

#[test]
fn test_parallel_transaction_processor() {
    let processor = ParallelTransactionProcessor::new(Some(4), 100);
    
    // Create test transactions
    let keypair1 = CCKeypair::generate();
    let keypair2 = CCKeypair::generate();
    let keypair3 = CCKeypair::generate();
    
    let mut tx1 = Transaction::new(keypair1.public_key(), keypair2.public_key(), 1000, 10, 0, vec![]);
    let mut tx2 = Transaction::new(keypair2.public_key(), keypair3.public_key(), 2000, 20, 0, vec![]);
    let mut tx3 = Transaction::new(keypair3.public_key(), keypair1.public_key(), 3000, 30, 0, vec![]);
    
    tx1.sign(&keypair1);
    tx2.sign(&keypair2);
    tx3.sign(&keypair3);
    
    let transactions = vec![tx1, tx2, tx3];
    
    // Test parallel signature verification
    let verification_results = processor.verify_signatures_parallel(&transactions);
    assert_eq!(verification_results.len(), 3);
    for result in verification_results {
        assert!(result);
    }
    
    // Test parallel execution with custom function
    let sizes = processor.execute_parallel(&transactions, |tx| tx.size());
    assert_eq!(sizes.len(), 3);
    for size in sizes {
        assert!(size > 0);
    }
}

#[test]
fn test_transaction_batch() {
    let keypair1 = CCKeypair::generate();
    let keypair2 = CCKeypair::generate();
    
    let mut tx1 = Transaction::new(keypair1.public_key(), keypair2.public_key(), 1000, 10, 0, vec![]);
    let mut tx2 = Transaction::new(keypair2.public_key(), keypair1.public_key(), 2000, 20, 1, vec![]);
    
    tx1.sign(&keypair1);
    tx2.sign(&keypair2);
    
    let transactions = vec![tx1, tx2];
    let batch = TransactionBatch::new(transactions);
    
    assert_eq!(batch.metadata.tx_count, 2);
    assert_eq!(batch.metadata.avg_fee, 15); // (10 + 20) / 2
    assert!(batch.metadata.size_bytes > 0);
    assert!(batch.metadata.priority_score > 0.0);
    
    // Test batch validation
    let validation_results = batch.validate_all();
    assert_eq!(validation_results.len(), 2);
    
    // Test batch splitting
    let chunks = batch.split_into_chunks(1);
    assert_eq!(chunks.len(), 2);
    
    // Test batch hash
    let hash = batch.batch_hash();
    assert_ne!(hash, [0u8; 32]);
}

#[test]
fn test_smart_batcher() {
    let mut batcher = SmartBatcher::new(3, 1000, Duration::from_millis(100));
    
    let keypair1 = CCKeypair::generate();
    let keypair2 = CCKeypair::generate();
    
    let mut tx1 = Transaction::new(keypair1.public_key(), keypair2.public_key(), 1000, 10, 0, vec![]);
    let mut tx2 = Transaction::new(keypair2.public_key(), keypair1.public_key(), 2000, 20, 1, vec![]);
    
    tx1.sign(&keypair1);
    tx2.sign(&keypair2);
    
    // Add transactions and check batching behavior
    assert_eq!(batcher.pending_count(), 0);
    
    let result1 = batcher.add_transaction(tx1);
    assert!(result1.is_none()); // Should not create batch yet
    assert_eq!(batcher.pending_count(), 1);
    
    let result2 = batcher.add_transaction(tx2);
    assert!(result2.is_none()); // Should not create batch yet
    assert_eq!(batcher.pending_count(), 2);
    
    // Force batch creation
    let batch = batcher.force_batch();
    assert!(batch.is_some());
    assert_eq!(batcher.pending_count(), 0);
    
    if let Some(batch) = batch {
        assert_eq!(batch.metadata.tx_count, 2);
    }
}

#[test]
fn test_state_snapshot_and_rollback() {
    let state_manager = StateManager::new();
    
    // Set up initial state
    let keypair1 = CCKeypair::generate();
    let keypair2 = CCKeypair::generate();
    let account1 = Account {
        balance: 1000,
        nonce: 0,
        storage_root: [0u8; 32],
        code_hash: [0u8; 32],
    };
    let account2 = Account {
        balance: 2000,
        nonce: 1,
        storage_root: [0u8; 32],
        code_hash: [0u8; 32],
    };
    
    state_manager.set_account(keypair1.public_key(), account1.clone());
    state_manager.set_account(keypair2.public_key(), account2.clone());
    
    // Create snapshot
    let snapshot = state_manager.create_snapshot();
    
    // Modify state
    let modified_account1 = Account {
        balance: 500,
        nonce: 1,
        storage_root: [0u8; 32],
        code_hash: [0u8; 32],
    };
    state_manager.set_account(keypair1.public_key(), modified_account1);
    
    // Verify state was modified
    let current_account1 = state_manager.get_account(&keypair1.public_key());
    assert_eq!(current_account1.balance, 500);
    assert_eq!(current_account1.nonce, 1);
    
    // Restore from snapshot
    state_manager.restore_from_snapshot(&snapshot);
    
    // Verify state was restored
    let restored_account1 = state_manager.get_account(&keypair1.public_key());
    assert_eq!(restored_account1.balance, 1000);
    assert_eq!(restored_account1.nonce, 0);
}

#[test]
fn test_state_cache() {
    let cache = StateCache::new(100, 50);
    
    let keypair = CCKeypair::generate();
    let account = Account {
        balance: 1000,
        nonce: 0,
        storage_root: [0u8; 32],
        code_hash: [0u8; 32],
    };
    
    // Test account caching
    assert!(cache.get_account(&keypair.public_key()).is_none());
    
    cache.put_account(keypair.public_key(), account.clone());
    
    let cached_account = cache.get_account(&keypair.public_key());
    assert!(cached_account.is_some());
    assert_eq!(cached_account.unwrap(), account);
    
    // Test state root caching
    let state_root = [1u8; 32];
    assert!(cache.get_state_root(100).is_none());
    
    cache.put_state_root(100, state_root);
    
    let cached_root = cache.get_state_root(100);
    assert!(cached_root.is_some());
    assert_eq!(cached_root.unwrap(), state_root);
    
    // Test cache statistics
    let stats = cache.get_stats();
    assert!(stats.account_requests > 0);
    assert!(stats.account_hits > 0);
    assert!(stats.state_root_requests > 0);
    assert!(stats.state_root_hits > 0);
    
    assert!(stats.account_hit_rate() > 0.0);
    assert!(stats.state_root_hit_rate() > 0.0);
    
    // Test cache clearing
    cache.clear_all();
    let cleared_stats = cache.get_stats();
    assert_eq!(cleared_stats.account_requests, 0);
    assert_eq!(cleared_stats.state_root_requests, 0);
}

#[test]
fn test_state_statistics() {
    let state_manager = StateManager::new();
    
    // Add some test data
    let keypair1 = CCKeypair::generate();
    let keypair2 = CCKeypair::generate();
    
    let account1 = Account {
        balance: 1000,
        nonce: 0,
        storage_root: [0u8; 32],
        code_hash: [0u8; 32],
    };
    
    let account2 = Account {
        balance: 2000,
        nonce: 1,
        storage_root: [0u8; 32],
        code_hash: [0u8; 32],
    };
    
    state_manager.set_account(keypair1.public_key(), account1);
    state_manager.set_account(keypair2.public_key(), account2);
    
    state_manager.add_validator(keypair1.public_key(), 5000);
    state_manager.add_validator(keypair2.public_key(), 3000);
    
    // Get statistics
    let stats = state_manager.get_state_stats();
    
    assert_eq!(stats.account_count, 2);
    assert_eq!(stats.validator_count, 2);
    assert_eq!(stats.total_balance, 3000); // 1000 + 2000
    assert_eq!(stats.total_validator_stake, 8000); // 5000 + 3000
}

#[test]
fn test_performance_monitor() {
    let monitor = PerformanceMonitor::new();
    
    // Record some test data
    monitor.record_block(10, Duration::from_millis(500));
    monitor.record_block(15, Duration::from_millis(600));
    monitor.record_confirmation(Duration::from_millis(100));
    monitor.record_confirmation(Duration::from_millis(200));
    
    let metrics = monitor.get_metrics();
    
    assert!(metrics.tps > 0.0);
    assert!(metrics.avg_block_time.as_millis() > 0);
    assert!(metrics.avg_confirmation_time.as_millis() > 0);
    
    // Test reset
    monitor.reset();
    let reset_metrics = monitor.get_metrics();
    assert_eq!(reset_metrics.tps, 0.0);
}

#[test]
fn test_adaptive_params() {
    let mut adaptive_params = AdaptiveParams::new();
    
    // Record some performance data
    adaptive_params.get_monitor().record_block(1000, Duration::from_millis(2500)); // Slow block
    adaptive_params.get_monitor().record_confirmation(Duration::from_secs(6)); // Slow confirmation
    
    let initial_params = adaptive_params.get_params();
    
    // Adapt parameters based on performance
    adaptive_params.adapt(100, Duration::from_millis(200));
    
    let adapted_params = adaptive_params.get_params();
    
    // Parameters should have changed due to poor performance
    assert!(adapted_params.0 >= initial_params.0); // Block time should increase
    assert!(adapted_params.2 >= initial_params.2); // Base fee should increase
}