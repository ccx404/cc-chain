//! Supported blockchain chains configuration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported blockchain networks
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SupportedChain {
    /// CC Chain (native chain)
    CcChain,
    /// Ethereum mainnet
    Ethereum,
    /// Binance Smart Chain
    BinanceSmartChain,
    /// Polygon
    Polygon,
    /// Avalanche
    Avalanche,
    /// Fantom
    Fantom,
    /// Arbitrum
    Arbitrum,
    /// Optimism
    Optimism,
    /// Solana
    Solana,
    /// Bitcoin (limited functionality)
    Bitcoin,
}

impl SupportedChain {
    /// Get the chain ID for this blockchain
    pub fn chain_id(&self) -> u64 {
        match self {
            SupportedChain::CcChain => 1337,
            SupportedChain::Ethereum => 1,
            SupportedChain::BinanceSmartChain => 56,
            SupportedChain::Polygon => 137,
            SupportedChain::Avalanche => 43114,
            SupportedChain::Fantom => 250,
            SupportedChain::Arbitrum => 42161,
            SupportedChain::Optimism => 10,
            SupportedChain::Solana => 101, // Mainnet
            SupportedChain::Bitcoin => 0, // Bitcoin doesn't have chain ID
        }
    }
    
    /// Get the native currency symbol
    pub fn native_currency(&self) -> &'static str {
        match self {
            SupportedChain::CcChain => "CC",
            SupportedChain::Ethereum => "ETH",
            SupportedChain::BinanceSmartChain => "BNB",
            SupportedChain::Polygon => "MATIC",
            SupportedChain::Avalanche => "AVAX",
            SupportedChain::Fantom => "FTM",
            SupportedChain::Arbitrum => "ETH",
            SupportedChain::Optimism => "ETH",
            SupportedChain::Solana => "SOL",
            SupportedChain::Bitcoin => "BTC",
        }
    }
    
    /// Check if this chain supports smart contracts
    pub fn supports_smart_contracts(&self) -> bool {
        match self {
            SupportedChain::Bitcoin => false,
            _ => true,
        }
    }
    
    /// Get the block confirmation requirement for this chain
    pub fn default_confirmations(&self) -> u32 {
        match self {
            SupportedChain::CcChain => 1,
            SupportedChain::Ethereum => 12,
            SupportedChain::BinanceSmartChain => 15,
            SupportedChain::Polygon => 20,
            SupportedChain::Avalanche => 1,
            SupportedChain::Fantom => 1,
            SupportedChain::Arbitrum => 1,
            SupportedChain::Optimism => 1,
            SupportedChain::Solana => 1,
            SupportedChain::Bitcoin => 6,
        }
    }
}

/// Chain configuration for bridge operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    /// Chain identifier
    pub chain: SupportedChain,
    /// RPC endpoint URL
    pub rpc_url: String,
    /// Bridge contract address (if applicable)
    pub bridge_contract_address: Option<String>,
    /// Gas limit for bridge transactions
    pub gas_limit: u64,
    /// Gas price strategy
    pub gas_price_strategy: GasPriceStrategy,
    /// Required confirmations before processing
    pub required_confirmations: u32,
    /// Block time in seconds
    pub block_time: u64,
    /// Maximum retry attempts for failed transactions
    pub max_retries: u32,
    /// Timeout for transaction confirmation
    pub confirmation_timeout: u64,
    /// Chain-specific configuration parameters
    pub chain_params: HashMap<String, String>,
}

/// Gas price strategy for different chains
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GasPriceStrategy {
    /// Fixed gas price
    Fixed { price: u64 },
    /// Dynamic gas price based on network conditions
    Dynamic { multiplier: f64 },
    /// EIP-1559 style (base fee + priority fee)
    Eip1559 { max_fee: u64, priority_fee: u64 },
}

impl ChainConfig {
    /// Create a default configuration for a chain
    pub fn default_for_chain(chain: SupportedChain, rpc_url: String) -> Self {
        let (gas_limit, gas_price_strategy) = match chain {
            SupportedChain::CcChain => (
                1_000_000,
                GasPriceStrategy::Fixed { price: 1000 }
            ),
            SupportedChain::Ethereum => (
                200_000,
                GasPriceStrategy::Eip1559 { max_fee: 100_000_000_000, priority_fee: 2_000_000_000 }
            ),
            SupportedChain::BinanceSmartChain => (
                200_000,
                GasPriceStrategy::Fixed { price: 5_000_000_000 }
            ),
            SupportedChain::Polygon => (
                200_000,
                GasPriceStrategy::Dynamic { multiplier: 1.2 }
            ),
            SupportedChain::Avalanche => (
                200_000,
                GasPriceStrategy::Fixed { price: 25_000_000_000 }
            ),
            SupportedChain::Fantom => (
                200_000,
                GasPriceStrategy::Fixed { price: 1_000_000_000 }
            ),
            SupportedChain::Arbitrum => (
                1_000_000,
                GasPriceStrategy::Fixed { price: 100_000_000 }
            ),
            SupportedChain::Optimism => (
                1_000_000,
                GasPriceStrategy::Fixed { price: 1_000_000 }
            ),
            SupportedChain::Solana => (
                1_400_000,
                GasPriceStrategy::Fixed { price: 5000 }
            ),
            SupportedChain::Bitcoin => (
                0, // Bitcoin doesn't use gas
                GasPriceStrategy::Fixed { price: 10 } // satoshis per byte
            ),
        };
        
        Self {
            chain: chain.clone(),
            rpc_url,
            bridge_contract_address: None,
            gas_limit,
            gas_price_strategy,
            required_confirmations: chain.default_confirmations(),
            block_time: match chain {
                SupportedChain::CcChain => 2,
                SupportedChain::Ethereum => 12,
                SupportedChain::BinanceSmartChain => 3,
                SupportedChain::Polygon => 2,
                SupportedChain::Avalanche => 2,
                SupportedChain::Fantom => 1,
                SupportedChain::Arbitrum => 1,
                SupportedChain::Optimism => 2,
                SupportedChain::Solana => 1,
                SupportedChain::Bitcoin => 600,
            },
            max_retries: 3,
            confirmation_timeout: 300, // 5 minutes
            chain_params: HashMap::new(),
        }
    }
    
    /// Set bridge contract address
    pub fn with_bridge_contract(mut self, address: String) -> Self {
        self.bridge_contract_address = Some(address);
        self
    }
    
    /// Add a chain-specific parameter
    pub fn with_param(mut self, key: String, value: String) -> Self {
        self.chain_params.insert(key, value);
        self
    }
}