//! Inter-Contract Communication System
//!
//! This module provides capabilities for contracts to call other contracts.

use crate::vm::{
    gas::{GasMeter, GasOperation},
    runtime::{ExecutionContext, ExecutionResult},
    storage::ContractStorage,
};
use crate::{CCError, Result};
use serde::{Deserialize, Serialize};

/// Inter-contract call manager
#[derive(Debug)]
pub struct InterContractManager {
    /// Maximum call depth to prevent infinite recursion
    max_call_depth: usize,

    /// Current call stack
    call_stack: Vec<CallFrame>,

    /// Inter-contract call statistics
    stats: InterContractStats,
}

/// Represents a frame in the contract call stack
#[derive(Debug, Clone)]
pub struct CallFrame {
    /// Contract address
    pub contract_address: String,

    /// Function being called
    pub function_name: String,

    /// Caller address
    pub caller: String,

    /// Gas allocated for this call
    pub gas_limit: u64,

    /// Value transferred
    pub value: u64,

    /// Call depth
    pub depth: usize,
}

/// Statistics for inter-contract calls
#[derive(Debug, Clone, Default)]
pub struct InterContractStats {
    /// Total inter-contract calls
    pub total_calls: u64,

    /// Successful calls
    pub successful_calls: u64,

    /// Failed calls
    pub failed_calls: u64,

    /// Maximum call depth reached
    pub max_depth_reached: usize,

    /// Total gas consumed in inter-contract calls
    pub total_gas_consumed: u64,
}

/// Inter-contract call request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterContractCall {
    /// Target contract address
    pub target_contract: String,

    /// Function to call
    pub function_name: String,

    /// Function arguments
    pub args: Vec<u8>,

    /// Gas limit for the call
    pub gas_limit: u64,

    /// Value to transfer
    pub value: u64,

    /// Whether the call should revert on failure
    pub revert_on_failure: bool,
}

/// Result of an inter-contract call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterContractResult {
    /// Return value from the called function
    pub return_value: Vec<u8>,

    /// Gas consumed by the call
    pub gas_used: u64,

    /// Whether the call was successful
    pub success: bool,

    /// Error message if call failed
    pub error: Option<String>,

    /// Events emitted during the call
    pub events: Vec<crate::vm::events::ContractEvent>,

    /// State changes made during the call
    pub state_changes: Vec<crate::vm::runtime::StateChange>,
}

/// Call context for inter-contract calls
#[derive(Debug, Clone)]
pub struct CallContext {
    /// Original caller (msg.sender in the first contract)
    pub origin: String,

    /// Current caller (immediate caller)
    pub caller: String,

    /// Current contract being executed
    pub current_contract: String,

    /// Total gas available
    pub gas_limit: u64,

    /// Current call depth
    pub depth: usize,
}

impl InterContractManager {
    /// Create a new inter-contract manager
    pub fn new(max_call_depth: usize) -> Self {
        Self {
            max_call_depth,
            call_stack: Vec::new(),
            stats: InterContractStats::default(),
        }
    }

