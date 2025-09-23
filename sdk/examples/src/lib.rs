//! CC Chain SDK Examples
//!
//! This crate provides practical examples demonstrating how to use
//! the CC Chain SDK for building blockchain applications.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SdkExampleError {
    #[error("Example execution error: {0}")]
    ExecutionError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
}

pub type Result<T> = std::result::Result<T, SdkExampleError>;

/// Example application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleConfig {
    pub network_endpoint: String,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
    pub debug_mode: bool,
}

/// Simple wallet example
#[derive(Debug)]
pub struct WalletExample {
    config: ExampleConfig,
    balance: u64,
    transactions: Vec<Transaction>,
}

/// Example transaction structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub timestamp: u64,
}

/// Token contract example
#[derive(Debug)]
pub struct TokenContractExample {
    config: ExampleConfig,
    total_supply: u64,
    balances: HashMap<String, u64>,
}

/// DeFi application example
#[derive(Debug)]
pub struct DeFiExample {
    config: ExampleConfig,
    liquidity_pools: HashMap<String, LiquidityPool>,
}

/// Liquidity pool for DeFi example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityPool {
    pub token_a: String,
    pub token_b: String,
    pub reserve_a: u64,
    pub reserve_b: u64,
    pub total_shares: u64,
}

/// NFT marketplace example
#[derive(Debug)]
pub struct NFTMarketplaceExample {
    config: ExampleConfig,
    nfts: HashMap<String, NFT>,
    listings: HashMap<String, Listing>,
}

/// NFT structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFT {
    pub token_id: String,
    pub owner: String,
    pub metadata_uri: String,
    pub properties: HashMap<String, String>,
}

/// NFT listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Listing {
    pub nft_id: String,
    pub seller: String,
    pub price: u64,
    pub currency: String,
    pub expires_at: u64,
}

/// Oracle data provider example
#[derive(Debug)]
pub struct OracleExample {
    config: ExampleConfig,
    data_feeds: HashMap<String, DataFeed>,
}

/// Oracle data feed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFeed {
    pub symbol: String,
    pub price: f64,
    pub timestamp: u64,
    pub source: String,
}

impl ExampleConfig {
    /// Create configuration for local testnet
    pub fn testnet() -> Self {
        Self {
            network_endpoint: "http://localhost:8545".to_string(),
            timeout_seconds: 30,
            retry_attempts: 3,
            debug_mode: true,
        }
    }

    /// Create configuration for mainnet
    pub fn mainnet() -> Self {
        Self {
            network_endpoint: "https://mainnet.cc-chain.org".to_string(),
            timeout_seconds: 60,
            retry_attempts: 5,
            debug_mode: false,
        }
    }
}

impl WalletExample {
    /// Create a new wallet example
    pub fn new(config: ExampleConfig) -> Self {
        Self {
            config,
            balance: 0,
            transactions: Vec::new(),
        }
    }

    /// Initialize wallet with demo data
    pub async fn initialize(&mut self) -> Result<()> {
        println!("🔗 Connecting to CC Chain at {}", self.config.network_endpoint);
        
        // Simulate network connection
        self.simulate_network_delay().await;
        
        // Set initial balance
        self.balance = 1000000; // 1M tokens
        
        // Add some demo transactions
        self.add_demo_transactions();
        
        println!("✅ Wallet initialized with balance: {} tokens", self.balance);
        Ok(())
    }

    /// Send tokens to another address
    pub async fn send_tokens(&mut self, to: &str, amount: u64) -> Result<String> {
        if amount > self.balance {
            return Err(SdkExampleError::ExecutionError("Insufficient balance".to_string()));
        }

        let tx_id = format!("tx_{}", chrono::Utc::now().timestamp());
        let transaction = Transaction {
            id: tx_id.clone(),
            from: "user_wallet".to_string(),
            to: to.to_string(),
            amount,
            timestamp: chrono::Utc::now().timestamp() as u64,
        };

        self.balance -= amount;
        self.transactions.push(transaction);

        println!("💸 Sent {} tokens to {}", amount, to);
        Ok(tx_id)
    }

    /// Get wallet balance
    pub fn get_balance(&self) -> u64 {
        self.balance
    }

    /// Get transaction history
    pub fn get_transactions(&self) -> &[Transaction] {
        &self.transactions
    }

