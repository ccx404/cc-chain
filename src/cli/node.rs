use crate::{
    consensus::{CCConsensus, ConsensusMessage},
    core::crypto::{CCKeypair, CCPublicKey},
    core::state::StateManager,
    core::transaction::Transaction,
    core::utils::{AdaptiveParams, PerformanceMonitor},
    core::{
        block::{Block, Blockchain},
        error::Result,
    },
    networking::network::{LightNetworkClient, NetworkManager, NetworkMessage},
    storage::mempool::Mempool,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Node types
#[derive(Debug, Clone)]
pub enum NodeType {
    /// Full validator node with consensus participation
    Validator,
    /// Light compute node for transaction processing
    LightCompute,
    /// Wallet node for basic transaction and balance operations
    Wallet,
}

/// CC Chain node configuration
#[derive(Debug, Clone)]
pub struct NodeConfig {
    /// Node type
    pub node_type: NodeType,
    /// Network listening address
    pub listen_addr: SocketAddr,
    /// Genesis validator keys (for validators)
    pub validator_keypair: Option<CCKeypair>,
    /// Bootstrap peers
    pub bootstrap_peers: Vec<SocketAddr>,
    /// Data directory
    pub data_dir: String,
    /// Maximum mempool size
    pub max_mempool_size: usize,
    /// Enable metrics collection
    pub enable_metrics: bool,
}

/// Main CC Chain node
pub struct CCNode {
    /// Node configuration
    config: NodeConfig,
    /// Blockchain state
    blockchain: Arc<Blockchain>,
    /// State manager
    state_manager: Arc<StateManager>,
    /// Consensus engine (for validators)
    consensus: Option<Arc<CCConsensus>>,
    /// Transaction mempool
    mempool: Arc<Mempool>,
    /// Network manager
    network: Option<Arc<NetworkManager>>,
    /// Light client (for light nodes)
    light_client: Option<LightNetworkClient>,
    /// Performance monitor
    performance_monitor: Arc<PerformanceMonitor>,
    /// Adaptive parameters
    adaptive_params: Arc<parking_lot::RwLock<AdaptiveParams>>,

}

impl CCNode {
    /// Create new CC Chain node
    pub async fn new(config: NodeConfig) -> Result<Self> {
        // Initialize genesis state
        let state_manager = Arc::new(StateManager::new());

        // Create genesis block
        let genesis_keypair = CCKeypair::generate();
        let genesis_state_root = state_manager.initialize_genesis(vec![
            (genesis_keypair.public_key(), 1_000_000_000), // 1B initial tokens
        ])?;

        let genesis_block = Block::genesis(genesis_keypair.public_key(), genesis_state_root);
        let blockchain = Arc::new(Blockchain::new(genesis_block)?);

        // Initialize mempool
        let mempool = Arc::new(Mempool::new(
            config.max_mempool_size,
            100_000_000, // 100MB mempool size limit
        ));

        // Initialize performance monitoring
        let performance_monitor = Arc::new(PerformanceMonitor::new());
        let adaptive_params = Arc::new(parking_lot::RwLock::new(AdaptiveParams::new()));

        // Initialize networking based on node type
        let (network, light_client, consensus, _keypair) = match config.node_type {
            NodeType::Wallet => {
                // Wallet node connects to light compute nodes for basic operations
                let light_compute_addr = config
                    .bootstrap_peers
                    .first()
                    .copied()
                    .unwrap_or_else(|| "127.0.0.1:8000".parse().unwrap());

                let light_client = LightNetworkClient::new(light_compute_addr);
                (None, Some(light_client), None, None)
            }
            NodeType::LightCompute | NodeType::Validator => {
                // Create channels for network communication
                let (tx_sender, mut tx_receiver) = mpsc::unbounded_channel::<NetworkMessage>();
                let (consensus_sender, mut consensus_receiver) =
                    mpsc::unbounded_channel::<ConsensusMessage>();
                let (block_sender, mut block_receiver) = mpsc::unbounded_channel::<Block>();

                // Initialize network manager
                let network = Arc::new(NetworkManager::new(
                    config.listen_addr,
                    tx_sender,
                    consensus_sender,
                    block_sender,
                ));

                // Initialize consensus for validators
                let (consensus, keypair) = if matches!(config.node_type, NodeType::Validator) {
                    let keypair = config
                        .validator_keypair
                        .clone()
                        .unwrap_or_else(|| CCKeypair::generate());

                    let mut consensus_engine = CCConsensus::new(keypair.clone());

                    // Set up consensus callbacks
                    let blockchain_clone = blockchain.clone();
                    let state_manager_clone = state_manager.clone();
                    let mempool_clone = mempool.clone();

                    let keypair_clone = keypair.clone();
                    consensus_engine.set_block_proposer(move |height| {
                        let transactions =
                            mempool_clone.get_transactions_for_block(1000, 1_000_000);
                        if !transactions.is_empty() || height == 0 {
                            let prev_block = blockchain_clone
                                .get_head_block()
                                .unwrap_or_else(|| blockchain_clone.get_genesis_block().unwrap());

                            let timestamp = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_millis() as u64;

                            // Apply transactions to get new state root
                            let new_state_root = state_manager_clone
                                .apply_transactions(&transactions)
                                .unwrap_or(prev_block.header.state_root);

                            Some(Block::new(
                                prev_block.hash(),
                                height,
                                timestamp,
                                keypair_clone.public_key(),
                                transactions,
                                new_state_root,
                                10_000_000, // 10M gas limit
                            ))
                        } else {
                            None
                        }
                    });

                    let blockchain_clone = blockchain.clone();
                    let performance_monitor_clone = performance_monitor.clone();

                    consensus_engine.set_block_committer(move |block| {
                        // Add block to blockchain
                        blockchain_clone.add_block(block.clone())?;

                        // Record performance metrics
                        performance_monitor_clone.record_block(
                            block.transactions.len(),
                            std::time::Duration::from_millis(1000), // Placeholder block time
                        );

                        tracing::info!(
                            "Committed block {} at height {}",
                            hex::encode(block.hash()),
                            block.header.height
                        );

                        Ok(())
                    });

                    (Some(Arc::new(consensus_engine)), Some(keypair))
                } else {
                    (None, None)
                };

                // Start message processing tasks
                let mempool_clone = mempool.clone();
                tokio::spawn(async move {
                    while let Some(msg) = tx_receiver.recv().await {
                        if let NetworkMessage::Transaction(tx) = msg {
                            if let Err(e) = mempool_clone.add_transaction(tx) {
                                tracing::warn!("Failed to add transaction to mempool: {}", e);
                            }
                        }
                    }
                });

                if let Some(consensus_ref) = &consensus {
                    let consensus_clone = consensus_ref.clone();
                    tokio::spawn(async move {
                        while let Some(consensus_msg) = consensus_receiver.recv().await {
                            if let Err(e) = consensus_clone.process_message(consensus_msg) {
                                tracing::warn!("Failed to process consensus message: {}", e);
                            }
                        }
                    });
                }

                let blockchain_clone = blockchain.clone();
                let state_manager_clone = state_manager.clone();
                tokio::spawn(async move {
                    while let Some(block) = block_receiver.recv().await {
                        // Validate and add block
                        if let Err(e) = block.validate() {
                            tracing::warn!("Received invalid block: {}", e);
                            continue;
                        }

                        // Apply transactions to state
                        if let Err(e) = state_manager_clone.apply_transactions(&block.transactions)
                        {
                            tracing::warn!("Failed to apply block transactions: {}", e);
                            continue;
                        }

                        // Add to blockchain
                        if let Err(e) = blockchain_clone.add_block(block.clone()) {
                            tracing::warn!("Failed to add block to blockchain: {}", e);
                        } else {
                            tracing::info!(
                                "Added block {} at height {}",
                                hex::encode(block.hash()),
                                block.header.height
                            );
                        }
                    }
                });

                (Some(network), None, consensus, keypair)
            }
        };

        Ok(Self {
            config,
            blockchain,
            state_manager,
            consensus,
            mempool,
            network,
            light_client,
            performance_monitor,
            adaptive_params,
        })
    }

    /// Start the node
    pub async fn start(&self) -> Result<()> {
        tracing::info!("Starting CC Chain node ({:?})", self.config.node_type);

        match &self.config.node_type {
            NodeType::Wallet => {
                tracing::info!("Wallet node started");
                // Wallet nodes just need to connect to light compute nodes for basic operations
            }
            NodeType::LightCompute | NodeType::Validator => {
                // Start network listener
                if let Some(ref network) = self.network {
                    network.start_listener().await?;

                    // Connect to bootstrap peers
                    for peer_addr in &self.config.bootstrap_peers {
                        if let Err(e) = network.connect_to_peer(*peer_addr).await {
                            tracing::warn!(
                                "Failed to connect to bootstrap peer {}: {}",
                                peer_addr,
                                e
                            );
                        }
                    }
                }

                // Start consensus for validators
                if let Some(ref consensus) = self.consensus {
                    consensus.start_round(0, 0)?;
                    tracing::info!("Validator consensus started");
                }

                if matches!(self.config.node_type, NodeType::Validator) {
                    tracing::info!("Validator node started on {}", self.config.listen_addr);
                } else {
                    tracing::info!("Light compute node started on {}", self.config.listen_addr);
                }
            }
        }

        // Start background tasks
        self.start_background_tasks().await;

        Ok(())
    }

    /// Start background maintenance tasks
    async fn start_background_tasks(&self) {
        // Performance monitoring task
        let performance_monitor = self.performance_monitor.clone();
        let adaptive_params = self.adaptive_params.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));

            loop {
                interval.tick().await;

                // Update adaptive parameters
                adaptive_params
                    .write()
                    .adapt(100, std::time::Duration::from_millis(50));

                // Log performance metrics
                let metrics = performance_monitor.get_metrics();
                tracing::info!(
                    "Performance: TPS={:.2}, Block Time={:?}, Confirmation Time={:?}",
                    metrics.tps,
                    metrics.avg_block_time,
                    metrics.avg_confirmation_time
                );
            }
        });

        // Consensus timeout handling for validators
        if let Some(ref consensus) = self.consensus {
            let consensus_clone = consensus.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_millis(100));

                loop {
                    interval.tick().await;

                    if consensus_clone.check_timeout() {
                        if let Err(e) = consensus_clone.handle_timeout() {
                            tracing::error!("Consensus timeout error: {}", e);
                        }
                    }
                }
            });
        }
    }

    /// Submit transaction to the network
    pub async fn submit_transaction(&self, tx: Transaction) -> Result<()> {
        match &self.config.node_type {
            NodeType::Wallet => {
                if let Some(ref client) = self.light_client {
                    client.submit_transaction(tx).await?;
                }
            }
            NodeType::LightCompute | NodeType::Validator => {
                // Validate transaction
                self.state_manager.validate_transaction(&tx)?;

                // Add to mempool
                self.mempool.add_transaction(tx.clone())?;

                // Broadcast to network
                if let Some(ref network) = self.network {
                    network.broadcast(NetworkMessage::Transaction(tx)).await?;
                }
            }
        }

        Ok(())
    }

    /// Get current blockchain height
    pub fn get_height(&self) -> u64 {
        self.blockchain.get_height()
    }

    /// Get account balance
    pub fn get_balance(&self, pubkey: &CCPublicKey) -> u64 {
        self.state_manager.get_account(pubkey).balance
    }

    /// Get mempool statistics
    pub fn get_mempool_stats(&self) -> crate::mempool::MempoolStats {
        self.mempool.stats()
    }

    /// Get performance metrics
    pub fn get_performance_metrics(&self) -> crate::utils::PerformanceMetrics {
        self.performance_monitor.get_metrics()
    }

    /// Get network statistics
    pub fn get_network_stats(&self) -> Option<crate::network::NetworkStats> {
        self.network.as_ref().map(|n| n.get_stats())
    }
}
