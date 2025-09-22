//! CC Chain Storage Layer
//!
//! This crate handles storage-related functionality:
//! - Transaction mempool
//! - State storage and caching
//! - Persistent storage management

pub mod mempool;

// Re-export storage types
pub use mempool::{Mempool, MempoolStats};