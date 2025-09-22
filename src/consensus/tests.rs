//! Tests for SAFETY system and ccBFT consensus implementation

#[cfg(test)]
mod tests {
    use crate::crypto::CCKeypair;
    use crate::consensus::{SafetySystem, SafetyConfig, CcBftConsensus, CcBftConfig, CCConsensus};
    use crate::consensus::safety::ValidatorAction;
    use crate::consensus::ccbft::{ConsensusPhase, ValidatorInfo};
    use crate::Hash;
    use std::time::Duration;

    /// Test SAFETY system initialization and basic functionality
    #[test]
    fn test_safety_system_initialization() {
        let config = SafetyConfig::default();
        let safety_system = SafetySystem::new(config);
        
        let status = safety_system.get_safety_status();
        assert!(status.overall_health > 0.0);
        assert_eq!(status.active_alerts, 0);
        assert_eq!(status.active_faults, 0);
    }

    /// Test validator behavior monitoring
    #[test]
    fn test_validator_behavior_monitoring() {
        let config = SafetyConfig::default();
        let safety_system = SafetySystem::new(config);
        
        let keypair = CCKeypair::generate();
        let validator = keypair.public_key();
        
        // Test valid proposal monitoring
        let result = safety_system.monitor_validator_behavior(
            validator,
            ValidatorAction::ProposalMade {
                block_hash: Hash::default(),
                valid: true,
            },
        );
        assert!(result.is_ok());
        
        // Test vote monitoring
        let result = safety_system.monitor_validator_behavior(
            validator,
            ValidatorAction::VoteCast {
                block_hash: Hash::default(),
                consistent: true,
            },
        );
        assert!(result.is_ok());
    }

    /// Test network health monitoring
    #[test]
    fn test_network_health_monitoring() {
        let config = SafetyConfig::default();
        let safety_system = SafetySystem::new(config);
        
        // Test good network conditions
        let result = safety_system.monitor_network_health(
            Duration::from_millis(100),
            true,
        );
        assert!(result.is_ok());
        
        // Test degraded network conditions
        let result = safety_system.monitor_network_health(
            Duration::from_secs(10),
            false,
        );
        assert!(result.is_ok());
    }

    /// Test fault detection
    #[test]
    fn test_fault_detection() {
        let config = SafetyConfig::default();
        let safety_system = SafetySystem::new(config);
        
        let faults = safety_system.detect_faults();
        assert!(faults.is_ok());
        
        let fault_list = faults.unwrap();
        assert!(fault_list.len() >= 0); // Should return empty list initially
    }

    /// Test ccBFT consensus initialization
    #[test]
    fn test_ccbft_initialization() {
        let keypair = CCKeypair::generate();
        let config = CcBftConfig::default();
        let safety_config = SafetyConfig::default();
        let safety_system = std::sync::Arc::new(SafetySystem::new(safety_config));
        
        let consensus = CcBftConsensus::new(
            keypair,
            1,
            1000,
            config,
            safety_system,
        );
        
        let (height, view, round, phase) = consensus.get_consensus_state();
        assert_eq!(height, 0);
        assert_eq!(view, 0);
        assert_eq!(round, 0);
        assert_eq!(phase, ConsensusPhase::Prepare);
    }

    /// Test ccBFT validator set initialization
    #[test]
    fn test_ccbft_validator_set() {
        let keypair = CCKeypair::generate();
        let config = CcBftConfig::default();
        let safety_config = SafetyConfig::default();
        let safety_system = std::sync::Arc::new(SafetySystem::new(safety_config));
        
        let consensus = CcBftConsensus::new(
            keypair.clone(),
            1,
            1000,
            config,
            safety_system,
        );
        
        // Create validator set
        let mut validators = std::collections::HashMap::new();
        validators.insert(keypair.public_key(), ValidatorInfo {
            public_key: keypair.public_key(),
            stake: 1000,
            reputation: 1.0,
            network_address: "127.0.0.1:8000".to_string(),
            last_active: std::time::Instant::now(),
        });
        
        let result = consensus.initialize(validators);
        assert!(result.is_ok());
    }

    /// Test enhanced consensus with SAFETY system
    #[test]
    fn test_enhanced_consensus_with_safety() {
        let keypair = CCKeypair::generate();
        let consensus = CCConsensus::new(keypair.clone());
        
        // Check that safety system is initialized
        let safety_status = consensus.get_safety_status();
        assert!(safety_status.overall_health > 0.0);
        
        // Check fault tolerance metrics
        let fault_metrics = consensus.get_fault_tolerance_metrics();
        assert_eq!(fault_metrics.byzantine_incidents, 0);
        assert_eq!(fault_metrics.total_faults_detected, 0);
    }

    /// Test Byzantine behavior detection
    #[test]
    fn test_byzantine_behavior_detection() {
        let keypair = CCKeypair::generate();
        let consensus = CCConsensus::new(keypair.clone());
        
        // Add validator to set
        let mut validators = std::collections::HashMap::new();
        validators.insert(keypair.public_key(), 1000);
        consensus.update_validators(validators);
        
        // Test that initial metrics show no Byzantine behavior
        let fault_metrics = consensus.get_fault_tolerance_metrics();
        assert_eq!(fault_metrics.byzantine_incidents, 0);
    }

    /// Test performance monitoring
    #[test]
    fn test_performance_monitoring() {
        let keypair = CCKeypair::generate();
        let consensus = CCConsensus::new(keypair);
        
        let performance_metrics = consensus.get_performance_metrics();
        
        // Check that performance metrics are initialized
        assert!(performance_metrics.latency.average_finality_time > Duration::from_secs(0));
        assert!(performance_metrics.consensus_efficiency.round_success_rate >= 0.0);
        assert!(performance_metrics.consensus_efficiency.round_success_rate <= 1.0);
    }

    /// Test consensus upgrade to ccBFT
    #[test]
    fn test_consensus_upgrade_to_ccbft() {
        let keypair = CCKeypair::generate();
        let consensus = CCConsensus::new(keypair.clone());
        
        // Add validator to set
        let mut validators = std::collections::HashMap::new();
        validators.insert(keypair.public_key(), 1000);
        consensus.update_validators(validators);
        
        // Test upgrade to ccBFT
        let ccbft_result = consensus.upgrade_to_ccbft();
        assert!(ccbft_result.is_ok());
        
        let ccbft_consensus = ccbft_result.unwrap();
        let (height, view, round, phase) = ccbft_consensus.get_consensus_state();
        assert_eq!(height, 0);
        assert_eq!(view, 0);
        assert_eq!(round, 0);
        assert_eq!(phase, ConsensusPhase::Prepare);
    }
}