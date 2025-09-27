//! Core cryptography functionality
//!
//! This module provides essential cryptographic primitives including
//! hashing, digital signatures, key generation, and encryption/decryption.

use ed25519_dalek::{Signer, Verifier, Signature, VerifyingKey, SigningKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha512};
use blake3;
use std::fmt;
use thiserror::Error;

/// Cryptography-related errors
#[derive(Error, Debug, Clone, PartialEq)]
pub enum CryptographyError {
    #[error("Invalid key format")]
    InvalidKeyFormat,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Key generation failed")]
    KeyGenerationFailed,
    #[error("Signature verification failed")]
    SignatureVerificationFailed,
    #[error("Encryption failed")]
    EncryptionFailed,
    #[error("Decryption failed")]
    DecryptionFailed,
    #[error("Hash computation failed")]
    HashComputationFailed,
    #[error("Unsupported algorithm: {0}")]
    UnsupportedAlgorithm(String),
}

pub type Result<T> = std::result::Result<T, CryptographyError>;

/// Hash algorithms supported by the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HashAlgorithm {
    Sha256,
    Sha512,
    Blake3,
}

impl fmt::Display for HashAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HashAlgorithm::Sha256 => write!(f, "SHA256"),
            HashAlgorithm::Sha512 => write!(f, "SHA512"),
            HashAlgorithm::Blake3 => write!(f, "BLAKE3"),
        }
    }
}

/// Generic hash output
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hash {
    algorithm: HashAlgorithm,
    bytes: Vec<u8>,
}

impl Hash {
    /// Create new hash with specified algorithm and bytes
    pub fn new(algorithm: HashAlgorithm, bytes: Vec<u8>) -> Self {
        Hash { algorithm, bytes }
    }

    /// Get the hash algorithm used
    pub fn algorithm(&self) -> HashAlgorithm {
        self.algorithm
    }

    /// Get hash bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Get hash length
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Check if hash is empty
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(&self.bytes)
    }

    /// Create hash from hex string
    pub fn from_hex(algorithm: HashAlgorithm, hex_str: &str) -> Result<Self> {
        let bytes = hex::decode(hex_str).map_err(|_| CryptographyError::InvalidKeyFormat)?;
        Ok(Hash::new(algorithm, bytes))
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.algorithm, self.to_hex())
    }
}

/// Cryptographic hasher
#[derive(Debug)]
pub struct Hasher {
    algorithm: HashAlgorithm,
}

impl Hasher {
    /// Create new hasher with specified algorithm
    pub fn new(algorithm: HashAlgorithm) -> Self {
        Hasher { algorithm }
    }

    /// Hash single piece of data
    pub fn hash(&self, data: &[u8]) -> Result<Hash> {
        let bytes = match self.algorithm {
            HashAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(data);
                hasher.finalize().to_vec()
            }
            HashAlgorithm::Sha512 => {
                let mut hasher = Sha512::new();
                hasher.update(data);
                hasher.finalize().to_vec()
            }
            HashAlgorithm::Blake3 => {
                blake3::hash(data).as_bytes().to_vec()
            }
        };
        
        Ok(Hash::new(self.algorithm, bytes))
    }

    /// Hash multiple pieces of data
    pub fn hash_multiple(&self, data_pieces: &[&[u8]]) -> Result<Hash> {
        let bytes = match self.algorithm {
            HashAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                for piece in data_pieces {
                    hasher.update(piece);
                }
                hasher.finalize().to_vec()
            }
            HashAlgorithm::Sha512 => {
                let mut hasher = Sha512::new();
                for piece in data_pieces {
                    hasher.update(piece);
                }
                hasher.finalize().to_vec()
            }
            HashAlgorithm::Blake3 => {
                let mut hasher = blake3::Hasher::new();
                for piece in data_pieces {
                    hasher.update(piece);
                }
                hasher.finalize().as_bytes().to_vec()
            }
        };
        
        Ok(Hash::new(self.algorithm, bytes))
    }

    /// Create HMAC hash with key
    pub fn hmac(&self, key: &[u8], data: &[u8]) -> Result<Hash> {
        // Simplified HMAC implementation
        let key_hash = self.hash(key)?;
        let combined = [key_hash.as_bytes(), data].concat();
        self.hash(&combined)
    }
}

