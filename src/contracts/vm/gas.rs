//! Gas Metering System for Smart Contracts
//!
//! This module provides gas metering and billing for contract execution
//! to prevent infinite loops and resource exhaustion attacks.

use crate::{CCError, Result};
use serde::{Deserialize, Serialize};

/// Gas counter for tracking consumption during execution
#[derive(Debug, Clone)]
pub struct GasCounter {
    /// Total gas consumed
    consumed: u64,

    /// Gas limit for this execution
    limit: u64,

    /// Gas cost schedule
    schedule: GasSchedule,
}

/// Gas metering system
#[derive(Debug)]
pub struct GasMeter {
    /// Current gas counter
    counter: GasCounter,

    /// Gas price in native tokens
    gas_price: u64,

    /// Execution metrics
    metrics: ExecutionMetrics,
}

/// Gas cost schedule for different operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasSchedule {
    /// Base cost for any instruction
    pub base: u64,

    /// Memory operations
    pub memory_read: u64,
    pub memory_write: u64,
    pub memory_grow: u64,

    /// Storage operations
    pub storage_read: u64,
    pub storage_write: u64,
    pub storage_delete: u64,

    /// Crypto operations
    pub hash_blake3: u64,
    pub signature_verify: u64,

    /// Call operations
    pub call_base: u64,
    pub call_value_transfer: u64,
    pub contract_creation: u64,

    /// System operations
    pub log_base: u64,
    pub log_per_byte: u64,
    pub copy_per_byte: u64,
}

/// Execution metrics for performance analysis
#[derive(Debug, Clone, Default)]
pub struct ExecutionMetrics {
    /// Instructions executed
    pub instructions_executed: u64,

    /// Memory operations count
    pub memory_operations: u64,

    /// Storage operations count
    pub storage_operations: u64,

    /// External calls made
    pub external_calls: u64,

    /// Execution time in microseconds
    pub execution_time_us: u64,
}

/// Gas operation categories for detailed tracking
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GasOperation {
    /// Basic instruction execution
    Instruction,

    /// Memory read operation
    MemoryRead,

    /// Memory write operation
    MemoryWrite,

    /// Memory expansion
    MemoryGrow,

    /// Storage read operation
    StorageRead,

    /// Storage write operation
    StorageWrite,

    /// Storage deletion
    StorageDelete,

    /// Cryptographic hash operation
    Hash,

    /// Signature verification
    SignatureVerify,

    /// Contract call
    Call,

    /// Value transfer
    Transfer,

    /// Contract creation
    Create,

    /// Event logging
    Log,

    /// Data copying
    Copy,
}

impl Default for GasSchedule {
    fn default() -> Self {
        Self {
            base: 1,
            memory_read: 3,
            memory_write: 3,
            memory_grow: 1,
            storage_read: 200,
            storage_write: 5000,
            storage_delete: 5000,
            hash_blake3: 30,
            signature_verify: 3000,
            call_base: 40,
            call_value_transfer: 2300,
            contract_creation: 32000,
            log_base: 375,
            log_per_byte: 8,
            copy_per_byte: 3,
        }
    }
}

impl GasCounter {
    /// Create a new gas counter with given limit
    pub fn new(limit: u64) -> Self {
        Self {
            consumed: 0,
            limit,
            schedule: GasSchedule::default(),
        }
    }

    /// Create with custom gas schedule
    pub fn with_schedule(limit: u64, schedule: GasSchedule) -> Self {
        Self {
            consumed: 0,
            limit,
            schedule,
        }
    }

    /// Consume gas for an operation
    pub fn consume(&mut self, operation: GasOperation, units: u64) -> Result<()> {
        let cost = self.calculate_cost(&operation, units);

        if self.consumed + cost > self.limit {
            return Err(CCError::OutOfGas {
                required: self.consumed + cost,
                available: self.limit,
            });
        }

        self.consumed += cost;
        Ok(())
    }

    /// Get remaining gas
    pub fn remaining(&self) -> u64 {
        self.limit.saturating_sub(self.consumed)
    }

    /// Get consumed gas
    pub fn consumed(&self) -> u64 {
        self.consumed
    }

    /// Get gas limit
    pub fn limit(&self) -> u64 {
        self.limit
    }

    /// Check if enough gas is available
    pub fn has_gas(&self, operation: GasOperation, units: u64) -> bool {
        let cost = self.calculate_cost(&operation, units);
        self.consumed + cost <= self.limit
    }

    /// Calculate cost for an operation
    fn calculate_cost(&self, operation: &GasOperation, units: u64) -> u64 {
        let base_cost = match operation {
            GasOperation::Instruction => self.schedule.base,
            GasOperation::MemoryRead => self.schedule.memory_read,
            GasOperation::MemoryWrite => self.schedule.memory_write,
            GasOperation::MemoryGrow => self.schedule.memory_grow,
            GasOperation::StorageRead => self.schedule.storage_read,
            GasOperation::StorageWrite => self.schedule.storage_write,
            GasOperation::StorageDelete => self.schedule.storage_delete,
            GasOperation::Hash => self.schedule.hash_blake3,
            GasOperation::SignatureVerify => self.schedule.signature_verify,
            GasOperation::Call => self.schedule.call_base,
            GasOperation::Transfer => self.schedule.call_value_transfer,
            GasOperation::Create => self.schedule.contract_creation,
            GasOperation::Log => self.schedule.log_base,
            GasOperation::Copy => self.schedule.copy_per_byte,
        };

        base_cost * units
    }

