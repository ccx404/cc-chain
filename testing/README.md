# CC Chain Testing Framework

A comprehensive testing framework for the CC Chain blockchain project, providing utilities for unit testing, integration testing, performance testing, stress testing, benchmarking, mocking, and test data management.

## 📦 Components

### 🔧 [testing-helpers](helpers/)
**Common testing utilities and helpers**

- **Test Data Generators**: Random addresses, transaction hashes, keypairs, block data
- **Assertion Helpers**: Specialized assertions for blockchain operations 
- **Environment Setup**: Test environment configuration and cleanup
- **Timing Utilities**: Performance measurement and timing helpers

```rust
use testing::helpers::generators;

let address = generators::random_address();
let tx_hash = generators::random_tx_hash();
let block = generators::test_block_data(42);
```

### 🗂️ [testing-fixtures](fixtures/)
**Pre-defined test data fixtures**

- **Account Fixtures**: Alice, Bob, and other test accounts
- **Block Fixtures**: Genesis block, sample blocks with transactions
- **Transaction Fixtures**: Sample transactions for testing
- **Contract Fixtures**: Smart contract test data
- **Consensus Message Fixtures**: Sample consensus protocol messages

```rust
use testing::fixtures::common;

let alice = common::alice();
let genesis = common::genesis_block(); 
let tx = common::sample_transaction();
```

### 🎭 [testing-mocks](mocks/)
**Mock implementations for testing**

- **Mock Blockchain**: Simulated blockchain with accounts, transactions, mining
- **Mock Network**: Peer-to-peer network simulation
- **Mock Consensus**: Consensus algorithm simulation with validators

```rust
use testing::mocks::MockBlockchain;

let blockchain = MockBlockchain::new();
blockchain.create_account("alice")?;
blockchain.create_account("bob")?;
let tx_hash = blockchain.submit_transaction("alice", "bob", 1000)?;
```

### 🛠️ [testing-utilities](utilities/)
**General-purpose testing utilities**

- **Comparison Utilities**: JSON, bytes, float comparison with tolerance
- **Retry Logic**: Exponential backoff and fixed interval retry mechanisms  
- **Logging**: Test logger with level filtering and verification
- **Wait/Polling**: Condition waiting and polling utilities
- **Random Generation**: Random strings, bytes, numbers for testing
- **File System**: Temporary directory management

```rust
use testing::utilities::{comparison, retry, wait};

// Compare with tolerance
comparison::compare_floats(1.0, 1.001, 0.01)?;

// Retry with exponential backoff
let result = retry::with_exponential_backoff(|| api_call(), 3, Duration::from_millis(100))?;

// Wait for condition
wait::for_condition(|| service.is_ready(), Duration::from_secs(30))?;
```

### 📊 [testing-performance](performance/)
**Performance testing and monitoring**

- **Benchmark Runner**: Configurable benchmarking with warmup phases
- **Memory Monitor**: Memory usage tracking and statistics
- **Throughput Meter**: Operations per second and bytes per second measurement
- **Load Testing**: Concurrent user simulation and load generation

```rust
use testing::performance::BenchmarkRunner;

let mut runner = BenchmarkRunner::new();
runner.add_benchmark("hash_function", config, || {
    hash_some_data();
    Ok(Duration::from_millis(1))
});

let results = runner.run_all()?;
```

### 🔥 [testing-stress](stress/)
**Stress testing utilities**

- **Stress Test Runner**: High-load testing with configurable ramp-up
- **Memory Stress**: Memory exhaustion testing
- **Connection Stress**: Connection limit testing
- **Resource Monitoring**: System resource usage tracking

```rust
use testing::stress::{StressTestRunner, StressTestConfig};

let config = StressTestConfig {
    name: "api_stress".to_string(),
    duration: Duration::from_secs(60),
    max_load: 1000,
    ramp_up_duration: Duration::from_secs(10),
    memory_limit: None,
    cpu_limit: None,
};

let runner = StressTestRunner::new(config);
let result = runner.run(|| api_call())?;
```

### 📈 [testing-benchmarks](benchmarks/)
**Comprehensive benchmarking framework**

- **Benchmark Suite**: Organized benchmark execution with reporting
- **Statistical Analysis**: Mean, median, standard deviation, throughput calculation
- **Performance Regression Detection**: Baseline comparison and regression alerting
- **Report Generation**: Detailed performance reports

```rust
use testing::benchmarks::BenchmarkSuite;

let mut suite = BenchmarkSuite::new("crypto_benchmarks");
suite.add_benchmark("sha256", config, || {
    let data = b"benchmark data";
    let _hash = sha256(data);
    Ok(Duration::from_nanos(1000))
});

let results = suite.run_all()?;
let report = suite.generate_report();
```

### 🔗 [testing-integration](integration/)
**Integration testing utilities**

- **Test Suite Runner**: Multi-component integration test orchestration
- **Component Interface**: Standardized test component lifecycle management
- **Test Case Management**: Integration test case definition and execution

```rust
use testing::integration::{IntegrationTestSuite, IntegrationTestCase};

let mut suite = IntegrationTestSuite::new("blockchain_integration");
suite.add_component(Box::new(ConsensusComponent::new()));
suite.add_component(Box::new(NetworkComponent::new()));

let test_case = IntegrationTestCase {
    name: "consensus_network_interaction".to_string(),
    description: "Test consensus messages over network".to_string(),
    components: vec!["consensus".to_string(), "network".to_string()],
    timeout: Duration::from_secs(30),
};

suite.add_test_case(test_case);
let results = suite.run_all()?;
```

### 🧪 [testing-unit](unit/)
**Unit testing specialized utilities**

