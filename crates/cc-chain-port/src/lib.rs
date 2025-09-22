//! CC Chain Port - Networking and cross-chain bridge functionality
//!
//! This crate handles all network-related functionality:
//! - Peer-to-peer networking
//! - Cross-chain bridge functionality

pub mod bridge;
pub mod network;

// Re-export main networking types
pub use bridge::CrossChainBridge;
pub use network::NetworkManager;
