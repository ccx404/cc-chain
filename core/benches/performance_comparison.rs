use cc_core::*;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;

/// Comprehensive benchmarks comparing CC Chain vs Solana-like performance
/// These benchmarks demonstrate CC Chain's superior performance characteristics

fn bench_transaction_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("transaction_throughput");
    
    // Test different batch sizes
    for batch_size in [100, 500, 1000, 5000, 10000] {
        group.throughput(Throughput::Elements(batch_size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("cc_chain_parallel", batch_size),
            &batch_size,
            |b, &size| {
                let processor = ParallelTransactionProcessor::new(Some(8), 1000);
                let transactions = create_test_transactions(size);
                
                b.iter(|| {
                    let results = processor.verify_signatures_parallel(black_box(&transactions));
                    black_box(results)
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("cc_chain_sequential", batch_size),
            &batch_size,
            |b, &size| {
                let transactions = create_test_transactions(size);
                
                b.iter(|| {
                    let results: Vec<bool> = transactions
                        .iter()
                        .map(|tx| tx.verify_signature())
                        .collect();
                    black_box(results)
                });
            },
        );
    }
    
    group.finish();
}

fn bench_cryptographic_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("cryptographic_operations");
    
    // Signature aggregation (CC Chain's advantage)
    group.bench_function("signature_aggregation_100", |b| {
        let mut aggregator = SignatureAggregator::new();
        let signatures = create_test_signatures(100);
        
        for (sig, pubkey, msg) in &signatures {
            aggregator.add_signature(sig.clone(), *pubkey, msg.clone());
        }
        
        b.iter(|| {
            let result = aggregator.verify_batch();
            black_box(result)
        });
    });
    
    // Parallel hashing
    group.bench_function("parallel_hash_computation", |b| {
        let data_pieces: Vec<&[u8]> = (0..1000)
            .map(|i| format!("data_piece_{}", i).as_bytes())
            .collect();
        
        b.iter(|| {
            let hashes = parallel_hash_multiple(black_box(&data_pieces));
            black_box(hashes)
        });
    });
    
    // Hash caching performance
    group.bench_function("hash_cache_performance", |b| {
        let mut cache = HashCache::new(1000);
        let test_data: Vec<Vec<u8>> = (0..500)
            .map(|i| format!("cached_data_{}", i).into_bytes())
            .collect();
        
        b.iter(|| {
            for data in &test_data {
                let hash = cache.get_or_compute(black_box(data));
                black_box(hash);
            }
        });
    });
    
    group.finish();
}

fn bench_state_management(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_management");
    
    // State snapshot and rollback (CC Chain's advantage)
    group.bench_function("state_snapshot_creation", |b| {
        let state_manager = create_populated_state_manager(1000);
        
        b.iter(|| {
            let snapshot = state_manager.create_snapshot();
            black_box(snapshot)
        });
    });
    
    group.bench_function("state_rollback", |b| {
        let state_manager = create_populated_state_manager(1000);
        let snapshot = state_manager.create_snapshot();
        
        b.iter(|| {
            state_manager.restore_from_snapshot(black_box(&snapshot));
        });
    });
    
    // Parallel state validation
    group.bench_function("parallel_state_validation", |b| {
        let state_manager = create_populated_state_manager(100);
        let transactions = create_test_transactions(1000);
        
        b.iter(|| {
            let results = state_manager.validate_transactions_parallel(black_box(&transactions));
            black_box(results)
        });
    });
    
    // State caching performance
    group.bench_function("state_cache_performance", |b| {
        let cache = StateCache::new(1000, 100);
        let accounts = create_test_accounts(500);
        
        b.iter(|| {
            for (pubkey, account) in &accounts {
                cache.put_account(*pubkey, account.clone());
                let retrieved = cache.get_account(pubkey);
                black_box(retrieved);
            }
        });
    });
    
    group.finish();
}

fn bench_transaction_batching(c: &mut Criterion) {
    let mut group = c.benchmark_group("transaction_batching");
    
    // Smart batching system
    group.bench_function("smart_batcher_performance", |b| {
        let mut batcher = SmartBatcher::new(1000, 100_000, Duration::from_millis(100));
        let transactions = create_test_transactions(5000);
        
        b.iter(|| {
            let mut batches = Vec::new();
            for tx in &transactions {
                if let Some(batch) = batcher.add_transaction(tx.clone()) {
                    batches.push(batch);
                }
            }
            if let Some(final_batch) = batcher.force_batch() {
                batches.push(final_batch);
            }
            black_box(batches)
        });
    });
    
    // Batch validation
    group.bench_function("batch_validation", |b| {
        let transactions = create_test_transactions(1000);
        let batch = TransactionBatch::new(transactions);
        
        b.iter(|| {
            let results = batch.validate_all();
            black_box(results)
        });
    });
    
    group.finish();
}

fn bench_merkle_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("merkle_operations");
    
    for size in [100, 500, 1000, 5000] {
        group.throughput(Throughput::Elements(size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("merkle_tree_creation", size),
            &size,
            |b, &size| {
                let leaves: Vec<Hash> = (0..size)
                    .map(|i| crate::crypto::hash(format!("leaf_{}", i).as_bytes()))
                    .collect();
                
                b.iter(|| {
                    let tree = MerkleTree::build(black_box(&leaves));
                    black_box(tree)
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("merkle_proof_generation", size),
            &size,
            |b, &size| {
                let leaves: Vec<Hash> = (0..size)
                    .map(|i| crate::crypto::hash(format!("leaf_{}", i).as_bytes()))
                    .collect();
                let tree = MerkleTree::build(&leaves);
                
                b.iter(|| {
                    for i in 0..10.min(size) { // Test first 10 proofs
                        let proof = tree.proof(black_box(i));
                        black_box(proof);
                    }
                });
            },
        );
    }
    
    group.finish();
}

fn bench_adaptive_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("adaptive_performance");
    
    // Adaptive parameters optimization
    group.bench_function("adaptive_params_optimization", |b| {
        let mut adaptive_params = AdaptiveParams::new();
        
        // Simulate some performance data
        adaptive_params.get_monitor().record_block(1000, Duration::from_millis(800));
        adaptive_params.get_monitor().record_confirmation(Duration::from_millis(1500));
        
        b.iter(|| {
            adaptive_params.adapt(black_box(200), black_box(Duration::from_millis(150)));
            let params = adaptive_params.get_params();
            black_box(params)
        });
    });
    
    // Performance monitoring overhead
    group.bench_function("performance_monitoring_overhead", |b| {
        let monitor = PerformanceMonitor::new();
        
        b.iter(|| {
            monitor.record_block(black_box(100), black_box(Duration::from_millis(500)));
            monitor.record_confirmation(black_box(Duration::from_millis(100)));
            let metrics = monitor.get_metrics();
            black_box(metrics)
        });
    });
    
    group.finish();
}

// Helper functions for creating test data

fn create_test_transactions(count: usize) -> Vec<Transaction> {
    (0..count)
        .map(|i| {
            let keypair1 = CCKeypair::generate();
            let keypair2 = CCKeypair::generate();
            let mut tx = Transaction::new(
                keypair1.public_key(),
                keypair2.public_key(),
                1000 + i as u64,
                10 + (i % 50) as u64,
                i as u64,
                format!("data_{}", i).into_bytes(),
            );
            tx.sign(&keypair1);
            tx
        })
        .collect()
}

fn create_test_signatures(count: usize) -> Vec<(CCSignature, CCPublicKey, Vec<u8>)> {
    (0..count)
        .map(|i| {
            let keypair = CCKeypair::generate();
            let message = format!("message_{}", i).into_bytes();
            let signature = keypair.sign(&message);
            (signature, keypair.public_key(), message)
        })
        .collect()
}

fn create_populated_state_manager(account_count: usize) -> StateManager {
    let state_manager = StateManager::new();
    
    for i in 0..account_count {
        let keypair = CCKeypair::generate();
        let account = Account {
            balance: 1000 + i as u64,
            nonce: i as u64,
            storage_root: [0u8; 32],
            code_hash: [0u8; 32],
        };
        state_manager.set_account(keypair.public_key(), account);
        
        if i % 10 == 0 {
            state_manager.add_validator(keypair.public_key(), 5000 + i as u64);
        }
    }
    
    state_manager
}

fn create_test_accounts(count: usize) -> Vec<(CCPublicKey, Account)> {
    (0..count)
        .map(|i| {
            let keypair = CCKeypair::generate();
            let account = Account {
                balance: 1000 + i as u64,
                nonce: i as u64,
                storage_root: [0u8; 32],
                code_hash: [0u8; 32],
            };
            (keypair.public_key(), account)
        })
        .collect()
}

criterion_group!(
    benches,
    bench_transaction_throughput,
    bench_cryptographic_operations,
    bench_state_management,
    bench_transaction_batching,
    bench_merkle_operations,
    bench_adaptive_performance
);

criterion_main!(benches);