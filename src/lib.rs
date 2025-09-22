//! CC Chain - High efficiency blockchain with modified BFT consensus
//!
//! CC Chain is designed for high efficiency and fast speed, allowing normal PCs
//! to participate as lightweight nodes while maintaining security and performance
//! that exceeds existing blockchain solutions.
//!
//! This is the main crate that re-exports functionality from specialized sub-crates:
//! - `cc-chain-sdk`: Core types, cryptography, and utilities  
//! - `cc-chain-consensus`: BFT consensus implementation with safety system
//! - `cc-chain-wallet`: Lightweight client and wallet functionality
//! - `cc-chain-port`: Networking and cross-chain bridge functionality
//! - `cc-chain-vm`: Smart contract execution and WASM runtime
//! - `cc-chain-storage`: Memory pool and storage management

pub mod cli;

// Re-export from sub-crates
pub use cc_chain_sdk as core;
pub use cc_chain_consensus as consensus;
pub use cc_chain_wallet as wallet;
pub use cc_chain_port as networking;
// pub use cc_chain_vm as vm; // TODO: Fix VM crate and re-enable
pub use cc_chain_storage as storage;

// Re-export commonly used types for convenience
pub use cc_chain_sdk::{Block, BlockHeader, StateManager, Transaction};
pub use cc_chain_sdk::{CCError, CCKeypair, CCPublicKey, CCSignature, Hash, Result};

// Legacy re-exports for backward compatibility (will be deprecated)
pub use cli::node;
// pub use cc_chain_vm as contracts; // TODO: Fix VM crate and re-enable
pub use cc_chain_sdk as block;
pub use cc_chain_sdk as crypto;
pub use cc_chain_sdk as error;
pub use cc_chain_sdk as state;
pub use cc_chain_sdk as transaction;
pub use cc_chain_sdk as utils;
pub use cc_chain_port as bridge;
pub use cc_chain_port as network;
pub use cc_chain_storage as mempool;
