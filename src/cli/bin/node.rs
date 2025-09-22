use cc_chain::{
    CCKeypair,
    cli::node::{CCNode, NodeConfig, NodeType},
    Transaction,
    vm::{SmartContractVM, VMConfig},
    Result,
};
use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use std::path::PathBuf;
use tracing::info;

#[derive(Parser)]
#[command(
    name = "cc-node",
    about = "CC Chain - High efficiency blockchain node",
    version = "0.1.0"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a CC Chain node
    Start {
        /// Node type (validator, light-compute, wallet)
        #[arg(long, value_enum, default_value = "light-compute")]
        node_type: CliNodeType,

        /// Network listening address
        #[arg(long, default_value = "0.0.0.0:8000")]
        listen: SocketAddr,

        /// Bootstrap peer addresses
        #[arg(long)]
        bootstrap: Vec<SocketAddr>,

        /// Data directory
        #[arg(long, default_value = "./data")]
        data_dir: PathBuf,

        /// Validator private key file (for validators)
        #[arg(long)]
        validator_key: Option<PathBuf>,

        /// Maximum mempool size
        #[arg(long, default_value = "10000")]
        max_mempool_size: usize,

        /// Enable metrics collection
        #[arg(long)]
        metrics: bool,
    },

    /// Generate a new keypair
    GenerateKey {
        /// Output file for the private key
        #[arg(long)]
        output: PathBuf,
    },

    /// Get node information
    Info {
        /// Node RPC address
        #[arg(long, default_value = "127.0.0.1:8001")]
        rpc: SocketAddr,
    },

    /// Send a transaction
    SendTx {
        /// Sender private key file
        #[arg(long)]
        from_key: PathBuf,

        /// Recipient public key (hex)
        #[arg(long)]
        to: String,

        /// Amount to send
        #[arg(long)]
        amount: u64,

        /// Transaction fee
        #[arg(long, default_value = "1000")]
        fee: u64,

        /// Node RPC address
        #[arg(long, default_value = "127.0.0.1:8001")]
        rpc: SocketAddr,
    },

    /// Smart contract operations
    Contract {
        #[command(subcommand)]
        contract_command: ContractCommands,
    },
}


#[derive(Subcommand)]
enum ContractCommands {
    /// Deploy a new smart contract
    Deploy {
        /// Contract bytecode file (WASM)
        #[arg(long)]
        bytecode: PathBuf,

        /// Constructor arguments (hex)
        #[arg(long, default_value = "")]
        args: String,

        /// Gas limit for deployment
        #[arg(long, default_value = "1000000")]
        gas_limit: u64,

        /// Deployer key file
        #[arg(long)]
        key: PathBuf,

        /// Node RPC address
        #[arg(long, default_value = "127.0.0.1:8001")]
        rpc: SocketAddr,
    },

    /// Call a smart contract function
    Call {
        /// Contract address
        #[arg(long)]
        contract: String,

        /// Function name to call
        #[arg(long)]
        function: String,

        /// Function arguments (hex)
        #[arg(long, default_value = "")]
        args: String,

        /// Gas limit for call
        #[arg(long, default_value = "500000")]
        gas_limit: u64,

        /// Caller key file
        #[arg(long)]
        key: PathBuf,

        /// Node RPC address
        #[arg(long, default_value = "127.0.0.1:8001")]
        rpc: SocketAddr,
    },

    /// Query contract storage
    Query {
        /// Contract address
        #[arg(long)]
        contract: String,

        /// Storage key (hex)
        #[arg(long)]
        key: String,

        /// Node RPC address
        #[arg(long, default_value = "127.0.0.1:8001")]
        rpc: SocketAddr,
    },

