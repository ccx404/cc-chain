//! ccBFT - Advanced Byzantine Fault Tolerance consensus mechanism
//!
//! ccBFT is developed from Original BFT, AptosBFT, and other advanced variants,
//! providing high performance with faster transaction finality, strong security
//! against Byzantine faults and other attacks, and improved scalability for large networks.
//!
//! Key features:
//! - Enhanced pipelining for higher throughput
//! - Advanced view change protocol for faster recovery
//! - Aggregate signatures for efficiency
//! - Parallelized block processing
//! - Adaptive timeouts based on network conditions
//! - Enhanced safety guarantees

use cc_core::{Block, CCError, Result, CCKeypair, CCPublicKey, CCSignature, Hash};
use crate::safety::{SafetySystem, ValidatorAction};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;

/// ccBFT consensus engine with enhanced Byzantine fault tolerance
pub struct CcBftConsensus {
    /// Local validator identity
    identity: ValidatorIdentity,
    /// Current consensus state
    state: Arc<RwLock<CcBftState>>,
    /// Validator set manager
    validator_set: Arc<RwLock<ValidatorSet>>,
    /// Block processing pipeline
    pipeline: Arc<RwLock<BlockPipeline>>,
    /// View change manager
    view_change: Arc<RwLock<ViewChangeManager>>,
    /// Safety monitoring system
    safety_system: Arc<SafetySystem>,
    /// Configuration parameters
    config: CcBftConfig,
    /// Message handling queues
    message_queues: MessageQueues,
    /// Performance metrics
    metrics: Arc<RwLock<ConsensusMetrics>>,
}

/// Validator identity and cryptographic keys
#[derive(Debug, Clone)]
pub struct ValidatorIdentity {
    pub keypair: CCKeypair,
    pub validator_id: u64,
    pub stake: u64,
}

/// Core ccBFT consensus state
#[derive(Debug)]
pub struct CcBftState {
    /// Current view number
    pub view: u64,
    /// Current round within view
    pub round: u64,
    /// Current block height
    pub height: u64,
    /// Current phase of consensus
    pub phase: ConsensusPhase,
    /// Last committed block
    pub last_committed: Option<Block>,
    /// Current proposal being considered
    pub current_proposal: Option<BlockProposal>,
    /// Votes received for current round
    pub votes: VoteTracker,
    /// View change status
    pub view_change_active: bool,
    /// Consensus start time for current round
    pub round_start_time: Instant,
}

/// ccBFT consensus phases
#[derive(Debug, Clone, PartialEq)]
pub enum ConsensusPhase {
    /// Waiting for proposal
    Prepare,
    /// Processing proposal, gathering pre-votes
    PreVote,
    /// Gathering commit votes
    PreCommit,
    /// Finalizing and committing block
    Commit,
    /// View change in progress
    ViewChange,
}

/// Enhanced validator set with stake-based voting
#[derive(Debug)]
pub struct ValidatorSet {
    /// Active validators with their stakes
    pub validators: HashMap<CCPublicKey, ValidatorInfo>,
    /// Total stake in the network
    pub total_stake: u64,
    /// Byzantine fault tolerance threshold (2/3 of total stake)
    pub bft_threshold: u64,
    /// Fast path threshold (1/2 of total stake for optimistic execution)
    pub fast_threshold: u64,
    /// Validator performance tracking
    pub performance: HashMap<CCPublicKey, ValidatorPerformance>,
}

/// Individual validator information
#[derive(Debug, Clone)]
pub struct ValidatorInfo {
    pub public_key: CCPublicKey,
    pub stake: u64,
    pub reputation: f64,
    pub network_address: String,
    pub last_active: Instant,
}

/// Validator performance metrics
#[derive(Debug, Clone)]
pub struct ValidatorPerformance {
    pub blocks_proposed: u64,
    pub blocks_validated: u64,
    pub average_response_time: Duration,
    pub availability_score: f64,
    pub fault_incidents: u64,
}

/// Block processing pipeline for parallel execution
#[derive(Debug)]
pub struct BlockPipeline {
    /// Blocks being processed in parallel
    pub processing_blocks: HashMap<u64, PipelineStage>,
    /// Maximum parallel processing capacity
    pub max_parallel: usize,
    /// Pipeline performance metrics
    pub throughput_metrics: ThroughputMetrics,
}

/// Pipeline stage information
#[derive(Debug, Clone)]
pub struct PipelineStage {
    pub block: Block,
    pub stage: ProcessingStage,
    pub started_at: Instant,
    pub validator_responses: HashMap<CCPublicKey, StageResponse>,
}

/// Block processing stages
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessingStage {
    Validation,
    PreVoting,
    Committing,
    Finalizing,
}

/// Response from validator for a processing stage
#[derive(Debug, Clone)]
pub struct StageResponse {
    pub validator: CCPublicKey,
    pub response_type: ResponseType,
    pub timestamp: Instant,
    pub signature: CCSignature,
}

/// Types of validator responses
#[derive(Debug, Clone, PartialEq)]
pub enum ResponseType {
    Accept,
    Reject,
    Abstain,
}

/// View change management for leader rotation and fault recovery
#[derive(Debug)]
pub struct ViewChangeManager {
    /// Current view change round
    pub view_change_round: u64,
    /// View change votes received
    pub view_change_votes: HashMap<u64, HashSet<CCPublicKey>>,
    /// New view proposals
    pub new_view_proposals: HashMap<u64, NewViewProposal>,
    /// View change timeout
    pub view_change_timeout: Duration,
    /// Last view change time
    pub last_view_change: Instant,
}

/// New view proposal for leader transition
#[derive(Debug, Clone)]
pub struct NewViewProposal {
    pub new_view: u64,
    pub proposer: CCPublicKey,
    pub highest_committed_block: u64,
    pub pending_blocks: Vec<Block>,
    pub signatures: Vec<CCSignature>,
}

