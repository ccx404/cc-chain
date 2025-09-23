//! Cross-chain bridge implementation

use crate::chains::{SupportedChain, ChainConfig};
use crate::validation::BridgeValidator;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Bridge error types
#[derive(Error, Debug)]
pub enum BridgeError {
    #[error("Invalid chain configuration: {0}")]
    InvalidChain(String),
    
    #[error("Insufficient validators: {0}")]
    InsufficientValidators(String),
    
    #[error("Transfer failed: {0}")]
    TransferFailed(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Chain not supported: {0}")]
    UnsupportedChain(String),
    
    #[error("Bridge is paused")]
    BridgePaused,
}

pub type Result<T> = std::result::Result<T, BridgeError>;

/// Bridge configuration for a chain pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConfig {
    /// Source blockchain
    pub source_chain: SupportedChain,
    /// Destination blockchain
    pub destination_chain: SupportedChain,
    /// Minimum required validators
    pub min_validators: u32,
    /// Confirmation blocks required
    pub confirmation_blocks: u64,
    /// Maximum transfer amount per transaction
    pub max_transfer_amount: u64,
    /// Daily transfer limit
    pub daily_limit: u64,
    /// Bridge fee rate (basis points)
    pub fee_rate: u16,
    /// Emergency pause flag
    pub paused: bool,
}

/// Cross-chain asset definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainAsset {
    /// Asset symbol (e.g., "ETH", "BTC")
    pub symbol: String,
    /// Asset name
    pub name: String,
    /// Native chain for this asset
    pub native_chain: SupportedChain,
    /// Contract addresses on different chains
    pub contract_addresses: HashMap<SupportedChain, String>,
    /// Decimal places
    pub decimals: u8,
    /// Minimum transfer amount
    pub min_transfer_amount: u64,
    /// Maximum transfer amount
    pub max_transfer_amount: u64,
}

/// Bridge transfer request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRequest {
    /// Unique transfer ID
    pub id: String,
    /// Source chain
    pub source_chain: SupportedChain,
    /// Destination chain
    pub destination_chain: SupportedChain,
    /// Asset being transferred
    pub asset_symbol: String,
    /// Transfer amount
    pub amount: u64,
    /// Sender address on source chain
    pub sender: String,
    /// Recipient address on destination chain
    pub recipient: String,
    /// Source transaction hash
    pub source_tx_hash: Option<String>,
    /// Transfer status
    pub status: TransferStatus,
    /// Creation timestamp
    pub created_at: u64,
    /// Completion timestamp
    pub completed_at: Option<u64>,
}

/// Transfer status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransferStatus {
    /// Transfer initiated, waiting for confirmations
    Pending,
    /// Transfer confirmed on source chain
    Confirmed,
    /// Transfer being processed by validators
    Processing,
    /// Transfer completed successfully
    Completed,
    /// Transfer failed
    Failed,
    /// Transfer requires manual intervention
    RequiresIntervention,
}

/// Bridge statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeStats {
    /// Total number of transfers processed
    pub total_transfers: u64,
    /// Number of successful transfers
    pub successful_transfers: u64,
    /// Number of failed transfers
    pub failed_transfers: u64,
    /// Total volume transferred (in base units)
    pub total_volume: u64,
    /// Number of active validators
    pub active_validators: u32,
    /// Number of supported chains
    pub supported_chains: u32,
    /// Number of supported assets
    pub supported_assets: u32,
    /// Bridge uptime percentage
    pub uptime_percentage: f64,
    /// Average transfer time in seconds
    pub avg_transfer_time: f64,
}

