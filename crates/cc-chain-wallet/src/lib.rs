//! CC Chain Wallet - Lightweight client and wallet functionality
//!
//! This crate provides wallet functionality and lightweight client operations:
//! - Transaction creation and signing
//! - Balance queries
//! - Account management
//! - Light client networking

use cc_chain_sdk::{Transaction, CCKeypair, CCPublicKey, Result};
use std::net::SocketAddr;

pub mod client;

pub use client::LightClient;

/// Simple wallet for managing keys and creating transactions
#[derive(Debug)]
pub struct Wallet {
    /// Wallet keypair
    keypair: CCKeypair,
    /// Connected light client (optional)
    client: Option<LightClient>,
}

impl Wallet {
    /// Create a new wallet with a generated keypair
    pub fn new() -> Self {
        Self {
            keypair: CCKeypair::generate(),
            client: None,
        }
    }

    /// Create a wallet from an existing keypair
    pub fn from_keypair(keypair: CCKeypair) -> Self {
        Self {
            keypair,
            client: None,
        }
    }

    /// Connect to a CC Chain node
    pub async fn connect(&mut self, node_address: SocketAddr) -> Result<()> {
        let client = LightClient::connect(node_address).await?;
        self.client = Some(client);
        Ok(())
    }

    /// Get the wallet's public key
    pub fn public_key(&self) -> CCPublicKey {
        self.keypair.public_key()
    }

    /// Create and sign a transaction
    pub fn create_transaction(
        &self,
        to: CCPublicKey,
        amount: u64,
        fee: u64,
        nonce: u64,
    ) -> Result<Transaction> {
        let mut tx = Transaction::new(
            self.keypair.public_key(),
            to,
            amount,
            fee,
            nonce,
            Vec::new(),
        );
        tx.sign(&self.keypair);
        Ok(tx)
    }

    /// Submit a transaction to the network
    pub async fn submit_transaction(&self, transaction: Transaction) -> Result<()> {
        if let Some(ref client) = self.client {
            client.submit_transaction(transaction).await
        } else {
            Err(cc_chain_sdk::CCError::Network("Not connected to any node".to_string()))
        }
    }

    /// Get account balance
    pub async fn get_balance(&self) -> Result<u64> {
        if let Some(ref client) = self.client {
            client.get_balance(&self.keypair.public_key()).await
        } else {
            Err(cc_chain_sdk::CCError::Network("Not connected to any node".to_string()))
        }
    }
}

impl Default for Wallet {
    fn default() -> Self {
        Self::new()
    }
}