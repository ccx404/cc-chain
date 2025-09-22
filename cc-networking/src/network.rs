use cc_core::{Block, Transaction, Result, Hash};
use cc_consensus::ConsensusMessage;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

/// Network message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// Handshake message for peer discovery
    Handshake {
        node_id: String,
        version: String,
        height: u64,
        genesis_hash: Hash,
    },
    /// Transaction propagation
    Transaction(Transaction),
    /// Block propagation
    Block(Block),
    /// Consensus message
    Consensus(ConsensusMessage),
    /// Peer list request
    PeerListRequest,
    /// Peer list response
    PeerListResponse(Vec<SocketAddr>),
    /// Block request
    BlockRequest(crate::crypto::Hash),
    /// Block response
    BlockResponse(Option<Block>),
    /// Sync request for range of blocks
    SyncRequest { start_height: u64, end_height: u64 },
    /// Sync response with blocks
    SyncResponse(Vec<Block>),
}

/// Peer information
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub address: SocketAddr,
    pub node_id: String,
    pub version: String,
    pub height: u64,
    pub last_seen: std::time::Instant,
    pub is_validator: bool,
}

/// Network manager for peer-to-peer communication
pub struct NetworkManager {
    /// Local node information
    node_id: String,
    version: String,
    local_addr: SocketAddr,

    /// Connected peers
    peers: Arc<dashmap::DashMap<String, PeerInfo>>,

    /// Message channels
    tx_sender: mpsc::UnboundedSender<NetworkMessage>,
    consensus_sender: mpsc::UnboundedSender<ConsensusMessage>,
    block_sender: mpsc::UnboundedSender<Block>,

    /// Network statistics
    stats: Arc<parking_lot::RwLock<NetworkStats>>,

    /// Validator nodes (for priority connections)
    validator_addresses: Arc<dashmap::DashSet<SocketAddr>>,
}

#[derive(Debug, Default)]
pub struct NetworkStats {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connected_peers: usize,
    pub validator_peers: usize,
}

impl NetworkManager {
    /// Create new network manager
    pub fn new(
        local_addr: SocketAddr,
        tx_sender: mpsc::UnboundedSender<NetworkMessage>,
        consensus_sender: mpsc::UnboundedSender<ConsensusMessage>,
        block_sender: mpsc::UnboundedSender<Block>,
    ) -> Self {
        let node_id = uuid::Uuid::new_v4().to_string();

        Self {
            node_id,
            version: "0.1.0".to_string(),
            local_addr,
            peers: Arc::new(dashmap::DashMap::new()),
            tx_sender,
            consensus_sender,
            block_sender,
            stats: Arc::new(parking_lot::RwLock::new(NetworkStats::default())),
            validator_addresses: Arc::new(dashmap::DashSet::new()),
        }
    }

