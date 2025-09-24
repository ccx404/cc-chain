//! CC Chain API Handlers
//!
//! This module provides comprehensive request handlers for the CC Chain API,
//! including handlers for blocks, transactions, accounts, and network information.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HandlerError {
    #[error("Invalid request parameter: {param} - {reason}")]
    InvalidParameter { param: String, reason: String },
    #[error("Resource not found: {resource}")]
    NotFound { resource: String },
    #[error("Internal server error: {0}")]
    Internal(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

pub type Result<T> = std::result::Result<T, HandlerError>;

/// Standard API response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub pagination: Option<PaginationInfo>,
    pub metadata: HashMap<String, String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            pagination: None,
            metadata: HashMap::new(),
        }
    }

    pub fn success_with_pagination(data: T, pagination: PaginationInfo) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            pagination: Some(pagination),
            metadata: HashMap::new(),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            pagination: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Pagination information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub page: u32,
    pub per_page: u32,
    pub total_items: u64,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

impl PaginationInfo {
    pub fn new(page: u32, per_page: u32, total_items: u64) -> Self {
        let total_pages = ((total_items as f64) / (per_page as f64)).ceil() as u32;
        
        Self {
            page,
            per_page,
            total_items,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        }
    }
}

/// Block data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub hash: String,
    pub height: u64,
    pub parent_hash: String,
    pub timestamp: u64,
    pub proposer: String,
    pub transaction_count: u32,
    pub transactions: Vec<String>, // Transaction hashes
    pub gas_used: u64,
    pub gas_limit: u64,
    pub size: u64,
}

/// Transaction data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub hash: String,
    pub block_hash: Option<String>,
    pub block_height: Option<u64>,
    pub transaction_index: Option<u32>,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub fee: u64,
    pub gas_limit: u64,
    pub gas_used: Option<u64>,
    pub status: TransactionStatus,
    pub timestamp: u64,
    pub data: Option<String>,
}

/// Transaction status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
}

/// Account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub address: String,
    pub balance: u64,
    pub nonce: u64,
    pub transaction_count: u64,
    pub last_activity: Option<u64>,
}

/// Network information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub chain_id: String,
    pub network_name: String,
    pub latest_height: u64,
    pub latest_block_hash: String,
    pub total_transactions: u64,
    pub peer_count: u32,
    pub sync_status: SyncStatus,
    pub version: String,
}

/// Sync status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncStatus {
    Synced,
    Syncing { current: u64, target: u64 },
    NotSynced,
}

/// Block handler
pub struct BlockHandler {
    blocks: HashMap<String, Block>, // In-memory store for demo
    height_to_hash: HashMap<u64, String>,
}

impl BlockHandler {
    pub fn new() -> Self {
        let mut handler = Self {
            blocks: HashMap::new(),
            height_to_hash: HashMap::new(),
        };
        
        // Add some sample blocks
        handler.add_sample_data();
        handler
    }

    /// Get block by hash
    pub fn get_block_by_hash(&self, hash: &str) -> Result<ApiResponse<Block>> {
        match self.blocks.get(hash) {
            Some(block) => Ok(ApiResponse::success(block.clone())),
            None => Err(HandlerError::NotFound {
                resource: format!("Block with hash {}", hash),
            }),
        }
    }

    /// Get block by height
    pub fn get_block_by_height(&self, height: u64) -> Result<ApiResponse<Block>> {
        match self.height_to_hash.get(&height) {
            Some(hash) => self.get_block_by_hash(hash),
            None => Err(HandlerError::NotFound {
                resource: format!("Block at height {}", height),
            }),
        }
    }

    /// Get latest block
    pub fn get_latest_block(&self) -> Result<ApiResponse<Block>> {
        let max_height = self.height_to_hash.keys().max();
        match max_height {
            Some(height) => self.get_block_by_height(*height),
            None => Err(HandlerError::NotFound {
                resource: "No blocks available".to_string(),
            }),
        }
    }

    /// List blocks with pagination
    pub fn list_blocks(&self, page: u32, per_page: u32) -> Result<ApiResponse<Vec<Block>>> {
        let total_items = self.blocks.len() as u64;
        let pagination = PaginationInfo::new(page, per_page, total_items);

        let offset = ((page - 1) * per_page) as usize;
        let limit = per_page as usize;

        let mut blocks: Vec<Block> = self.blocks.values().cloned().collect();
        blocks.sort_by(|a, b| b.height.cmp(&a.height)); // Sort by height descending

        let page_blocks = blocks
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect();

        Ok(ApiResponse::success_with_pagination(page_blocks, pagination))
    }

