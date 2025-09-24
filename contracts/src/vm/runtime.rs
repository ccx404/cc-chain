//! WASM Runtime for Smart Contract Execution
//!
//! This module provides a WebAssembly runtime for executing smart contracts
//! in a sandboxed and deterministic environment.

use crate::vm::{
    gas::{GasMeter, GasOperation},
    VMConfig,
};
use cc_core::{CCError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// WASM runtime instance for contract execution
#[derive(Debug)]
pub struct WasmRuntime {
    /// Configuration
    config: VMConfig,

    /// Loaded modules cache
    modules: HashMap<String, WasmModule>,

    /// Runtime statistics
    stats: RuntimeStats,
}

/// Compiled WASM module
#[derive(Debug, Clone)]
pub struct WasmModule {
    /// Module bytecode
    #[allow(dead_code)]
    bytecode: Vec<u8>,

    /// Module hash for identification
    #[allow(dead_code)]
    code_hash: String,

    /// Exported functions
    exports: Vec<String>,

    /// Memory requirements
    #[allow(dead_code)]
    memory_pages: u32,

    /// Compilation timestamp
    #[allow(dead_code)]
    compiled_at: u64,
}

/// Execution context for contract calls
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Contract address being executed
    pub contract_address: String,

    /// Function being called
    pub function_name: String,

    /// Input arguments
    pub args: Vec<u8>,

    /// Gas limit for this execution
    pub gas_limit: u64,

    /// Caller address
    pub caller: Option<String>,

    /// Value transferred with the call
    pub value: u64,

    /// Block timestamp
    pub timestamp: u64,

    /// Block number
    pub block_number: u64,
}

/// Execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Return value from the function
    pub return_value: Vec<u8>,

    /// Gas consumed during execution
    pub gas_used: u64,

    /// Whether execution was successful
    pub success: bool,

    /// Error message if execution failed
    pub error: Option<String>,

    /// Logs emitted during execution
    pub logs: Vec<ContractLog>,

    /// State changes made during execution
    pub state_changes: Vec<StateChange>,
}

/// Contract log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractLog {
    /// Topics for the log (indexed parameters)
    pub topics: Vec<Vec<u8>>,

    /// Log data (non-indexed parameters)
    pub data: Vec<u8>,

    /// Contract that emitted the log
    pub contract: String,
}

/// State change during execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChange {
    /// Contract address
    pub contract: String,

    /// Storage key
    pub key: Vec<u8>,

    /// Previous value
    pub old_value: Option<Vec<u8>>,

    /// New value
    pub new_value: Option<Vec<u8>>,
}

/// Runtime statistics
#[derive(Debug, Clone, Default)]
pub struct RuntimeStats {
    /// Total executions
    pub executions: u64,

    /// Successful executions
    pub successful_executions: u64,

    /// Failed executions
    pub failed_executions: u64,

    /// Total gas consumed
    pub total_gas_used: u64,

    /// Average execution time in microseconds
    pub avg_execution_time_us: u64,

    /// Modules loaded
    pub modules_loaded: u64,

    /// Cache hits
    pub cache_hits: u64,
}

/// Host functions available to contracts
pub struct HostFunctions {
    /// Gas meter for tracking consumption
    gas_meter: *mut GasMeter,

    /// Storage interface
    storage: HashMap<Vec<u8>, Vec<u8>>,

    /// Execution context
    context: ExecutionContext,

    /// Logs buffer
    logs: Vec<ContractLog>,

    /// State changes buffer
    state_changes: Vec<StateChange>,
}

