//! CC Chain Storage - Memory pool and storage management
//!
//! This crate handles storage-related functionality:
//! - Transaction mempool
//! - State storage and caching

pub mod mempool;

// Re-export storage types
pub use mempool::{Mempool, MempoolStats, TransactionPool};
