//! Storage layer
//!
//! This module handles storage-related functionality:
//! - Transaction mempool
//! - State storage and caching

pub mod mempool;

// Re-export storage types
pub use mempool::Mempool;
