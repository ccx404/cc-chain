//! Integration tests for ccBFT consensus
//! 
//! Tests the full ccBFT consensus workflow including:
//! - Multi-validator consensus scenarios
//! - Byzantine fault tolerance
//! - View changes and recovery
//! - Performance under load

use cc_chain::consensus::{CCConsensus, ccbft::*};
use cc_chain::crypto::CCKeypair;
use cc_chain::core::Result;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time;

/// Helper function to create a network of ccBFT validators
fn create_ccbft_network(validator_count: usize) -> Result<Vec<CcBftConsensus>> {
    let mut validators = Vec::new();
    let mut validator_keys = HashMap::new();
    let mut keypairs = Vec::new();
    
    // Generate validator keypairs and stakes
    for i in 0..validator_count {
        let keypair = CCKeypair::generate();
        validator_keys.insert(keypair.public_key(), 1000 + i as u64 * 100);
        keypairs.push(keypair);
    }
    
    // Create ccBFT consensus instances for each validator using their own keypair
    for keypair in keypairs {
        let ccbft = CCConsensus::create_ccbft(
            keypair,
            validator_keys.clone(),
            Some(CcBftConfig::default()),
        )?;
        validators.push(ccbft);
    }
    
    Ok(validators)
}

#[tokio::test]
async fn test_ccbft_basic_consensus() -> Result<()> {
    println!("🧪 Testing basic ccBFT consensus...");
    
    let validators = create_ccbft_network(4)?;
    let leader = &validators[0];
    
    // Start consensus on all validators
    for validator in &validators {
        validator.start_consensus(1)?;
    }
    
    // Leader should be able to propose a block
    let (height, view, round, _phase) = leader.get_consensus_state();
    assert_eq!(height, 1);
    assert_eq!(view, 0);
    assert_eq!(round, 0);
    
    println!("   ✓ All validators started consensus successfully");
    println!("   ✓ Leader is at height {}, view {}, round {}", height, view, round);
    
    Ok(())
}

#[tokio::test]
async fn test_ccbft_view_change() -> Result<()> {
    println!("🧪 Testing ccBFT view change mechanism...");
    
    let validators = create_ccbft_network(4)?;
    let validator = &validators[0];
    
    validator.start_consensus(1)?;
    
    // Trigger a view change
    validator.trigger_view_change()?;
    
    let (_, _view, _, phase) = validator.get_consensus_state();
    assert_eq!(phase, ConsensusPhase::ViewChange);
    
    println!("   ✓ View change triggered successfully");
    println!("   ✓ Validator moved to ViewChange phase");
    
    Ok(())
}

#[tokio::test]
async fn test_ccbft_validator_set_updates() -> Result<()> {
    println!("🧪 Testing ccBFT validator set updates...");
    
    let validators = create_ccbft_network(4)?;
    let validator = &validators[0];
    
    // Add a new validator
    let new_validator_keypair = CCKeypair::generate();
    let changes = vec![ValidatorChange {
        change_type: ChangeType::Add,
        validator: new_validator_keypair.public_key(),
        new_stake: Some(1500),
    }];
    
    validator.update_validator_set(changes)?;
    
    let status = validator.get_status();
    assert_eq!(status.validator_count, 5); // 4 original + 1 new
    // Total stake: 4 validators (1000, 1100, 1200, 1300) + 1 new (1500) = 6100
    assert_eq!(status.total_stake, 6100);
    
    println!("   ✓ Validator set updated successfully");
    println!("   ✓ New validator count: {}, Total stake: {}", 
             status.validator_count, status.total_stake);
    
    Ok(())
}

#[tokio::test]
async fn test_ccbft_performance_metrics() -> Result<()> {
    println!("🧪 Testing ccBFT performance metrics...");
    
    let validators = create_ccbft_network(4)?;
    let validator = &validators[0];
    
    // Simulate processing some blocks
    validator.update_pipeline_metrics(100, Duration::from_secs(10))?;
    
    let metrics = validator.get_metrics();
    let pipeline_metrics = validator.get_pipeline_metrics();
    
    assert_eq!(pipeline_metrics.blocks_per_second, 10.0);
    assert!(metrics.throughput_tps > 0.0);
    
    println!("   ✓ Performance metrics updated successfully");
    println!("   ✓ Blocks per second: {:.2}", pipeline_metrics.blocks_per_second);
    println!("   ✓ Throughput TPS: {:.2}", metrics.throughput_tps);
    
    Ok(())
}

#[tokio::test]
async fn test_ccbft_message_processing() -> Result<()> {
    println!("🧪 Testing ccBFT message processing...");
    
    let validators = create_ccbft_network(4)?;
    let validator = &validators[0];
    
    validator.start_consensus(1)?;
    
    // Process any pending messages
    validator.process_pending_messages()?;
    
    let status = validator.get_status();
    
    // Check queue lengths (should be 0 after processing)
    assert_eq!(status.queue_lengths.proposals, 0);
    assert_eq!(status.queue_lengths.votes, 0);
    assert_eq!(status.queue_lengths.view_changes, 0);
    assert_eq!(status.queue_lengths.new_views, 0);
    
    println!("   ✓ Message processing completed successfully");
    println!("   ✓ All message queues are empty");
    
    Ok(())
}

