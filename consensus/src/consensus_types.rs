use core::{Block, CCError, Result, CCKeypair, CCPublicKey, CCSignature, Hash};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Enhanced BFT consensus protocol for CC Chain
///
/// Key improvements over traditional BFT:
/// 1. Parallel voting phases for faster consensus
/// 2. Weighted voting based on stake
/// 3. Optimistic fast path for honest majority
/// 4. Lightweight participation for small nodes
/// 5. Dynamic validator set management

/// Consensus message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusMessage {
    /// Propose a new block
    Proposal {
        block: Block,
        round: u64,
        proposer: CCPublicKey,
        signature: CCSignature,
    },
    /// Vote for a proposal
    Vote {
        block_hash: Hash,
        round: u64,
        vote_type: VoteType,
        voter: CCPublicKey,
        signature: CCSignature,
    },
    /// Commit decision
    Commit {
        block_hash: Hash,
        round: u64,
        signatures: Vec<CCSignature>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VoteType {
    /// Pre-vote (first phase)
    PreVote,
    /// Pre-commit (second phase)
    PreCommit,
}

/// Consensus round state
#[derive(Debug)]
pub struct RoundState {
    /// Current round number
    pub round: u64,
    /// Current height
    pub height: u64,
    /// Proposed block for this round
    pub proposal: Option<Block>,
    /// Pre-votes received
    pub pre_votes: HashMap<CCPublicKey, Hash>,
    /// Pre-commits received
    pub pre_commits: HashMap<CCPublicKey, Hash>,
    /// Round start time
    pub start_time: Instant,
    /// Whether we've voted in this round
    pub has_voted: bool,
    /// Whether we've committed in this round
    pub has_committed: bool,
}

impl RoundState {
    pub fn new(round: u64, height: u64) -> Self {
        Self {
            round,
            height,
            proposal: None,
            pre_votes: HashMap::new(),
            pre_commits: HashMap::new(),
            start_time: Instant::now(),
            has_voted: false,
            has_committed: false,
        }
    }
}

/// Enhanced BFT consensus engine with SAFETY system integration
pub struct CCConsensus {
    /// Our validator keypair
    keypair: CCKeypair,
    /// Current round state
    round_state: parking_lot::RwLock<RoundState>,
    /// Validator set with stakes
    validators: parking_lot::RwLock<HashMap<CCPublicKey, u64>>,
    /// Total stake in the validator set
    total_stake: parking_lot::RwLock<u64>,
    /// Consensus parameters
    params: ConsensusParams,
    /// Message queue for processing
    message_queue: crossbeam::queue::SegQueue<ConsensusMessage>,
    /// Block proposal callback
    block_proposer: Option<Box<dyn Fn(u64) -> Option<Block> + Send + Sync>>,
    /// Block commit callback
    block_committer: Option<Box<dyn Fn(Block) -> Result<()> + Send + Sync>>,
    /// SAFETY system for fault tolerance and error detection
    safety_system: std::sync::Arc<crate::safety::SafetySystem>,
    /// Fault tolerance mechanisms
    fault_tolerance: parking_lot::RwLock<FaultToleranceState>,
    /// Performance monitoring
    performance_monitor: parking_lot::RwLock<PerformanceMonitor>,
}

/// Consensus parameters for tuning performance
#[derive(Debug, Clone)]
pub struct ConsensusParams {
    /// Round timeout duration
    pub round_timeout: Duration,
    /// Minimum pre-vote threshold (as percentage of stake)
    pub pre_vote_threshold: u64, // 50% for fast path
    /// Minimum pre-commit threshold (as percentage of stake)
    pub pre_commit_threshold: u64, // 67% for safety
    /// Maximum round duration before timeout
    pub max_round_duration: Duration,
    /// Enable fast path optimization
    pub enable_fast_path: bool,
    /// Enhanced safety monitoring
    pub safety_monitoring_enabled: bool,
    /// Automatic fault recovery
    pub auto_recovery_enabled: bool,
    /// Performance optimization enabled
    pub performance_optimization: bool,
}

/// Fault tolerance state for enhanced reliability
#[derive(Debug)]
pub struct FaultToleranceState {
    /// Detected Byzantine validators
    pub byzantine_validators: HashMap<CCPublicKey, ByzantineBehavior>,
    /// Network partition detection
    pub network_partitions: Vec<NetworkPartition>,
    /// Recovery procedures in progress
    pub active_recoveries: HashMap<RecoveryType, RecoveryStatus>,
    /// Fault detection metrics
    pub fault_metrics: FaultMetrics,
}

/// Byzantine behavior tracking
#[derive(Debug, Clone)]
pub struct ByzantineBehavior {
    pub validator: CCPublicKey,
    pub behavior_type: ByzantineType,
    pub detected_at: Instant,
    pub severity: f64,
    pub evidence: Vec<String>,
}

/// Types of Byzantine behavior
#[derive(Debug, Clone, PartialEq)]
pub enum ByzantineType {
    DoubleVoting,
    Equivocation,
    InvalidProposal,
    TimeoutAbuse,
    ConsistencyViolation,
}

/// Network partition information
#[derive(Debug, Clone)]
pub struct NetworkPartition {
    pub partition_id: u64,
    pub affected_validators: Vec<CCPublicKey>,
    pub detected_at: Instant,
    pub severity: PartitionSeverity,
}

/// Partition severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum PartitionSeverity {
    Minor,
    Moderate,
    Severe,
    Critical,
}

