//! Bridge message types and handling

use crate::chains::SupportedChain;
use serde::{Deserialize, Serialize};

/// Types of cross-chain messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    /// Asset transfer between chains
    AssetTransfer,
    /// Smart contract call across chains
    ContractCall,
    /// State synchronization message
    StateSync,
    /// Validator set update
    ValidatorUpdate,
    /// Emergency stop/pause message
    EmergencyStop,
    /// Configuration update
    ConfigUpdate,
    /// Health check/ping message
    HealthCheck,
}

/// Cross-chain bridge message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeMessage {
    /// Unique message ID
    pub id: String,
    /// Message type
    pub message_type: MessageType,
    /// Source chain
    pub source_chain: SupportedChain,
    /// Destination chain
    pub destination_chain: SupportedChain,
    /// Message payload
    pub payload: MessagePayload,
    /// Message timestamp
    pub timestamp: u64,
    /// Required validator signatures
    pub required_signatures: u32,
    /// Collected validator signatures
    pub signatures: Vec<ValidatorSignature>,
    /// Message nonce (for ordering)
    pub nonce: u64,
    /// Expiration timestamp
    pub expires_at: Option<u64>,
}

/// Message payload variants
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum MessagePayload {
    /// Asset transfer payload
    AssetTransfer(AssetTransferPayload),
    /// Contract call payload
    ContractCall(ContractCallPayload),
    /// State sync payload
    StateSync(StateSyncPayload),
    /// Validator update payload
    ValidatorUpdate(ValidatorUpdatePayload),
    /// Emergency stop payload
    EmergencyStop(EmergencyStopPayload),
    /// Configuration update payload
    ConfigUpdate(ConfigUpdatePayload),
    /// Health check payload
    HealthCheck(HealthCheckPayload),
}

/// Asset transfer message payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetTransferPayload {
    /// Asset symbol
    pub asset_symbol: String,
    /// Transfer amount
    pub amount: u64,
    /// Sender address on source chain
    pub sender: String,
    /// Recipient address on destination chain
    pub recipient: String,
    /// Source transaction hash
    pub source_tx_hash: String,
    /// Source block height
    pub source_block_height: u64,
    /// Transfer fee
    pub fee: u64,
}

/// Contract call message payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractCallPayload {
    /// Target contract address
    pub contract_address: String,
    /// Function signature
    pub function_signature: String,
    /// Function arguments (encoded)
    pub args: Vec<u8>,
    /// Gas limit for the call
    pub gas_limit: u64,
    /// Value to send with the call
    pub value: u64,
    /// Caller address
    pub caller: String,
}

/// State synchronization payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSyncPayload {
    /// State root hash
    pub state_root: String,
    /// Block height
    pub block_height: u64,
    /// Block hash
    pub block_hash: String,
    /// State update data
    pub state_updates: Vec<StateUpdate>,
}

/// State update entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateUpdate {
    /// Account address
    pub address: String,
    /// Updated balance
    pub balance: Option<u64>,
    /// Updated nonce
    pub nonce: Option<u64>,
    /// Storage updates
    pub storage: Vec<StorageUpdate>,
}

/// Storage update entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageUpdate {
    /// Storage key
    pub key: String,
    /// Storage value
    pub value: String,
}

/// Validator update payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorUpdatePayload {
    /// New validator set
    pub new_validators: Vec<ValidatorInfo>,
    /// Validators to remove
    pub removed_validators: Vec<String>,
    /// New minimum required signatures
    pub new_min_signatures: u32,
    /// Update epoch
    pub epoch: u64,
}

/// Validator information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    /// Validator ID
    pub id: String,
    /// Validator public key
    pub public_key: String,
    /// Validator voting power
    pub voting_power: u64,
    /// Validator network address
    pub network_address: String,
}

/// Emergency stop payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyStopPayload {
    /// Stop reason
    pub reason: String,
    /// Affected operations
    pub affected_operations: Vec<String>,
    /// Stop duration (seconds, None for indefinite)
    pub duration: Option<u64>,
    /// Emergency contact
    pub emergency_contact: String,
}

/// Configuration update payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigUpdatePayload {
    /// Configuration key
    pub key: String,
    /// New configuration value
    pub value: String,
    /// Configuration scope (global, chain-specific, etc.)
    pub scope: String,
    /// Update reason
    pub reason: String,
}

/// Health check payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckPayload {
    /// Health status
    pub status: String,
    /// Additional metrics
    pub metrics: std::collections::HashMap<String, String>,
    /// Last block height
    pub last_block_height: u64,
    /// Peer count
    pub peer_count: u32,
}

/// Validator signature on a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorSignature {
    /// Validator ID
    pub validator_id: String,
    /// Signature bytes (hex-encoded)
    pub signature: String,
    /// Signature timestamp
    pub timestamp: u64,
}

impl BridgeMessage {
    /// Create a new bridge message
    pub fn new(
        message_type: MessageType,
        source_chain: SupportedChain,
        destination_chain: SupportedChain,
        payload: MessagePayload,
        required_signatures: u32,
        nonce: u64,
    ) -> Self {
        let id = format!("{}_{}_{}_{}", 
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            source_chain.chain_id(),
            destination_chain.chain_id(),
            rand::random::<u32>()
        );
        
        Self {
            id,
            message_type,
            source_chain,
            destination_chain,
            payload,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            required_signatures,
            signatures: Vec::new(),
            nonce,
            expires_at: None,
        }
    }
    
    /// Add a validator signature
    pub fn add_signature(&mut self, signature: ValidatorSignature) -> bool {
        // Check if this validator already signed
        if self.signatures.iter().any(|s| s.validator_id == signature.validator_id) {
            return false;
        }
        
        self.signatures.push(signature);
        self.signatures.len() >= self.required_signatures as usize
    }
    
    /// Check if message has enough signatures
    pub fn is_valid(&self) -> bool {
        self.signatures.len() >= self.required_signatures as usize
    }
    
    /// Check if message is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            now > expires_at
        } else {
            false
        }
    }
    
    /// Set expiration time
    pub fn with_expiration(mut self, expires_at: u64) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
    
    /// Calculate message hash for signing
    pub fn message_hash(&self) -> String {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(self.id.as_bytes());
        hasher.update(&self.source_chain.chain_id().to_le_bytes());
        hasher.update(&self.destination_chain.chain_id().to_le_bytes());
        hasher.update(&self.nonce.to_le_bytes());
        hasher.update(&self.timestamp.to_le_bytes());
        
        // Add payload hash
        if let Ok(payload_bytes) = serde_json::to_vec(&self.payload) {
            hasher.update(&payload_bytes);
        }
        
        hex::encode(hasher.finalize())
    }
}