    fn add_sample_data(&mut self) {
        for i in 1..=10 {
            let hash = format!("0x{:064x}", i);
            let block = Block {
                hash: hash.clone(),
                height: i,
                parent_hash: if i > 1 { format!("0x{:064x}", i - 1) } else { "0x0000000000000000000000000000000000000000000000000000000000000000".to_string() },
                timestamp: 1640995200 + (i * 60), // Mock timestamps
                proposer: format!("validator_{}", i % 3 + 1),
                transaction_count: (i % 5) as u32,
                transactions: (0..(i % 5)).map(|j| format!("0x{:064x}", i * 1000 + j)).collect(),
                gas_used: i * 21000,
                gas_limit: 10000000,
                size: 1024 + (i * 100),
            };
            
            self.height_to_hash.insert(i, hash.clone());
            self.blocks.insert(hash, block);
        }
    }
}

impl Default for BlockHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Transaction handler
pub struct TransactionHandler {
    transactions: HashMap<String, Transaction>,
}

impl TransactionHandler {
    pub fn new() -> Self {
        let mut handler = Self {
            transactions: HashMap::new(),
        };
        
        handler.add_sample_data();
        handler
    }

    /// Get transaction by hash
    pub fn get_transaction(&self, hash: &str) -> Result<ApiResponse<Transaction>> {
        match self.transactions.get(hash) {
            Some(tx) => Ok(ApiResponse::success(tx.clone())),
            None => Err(HandlerError::NotFound {
                resource: format!("Transaction with hash {}", hash),
            }),
        }
    }

    /// Submit new transaction
    pub fn submit_transaction(&mut self, tx_data: SubmitTransactionRequest) -> Result<ApiResponse<SubmitTransactionResponse>> {
        // Validate transaction data
        self.validate_transaction(&tx_data)?;

        // Generate transaction hash (simplified)
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        tx_data.from.hash(&mut hasher);
        tx_data.to.hash(&mut hasher);
        tx_data.amount.hash(&mut hasher);
        let tx_hash = format!("0x{:064x}", hasher.finish());

        // Create transaction
        let transaction = Transaction {
            hash: tx_hash.clone(),
            block_hash: None, // Will be set when included in block
            block_height: None,
            transaction_index: None,
            from: tx_data.from,
            to: tx_data.to,
            amount: tx_data.amount,
            fee: tx_data.fee,
            gas_limit: tx_data.gas_limit.unwrap_or(21000),
            gas_used: None,
            status: TransactionStatus::Pending,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            data: tx_data.data,
        };

        self.transactions.insert(tx_hash.clone(), transaction);

        Ok(ApiResponse::success(SubmitTransactionResponse {
            transaction_hash: tx_hash,
            status: "pending".to_string(),
        }))
    }

    /// List transactions with pagination
    pub fn list_transactions(&self, page: u32, per_page: u32) -> Result<ApiResponse<Vec<Transaction>>> {
        let total_items = self.transactions.len() as u64;
        let pagination = PaginationInfo::new(page, per_page, total_items);

        let offset = ((page - 1) * per_page) as usize;
        let limit = per_page as usize;

        let mut transactions: Vec<Transaction> = self.transactions.values().cloned().collect();
        transactions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp)); // Sort by timestamp descending

        let page_transactions = transactions
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect();

        Ok(ApiResponse::success_with_pagination(page_transactions, pagination))
    }

    fn validate_transaction(&self, tx_data: &SubmitTransactionRequest) -> Result<()> {
        if tx_data.from.is_empty() {
            return Err(HandlerError::InvalidParameter {
                param: "from".to_string(),
                reason: "From address cannot be empty".to_string(),
            });
        }

        if tx_data.to.is_empty() {
            return Err(HandlerError::InvalidParameter {
                param: "to".to_string(),
                reason: "To address cannot be empty".to_string(),
            });
        }

        if tx_data.amount == 0 {
            return Err(HandlerError::InvalidParameter {
                param: "amount".to_string(),
                reason: "Amount must be greater than 0".to_string(),
            });
        }

        if tx_data.fee < 100 {
            return Err(HandlerError::InvalidParameter {
                param: "fee".to_string(),
                reason: "Fee must be at least 100".to_string(),
            });
        }

        Ok(())
    }

    fn add_sample_data(&mut self) {
        for i in 1..=5 {
            let hash = format!("0x{:064x}", i * 1000);
            let transaction = Transaction {
                hash: hash.clone(),
                block_hash: Some(format!("0x{:064x}", i)),
                block_height: Some(i),
                transaction_index: Some(0),
                from: format!("0x{:040x}", i * 10),
                to: format!("0x{:040x}", i * 10 + 1),
                amount: i * 1000,
                fee: 200,
                gas_limit: 21000,
                gas_used: Some(21000),
                status: TransactionStatus::Confirmed,
                timestamp: 1640995200 + (i * 60),
                data: None,
            };
            
            self.transactions.insert(hash, transaction);
        }
    }
}

