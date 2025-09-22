//! Smart Contract Definitions and Management
//!
//! This module defines the structure and lifecycle of smart contracts.

use crate::crypto::Hash;
use crate::{CCError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Contract address type
pub type ContractAddress = String;

/// Represents a deployed smart contract
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Contract {
    /// Unique contract address
    pub address: String,

    /// Contract bytecode
    pub code: ContractCode,

    /// Contract storage state
    pub state: ContractState,

    /// Contract metadata
    pub metadata: ContractMetadata,

    /// Creation timestamp
    pub created_at: u64,

    /// Last execution timestamp
    pub last_executed: u64,
}

/// Contract bytecode and related information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContractCode {
    /// WASM bytecode
    pub bytecode: Vec<u8>,

    /// Code hash for verification
    pub code_hash: Hash,

    /// Code size in bytes
    pub size: usize,

    /// ABI (Application Binary Interface) definition
    pub abi: Option<ContractABI>,
}

/// Contract storage state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContractState {
    /// Key-value storage
    pub storage: HashMap<Vec<u8>, Vec<u8>>,

    /// Storage root hash
    pub storage_root: Hash,

    /// Balance held by the contract
    pub balance: u64,

    /// Nonce for contract calls
    pub nonce: u64,
}

/// Contract metadata and information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContractMetadata {
    /// Contract name
    pub name: Option<String>,

    /// Contract version
    pub version: Option<String>,

    /// Contract description
    pub description: Option<String>,

    /// Contract author
    pub author: Option<String>,

    /// License information
    pub license: Option<String>,

    /// Source code URL or IPFS hash
    pub source: Option<String>,
}

/// Application Binary Interface definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContractABI {
    /// Available functions
    pub functions: Vec<ABIFunction>,

    /// Events that can be emitted
    pub events: Vec<ABIEvent>,

    /// Constructor parameters
    pub constructor: Option<ABIFunction>,
}

/// ABI function definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ABIFunction {
    /// Function name
    pub name: String,

    /// Input parameters
    pub inputs: Vec<ABIParameter>,

    /// Output parameters
    pub outputs: Vec<ABIParameter>,

    /// Function mutability
    pub mutability: FunctionMutability,
}

/// ABI event definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ABIEvent {
    /// Event name
    pub name: String,

    /// Event parameters
    pub inputs: Vec<ABIParameter>,

    /// Whether event is anonymous
    pub anonymous: bool,
}

/// ABI parameter definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ABIParameter {
    /// Parameter name
    pub name: String,

    /// Parameter type
    pub type_name: String,

    /// Whether parameter is indexed (for events)
    pub indexed: bool,
}

/// Function mutability levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FunctionMutability {
    /// Function does not modify state
    View,

    /// Function modifies state
    Nonpayable,

    /// Function can receive native tokens
    Payable,
}

impl Contract {
    /// Create a new contract instance
    pub fn new(address: String, bytecode: Vec<u8>, metadata: ContractMetadata) -> Result<Self> {
        if bytecode.is_empty() {
            return Err(CCError::InvalidInput("Empty bytecode".to_string()));
        }

        let code_hash = blake3::hash(&bytecode);
        let code = ContractCode {
            size: bytecode.len(),
            code_hash: code_hash.into(),
            bytecode,
            abi: None,
        };

        let state = ContractState {
            storage: HashMap::new(),
            storage_root: Hash::default(),
            balance: 0,
            nonce: 0,
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Ok(Self {
            address,
            code,
            state,
            metadata,
            created_at: now,
            last_executed: now,
        })
    }

    /// Update contract state
    pub fn update_state(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        self.state.storage.insert(key, value);
        self.recalculate_storage_root();
        Ok(())
    }

    /// Get value from contract state
    pub fn get_state(&self, key: &[u8]) -> Option<&Vec<u8>> {
        self.state.storage.get(key)
    }

    /// Remove value from contract state
    pub fn remove_state(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        let result = self.state.storage.remove(key);
        if result.is_some() {
            self.recalculate_storage_root();
        }
        result
    }

    /// Update contract balance
    pub fn update_balance(&mut self, new_balance: u64) {
        self.state.balance = new_balance;
    }

    /// Increment contract nonce
    pub fn increment_nonce(&mut self) {
        self.state.nonce += 1;
    }

    /// Update last execution timestamp
    pub fn update_execution_time(&mut self) {
        self.last_executed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    /// Validate contract bytecode
    pub fn validate_bytecode(&self) -> Result<()> {
        if self.code.bytecode.is_empty() {
            return Err(CCError::InvalidInput("Empty bytecode".to_string()));
        }

        // Basic WASM header validation
        if self.code.bytecode.len() < 8 {
            return Err(CCError::InvalidInput("Invalid WASM header".to_string()));
        }

        let magic = &self.code.bytecode[0..4];
        let version = &self.code.bytecode[4..8];

        if magic != b"\0asm" {
            return Err(CCError::InvalidInput(
                "Invalid WASM magic number".to_string(),
            ));
        }

        if version != &[1, 0, 0, 0] {
            return Err(CCError::InvalidInput(
                "Unsupported WASM version".to_string(),
            ));
        }

        Ok(())
    }

    /// Calculate storage root hash
    fn recalculate_storage_root(&mut self) {
        let mut hasher = blake3::Hasher::new();

        // Sort keys for deterministic hashing
        let mut sorted_entries: Vec<_> = self.state.storage.iter().collect();
        sorted_entries.sort_by(|a, b| a.0.cmp(b.0));

        for (key, value) in sorted_entries {
            hasher.update(key);
            hasher.update(value);
        }

        self.state.storage_root = hasher.finalize().into();
    }
}

impl Default for ContractMetadata {
    fn default() -> Self {
        Self {
            name: None,
            version: None,
            description: None,
            author: None,
            license: None,
            source: None,
        }
    }
}

impl Default for ContractState {
    fn default() -> Self {
        Self {
            storage: HashMap::new(),
            storage_root: Hash::default(),
            balance: 0,
            nonce: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_creation() {
        let bytecode = b"\0asm\x01\x00\x00\x00".to_vec(); // Minimal WASM header
        let metadata = ContractMetadata::default();
        let contract = Contract::new("test_address".to_string(), bytecode, metadata);

        assert!(contract.is_ok());
        let contract = contract.unwrap();
        assert_eq!(contract.address, "test_address");
        assert_eq!(contract.code.size, 8);
    }

    #[test]
    fn test_contract_validation() {
        let bytecode = b"\0asm\x01\x00\x00\x00".to_vec();
        let metadata = ContractMetadata::default();
        let contract = Contract::new("test".to_string(), bytecode, metadata).unwrap();

        assert!(contract.validate_bytecode().is_ok());
    }

    #[test]
    fn test_contract_state_management() {
        let bytecode = b"\0asm\x01\x00\x00\x00".to_vec();
        let metadata = ContractMetadata::default();
        let mut contract = Contract::new("test".to_string(), bytecode, metadata).unwrap();

        let key = b"test_key".to_vec();
        let value = b"test_value".to_vec();

        assert!(contract.update_state(key.clone(), value.clone()).is_ok());
        assert_eq!(contract.get_state(&key), Some(&value));

        let removed = contract.remove_state(&key);
        assert_eq!(removed, Some(value));
        assert_eq!(contract.get_state(&key), None);
    }

    #[test]
    fn test_invalid_bytecode() {
        let empty_bytecode = vec![];
        let metadata = ContractMetadata::default();
        let result = Contract::new("test".to_string(), empty_bytecode, metadata);

        assert!(result.is_err());
    }
}
