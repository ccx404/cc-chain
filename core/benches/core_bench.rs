use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cc_core::{
    block::Block,
    transaction::Transaction,
    crypto::CCKeypair,
};

fn benchmark_transaction_processing(c: &mut Criterion) {
    c.bench_function("transaction_creation", |b| {
        let keypair = CCKeypair::generate();
        let from = keypair.public_key();
        let to = CCKeypair::generate().public_key();
        
        b.iter(|| {
            let tx = Transaction::new(from, to, 1000, 10, 0, vec![]);
            black_box(tx);
        })
    });

    c.bench_function("transaction_validation", |b| {
        let keypair = CCKeypair::generate();
        let from = keypair.public_key();
        let to = CCKeypair::generate().public_key();
        let tx = Transaction::new(from, to, 1000, 10, 0, vec![]);
        
        b.iter(|| {
            let result = tx.validate();
            black_box(result);
        })
    });
}

fn benchmark_block_processing(c: &mut Criterion) {
    c.bench_function("block_creation", |b| {
        let keypair = CCKeypair::generate();
        let proposer = keypair.public_key();
        let parent_hash = [0u8; 32];
        
        b.iter(|| {
            let block = Block::genesis(proposer, parent_hash);
            black_box(block);
        })
    });

    c.bench_function("block_validation", |b| {
        let keypair = CCKeypair::generate();
        let proposer = keypair.public_key();
        let parent_hash = [0u8; 32];
        let block = Block::genesis(proposer, parent_hash);
        
        b.iter(|| {
            let result = block.validate();
            black_box(result);
        })
    });
}

fn benchmark_crypto(c: &mut Criterion) {
    c.bench_function("keypair_generation", |b| {
        b.iter(|| {
            let keypair = CCKeypair::generate();
            black_box(keypair);
        })
    });
}

criterion_group!(
    benches, 
    benchmark_transaction_processing, 
    benchmark_block_processing,
    benchmark_crypto
);
criterion_main!(benches);