impl Default for TransactionHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Account handler
pub struct AccountHandler {
    accounts: HashMap<String, Account>,
}

impl AccountHandler {
    pub fn new() -> Self {
        let mut handler = Self {
            accounts: HashMap::new(),
        };
        
        handler.add_sample_data();
        handler
    }

    /// Get account information
    pub fn get_account(&self, address: &str) -> Result<ApiResponse<Account>> {
        match self.accounts.get(address) {
            Some(account) => Ok(ApiResponse::success(account.clone())),
            None => {
                // Return default account for unknown addresses
                Ok(ApiResponse::success(Account {
                    address: address.to_string(),
                    balance: 0,
                    nonce: 0,
                    transaction_count: 0,
                    last_activity: None,
                }))
            }
        }
    }

    /// Get account balance
    pub fn get_balance(&self, address: &str) -> Result<ApiResponse<BalanceResponse>> {
        let account = match self.accounts.get(address) {
            Some(acc) => acc.clone(),
            None => Account {
                address: address.to_string(),
                balance: 0,
                nonce: 0,
                transaction_count: 0,
                last_activity: None,
            },
        };

        Ok(ApiResponse::success(BalanceResponse {
            address: account.address,
            balance: account.balance,
            nonce: account.nonce,
        }))
    }

    fn add_sample_data(&mut self) {
        for i in 1..=5 {
            let address = format!("0x{:040x}", i * 10);
            let account = Account {
                address: address.clone(),
                balance: i * 1000000,
                nonce: i * 5,
                transaction_count: i * 3,
                last_activity: Some(1640995200 + (i * 3600)),
            };
            
            self.accounts.insert(address, account);
        }
    }
}

impl Default for AccountHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Network handler
pub struct NetworkHandler;

impl NetworkHandler {
    pub fn new() -> Self {
        Self
    }

    /// Get network information
    pub fn get_network_info(&self) -> Result<ApiResponse<NetworkInfo>> {
        let network_info = NetworkInfo {
            chain_id: "cc-chain-mainnet".to_string(),
            network_name: "CC Chain Mainnet".to_string(),
            latest_height: 12345,
            latest_block_hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            total_transactions: 987654,
            peer_count: 42,
            sync_status: SyncStatus::Synced,
            version: "1.0.0".to_string(),
        };

        Ok(ApiResponse::success(network_info))
    }

    /// Get node status
    pub fn get_node_status(&self) -> Result<ApiResponse<NodeStatus>> {
        let node_status = NodeStatus {
            is_syncing: false,
            is_validator: true,
            uptime: 7200, // 2 hours
            connections: 42,
            memory_usage: 512 * 1024 * 1024, // 512 MB
            disk_usage: 10 * 1024 * 1024 * 1024, // 10 GB
        };

        Ok(ApiResponse::success(node_status))
    }
}

impl Default for NetworkHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Request/Response structures
#[derive(Debug, Deserialize, Hash)]
pub struct SubmitTransactionRequest {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub fee: u64,
    pub gas_limit: Option<u64>,
    pub data: Option<String>,
    pub signature: String,
}

#[derive(Debug, Serialize)]
pub struct SubmitTransactionResponse {
    pub transaction_hash: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct BalanceResponse {
    pub address: String,
    pub balance: u64,
    pub nonce: u64,
}

#[derive(Debug, Serialize)]
pub struct NodeStatus {
    pub is_syncing: bool,
    pub is_validator: bool,
    pub uptime: u64,
    pub connections: u32,
    pub memory_usage: u64,
    pub disk_usage: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_handler_get_by_height() {
        let handler = BlockHandler::new();
        let result = handler.get_block_by_height(1);
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.success);
        assert!(response.data.is_some());
        