/// Digital signature wrapper
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CCSignature {
    bytes: Vec<u8>,
}

impl CCSignature {
    /// Create signature from bytes
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        CCSignature { bytes }
    }

    /// Get signature bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(&self.bytes)
    }

    /// Create from hex string
    pub fn from_hex(hex_str: &str) -> Result<Self> {
        let bytes = hex::decode(hex_str).map_err(|_| CryptographyError::InvalidSignature)?;
        Ok(CCSignature::from_bytes(bytes))
    }
}

impl fmt::Display for CCSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Public key wrapper
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CCPublicKey {
    bytes: Vec<u8>,
}

impl CCPublicKey {
    /// Create public key from bytes
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        if bytes.len() != 32 {
            return Err(CryptographyError::InvalidKeyFormat);
        }
        Ok(CCPublicKey { bytes })
    }

    /// Get key bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(&self.bytes)
    }

    /// Create from hex string
    pub fn from_hex(hex_str: &str) -> Result<Self> {
        let bytes = hex::decode(hex_str).map_err(|_| CryptographyError::InvalidKeyFormat)?;
        Self::from_bytes(bytes)
    }

    /// Verify signature against message
    pub fn verify(&self, message: &[u8], signature: &CCSignature) -> Result<bool> {
        if self.bytes.len() != 32 {
            return Err(CryptographyError::InvalidKeyFormat);
        }

        if signature.as_bytes().len() != 64 {
            return Err(CryptographyError::InvalidSignature);
        }

        let mut public_key_bytes = [0u8; 32];
        public_key_bytes.copy_from_slice(&self.bytes);

        let mut signature_bytes = [0u8; 64];
        signature_bytes.copy_from_slice(signature.as_bytes());

        let public_key = VerifyingKey::from_bytes(&public_key_bytes)
            .map_err(|_| CryptographyError::InvalidKeyFormat)?;

        let signature = Signature::try_from(signature.as_bytes())
            .map_err(|_| CryptographyError::InvalidSignature)?;

        match public_key.verify(message, &signature) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

impl fmt::Display for CCPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Private key wrapper
#[derive(Debug, Clone)]
pub struct CCPrivateKey {
    bytes: Vec<u8>,
}

impl CCPrivateKey {
    /// Create private key from bytes
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        if bytes.len() != 32 {
            return Err(CryptographyError::InvalidKeyFormat);
        }
        Ok(CCPrivateKey { bytes })
    }

    /// Get key bytes (use with caution)
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Convert to hex string (use with extreme caution)
    pub fn to_hex(&self) -> String {
        hex::encode(&self.bytes)
    }

    /// Create from hex string
    pub fn from_hex(hex_str: &str) -> Result<Self> {
        let bytes = hex::decode(hex_str).map_err(|_| CryptographyError::InvalidKeyFormat)?;
        Self::from_bytes(bytes)
    }

    /// Get corresponding public key
    pub fn public_key(&self) -> Result<CCPublicKey> {
        if self.bytes.len() != 32 {
            return Err(CryptographyError::InvalidKeyFormat);
        }

        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&self.bytes);

        let secret_key = SigningKey::from_bytes(&key_bytes);
        let public_key = secret_key.verifying_key();
        CCPublicKey::from_bytes(public_key.as_bytes().to_vec())
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Result<CCSignature> {
        if self.bytes.len() != 32 {
            return Err(CryptographyError::InvalidKeyFormat);
        }

        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&self.bytes);

        let secret_key = SigningKey::from_bytes(&key_bytes);
        let signature = secret_key.sign(message);
        Ok(CCSignature::from_bytes(signature.to_bytes().to_vec()))
    }
}

/// Key pair (public + private key)
#[derive(Debug, Clone)]
pub struct CCKeyPair {
    public_key: CCPublicKey,
    private_key: CCPrivateKey,
}

