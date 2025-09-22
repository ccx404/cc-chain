//! CLI and node management
//!
//! This module contains node functionality and command-line interfaces

pub mod node;

// Re-export node types
pub use node::{CCNode, NodeConfig, NodeType};