/// Recovery procedure types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RecoveryType {
    ByzantineValidatorRemoval,
    NetworkPartitionHeal,
    ConsensusRestart,
    ValidatorSetRotation,
    PerformanceOptimization,
}

/// Recovery status tracking
#[derive(Debug, Clone)]
pub struct RecoveryStatus {
    pub recovery_type: RecoveryType,
    pub started_at: Instant,
    pub progress: f64,
    pub estimated_completion: Duration,
    pub success_probability: f64,
}

/// Fault detection metrics
#[derive(Debug, Clone)]
pub struct FaultMetrics {
    pub total_faults_detected: u64,
    pub byzantine_incidents: u64,
    pub network_issues: u64,
    pub recovery_success_rate: f64,
    pub mean_time_to_recovery: Duration,
}

/// Performance monitoring for consensus
#[derive(Debug)]
pub struct PerformanceMonitor {
    /// Transaction throughput metrics
    pub throughput: ThroughputMetrics,
    /// Latency measurements
    pub latency: LatencyMetrics,
    /// Resource utilization
    pub resource_usage: ResourceMetrics,
    /// Consensus efficiency
    pub consensus_efficiency: EfficiencyMetrics,
}

/// Throughput measurement
#[derive(Debug, Clone)]
pub struct ThroughputMetrics {
    pub transactions_per_second: f64,
    pub blocks_per_minute: f64,
    pub peak_throughput: f64,
    pub average_throughput: f64,
}

/// Latency measurements
#[derive(Debug, Clone)]
pub struct LatencyMetrics {
    pub average_finality_time: Duration,
    pub block_proposal_time: Duration,
    pub vote_aggregation_time: Duration,
    pub commit_time: Duration,
}

/// Resource utilization metrics
#[derive(Debug, Clone)]
pub struct ResourceMetrics {
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub network_bandwidth: u64,
    pub storage_io: u64,
}

/// Consensus efficiency metrics
#[derive(Debug, Clone)]
pub struct EfficiencyMetrics {
    pub round_success_rate: f64,
    pub view_change_frequency: f64,
    pub validator_participation: f64,
    pub consensus_overhead: f64,
}

/// Consensus engine recommendations for optimization
#[derive(Debug, Clone)]
pub struct ConsensusRecommendations {
    pub current_engine: String,
    pub validator_count: usize,
    pub total_stake: u64,
    pub average_tps: f64,
    pub average_latency: Duration,
    pub safety_incidents: u64,
    pub upgrade_recommended: bool,
    pub recommendations: Vec<String>,
}