- **Unit Test Context**: Test environment setup and cleanup
- **Blockchain Assertions**: Specialized assertions for blockchain components
- **Test Data Builders**: Fluent API for building test objects

```rust
use testing::unit::{UnitTestContext, assertions, builders};

let context = UnitTestContext::new("blockchain_test");
context.setup()?;

let tx = builders::TransactionBuilder::new()
    .with_hash("tx123")
    .with_from("alice") 
    .with_to("bob")
    .with_amount(1000)
    .build();

assertions::assert_transaction_valid(&tx.hash, &tx.from, &tx.to, tx.amount)?;
```

## 🚀 Quick Start

### Basic Usage

```rust
use testing::common;

#[test]
fn test_transaction_processing() {
    // Get test fixtures
    let alice = common::alice_account();
    let bob = common::bob_account();
    
    // Create mock blockchain
    let blockchain = common::mock_blockchain();
    blockchain.create_account(&alice.address).unwrap();
    blockchain.create_account(&bob.address).unwrap();
    
    // Test transaction
    let tx_hash = blockchain
        .submit_transaction(&alice.address, &bob.address, 1000)
        .unwrap();
    
    assert!(!tx_hash.is_empty());
    assert_eq!(blockchain.get_balance(&alice.address).unwrap(), 999000);
    assert_eq!(blockchain.get_balance(&bob.address).unwrap(), 1001000);
}
```

### Performance Testing

```rust
use testing::performance::BenchmarkRunner;
use std::time::Duration;

#[test]
fn benchmark_hash_function() {
    let mut runner = BenchmarkRunner::new();
    
    let config = BenchmarkConfig {
        name: "hash_benchmark".to_string(),
        warmup_iterations: 100,
        measurement_iterations: 1000,
        timeout: Duration::from_secs(30),
        memory_limit: None,
        cpu_limit: None,
    };
    
    runner.add_benchmark("sha256", config, || {
        let data = b"test data for hashing";
        let _hash = sha256(data);
        Ok(Duration::from_nanos(500))
    });
    
    let results = runner.run_all().unwrap();
    assert!(!results.is_empty());
    
    let report = runner.generate_report();
    println!("{}", report);
}
```

## 📊 Features

### ✅ Comprehensive Test Coverage
- **33+ Unit Tests** across all testing utilities
- **Mock implementations** for blockchain, network, and consensus
- **Test fixtures** for accounts, blocks, transactions, and contracts
- **Performance benchmarking** with statistical analysis

### 🔧 Developer Experience
- **Fluent APIs** for test data building
- **Easy-to-use utilities** with sensible defaults
- **Extensive documentation** with code examples
- **Modular design** - use only what you need

### 📈 Performance & Monitoring
- **Memory usage tracking** with peak and average monitoring
- **Throughput measurement** in operations/sec and bytes/sec
- **Load testing** with concurrent user simulation
- **Stress testing** with resource exhaustion detection

### 🎯 Production Ready
- **Error handling** with typed errors and detailed messages
- **Resource cleanup** with RAII patterns
- **Thread safety** where applicable
- **Configurable timeouts** and limits

## 📚 Documentation

Each submodule contains detailed documentation and examples. Key documentation files:

- `testing/helpers/src/lib.rs` - Helper utilities and generators
- `testing/fixtures/src/lib.rs` - Test data fixtures and common data
- `testing/mocks/src/lib.rs` - Mock implementations
- `testing/utilities/src/lib.rs` - General testing utilities
- `testing/performance/src/lib.rs` - Performance testing framework
- `testing/stress/src/lib.rs` - Stress testing utilities  
- `testing/benchmarks/src/lib.rs` - Benchmarking framework
- `testing/integration/src/lib.rs` - Integration testing
- `testing/unit/src/lib.rs` - Unit testing utilities

## 🧪 Running Tests

```bash
# Run all testing framework tests
cargo test --package testing

# Run specific submodule tests
cargo test --package testing-helpers
cargo test --package testing-fixtures
cargo test --package testing-mocks
cargo test --package testing-utilities
cargo test --package testing-performance
cargo test --package testing-stress
cargo test --package testing-benchmarks
cargo test --package testing-integration
cargo test --package testing-unit

# Run with verbose output
cargo test --package testing -- --nocapture
```

## 🏗️ Architecture

The testing framework follows a modular architecture where each component can be used independently:

```
testing/
├── src/lib.rs          # Main framework facade
├── helpers/            # Basic testing utilities
├── fixtures/           # Test data fixtures
├── mocks/              # Mock implementations
├── utilities/          # General utilities
├── performance/        # Performance testing
├── stress/             # Stress testing  
├── benchmarks/         # Benchmarking
├── integration/        # Integration testing
└── unit/               # Unit testing utilities
```

Each module provides:
- **Comprehensive API** with builder patterns where appropriate
- **Error handling** with detailed error types
- **Documentation** with usage examples
- **Unit tests** validating functionality
- **Type safety** with strongly-typed interfaces

## 🎯 Use Cases

### Unit Testing
- Test individual blockchain functions and algorithms
- Verify transaction processing logic
- Test cryptographic operations
- Validate state transitions

### Integration Testing  
- Test component interactions
- Verify consensus protocol implementation
- Test network message handling
- Validate cross-chain bridge functionality

### Performance Testing
- Benchmark transaction processing throughput
- Measure block validation performance
- Test memory usage under load
- Verify response time requirements

### Stress Testing
- Test system behavior under extreme load
- Verify graceful degradation
- Test resource exhaustion scenarios
- Validate system limits and thresholds

This testing framework provides a solid foundation for ensuring CC Chain's reliability, performance, and correctness across all components.