    /// Consume a specific amount of gas directly
    pub fn consume_amount(&mut self, amount: u64) -> Result<()> {
        if self.consumed + amount > self.limit {
            return Err(CCError::OutOfGas {
                required: self.consumed + amount,
                available: self.limit,
            });
        }

        self.consumed += amount;
        Ok(())
    }

    /// Refund gas (for failed operations)
    pub fn refund(&mut self, amount: u64) {
        let refund_amount = amount.min(self.consumed);
        self.consumed = self.consumed.saturating_sub(refund_amount);
    }
}

impl GasMeter {
    /// Create a new gas meter
    pub fn new(gas_limit: u64) -> Self {
        Self {
            counter: GasCounter::new(gas_limit),
            gas_price: 1, // Default gas price
            metrics: ExecutionMetrics::default(),
        }
    }

    /// Create with custom gas price
    pub fn with_price(gas_limit: u64, gas_price: u64) -> Self {
        Self {
            counter: GasCounter::new(gas_limit),
            gas_price,
            metrics: ExecutionMetrics::default(),
        }
    }

    /// Consume gas and update metrics
    pub fn consume_gas(&mut self, operation: GasOperation, units: u64) -> Result<()> {
        self.counter.consume(operation.clone(), units)?;
        self.update_metrics(&operation, units);
        Ok(())
    }

    /// Get remaining gas
    pub fn remaining(&self) -> u64 {
        self.counter.remaining()
    }

    /// Get consumed gas
    pub fn consumed(&self) -> u64 {
        self.counter.consumed()
    }

    /// Calculate total cost in native tokens
    pub fn total_cost(&self) -> u64 {
        self.counter.consumed() * self.gas_price
    }

    /// Get execution metrics
    pub fn metrics(&self) -> &ExecutionMetrics {
        &self.metrics
    }

    /// Reset the meter for new execution
    pub fn reset(&mut self, new_limit: u64) {
        self.counter = GasCounter::new(new_limit);
        self.metrics = ExecutionMetrics::default();
    }

    /// Set gas price
    pub fn set_gas_price(&mut self, price: u64) {
        self.gas_price = price;
    }

    /// Check if enough gas is available
    pub fn has_gas(&self, operation: GasOperation, units: u64) -> bool {
        self.counter.has_gas(operation, units)
    }

    /// Consume a specific amount of gas
    pub fn consume_gas_amount(&mut self, amount: u64) -> Result<()> {
        self.counter.consume_amount(amount)
    }

    /// Refund unused gas
    pub fn refund_gas(&mut self, amount: u64) {
        self.counter.refund(amount);
    }

    /// Update execution metrics
    fn update_metrics(&mut self, operation: &GasOperation, units: u64) {
        match operation {
            GasOperation::Instruction => {
                self.metrics.instructions_executed += units;
            }
            GasOperation::MemoryRead | GasOperation::MemoryWrite | GasOperation::MemoryGrow => {
                self.metrics.memory_operations += units;
            }
            GasOperation::StorageRead
            | GasOperation::StorageWrite
            | GasOperation::StorageDelete => {
                self.metrics.storage_operations += units;
            }
            GasOperation::Call | GasOperation::Transfer | GasOperation::Create => {
                self.metrics.external_calls += units;
            }
            _ => {}
        }
    }
}

/// Utility functions for gas estimation
pub fn estimate_gas_for_bytecode(bytecode: &[u8]) -> u64 {
    // Simple estimation based on bytecode size
    // In a real implementation, this would analyze the WASM instructions
    let base_cost = 21000; // Base execution cost
    let per_byte_cost = 68; // Cost per byte of code

    base_cost + (bytecode.len() as u64 * per_byte_cost)
}

/// Estimate gas for storage operations
pub fn estimate_storage_gas(key_size: usize, value_size: usize, is_new_key: bool) -> u64 {
    let schedule = GasSchedule::default();
    let mut cost = schedule.storage_write;

    if is_new_key {
        // Additional cost for new storage slot
        cost += 15000;
    }

    // Add cost based on data size
    cost += (key_size + value_size) as u64 * 68;

    cost
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_counter_basic() {
        let mut counter = GasCounter::new(1000);

        assert!(counter.consume(GasOperation::Instruction, 10).is_ok());
        assert_eq!(counter.consumed(), 10);
        assert_eq!(counter.remaining(), 990);
    }

    #[test]
    fn test_gas_counter_out_of_gas() {
        let mut counter = GasCounter::new(100);

        // This should exceed the limit
        let result = counter.consume(GasOperation::StorageWrite, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_gas_meter() {
        let mut meter = GasMeter::new(10000);

        assert!(meter.consume_gas(GasOperation::Instruction, 100).is_ok());
        assert!(meter.consume_gas(GasOperation::MemoryRead, 10).is_ok());

        assert!(meter.consumed() > 0);
        assert!(meter.metrics().instructions_executed == 100);
        assert!(meter.metrics().memory_operations == 10);
    }

    #[test]
    fn test_gas_estimation() {
        let bytecode = vec![0u8; 1000];
        let estimated = estimate_gas_for_bytecode(&bytecode);
        assert!(estimated > 21000);

        let storage_gas = estimate_storage_gas(32, 64, true);
        assert!(storage_gas > 0);
    }

    #[test]
    fn test_gas_schedule() {
        let schedule = GasSchedule::default();
        assert!(schedule.base > 0);
        assert!(schedule.storage_write > schedule.storage_read);
        assert!(schedule.signature_verify > schedule.hash_blake3);
    }
}
