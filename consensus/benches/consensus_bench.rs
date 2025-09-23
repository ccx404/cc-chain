use criterion::{black_box, criterion_group, criterion_main, Criterion};
use consensus::{CcBftConsensus, CcBftConfig};
use consensus::safety::{SafetySystem, SafetyConfig};
use cc_core::crypto::CCKeypair;
use std::sync::Arc;

fn benchmark_consensus(c: &mut Criterion) {
    c.bench_function("consensus_creation", |b| {
        b.iter(|| {
            let keypair = CCKeypair::generate();
            let validator_id = 1;
            let stake = 1000;
            let config = CcBftConfig::default();
            let safety_system = Arc::new(SafetySystem::new(SafetyConfig::default()));
            let consensus = CcBftConsensus::new(keypair, validator_id, stake, config, safety_system);
            black_box(consensus);
        })
    });

    c.bench_function("consensus_metrics", |b| {
        let keypair = CCKeypair::generate();
        let validator_id = 1;
        let stake = 1000;
        let config = CcBftConfig::default();
        let safety_system = Arc::new(SafetySystem::new(SafetyConfig::default()));
        let consensus = CcBftConsensus::new(keypair, validator_id, stake, config, safety_system);
        b.iter(|| {
            let metrics = consensus.get_metrics();
            black_box(metrics);
        })
    });
}

fn benchmark_config(c: &mut Criterion) {
    c.bench_function("config_creation", |b| {
        b.iter(|| {
            let config = CcBftConfig::default();
            black_box(config);
        })
    });
}

criterion_group!(benches, benchmark_consensus, benchmark_config);
criterion_main!(benches);