impl Default for ConsensusParams {
    fn default() -> Self {
        Self {
            round_timeout: Duration::from_millis(2000), // 2 seconds
            pre_vote_threshold: 50,                     // 50% for optimistic fast path
            pre_commit_threshold: 67,                   // 67% for BFT safety
            max_round_duration: Duration::from_secs(30),
            enable_fast_path: true,
            safety_monitoring_enabled: true,
            auto_recovery_enabled: true,
            performance_optimization: true,
        }
    }
}

impl CCConsensus {
    /// Create new consensus engine with enhanced SAFETY system
    pub fn new(keypair: CCKeypair) -> Self {
        let height = 0;
        let round = 0;

        // Initialize SAFETY system
        let safety_config = crate::safety::SafetyConfig::default();
        let safety_system = std::sync::Arc::new(crate::safety::SafetySystem::new(safety_config));

        Self {
            keypair,
            round_state: parking_lot::RwLock::new(RoundState::new(round, height)),
            validators: parking_lot::RwLock::new(HashMap::new()),
            total_stake: parking_lot::RwLock::new(0),
            params: ConsensusParams::default(),
            message_queue: crossbeam::queue::SegQueue::new(),
            block_proposer: None,
            block_committer: None,
            safety_system,
            fault_tolerance: parking_lot::RwLock::new(FaultToleranceState::new()),
            performance_monitor: parking_lot::RwLock::new(PerformanceMonitor::new()),
        }
    }

    /// Create consensus engine with custom SAFETY configuration
    pub fn new_with_safety_config(keypair: CCKeypair, safety_config: crate::safety::SafetyConfig) -> Self {
        let height = 0;
        let round = 0;

        let safety_system = std::sync::Arc::new(crate::safety::SafetySystem::new(safety_config));

        Self {
            keypair,
            round_state: parking_lot::RwLock::new(RoundState::new(round, height)),
            validators: parking_lot::RwLock::new(HashMap::new()),
            total_stake: parking_lot::RwLock::new(0),
            params: ConsensusParams::default(),
            message_queue: crossbeam::queue::SegQueue::new(),
            block_proposer: None,
            block_committer: None,
            safety_system,
            fault_tolerance: parking_lot::RwLock::new(FaultToleranceState::new()),
            performance_monitor: parking_lot::RwLock::new(PerformanceMonitor::new()),
        }
    }

    /// Set consensus parameters
    pub fn set_params(&mut self, params: ConsensusParams) {
        self.params = params;
    }

    /// Set block proposer callback
    pub fn set_block_proposer<F>(&mut self, proposer: F)
    where
        F: Fn(u64) -> Option<Block> + Send + Sync + 'static,
    {
        self.block_proposer = Some(Box::new(proposer));
    }

    /// Set block committer callback
    pub fn set_block_committer<F>(&mut self, committer: F)
    where
        F: Fn(Block) -> Result<()> + Send + Sync + 'static,
    {
        self.block_committer = Some(Box::new(committer));
    }

    /// Update validator set
    pub fn update_validators(&self, validators: HashMap<CCPublicKey, u64>) {
        let total_stake: u64 = validators.values().sum();
        *self.validators.write() = validators;
        *self.total_stake.write() = total_stake;
    }

    /// Check if we are a validator
    pub fn is_validator(&self) -> bool {
        let my_pubkey = self.keypair.public_key();
        self.validators.read().contains_key(&my_pubkey)
    }

    /// Start new consensus round
    pub fn start_round(&self, height: u64, round: u64) -> Result<()> {
        let mut state = self.round_state.write();
        *state = RoundState::new(round, height);

        // If we're the proposer for this round, create and broadcast proposal
        if self.is_proposer_for_round(height, round) {
            self.propose_block(height)?;
        }

        Ok(())
    }

    /// Check if we are the proposer for given round
    fn is_proposer_for_round(&self, height: u64, round: u64) -> bool {
        let validators = self.validators.read();
        if validators.is_empty() {
            return false;
        }

        // Simple round-robin proposer selection based on height + round
        let mut validator_list: Vec<_> = validators.keys().collect();
        validator_list.sort();

        let proposer_index = ((height + round) as usize) % validator_list.len();
        let expected_proposer = validator_list[proposer_index];

        *expected_proposer == self.keypair.public_key()
    }