    async fn simulate_network_delay(&self) {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    fn add_demo_transactions(&mut self) {
        let demo_txs = vec![
            Transaction {
                id: "tx_genesis".to_string(),
                from: "genesis".to_string(),
                to: "user_wallet".to_string(),
                amount: 1000000,
                timestamp: chrono::Utc::now().timestamp() as u64 - 3600,
            },
        ];
        self.transactions.extend(demo_txs);
    }
}

impl TokenContractExample {
    /// Create a new token contract example
    pub fn new(config: ExampleConfig, total_supply: u64) -> Self {
        let mut balances = HashMap::new();
        balances.insert("creator".to_string(), total_supply);
        
        Self {
            config,
            total_supply,
            balances,
        }
    }

    /// Transfer tokens between accounts
    pub fn transfer(&mut self, from: &str, to: &str, amount: u64) -> Result<()> {
        let from_balance = self.balances.get(from).unwrap_or(&0);
        
        if *from_balance < amount {
            return Err(SdkExampleError::ExecutionError("Insufficient balance".to_string()));
        }

        self.balances.insert(from.to_string(), from_balance - amount);
        let to_balance = self.balances.get(to).unwrap_or(&0);
        self.balances.insert(to.to_string(), to_balance + amount);

        println!("🔄 Transferred {} tokens from {} to {}", amount, from, to);
        Ok(())
    }

    /// Get balance for an account
    pub fn balance_of(&self, account: &str) -> u64 {
        *self.balances.get(account).unwrap_or(&0)
    }

    /// Mint new tokens (for demonstration)
    pub fn mint(&mut self, to: &str, amount: u64) -> Result<()> {
        self.total_supply += amount;
        let current_balance = self.balances.get(to).unwrap_or(&0);
        self.balances.insert(to.to_string(), current_balance + amount);
        
        println!("🪙 Minted {} tokens to {}", amount, to);
        Ok(())
    }
}

impl DeFiExample {
    /// Create a new DeFi example
    pub fn new(config: ExampleConfig) -> Self {
        Self {
            config,
            liquidity_pools: HashMap::new(),
        }
    }

    /// Create a liquidity pool
    pub fn create_pool(&mut self, token_a: &str, token_b: &str, reserve_a: u64, reserve_b: u64) -> Result<String> {
        let pool_id = format!("{}-{}", token_a, token_b);
        let pool = LiquidityPool {
            token_a: token_a.to_string(),
            token_b: token_b.to_string(),
            reserve_a,
            reserve_b,
            total_shares: reserve_a + reserve_b, // Simplified calculation
        };

        self.liquidity_pools.insert(pool_id.clone(), pool);
        println!("🏊 Created liquidity pool: {}", pool_id);
        Ok(pool_id)
    }

    /// Add liquidity to a pool
    pub fn add_liquidity(&mut self, pool_id: &str, amount_a: u64, amount_b: u64) -> Result<u64> {
        let pool = self.liquidity_pools.get_mut(pool_id)
            .ok_or_else(|| SdkExampleError::ExecutionError("Pool not found".to_string()))?;

        pool.reserve_a += amount_a;
        pool.reserve_b += amount_b;
        let shares = amount_a + amount_b; // Simplified calculation
        pool.total_shares += shares;

        println!("💧 Added liquidity to {}: {} + {} tokens", pool_id, amount_a, amount_b);
        Ok(shares)
    }

    /// Swap tokens in a pool
    pub fn swap(&mut self, pool_id: &str, token_in: &str, amount_in: u64) -> Result<u64> {
        let pool = self.liquidity_pools.get_mut(pool_id)
            .ok_or_else(|| SdkExampleError::ExecutionError("Pool not found".to_string()))?;

        // Simplified constant product formula: x * y = k
        let (reserve_in, reserve_out) = if token_in == pool.token_a {
            (pool.reserve_a, pool.reserve_b)
        } else {
            (pool.reserve_b, pool.reserve_a)
        };

        let amount_out = (reserve_out * amount_in) / (reserve_in + amount_in);

        if token_in == pool.token_a {
            pool.reserve_a += amount_in;
            pool.reserve_b -= amount_out;
        } else {
            pool.reserve_b += amount_in;
            pool.reserve_a -= amount_out;
        }

        println!("🔄 Swapped {} {} for {} tokens in {}", amount_in, token_in, amount_out, pool_id);
        Ok(amount_out)
    }
}

impl NFTMarketplaceExample {
    /// Create a new NFT marketplace example
    pub fn new(config: ExampleConfig) -> Self {
        Self {
            config,
            nfts: HashMap::new(),
            listings: HashMap::new(),
        }
    }

