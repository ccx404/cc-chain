//! Contract Executor - Orchestrates contract deployment and execution
//!
//! This module coordinates between the WASM runtime, gas meter, and storage
//! to provide a complete contract execution environment.

use crate::vm::{
    contract::{Contract, ContractMetadata},
    gas::{GasMeter, GasOperation},
    runtime::{ExecutionContext, ExecutionResult, WasmRuntime},
    storage::ContractStorage,
    VMConfig,
};
use crate::{CCError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Contract executor that manages the complete execution lifecycle
#[derive(Debug)]
pub struct ContractExecutor {
    /// VM configuration
    config: VMConfig,

    /// Deployed contracts registry
    contracts: HashMap<String, Contract>,

    /// Execution statistics
    stats: ExecutorStats,

    /// Next contract address counter
    next_address_id: u64,
}

/// Execution statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct ExecutorStats {
    /// Total deployments
    pub deployments: u64,

    /// Successful deployments
    pub successful_deployments: u64,

    /// Total function calls
    pub function_calls: u64,

    /// Successful function calls
    pub successful_calls: u64,

    /// Total gas consumed across all executions
    pub total_gas_consumed: u64,

    /// Average gas per deployment
    pub avg_deployment_gas: u64,

    /// Average gas per call
    pub avg_call_gas: u64,
}

/// Contract deployment transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentTransaction {
    /// Contract bytecode
    pub code: Vec<u8>,

    /// Constructor arguments
    pub constructor_args: Vec<u8>,

    /// Initial contract balance
    pub initial_balance: u64,

    /// Gas limit for deployment
    pub gas_limit: u64,

    /// Deployer address
    pub deployer: String,

    /// Contract metadata
    pub metadata: ContractMetadata,
}

/// Contract call transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallTransaction {
    /// Target contract address
    pub contract_address: String,

    /// Function to call
    pub function_name: String,

    /// Function arguments
    pub args: Vec<u8>,

    /// Gas limit for the call
    pub gas_limit: u64,

    /// Caller address
    pub caller: String,

    /// Value to transfer
    pub value: u64,
}

/// Deployment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentResult {
    /// Deployed contract address
    pub contract_address: String,

    /// Gas consumed during deployment
    pub gas_used: u64,

    /// Whether deployment was successful
    pub success: bool,

    /// Error message if deployment failed
    pub error: Option<String>,

    /// Initialization result from constructor
    pub initialization_result: Option<ExecutionResult>,
}

impl ContractExecutor {
    /// Create a new contract executor
    pub fn new(config: VMConfig) -> Self {
        Self {
            config,
            contracts: HashMap::new(),
            stats: ExecutorStats::default(),
            next_address_id: 1,
        }
    }

    /// Deploy a new smart contract
    pub fn deploy_contract(
        &mut self,
        runtime: &mut WasmRuntime,
        gas_meter: &mut GasMeter,
        storage: &mut ContractStorage,
        code: Vec<u8>,
        constructor_args: Vec<u8>,
        gas_limit: u64,
    ) -> Result<Contract> {
        let initial_gas = gas_meter.consumed();

        // Generate contract address
        let contract_address = self.generate_contract_address();

        // Consume gas for deployment
        gas_meter.consume_gas(GasOperation::Create, 1)?;

        // Validate bytecode size
        if code.len() > self.config.max_code_size {
            return Err(CCError::InvalidInput("Contract code too large".to_string()));
        }

        // Load the module into runtime
        let module_hash = runtime.load_module(code.clone())?;

        // Create contract instance
        let metadata = ContractMetadata::default();
        let mut contract = Contract::new(contract_address.clone(), code, metadata)?;

        // Execute constructor if args provided
        let _initialization_result = if !constructor_args.is_empty() {
            let context = ExecutionContext {
                contract_address: contract_address.clone(),
                function_name: "init".to_string(),
                args: constructor_args,
                gas_limit,
                caller: None,
                value: 0,
                timestamp: self.current_timestamp(),
                block_number: 0, // Would be provided by blockchain state
            };

            let result = runtime.execute(
                &module_hash,
                "init",
                context.args.clone(),
                gas_meter,
                context,
            )?;

            // Apply state changes to storage
            for state_change in &result.state_changes {
                if let Some(ref new_value) = state_change.new_value {
                    storage.set(
                        &state_change.contract,
                        state_change.key.clone(),
                        new_value.clone(),
                    )?;
                    contract.update_state(state_change.key.clone(), new_value.clone())?;
                } else {
                    storage.delete(&state_change.contract, &state_change.key)?;
                    contract.remove_state(&state_change.key);
                }
            }

            if !result.success {
                return Err(CCError::ContractExecutionFailed(
                    result
                        .error
                        .unwrap_or_else(|| "Constructor failed".to_string()),
                ));
            }

            Some(result)
        } else {
            None
        };

        // Store contract
        self.contracts
            .insert(contract_address.clone(), contract.clone());

        // Update statistics
        let gas_used = gas_meter.consumed() - initial_gas;
        self.update_deployment_stats(true, gas_used);

        Ok(contract)
    }

