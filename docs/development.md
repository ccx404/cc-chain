# CC Chain Development Guide

## Getting Started

This guide will help you set up a development environment and contribute to CC Chain.

## Prerequisites

### Required Software

- **Rust** 1.89+ (install from [rustup.rs](https://rustup.rs/))
- **Git** for version control
- **VS Code** or your preferred editor with Rust support

### Optional Tools

- **Docker** for containerized development
- **Kubernetes** for orchestration testing
- **Postman** or similar for API testing

## Development Environment Setup

### 1. Clone and Build

```bash
# Clone the repository
git clone https://github.com/ccx404/cc-chain.git
cd cc-chain

# Build the project
cargo build

# Run tests
cargo test

# Build documentation
cargo doc --open
```

### 2. IDE Configuration

#### VS Code Extensions

Install these extensions for the best development experience:

- **rust-analyzer**: Advanced Rust language support
- **CodeLLDB**: Debugging support
- **Error Lens**: Inline error display
- **GitLens**: Enhanced Git integration
- **Thunder Client**: API testing

#### VS Code Settings

Add to your `.vscode/settings.json`:

```json
{
  "rust-analyzer.cargo.allFeatures": true,
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.procMacro.enable": true,
  "editor.formatOnSave": true,
  "editor.defaultFormatter": "rust-lang.rust-analyzer"
}
```

### 3. Environment Variables

Create a `.env` file in the project root:

```bash
# Development settings
RUST_LOG=debug
RUST_BACKTRACE=1

# Test network configuration
CC_CHAIN_NETWORK=testnet
CC_CHAIN_DATA_DIR=./dev-data

# API settings
CC_CHAIN_RPC_PORT=8001
CC_CHAIN_API_PORT=8080
```

## Project Structure

Understanding the codebase organization:

```
cc-chain/
├── core/               # Core blockchain components
│   ├── src/
│   │   ├── block.rs    # Block and blockchain structures
│   │   ├── crypto.rs   # Cryptographic primitives
│   │   ├── error.rs    # Error types and handling
│   │   ├── state.rs    # State management
│   │   ├── transaction.rs  # Transaction types
│   │   └── utils.rs    # Utility functions
│   └── Cargo.toml
├── consensus/          # ccBFT consensus implementation
│   ├── src/
│   │   ├── ccbft.rs    # Main ccBFT algorithm
│   │   ├── safety.rs   # Safety and monitoring systems
│   │   └── consensus_types.rs  # Consensus message types
│   └── Cargo.toml
├── networking/         # P2P networking and bridge
│   ├── src/
│   │   ├── network.rs  # Core networking
│   │   └── bridge.rs   # Cross-chain bridge
│   └── Cargo.toml
├── cli/               # Command-line interface
│   ├── src/
│   │   ├── bin/
│   │   │   └── node.rs # Main node binary
│   │   └── node.rs     # Node implementation
│   └── Cargo.toml
├── tests/             # Integration tests
│   └── ccbft_integration.rs
├── docs/              # Documentation
├── examples/          # Example code
└── Cargo.toml         # Workspace configuration
```

## Development Workflow

### 1. Feature Development

1. **Create a feature branch**
   ```bash
   git checkout -b feature/new-feature
   ```

2. **Write tests first** (TDD approach)
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_new_feature() {
           // Test implementation
           assert!(true);
       }
   }
   ```

3. **Implement the feature**
   ```rust
   pub fn new_feature() -> Result<String> {
       // Implementation
       Ok("feature implemented".to_string())
   }
   ```

4. **Run tests and checks**
   ```bash
   cargo test
   cargo clippy
   cargo fmt
   ```

### 2. Testing Strategy

#### Unit Tests

Each module should have comprehensive unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use cc_core::{crypto::CCKeypair, transaction::Transaction};

    #[test]
    fn test_transaction_creation() {
        let keypair = CCKeypair::generate();
        let tx = Transaction::new(
            keypair.public_key(),
            keypair.public_key(), // self-transfer for test
            1000,
            100,
            0,
        );
        assert!(tx.validate().is_ok());
    }

    #[test]
    fn test_block_validation() {
        let genesis = Block::genesis(
            CCKeypair::generate().public_key(),
            [0u8; 32],
        );
        assert!(genesis.validate().is_ok());
        assert!(genesis.is_genesis());
    }
}
```

#### Integration Tests

Integration tests should test cross-module functionality:

```rust
// tests/consensus_integration.rs
use cc_core::{Block, Transaction};
use consensus::{CCConsensus, ConsensusMessage};

#[tokio::test]
async fn test_consensus_with_transactions() {
    let consensus = CCConsensus::new(CCKeypair::generate());
    
    // Add some transactions
    let tx = Transaction::new(/* ... */);
    consensus.add_transaction(tx).await.unwrap();
    
    // Test consensus process
    let block = consensus.propose_block().await.unwrap();
    assert!(!block.transactions.is_empty());
}
```

#### Performance Tests

Use criterion for benchmarking:

```rust
// benches/consensus_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use consensus::CCConsensus;

fn benchmark_consensus(c: &mut Criterion) {
    c.bench_function("consensus_round", |b| {
        b.iter(|| {
            let consensus = CCConsensus::new(CCKeypair::generate());
            black_box(consensus.process_round());
        })
    });
}

criterion_group!(benches, benchmark_consensus);
criterion_main!(benches);
```

### 3. Code Quality

#### Formatting

Use `rustfmt` with these settings in `rustfmt.toml`:

```toml
edition = "2021"
max_width = 100
use_small_heuristics = "Default"
imports_granularity = "Module"
group_imports = "StdExternalCrate"
```

#### Linting

Configure `clippy` for strict linting:

```bash
cargo clippy -- -D warnings -D clippy::all -D clippy::pedantic
```

#### Documentation

Document all public APIs:

```rust
/// Creates a new transaction with the specified parameters.
///
/// # Arguments
///
/// * `from` - The sender's public key
/// * `to` - The recipient's public key  
/// * `amount` - The amount to transfer
/// * `fee` - The transaction fee
/// * `nonce` - The sender's current nonce
///
/// # Examples
///
/// ```
/// use cc_core::{Transaction, crypto::CCKeypair};
/// 
/// let keypair = CCKeypair::generate();
/// let tx = Transaction::new(
///     keypair.public_key(),
///     keypair.public_key(),
///     1000,
///     100,
///     0,
/// );
/// ```
///
/// # Errors
///
/// Returns an error if the amount or fee is zero.
pub fn new(
    from: CCPublicKey,
    to: CCPublicKey,
    amount: u64,
    fee: u64,
    nonce: u64,
) -> Result<Self> {
    // Implementation
}
```

## Building and Running

### Development Build

```bash
# Debug build (faster compilation, slower execution)
cargo build

# Release build (slower compilation, faster execution)
cargo build --release

# Build specific package
cargo build --package cc-core
```

### Running Nodes

#### Single Node

```bash
cargo run --bin cc-node -- start \
  --node-type validator \
  --listen 127.0.0.1:8000 \
  --data-dir ./dev-data
```

#### Multi-Node Network

Use the included script to start a test network:

```bash
# Start 4-node network
./scripts/start-testnet.sh
```

Or manually:

```bash
# Node 1 (Genesis validator)
cargo run --bin cc-node -- start \
  --node-type validator \
  --listen 127.0.0.1:8000 \
  --data-dir ./node1-data \
  --validator-key ./keys/validator1.key

# Node 2
cargo run --bin cc-node -- start \
  --node-type validator \
  --listen 127.0.0.1:8001 \
  --bootstrap 127.0.0.1:8000 \
  --data-dir ./node2-data \
  --validator-key ./keys/validator2.key

# Node 3 (Light compute)
cargo run --bin cc-node -- start \
  --node-type light-compute \
  --listen 127.0.0.1:8002 \
  --bootstrap 127.0.0.1:8000 \
  --data-dir ./node3-data
```

### Testing Transactions

```bash
# Generate keys
cargo run --bin cc-node -- generate-key --output test-key.key

# Send transaction
cargo run --bin cc-node -- send-tx \
  --from-key test-key.key \
  --to 0x1234567890abcdef... \
  --amount 1000000 \
  --fee 1000 \
  --rpc 127.0.0.1:8001
```

## Debugging

### Logging

Configure logging levels:

```bash
# Verbose logging
RUST_LOG=debug cargo run --bin cc-node

# Module-specific logging
RUST_LOG=consensus=debug,networking=info cargo run --bin cc-node

# Structured logging
RUST_LOG=debug cargo run --bin cc-node 2>&1 | jq
```

### Debugging with GDB/LLDB

1. Build with debug info:
   ```bash
   cargo build --profile dev
   ```

2. Run with debugger:
   ```bash
   rust-gdb target/debug/cc-node
   ```

3. Set breakpoints:
   ```
   (gdb) break consensus::ccbft::process_message
   (gdb) run start --node-type validator
   ```

### Profiling

Use `perf` for performance profiling:

```bash
# Build with profiling info
cargo build --release

# Profile the application
perf record --call-graph dwarf target/release/cc-node start --node-type validator

# Analyze results
perf report
```

## Contributing Guidelines

### Code Review Process

1. **Fork the repository**
2. **Create a feature branch**
3. **Make changes with tests**
4. **Submit a pull request**
5. **Address review feedback**
6. **Merge after approval**

### Commit Message Format

Use conventional commits:

```
feat: add ccBFT timeout adaptation
fix: resolve consensus deadlock in partition scenarios
docs: update API documentation for bridge methods
test: add integration tests for validator slashing
refactor: simplify transaction validation logic
```

### Pull Request Guidelines

- **Title**: Clear, descriptive title
- **Description**: Detailed explanation of changes
- **Tests**: Include relevant tests
- **Documentation**: Update docs if needed
- **Breaking Changes**: Clearly marked

Example PR template:

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature  
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing completed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] No new compiler warnings
```

### Issue Guidelines

When reporting issues:

1. **Search existing issues** first
2. **Use issue templates** when available
3. **Provide reproduction steps**
4. **Include environment details**
5. **Add relevant labels**

## Performance Optimization

### Profiling Tools

- **cargo flamegraph**: Visual profiling
- **criterion**: Benchmarking framework  
- **pprof**: Production profiling
- **valgrind**: Memory debugging

### Common Optimizations

1. **Avoid unnecessary allocations**
   ```rust
   // Instead of
   let result = format!("Block {}", height);
   
   // Use
   let mut result = String::with_capacity(20);
   write!(result, "Block {}", height).unwrap();
   ```

2. **Use efficient data structures**
   ```rust
   // Use HashMap for O(1) lookups
   use std::collections::HashMap;
   let mut cache: HashMap<Hash, Block> = HashMap::new();
   
   // Use BTreeMap for ordered iteration
   use std::collections::BTreeMap;
   let mut sorted_txs: BTreeMap<u64, Transaction> = BTreeMap::new();
   ```

3. **Parallelize when possible**
   ```rust
   use rayon::prelude::*;
   
   // Parallel transaction validation
   let valid_transactions: Vec<_> = transactions
       .par_iter()
       .filter(|tx| tx.validate().is_ok())
       .collect();
   ```

## Security Considerations

### Secure Coding Practices

1. **Input validation**
   ```rust
   pub fn process_transaction(tx: &Transaction) -> Result<()> {
       // Validate all inputs
       if tx.amount == 0 {
           return Err(CCError::InvalidInput("Amount cannot be zero".to_string()));
       }
       
       if tx.fee < MIN_FEE {
           return Err(CCError::InvalidInput("Fee too low".to_string()));
       }
       
       // Process transaction
       Ok(())
   }
   ```

2. **Safe arithmetic**
   ```rust
   // Use checked arithmetic to prevent overflows
   let total = balance
       .checked_add(amount)
       .ok_or(CCError::InvalidData("Balance overflow".to_string()))?;
   ```

3. **Constant-time operations for cryptography**
   ```rust
   use subtle::ConstantTimeEq;
   
   // Use constant-time comparison for secrets
   if signature.ct_eq(&expected_signature).into() {
       // Signatures match
   }
   ```

### Audit Checklist

Before releasing:

- [ ] All inputs validated
- [ ] No integer overflows possible
- [ ] Cryptographic operations use constant time
- [ ] No sensitive data in logs
- [ ] Error messages don't leak information
- [ ] Dependencies are up to date
- [ ] Security tests pass

## Release Process

### Version Management

We use semantic versioning (SemVer):

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Release Checklist

1. **Update version numbers**
   ```bash
   # Update workspace version in Cargo.toml
   vim Cargo.toml
   
   # Update all package versions
   cargo update
   ```

2. **Run full test suite**
   ```bash
   cargo test --all-features
   cargo test --release
   cargo bench
   ```

3. **Update documentation**
   ```bash
   cargo doc --no-deps
   # Review generated docs
   ```

4. **Create release**
   ```bash
   git tag -a v1.0.0 -m "Release version 1.0.0"
   git push origin v1.0.0
   ```

## Troubleshooting

### Common Issues

#### Compilation Errors

```bash
# Clear cargo cache
cargo clean

# Update dependencies
cargo update

# Check for conflicting features
cargo tree -d
```

#### Runtime Issues

```bash
# Enable detailed logging
RUST_LOG=trace cargo run

# Check system resources
htop
df -h

# Verify network connectivity
netstat -tulpn | grep 8000
```

#### Test Failures

```bash
# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_consensus_basic

# Run tests in single thread
cargo test -- --test-threads=1
```

### Getting Help

- **Documentation**: https://docs.cc-chain.org
- **Discord**: https://discord.gg/cc-chain
- **GitHub Discussions**: https://github.com/ccx404/cc-chain/discussions
- **Stack Overflow**: Tag questions with `cc-chain`

## Resources

### Learning Rust

- **The Rust Book**: https://doc.rust-lang.org/book/
- **Rust by Example**: https://doc.rust-lang.org/rust-by-example/
- **Rustlings**: https://github.com/rust-lang/rustlings

### Blockchain Development

- **Ethereum Yellow Paper**: Technical blockchain specification
- **Tendermint Spec**: BFT consensus algorithm
- **Libp2p Docs**: P2P networking protocols

### Tools and Libraries

- **Tokio**: Async runtime
- **Serde**: Serialization framework
- **Clap**: Command line argument parsing
- **Tracing**: Structured logging
- **Criterion**: Benchmarking framework