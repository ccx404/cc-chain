//! CC Chain Cross-Chain Bridge Implementation
//! 
//! This module provides comprehensive cross-chain bridge functionality for connecting
//! CC Chain with other blockchain networks like Ethereum, Bitcoin, and others.

pub mod bridge;
pub mod chains;
pub mod messages;
pub mod validation;
pub mod recovery;
pub mod monitoring;

// Re-export important types
pub use bridge::{CrossChainBridge, BridgeConfig, BridgeStats};
pub use chains::{SupportedChain, ChainConfig};
pub use messages::{BridgeMessage, MessageType};
pub use validation::BridgeValidator;

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