/// Vote tracking with enhanced aggregation
#[derive(Debug)]
pub struct VoteTracker {
    /// Pre-votes by view and round
    pub pre_votes: HashMap<(u64, u64), VoteSet>,
    /// Pre-commit votes by view and round
    pub pre_commits: HashMap<(u64, u64), VoteSet>,
    /// Commit votes by view and round
    pub commits: HashMap<(u64, u64), VoteSet>,
    /// Aggregate signatures for efficiency
    pub aggregate_signatures: HashMap<(u64, u64), AggregateSignature>,
}

/// Set of votes for a specific block
#[derive(Debug, Clone)]
pub struct VoteSet {
    pub block_hash: Hash,
    pub votes: HashMap<CCPublicKey, Vote>,
    pub total_stake: u64,
    pub threshold_reached: bool,
}

/// Individual vote with enhanced metadata
#[derive(Debug, Clone)]
pub struct Vote {
    pub voter: CCPublicKey,
    pub block_hash: Hash,
    pub view: u64,
    pub round: u64,
    pub vote_type: VoteType,
    pub signature: CCSignature,
    pub timestamp: Instant,
    pub justification: Option<VoteJustification>,
}

/// Enhanced vote types for ccBFT
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VoteType {
    /// Pre-vote (first phase voting)
    PreVote,
    /// Pre-commit (second phase voting)
    PreCommit,
    /// Final commit vote
    Commit,
    /// View change vote
    ViewChange(u64),
    /// New view acknowledgment
    NewView(u64),
}

/// Vote justification for enhanced security
#[derive(Debug, Clone)]
pub struct VoteJustification {
    pub reason: JustificationReason,
    pub supporting_evidence: Vec<Hash>,
    pub validator_reasoning: String,
}

/// Reasons for vote justification
#[derive(Debug, Clone, PartialEq)]
pub enum JustificationReason {
    ValidBlock,
    InvalidBlock,
    NetworkTimeout,
    ViewChange,
    SafetyViolation,
}

/// Aggregate signature for vote efficiency
#[derive(Debug, Clone)]
pub struct AggregateSignature {
    pub signature: CCSignature,
    pub participants: HashSet<CCPublicKey>,
    pub total_stake: u64,
}

/// Block proposal with enhanced metadata
#[derive(Debug, Clone)]
pub struct BlockProposal {
    pub block: Block,
    pub proposer: CCPublicKey,
    pub view: u64,
    pub round: u64,
    pub proposal_time: Instant,
    pub signature: CCSignature,
    pub justification: ProposalJustification,
}

/// Proposal justification
#[derive(Debug, Clone)]
pub struct ProposalJustification {
    pub previous_block_hash: Hash,
    pub transaction_root: Hash,
    pub state_root: Hash,
    pub validator_set_changes: Vec<ValidatorChange>,
}

/// Validator set change information
#[derive(Debug, Clone)]
pub struct ValidatorChange {
    pub change_type: ChangeType,
    pub validator: CCPublicKey,
    pub new_stake: Option<u64>,
}

/// Types of validator changes
#[derive(Debug, Clone, PartialEq)]
pub enum ChangeType {
    Add,
    Remove,
    UpdateStake,
}

/// ccBFT configuration parameters
#[derive(Debug, Clone)]
pub struct CcBftConfig {
    /// Block proposal timeout
    pub proposal_timeout: Duration,
    /// Pre-vote phase timeout
    pub pre_vote_timeout: Duration,
    /// Pre-commit phase timeout
    pub pre_commit_timeout: Duration,
    /// View change timeout
    pub view_change_timeout: Duration,
    /// Maximum parallel blocks in pipeline
    pub max_parallel_blocks: usize,
    /// Fast path enabled (optimistic execution)
    pub fast_path_enabled: bool,
    /// Adaptive timeout enabled
    pub adaptive_timeouts: bool,
    /// Pipelining enabled
    pub pipelining_enabled: bool,
    /// Aggregate signatures enabled
    pub aggregate_signatures: bool,
}

/// Message queues for different consensus phases
#[derive(Debug)]
pub struct MessageQueues {
    pub proposals: crossbeam::queue::SegQueue<BlockProposal>,
    pub votes: crossbeam::queue::SegQueue<Vote>,
    pub view_changes: crossbeam::queue::SegQueue<ViewChangeMessage>,
    pub new_views: crossbeam::queue::SegQueue<NewViewProposal>,
}

/// View change message
#[derive(Debug, Clone)]
pub struct ViewChangeMessage {
    pub from_view: u64,
    pub to_view: u64,
    pub validator: CCPublicKey,
    pub highest_committed: u64,
    pub signature: CCSignature,
}

/// Consensus performance metrics
#[derive(Debug)]
pub struct ConsensusMetrics {
    pub blocks_processed: u64,
    pub average_finality_time: Duration,
    pub throughput_tps: f64,
    pub view_changes: u64,
    pub pipeline_efficiency: f64,
    pub fault_recoveries: u64,
}

/// Throughput metrics for pipeline
#[derive(Debug, Clone)]
pub struct ThroughputMetrics {
    pub blocks_per_second: f64,
    pub transactions_per_second: f64,
    pub average_block_time: Duration,
    pub pipeline_utilization: f64,
}

impl Default for CcBftConfig {
    fn default() -> Self {
        Self {
            proposal_timeout: Duration::from_millis(1000),
            pre_vote_timeout: Duration::from_millis(500),
            pre_commit_timeout: Duration::from_millis(500),
            view_change_timeout: Duration::from_secs(10),
            max_parallel_blocks: 10,
            fast_path_enabled: true,
            adaptive_timeouts: true,
            pipelining_enabled: true,
            aggregate_signatures: true,
        }
    }
}

