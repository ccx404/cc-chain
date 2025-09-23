//! Network-related API endpoints

use serde::{Deserialize, Serialize};

/// Network status information
#[derive(Debug, Serialize)]
pub struct NetworkStatus {
    pub peer_count: u32,
    pub connected_peers: Vec<PeerSummary>,
    pub network_id: String,
    pub protocol_version: String,
    pub sync_status: SyncStatus,
}

/// Peer summary information
#[derive(Debug, Serialize)]
pub struct PeerSummary {
    pub id: String,
    pub address: String,
    pub direction: String,
    pub protocol_version: String,
    pub height: u64,
    pub latency: Option<u64>,
}

/// Network synchronization status
#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncStatus {
    Synced,
    Syncing { current_height: u64, target_height: u64 },
    NotSynced,
}

/// Network statistics request
#[derive(Debug, Deserialize)]
pub struct NetworkStatsRequest {
    pub include_peer_details: Option<bool>,
}

/// Network statistics response
#[derive(Debug, Serialize)]
pub struct NetworkStats {
    pub total_peers: u32,
    pub inbound_peers: u32,
    pub outbound_peers: u32,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub uptime: u64,
}

/// Connect to peer request
#[derive(Debug, Deserialize)]
pub struct ConnectPeerRequest {
    pub address: String,
    pub force: Option<bool>,
}

/// Disconnect from peer request
#[derive(Debug, Deserialize)]
pub struct DisconnectPeerRequest {
    pub peer_id: String,
}