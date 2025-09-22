//! CC Chain Smart Contracts and Virtual Machine
//!
//! This crate contains the virtual machine and smart contract execution environment:
//! - WASM-based virtual machine
//! - Contract deployment and execution
//! - Inter-contract communication
//! - Gas metering and resource management

pub mod vm;

// Re-export VM types
pub use vm::{SmartContractVM, VMConfig};