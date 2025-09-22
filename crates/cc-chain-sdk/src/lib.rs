//! CC Chain SDK - Core blockchain components
//!
//! This SDK provides the fundamental building blocks of the CC Chain blockchain:
//! - Block and transaction structures
//! - State management
//! - Cryptographic primitives
//! - Error handling
//! - Utility functions

pub mod block;
pub mod crypto;
pub mod error;
pub mod state;
pub mod transaction;
pub mod utils;

// Re-export commonly used types
pub use block::{Block, BlockHeader};
pub use crypto::{CCKeypair, CCPublicKey, CCSignature, Hash};
pub use error::{CCError, Result};
pub use state::StateManager;
pub use transaction::Transaction;
