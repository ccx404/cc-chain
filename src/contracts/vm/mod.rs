//! Smart Contract Virtual Machine Module
//!
//! This module provides the execution environment for smart contracts,
//! including WASM runtime, gas metering, and contract lifecycle management.

pub mod contract;
pub mod events;
pub mod executor;
pub mod gas;
pub mod interop;
pub mod runtime;
pub mod storage;

pub use contract::{Contract, ContractCode, ContractState};
pub use events::{ContractEvent, EventFilter, EventManager};
pub use executor::ContractExecutor;
pub use gas::{GasCounter, GasMeter};
pub use interop::{CallContext, InterContractCall, InterContractManager};
pub use runtime::WasmRuntime;
pub use storage::ContractStorage;

use crate::Result;
use serde::{Deserialize, Serialize};

/// Virtual Machine for executing smart contracts
#[derive(Debug)]
pub struct SmartContractVM {
    /// WASM runtime instance
    runtime: WasmRuntime,

    /// Gas metering system
    gas_meter: GasMeter,

    /// Contract storage layer
    storage: ContractStorage,

    /// Contract executor
    executor: ContractExecutor,

    /// Event management system
    event_manager: EventManager,

    /// Inter-contract communication manager
    interop_manager: InterContractManager,
}

/// Configuration for the Smart Contract VM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VMConfig {
    /// Maximum gas limit per contract execution
    pub max_gas_limit: u64,

    /// Base gas cost for contract deployment
    pub deployment_gas_cost: u64,

    /// Gas cost per byte of storage
    pub storage_gas_cost: u64,

    /// Maximum contract code size in bytes
    pub max_code_size: usize,

    /// Maximum stack depth for contract calls
    pub max_call_depth: usize,

    /// Enable debugging features
    pub debug_mode: bool,
}

impl Default for VMConfig {
    fn default() -> Self {
        Self {
            max_gas_limit: 10_000_000,
            deployment_gas_cost: 50_000,
            storage_gas_cost: 100,
            max_code_size: 1024 * 1024, // 1MB
            max_call_depth: 1024,
            debug_mode: false,
        }
    }
}

impl SmartContractVM {
    /// Create a new Smart Contract VM instance
    pub fn new(config: VMConfig) -> Result<Self> {
        let runtime = WasmRuntime::new(&config)?;
        let gas_meter = GasMeter::new(config.max_gas_limit);
        let storage = ContractStorage::new();
        let executor = ContractExecutor::new(config.clone());
        let event_manager = EventManager::new();
        let interop_manager = InterContractManager::new(config.max_call_depth);

        Ok(Self {
            runtime,
            gas_meter,
            storage,
            executor,
            event_manager,
            interop_manager,
        })
    }

    /// Deploy a new smart contract
    pub fn deploy_contract(
        &mut self,
        code: Vec<u8>,
        constructor_args: Vec<u8>,
        gas_limit: u64,
    ) -> Result<Contract> {
        self.executor.deploy_contract(
            &mut self.runtime,
            &mut self.gas_meter,
            &mut self.storage,
            code,
            constructor_args,
            gas_limit,
        )
    }

    /// Execute a contract function call
    pub fn call_contract(
        &mut self,
        contract_address: &str,
        function_name: &str,
        args: Vec<u8>,
        gas_limit: u64,
    ) -> Result<Vec<u8>> {
        self.executor.call_contract(
            &mut self.runtime,
            &mut self.gas_meter,
            &mut self.storage,
            contract_address,
            function_name,
            args,
            gas_limit,
        )
    }

    /// Execute an inter-contract call
    pub fn inter_contract_call(
        &mut self,
        call: InterContractCall,
        caller: String,
        origin: String,
    ) -> Result<crate::vm::interop::InterContractResult> {
        let context = CallContext {
            origin,
            caller: caller.clone(),
            current_contract: call.target_contract.clone(),
            gas_limit: call.gas_limit,
            depth: 0,
        };

        self.interop_manager.call_contract(
            &mut self.runtime,
            &mut self.gas_meter,
            &mut self.storage,
            call,
            context,
        )
    }

    /// Emit an event from a contract
    pub fn emit_event(
        &mut self,
        contract_address: String,
        event_name: String,
        topics: Vec<Vec<u8>>,
        data: Vec<u8>,
        block_number: u64,
        transaction_hash: String,
    ) -> Result<()> {
        self.event_manager.emit_event(
            contract_address,
            event_name,
            topics,
            data,
            block_number,
            transaction_hash,
            0, // log_index would be managed by the blockchain
        )
    }

    /// Query events with filter
    pub fn query_events(
        &self,
        filter: &EventFilter,
    ) -> Result<crate::vm::events::EventQueryResult> {
        self.event_manager.query_events(filter)
    }

    /// Register event listener
    pub fn register_event_listener(
        &mut self,
        contract_address: String,
        listener: crate::vm::events::EventListener,
    ) -> Result<()> {
        self.event_manager
            .register_listener(contract_address, listener)
    }

    /// Get contract storage value
    pub fn get_storage(&self, contract_address: &str, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.storage.get(contract_address, key)
    }

    /// Get remaining gas after execution
    pub fn remaining_gas(&self) -> u64 {
        self.gas_meter.remaining()
    }

    /// Get inter-contract call statistics
    pub fn get_interop_stats(&self) -> &crate::vm::interop::InterContractStats {
        self.interop_manager.get_stats()
    }

    /// Get event count for a contract
    pub fn get_event_count(&self, contract_address: Option<&str>) -> usize {
        self.event_manager.get_event_count(contract_address)
    }
}

