//! Bridge validator functionality

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Bridge validator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeValidator {
    /// Unique validator ID
    pub id: String,
    /// Validator public key for signature verification
    pub public_key: String,
    /// Validator network address
    pub network_address: String,
    /// Validator voting power/stake
    pub voting_power: u64,
    /// Validator status
    pub status: ValidatorStatus,
    /// Last seen timestamp
    pub last_seen: u64,
    /// Validator reputation score
    pub reputation: f64,
    /// Number of successful validations
    pub successful_validations: u64,
    /// Number of failed validations
    pub failed_validations: u64,
    /// Validator metadata
    pub metadata: HashMap<String, String>,
}

/// Validator status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValidatorStatus {
    /// Validator is active and participating
    Active,
    /// Validator is temporarily inactive
    Inactive,
    /// Validator is jailed due to misbehavior
    Jailed,
    /// Validator has been permanently slashed
    Slashed,
    /// Validator is in the process of unbonding
    Unbonding,
}

impl BridgeValidator {
    /// Create a new bridge validator
    pub fn new(
        id: String,
        public_key: String,
        network_address: String,
        voting_power: u64,
    ) -> Self {
        Self {
            id,
            public_key,
            network_address,
            voting_power,
            status: ValidatorStatus::Active,
            last_seen: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            reputation: 1.0,
            successful_validations: 0,
            failed_validations: 0,
            metadata: HashMap::new(),
        }
    }
    
    /// Update validator's last seen timestamp
    pub fn update_last_seen(&mut self) {
        self.last_seen = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
    
    /// Record a successful validation
    pub fn record_success(&mut self) {
        self.successful_validations += 1;
        self.update_reputation();
        self.update_last_seen();
    }
    
    /// Record a failed validation
    pub fn record_failure(&mut self) {
        self.failed_validations += 1;
        self.update_reputation();
        self.update_last_seen();
    }
    
    /// Update reputation score based on performance
    fn update_reputation(&mut self) {
        let total = self.successful_validations + self.failed_validations;
        if total > 0 {
            self.reputation = self.successful_validations as f64 / total as f64;
        }
    }
    
    /// Check if validator is active and reliable
    pub fn is_reliable(&self) -> bool {
        matches!(self.status, ValidatorStatus::Active) && self.reputation > 0.9
    }
    
    /// Get validator's success rate
    pub fn success_rate(&self) -> f64 {
        let total = self.successful_validations + self.failed_validations;
        if total > 0 {
            self.successful_validations as f64 / total as f64
        } else {
            1.0
        }
    }
    
    /// Check if validator has been inactive for too long
    pub fn is_stale(&self, max_age_seconds: u64) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        now - self.last_seen > max_age_seconds
    }
    
    /// Jail the validator
    pub fn jail(&mut self, reason: String) {
        self.status = ValidatorStatus::Jailed;
        self.metadata.insert("jail_reason".to_string(), reason);
        self.metadata.insert("jailed_at".to_string(), 
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string()
        );
    }
    
    /// Unjail the validator
    pub fn unjail(&mut self) {
        if matches!(self.status, ValidatorStatus::Jailed) {
            self.status = ValidatorStatus::Active;
            self.metadata.remove("jail_reason");
            self.metadata.remove("jailed_at");
        }
    }
    
    /// Slash the validator
    pub fn slash(&mut self, reason: String) {
        self.status = ValidatorStatus::Slashed;
        self.voting_power = 0;
        self.metadata.insert("slash_reason".to_string(), reason);
        self.metadata.insert("slashed_at".to_string(), 
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string()
        );
    }
    
    /// Set validator metadata
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
    
    /// Get validator metadata
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

/// Validator set management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorSet {
    /// All validators in the set
    pub validators: Vec<BridgeValidator>,
    /// Current epoch
    pub epoch: u64,
    /// Minimum required voting power for consensus
    pub min_voting_power: u64,
    /// Threshold for achieving consensus (e.g., 2/3)
    pub consensus_threshold: f64,
}

impl ValidatorSet {
    /// Create a new validator set
    pub fn new(min_voting_power: u64, consensus_threshold: f64) -> Self {
        Self {
            validators: Vec::new(),
            epoch: 0,
            min_voting_power,
            consensus_threshold,
        }
    }
    
    /// Add a validator to the set
    pub fn add_validator(&mut self, validator: BridgeValidator) -> Result<(), String> {
        // Check if validator already exists
        if self.validators.iter().any(|v| v.id == validator.id) {
            return Err("Validator already exists".to_string());
        }
        
        self.validators.push(validator);
        Ok(())
    }
    
    /// Remove a validator from the set
    pub fn remove_validator(&mut self, validator_id: &str) -> Result<(), String> {
        let index = self.validators
            .iter()
            .position(|v| v.id == validator_id)
            .ok_or("Validator not found")?;
        
        self.validators.remove(index);
        Ok(())
    }
    
    /// Get active validators
    pub fn get_active_validators(&self) -> Vec<&BridgeValidator> {
        self.validators
            .iter()
            .filter(|v| matches!(v.status, ValidatorStatus::Active))
            .collect()
    }
    
    /// Get total voting power of active validators
    pub fn total_voting_power(&self) -> u64 {
        self.get_active_validators()
            .iter()
            .map(|v| v.voting_power)
            .sum()
    }
    
    /// Check if we have enough voting power for consensus
    pub fn has_sufficient_voting_power(&self) -> bool {
        self.total_voting_power() >= self.min_voting_power
    }
    
    /// Calculate required voting power for consensus
    pub fn required_voting_power_for_consensus(&self) -> u64 {
        let total = self.total_voting_power();
        (total as f64 * self.consensus_threshold).ceil() as u64
    }
    
    /// Check if a set of validators meets consensus threshold
    pub fn meets_consensus_threshold(&self, validator_ids: &[String]) -> bool {
        let voting_power: u64 = self.validators
            .iter()
            .filter(|v| validator_ids.contains(&v.id) && matches!(v.status, ValidatorStatus::Active))
            .map(|v| v.voting_power)
            .sum();
        
        voting_power >= self.required_voting_power_for_consensus()
    }
    
    /// Update epoch
    pub fn next_epoch(&mut self) {
        self.epoch += 1;
    }
    
    /// Clean up stale validators
    pub fn cleanup_stale_validators(&mut self, max_age_seconds: u64) {
        for validator in &mut self.validators {
            if validator.is_stale(max_age_seconds) && matches!(validator.status, ValidatorStatus::Active) {
                validator.status = ValidatorStatus::Inactive;
            }
        }
    }
    
    /// Get validator by ID
    pub fn get_validator(&self, validator_id: &str) -> Option<&BridgeValidator> {
        self.validators.iter().find(|v| v.id == validator_id)
    }
    
    /// Get mutable validator by ID
    pub fn get_validator_mut(&mut self, validator_id: &str) -> Option<&mut BridgeValidator> {
        self.validators.iter_mut().find(|v| v.id == validator_id)
    }
}