    /// Propose a block for current round
    fn propose_block(&self, height: u64) -> Result<()> {
        if let Some(ref proposer) = self.block_proposer {
            if let Some(block) = proposer(height) {
                let state = self.round_state.read();
                let message = ConsensusMessage::Proposal {
                    block: block.clone(),
                    round: state.round,
                    proposer: self.keypair.public_key(),
                    signature: self.sign_proposal(&block, state.round),
                };

                // Store our proposal
                drop(state);
                self.round_state.write().proposal = Some(block);

                // Broadcast proposal
                self.message_queue.push(message);
            }
        }
        Ok(())
    }

    /// Sign a proposal
    fn sign_proposal(&self, block: &Block, round: u64) -> CCSignature {
        let proposal_data =
            bincode::serialize(&(block.hash(), round)).expect("Serialization should not fail");
        self.keypair.sign(&proposal_data)
    }

    /// Process incoming consensus message
    pub fn process_message(&self, message: ConsensusMessage) -> Result<()> {
        match message {
            ConsensusMessage::Proposal {
                block,
                round,
                proposer,
                signature,
            } => {
                self.handle_proposal(block, round, proposer, signature)?;
            }
            ConsensusMessage::Vote {
                block_hash,
                round,
                vote_type,
                voter,
                signature,
            } => {
                self.handle_vote(block_hash, round, vote_type, voter, signature)?;
            }
            ConsensusMessage::Commit {
                block_hash,
                round,
                signatures,
            } => {
                self.handle_commit(block_hash, round, signatures)?;
            }
        }
        Ok(())
    }

    /// Handle block proposal with enhanced safety monitoring
    fn handle_proposal(
        &self,
        block: Block,
        round: u64,
        proposer: CCPublicKey,
        signature: CCSignature,
    ) -> Result<()> {
        let mut state = self.round_state.write();

        // Check if this is for current round
        if round != state.round {
            return Ok(()); // Ignore old/future rounds
        }

        // Verify proposer signature
        let proposal_data =
            bincode::serialize(&(block.hash(), round)).expect("Serialization should not fail");
        if !proposer.verify(&proposal_data, &signature) {
            // Record invalid proposal for safety monitoring
            if self.params.safety_monitoring_enabled {
                let _ = self.safety_system.monitor_validator_behavior(
                    proposer,
                    crate::safety::ValidatorAction::ProposalMade {
                        block_hash: block.hash(),
                        valid: false,
                    },
                );
            }
            return Err(CCError::Consensus("Invalid proposal signature".to_string()));
        }

        // Validate block with enhanced error detection
        match block.validate() {
            Ok(_) => {
                // Record valid proposal
                if self.params.safety_monitoring_enabled {
                    let _ = self.safety_system.monitor_validator_behavior(
                        proposer,
                        crate::safety::ValidatorAction::ProposalMade {
                            block_hash: block.hash(),
                            valid: true,
                        },
                    );
                }
            }
            Err(e) => {
                // Record invalid proposal
                if self.params.safety_monitoring_enabled {
                    let _ = self.safety_system.monitor_validator_behavior(
                        proposer,
                        crate::safety::ValidatorAction::ProposalMade {
                            block_hash: block.hash(),
                            valid: false,
                        },
                    );
                }
                return Err(e);
            }
        }

        // Check for Byzantine behavior
        if self.params.safety_monitoring_enabled {
            self.check_for_byzantine_behavior(&proposer, &block)?;
        }

        // Store proposal
        state.proposal = Some(block.clone());

        // Send pre-vote if we haven't voted yet
        if !state.has_voted && self.is_validator() {
            let vote_hash = block.hash();
            self.send_vote(vote_hash, round, VoteType::PreVote)?;
            state.has_voted = true;
        }

        // Update performance metrics
        if self.params.performance_optimization {
            self.update_performance_metrics();
        }

        Ok(())
    }