/// Main cross-chain bridge implementation
pub struct CrossChainBridge {
    /// Bridge configurations for different chain pairs
    configs: HashMap<(SupportedChain, SupportedChain), BridgeConfig>,
    /// Validators participating in the bridge
    validators: Vec<BridgeValidator>,
    /// Supported assets for cross-chain transfers
    assets: HashMap<String, CrossChainAsset>,
    /// Active transfer requests
    active_transfers: HashMap<String, TransferRequest>,
    /// Chain configurations
    chain_configs: HashMap<SupportedChain, ChainConfig>,
    /// Bridge statistics
    stats: BridgeStats,
    /// Emergency pause flag
    emergency_paused: bool,
    /// Pause reason
    pause_reason: Option<String>,
}

impl CrossChainBridge {
    /// Create a new cross-chain bridge
    pub fn new() -> Self {
        Self {
            configs: HashMap::new(),
            validators: Vec::new(),
            assets: HashMap::new(),
            active_transfers: HashMap::new(),
            chain_configs: HashMap::new(),
            stats: BridgeStats {
                total_transfers: 0,
                successful_transfers: 0,
                failed_transfers: 0,
                total_volume: 0,
                active_validators: 0,
                supported_chains: 0,
                supported_assets: 0,
                uptime_percentage: 100.0,
                avg_transfer_time: 0.0,
            },
            emergency_paused: false,
            pause_reason: None,
        }
    }
    
    /// Add a bridge configuration for a chain pair
    pub fn add_bridge_config(&mut self, config: BridgeConfig) -> Result<()> {
        if config.min_validators == 0 {
            return Err(BridgeError::InvalidChain("Minimum validators must be greater than 0".to_string()));
        }
        
        let key = (config.source_chain.clone(), config.destination_chain.clone());
        self.configs.insert(key, config);
        Ok(())
    }
    
    /// Add a chain configuration
    pub fn add_chain_config(&mut self, chain: SupportedChain, config: ChainConfig) {
        self.chain_configs.insert(chain, config);
        self.stats.supported_chains = self.chain_configs.len() as u32;
    }
    
    /// Add a bridge validator
    pub fn add_validator(&mut self, validator: BridgeValidator) -> Result<()> {
        // Check if validator already exists
        if self.validators.iter().any(|v| v.id == validator.id) {
            return Err(BridgeError::ValidationError("Validator already exists".to_string()));
        }
        
        self.validators.push(validator);
        self.stats.active_validators = self.validators.len() as u32;
        Ok(())
    }
    
    /// Add a supported asset
    pub fn add_asset(&mut self, asset: CrossChainAsset) {
        self.assets.insert(asset.symbol.clone(), asset);
        self.stats.supported_assets = self.assets.len() as u32;
    }
    
    /// Initiate a cross-chain transfer
    pub fn initiate_transfer(
        &mut self,
        source_chain: SupportedChain,
        destination_chain: SupportedChain,
        asset_symbol: String,
        amount: u64,
        sender: String,
        recipient: String,
    ) -> Result<String> {
        // Check if bridge is paused
        if self.emergency_paused {
            return Err(BridgeError::BridgePaused);
        }
        
        // Check if bridge config exists for this chain pair
        let config_key = (source_chain.clone(), destination_chain.clone());
        let config = self.configs.get(&config_key)
            .ok_or_else(|| BridgeError::UnsupportedChain(format!("{:?} -> {:?}", source_chain, destination_chain)))?;
        
        if config.paused {
            return Err(BridgeError::BridgePaused);
        }
        
        // Check if asset is supported
        let asset = self.assets.get(&asset_symbol)
            .ok_or_else(|| BridgeError::UnsupportedChain(format!("Asset {} not supported", asset_symbol)))?;
        
        // Validate transfer amount
        if amount < asset.min_transfer_amount || amount > asset.max_transfer_amount {
            return Err(BridgeError::ValidationError(
                format!("Transfer amount {} is outside allowed range [{}, {}]", 
                    amount, asset.min_transfer_amount, asset.max_transfer_amount)
            ));
        }
        
        if amount > config.max_transfer_amount {
            return Err(BridgeError::ValidationError(
                format!("Transfer amount {} exceeds bridge limit {}", amount, config.max_transfer_amount)
            ));
        }
        
        // Check if we have sufficient validators
        if self.stats.active_validators < config.min_validators {
            return Err(BridgeError::InsufficientValidators(
                format!("Need {} validators, have {}", config.min_validators, self.stats.active_validators)
            ));
        }
        
        // Generate transfer ID
        let transfer_id = format!("{}_{}", 
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            rand::random::<u32>()
        );
        
        // Create transfer request
        let transfer = TransferRequest {
            id: transfer_id.clone(),
            source_chain,
            destination_chain,
            asset_symbol,
            amount,
            sender,
            recipient,
            source_tx_hash: None,
            status: TransferStatus::Pending,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            completed_at: None,
        };
        
        self.active_transfers.insert(transfer_id.clone(), transfer);
        self.stats.total_transfers += 1;
        
        Ok(transfer_id)
    }
    