    /// Call a function on a deployed contract
    pub fn call_contract(
        &mut self,
        runtime: &mut WasmRuntime,
        gas_meter: &mut GasMeter,
        storage: &mut ContractStorage,
        contract_address: &str,
        function_name: &str,
        args: Vec<u8>,
        gas_limit: u64,
    ) -> Result<Vec<u8>> {
        let initial_gas = gas_meter.consumed();

        // Get contract
        let contract = self.contracts.get(contract_address).ok_or_else(|| {
            CCError::InvalidInput(format!("Contract not found: {}", contract_address))
        })?;

        // Consume base call gas
        gas_meter.consume_gas(GasOperation::Call, 1)?;

        // Load module
        let module_hash = runtime.load_module(contract.code.bytecode.clone())?;

        // Create execution context
        let context = ExecutionContext {
            contract_address: contract_address.to_string(),
            function_name: function_name.to_string(),
            args,
            gas_limit,
            caller: Some("caller_address".to_string()), // Would be provided by transaction
            value: 0,
            timestamp: self.current_timestamp(),
            block_number: 0, // Would be provided by blockchain state
        };

        // Execute function
        let result = runtime.execute(
            &module_hash,
            function_name,
            context.args.clone(),
            gas_meter,
            context,
        )?;

        // Apply state changes
        if result.success {
            for state_change in &result.state_changes {
                if let Some(ref new_value) = state_change.new_value {
                    storage.set(
                        &state_change.contract,
                        state_change.key.clone(),
                        new_value.clone(),
                    )?;
                } else {
                    storage.delete(&state_change.contract, &state_change.key)?;
                }
            }

            // Update contract state
            if let Some(contract) = self.contracts.get_mut(contract_address) {
                contract.update_execution_time();
                contract.increment_nonce();

                for state_change in &result.state_changes {
                    if let Some(ref new_value) = state_change.new_value {
                        contract.update_state(state_change.key.clone(), new_value.clone())?;
                    } else {
                        contract.remove_state(&state_change.key);
                    }
                }
            }
        }

        // Update statistics
        let gas_used = gas_meter.consumed() - initial_gas;
        self.update_call_stats(result.success, gas_used);

        if result.success {
            Ok(result.return_value)
        } else {
            Err(CCError::ContractExecutionFailed(
                result
                    .error
                    .unwrap_or_else(|| "Function call failed".to_string()),
            ))
        }
    }

    /// Get contract by address
    pub fn get_contract(&self, address: &str) -> Option<&Contract> {
        self.contracts.get(address)
    }

    /// Get all deployed contracts
    pub fn list_contracts(&self) -> Vec<&Contract> {
        self.contracts.values().collect()
    }

    /// Get executor statistics
    pub fn stats(&self) -> &ExecutorStats {
        &self.stats
    }

    /// Check if contract exists
    pub fn contract_exists(&self, address: &str) -> bool {
        self.contracts.contains_key(address)
    }

    /// Estimate gas for contract deployment
    pub fn estimate_deployment_gas(&self, code: &[u8], constructor_args: &[u8]) -> u64 {
        let base_cost = self.config.deployment_gas_cost;
        let code_cost = (code.len() as u64) * 68; // Gas per byte
        let args_cost = (constructor_args.len() as u64) * 16; // Gas per arg byte

        base_cost + code_cost + args_cost
    }

    /// Estimate gas for contract call
    pub fn estimate_call_gas(
        &self,
        contract_address: &str,
        function_name: &str,
        args: &[u8],
    ) -> u64 {
        if !self.contract_exists(contract_address) {
            return 0;
        }

        let base_cost = 21000; // Base call cost
        let args_cost = (args.len() as u64) * 16; // Gas per arg byte

        // Add function-specific costs
        let function_cost = match function_name {
            "init" => 50000, // Constructor calls are expensive
            name if name.starts_with("query") || name.starts_with("get") => 5000, // Read operations
            _ => 25000,      // Regular function calls
        };

        base_cost + function_cost + args_cost
    }

    /// Generate a new contract address
    fn generate_contract_address(&mut self) -> String {
        let address = format!("contract_{:08x}", self.next_address_id);
        self.next_address_id += 1;
        address
    }