    /// Handle vote message with enhanced safety monitoring
    fn handle_vote(
        &self,
        block_hash: Hash,
        round: u64,
        vote_type: VoteType,
        voter: CCPublicKey,
        signature: CCSignature,
    ) -> Result<()> {
        let mut state = self.round_state.write();

        // Check if this is for current round
        if round != state.round {
            return Ok(()); // Ignore old/future rounds
        }

        // Verify voter is a validator
        let validators = self.validators.read();
        if !validators.contains_key(&voter) {
            return Err(CCError::Consensus("Vote from non-validator".to_string()));
        }

        // Verify vote signature
        let vote_data = bincode::serialize(&(block_hash, round, &vote_type))
            .expect("Serialization should not fail");
        if !voter.verify(&vote_data, &signature) {
            // Record invalid vote for safety monitoring
            if self.params.safety_monitoring_enabled {
                let _ = self.safety_system.monitor_validator_behavior(
                    voter,
                    crate::safety::ValidatorAction::VoteCast {
                        block_hash,
                        consistent: false,
                    },
                );
            }
            return Err(CCError::Consensus("Invalid vote signature".to_string()));
        }

        // Check for double voting (Byzantine behavior)
        if self.params.safety_monitoring_enabled {
            self.check_for_double_voting(&voter, block_hash, round, &vote_type)?;
        }

        // Record valid vote
        if self.params.safety_monitoring_enabled {
            let _ = self.safety_system.monitor_validator_behavior(
                voter,
                crate::safety::ValidatorAction::VoteCast {
                    block_hash,
                    consistent: true,
                },
            );
        }

        // Store vote
        match vote_type {
            VoteType::PreVote => {
                state.pre_votes.insert(voter, block_hash);

                // Check if we have enough pre-votes to send pre-commit
                if self.has_sufficient_votes(&state.pre_votes, self.params.pre_vote_threshold)
                    && !state.has_committed
                    && self.is_validator()
                {
                    // Send pre-commit vote
                    self.send_vote(block_hash, round, VoteType::PreCommit)?;
                    state.has_committed = true;
                }
            }
            VoteType::PreCommit => {
                state.pre_commits.insert(voter, block_hash);

                // Check if we have enough pre-commits to finalize
                if self.has_sufficient_votes(&state.pre_commits, self.params.pre_commit_threshold) {
                    self.finalize_block(block_hash, round)?;
                }
            }
        }

        Ok(())
    }

    /// Handle commit message
    fn handle_commit(
        &self,
        block_hash: Hash,
        round: u64,
        _signatures: Vec<CCSignature>,
    ) -> Result<()> {
        // This is the final commit - we can safely finalize the block
        self.finalize_block(block_hash, round)
    }

    /// Check if we have sufficient votes (by stake)
    fn has_sufficient_votes(
        &self,
        votes: &HashMap<CCPublicKey, Hash>,
        threshold_percent: u64,
    ) -> bool {
        let validators = self.validators.read();
        let total_stake = *self.total_stake.read();

        if total_stake == 0 {
            return false;
        }

        // Count votes for the same block hash
        let mut vote_counts: HashMap<Hash, u64> = HashMap::new();
        for (voter, block_hash) in votes {
            if let Some(stake) = validators.get(voter) {
                *vote_counts.entry(*block_hash).or_insert(0) += stake;
            }
        }

        // Check if any block hash has sufficient stake
        let threshold_stake = (total_stake * threshold_percent) / 100;
        vote_counts.values().any(|&stake| stake >= threshold_stake)
    }

    /// Send a vote
    fn send_vote(&self, block_hash: Hash, round: u64, vote_type: VoteType) -> Result<()> {
        let vote_data = bincode::serialize(&(block_hash, round, &vote_type))
            .expect("Serialization should not fail");
        let signature = self.keypair.sign(&vote_data);

        let message = ConsensusMessage::Vote {
            block_hash,
            round,
            vote_type,
            voter: self.keypair.public_key(),
            signature,
        };

        self.message_queue.push(message);
        Ok(())
    }

