//! Blockchain-related API endpoints

use serde::{Deserialize, Serialize};

/// Request to get a specific block
#[derive(Debug, Deserialize)]
pub struct GetBlockRequest {
    pub height: u64,
}

/// Request to get block range
#[derive(Debug, Deserialize)]
pub struct GetBlockRangeRequest {
    pub start_height: u64,
    pub end_height: u64,
}

/// Block summary for listing
#[derive(Debug, Serialize)]
pub struct BlockSummary {
    pub height: u64,
    pub hash: String,
    pub timestamp: u64,
    pub transaction_count: u32,
    pub proposer: String,
}

/// Blockchain statistics
#[derive(Debug, Serialize)]
pub struct BlockchainStats {
    pub total_blocks: u64,
    pub total_transactions: u64,
    pub average_block_time: f64,
    pub network_hash_rate: Option<f64>,
    pub active_validators: u32,
}