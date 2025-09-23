//! Transaction-related API endpoints

use serde::{Deserialize, Serialize};

/// Request to create a new transaction
#[derive(Debug, Deserialize)]
pub struct CreateTransactionRequest {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub fee: u64,
    pub data: Option<String>,
}

/// Transaction search parameters
#[derive(Debug, Deserialize)]
pub struct TransactionSearchParams {
    pub address: Option<String>,
    pub block_height: Option<u64>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// Transaction summary for listing
#[derive(Debug, Serialize)]
pub struct TransactionSummary {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub fee: u64,
    pub block_height: Option<u64>,
    pub timestamp: u64,
}

/// Transaction fee estimation request
#[derive(Debug, Deserialize)]
pub struct FeeEstimationRequest {
    pub transaction_size: u64,
    pub priority: FeePriority,
}

/// Fee priority levels
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FeePriority {
    Low,
    Medium,
    High,
}

/// Fee estimation response
#[derive(Debug, Serialize)]
pub struct FeeEstimationResponse {
    pub estimated_fee: u64,
    pub confirmation_blocks: u32,
}