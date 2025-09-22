// Enhanced consensus modules
pub mod safety;
pub mod ccbft;

// Re-export key types
pub use ccbft::{CcBftConsensus, CcBftConfig};
pub use safety::{SafetySystem, SafetyConfig};