    /// Mint a new NFT
    pub fn mint_nft(&mut self, token_id: &str, owner: &str, metadata_uri: &str) -> Result<()> {
        let nft = NFT {
            token_id: token_id.to_string(),
            owner: owner.to_string(),
            metadata_uri: metadata_uri.to_string(),
            properties: HashMap::new(),
        };

        self.nfts.insert(token_id.to_string(), nft);
        println!("🎨 Minted NFT {} for {}", token_id, owner);
        Ok(())
    }

    /// List an NFT for sale
    pub fn list_nft(&mut self, nft_id: &str, seller: &str, price: u64, currency: &str) -> Result<String> {
        let nft = self.nfts.get(nft_id)
            .ok_or_else(|| SdkExampleError::ExecutionError("NFT not found".to_string()))?;

        if nft.owner != seller {
            return Err(SdkExampleError::ExecutionError("Only owner can list NFT".to_string()));
        }

        let listing_id = format!("listing_{}", chrono::Utc::now().timestamp());
        let listing = Listing {
            nft_id: nft_id.to_string(),
            seller: seller.to_string(),
            price,
            currency: currency.to_string(),
            expires_at: chrono::Utc::now().timestamp() as u64 + 86400, // 24 hours
        };

        self.listings.insert(listing_id.clone(), listing);
        println!("🏷️ Listed NFT {} for {} {}", nft_id, price, currency);
        Ok(listing_id)
    }

    /// Purchase an NFT
    pub fn purchase_nft(&mut self, listing_id: &str, buyer: &str) -> Result<()> {
        let listing = self.listings.remove(listing_id)
            .ok_or_else(|| SdkExampleError::ExecutionError("Listing not found".to_string()))?;

        let nft = self.nfts.get_mut(&listing.nft_id)
            .ok_or_else(|| SdkExampleError::ExecutionError("NFT not found".to_string()))?;

        nft.owner = buyer.to_string();
        println!("🛒 {} purchased NFT {} for {} {}", buyer, listing.nft_id, listing.price, listing.currency);
        Ok(())
    }
}

impl OracleExample {
    /// Create a new oracle example
    pub fn new(config: ExampleConfig) -> Self {
        Self {
            config,
            data_feeds: HashMap::new(),
        }
    }

    /// Update price feed
    pub fn update_price(&mut self, symbol: &str, price: f64, source: &str) -> Result<()> {
        let feed = DataFeed {
            symbol: symbol.to_string(),
            price,
            timestamp: chrono::Utc::now().timestamp() as u64,
            source: source.to_string(),
        };

        self.data_feeds.insert(symbol.to_string(), feed);
        println!("📊 Updated {} price to ${:.2} from {}", symbol, price, source);
        Ok(())
    }

    /// Get latest price
    pub fn get_price(&self, symbol: &str) -> Option<f64> {
        self.data_feeds.get(symbol).map(|feed| feed.price)
    }

    /// Simulate price updates
    pub async fn simulate_price_updates(&mut self) -> Result<()> {
        let symbols = vec![
            ("BTC", 45000.0),
            ("ETH", 3000.0),
            ("CC", 10.0),
            ("USDC", 1.0),
        ];

        for (symbol, base_price) in symbols {
            // Simulate price fluctuation
            let fluctuation = (rand::random::<f64>() - 0.5) * 0.1; // ±5%
            let price = base_price * (1.0 + fluctuation);
            self.update_price(symbol, price, "MockExchange")?;
            
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        Ok(())
    }
}

/// Example runner to demonstrate all SDK examples
pub struct ExampleRunner {
    config: ExampleConfig,
}

impl ExampleRunner {
    /// Create a new example runner
    pub fn new(config: ExampleConfig) -> Self {
        Self { config }
    }

