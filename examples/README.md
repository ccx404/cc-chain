# CC Chain Examples

This directory contains example code and tutorials for working with CC Chain.

## Examples List

### Basic Examples

- [`basic_transaction.rs`](basic_transaction.rs) - Creating and sending transactions
- [`key_generation.rs`](key_generation.rs) - Generating and managing keypairs
- [`block_explorer.rs`](block_explorer.rs) - Querying blockchain data

### Advanced Examples

- [`custom_validator.rs`](custom_validator.rs) - Running a custom validator node
- [`smart_contract.rs`](smart_contract.rs) - Deploying and interacting with smart contracts
- [`bridge_transfer.rs`](bridge_transfer.rs) - Cross-chain asset transfers

### Network Examples

- [`testnet_setup.rs`](testnet_setup.rs) - Setting up a local test network
- [`peer_discovery.rs`](peer_discovery.rs) - P2P networking and peer management

## Running Examples

Each example can be run with:

```bash
cargo run --example example_name
```

For example:
```bash
cargo run --example basic_transaction
```

## Prerequisites

Make sure you have:
1. Rust 1.89+ installed
2. A running CC Chain node (for most examples)
3. Test keys generated (some examples)

## Getting Started

1. Start a test node:
   ```bash
   cargo run --bin cc-node -- start --node-type validator --listen 127.0.0.1:8000
   ```

2. Generate test keys:
   ```bash
   cargo run --bin cc-node -- generate-key --output examples/test-key.key
   ```

3. Run an example:
   ```bash
   cargo run --example basic_transaction
   ```