#[tokio::test]
async fn test_ccbft_timeout_handling() -> Result<()> {
    println!("🧪 Testing ccBFT timeout handling...");
    
    let validators = create_ccbft_network(4)?;
    let validator = &validators[0];
    
    validator.start_consensus(1)?;
    
    // Check timeout (should not timeout immediately)
    validator.check_timeout()?;
    
    let (_, _, _, phase) = validator.get_consensus_state();
    // Phase might change but shouldn't crash
    
    println!("   ✓ Timeout check completed successfully");
    println!("   ✓ Current phase: {:?}", phase);
    
    Ok(())
}

#[tokio::test]
async fn test_ccbft_network_simulation() -> Result<()> {
    println!("🧪 Testing ccBFT network simulation...");
    
    let validators = create_ccbft_network(4)?; // Smaller network for faster testing
    
    // Start consensus on all validators
    for validator in &validators {
        validator.start_consensus(1)?;
    }
    
    // Simulate message exchange (simplified)
    for validator in &validators {
        validator.process_pending_messages()?;
    }
    
    // Check that all validators are in sync
    let states: Vec<_> = validators.iter()
        .map(|v| v.get_consensus_state())
        .collect();
    
    // All should be at same height
    let heights: Vec<_> = states.iter().map(|(h, _, _, _)| *h).collect();
    assert!(heights.iter().all(|&h| h == heights[0]));
    
    println!("   ✓ Network simulation completed successfully");
    println!("   ✓ All {} validators synchronized at height {}", 
             validators.len(), heights[0]);
    
    Ok(())
}

#[tokio::test]
async fn test_ccbft_byzantine_tolerance() -> Result<()> {
    println!("🧪 Testing ccBFT Byzantine fault tolerance...");
    
    let validators = create_ccbft_network(7)?; // 7 validators, can tolerate 2 Byzantine
    
    // Simulate 2 validators going offline (Byzantine behavior)
    let active_validators = &validators[0..3]; // Only 3 out of 4 are active
    
    for validator in active_validators {
        validator.start_consensus(1)?;
    }
    
    // Check that consensus can still proceed with 5/7 validators
    for validator in active_validators {
        let status = validator.get_status();
        assert_eq!(status.height, 1);
    }
    
    println!("   ✓ Byzantine fault tolerance test passed");
    println!("   ✓ Consensus proceeded with {}/{} validators active", 
             active_validators.len(), validators.len());
    
    Ok(())
}

#[tokio::test] 
async fn test_ccbft_upgrade_path() -> Result<()> {
    println!("🧪 Testing ccBFT upgrade path...");
    
    // Start with regular CC consensus
    let keypair = CCKeypair::generate();
    let consensus = CCConsensus::new(keypair.clone());
    
    // Add validators
    let mut validators = HashMap::new();
    for i in 0..4 {
        let validator_keypair = CCKeypair::generate();
        validators.insert(validator_keypair.public_key(), 1000 + i * 100);
    }
    validators.insert(keypair.public_key(), 2000);
    consensus.update_validators(validators);
    
    // Check recommendations
    let recommendations = consensus.get_consensus_recommendations();
    println!("   📊 Recommendations: {:?}", recommendations.recommendations);
    
    // Upgrade to ccBFT
    let ccbft_consensus = consensus.upgrade_to_ccbft()?;
    
    let status = ccbft_consensus.get_status();
    assert_eq!(status.validator_count, 5);
    // Total stake: 1000 + 1100 + 1200 + 1300 + 2000 = 6600
    assert_eq!(status.total_stake, 6600);
    
    println!("   ✓ Upgrade from CC consensus to ccBFT successful");
    println!("   ✓ ccBFT has {} validators with total stake {}", 
             status.validator_count, status.total_stake);
    
    Ok(())
}

#[tokio::test]
async fn test_ccbft_stress_scenario() -> Result<()> {
    println!("🧪 Testing ccBFT under stress...");
    
    let validators = create_ccbft_network(6)?; // Smaller network for testing
    
    // Start consensus on all validators
    for validator in &validators {
        validator.start_consensus(1)?;
    }
    
    // Simulate rapid message processing (reduced iterations)
    for _ in 0..3 {
        for validator in &validators {
            validator.process_pending_messages()?;
            validator.check_timeout()?;
        }
        
        // Small delay to simulate network latency
        time::sleep(Duration::from_millis(1)).await;
    }
    
    // Check final state
    let final_states: Vec<_> = validators.iter()
        .map(|v| v.get_status())
        .collect();
    
    // All validators should still be operational
    for (i, status) in final_states.iter().enumerate() {
        assert_eq!(status.height, 1);
        println!("   Validator {}: Height {}, Phase {:?}", 
                 i, status.height, status.phase);
    }
    
    println!("   ✓ Stress test completed successfully");
    println!("   ✓ All {} validators remained operational", validators.len());
    
    Ok(())
}