impl CCKeyPair {
    /// Generate new random keypair
    pub fn generate() -> Result<Self> {
        let private_key_bytes = CryptoUtils::random_bytes(32);
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&private_key_bytes);
        
        let signing_key = SigningKey::from_bytes(&key_bytes);
        let verifying_key = signing_key.verifying_key();

        let private_key = CCPrivateKey::from_bytes(signing_key.as_bytes().to_vec())?;
        let public_key = CCPublicKey::from_bytes(verifying_key.as_bytes().to_vec())?;

        Ok(CCKeyPair {
            public_key,
            private_key,
        })
    }

    /// Create keypair from existing private key
    pub fn from_private_key(private_key: CCPrivateKey) -> Result<Self> {
        let public_key = private_key.public_key()?;
        Ok(CCKeyPair {
            public_key,
            private_key,
        })
    }

    /// Get public key
    pub fn public_key(&self) -> &CCPublicKey {
        &self.public_key
    }

    /// Get private key
    pub fn private_key(&self) -> &CCPrivateKey {
        &self.private_key
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Result<CCSignature> {
        self.private_key.sign(message)
    }

    /// Verify a signature
    pub fn verify(&self, message: &[u8], signature: &CCSignature) -> Result<bool> {
        self.public_key.verify(message, signature)
    }
}

/// Utility functions for common cryptographic operations
pub struct CryptoUtils;

impl CryptoUtils {
    /// Generate secure random bytes
    pub fn random_bytes(length: usize) -> Vec<u8> {
        use rand::RngCore;
        let mut bytes = vec![0u8; length];
        OsRng.fill_bytes(&mut bytes);
        bytes
    }

    /// Derive key from password using simple method (not production-ready)
    pub fn derive_key_from_password(password: &str, salt: &[u8]) -> Result<CCPrivateKey> {
        let hasher = Hasher::new(HashAlgorithm::Sha256);
        let combined = [password.as_bytes(), salt].concat();
        let hash = hasher.hash(&combined)?;
        
        // Hash again to get 32 bytes
        let key_hash = hasher.hash(hash.as_bytes())?;
        CCPrivateKey::from_bytes(key_hash.as_bytes().to_vec())
    }

    /// Create a simple checksum
    pub fn checksum(data: &[u8]) -> Result<Hash> {
        let hasher = Hasher::new(HashAlgorithm::Sha256);
        hasher.hash(data)
    }

    /// Verify data integrity using checksum
    pub fn verify_checksum(data: &[u8], expected_checksum: &Hash) -> Result<bool> {
        let calculated_checksum = Self::checksum(data)?;
        Ok(calculated_checksum == *expected_checksum)
    }