    /// Get current timestamp
    fn current_timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Update deployment statistics
    fn update_deployment_stats(&mut self, success: bool, gas_used: u64) {
        self.stats.deployments += 1;

        if success {
            self.stats.successful_deployments += 1;
        }

        self.stats.total_gas_consumed += gas_used;

        // Update average deployment gas
        if self.stats.successful_deployments > 0 {
            self.stats.avg_deployment_gas =
                self.stats.total_gas_consumed / self.stats.successful_deployments;
        }
    }

    /// Update call statistics
    fn update_call_stats(&mut self, success: bool, gas_used: u64) {
        self.stats.function_calls += 1;

        if success {
            self.stats.successful_calls += 1;
        }

        self.stats.total_gas_consumed += gas_used;

        // Update average call gas
        if self.stats.successful_calls > 0 {
            self.stats.avg_call_gas = self.stats.total_gas_consumed / self.stats.successful_calls;
        }
    }
}

/// Utility functions for contract execution
pub mod utils {
    use super::*;

    /// Create a deployment transaction
    pub fn create_deployment_tx(
        code: Vec<u8>,
        constructor_args: Vec<u8>,
        gas_limit: u64,
        deployer: String,
    ) -> DeploymentTransaction {
        DeploymentTransaction {
            code,
            constructor_args,
            initial_balance: 0,
            gas_limit,
            deployer,
            metadata: ContractMetadata::default(),
        }
    }

    /// Create a call transaction
    pub fn create_call_tx(
        contract_address: String,
        function_name: String,
        args: Vec<u8>,
        gas_limit: u64,
        caller: String,
        value: u64,
    ) -> CallTransaction {
        CallTransaction {
            contract_address,
            function_name,
            args,
            gas_limit,
            caller,
            value,
        }
    }

    /// Validate contract address format
    pub fn validate_contract_address(address: &str) -> bool {
        address.starts_with("contract_") && address.len() == 17
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::VMConfig;

    #[test]
    fn test_executor_creation() {
        let config = VMConfig::default();
        let executor = ContractExecutor::new(config);

        assert_eq!(executor.stats().deployments, 0);
        assert_eq!(executor.stats().function_calls, 0);
        assert_eq!(executor.contracts.len(), 0);
    }

    #[test]
    fn test_contract_address_generation() {
        let config = VMConfig::default();
        let mut executor = ContractExecutor::new(config);

        let addr1 = executor.generate_contract_address();
        let addr2 = executor.generate_contract_address();

        assert_ne!(addr1, addr2);
        assert!(addr1.starts_with("contract_"));
        assert!(addr2.starts_with("contract_"));
    }

    #[test]
    fn test_gas_estimation() {
        let config = VMConfig::default();
        let executor = ContractExecutor::new(config.clone());

        let code = vec![0u8; 1000];
        let args = vec![0u8; 100];

        let deployment_gas = executor.estimate_deployment_gas(&code, &args);
        assert!(deployment_gas > config.deployment_gas_cost);

        let call_gas = executor.estimate_call_gas("nonexistent", "test", &args);
        assert_eq!(call_gas, 0); // Contract doesn't exist
    }

    #[test]
    fn test_contract_existence() {
        let config = VMConfig::default();
        let executor = ContractExecutor::new(config);

        assert!(!executor.contract_exists("contract_00000001"));
        assert_eq!(executor.get_contract("contract_00000001"), None);
        assert_eq!(executor.list_contracts().len(), 0);
    }

    #[test]
    fn test_deployment_transaction() {
        let tx = utils::create_deployment_tx(
            vec![0u8; 100],
            vec![1u8; 50],
            1_000_000,
            "deployer_address".to_string(),
        );

        assert_eq!(tx.code.len(), 100);
        assert_eq!(tx.constructor_args.len(), 50);
        assert_eq!(tx.gas_limit, 1_000_000);
        assert_eq!(tx.deployer, "deployer_address");
    }

    #[test]
    fn test_call_transaction() {
        let tx = utils::create_call_tx(
            "contract_00000001".to_string(),
            "test_function".to_string(),
            vec![2u8; 32],
            500_000,
            "caller_address".to_string(),
            1000,
        );

        assert_eq!(tx.contract_address, "contract_00000001");
        assert_eq!(tx.function_name, "test_function");
        assert_eq!(tx.args.len(), 32);
        assert_eq!(tx.gas_limit, 500_000);
        assert_eq!(tx.value, 1000);
    }

    #[test]
    fn test_address_validation() {
        assert!(utils::validate_contract_address("contract_00000001"));
        assert!(utils::validate_contract_address("contract_12345678"));
        assert!(!utils::validate_contract_address("invalid_address"));
        assert!(!utils::validate_contract_address("contract_"));
        assert!(!utils::validate_contract_address("contract_toolong123"));
    }
}
