//! Smart contracts and virtual machine
//!
//! This module contains the virtual machine and smart contract execution environment

pub mod vm;

// Re-export VM types
pub use vm::{SmartContractVM, VMConfig};
