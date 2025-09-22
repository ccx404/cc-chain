//! Basic transaction example
//!
//! This example demonstrates how to:
//! - Generate keypairs
//! - Create transactions
//! - Sign and send transactions
//! - Query transaction status

use cc_core::{
    crypto::CCKeypair,
    transaction::Transaction,
    CCError,
    Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 CC Chain Basic Transaction Example");
    println!("=====================================");

    // Generate sender and receiver keypairs
    println!("\n📋 Step 1: Generating keypairs...");
    let sender_keypair = CCKeypair::generate();
    let receiver_keypair = CCKeypair::generate();

    println!("   Sender address: 0x{}", hex::encode(sender_keypair.public_key().to_bytes()));
    println!("   Receiver address: 0x{}", hex::encode(receiver_keypair.public_key().to_bytes()));

    // Create a transaction
    println!("\n💰 Step 2: Creating transaction...");
    let amount = 1_000_000; // 1 CC token (assuming 6 decimals)
    let fee = 1_000;        // 0.001 CC token fee
    let nonce = 0;          // First transaction
    let data = Vec::new();  // No additional data

    let mut transaction = Transaction::new(
        sender_keypair.public_key(),
        receiver_keypair.public_key(),
        amount,
        fee,
        nonce,
        data,
    );

    println!("   Amount: {} units", amount);
    println!("   Fee: {} units", fee);
    println!("   Nonce: {}", nonce);

    // Sign the transaction
    println!("\n✍️  Step 3: Signing transaction...");
    transaction.sign(&sender_keypair);
    
    let tx_hash = transaction.hash();
    println!("   Transaction hash: 0x{}", hex::encode(&tx_hash));
    println!("   Signature created: ✓");

    // Validate the transaction
    println!("\n🔍 Step 4: Validating transaction...");
    match transaction.validate() {
        Ok(()) => println!("   Transaction validation: ✓ Passed"),
        Err(e) => {
            println!("   Transaction validation: ✗ Failed - {}", e);
            return Err(e);
        }
    }

    // Verify signature
    match transaction.verify_signature() {
        true => println!("   Signature verification: ✓ Valid"),
        false => {
            println!("   Signature verification: ✗ Invalid");
            return Err(CCError::Transaction("Invalid signature".to_string()));
        }
    }

    // Display transaction details
    println!("\n📄 Transaction Details:");
    println!("   From: 0x{}", hex::encode(transaction.from.to_bytes()));
    println!("   To: 0x{}", hex::encode(transaction.to.to_bytes()));
    println!("   Amount: {}", transaction.amount);
    println!("   Fee: {}", transaction.fee);
    println!("   Nonce: {}", transaction.nonce);
    println!("   Hash: 0x{}", hex::encode(&transaction.hash()));
    println!("   Size: {} bytes", transaction.size());

    // Serialize transaction for network transmission
    println!("\n📦 Step 5: Serializing transaction...");
    let serialized = bincode::serialize(&transaction)
        .map_err(|e| CCError::Serialization(e))?;
    
    println!("   Serialized size: {} bytes", serialized.len());

    // Deserialize to verify
    let deserialized: Transaction = bincode::deserialize(&serialized)
        .map_err(|e| CCError::Serialization(e))?;
    
    println!("   Deserialization: ✓ Success");
    println!("   Hash matches: {}", transaction.hash() == deserialized.hash());

    // In a real application, you would send this transaction to a node:
    // let client = CCChainClient::new("http://localhost:8001");
    // let result = client.send_transaction(transaction).await?;
    // println!("   Transaction submitted: {}", result.transaction_hash);

    println!("\n✅ Example completed successfully!");
    println!("\n💡 Next steps:");
    println!("   1. Start a CC Chain node");
    println!("   2. Use the CLI to send real transactions");
    println!("   3. Check transaction status on the blockchain");

    Ok(())
}

// Helper function to create a sample transaction for testing
pub fn create_sample_transaction() -> Result<Transaction> {
    let sender = CCKeypair::generate();
    let receiver = CCKeypair::generate();
    
    let mut tx = Transaction::new(
        sender.public_key(),
        receiver.public_key(),
        1_000_000,
        1_000,
        0,
        Vec::new(),
    );
    
    tx.sign(&sender);
    Ok(tx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_transaction() {
        let tx = create_sample_transaction().unwrap();
        assert!(tx.validate().is_ok());
        assert!(tx.verify_signature());
    }

    #[test]
    fn test_transaction_serialization() {
        let tx = create_sample_transaction().unwrap();
        let serialized = bincode::serialize(&tx).unwrap();
        let deserialized: Transaction = bincode::deserialize(&serialized).unwrap();
        
        assert_eq!(tx.hash(), deserialized.hash());
        assert_eq!(tx.amount, deserialized.amount);
        assert_eq!(tx.fee, deserialized.fee);
    }
}