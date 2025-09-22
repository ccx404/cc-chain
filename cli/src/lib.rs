//! CC Chain CLI and Node Management
//!
//! This crate contains node functionality and command-line interfaces:
//! - Node startup and management
//! - CLI commands and tools
//! - Configuration management

pub mod node;

// Re-export node types
pub use node::{CCNode, NodeConfig, NodeType};