    /// Process a deposit event from source chain
    pub fn process_deposit(&mut self, transfer_id: String, source_tx_hash: String) -> Result<()> {
        let transfer = self.active_transfers.get_mut(&transfer_id)
            .ok_or_else(|| BridgeError::TransferFailed("Transfer not found".to_string()))?;
        
        transfer.source_tx_hash = Some(source_tx_hash);
        transfer.status = TransferStatus::Confirmed;
        
        // TODO: Implement actual deposit processing logic
        // This would involve:
        // 1. Validating the deposit on the source chain
        // 2. Getting validator signatures
        // 3. Creating withdrawal transaction on destination chain
        
        Ok(())
    }
    
    /// Complete a transfer
    pub fn complete_transfer(&mut self, transfer_id: String) -> Result<()> {
        let transfer = self.active_transfers.get_mut(&transfer_id)
            .ok_or_else(|| BridgeError::TransferFailed("Transfer not found".to_string()))?;
        
        transfer.status = TransferStatus::Completed;
        transfer.completed_at = Some(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs());
        
        self.stats.successful_transfers += 1;
        self.stats.total_volume += transfer.amount;
        
        Ok(())
    }
    
    /// Fail a transfer
    pub fn fail_transfer(&mut self, transfer_id: String, reason: String) -> Result<()> {
        let transfer = self.active_transfers.get_mut(&transfer_id)
            .ok_or_else(|| BridgeError::TransferFailed("Transfer not found".to_string()))?;
        
        transfer.status = TransferStatus::Failed;
        self.stats.failed_transfers += 1;
        
        tracing::warn!("Transfer {} failed: {}", transfer_id, reason);
        Ok(())
    }
    
    /// Get transfer by ID
    pub fn get_transfer(&self, transfer_id: &str) -> Option<&TransferRequest> {
        self.active_transfers.get(transfer_id)
    }
    
    /// Get all active transfers
    pub fn get_active_transfers(&self) -> Vec<&TransferRequest> {
        self.active_transfers.values().collect()
    }
    
    /// Get bridge statistics
    pub fn get_bridge_stats(&self) -> &BridgeStats {
        &self.stats
    }
    
    /// Emergency stop all bridge operations
    pub fn emergency_stop(&mut self, reason: String) -> Result<()> {
        self.emergency_paused = true;
        self.pause_reason = Some(reason.clone());
        tracing::warn!("Bridge emergency stopped: {}", reason);
        Ok(())
    }
    
    /// Reactivate bridge after emergency stop
    pub fn reactivate_bridge(&mut self) -> Result<()> {
        self.emergency_paused = false;
        self.pause_reason = None;
        tracing::info!("Bridge reactivated");
        Ok(())
    }
    
    /// Check if bridge is operational
    pub fn is_operational(&self) -> bool {
        !self.emergency_paused && self.stats.active_validators > 0
    }
}

impl Default for CrossChainBridge {
    fn default() -> Self {
        Self::new()
    }
}