//! CC Chain REST API Server
//! 
//! This module provides a comprehensive REST API for interacting with the CC Chain blockchain.
//! It includes endpoints for transactions, blocks, accounts, and network information.

pub mod server;
pub mod models;
pub mod error;

// Re-export important types
pub use server::ApiServer;
pub use models::*;
pub use error::ApiError;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }
}