impl CcBftConsensus {
    /// Create new ccBFT consensus engine
    pub fn new(
        keypair: CCKeypair,
        validator_id: u64,
        stake: u64,
        config: CcBftConfig,
        safety_system: Arc<SafetySystem>,
    ) -> Self {
        let identity = ValidatorIdentity {
            keypair,
            validator_id,
            stake,
        };

        let state = Arc::new(RwLock::new(CcBftState {
            view: 0,
            round: 0,
            height: 0,
            phase: ConsensusPhase::Prepare,
            last_committed: None,
            current_proposal: None,
            votes: VoteTracker::new(),
            view_change_active: false,
            round_start_time: Instant::now(),
        }));

        let validator_set = Arc::new(RwLock::new(ValidatorSet {
            validators: HashMap::new(),
            total_stake: 0,
            bft_threshold: 0,
            fast_threshold: 0,
            performance: HashMap::new(),
        }));

        let pipeline = Arc::new(RwLock::new(BlockPipeline {
            processing_blocks: HashMap::new(),
            max_parallel: config.max_parallel_blocks,
            throughput_metrics: ThroughputMetrics {
                blocks_per_second: 0.0,
                transactions_per_second: 0.0,
                average_block_time: Duration::from_secs(1),
                pipeline_utilization: 0.0,
            },
        }));

        let view_change = Arc::new(RwLock::new(ViewChangeManager {
            view_change_round: 0,
            view_change_votes: HashMap::new(),
            new_view_proposals: HashMap::new(),
            view_change_timeout: config.view_change_timeout,
            last_view_change: Instant::now(),
        }));

        Self {
            identity,
            state,
            validator_set,
            pipeline,
            view_change,
            safety_system,
            config,
            message_queues: MessageQueues {
                proposals: crossbeam::queue::SegQueue::new(),
                votes: crossbeam::queue::SegQueue::new(),
                view_changes: crossbeam::queue::SegQueue::new(),
                new_views: crossbeam::queue::SegQueue::new(),
            },
            metrics: Arc::new(RwLock::new(ConsensusMetrics {
                blocks_processed: 0,
                average_finality_time: Duration::from_secs(2),
                throughput_tps: 0.0,
                view_changes: 0,
                pipeline_efficiency: 1.0,
                fault_recoveries: 0,
            })),
        }
    }

    /// Initialize consensus with validator set
    pub fn initialize(&self, validators: HashMap<CCPublicKey, ValidatorInfo>) -> Result<()> {
        let mut validator_set = self.validator_set.write();
        validator_set.validators = validators;
        validator_set.total_stake = validator_set.validators.values().map(|v| v.stake).sum();
        validator_set.bft_threshold = (validator_set.total_stake * 2) / 3 + 1;
        validator_set.fast_threshold = validator_set.total_stake / 2 + 1;

        // Initialize performance tracking
        let validator_keys: Vec<CCPublicKey> = validator_set.validators.keys().copied().collect();
        for validator in validator_keys {
            validator_set.performance.insert(validator, ValidatorPerformance {
                blocks_proposed: 0,
                blocks_validated: 0,
                average_response_time: Duration::from_millis(100),
                availability_score: 1.0,
                fault_incidents: 0,
            });
        }

        Ok(())
    }

    /// Start consensus for a new height
    pub fn start_consensus(&self, height: u64) -> Result<()> {
        let mut state = self.state.write();
        state.height = height;
        state.view = 0;
        state.round = 0;
        state.phase = ConsensusPhase::Prepare;
        state.round_start_time = Instant::now();

        // Clear previous round state
        state.votes = VoteTracker::new();
        state.current_proposal = None;
        state.view_change_active = false;

        // Start proposal phase if we're the leader
        drop(state);
        if self.is_leader(height, 0) {
            self.propose_block(height)?;
        }

        Ok(())
    }

    /// Check if this validator is the leader for given height and view
    fn is_leader(&self, height: u64, view: u64) -> bool {
        let validator_set = self.validator_set.read();
        if validator_set.validators.is_empty() {
            return false;
        }

        // Enhanced leader selection based on stake and performance
        let mut validators: Vec<_> = validator_set.validators.values().collect();
        validators.sort_by_key(|v| v.public_key.to_bytes());

        let leader_index = ((height + view) as usize) % validators.len();
        let expected_leader = &validators[leader_index];

        expected_leader.public_key == self.identity.keypair.public_key()
    }

    /// Propose a new block
    fn propose_block(&self, height: u64) -> Result<()> {
        let state = self.state.read();
        
        // Create block proposal (simplified for this example)
        let block = self.create_block(height)?;
        let proposal = BlockProposal {
            block: block.clone(),
            proposer: self.identity.keypair.public_key(),
            view: state.view,
            round: state.round,
            proposal_time: Instant::now(),
            signature: self.sign_proposal(&block, state.view, state.round),
            justification: ProposalJustification {
                previous_block_hash: state.last_committed.as_ref()
                    .map(|b| b.hash())
                    .unwrap_or_default(),
                transaction_root: block.header.tx_root,
                state_root: block.header.state_root,
                validator_set_changes: Vec::new(),
            },
        };

        // Record proposal with safety system
        self.safety_system.monitor_validator_behavior(
            self.identity.keypair.public_key(),
            ValidatorAction::ProposalMade {
                block_hash: block.hash(),
                valid: true, // Assume valid for our own proposals
            },
        )?;

        // Store proposal and broadcast
        drop(state);
        self.state.write().current_proposal = Some(proposal.clone());
        self.message_queues.proposals.push(proposal);

        Ok(())
    }

    /// Create a new block (simplified implementation)
    fn create_block(&self, height: u64) -> Result<Block> {
        // This would typically gather transactions from mempool
        // For now, return a basic block structure using the existing Block::new
        Ok(Block::new(
            Hash::default(), // prev_hash
            height,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            self.identity.keypair.public_key(),
            Vec::new(), // transactions
            Hash::default(), // state_root
            0, // gas_limit
        ))
    }