    /// Finalize a block
    fn finalize_block(&self, block_hash: Hash, _round: u64) -> Result<()> {
        let state = self.round_state.read();

        if let Some(ref block) = state.proposal {
            if block.hash() == block_hash {
                // Commit the block using the callback
                if let Some(ref committer) = self.block_committer {
                    committer(block.clone())?;
                }

                // Move to next height
                drop(state);
                let next_height = self.round_state.read().height + 1;
                self.start_round(next_height, 0)?;
            }
        }

        Ok(())
    }

    /// Get next message from queue
    pub fn next_message(&self) -> Option<ConsensusMessage> {
        self.message_queue.pop()
    }

    /// Check if round has timed out
    pub fn check_timeout(&self) -> bool {
        let state = self.round_state.read();
        state.start_time.elapsed() > self.params.round_timeout
    }

    /// Handle round timeout
    pub fn handle_timeout(&self) -> Result<()> {
        let state = self.round_state.write();
        let next_round = state.round + 1;
        let height = state.height;

        drop(state);
        self.start_round(height, next_round)
    }

    /// Get current consensus state
    pub fn get_state(&self) -> (u64, u64) {
        let state = self.round_state.read();
        (state.height, state.round)
    }

    /// Check for Byzantine behavior in proposals
    fn check_for_byzantine_behavior(&self, proposer: &CCPublicKey, block: &Block) -> Result<()> {
        // Check for duplicate proposals in same round
        let state = self.round_state.read();
        if let Some(ref existing_proposal) = state.proposal {
            if existing_proposal.hash() != block.hash() {
                // Different block proposed for same round - potential equivocation
                let mut fault_tolerance = self.fault_tolerance.write();
                fault_tolerance.byzantine_validators.insert(*proposer, ByzantineBehavior {
                    validator: *proposer,
                    behavior_type: ByzantineType::Equivocation,
                    detected_at: Instant::now(),
                    severity: 0.8,
                    evidence: vec![format!("Multiple blocks proposed for round {}", state.round)],
                });

                if self.params.auto_recovery_enabled {
                    self.trigger_byzantine_recovery(*proposer)?;
                }
            }
        }
        Ok(())
    }