    /// Execute an inter-contract call
    pub fn call_contract(
        &mut self,
        runtime: &mut crate::vm::runtime::WasmRuntime,
        gas_meter: &mut GasMeter,
        storage: &mut ContractStorage,
        call: InterContractCall,
        context: CallContext,
    ) -> Result<InterContractResult> {
        // Check call depth limit
        if context.depth >= self.max_call_depth {
            return Err(CCError::ContractExecutionFailed(
                "Maximum call depth exceeded".to_string(),
            ));
        }

        // Check if target contract exists
        if !self.contract_exists(&call.target_contract, storage)? {
            return if call.revert_on_failure {
                Err(CCError::ContractExecutionFailed(
                    "Target contract does not exist".to_string(),
                ))
            } else {
                Ok(InterContractResult {
                    return_value: vec![],
                    gas_used: 0,
                    success: false,
                    error: Some("Target contract does not exist".to_string()),
                    events: vec![],
                    state_changes: vec![],
                })
            };
        }

        // Consume gas for inter-contract call
        gas_meter.consume_gas(GasOperation::Call, 1)?;

        // Reserve gas for the call
        let available_gas = gas_meter.remaining().min(call.gas_limit);
        gas_meter.consume_gas_amount(available_gas)?;

        // Create call frame
        let call_frame = CallFrame {
            contract_address: call.target_contract.clone(),
            function_name: call.function_name.clone(),
            caller: context.caller.clone(),
            gas_limit: available_gas,
            value: call.value,
            depth: context.depth + 1,
        };

        // Push to call stack
        self.call_stack.push(call_frame);
        self.stats.max_depth_reached = self.stats.max_depth_reached.max(context.depth + 1);

        // Create execution context for the call
        let exec_context = ExecutionContext {
            contract_address: call.target_contract.clone(),
            function_name: call.function_name.clone(),
            args: call.args,
            gas_limit: available_gas,
            caller: Some(context.caller),
            value: call.value,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            block_number: 0, // Would be provided by blockchain state
        };

        // Execute the contract function
        let result = self.execute_with_context(
            runtime,
            gas_meter,
            storage,
            &call.target_contract,
            &call.function_name,
            exec_context.clone(),
        );

        // Pop from call stack
        self.call_stack.pop();

        // Update statistics
        self.stats.total_calls += 1;

        match result {
            Ok(exec_result) => {
                self.stats.successful_calls += 1;
                self.stats.total_gas_consumed += exec_result.gas_used;

                // Refund unused gas
                let unused_gas = available_gas.saturating_sub(exec_result.gas_used);
                gas_meter.refund_gas(unused_gas);

                Ok(InterContractResult {
                    return_value: exec_result.return_value,
                    gas_used: exec_result.gas_used,
                    success: exec_result.success,
                    error: exec_result.error,
                    events: exec_result
                        .logs
                        .into_iter()
                        .map(|log| {
                            crate::vm::events::ContractEvent {
                                contract_address: log.contract,
                                event_name: "Log".to_string(), // Simplified
                                topics: log.topics,
                                data: log.data,
                                block_number: 0,
                                transaction_hash: "".to_string(),
                                log_index: 0,
                                timestamp: exec_context.timestamp,
                            }
                        })
                        .collect(),
                    state_changes: exec_result.state_changes,
                })
            }
            Err(error) => {
                self.stats.failed_calls += 1;

                // Refund all gas if call failed
                gas_meter.refund_gas(available_gas);

                if call.revert_on_failure {
                    Err(error)
                } else {
                    Ok(InterContractResult {
                        return_value: vec![],
                        gas_used: 0,
                        success: false,
                        error: Some(error.to_string()),
                        events: vec![],
                        state_changes: vec![],
                    })
                }
            }
        }
    }

    /// Check if a contract exists
    fn contract_exists(&self, contract_address: &str, storage: &ContractStorage) -> Result<bool> {
        // Check if contract has code stored
        storage
            .get(contract_address, b"__code__")
            .map(|result| result.is_some())
    }

    /// Execute contract with given context
    fn execute_with_context(
        &self,
        runtime: &mut crate::vm::runtime::WasmRuntime,
        gas_meter: &mut GasMeter,
        _storage: &mut ContractStorage,
        contract_address: &str,
        function_name: &str,
        context: ExecutionContext,
    ) -> Result<ExecutionResult> {
        // Get contract's module hash
        let module_hash = format!("{}_{}", contract_address, function_name); // Simplified

        // Execute the function
        runtime.execute(
            &module_hash,
            function_name,
            context.args.clone(),
            gas_meter,
            context,
        )
    }

    /// Get current call stack depth
    pub fn current_depth(&self) -> usize {
        self.call_stack.len()
    }

    /// Get current call frame
    pub fn current_frame(&self) -> Option<&CallFrame> {
        self.call_stack.last()
    }

    /// Get caller address for the current frame
    pub fn get_caller(&self) -> Option<String> {
        self.call_stack.last().map(|frame| frame.caller.clone())
    }