    /// Start network listener
    pub async fn start_listener(&self) -> Result<()> {
        let listener = TcpListener::bind(self.local_addr).await?;
        tracing::info!("Network listener started on {}", self.local_addr);

        let peers = self.peers.clone();
        let stats = self.stats.clone();
        let tx_sender = self.tx_sender.clone();
        let consensus_sender = self.consensus_sender.clone();
        let block_sender = self.block_sender.clone();
        let node_id = self.node_id.clone();
        let version = self.version.clone();

        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, peer_addr)) => {
                        tracing::debug!("New connection from {}", peer_addr);

                        let peers = peers.clone();
                        let stats = stats.clone();
                        let tx_sender = tx_sender.clone();
                        let consensus_sender = consensus_sender.clone();
                        let block_sender = block_sender.clone();
                        let node_id = node_id.clone();
                        let version = version.clone();

                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_connection(
                                stream,
                                peer_addr,
                                peers,
                                stats,
                                tx_sender,
                                consensus_sender,
                                block_sender,
                                node_id,
                                version,
                            )
                            .await
                            {
                                tracing::error!("Connection error with {}: {}", peer_addr, e);
                            }
                        });
                    }
                    Err(e) => {
                        tracing::error!("Failed to accept connection: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Handle incoming connection
    async fn handle_connection(
        mut stream: TcpStream,
        peer_addr: SocketAddr,
        peers: Arc<dashmap::DashMap<String, PeerInfo>>,
        stats: Arc<parking_lot::RwLock<NetworkStats>>,
        tx_sender: mpsc::UnboundedSender<NetworkMessage>,
        consensus_sender: mpsc::UnboundedSender<ConsensusMessage>,
        block_sender: mpsc::UnboundedSender<Block>,
        node_id: String,
        version: String,
    ) -> Result<()> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        // Send handshake
        let handshake = NetworkMessage::Handshake {
            node_id: node_id.clone(),
            version: version.clone(),
            height: 0,               // TODO: Get actual height
            genesis_hash: [0u8; 32], // TODO: Get actual genesis hash
        };

        let handshake_data = bincode::serialize(&handshake)?;
        let length = handshake_data.len() as u32;
        stream.write_all(&length.to_be_bytes()).await?;
        stream.write_all(&handshake_data).await?;

        // Read peer handshake
        let mut length_buf = [0u8; 4];
        stream.read_exact(&mut length_buf).await?;
        let length = u32::from_be_bytes(length_buf) as usize;

        let mut message_buf = vec![0u8; length];
        stream.read_exact(&mut message_buf).await?;

        let peer_handshake: NetworkMessage = bincode::deserialize(&message_buf)?;

        if let NetworkMessage::Handshake {
            node_id: peer_id,
            version: peer_version,
            height,
            ..
        } = peer_handshake
        {
            // Add peer to list
            let peer_info = PeerInfo {
                address: peer_addr,
                node_id: peer_id.clone(),
                version: peer_version,
                height,
                last_seen: std::time::Instant::now(),
                is_validator: false, // TODO: Determine validator status
            };

            peers.insert(peer_id, peer_info);
            stats.write().connected_peers = peers.len();

            tracing::info!("Established connection with peer {}", peer_addr);

            // Continue reading messages
            loop {
                let mut length_buf = [0u8; 4];
                if stream.read_exact(&mut length_buf).await.is_err() {
                    break;
                }

                let length = u32::from_be_bytes(length_buf) as usize;
                if length > 10_000_000 {
                    // 10MB max message size
                    break;
                }

                let mut message_buf = vec![0u8; length];
                if stream.read_exact(&mut message_buf).await.is_err() {
                    break;
                }

                if let Ok(message) = bincode::deserialize::<NetworkMessage>(&message_buf) {
                    stats.write().messages_received += 1;
                    stats.write().bytes_received += length as u64;

                    // Route message to appropriate handler
                    match message {
                        NetworkMessage::Transaction(tx) => {
                            let _ = tx_sender.send(NetworkMessage::Transaction(tx));
                        }
                        NetworkMessage::Block(block) => {
                            let _ = block_sender.send(block);
                        }
                        NetworkMessage::Consensus(consensus_msg) => {
                            let _ = consensus_sender.send(consensus_msg);
                        }
                        _ => {
                            // Handle other message types
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Connect to a peer
    pub async fn connect_to_peer(&self, addr: SocketAddr) -> Result<()> {
        let stream = TcpStream::connect(addr).await?;

        let peers = self.peers.clone();
        let stats = self.stats.clone();
        let tx_sender = self.tx_sender.clone();
        let consensus_sender = self.consensus_sender.clone();
        let block_sender = self.block_sender.clone();
        let node_id = self.node_id.clone();
        let version = self.version.clone();

        tokio::spawn(async move {
            if let Err(e) = Self::handle_connection(
                stream,
                addr,
                peers,
                stats,
                tx_sender,
                consensus_sender,
                block_sender,
                node_id,
                version,
            )
            .await
            {
                tracing::error!("Connection error with {}: {}", addr, e);
            }
        });

        Ok(())
    }

    /// Broadcast message to all peers
    pub async fn broadcast(&self, message: NetworkMessage) -> Result<()> {
        let serialized = bincode::serialize(&message)?;
        let _length = serialized.len() as u32;

        for _peer in self.peers.iter() {
            // TODO: Send message to peer
            // This would require maintaining active connections
        }

        self.stats.write().messages_sent += self.peers.len() as u64;
        self.stats.write().bytes_sent += (serialized.len() * self.peers.len()) as u64;

        Ok(())
    }

    /// Send message to specific peer
    pub async fn send_to_peer(&self, _peer_id: &str, _message: NetworkMessage) -> Result<()> {
        // TODO: Implement sending to specific peer
        Ok(())
    }

    /// Add validator address for priority connections
    pub fn add_validator_address(&self, addr: SocketAddr) {
        self.validator_addresses.insert(addr);
    }

    /// Get connected peers
    pub fn get_peers(&self) -> Vec<PeerInfo> {
        self.peers
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get network statistics
    pub fn get_stats(&self) -> NetworkStats {
        let stats = self.stats.read();
        NetworkStats {
            messages_sent: stats.messages_sent,
            messages_received: stats.messages_received,
            bytes_sent: stats.bytes_sent,
            bytes_received: stats.bytes_received,
            connected_peers: stats.connected_peers,
            validator_peers: stats.validator_peers,
        }
    }

    /// Remove disconnected peers
    pub fn cleanup_peers(&self, timeout: std::time::Duration) {
        let now = std::time::Instant::now();
        let mut to_remove = Vec::new();

        for entry in self.peers.iter() {
            if now.duration_since(entry.last_seen) > timeout {
                to_remove.push(entry.key().clone());
            }
        }

        for peer_id in to_remove {
            self.peers.remove(&peer_id);
        }

        self.stats.write().connected_peers = self.peers.len();
    }
}

/// Lightweight network client for small nodes
pub struct LightNetworkClient {
    /// Connection to a full node
    full_node_addr: SocketAddr,
    /// Local node ID
    #[allow(dead_code)]
    node_id: String,
}

impl LightNetworkClient {
    pub fn new(full_node_addr: SocketAddr) -> Self {
        Self {
            full_node_addr,
            node_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// Connect to full node
    pub async fn connect(&self) -> Result<TcpStream> {
        let stream = TcpStream::connect(self.full_node_addr).await?;
        Ok(stream)
    }

    /// Request block by hash
    pub async fn request_block(&self, block_hash: crate::crypto::Hash) -> Result<Option<Block>> {
        let mut stream = self.connect().await?;

        let request = NetworkMessage::BlockRequest(block_hash);
        let serialized = bincode::serialize(&request)?;
        let length = serialized.len() as u32;

        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        stream.write_all(&length.to_be_bytes()).await?;
        stream.write_all(&serialized).await?;

        // Read response
        let mut length_buf = [0u8; 4];
        stream.read_exact(&mut length_buf).await?;
        let length = u32::from_be_bytes(length_buf) as usize;

        let mut response_buf = vec![0u8; length];
        stream.read_exact(&mut response_buf).await?;

        let response: NetworkMessage = bincode::deserialize(&response_buf)?;

        if let NetworkMessage::BlockResponse(block) = response {
            Ok(block)
        } else {
            Ok(None)
        }
    }

    /// Submit transaction to network
    pub async fn submit_transaction(&self, tx: Transaction) -> Result<()> {
        let mut stream = self.connect().await?;

        let message = NetworkMessage::Transaction(tx);
        let serialized = bincode::serialize(&message)?;
        let length = serialized.len() as u32;

        use tokio::io::AsyncWriteExt;

        stream.write_all(&length.to_be_bytes()).await?;
        stream.write_all(&serialized).await?;

        Ok(())
    }
}