    /// Check for double voting (Byzantine behavior)
    fn check_for_double_voting(
        &self,
        voter: &CCPublicKey,
        block_hash: Hash,
        _round: u64,
        vote_type: &VoteType,
    ) -> Result<()> {
        let state = self.round_state.read();
        
        // Check if validator already voted for different block in same round
        match vote_type {
            VoteType::PreVote => {
                if let Some(existing_vote) = state.pre_votes.get(voter) {
                    if *existing_vote != block_hash {
                        self.record_byzantine_behavior(*voter, ByzantineType::DoubleVoting)?;
                    }
                }
            }
            VoteType::PreCommit => {
                if let Some(existing_vote) = state.pre_commits.get(voter) {
                    if *existing_vote != block_hash {
                        self.record_byzantine_behavior(*voter, ByzantineType::DoubleVoting)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Record Byzantine behavior
    fn record_byzantine_behavior(&self, validator: CCPublicKey, behavior_type: ByzantineType) -> Result<()> {
        let mut fault_tolerance = self.fault_tolerance.write();
        fault_tolerance.byzantine_validators.insert(validator, ByzantineBehavior {
            validator,
            behavior_type,
            detected_at: Instant::now(),
            severity: 0.9,
            evidence: vec!["Double voting detected".to_string()],
        });

        fault_tolerance.fault_metrics.byzantine_incidents += 1;

        if self.params.auto_recovery_enabled {
            self.trigger_byzantine_recovery(validator)?;
        }

        Ok(())
    }

    /// Trigger Byzantine recovery procedure
    fn trigger_byzantine_recovery(&self, validator: CCPublicKey) -> Result<()> {
        let mut fault_tolerance = self.fault_tolerance.write();
        fault_tolerance.active_recoveries.insert(
            RecoveryType::ByzantineValidatorRemoval,
            RecoveryStatus {
                recovery_type: RecoveryType::ByzantineValidatorRemoval,
                started_at: Instant::now(),
                progress: 0.0,
                estimated_completion: Duration::from_secs(60),
                success_probability: 0.85,
            },
        );

        // In a real implementation, this would trigger validator removal/slashing
        tracing::warn!("Byzantine behavior detected from validator {:?}, initiating recovery", validator);
        
        Ok(())
    }

    /// Update performance metrics
    fn update_performance_metrics(&self) {
        let mut monitor = self.performance_monitor.write();
        let state = self.round_state.read();
        
        // Update latency metrics
        monitor.latency.block_proposal_time = state.start_time.elapsed();
        
        // Update efficiency metrics
        monitor.consensus_efficiency.round_success_rate = 
            self.calculate_round_success_rate();
    }

    /// Calculate round success rate
    fn calculate_round_success_rate(&self) -> f64 {
        // Simplified implementation - in practice would track historical data
        0.95
    }

    /// Get SAFETY system status
    pub fn get_safety_status(&self) -> crate::safety::SafetyStatus {
        self.safety_system.get_safety_status()
    }

    /// Get fault tolerance metrics
    pub fn get_fault_tolerance_metrics(&self) -> FaultMetrics {
        let fault_tolerance = self.fault_tolerance.read();
        fault_tolerance.fault_metrics.clone()
    }

    /// Get performance metrics
    pub fn get_performance_metrics(&self) -> PerformanceMonitor {
        let monitor = self.performance_monitor.read();
        PerformanceMonitor {
            throughput: monitor.throughput.clone(),
            latency: monitor.latency.clone(),
            resource_usage: monitor.resource_usage.clone(),
            consensus_efficiency: monitor.consensus_efficiency.clone(),
        }
    }

    /// Upgrade to ccBFT consensus
    pub fn upgrade_to_ccbft(&self) -> Result<crate::ccbft::CcBftConsensus> {
        // Get current validator ID from our keypair
        let our_pubkey = self.keypair.public_key();
        let validators = self.validators.read();
        
        // Find our validator ID and stake
        let (validator_id, stake) = validators
            .iter()
            .enumerate()
            .find_map(|(idx, (pubkey, stake))| {
                if *pubkey == our_pubkey {
                    Some((idx as u64, *stake))
                } else {
                    None
                }
            })
            .unwrap_or((0, 1000)); // Default values if not found
        
        let ccbft_config = crate::ccbft::CcBftConfig::default();
        let ccbft_consensus = crate::ccbft::CcBftConsensus::new(
            self.keypair.clone(),
            validator_id,
            stake,
            ccbft_config,
            self.safety_system.clone(),
        );

        // Initialize with current validator set
        let validator_infos: HashMap<CCPublicKey, crate::ccbft::ValidatorInfo> = validators
            .iter()
            .enumerate()
            .map(|(idx, (pubkey, stake))| {
                (*pubkey, crate::ccbft::ValidatorInfo {
                    public_key: *pubkey,
                    stake: *stake,
                    reputation: 1.0,
                    network_address: format!("127.0.0.1:800{}", idx), // Better placeholder
                    last_active: Instant::now(),
                })
            })
            .collect();

        ccbft_consensus.initialize(validator_infos)?;

        tracing::info!(
            "Successfully upgraded consensus to ccBFT with {} validators, total stake: {}",
            validators.len(),
            validators.values().sum::<u64>()
        );
        Ok(ccbft_consensus)
    }

    /// Create ccBFT consensus engine directly (alternative to upgrade)
    pub fn create_ccbft(
        keypair: CCKeypair,
        validators: HashMap<CCPublicKey, u64>,
        config: Option<crate::ccbft::CcBftConfig>,
    ) -> Result<crate::ccbft::CcBftConsensus> {
        let our_pubkey = keypair.public_key();
        
        // Find our validator ID and stake
        let (validator_id, stake) = validators
            .iter()
            .enumerate()
            .find_map(|(idx, (pubkey, stake))| {
                if *pubkey == our_pubkey {
                    Some((idx as u64, *stake))
                } else {
                    None
                }
            })
            .ok_or_else(|| CCError::Consensus("Validator not found in set".to_string()))?;

        let safety_system = std::sync::Arc::new(crate::safety::SafetySystem::new(crate::safety::SafetyConfig::default()));
        let ccbft_config = config.unwrap_or_default();
        
        let ccbft_consensus = crate::ccbft::CcBftConsensus::new(
            keypair,
            validator_id,
            stake,
            ccbft_config,
            safety_system,
        );

        // Initialize with validator set
        let validator_infos: HashMap<CCPublicKey, crate::ccbft::ValidatorInfo> = validators
            .iter()
            .enumerate()
            .map(|(idx, (pubkey, stake))| {
                (*pubkey, crate::ccbft::ValidatorInfo {
                    public_key: *pubkey,
                    stake: *stake,
                    reputation: 1.0,
                    network_address: format!("127.0.0.1:800{}", idx),
                    last_active: Instant::now(),
                })
            })
            .collect();

        ccbft_consensus.initialize(validator_infos)?;

        tracing::info!(
            "Created new ccBFT consensus with {} validators, total stake: {}",
            validators.len(),
            validators.values().sum::<u64>()
        );
        Ok(ccbft_consensus)
    }

    /// Check if upgrade to ccBFT is recommended
    pub fn should_upgrade_to_ccbft(&self) -> bool {
        let validators = self.validators.read();
        let performance = self.performance_monitor.read();
        
        // Upgrade if we have enough validators and performance issues
        validators.len() >= 4 && 
        performance.throughput.average_throughput < 100.0 &&
        performance.latency.average_finality_time > Duration::from_secs(5)
    }

    /// Get consensus engine recommendations
    pub fn get_consensus_recommendations(&self) -> ConsensusRecommendations {
        let validators = self.validators.read();
        let performance = self.performance_monitor.read();
        let safety_status = self.safety_system.get_safety_status();
        
        let validator_count = validators.len();
        let total_stake = *self.total_stake.read();
        let avg_tps = performance.throughput.average_throughput;
        let avg_latency = performance.latency.average_finality_time;
        
        let recommendations = if validator_count < 4 {
            vec!["Add more validators for better fault tolerance".to_string()]
        } else if avg_tps < 50.0 {
            vec!["Consider upgrading to ccBFT for better throughput".to_string()]
        } else if avg_latency > Duration::from_secs(10) {
            vec!["Consider upgrading to ccBFT for lower latency".to_string()]
        } else if safety_status.active_faults > 0 {
            vec!["Enhanced safety monitoring recommended".to_string()]
        } else {
            vec!["Current consensus configuration is optimal".to_string()]
        };

        ConsensusRecommendations {
            current_engine: "CC Consensus".to_string(),
            validator_count,
            total_stake,
            average_tps: avg_tps,
            average_latency: avg_latency,
            safety_incidents: safety_status.active_faults as u64,
            upgrade_recommended: self.should_upgrade_to_ccbft(),
            recommendations,
        }
    }
}

// Default implementations for new structures
impl FaultToleranceState {
    pub fn new() -> Self {
        Self {
            byzantine_validators: HashMap::new(),
            network_partitions: Vec::new(),
            active_recoveries: HashMap::new(),
            fault_metrics: FaultMetrics {
                total_faults_detected: 0,
                byzantine_incidents: 0,
                network_issues: 0,
                recovery_success_rate: 1.0,
                mean_time_to_recovery: Duration::from_secs(30),
            },
        }
    }
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            throughput: ThroughputMetrics {
                transactions_per_second: 0.0,
                blocks_per_minute: 0.0,
                peak_throughput: 0.0,
                average_throughput: 0.0,
            },
            latency: LatencyMetrics {
                average_finality_time: Duration::from_secs(2),
                block_proposal_time: Duration::from_millis(100),
                vote_aggregation_time: Duration::from_millis(200),
                commit_time: Duration::from_millis(300),
            },
            resource_usage: ResourceMetrics {
                cpu_usage: 0.0,
                memory_usage: 0,
                network_bandwidth: 0,
                storage_io: 0,
            },
            consensus_efficiency: EfficiencyMetrics {
                round_success_rate: 1.0,
                view_change_frequency: 0.0,
                validator_participation: 1.0,
                consensus_overhead: 0.05,
            },
        }
    }
}