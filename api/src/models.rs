//! API data models and request/response structures

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Transaction request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRequest {
    /// Sender's public key (hex-encoded)
    pub from: String,
    /// Recipient's public key (hex-encoded)  
    pub to: String,
    /// Amount to transfer
    pub amount: u64,
    /// Transaction fee
    pub fee: u64,
    /// Optional data payload (hex-encoded)
    pub data: Option<String>,
    /// Transaction signature (hex-encoded)
    pub signature: String,
}

/// Transaction response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResponse {
    /// Transaction hash
    pub hash: String,
    /// Block height where transaction is included (None if pending)
    pub block_height: Option<u64>,
    /// Block hash where transaction is included
    pub block_hash: Option<String>,
    /// Transaction index in block
    pub transaction_index: Option<u32>,
    /// Sender address
    pub from: String,
    /// Recipient address
    pub to: String,
    /// Amount transferred
    pub amount: u64,
    /// Transaction fee
    pub fee: u64,
    /// Transaction data payload
    pub data: Option<String>,
    /// Transaction status
    pub status: TransactionStatus,
    /// Gas used
    pub gas_used: Option<u64>,
    /// Transaction timestamp
    pub timestamp: DateTime<Utc>,
}

/// Transaction submission response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSubmitResponse {
    /// Hash of the submitted transaction
    pub transaction_hash: String,
}

/// Transaction status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionStatus {
    /// Transaction is pending in mempool
    Pending,
    /// Transaction has been confirmed in a block
    Confirmed,
    /// Transaction failed during execution
    Failed,
}

/// Block response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockResponse {
    /// Block hash
    pub hash: String,
    /// Block height
    pub height: u64,
    /// Previous block hash
    pub parent_hash: String,
    /// Block timestamp
    pub timestamp: DateTime<Utc>,
    /// Block proposer/miner
    pub proposer: String,
    /// Merkle root of transactions
    pub transactions_root: String,
    /// State root after applying all transactions
    pub state_root: String,
    /// List of transaction hashes in this block
    pub transactions: Vec<String>,
    /// Number of transactions
    pub transaction_count: u32,
    /// Block size in bytes
    pub size: u64,
    /// Gas limit for the block
    pub gas_limit: u64,
    /// Gas used by all transactions
    pub gas_used: u64,
}

/// Account balance response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceResponse {
    /// Account address
    pub address: String,
    /// Account balance
    pub balance: u64,
}

/// Blockchain height response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeightResponse {
    /// Current blockchain height
    pub height: u64,
}

/// Chain information response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainInfo {
    /// Chain ID
    pub chain_id: String,
    /// Chain name
    pub name: String,
    /// Current block height
    pub height: u64,
    /// Latest block hash
    pub latest_block_hash: String,
    /// Genesis block hash
    pub genesis_hash: String,
    /// Average block time in seconds
    pub avg_block_time: f64,
    /// Total number of transactions
    pub total_transactions: u64,
    /// Network version
    pub version: String,
}

/// Mempool status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolStatus {
    /// Number of pending transactions
    pub pending_count: u64,
    /// Total size of pending transactions in bytes
    pub pending_size: u64,
    /// Maximum mempool size
    pub max_size: u64,
    /// Minimum fee rate for inclusion
    pub min_fee_rate: u64,
}

/// Peer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Peer ID
    pub id: String,
    /// Peer network address
    pub address: String,
    /// Connection direction (inbound/outbound)
    pub direction: String,
    /// Peer protocol version
    pub version: String,
    /// Connection uptime in seconds
    pub uptime: u64,
    /// Last seen timestamp
    pub last_seen: DateTime<Utc>,
}

/// Network peers response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeersResponse {
    /// List of connected peers
    pub peers: Vec<PeerInfo>,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Health status
    pub status: String,
    /// Response timestamp
    pub timestamp: DateTime<Utc>,
}

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Optional error details
    pub details: Option<String>,
    /// Error timestamp
    pub timestamp: DateTime<Utc>,
}