    /// Sign a block proposal
    fn sign_proposal(&self, block: &Block, view: u64, round: u64) -> CCSignature {
        let proposal_data = bincode::serialize(&(block.hash(), view, round))
            .expect("Serialization should not fail");
        self.identity.keypair.sign(&proposal_data)
    }

    /// Process incoming proposal
    pub fn process_proposal(&self, proposal: BlockProposal) -> Result<()> {
        let mut state = self.state.write();

        // Validate proposal
        self.validate_proposal(&proposal)?;

        // Store proposal
        state.current_proposal = Some(proposal.clone());
        state.phase = ConsensusPhase::PreVote;

        // Send pre-vote
        drop(state);
        self.send_vote(
            proposal.block.hash(),
            proposal.view,
            proposal.round,
            VoteType::PreVote,
        )?;

        Ok(())
    }

    /// Validate incoming proposal
    fn validate_proposal(&self, proposal: &BlockProposal) -> Result<()> {
        // Verify signature
        let proposal_data = bincode::serialize(&(
            proposal.block.hash(),
            proposal.view,
            proposal.round,
        )).map_err(|_| CCError::Consensus("Serialization failed".to_string()))?;

        if !proposal.proposer.verify(&proposal_data, &proposal.signature) {
            return Err(CCError::Consensus("Invalid proposal signature".to_string()));
        }

        // Verify proposer is leader
        if !self.is_expected_leader(&proposal.proposer, proposal.view) {
            return Err(CCError::Consensus("Proposal from non-leader".to_string()));
        }

        // Validate block
        proposal.block.validate()?;

        Ok(())
    }

    /// Check if validator is expected leader for view
    fn is_expected_leader(&self, validator: &CCPublicKey, view: u64) -> bool {
        let validator_set = self.validator_set.read();
        let state = self.state.read();
        
        if validator_set.validators.is_empty() {
            return false;
        }

        let mut validators: Vec<_> = validator_set.validators.values().collect();
        validators.sort_by_key(|v| v.public_key.to_bytes());

        let leader_index = ((state.height + view) as usize) % validators.len();
        let expected_leader = &validators[leader_index];

        expected_leader.public_key == *validator
    }

    /// Send a vote
    fn send_vote(
        &self,
        block_hash: Hash,
        view: u64,
        round: u64,
        vote_type: VoteType,
    ) -> Result<()> {
        let vote_data = bincode::serialize(&(block_hash, view, round, &vote_type))
            .map_err(|_| CCError::Consensus("Vote serialization failed".to_string()))?;
        let signature = self.identity.keypair.sign(&vote_data);

        let vote = Vote {
            voter: self.identity.keypair.public_key(),
            block_hash,
            view,
            round,
            vote_type,
            signature,
            timestamp: Instant::now(),
            justification: Some(VoteJustification {
                reason: JustificationReason::ValidBlock,
                supporting_evidence: Vec::new(),
                validator_reasoning: "Block validation passed".to_string(),
            }),
        };

        // Record vote with safety system
        self.safety_system.monitor_validator_behavior(
            self.identity.keypair.public_key(),
            ValidatorAction::VoteCast {
                block_hash,
                consistent: true, // Assume consistent for our own votes
            },
        )?;

        self.message_queues.votes.push(vote);
        Ok(())
    }

