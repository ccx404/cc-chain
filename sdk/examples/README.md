# CC Chain SDK Examples

This crate provides practical, real-world examples demonstrating how to use the CC Chain SDK to build various types of blockchain applications.

## 🚀 Quick Start

```bash
cargo add sdk-examples
```

```rust
use sdk_examples::{ExampleRunner, ExampleConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ExampleConfig::testnet();
    let runner = ExampleRunner::new(config);
    
    // Run all examples
    runner.run_all_examples().await?;
    
    Ok(())
}
```

## 📦 Available Examples

### 1. 📱 Wallet Example
A comprehensive wallet implementation demonstrating:
- Balance management
- Transaction sending
- Transaction history
- Network connectivity

```rust
use sdk_examples::{WalletExample, ExampleConfig};

let mut wallet = WalletExample::new(ExampleConfig::testnet());
wallet.initialize().await?;

// Send tokens
let tx_id = wallet.send_tokens("alice", 1000).await?;
println!("Transaction sent: {}", tx_id);

// Check balance
println!("Balance: {} tokens", wallet.get_balance());
```

### 2. 🪙 Token Contract Example
A complete ERC-20 style token implementation:
- Token transfers
- Balance queries
- Minting capabilities
- Supply management

```rust
use sdk_examples::TokenContractExample;

let mut token = TokenContractExample::new(config, 1_000_000);

// Transfer tokens
token.transfer("alice", "bob", 500)?;

// Check balances
println!("Alice: {}", token.balance_of("alice"));
println!("Bob: {}", token.balance_of("bob"));

// Mint new tokens
token.mint("charlie", 1000)?;
```

### 3. 🏦 DeFi Example
Decentralized Finance application with:
- Liquidity pools
- Token swapping
- Liquidity provision
- Automated market making

```rust
use sdk_examples::DeFiExample;

let mut defi = DeFiExample::new(config);

// Create a liquidity pool
let pool_id = defi.create_pool("CC", "USDC", 100_000, 1_000_000)?;

// Add liquidity
let shares = defi.add_liquidity(&pool_id, 50_000, 500_000)?;

// Perform a swap
let output = defi.swap(&pool_id, "CC", 1000)?;
println!("Swapped 1000 CC for {} USDC", output);
```

### 4. 🎨 NFT Marketplace Example
Complete NFT marketplace functionality:
- NFT minting
- Marketplace listings
- Purchase mechanics
- Ownership tracking

```rust
use sdk_examples::NFTMarketplaceExample;

let mut marketplace = NFTMarketplaceExample::new(config);

// Mint an NFT
marketplace.mint_nft("nft_001", "artist", "https://example.com/metadata")?;

// List for sale
let listing_id = marketplace.list_nft("nft_001", "artist", 1000, "CC")?;

// Purchase NFT
marketplace.purchase_nft(&listing_id, "collector")?;
```

### 5. 📊 Oracle Example
Price oracle implementation with:
- Price feed updates
- Data aggregation
- Real-time simulation
- Multi-source support

```rust
use sdk_examples::OracleExample;

let mut oracle = OracleExample::new(config);

// Update prices
oracle.update_price("BTC", 45000.0, "CoinGecko")?;
oracle.update_price("ETH", 3000.0, "CoinMarketCap")?;

// Get current price
if let Some(price) = oracle.get_price("BTC") {
    println!("BTC price: ${:.2}", price);
}

// Simulate live price updates
oracle.simulate_price_updates().await?;
```

## ⚙️ Configuration

### Network Configurations

```rust
use sdk_examples::ExampleConfig;

// Local testnet (for development)
let testnet_config = ExampleConfig::testnet();

// Mainnet (for production)
let mainnet_config = ExampleConfig::mainnet();

// Custom configuration
let custom_config = ExampleConfig {
    network_endpoint: "https://custom-node.example.com".to_string(),
    timeout_seconds: 30,
    retry_attempts: 5,
    debug_mode: false,
};
```

## 🏃 Running Examples

### Run All Examples

```rust
use sdk_examples::{ExampleRunner, ExampleConfig};

let config = ExampleConfig::testnet();
let runner = ExampleRunner::new(config);

// This will run all 5 examples in sequence
let results = runner.run_all_examples().await?;
```

### Run Individual Examples

```rust
// Run only the wallet example
runner.run_wallet_example().await?;

// Run only the DeFi example
runner.run_defi_example().await?;

// Run only the NFT marketplace example
runner.run_nft_example().await?;
```

## 🧪 Testing

All examples include comprehensive tests:

```bash
cargo test --package sdk-examples
```

### Test Coverage

- ✅ Wallet functionality
- ✅ Token operations
- ✅ DeFi pool operations
- ✅ NFT marketplace operations
- ✅ Oracle price feeds

## 🔧 Data Structures

### Transaction Structure

```rust
Transaction {
    id: String,           // Unique transaction ID
    from: String,         // Sender address
    to: String,           // Recipient address
    amount: u64,          // Transfer amount
    timestamp: u64,       // Transaction timestamp
}
```

### NFT Structure

```rust
NFT {
    token_id: String,                    // Unique token identifier
    owner: String,                       // Current owner address
    metadata_uri: String,                // Metadata location
    properties: HashMap<String, String>, // Custom properties
}
```

### Liquidity Pool Structure

```rust
LiquidityPool {
    token_a: String,      // First token symbol
    token_b: String,      // Second token symbol
    reserve_a: u64,       // First token reserves
    reserve_b: u64,       // Second token reserves
    total_shares: u64,    // Total liquidity shares
}
```

## 🎯 Use Cases

### For Developers
- Learn CC Chain SDK patterns
- Understand blockchain application architecture
- Bootstrap new projects quickly
- Reference implementations

### For Educators
- Teaching material for blockchain courses
- Workshop examples
- Code-along sessions
- Assessment templates

### For Entrepreneurs
- Prototype validation
- MVP development
- Investor demonstrations
- Proof of concept

## 🛠️ Advanced Features

### Error Handling

All examples include comprehensive error handling:

```rust
use sdk_examples::SdkExampleError;

match wallet.send_tokens("alice", 1000000).await {
    Ok(tx_id) => println!("Success: {}", tx_id),
    Err(SdkExampleError::ExecutionError(msg)) => println!("Execution failed: {}", msg),
    Err(SdkExampleError::NetworkError(msg)) => println!("Network error: {}", msg),
    Err(SdkExampleError::ConfigError(msg)) => println!("Config error: {}", msg),
}
```

### Async Support

All examples are built with async/await support for modern Rust applications:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // All examples support async operations
    wallet.initialize().await?;
    oracle.simulate_price_updates().await?;
    runner.run_all_examples().await?;
    Ok(())
}
```

## 📈 Performance Considerations

- Examples use mock data for fast execution
- Real network calls would have different performance characteristics
- Consider connection pooling for production applications
- Implement proper retry mechanisms for network operations

## 🤝 Contributing

Want to add more examples? See [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

Common additions:
- Gaming applications
- Supply chain tracking
- Identity management
- Governance systems
- Cross-chain bridges

## 📚 Learning Path

1. **Start with Wallet** - Basic blockchain interactions
2. **Token Contract** - Smart contract basics
3. **DeFi Application** - Advanced DeFi concepts
4. **NFT Marketplace** - Digital asset management
5. **Oracle Integration** - External data integration

## 🔗 Related Resources

- [CC Chain Documentation](https://docs.cc-chain.org)
- [SDK Reference](../sdk/)
- [Tutorials](../tutorials/)
- [Development Tools](../../tools/development/)