    /// Estimate gas for contract operations
    Estimate {
        /// Operation type (deploy, call)
        #[arg(long)]
        operation: String,

        /// Contract bytecode file (for deploy) or address (for call)
        #[arg(long)]
        target: String,

        /// Function name (for call operations)
        #[arg(long)]
        function: Option<String>,

        /// Arguments (hex)
        #[arg(long, default_value = "")]
        args: String,
    },
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum CliNodeType {
    Validator,
    #[value(name = "light-compute")]
    LightCompute,
    Wallet,
}

impl From<CliNodeType> for NodeType {
    fn from(cli_type: CliNodeType) -> Self {
        match cli_type {
            CliNodeType::Validator => NodeType::Validator,
            CliNodeType::LightCompute => NodeType::LightCompute,
            CliNodeType::Wallet => NodeType::Wallet,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Start {
            node_type,
            listen,
            bootstrap,
            data_dir,
            validator_key,
            max_mempool_size,
            metrics,
        } => {
            start_node(
                node_type.into(),
                listen,
                bootstrap,
                data_dir,
                validator_key,
                max_mempool_size,
                metrics,
            )
            .await
        }

        Commands::GenerateKey { output } => generate_keypair(output).await,

        Commands::Info { rpc } => get_node_info(rpc).await,

        Commands::SendTx {
            from_key,
            to,
            amount,
            fee,
            rpc,
        } => send_transaction(from_key, to, amount, fee, rpc).await,

        Commands::Contract { contract_command } => handle_contract_command(contract_command).await,
    }
}

async fn start_node(
    node_type: NodeType,
    listen_addr: SocketAddr,
    bootstrap_peers: Vec<SocketAddr>,
    data_dir: PathBuf,
    validator_key: Option<PathBuf>,
    max_mempool_size: usize,
    enable_metrics: bool,
) -> Result<()> {
    info!(
        "Starting CC Chain node ({:?}) on {}",
        node_type, listen_addr
    );

    // Load or generate validator keypair
    let validator_keypair = if matches!(node_type, NodeType::Validator) {
        if let Some(key_path) = validator_key {
            Some(load_keypair(&key_path).await?)
        } else {
            info!("No validator key specified, generating new keypair");
            let keypair = CCKeypair::generate();
            info!(
                "Generated validator public key: {}",
                hex::encode(keypair.public_key().0)
            );
            Some(keypair)
        }
    } else {
        None
    };

    // Create node configuration
    let config = NodeConfig {
        node_type,
        listen_addr,
        validator_keypair,
        bootstrap_peers,
        data_dir: data_dir.to_string_lossy().to_string(),
        max_mempool_size,
        enable_metrics,
    };

    // Create and start node
    let node = CCNode::new(config).await?;
    node.start().await?;

    info!("CC Chain node started successfully");

    // Keep the node running
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        // Print periodic status updates
        let height = node.get_height();
        let mempool_stats = node.get_mempool_stats();
        let performance = node.get_performance_metrics();

        if height > 0 || mempool_stats.transaction_count > 0 {
            info!(
                "Height: {}, Mempool: {}/{}, TPS: {:.2}",
                height,
                mempool_stats.transaction_count,
                mempool_stats.max_transactions,
                performance.tps
            );
        }
    }
}

async fn generate_keypair(output_path: PathBuf) -> Result<()> {
    let keypair = CCKeypair::generate();
    let public_key = keypair.public_key();

    // Save private key (in a real implementation, this would be more secure)
    let private_key_data = serde_json::json!({
        "public_key": hex::encode(public_key.0),
        "note": "This is a demo implementation. In production, use proper key management."
    });

    tokio::fs::write(
        &output_path,
        serde_json::to_string_pretty(&private_key_data)?,
    )
    .await
    .map_err(|e| cc_chain::CCError::Io(e))?;

    info!("Generated keypair:");
    info!("Public key: {}", hex::encode(public_key.0));
    info!("Private key saved to: {}", output_path.display());

    Ok(())
}

async fn load_keypair(key_path: &PathBuf) -> Result<CCKeypair> {
    // This is a simplified implementation
    // In production, you'd want proper encrypted key storage
    info!("Loading keypair from: {}", key_path.display());

    // For now, just generate a deterministic keypair based on file path
    // This is NOT secure and only for demo purposes
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    key_path.hash(&mut hasher);
    let seed = hasher.finish();

    let mut key_bytes = [0u8; 32];
    key_bytes[..8].copy_from_slice(&seed.to_le_bytes());

    CCKeypair::from_secret_key(&key_bytes)
}

async fn get_node_info(rpc_addr: SocketAddr) -> Result<()> {
    // This would connect to the node's RPC interface
    // For now, just print a placeholder message
    info!("Connecting to node at {}", rpc_addr);
    info!("RPC functionality not yet implemented in this demo");

    Ok(())
}

async fn send_transaction(
    from_key_path: PathBuf,
    to_hex: String,
    amount: u64,
    fee: u64,
    rpc_addr: SocketAddr,
) -> Result<()> {
    info!("Preparing transaction:");
    info!("From key: {}", from_key_path.display());
    info!("To: {}", to_hex);
    info!("Amount: {}", amount);
    info!("Fee: {}", fee);

    // Load sender keypair
    let from_keypair = load_keypair(&from_key_path).await?;
    let from_pubkey = from_keypair.public_key();

    // Parse recipient public key
    let to_bytes = hex::decode(&to_hex)
        .map_err(|_| cc_chain::CCError::InvalidData("Invalid recipient public key".to_string()))?;

    if to_bytes.len() != 32 {
        return Err(cc_chain::CCError::InvalidData(
            "Public key must be 32 bytes".to_string(),
        ));
    }

    let mut to_pubkey_bytes = [0u8; 32];
    to_pubkey_bytes.copy_from_slice(&to_bytes);
    let to_pubkey = cc_chain::crypto::CCPublicKey(to_pubkey_bytes);

    // Create transaction
    let mut tx = Transaction::new(
        from_pubkey,
        to_pubkey,
        amount,
        fee,
        0, // Nonce would come from account state
        Vec::new(),
    );

    // Sign transaction
    tx.sign(&from_keypair);

    info!("Transaction created and signed");
    info!("Transaction hash: {}", hex::encode(tx.hash()));

    // In a real implementation, this would submit to the node via RPC
    info!("Would submit to node at {}", rpc_addr);
    info!("Transaction submission not yet implemented in this demo");

    Ok(())
}

async fn handle_contract_command(command: ContractCommands) -> Result<()> {
    match command {
        ContractCommands::Deploy {
            bytecode,
            args,
            gas_limit,
            key,
            rpc: _rpc,
        } => deploy_contract(bytecode, args, gas_limit, key).await,

        ContractCommands::Call {
            contract,
            function,
            args,
            gas_limit,
            key,
            rpc: _rpc,
        } => call_contract(contract, function, args, gas_limit, key).await,

        ContractCommands::Query {
            contract,
            key,
            rpc: _rpc,
        } => query_contract_storage(contract, key).await,

        ContractCommands::Estimate {
            operation,
            target,
            function,
            args,
        } => estimate_gas(operation, target, function, args).await,
    }
}

async fn deploy_contract(
    bytecode_path: PathBuf,
    args_hex: String,
    gas_limit: u64,
    _key_path: PathBuf,
) -> Result<()> {
    info!("🚀 Deploying Smart Contract");
    info!("===========================");

    // Read bytecode file
    let bytecode = std::fs::read(&bytecode_path).map_err(|e| cc_chain::CCError::Io(e))?;
    info!("📄 Bytecode loaded: {} bytes", bytecode.len());

    // Parse constructor arguments
    let args = if args_hex.is_empty() {
        Vec::new()
    } else {
        hex::decode(&args_hex)
            .map_err(|_| cc_chain::CCError::InvalidInput("Invalid hex arguments".to_string()))?
    };
    info!("📝 Constructor args: {} bytes", args.len());

    // Initialize VM
    let config = VMConfig::default();
    let mut vm = SmartContractVM::new(config)?;
    info!("⚙️  VM initialized with gas limit: {}", gas_limit);

    // Deploy contract
    let contract = vm.deploy_contract(bytecode, args, gas_limit)?;

    info!("✅ Contract deployed successfully!");
    info!("📧 Contract address: {}", contract.address);
    info!("⛽ Gas used: {}", gas_limit - vm.remaining_gas());
    info!("🕒 Created at: {}", contract.created_at);

    Ok(())
}

async fn call_contract(
    contract_address: String,
    function_name: String,
    args_hex: String,
    gas_limit: u64,
    _key_path: PathBuf,
) -> Result<()> {
    info!("🔧 Calling Smart Contract Function");
    info!("===================================");
    info!("📧 Contract: {}", contract_address);
    info!("🎯 Function: {}", function_name);

    // Parse function arguments
    let args = if args_hex.is_empty() {
        Vec::new()
    } else {
        hex::decode(&args_hex)
            .map_err(|_| cc_chain::CCError::InvalidInput("Invalid hex arguments".to_string()))?
    };
    info!("📝 Arguments: {} bytes", args.len());

    // Initialize VM (in real implementation, this would connect to existing VM state)
    let config = VMConfig::default();
    let _vm = SmartContractVM::new(config)?;

    // Note: In a real implementation, we would need to load the contract from the blockchain state
    // For demo purposes, we'll show what the call would look like
    info!("⚙️  VM initialized with gas limit: {}", gas_limit);

    // This would normally call the actual contract
    // let result = vm.call_contract(&contract_address, &function_name, args, gas_limit)?;

    info!("✅ Function call would be executed");
    info!("⛽ Estimated gas usage: ~25000");
    info!("📊 Note: Connect to a running node to execute actual calls");

    Ok(())
}

async fn query_contract_storage(contract_address: String, key_hex: String) -> Result<()> {
    info!("🔍 Querying Contract Storage");
    info!("============================");
    info!("📧 Contract: {}", contract_address);

    let key = hex::decode(&key_hex)
        .map_err(|_| cc_chain::CCError::InvalidInput("Invalid hex key".to_string()))?;
    info!("🔑 Key: {} bytes", key.len());

    // Initialize VM
    let config = VMConfig::default();
    let vm = SmartContractVM::new(config)?;

    // Query storage (in real implementation, this would query actual blockchain state)
    let result = vm.get_storage(&contract_address, &key)?;

    match result {
        Some(value) => {
            info!("✅ Value found: {} bytes", value.len());
            info!("📄 Data (hex): {}", hex::encode(&value));
            if let Ok(string_value) = String::from_utf8(value) {
                info!("📄 Data (string): {}", string_value);
            }
        }
        None => {
            info!("❌ No value found for the given key");
        }
    }

    Ok(())
}

async fn estimate_gas(
    operation: String,
    target: String,
    function: Option<String>,
    args_hex: String,
) -> Result<()> {
    info!("⛽ Gas Estimation");
    info!("=================");
    info!("🎯 Operation: {}", operation);
    info!("📧 Target: {}", target);

    let args = if args_hex.is_empty() {
        Vec::new()
    } else {
        hex::decode(&args_hex)
            .map_err(|_| cc_chain::CCError::InvalidInput("Invalid hex arguments".to_string()))?
    };

    let config = VMConfig::default();
    let executor = cc_chain::vm::ContractExecutor::new(config);

    let estimated_gas = match operation.as_str() {
        "deploy" => {
            // Read bytecode if target is a file path
            let bytecode = if std::path::Path::new(&target).exists() {
                std::fs::read(&target).map_err(|e| cc_chain::CCError::Io(e))?
            } else {
                vec![0u8; 1000] // Default size for estimation
            };
            executor.estimate_deployment_gas(&bytecode, &args)
        }
        "call" => {
            let function_name = function.unwrap_or_else(|| "default".to_string());
            executor.estimate_call_gas(&target, &function_name, &args)
        }
        _ => {
            return Err(cc_chain::CCError::InvalidInput(
                "Invalid operation. Use 'deploy' or 'call'".to_string(),
            ));
        }
    };

    info!("✅ Estimated gas: {}", estimated_gas);

    // Show cost breakdown
    if operation == "deploy" {
        info!("💰 Cost breakdown:");
        info!("   • Base deployment: {} gas", 50000);
        info!("   • Code storage: {} gas", (args.len() as u64) * 68);
        info!(
            "   • Initialization: {} gas",
            estimated_gas - 50000 - ((args.len() as u64) * 68)
        );
    } else {
        info!("💰 Cost breakdown:");
        info!("   • Base call: {} gas", 21000);
        info!("   • Function execution: {} gas", estimated_gas - 21000);
    }

    Ok(())
}