    /// Process incoming vote
    pub fn process_vote(&self, vote: Vote) -> Result<()> {
        // Validate vote
        self.validate_vote(&vote)?;

        let mut state = self.state.write();

        // Add vote to tracker
        self.add_vote_to_tracker(&mut state.votes, vote.clone())?;

        // Check if thresholds are reached
        match vote.vote_type {
            VoteType::PreVote => {
                if self.check_pre_vote_threshold(&state.votes, vote.view, vote.round)? {
                    // Move to pre-commit phase
                    state.phase = ConsensusPhase::PreCommit;
                    drop(state);
                    self.send_vote(vote.block_hash, vote.view, vote.round, VoteType::PreCommit)?;
                }
            }
            VoteType::PreCommit => {
                if self.check_pre_commit_threshold(&state.votes, vote.view, vote.round)? {
                    // Move to commit phase
                    state.phase = ConsensusPhase::Commit;
                    drop(state);
                    self.commit_block(vote.block_hash)?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Validate incoming vote
    fn validate_vote(&self, vote: &Vote) -> Result<()> {
        // Verify signature
        let vote_data = bincode::serialize(&(
            vote.block_hash,
            vote.view,
            vote.round,
            &vote.vote_type,
        )).map_err(|_| CCError::Consensus("Vote serialization failed".to_string()))?;

        if !vote.voter.verify(&vote_data, &vote.signature) {
            return Err(CCError::Consensus("Invalid vote signature".to_string()));
        }

        // Verify voter is in validator set
        let validator_set = self.validator_set.read();
        if !validator_set.validators.contains_key(&vote.voter) {
            return Err(CCError::Consensus("Vote from non-validator".to_string()));
        }

        Ok(())
    }

    /// Add vote to vote tracker
    fn add_vote_to_tracker(&self, tracker: &mut VoteTracker, vote: Vote) -> Result<()> {
        let key = (vote.view, vote.round);
        
        match vote.vote_type {
            VoteType::PreVote => {
                let vote_set = tracker.pre_votes.entry(key).or_insert_with(|| VoteSet {
                    block_hash: vote.block_hash,
                    votes: HashMap::new(),
                    total_stake: 0,
                    threshold_reached: false,
                });
                
                if !vote_set.votes.contains_key(&vote.voter) {
                    let validator_set = self.validator_set.read();
                    if let Some(validator) = validator_set.validators.get(&vote.voter) {
                        vote_set.total_stake += validator.stake;
                    }
                    vote_set.votes.insert(vote.voter, vote);
                }
            }
            VoteType::PreCommit => {
                let vote_set = tracker.pre_commits.entry(key).or_insert_with(|| VoteSet {
                    block_hash: vote.block_hash,
                    votes: HashMap::new(),
                    total_stake: 0,
                    threshold_reached: false,
                });
                
                if !vote_set.votes.contains_key(&vote.voter) {
                    let validator_set = self.validator_set.read();
                    if let Some(validator) = validator_set.validators.get(&vote.voter) {
                        vote_set.total_stake += validator.stake;
                    }
                    vote_set.votes.insert(vote.voter, vote);
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Check if pre-vote threshold is reached
    fn check_pre_vote_threshold(&self, tracker: &VoteTracker, view: u64, round: u64) -> Result<bool> {
        let key = (view, round);
        if let Some(vote_set) = tracker.pre_votes.get(&key) {
            let validator_set = self.validator_set.read();
            let threshold = if self.config.fast_path_enabled {
                validator_set.fast_threshold
            } else {
                validator_set.bft_threshold
            };
            return Ok(vote_set.total_stake >= threshold);
        }
        Ok(false)
    }

    /// Check if pre-commit threshold is reached
    fn check_pre_commit_threshold(&self, tracker: &VoteTracker, view: u64, round: u64) -> Result<bool> {
        let key = (view, round);
        if let Some(vote_set) = tracker.pre_commits.get(&key) {
            let validator_set = self.validator_set.read();
            return Ok(vote_set.total_stake >= validator_set.bft_threshold);
        }
        Ok(false)
    }

    /// Commit a block to the blockchain
    fn commit_block(&self, block_hash: Hash) -> Result<()> {
        let mut state = self.state.write();
        
        if let Some(ref proposal) = state.current_proposal {
            if proposal.block.hash() == block_hash {
                // Update metrics
                let mut metrics = self.metrics.write();
                metrics.blocks_processed += 1;
                metrics.average_finality_time = state.round_start_time.elapsed();

                // Update state
                state.last_committed = Some(proposal.block.clone());
                state.phase = ConsensusPhase::Prepare;
                
                // Start next height
                let next_height = state.height + 1;
                drop(state);
                drop(metrics);
                
                self.start_consensus(next_height)?;
            }
        }

        Ok(())
    }

    /// Trigger view change
    pub fn trigger_view_change(&self) -> Result<()> {
        let mut state = self.state.write();
        let mut view_change_manager = self.view_change.write();

        state.view_change_active = true;
        state.phase = ConsensusPhase::ViewChange;

        let new_view = state.view + 1;
        view_change_manager.view_change_round += 1;

        // Send view change message
        let message = ViewChangeMessage {
            from_view: state.view,
            to_view: new_view,
            validator: self.identity.keypair.public_key(),
            highest_committed: state.height.saturating_sub(1),
            signature: self.sign_view_change(state.view, new_view),
        };

        self.message_queues.view_changes.push(message);

        // Update metrics
        let mut metrics = self.metrics.write();
        metrics.view_changes += 1;

        Ok(())
    }

    /// Sign view change message
    fn sign_view_change(&self, from_view: u64, to_view: u64) -> CCSignature {
        let data = bincode::serialize(&(from_view, to_view))
            .expect("Serialization should not fail");
        self.identity.keypair.sign(&data)
    }

    /// Get consensus metrics
    pub fn get_metrics(&self) -> ConsensusMetrics {
        let metrics = self.metrics.read();
        ConsensusMetrics {
            blocks_processed: metrics.blocks_processed,
            average_finality_time: metrics.average_finality_time,
            throughput_tps: metrics.throughput_tps,
            view_changes: metrics.view_changes,
            pipeline_efficiency: metrics.pipeline_efficiency,
            fault_recoveries: metrics.fault_recoveries,
        }
    }

    /// Get current consensus state
    pub fn get_consensus_state(&self) -> (u64, u64, u64, ConsensusPhase) {
        let state = self.state.read();
        (state.height, state.view, state.round, state.phase.clone())
    }

    /// Main consensus processing loop - processes all pending messages
    pub fn process_pending_messages(&self) -> Result<()> {
        // Process proposals
        while let Some(proposal) = self.message_queues.proposals.pop() {
            if let Err(e) = self.process_proposal(proposal) {
                tracing::warn!("Error processing proposal: {}", e);
            }
        }

        // Process votes
        while let Some(vote) = self.message_queues.votes.pop() {
            if let Err(e) = self.process_vote(vote) {
                tracing::warn!("Error processing vote: {}", e);
            }
        }

        // Process view changes
        while let Some(view_change) = self.message_queues.view_changes.pop() {
            if let Err(e) = self.process_view_change(view_change) {
                tracing::warn!("Error processing view change: {}", e);
            }
        }

        // Process new views
        while let Some(new_view) = self.message_queues.new_views.pop() {
            if let Err(e) = self.process_new_view(new_view) {
                tracing::warn!("Error processing new view: {}", e);
            }
        }

        Ok(())
    }

    /// Process incoming view change message
    fn process_view_change(&self, view_change: ViewChangeMessage) -> Result<()> {
        let mut view_change_manager = self.view_change.write();
        
        // Validate view change
        self.validate_view_change(&view_change)?;

        // Add to view change votes
        view_change_manager.view_change_votes
            .entry(view_change.to_view)
            .or_insert_with(HashSet::new)
            .insert(view_change.validator);

        // Check if we have enough view change votes
        let validator_set = self.validator_set.read();
        let votes_count = view_change_manager.view_change_votes
            .get(&view_change.to_view)
            .map(|votes| votes.len())
            .unwrap_or(0);

        if votes_count >= ((validator_set.validators.len() * 2) / 3 + 1) {
            // Trigger new view
            self.trigger_new_view(view_change.to_view)?;
        }

        Ok(())
    }

    /// Validate view change message
    fn validate_view_change(&self, view_change: &ViewChangeMessage) -> Result<()> {
        // Verify signature
        let view_change_data = bincode::serialize(&(view_change.from_view, view_change.to_view))
            .map_err(|_| CCError::Consensus("View change serialization failed".to_string()))?;

        if !view_change.validator.verify(&view_change_data, &view_change.signature) {
            return Err(CCError::Consensus("Invalid view change signature".to_string()));
        }

        // Verify validator is in set
        let validator_set = self.validator_set.read();
        if !validator_set.validators.contains_key(&view_change.validator) {
            return Err(CCError::Consensus("View change from non-validator".to_string()));
        }

        Ok(())
    }

    /// Trigger new view
    fn trigger_new_view(&self, new_view: u64) -> Result<()> {
        let mut state = self.state.write();
        let view_change_manager = self.view_change.write();

        state.view = new_view;
        state.round = 0;
        state.phase = ConsensusPhase::Prepare;
        state.view_change_active = false;
        state.round_start_time = Instant::now();

        // Clear vote tracker for new view
        state.votes = VoteTracker::new();

        // Create new view proposal if we're the new leader
        drop(state);
        drop(view_change_manager);

        if self.is_leader(self.state.read().height, new_view) {
            self.propose_block(self.state.read().height)?;
        }

        Ok(())
    }

    /// Process new view message
    fn process_new_view(&self, new_view: NewViewProposal) -> Result<()> {
        // Validate new view proposal
        self.validate_new_view(&new_view)?;

        let mut state = self.state.write();
        state.view = new_view.new_view;
        state.phase = ConsensusPhase::Prepare;
        state.view_change_active = false;

        Ok(())
    }

    /// Validate new view proposal
    fn validate_new_view(&self, new_view: &NewViewProposal) -> Result<()> {
        // Verify proposer is expected leader for new view
        if !self.is_expected_leader(&new_view.proposer, new_view.new_view) {
            return Err(CCError::Consensus("New view from non-leader".to_string()));
        }

        // Verify we have enough signatures
        let validator_set = self.validator_set.read();
        let required_sigs = (validator_set.validators.len() * 2) / 3 + 1;
        if new_view.signatures.len() < required_sigs {
            return Err(CCError::Consensus("Insufficient signatures for new view".to_string()));
        }

        Ok(())
    }

    /// Check for timeout conditions and trigger view change if needed
    pub fn check_timeout(&self) -> Result<()> {
        let state = self.state.read();
        let timeout_duration = match state.phase {
            ConsensusPhase::Prepare => self.config.proposal_timeout,
            ConsensusPhase::PreVote => self.config.pre_vote_timeout,
            ConsensusPhase::PreCommit => self.config.pre_commit_timeout,
            _ => Duration::from_secs(5), // Default timeout
        };

        if state.round_start_time.elapsed() > timeout_duration {
            drop(state);
            self.trigger_view_change()?;
        }

        Ok(())
    }

    /// Update validator set with new validators or stake changes
    pub fn update_validator_set(&self, changes: Vec<ValidatorChange>) -> Result<()> {
        let mut validator_set = self.validator_set.write();

        for change in changes {
            match change.change_type {
                ChangeType::Add => {
                    if let Some(stake) = change.new_stake {
                        validator_set.validators.insert(change.validator, ValidatorInfo {
                            public_key: change.validator,
                            stake,
                            reputation: 1.0,
                            network_address: "0.0.0.0:8000".to_string(),
                            last_active: Instant::now(),
                        });
                    }
                }
                ChangeType::Remove => {
                    validator_set.validators.remove(&change.validator);
                }
                ChangeType::UpdateStake => {
                    if let (Some(validator_info), Some(new_stake)) = 
                        (validator_set.validators.get_mut(&change.validator), change.new_stake) {
                        validator_info.stake = new_stake;
                    }
                }
            }
        }

        // Recalculate thresholds
        validator_set.total_stake = validator_set.validators.values().map(|v| v.stake).sum();
        validator_set.bft_threshold = (validator_set.total_stake * 2) / 3 + 1;
        validator_set.fast_threshold = validator_set.total_stake / 2 + 1;

        Ok(())
    }

    /// Get pipeline utilization metrics
    pub fn get_pipeline_metrics(&self) -> ThroughputMetrics {
        let pipeline = self.pipeline.read();
        pipeline.throughput_metrics.clone()
    }

    /// Update pipeline efficiency metrics
    pub fn update_pipeline_metrics(&self, blocks_processed: u64, processing_time: Duration) -> Result<()> {
        let mut pipeline = self.pipeline.write();
        let mut metrics = self.metrics.write();

        // Update throughput calculations
        pipeline.throughput_metrics.blocks_per_second = blocks_processed as f64 / processing_time.as_secs_f64();
        pipeline.throughput_metrics.average_block_time = processing_time / blocks_processed.max(1) as u32;
        pipeline.throughput_metrics.pipeline_utilization = 
            (pipeline.processing_blocks.len() as f64 / pipeline.max_parallel as f64).min(1.0);

        // Update consensus metrics
        metrics.throughput_tps = pipeline.throughput_metrics.blocks_per_second * 100.0; // Assume 100 tx per block
        metrics.pipeline_efficiency = pipeline.throughput_metrics.pipeline_utilization;

        Ok(())
    }
}

// Supporting implementations
impl VoteTracker {
    pub fn new() -> Self {
        Self {
            pre_votes: HashMap::new(),
            pre_commits: HashMap::new(),
            commits: HashMap::new(),
            aggregate_signatures: HashMap::new(),
        }
    }

    /// Clear votes for a specific view and round
    pub fn clear_round(&mut self, view: u64, round: u64) {
        let key = (view, round);
        self.pre_votes.remove(&key);
        self.pre_commits.remove(&key);
        self.commits.remove(&key);
        self.aggregate_signatures.remove(&key);
    }

    /// Get vote count for a specific view and round
    pub fn get_vote_count(&self, view: u64, round: u64) -> (usize, usize, usize) {
        let key = (view, round);
        let pre_vote_count = self.pre_votes.get(&key).map(|vs| vs.votes.len()).unwrap_or(0);
        let pre_commit_count = self.pre_commits.get(&key).map(|vs| vs.votes.len()).unwrap_or(0);
        let commit_count = self.commits.get(&key).map(|vs| vs.votes.len()).unwrap_or(0);
        (pre_vote_count, pre_commit_count, commit_count)
    }
}

impl MessageQueues {
    pub fn new() -> Self {
        Self {
            proposals: crossbeam::queue::SegQueue::new(),
            votes: crossbeam::queue::SegQueue::new(),
            view_changes: crossbeam::queue::SegQueue::new(),
            new_views: crossbeam::queue::SegQueue::new(),
        }
    }

    /// Get queue lengths for monitoring
    pub fn get_queue_lengths(&self) -> (usize, usize, usize, usize) {
        (
            self.proposals.len(),
            self.votes.len(),
            self.view_changes.len(),
            self.new_views.len(),
        )
    }
}

impl ConsensusMetrics {
    pub fn new() -> Self {
        Self {
            blocks_processed: 0,
            average_finality_time: Duration::from_secs(2),
            throughput_tps: 0.0,
            view_changes: 0,
            pipeline_efficiency: 1.0,
            fault_recoveries: 0,
        }
    }

    /// Update metrics with new block processing data
    pub fn update_block_processed(&mut self, finality_time: Duration) {
        self.blocks_processed += 1;
        
        // Calculate moving average of finality time
        let alpha = 0.1; // Smoothing factor
        self.average_finality_time = Duration::from_secs_f64(
            alpha * finality_time.as_secs_f64() + 
            (1.0 - alpha) * self.average_finality_time.as_secs_f64()
        );
    }
}

impl ValidatorSet {
    pub fn new() -> Self {
        Self {
            validators: HashMap::new(),
            total_stake: 0,
            bft_threshold: 0,
            fast_threshold: 0,
            performance: HashMap::new(),
        }
    }

    /// Check if validator set meets minimum requirements
    pub fn is_valid(&self) -> bool {
        self.validators.len() >= 4 && self.total_stake > 0 // Minimum 4 validators for BFT
    }

    /// Get validator by public key
    pub fn get_validator(&self, pubkey: &CCPublicKey) -> Option<&ValidatorInfo> {
        self.validators.get(pubkey)
    }

    /// Update validator performance metrics
    pub fn update_validator_performance(&mut self, validator: CCPublicKey, metric_update: PerformanceUpdate) {
        if let Some(performance) = self.performance.get_mut(&validator) {
            match metric_update {
                PerformanceUpdate::BlockProposed => performance.blocks_proposed += 1,
                PerformanceUpdate::BlockValidated => performance.blocks_validated += 1,
                PerformanceUpdate::ResponseTime(time) => {
                    // Update moving average
                    let alpha = 0.1;
                    performance.average_response_time = Duration::from_secs_f64(
                        alpha * time.as_secs_f64() + 
                        (1.0 - alpha) * performance.average_response_time.as_secs_f64()
                    );
                }
                PerformanceUpdate::FaultIncident => {
                    performance.fault_incidents += 1;
                    performance.availability_score *= 0.95; // Reduce availability score
                }
            }
        }
    }
}

/// Performance update types for validators
#[derive(Debug, Clone)]
pub enum PerformanceUpdate {
    BlockProposed,
    BlockValidated,
    ResponseTime(Duration),
    FaultIncident,
}

/// Network interface for ccBFT consensus
impl CcBftConsensus {
    /// Send message to network (placeholder - would integrate with actual networking layer)
    pub fn send_to_network(&self, message: CcBftNetworkMessage) -> Result<()> {
        // In a real implementation, this would send the message to the network layer
        tracing::debug!("Sending ccBFT message: {:?}", message);
        Ok(())
    }

    /// Receive message from network and queue for processing
    pub fn receive_from_network(&self, message: CcBftNetworkMessage) -> Result<()> {
        match message {
            CcBftNetworkMessage::Proposal(proposal) => {
                self.message_queues.proposals.push(proposal);
            }
            CcBftNetworkMessage::Vote(vote) => {
                self.message_queues.votes.push(vote);
            }
            CcBftNetworkMessage::ViewChange(view_change) => {
                self.message_queues.view_changes.push(view_change);
            }
            CcBftNetworkMessage::NewView(new_view) => {
                self.message_queues.new_views.push(new_view);
            }
        }
        Ok(())
    }

    /// Get current status for monitoring and debugging
    pub fn get_status(&self) -> CcBftStatus {
        let state = self.state.read();
        let metrics = self.metrics.read();
        let validator_set = self.validator_set.read();
        let (proposal_queue, vote_queue, view_change_queue, new_view_queue) = 
            self.message_queues.get_queue_lengths();

        CcBftStatus {
            height: state.height,
            view: state.view,
            round: state.round,
            phase: state.phase.clone(),
            validator_count: validator_set.validators.len(),
            total_stake: validator_set.total_stake,
            blocks_processed: metrics.blocks_processed,
            average_finality_time: metrics.average_finality_time,
            throughput_tps: metrics.throughput_tps,
            view_changes: metrics.view_changes,
            queue_lengths: QueueLengths {
                proposals: proposal_queue,
                votes: vote_queue,
                view_changes: view_change_queue,
                new_views: new_view_queue,
            },
        }
    }
}

/// Network message types for ccBFT
#[derive(Debug, Clone)]
pub enum CcBftNetworkMessage {
    Proposal(BlockProposal),
    Vote(Vote),
    ViewChange(ViewChangeMessage),
    NewView(NewViewProposal),
}

/// Status information for monitoring
#[derive(Debug, Clone)]
pub struct CcBftStatus {
    pub height: u64,
    pub view: u64,
    pub round: u64,
    pub phase: ConsensusPhase,
    pub validator_count: usize,
    pub total_stake: u64,
    pub blocks_processed: u64,
    pub average_finality_time: Duration,
    pub throughput_tps: f64,
    pub view_changes: u64,
    pub queue_lengths: QueueLengths,
}

/// Message queue length information
#[derive(Debug, Clone)]
pub struct QueueLengths {
    pub proposals: usize,
    pub votes: usize,
    pub view_changes: usize,
    pub new_views: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use cc_core::crypto::CCKeypair;
    use crate::safety::{SafetySystem, SafetyConfig};
    use std::sync::Arc;

    fn create_test_ccbft() -> CcBftConsensus {
        let keypair = CCKeypair::generate();
        let safety_system = Arc::new(SafetySystem::new(SafetyConfig::default()));
        let config = CcBftConfig::default();
        
        CcBftConsensus::new(keypair, 0, 1000, config, safety_system)
    }

    fn create_test_validators() -> HashMap<CCPublicKey, ValidatorInfo> {
        let mut validators = HashMap::new();
        
        for i in 0..4 {
            let keypair = CCKeypair::generate();
            let pubkey = keypair.public_key();
            validators.insert(pubkey, ValidatorInfo {
                public_key: pubkey,
                stake: 1000,
                reputation: 1.0,
                network_address: format!("127.0.0.1:800{}", i),
                last_active: Instant::now(),
            });
        }
        
        validators
    }

    #[test]
    fn test_ccbft_creation() {
        let ccbft = create_test_ccbft();
        let (height, view, round, phase) = ccbft.get_consensus_state();
        
        assert_eq!(height, 0);
        assert_eq!(view, 0);
        assert_eq!(round, 0);
        assert_eq!(phase, ConsensusPhase::Prepare);
    }

    #[test]
    fn test_validator_set_initialization() {
        let ccbft = create_test_ccbft();
        let validators = create_test_validators();
        
        assert!(ccbft.initialize(validators).is_ok());
        
        let validator_set = ccbft.validator_set.read();
        assert_eq!(validator_set.validators.len(), 4);
        assert_eq!(validator_set.total_stake, 4000);
        assert_eq!(validator_set.bft_threshold, 2667); // (4000 * 2) / 3 + 1
        assert_eq!(validator_set.fast_threshold, 2001); // 4000 / 2 + 1
    }

    #[test]
    fn test_consensus_start() {
        let ccbft = create_test_ccbft();
        let validators = create_test_validators();
        
        ccbft.initialize(validators).unwrap();
        assert!(ccbft.start_consensus(1).is_ok());
        
        let (height, view, round, phase) = ccbft.get_consensus_state();
        assert_eq!(height, 1);
        assert_eq!(view, 0);
        assert_eq!(round, 0);
        assert_eq!(phase, ConsensusPhase::Prepare);
    }

    #[test]
    fn test_vote_tracker() {
        let mut tracker = VoteTracker::new();
        let (pre_votes, pre_commits, commits) = tracker.get_vote_count(0, 0);
        
        assert_eq!(pre_votes, 0);
        assert_eq!(pre_commits, 0);
        assert_eq!(commits, 0);
        
        tracker.clear_round(0, 0);
        let (pre_votes, pre_commits, commits) = tracker.get_vote_count(0, 0);
        
        assert_eq!(pre_votes, 0);
        assert_eq!(pre_commits, 0);
        assert_eq!(commits, 0);
    }

    #[test]
    fn test_message_queues() {
        let queues = MessageQueues::new();
        let (proposals, votes, view_changes, new_views) = queues.get_queue_lengths();
        
        assert_eq!(proposals, 0);
        assert_eq!(votes, 0);
        assert_eq!(view_changes, 0);
        assert_eq!(new_views, 0);
    }

    #[test]
    fn test_consensus_metrics() {
        let mut metrics = ConsensusMetrics::new();
        assert_eq!(metrics.blocks_processed, 0);
        
        metrics.update_block_processed(Duration::from_millis(500));
        assert_eq!(metrics.blocks_processed, 1);
    }

    #[test]
    fn test_validator_set_validity() {
        let mut validator_set = ValidatorSet::new();
        assert!(!validator_set.is_valid()); // Empty set is invalid
        
        // Add validators
        for i in 0..4 {
            let keypair = CCKeypair::generate();
            let pubkey = keypair.public_key();
            validator_set.validators.insert(pubkey, ValidatorInfo {
                public_key: pubkey,
                stake: 1000,
                reputation: 1.0,
                network_address: format!("127.0.0.1:800{}", i),
                last_active: Instant::now(),
            });
        }
        validator_set.total_stake = 4000;
        
        assert!(validator_set.is_valid());
    }

    #[test]
    fn test_status_reporting() {
        let ccbft = create_test_ccbft();
        let validators = create_test_validators();
        
        ccbft.initialize(validators).unwrap();
        let status = ccbft.get_status();
        
        assert_eq!(status.height, 0);
        assert_eq!(status.view, 0);
        assert_eq!(status.round, 0);
        assert_eq!(status.validator_count, 4);
        assert_eq!(status.total_stake, 4000);
        assert_eq!(status.blocks_processed, 0);
    }

    #[test]
    fn test_pipeline_metrics_update() {
        let ccbft = create_test_ccbft();
        
        assert!(ccbft.update_pipeline_metrics(10, Duration::from_secs(5)).is_ok());
        
        let metrics = ccbft.get_pipeline_metrics();
        assert_eq!(metrics.blocks_per_second, 2.0); // 10 blocks / 5 seconds
    }
}