    /// Simple XOR encryption (not secure for production)
    pub fn xor_encrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
        data.iter()
            .zip(key.iter().cycle())
            .map(|(d, k)| d ^ k)
            .collect()
    }

    /// Simple XOR decryption (same as encryption for XOR)
    pub fn xor_decrypt(encrypted_data: &[u8], key: &[u8]) -> Vec<u8> {
        Self::xor_encrypt(encrypted_data, key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_algorithms() {
        let data = b"test data";
        
        let sha256_hasher = Hasher::new(HashAlgorithm::Sha256);
        let sha256_hash = sha256_hasher.hash(data).unwrap();
        assert_eq!(sha256_hash.len(), 32);
        assert_eq!(sha256_hash.algorithm(), HashAlgorithm::Sha256);

        let sha512_hasher = Hasher::new(HashAlgorithm::Sha512);
        let sha512_hash = sha512_hasher.hash(data).unwrap();
        assert_eq!(sha512_hash.len(), 64);

        let blake3_hasher = Hasher::new(HashAlgorithm::Blake3);
        let blake3_hash = blake3_hasher.hash(data).unwrap();
        assert_eq!(blake3_hash.len(), 32);
    }

    #[test]
    fn test_keypair_generation() {
        let keypair = CCKeyPair::generate().unwrap();
        let public_key = keypair.public_key();
        let private_key = keypair.private_key();

        assert_eq!(public_key.as_bytes().len(), 32);
        assert_eq!(private_key.as_bytes().len(), 32);
    }

    #[test]
    fn test_signature_and_verification() {
        let keypair = CCKeyPair::generate().unwrap();
        let message = b"test message";

        let signature = keypair.sign(message).unwrap();
        let is_valid = keypair.verify(message, &signature).unwrap();

        assert!(is_valid);

        // Test with wrong message
        let wrong_message = b"wrong message";
        let is_valid_wrong = keypair.verify(wrong_message, &signature).unwrap();
        assert!(!is_valid_wrong);
    }

    #[test]
    fn test_public_key_verification() {
        let keypair = CCKeyPair::generate().unwrap();
        let message = b"test message";

        let signature = keypair.private_key().sign(message).unwrap();
        let is_valid = keypair.public_key().verify(message, &signature).unwrap();

        assert!(is_valid);
    }

    #[test]
    fn test_hash_hex_conversion() {
        let data = b"test data";
        let hasher = Hasher::new(HashAlgorithm::Sha256);
        let hash = hasher.hash(data).unwrap();

        let hex_str = hash.to_hex();
        let recreated_hash = Hash::from_hex(HashAlgorithm::Sha256, &hex_str).unwrap();

        assert_eq!(hash, recreated_hash);
    }

    #[test]
    fn test_key_hex_conversion() {
        let keypair = CCKeyPair::generate().unwrap();
        let public_key = keypair.public_key();

        let hex_str = public_key.to_hex();
        let recreated_key = CCPublicKey::from_hex(&hex_str).unwrap();

        assert_eq!(*public_key, recreated_key);
    }

    #[test]
    fn test_signature_hex_conversion() {
        let keypair = CCKeyPair::generate().unwrap();
        let message = b"test message";
        let signature = keypair.sign(message).unwrap();

        let hex_str = signature.to_hex();
        let recreated_signature = CCSignature::from_hex(&hex_str).unwrap();

        assert_eq!(signature, recreated_signature);
    }

    #[test]
    fn test_hmac_hash() {
        let hasher = Hasher::new(HashAlgorithm::Sha256);
        let key = b"secret key";
        let data = b"data to authenticate";

        let hmac_hash = hasher.hmac(key, data).unwrap();
        assert_eq!(hmac_hash.len(), 32);
    }

    #[test]
    fn test_checksum_verification() {
        let data = b"test data";
        let checksum = CryptoUtils::checksum(data).unwrap();
        
        assert!(CryptoUtils::verify_checksum(data, &checksum).unwrap());
        
        let modified_data = b"test data modified";
        assert!(!CryptoUtils::verify_checksum(modified_data, &checksum).unwrap());
    }

    #[test]
    fn test_xor_encryption() {
        let data = b"secret message";
        let key = b"encryption_key";

        let encrypted = CryptoUtils::xor_encrypt(data, key);
        let decrypted = CryptoUtils::xor_decrypt(&encrypted, key);

        assert_ne!(encrypted, data);
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_random_bytes() {
        let bytes1 = CryptoUtils::random_bytes(32);
        let bytes2 = CryptoUtils::random_bytes(32);

        assert_eq!(bytes1.len(), 32);
        assert_eq!(bytes2.len(), 32);
        assert_ne!(bytes1, bytes2); // Should be different
    }

    #[test]
    fn test_key_derivation() {
        let password = "secure_password";
        let salt = b"random_salt";

        let key1 = CryptoUtils::derive_key_from_password(password, salt).unwrap();
        let key2 = CryptoUtils::derive_key_from_password(password, salt).unwrap();

        // Same password and salt should produce same key
        assert_eq!(key1.as_bytes(), key2.as_bytes());

        let key3 = CryptoUtils::derive_key_from_password(password, b"different_salt").unwrap();
        // Different salt should produce different key
        assert_ne!(key1.as_bytes(), key3.as_bytes());
    }
}

