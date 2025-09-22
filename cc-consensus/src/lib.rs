//! CC Chain Consensus Algorithms
//!
//! This crate contains consensus algorithms and safety systems for the CC Chain blockchain:
//! - Enhanced BFT consensus protocol
//! - ccBFT consensus for high performance
//! - Safety monitoring and fault tolerance systems

pub mod ccbft;
pub mod safety;

// Re-export commonly used modules from mod.rs
mod consensus_types;
pub use consensus_types::*;

// Re-export key types
pub use ccbft::{CcBftConsensus, CcBftConfig};
pub use safety::{SafetySystem, SafetyConfig};