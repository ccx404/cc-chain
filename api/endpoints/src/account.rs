//! Account-related API endpoints

use serde::{Deserialize, Serialize};

/// Account information request
#[derive(Debug, Deserialize)]
pub struct GetAccountRequest {
    pub address: String,
}

/// Account information response
#[derive(Debug, Serialize)]
pub struct AccountInfo {
    pub address: String,
    pub balance: u64,
    pub nonce: u64,
    pub transaction_count: u64,
    pub created_at: Option<u64>,
    pub last_activity: Option<u64>,
}

/// Account transaction history request
#[derive(Debug, Deserialize)]
pub struct AccountTransactionHistoryRequest {
    pub address: String,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub direction: Option<TransactionDirection>,
}

/// Transaction direction filter
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionDirection {
    Incoming,
    Outgoing,
    All,
}

/// Account creation request (for testing/demo purposes)
#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    pub initial_balance: Option<u64>,
}

/// Account creation response
#[derive(Debug, Serialize)]
pub struct CreateAccountResponse {
    pub address: String,
    pub private_key: String, // For demo/testing only
    pub public_key: String,
}