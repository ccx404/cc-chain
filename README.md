# CC Chain

<div align="center">

**A High-Performance Blockchain with ccBFT Consensus**

[![Rust](https://img.shields.io/badge/rust-1.89+-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](#)

*Building the future of decentralized applications with enhanced security and performance*

</div>

## 🌟 Overview

CC Chain is a modern, high-performance blockchain platform built in Rust, featuring an innovative **ccBFT (Configurable Byzantine Fault Tolerance)** consensus mechanism. Designed for enterprise-grade applications, CC Chain offers unparalleled speed, security, and scalability while maintaining decentralization.

### ✨ Key Features

- **🚀 ccBFT Consensus**: Next-generation Byzantine Fault Tolerance with adaptive parameters
- **⚡ High Throughput**: Optimized for thousands of transactions per second
- **🔒 Enhanced Security**: Multi-layered security with validator monitoring and fault detection
- **🌐 Smart Contracts**: WebAssembly-based smart contract execution
- **🔗 Cross-Chain Bridge**: Seamless interoperability with other blockchains
- **📊 Real-time Metrics**: Comprehensive monitoring and analytics
- **🛠️ Developer-Friendly**: Rich SDK and tooling ecosystem

## 🏗️ Architecture

CC Chain is built as a modular, multi-crate architecture:

```
cc-chain/
├── core/           # Core blockchain components (blocks, transactions, crypto)
├── consensus/      # ccBFT consensus and safety systems
├── networking/     # P2P networking and bridge functionality
├── storage/        # Storage layer and mempool
├── cli/            # Command-line interface and node binary
├── sdk/            # Developer SDK and client libraries
├── contracts/      # Smart contract runtime (WASM)
├── api/            # REST API server
├── rpc/            # JSON-RPC interface
├── bridge/         # Cross-chain bridge implementation
├── wallet/         # Wallet functionality
├── indexer/        # Blockchain data indexer
├── explorer/       # Block explorer backend
├── validator/      # Validator-specific functionality
├── monitor/        # Network monitoring tools
├── metrics/        # Performance and health metrics
└── testing/        # Testing utilities and frameworks
```

## 🚀 Quick Start

### Prerequisites

- **Rust** 1.89+ (install from [rustup.rs](https://rustup.rs/))
- **Git** for version control

### Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/ccx404/cc-chain.git
   cd cc-chain
   ```

2. **Build the project**
   ```bash
   cargo build --release
   ```

3. **Run tests**
   ```bash
   cargo test
   ```

### Running a Node

1. **Generate a validator key**
   ```bash
   cargo run --bin cc-node -- generate-key --output validator.key
   ```

2. **Start a validator node**
   ```bash
   cargo run --bin cc-node -- start \
     --node-type validator \
     --listen 0.0.0.0:8000 \
     --validator-key validator.key \
     --data-dir ./node-data \
     --metrics
   ```

3. **Start a light compute node**
   ```bash
   cargo run --bin cc-node -- start \
     --node-type light-compute \
     --listen 0.0.0.0:8001 \
     --bootstrap 127.0.0.1:8000 \
     --data-dir ./light-node-data
   ```

## 📋 Node Types

CC Chain supports different node types for various use cases:

| Node Type | Description | Use Case |
|-----------|-------------|----------|
| **Validator** | Participates in consensus | Block production and validation |
| **Light Compute** | Lightweight node with computation | DApp backends, services |
| **Wallet** | Transaction-focused node | User wallets, lightweight clients |

## 💰 Transactions

### Sending Transactions

```bash
# Send tokens
cargo run --bin cc-node -- send-tx \
  --from-key sender.key \
  --to 0x1234...abcd \
  --amount 1000000 \
  --fee 1000 \
  --rpc 127.0.0.1:8001
```

### Smart Contracts

```bash
# Deploy a contract
cargo run --bin cc-node -- contract deploy \
  --bytecode contract.wasm \
  --args "0x1234" \
  --gas-limit 1000000 \
  --key deployer.key \
  --rpc 127.0.0.1:8001

# Call a contract function
cargo run --bin cc-node -- contract call \
  --contract 0xabcd...1234 \
  --function "transfer" \
  --args "0x5678,1000" \
  --gas-limit 500000 \
  --key caller.key \
  --rpc 127.0.0.1:8001
```

## 🔧 Configuration

### Node Configuration

The node can be configured through command-line arguments or configuration files:

```toml
# config.toml
[node]
node_type = "validator"
listen_address = "0.0.0.0:8000"
data_directory = "./data"
max_mempool_size = 10000

[validator]
key_file = "validator.key"

[consensus]
ccbft_enabled = true
timeout_ms = 3000
byzantine_threshold = 0.33

[networking]
bootstrap_peers = ["127.0.0.1:8001", "127.0.0.1:8002"]
max_peers = 50

[metrics]
enabled = true
endpoint = "0.0.0.0:9090"
```

## 🔐 ccBFT Consensus

CC Chain's **ccBFT (Configurable Byzantine Fault Tolerance)** consensus provides:

- **Adaptive Timeouts**: Dynamic adjustment based on network conditions
- **Performance Optimization**: Pipelined block processing
- **Byzantine Fault Tolerance**: Handles up to ⌊(n-1)/3⌋ malicious validators
- **Fast Finality**: Single-slot finality for most transactions
- **Validator Monitoring**: Real-time performance tracking and slashing

### Consensus Features

- **Safety System**: Multi-layered fault detection and recovery
- **Network Monitoring**: Partition detection and peer health tracking
- **Performance Metrics**: Throughput monitoring and optimization
- **Upgrade Path**: Seamless transition from standard consensus to ccBFT

## 🌐 Networking & Bridge

### P2P Networking
- Libp2p-based networking stack
- Gossip protocol for transaction and block propagation
- DHT-based peer discovery
- NAT traversal support

### Cross-Chain Bridge
- **Ethereum Bridge**: Bi-directional asset transfers
- **Bitcoin Bridge**: BTC wrapping and unwrapping
- **Generic Bridge**: Support for custom blockchain integrations
- **Atomic Swaps**: Trustless cross-chain exchanges

## 📊 Monitoring & Metrics

CC Chain provides comprehensive monitoring:

- **Performance Metrics**: TPS, latency, resource usage
- **Consensus Metrics**: Block time, finality, validator performance  
- **Network Metrics**: Peer count, message propagation, bandwidth
- **Smart Contract Metrics**: Gas usage, execution time, errors

Access metrics via:
- Prometheus endpoint: `http://localhost:9090/metrics`
- Built-in dashboard: `http://localhost:8080/metrics`
- CLI monitoring: `cargo run --bin cc-node -- info`

## 🛠️ Development

### SDK Usage

```rust
use cc_chain_sdk::{Client, Keypair, Transaction};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to node
    let client = Client::new("http://127.0.0.1:8001").await?;
    
    // Create a keypair
    let keypair = Keypair::generate();
    
    // Send a transaction
    let tx = Transaction::transfer(
        keypair.public_key(),
        recipient_address,
        1000000, // amount
        1000,    // fee
    );
    
    let signed_tx = keypair.sign_transaction(tx)?;
    let tx_hash = client.send_transaction(signed_tx).await?;
    
    println!("Transaction sent: {}", tx_hash);
    Ok(())
}
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test suites
cargo test --package cc-core
cargo test --package consensus
cargo test --package networking

# Run integration tests
cargo test --test ccbft_integration

# Run with logging
RUST_LOG=debug cargo test
```

### Benchmarks

```bash
# Run performance benchmarks
cargo bench

# Consensus benchmarks
cargo bench --package consensus

# Transaction processing benchmarks
cargo bench --package cc-core
```

## 📚 Documentation

- **[API Documentation](docs/api.md)** - REST API and RPC reference
- **[Architecture Guide](docs/architecture.md)** - Detailed system design
- **[Consensus Specification](docs/consensus.md)** - ccBFT algorithm details
- **[Developer Guide](docs/development.md)** - Building and contributing
- **[Smart Contracts](docs/contracts.md)** - WASM contract development
- **[Bridge Documentation](docs/bridge.md)** - Cross-chain integration

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Workflow

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Code Standards

- **Rust**: Follow `rustfmt` and `clippy` recommendations
- **Documentation**: Document all public APIs
- **Testing**: Maintain >90% test coverage
- **Security**: Follow secure coding practices

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🔗 Links

- **Website**: [https://cc-chain.org](https://cc-chain.org)
- **Documentation**: [https://docs.cc-chain.org](https://docs.cc-chain.org)
- **Discord**: [https://discord.gg/cc-chain](https://discord.gg/cc-chain)
- **Twitter**: [@CCChainProject](https://twitter.com/CCChainProject)

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) 🦀
- Powered by [Tokio](https://tokio.rs/) for async runtime
- Cryptography by [ed25519-dalek](https://docs.rs/ed25519-dalek/) and [blake3](https://docs.rs/blake3/)
- Inspired by the broader blockchain and cryptocurrency community

---

<div align="center">

**⭐ Star us on GitHub — it motivates us a lot!**

</div>