    /// Run all examples
    pub async fn run_all_examples(&self) -> Result<()> {
        println!("🚀 Running CC Chain SDK Examples");
        
        self.run_wallet_example().await?;
        self.run_token_example().await?;
        self.run_defi_example().await?;
        self.run_nft_example().await?;
        self.run_oracle_example().await?;
        
        println!("✅ All examples completed successfully!");
        Ok(())
    }

    async fn run_wallet_example(&self) -> Result<()> {
        println!("\n📱 === Wallet Example ===");
        let mut wallet = WalletExample::new(self.config.clone());
        wallet.initialize().await?;
        
        wallet.send_tokens("alice", 50000).await?;
        wallet.send_tokens("bob", 30000).await?;
        
        println!("Final balance: {} tokens", wallet.get_balance());
        println!("Transaction count: {}", wallet.get_transactions().len());
        Ok(())
    }

    async fn run_token_example(&self) -> Result<()> {
        println!("\n🪙 === Token Contract Example ===");
        let mut token = TokenContractExample::new(self.config.clone(), 1_000_000);
        
        token.transfer("creator", "alice", 100000)?;
        token.transfer("alice", "bob", 25000)?;
        token.mint("charlie", 50000)?;
        
        println!("Alice balance: {}", token.balance_of("alice"));
        println!("Bob balance: {}", token.balance_of("bob"));
        println!("Charlie balance: {}", token.balance_of("charlie"));
        Ok(())
    }

    async fn run_defi_example(&self) -> Result<()> {
        println!("\n🏦 === DeFi Example ===");
        let mut defi = DeFiExample::new(self.config.clone());
        
        let pool_id = defi.create_pool("CC", "USDC", 100000, 1000000)?;
        defi.add_liquidity(&pool_id, 50000, 500000)?;
        defi.swap(&pool_id, "CC", 1000)?;
        
        Ok(())
    }

    async fn run_nft_example(&self) -> Result<()> {
        println!("\n🎨 === NFT Marketplace Example ===");
        let mut marketplace = NFTMarketplaceExample::new(self.config.clone());
        
        marketplace.mint_nft("nft_001", "artist", "https://example.com/metadata/1")?;
        let listing_id = marketplace.list_nft("nft_001", "artist", 1000, "CC")?;
        marketplace.purchase_nft(&listing_id, "collector")?;
        
        Ok(())
    }

    async fn run_oracle_example(&self) -> Result<()> {
        println!("\n📊 === Oracle Example ===");
        let mut oracle = OracleExample::new(self.config.clone());
        
        oracle.simulate_price_updates().await?;
        
        if let Some(price) = oracle.get_price("BTC") {
            println!("Current BTC price: ${:.2}", price);
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> ExampleConfig {
        ExampleConfig::testnet()
    }

    #[tokio::test]
    async fn test_wallet_example() {
        let mut wallet = WalletExample::new(test_config());
        wallet.initialize().await.unwrap();
        
        let initial_balance = wallet.get_balance();
        wallet.send_tokens("test", 1000).await.unwrap();
        assert_eq!(wallet.get_balance(), initial_balance - 1000);
    }

    #[test]
    fn test_token_contract() {
        let mut token = TokenContractExample::new(test_config(), 1000);
        token.transfer("creator", "alice", 500).unwrap();
        
        assert_eq!(token.balance_of("creator"), 500);
        assert_eq!(token.balance_of("alice"), 500);
    }

    #[test]
    fn test_defi_pool() {
        let mut defi = DeFiExample::new(test_config());
        let pool_id = defi.create_pool("A", "B", 1000, 1000).unwrap();
        
        let shares = defi.add_liquidity(&pool_id, 500, 500).unwrap();
        assert_eq!(shares, 1000);
    }

    #[test]
    fn test_nft_marketplace() {
        let mut marketplace = NFTMarketplaceExample::new(test_config());
        marketplace.mint_nft("nft1", "owner", "uri").unwrap();
        
        let listing_id = marketplace.list_nft("nft1", "owner", 100, "CC").unwrap();
        marketplace.purchase_nft(&listing_id, "buyer").unwrap();
    }

    #[test]
    fn test_oracle_feeds() {
        let mut oracle = OracleExample::new(test_config());
        oracle.update_price("TEST", 123.45, "TestSource").unwrap();
        
        assert_eq!(oracle.get_price("TEST"), Some(123.45));
    }
}
