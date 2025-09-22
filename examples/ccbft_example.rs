//! ccBFT Consensus Engine Example
//! 
//! This example demonstrates how to use the ccBFT consensus engine
//! for Byzantine Fault Tolerant consensus in CC Chain.

use cc_chain::consensus::{CCConsensus, ccbft::{CcBftConsensus, CcBftConfig, ValidatorInfo}};
use cc_chain::crypto::CCKeypair;
use cc_chain::core::Result;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("🚀 CC Chain ccBFT Consensus Example");
    println!("=====================================");
    
    // Example 1: Create a basic CC consensus and upgrade to ccBFT
    println!("\n1. Creating basic consensus and upgrading to ccBFT...");
    
    let keypair = CCKeypair::generate();
    let mut consensus = CCConsensus::new(keypair.clone());
    
    // Add some validators
    let mut validators = HashMap::new();
    for i in 0..4 {
        let validator_keypair = CCKeypair::generate();
        validators.insert(validator_keypair.public_key(), 1000 + i * 100);
    }
    
    // Add ourselves as a validator
    validators.insert(keypair.public_key(), 2000);
    consensus.update_validators(validators.clone());
    
    println!("   ✓ Added {} validators with total stake: {}", 
             validators.len(), 
             validators.values().sum::<u64>());
    
    // Upgrade to ccBFT
    let ccbft_consensus = consensus.upgrade_to_ccbft()?;
    println!("   ✓ Successfully upgraded to ccBFT consensus");
    
    // Example 2: Create ccBFT consensus directly
    println!("\n2. Creating ccBFT consensus directly...");
    
    let config = CcBftConfig {
        proposal_timeout: Duration::from_millis(1000),
        pre_vote_timeout: Duration::from_millis(500),
        pre_commit_timeout: Duration::from_millis(500),
        view_change_timeout: Duration::from_secs(10),
        max_parallel_blocks: 10,
        fast_path_enabled: true,
        adaptive_timeouts: true,
        pipelining_enabled: true,
        aggregate_signatures: true,
    };
    
    let ccbft_direct = CCConsensus::create_ccbft(
        CCKeypair::generate(),
        validators.clone(),
        Some(config),
    )?;
    
    println!("   ✓ Created ccBFT consensus directly with custom config");
    
    // Example 3: Demonstrate consensus operations
    println!("\n3. Demonstrating consensus operations...");
    
    // Start consensus for height 1
    ccbft_consensus.start_consensus(1)?;
    let (height, view, round, phase) = ccbft_consensus.get_consensus_state();
    println!("   ✓ Started consensus at height: {}, view: {}, round: {}, phase: {:?}", 
             height, view, round, phase);
    
    // Process pending messages (would normally happen in a loop)
    ccbft_consensus.process_pending_messages()?;
    println!("   ✓ Processed pending messages");
    
    // Check for timeouts
    ccbft_consensus.check_timeout()?;
    println!("   ✓ Checked for timeouts");
    
    // Example 4: Performance monitoring
    println!("\n4. Performance monitoring...");
    
    let metrics = ccbft_consensus.get_metrics();
    println!("   📊 Consensus Metrics:");
    println!("      - Blocks processed: {}", metrics.blocks_processed);
    println!("      - Average finality time: {:?}", metrics.average_finality_time);
    println!("      - Throughput TPS: {:.2}", metrics.throughput_tps);
    println!("      - View changes: {}", metrics.view_changes);
    println!("      - Pipeline efficiency: {:.2}%", metrics.pipeline_efficiency * 100.0);
    
    let status = ccbft_consensus.get_status();
    println!("   📈 Current Status:");
    println!("      - Height: {}, View: {}, Round: {}", status.height, status.view, status.round);
    println!("      - Phase: {:?}", status.phase);
    println!("      - Validators: {}, Total stake: {}", status.validator_count, status.total_stake);
    println!("      - Queue lengths: Proposals({}), Votes({}), ViewChanges({}), NewViews({})",
             status.queue_lengths.proposals,
             status.queue_lengths.votes, 
             status.queue_lengths.view_changes,
             status.queue_lengths.new_views);
    
    // Example 5: Validator set management
    println!("\n5. Validator set management...");
    
    use cc_chain::consensus::ccbft::{ValidatorChange, ChangeType};
    
    let new_validator = CCKeypair::generate();
    let validator_changes = vec![
        ValidatorChange {
            change_type: ChangeType::Add,
            validator: new_validator.public_key(),
            new_stake: Some(1500),
        }
    ];
    
    ccbft_consensus.update_validator_set(validator_changes)?;
    println!("   ✓ Added new validator to the set");
    
    // Example 6: Pipeline metrics
    println!("\n6. Pipeline performance...");
    
    ccbft_consensus.update_pipeline_metrics(50, Duration::from_secs(10))?;
    let pipeline_metrics = ccbft_consensus.get_pipeline_metrics();
    println!("   🔄 Pipeline Metrics:");
    println!("      - Blocks per second: {:.2}", pipeline_metrics.blocks_per_second);
    println!("      - Transactions per second: {:.2}", pipeline_metrics.transactions_per_second);
    println!("      - Average block time: {:?}", pipeline_metrics.average_block_time);
    println!("      - Pipeline utilization: {:.2}%", pipeline_metrics.pipeline_utilization * 100.0);
    
    // Example 7: Consensus recommendations
    println!("\n7. Getting consensus recommendations...");
    
    let recommendations = consensus.get_consensus_recommendations();
    println!("   💡 Recommendations:");
    println!("      - Current engine: {}", recommendations.current_engine);
    println!("      - Upgrade recommended: {}", recommendations.upgrade_recommended);
    println!("      - Safety incidents: {}", recommendations.safety_incidents);
    for (i, rec) in recommendations.recommendations.iter().enumerate() {
        println!("      {}. {}", i + 1, rec);
    }
    
    println!("\n✅ ccBFT consensus example completed successfully!");
    println!("\nKey Features Demonstrated:");
    println!("  • Byzantine Fault Tolerant consensus with enhanced features");
    println!("  • Stake-based voting with configurable thresholds");
    println!("  • View change and leader rotation protocols");
    println!("  • Performance monitoring and metrics collection");
    println!("  • Pipeline processing for improved throughput");
    println!("  • Dynamic validator set management");
    println!("  • Safety system integration");
    
    Ok(())
}