    /// Get origin address (first caller in the stack)
    pub fn get_origin(&self) -> Option<String> {
        self.call_stack.first().map(|frame| frame.caller.clone())
    }

    /// Check if we're in the middle of a call stack
    pub fn is_in_call(&self) -> bool {
        !self.call_stack.is_empty()
    }

    /// Get statistics
    pub fn get_stats(&self) -> &InterContractStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = InterContractStats::default();
    }

    /// Create a delegate call (preserves original caller context)
    pub fn delegate_call(
        &mut self,
        runtime: &mut crate::vm::runtime::WasmRuntime,
        gas_meter: &mut GasMeter,
        storage: &mut ContractStorage,
        target_contract: String,
        function_name: String,
        args: Vec<u8>,
        gas_limit: u64,
        context: CallContext,
    ) -> Result<InterContractResult> {
        // Delegate call preserves the original caller context
        let delegate_call = InterContractCall {
            target_contract,
            function_name,
            args,
            gas_limit,
            value: 0, // Delegate calls don't transfer value
            revert_on_failure: true,
        };

        // Use the original context for delegate call
        self.call_contract(runtime, gas_meter, storage, delegate_call, context)
    }
}

/// Host functions for inter-contract operations
pub struct InterContractHostFunctions;

impl InterContractHostFunctions {
    /// Call another contract from within a contract
    pub fn call(
        target: &str,
        function: &str,
        args: &[u8],
        _gas: u64,
        _value: u64,
    ) -> Result<Vec<u8>> {
        // This would be implemented as a host function callable from WASM
        // For now, returning a placeholder
        tracing::debug!(
            "Inter-contract call: {} -> {}::{}",
            target,
            function,
            hex::encode(args)
        );
        Ok(vec![])
    }

    /// Make a delegate call to another contract
    pub fn delegate_call(target: &str, function: &str, args: &[u8], _gas: u64) -> Result<Vec<u8>> {
        tracing::debug!(
            "Delegate call: {} -> {}::{}",
            target,
            function,
            hex::encode(args)
        );
        Ok(vec![])
    }

    /// Get the caller address
    pub fn msg_sender() -> String {
        // This would return the actual caller address
        "caller_address".to_string()
    }

    /// Get the origin address (transaction sender)
    pub fn tx_origin() -> String {
        // This would return the transaction origin
        "origin_address".to_string()
    }

    /// Get the current contract address
    pub fn address() -> String {
        // This would return the current contract address
        "current_contract".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::gas::GasMeter;
    use crate::vm::storage::ContractStorage;

    #[test]
    fn test_call_depth_limit() {
        let mut manager = InterContractManager::new(2);
        let mut gas_meter = GasMeter::new(1000000);
        let mut storage = ContractStorage::new();

        let call = InterContractCall {
            target_contract: "contract1".to_string(),
            function_name: "test".to_string(),
            args: vec![],
            gas_limit: 10000,
            value: 0,
            revert_on_failure: true,
        };

        let context = CallContext {
            origin: "origin".to_string(),
            caller: "caller".to_string(),
            current_contract: "current".to_string(),
            gas_limit: 100000,
            depth: 2, // At the limit
        };

        // This should fail due to depth limit
        let result = manager.call_contract(
            &mut crate::vm::runtime::WasmRuntime::new(&crate::vm::VMConfig::default()).unwrap(),
            &mut gas_meter,
            &mut storage,
            call,
            context,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_call_stack_management() {
        let mut manager = InterContractManager::new(10);

        assert_eq!(manager.current_depth(), 0);
        assert!(!manager.is_in_call());

        let frame = CallFrame {
            contract_address: "test".to_string(),
            function_name: "func".to_string(),
            caller: "caller".to_string(),
            gas_limit: 10000,
            value: 0,
            depth: 1,
        };

        manager.call_stack.push(frame);

        assert_eq!(manager.current_depth(), 1);
        assert!(manager.is_in_call());
        assert_eq!(manager.get_caller(), Some("caller".to_string()));
    }
}