        let block = response.data.unwrap();
        assert_eq!(block.height, 1);
    }

    #[test]
    fn test_block_handler_get_by_hash() {
        let handler = BlockHandler::new();
        let hash = "0x0000000000000000000000000000000000000000000000000000000000000001";
        let result = handler.get_block_by_hash(hash);
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.success);
        assert!(response.data.is_some());
    }

    #[test]
    fn test_block_handler_not_found() {
        let handler = BlockHandler::new();
        let result = handler.get_block_by_height(999);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), HandlerError::NotFound { .. }));
    }

    #[test]
    fn test_block_handler_list_with_pagination() {
        let handler = BlockHandler::new();
        let result = handler.list_blocks(1, 5);
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.success);
        assert!(response.data.is_some());
        assert!(response.pagination.is_some());
        
        let blocks = response.data.unwrap();
        assert!(blocks.len() <= 5);
        
        let pagination = response.pagination.unwrap();
        assert_eq!(pagination.page, 1);
        assert_eq!(pagination.per_page, 5);
    }

    #[test]
    fn test_transaction_handler_submit() {
        let mut handler = TransactionHandler::new();
        let request = SubmitTransactionRequest {
            from: "0x1234567890123456789012345678901234567890".to_string(),
            to: "0xabcdefabcdefabcdefabcdefabcdefabcdefabcdef".to_string(),
            amount: 1000,
            fee: 200,
            gas_limit: Some(21000),
            data: None,
            signature: "0x123...".to_string(),
        };
        
        let result = handler.submit_transaction(request);
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert!(response.success);
        assert!(response.data.is_some());
        
        let submit_response = response.data.unwrap();
        assert!(!submit_response.transaction_hash.is_empty());
        assert_eq!(submit_response.status, "pending");
    }

    #[test]
    fn test_transaction_handler_validation() {
        let mut handler = TransactionHandler::new();
        let invalid_request = SubmitTransactionRequest {
            from: "".to_string(), // Invalid: empty
            to: "0xabcdefabcdefabcdefabcdefabcdefabcdefabcdef".to_string(),
            amount: 0, // Invalid: zero amount
            fee: 50,   // Invalid: too low
            gas_limit: Some(21000),
            data: None,
            signature: "0x123...".to_string(),
        };
        
        let result = handler.submit_transaction(invalid_request);
        assert!(result.is_err());
    }

    #[test]
    fn test_account_handler_get_existing() {
        let handler = AccountHandler::new();
        let address = "0x000000000000000000000000000000000000000a";
        let result = handler.get_account(address);
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.success);
        assert!(response.data.is_some());
        
        let account = response.data.unwrap();
        assert_eq!(account.address, address);
    }

    #[test]
    fn test_account_handler_get_nonexistent() {
        let handler = AccountHandler::new();
        let address = "0x9999999999999999999999999999999999999999";
        let result = handler.get_account(address);
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.success);
        assert!(response.data.is_some());
        
        let account = response.data.unwrap();
        assert_eq!(account.address, address);
        assert_eq!(account.balance, 0);
    }

    #[test]
    fn test_network_handler_info() {
        let handler = NetworkHandler::new();
        let result = handler.get_network_info();
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.success);
        assert!(response.data.is_some());
        
        let network_info = response.data.unwrap();
        assert_eq!(network_info.chain_id, "cc-chain-mainnet");
    }

    #[test]
    fn test_pagination_info() {
        let pagination = PaginationInfo::new(2, 10, 25);
        
        assert_eq!(pagination.page, 2);
        assert_eq!(pagination.per_page, 10);
        assert_eq!(pagination.total_items, 25);
        assert_eq!(pagination.total_pages, 3);
        assert!(pagination.has_prev);
        assert!(pagination.has_next);
    }

    #[test]
    fn test_api_response_success() {
        let response = ApiResponse::success("test_data".to_string());
        assert!(response.success);
        assert_eq!(response.data, Some("test_data".to_string()));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_api_response_error() {
        let response: ApiResponse<String> = ApiResponse::error("test error".to_string());
        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.error, Some("test error".to_string()));
    }
}
