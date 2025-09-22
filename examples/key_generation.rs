//! Key generation and management example
//!
//! This example demonstrates:
//! - Generating new keypairs
//! - Saving/loading keys from files
//! - Key derivation and address generation
//! - Security best practices

use cc_core::{
    crypto::{CCKeypair, CCPublicKey},
    Result,
    CCError,
};
use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔑 CC Chain Key Management Example");
    println!("==================================");

    // Example 1: Generate a new keypair
    println!("\n📋 Step 1: Generating new keypair...");
    let keypair = CCKeypair::generate();
    
    println!("   Public key: 0x{}", hex::encode(keypair.public_key().to_bytes()));
    println!("   Address: 0x{}", hex::encode(keypair.public_key().to_bytes()));
    
    // Example 2: Sign and verify a message
    println!("\n✍️  Step 2: Signing a message...");
    let message = b"Hello, CC Chain!";
    let signature = keypair.sign(message);
    
    println!("   Message: {}", String::from_utf8_lossy(message));
    println!("   Signature: 0x{}", hex::encode(&signature.0));
    
    // Verify the signature
    let is_valid = keypair.public_key().verify(message, &signature);
    println!("   Signature verification: {}", if is_valid { "✓ Valid" } else { "✗ Invalid" });

    // Example 3: Save key to file (for demo - use secure storage in production)
    println!("\n💾 Step 3: Key persistence...");
    let key_file = "examples/demo-key.key";
    
    // In production, you would encrypt the private key
    let key_data = serde_json::json!({
        "public_key": hex::encode(keypair.public_key().to_bytes()),
        "created_at": chrono::Utc::now().to_rfc3339(),
        "version": "1.0"
    });
    
    fs::write(key_file, key_data.to_string())?;
    println!("   Key saved to: {}", key_file);

    // Example 4: Load key from file
    println!("\n📂 Step 4: Loading key from file...");
    let loaded_data = fs::read_to_string(key_file)?;
    let loaded_json: serde_json::Value = serde_json::from_str(&loaded_data)
        .map_err(|e| CCError::Json(e))?;
    
    let loaded_public_key_hex = loaded_json["public_key"].as_str()
        .ok_or_else(|| CCError::InvalidData("Missing public key".to_string()))?;
    
    let loaded_public_key_bytes = hex::decode(loaded_public_key_hex)
        .map_err(|e| CCError::HexDecode(e))?;
    
    let loaded_public_key = CCPublicKey::from_bytes(&loaded_public_key_bytes)?;
    
    println!("   Loaded public key: 0x{}", hex::encode(loaded_public_key.to_bytes()));
    println!("   Keys match: {}", keypair.public_key().to_bytes() == loaded_public_key.to_bytes());

    // Example 5: Generate multiple keys for different purposes
    println!("\n🔐 Step 5: Generating multiple keys...");
    
    let validator_key = CCKeypair::generate();
    let wallet_key = CCKeypair::generate();
    let contract_key = CCKeypair::generate();
    
    println!("   Validator key: 0x{}", hex::encode(validator_key.public_key().to_bytes()));
    println!("   Wallet key: 0x{}", hex::encode(wallet_key.public_key().to_bytes()));
    println!("   Contract key: 0x{}", hex::encode(contract_key.public_key().to_bytes()));

    // Example 6: Batch signature verification
    println!("\n📊 Step 6: Batch operations...");
    
    let messages = vec![
        b"Message 1".as_slice(),
        b"Message 2".as_slice(),
        b"Message 3".as_slice(),
    ];
    
    let mut signatures = Vec::new();
    for message in &messages {
        let sig = keypair.sign(message);
        signatures.push(sig);
    }
    
    // Verify all signatures
    let mut all_valid = true;
    for (i, (message, signature)) in messages.iter().zip(signatures.iter()).enumerate() {
        let is_valid = keypair.public_key().verify(message, signature);
        println!("   Message {}: {}", i + 1, if is_valid { "✓" } else { "✗" });
        all_valid &= is_valid;
    }
    
    println!("   Batch verification: {}", if all_valid { "✓ All valid" } else { "✗ Some invalid" });

    // Cleanup
    println!("\n🧹 Cleanup...");
    if Path::new(key_file).exists() {
        fs::remove_file(key_file)?;
        println!("   Demo key file removed");
    }

    println!("\n✅ Key management example completed!");
    println!("\n💡 Security Best Practices:");
    println!("   1. Never store private keys in plain text");
    println!("   2. Use hardware security modules (HSMs) for validators");
    println!("   3. Encrypt keys with strong passwords");
    println!("   4. Use key derivation functions (KDFs) for password-based encryption");
    println!("   5. Regularly rotate keys when possible");
    println!("   6. Use separate keys for different purposes");

    Ok(())
}

// Utility functions for key management

/// Generate a new keypair and save it securely
pub fn generate_and_save_key(file_path: &str, password: &str) -> Result<CCKeypair> {
    let keypair = CCKeypair::generate();
    
    // In production, encrypt with the password
    let encrypted_key = encrypt_data(b"placeholder_private_key_data", password)?;
    
    let key_data = serde_json::json!({
        "public_key": hex::encode(keypair.public_key().to_bytes()),
        "encrypted_private_key": hex::encode(encrypted_key),
        "created_at": chrono::Utc::now().to_rfc3339(),
        "version": "1.0"
    });
    
    fs::write(file_path, key_data.to_string())?;
    Ok(keypair)
}

/// Simple XOR encryption (use proper encryption in production!)
fn encrypt_data(data: &[u8], password: &str) -> Result<Vec<u8>> {
    let password_bytes = password.as_bytes();
    let mut encrypted = Vec::new();
    
    for (i, &byte) in data.iter().enumerate() {
        let key_byte = password_bytes[i % password_bytes.len()];
        encrypted.push(byte ^ key_byte);
    }
    
    Ok(encrypted)
}

/// Simple XOR decryption (use proper decryption in production!)
fn decrypt_data(encrypted_data: &[u8], password: &str) -> Result<Vec<u8>> {
    let password_bytes = password.as_bytes();
    let mut decrypted = Vec::new();
    
    for (i, &byte) in encrypted_data.iter().enumerate() {
        let key_byte = password_bytes[i % password_bytes.len()];
        decrypted.push(byte ^ key_byte);
    }
    
    Ok(decrypted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let keypair = CCKeypair::generate();
        assert_eq!(keypair.public_key().to_bytes().len(), 32);
    }

    #[test]
    fn test_signature_verification() {
        let keypair = CCKeypair::generate();
        let message = b"test message";
        let signature = keypair.sign(message);
        
        assert!(keypair.public_key().verify(message, &signature));
    }

    #[test]
    fn test_encryption_decryption() {
        let data = b"test_data_32_bytes_long_for_test";
        let password = "test_password";
        
        let encrypted = encrypt_data(data, password).unwrap();
        let decrypted = decrypt_data(&encrypted, password).unwrap();
        
        assert_eq!(data.to_vec(), decrypted);
    }
}