impl WasmRuntime {
    /// Create a new WASM runtime
    pub fn new(config: &VMConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
            modules: HashMap::new(),
            stats: RuntimeStats::default(),
        })
    }

    /// Load and compile a WASM module
    pub fn load_module(&mut self, bytecode: Vec<u8>) -> Result<String> {
        // Validate WASM bytecode
        self.validate_bytecode(&bytecode)?;

        // Calculate module hash
        let code_hash = hex::encode(blake3::hash(&bytecode).as_bytes());

        // Check if already loaded
        if self.modules.contains_key(&code_hash) {
            self.stats.cache_hits += 1;
            return Ok(code_hash);
        }

        // Parse WASM module (simplified)
        let module = self.parse_module(bytecode)?;

        // Store module
        self.modules.insert(code_hash.clone(), module);
        self.stats.modules_loaded += 1;

        Ok(code_hash)
    }

    /// Execute a function in a loaded module
    pub fn execute(
        &mut self,
        module_hash: &str,
        function_name: &str,
        args: Vec<u8>,
        gas_meter: &mut GasMeter,
        context: ExecutionContext,
    ) -> Result<ExecutionResult> {
        let start_time = std::time::Instant::now();

        // Get the module
        let module = self
            .modules
            .get(module_hash)
            .ok_or_else(|| CCError::InvalidInput(format!("Module not found: {}", module_hash)))?;

        // Check if function exists
        if !module.exports.contains(&function_name.to_string()) {
            return Err(CCError::InvalidInput(format!(
                "Function not found: {}",
                function_name
            )));
        }

        // Set up execution environment
        let initial_gas = gas_meter.consumed();
        let mut host_functions = HostFunctions::new(gas_meter, context);

        // Execute the function (simplified simulation)
        let result = self.simulate_execution(module, function_name, args, &mut host_functions)?;

        // Calculate execution time
        let execution_time = start_time.elapsed().as_micros() as u64;

        // Update statistics
        self.update_stats(true, gas_meter.consumed() - initial_gas, execution_time);

        Ok(result)
    }

    /// Get runtime statistics
    pub fn stats(&self) -> &RuntimeStats {
        &self.stats
    }

    /// Clear module cache
    pub fn clear_cache(&mut self) {
        self.modules.clear();
    }

    /// Get loaded modules count
    pub fn modules_count(&self) -> usize {
        self.modules.len()
    }

    /// Validate WASM bytecode
    fn validate_bytecode(&self, bytecode: &[u8]) -> Result<()> {
        // Check minimum size
        if bytecode.len() < 8 {
            return Err(CCError::InvalidInput("WASM bytecode too small".to_string()));
        }

        // Check magic number
        if &bytecode[0..4] != b"\0asm" {
            return Err(CCError::InvalidInput(
                "Invalid WASM magic number".to_string(),
            ));
        }

        // Check version
        if &bytecode[4..8] != &[1, 0, 0, 0] {
            return Err(CCError::InvalidInput(
                "Unsupported WASM version".to_string(),
            ));
        }

        // Check size limits
        if bytecode.len() > self.config.max_code_size {
            return Err(CCError::InvalidInput("WASM bytecode too large".to_string()));
        }

        Ok(())
    }

    /// Parse WASM module (simplified implementation)
    fn parse_module(&self, bytecode: Vec<u8>) -> Result<WasmModule> {
        // In a real implementation, this would use a WASM parser
        // For now, we'll create a simplified module representation

        let code_hash = hex::encode(blake3::hash(&bytecode).as_bytes());

        // Extract basic information (simplified)
        let exports = self.extract_exports(&bytecode)?;
        let memory_pages = self.extract_memory_pages(&bytecode)?;

        Ok(WasmModule {
            bytecode,
            code_hash,
            exports,
            memory_pages,
            compiled_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
    }

    /// Extract exported functions from WASM bytecode
    fn extract_exports(&self, bytecode: &[u8]) -> Result<Vec<String>> {
        // Basic WASM parsing to find export section
        let mut cursor = 8; // Skip magic number and version
        let mut exports = Vec::new();
        
        while cursor < bytecode.len() {
            if cursor + 5 > bytecode.len() {
                break;
            }
            
            let section_id = bytecode[cursor];
            cursor += 1;
            
            // Read section size (simplified LEB128 decoding)
            let mut section_size = 0u32;
            let mut shift = 0;
            loop {
                if cursor >= bytecode.len() {
                    break;
                }
                let byte = bytecode[cursor];
                cursor += 1;
                section_size |= ((byte & 0x7F) as u32) << shift;
                if (byte & 0x80) == 0 {
                    break;
                }
                shift += 7;
                if shift >= 32 {
                    return Err(CCError::InvalidInput("Invalid LEB128 encoding".to_string()));
                }
            }
            
            if section_id == 7 { // Export section
                let section_end = cursor + section_size as usize;
                if section_end > bytecode.len() {
                    break;
                }
                
                // Parse export entries
                if cursor < bytecode.len() {
                    let export_count = bytecode[cursor];
                    cursor += 1;
                    
                    for _ in 0..export_count {
                        if cursor >= section_end {
                            break;
                        }
                        
                        // Read name length
                        if cursor >= bytecode.len() {
                            break;
                        }
                        let name_len = bytecode[cursor] as usize;
                        cursor += 1;
                        
                        // Read name
                        if cursor + name_len > bytecode.len() || cursor + name_len > section_end {
                            break;
                        }
                        let name = String::from_utf8_lossy(&bytecode[cursor..cursor + name_len]);
                        cursor += name_len;
                        
                        // Skip kind and index
                        cursor += 2;
                        
                        exports.push(name.to_string());
                    }
                }
                break;
            } else {
                cursor += section_size as usize;
            }
        }
        
        // If no exports found, return default functions
        if exports.is_empty() {
            exports = vec![
                "init".to_string(),
                "call".to_string(),
                "query".to_string(),
            ];
        }
        
        Ok(exports)
    }

    /// Extract memory requirements from WASM bytecode
    fn extract_memory_pages(&self, bytecode: &[u8]) -> Result<u32> {
        // Basic WASM parsing to find memory section
        let mut cursor = 8; // Skip magic number and version
        
        while cursor < bytecode.len() {
            if cursor + 5 > bytecode.len() {
                break;
            }
            
            let section_id = bytecode[cursor];
            cursor += 1;
            
            // Read section size (simplified LEB128 decoding)
            let mut section_size = 0u32;
            let mut shift = 0;
            loop {
                if cursor >= bytecode.len() {
                    break;
                }
                let byte = bytecode[cursor];
                cursor += 1;
                section_size |= ((byte & 0x7F) as u32) << shift;
                if (byte & 0x80) == 0 {
                    break;
                }
                shift += 7;
                if shift >= 32 {
                    return Err(CCError::InvalidInput("Invalid LEB128 encoding".to_string()));
                }
            }
            
            if section_id == 5 { // Memory section
                let section_end = cursor + section_size as usize;
                if section_end > bytecode.len() {
                    break;
                }
                
                // Parse memory entries
                if cursor < bytecode.len() {
                    let memory_count = bytecode[cursor];
                    cursor += 1;
                    
                    if memory_count > 0 && cursor < section_end {
                        // Read memory limits
                        let limits_type = bytecode[cursor];
                        cursor += 1;
                        
                        if limits_type == 0 || limits_type == 1 {
                            // Read initial size (minimum pages)
                            let mut initial_size = 0u32;
                            let mut shift = 0;
                            loop {
                                if cursor >= bytecode.len() || cursor >= section_end {
                                    break;
                                }
                                let byte = bytecode[cursor];
                                cursor += 1;
                                initial_size |= ((byte & 0x7F) as u32) << shift;
                                if (byte & 0x80) == 0 {
                                    break;
                                }
                                shift += 7;
                                if shift >= 32 {
                                    return Err(CCError::InvalidInput("Invalid LEB128 encoding".to_string()));
                                }
                            }
                            
                            return Ok(initial_size.max(1)); // At least 1 page
                        }
                    }
                }
                break;
            } else {
                cursor += section_size as usize;
            }
        }
        
        // Default to 1 page (64KB) if no memory section found
        Ok(1)
    }

    /// Enhanced contract execution simulation
    fn simulate_execution(
        &self,
        module: &WasmModule,
        function_name: &str,
        args: Vec<u8>,
        host_functions: &mut HostFunctions,
    ) -> Result<ExecutionResult> {
        // Consume base execution gas
        host_functions.consume_gas(GasOperation::Instruction, 100)?;
        
        // Check gas limits
        let remaining_gas = host_functions.gas_used();
        if remaining_gas > host_functions.context.gas_limit {
            return Ok(ExecutionResult {
                return_value: Vec::new(),
                gas_used: host_functions.gas_used(),
                success: false,
                error: Some("Out of gas".to_string()),
                logs: host_functions.logs.clone(),
                state_changes: host_functions.state_changes.clone(),
            });
        }
        
        // Simulate different function behaviors based on function name and args
        let (success, return_value, error) = match function_name {
            "init" => {
                // Constructor function - initialize contract state
                host_functions.consume_gas(GasOperation::StorageWrite, 2)?;
                
                // Parse constructor arguments if any
                let initial_value = if !args.is_empty() && args.len() >= 8 {
                    u64::from_le_bytes([
                        args[0], args[1], args[2], args[3],
                        args[4], args[5], args[6], args[7]
                    ])
                } else {
                    0
                };
                
                host_functions.set_storage(b"initialized".to_vec(), b"true".to_vec())?;
                host_functions.set_storage(b"owner".to_vec(), host_functions.context.caller.as_ref().unwrap_or(&"unknown".to_string()).as_bytes().to_vec())?;
                host_functions.set_storage(b"value".to_vec(), initial_value.to_le_bytes().to_vec())?;
                
                host_functions.emit_log(
                    vec![b"ContractInitialized".to_vec()],
                    serde_json::to_vec(&serde_json::json!({
                        "initial_value": initial_value,
                        "owner": host_functions.context.caller
                    })).unwrap_or_default()
                );
                
                (true, b"init_success".to_vec(), None)
            }
            "call" => {
                // Regular function call - process based on arguments
                host_functions.consume_gas(GasOperation::StorageRead, 1)?;
                
                if args.is_empty() {
                    return Ok(ExecutionResult {
                        return_value: Vec::new(),
                        gas_used: host_functions.gas_used(),
                        success: false,
                        error: Some("No function selector provided".to_string()),
                        logs: host_functions.logs.clone(),
                        state_changes: host_functions.state_changes.clone(),
                    });
                }
                
                // Simple function selector based on first byte
                match args[0] {
                    0x01 => {
                        // Set value function
                        host_functions.consume_gas(GasOperation::StorageWrite, 1)?;
                        
                        let new_value = if args.len() >= 9 {
                            u64::from_le_bytes([
                                args[1], args[2], args[3], args[4],
                                args[5], args[6], args[7], args[8]
                            ])
                        } else {
                            return Ok(ExecutionResult {
                                return_value: Vec::new(),
                                gas_used: host_functions.gas_used(),
                                success: false,
                                error: Some("Invalid arguments for set_value".to_string()),
                                logs: host_functions.logs.clone(),
                                state_changes: host_functions.state_changes.clone(),
                            });
                        };
                        
                        host_functions.set_storage(b"value".to_vec(), new_value.to_le_bytes().to_vec())?;
                        host_functions.emit_log(
                            vec![b"ValueChanged".to_vec()],
                            new_value.to_le_bytes().to_vec()
                        );
                        
                        (true, b"value_set".to_vec(), None)
                    }
                    0x02 => {
                        // Get value function
                        host_functions.consume_gas(GasOperation::StorageRead, 1)?;
                        
                        let value = host_functions.get_storage(b"value")
                            .unwrap_or_else(|| vec![0; 8]);
                        
                        (true, value, None)
                    }
                    0x03 => {
                        // Transfer function (simplified)
                        host_functions.consume_gas(GasOperation::StorageRead, 2)?;
                        host_functions.consume_gas(GasOperation::StorageWrite, 2)?;
                        
                        if args.len() < 41 { // 1 + 32 (address) + 8 (amount)
                            return Ok(ExecutionResult {
                                return_value: Vec::new(),
                                gas_used: host_functions.gas_used(),
                                success: false,
                                error: Some("Invalid arguments for transfer".to_string()),
                                logs: host_functions.logs.clone(),
                                state_changes: host_functions.state_changes.clone(),
                            });
                        }
                        
                        let to_address = hex::encode(&args[1..33]);
                        let amount = u64::from_le_bytes([
                            args[33], args[34], args[35], args[36],
                            args[37], args[38], args[39], args[40]
                        ]);
                        
                        // Emit transfer event
                        host_functions.emit_log(
                            vec![
                                b"Transfer".to_vec(),
                                host_functions.context.caller.as_ref().unwrap_or(&"unknown".to_string()).as_bytes().to_vec(),
                                to_address.as_bytes().to_vec()
                            ],
                            amount.to_le_bytes().to_vec()
                        );
                        
                        (true, b"transfer_success".to_vec(), None)
                    }
                    _ => {
                        (false, Vec::new(), Some("Unknown function selector".to_string()))
                    }
                }
            }
            "query" => {
                // Read-only function
                host_functions.consume_gas(GasOperation::StorageRead, 2)?;
                
                let is_initialized = host_functions.get_storage(b"initialized").is_some();
                let owner = host_functions.get_storage(b"owner").unwrap_or_default();
                let value = host_functions.get_storage(b"value").unwrap_or_else(|| vec![0; 8]);
                
                let result = serde_json::to_vec(&serde_json::json!({
                    "initialized": is_initialized,
                    "owner": String::from_utf8_lossy(&owner),
                    "value": u64::from_le_bytes([
                        value.get(0).copied().unwrap_or(0),
                        value.get(1).copied().unwrap_or(0),
                        value.get(2).copied().unwrap_or(0),
                        value.get(3).copied().unwrap_or(0),
                        value.get(4).copied().unwrap_or(0),
                        value.get(5).copied().unwrap_or(0),
                        value.get(6).copied().unwrap_or(0),
                        value.get(7).copied().unwrap_or(0),
                    ])
                })).unwrap_or_else(|_| b"query_error".to_vec());
                
                (true, result, None)
            }
            _ => {
                // Check if function exists in the module
                if module.exports.contains(&function_name.to_string()) {
                    // Simulate execution of unknown but existing function
                    host_functions.consume_gas(GasOperation::Instruction, 50)?;
                    let result = format!("executed_{}_{}_bytes", function_name, args.len());
                    (true, result.into_bytes(), None)
                } else {
                    (false, Vec::new(), Some(format!("Function '{}' not found in contract", function_name)))
                }
            }
        };
        
        Ok(ExecutionResult {
            return_value,
            gas_used: host_functions.gas_used(),
            success,
            error,
            logs: host_functions.logs.clone(),
            state_changes: host_functions.state_changes.clone(),
        })
    }

    /// Update runtime statistics
    fn update_stats(&mut self, success: bool, gas_used: u64, execution_time_us: u64) {
        self.stats.executions += 1;

        if success {
            self.stats.successful_executions += 1;
        } else {
            self.stats.failed_executions += 1;
        }

        self.stats.total_gas_used += gas_used;

        // Update average execution time
        let total_time =
            self.stats.avg_execution_time_us * (self.stats.executions - 1) + execution_time_us;
        self.stats.avg_execution_time_us = total_time / self.stats.executions;
    }
}

impl HostFunctions {
    /// Create new host functions interface
    pub fn new(gas_meter: &mut GasMeter, context: ExecutionContext) -> Self {
        Self {
            gas_meter: gas_meter as *mut GasMeter,
            storage: HashMap::new(),
            context,
            logs: Vec::new(),
            state_changes: Vec::new(),
        }
    }

    /// Consume gas for an operation
    pub fn consume_gas(&mut self, operation: GasOperation, units: u64) -> Result<()> {
        unsafe { (*self.gas_meter).consume_gas(operation, units) }
    }

    /// Get consumed gas
    pub fn gas_used(&self) -> u64 {
        unsafe { (*self.gas_meter).consumed() }
    }

    /// Set storage value
    pub fn set_storage(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        let old_value = self.storage.get(&key).cloned();
        self.storage.insert(key.clone(), value.clone());

        self.state_changes.push(StateChange {
            contract: self.context.contract_address.clone(),
            key,
            old_value,
            new_value: Some(value),
        });

        Ok(())
    }

    /// Get storage value
    pub fn get_storage(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.storage.get(key).cloned()
    }

    /// Emit a log
    pub fn emit_log(&mut self, topics: Vec<Vec<u8>>, data: Vec<u8>) {
        self.logs.push(ContractLog {
            topics,
            data,
            contract: self.context.contract_address.clone(),
        });
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            contract_address: String::new(),
            function_name: String::new(),
            args: Vec::new(),
            gas_limit: 1_000_000,
            caller: None,
            value: 0,
            timestamp: 0,
            block_number: 0,
        }
    }
}
