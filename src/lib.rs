//! CC Chain - High efficiency blockchain with modified BFT consensus
//!
//! CC Chain is designed for high efficiency and fast speed, allowing normal PCs
//! to participate as lightweight nodes while maintaining security and performance
//! that exceeds existing blockchain solutions.

pub mod cli;
pub mod consensus;
pub mod contracts;
pub mod core;
pub mod networking;
pub mod storage;

// Re-export commonly used types for convenience
pub use core::{Block, BlockHeader, StateManager, Transaction};
pub use core::{CCError, CCKeypair, CCPublicKey, CCSignature, Hash, Result};

// Legacy re-exports for backward compatibility (will be deprecated)
pub use cli::node;
pub use contracts::vm;
pub use core::block;
pub use core::crypto;
pub use core::error;
pub use core::state;
pub use core::transaction;
pub use core::utils;
pub use networking::{bridge, network};
pub use storage::mempool;
