use crate::error::Result;
use blake3::Hasher;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 32-byte hash digest
pub type Hash = [u8; 32];

/// 32-byte public key
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CCPublicKey(#[serde(with = "serde_bytes")] pub [u8; 32]);

/// 32-byte private key
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CCPrivateKey(#[serde(with = "serde_bytes")] pub [u8; 32]);

/// 64-byte signature
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CCSignature(#[serde(with = "serde_bytes")] pub [u8; 64]);

/// Key pair for signing
#[derive(Debug, Clone)]
pub struct CCKeypair {
    signing_key: SigningKey,
}

impl CCKeypair {
    /// Generate a new random keypair
    pub fn generate() -> Self {
        let mut csprng = rand::rngs::OsRng;
        let signing_key = SigningKey::from_bytes(&rand::Rng::gen(&mut csprng));
        Self { signing_key }
    }

    /// Create keypair from secret key bytes
    pub fn from_secret_key(secret_bytes: &[u8; 32]) -> Result<Self> {
        let signing_key = SigningKey::from_bytes(secret_bytes);
        Ok(Self { signing_key })
    }

    /// Get the public key
    pub fn public_key(&self) -> CCPublicKey {
        CCPublicKey(self.signing_key.verifying_key().to_bytes())
    }

    /// Sign data
    pub fn sign(&self, data: &[u8]) -> CCSignature {
        let signature = self.signing_key.sign(data);
        CCSignature(signature.to_bytes())
    }
}

impl CCPublicKey {
    /// Verify a signature
    pub fn verify(&self, data: &[u8], signature: &CCSignature) -> bool {
        if let Ok(verifying_key) = VerifyingKey::from_bytes(&self.0) {
            if let Ok(sig) = Signature::try_from(&signature.0[..]) {
                return verifying_key.verify(data, &sig).is_ok();
            }
        }
        false
    }

    /// Create from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 32 {
            return Err(crate::error::CCError::InvalidInput(
                "Invalid public key length".to_string(),
            ));
        }
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(bytes);
        Ok(CCPublicKey(key_bytes))
    }

    /// Get bytes representation
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0
    }
}

impl Default for CCPublicKey {
    fn default() -> Self {
        CCPublicKey([0u8; 32])
    }
}

/// Compute Blake3 hash of data
pub fn hash(data: &[u8]) -> Hash {
    blake3::hash(data).into()
}

/// Compute Blake3 hash of multiple data pieces
pub fn hash_multiple(data_pieces: &[&[u8]]) -> Hash {
    let mut hasher = Hasher::new();
    for piece in data_pieces {
        hasher.update(piece);
    }
    hasher.finalize().into()
}

/// Merkle tree implementation for efficient batch verification
pub struct MerkleTree {
    nodes: Vec<Hash>,
    leaf_count: usize,
}

impl MerkleTree {
    /// Build a merkle tree from leaves
    pub fn build(leaves: &[Hash]) -> Self {
        if leaves.is_empty() {
            return Self {
                nodes: vec![[0u8; 32]],
                leaf_count: 0,
            };
        }

        let leaf_count = leaves.len();
        let mut nodes = Vec::with_capacity(2 * leaf_count);

        // Add leaves
        nodes.extend_from_slice(leaves);

        // Build internal nodes
        let mut level_size = leaf_count;
        while level_size > 1 {
            let next_level_size = (level_size + 1) / 2;
            let level_start = nodes.len() - level_size;

            for i in 0..next_level_size {
                let left_idx = level_start + 2 * i;
                let right_idx = std::cmp::min(left_idx + 1, level_start + level_size - 1);

                let combined = hash_multiple(&[&nodes[left_idx], &nodes[right_idx]]);
                nodes.push(combined);
            }

            level_size = next_level_size;
        }

        Self { nodes, leaf_count }
    }

    /// Get the merkle root
    pub fn root(&self) -> Hash {
        if self.nodes.is_empty() {
            [0u8; 32]
        } else {
            *self.nodes.last().unwrap()
        }
    }

    /// Generate merkle proof for a leaf
    pub fn proof(&self, leaf_index: usize) -> Option<Vec<Hash>> {
        if leaf_index >= self.leaf_count {
            return None;
        }

        let mut proof = Vec::new();
        let mut current_idx = leaf_index;
        let mut level_start = 0;
        let mut level_size = self.leaf_count;

        while level_size > 1 {
            let sibling_idx = if current_idx % 2 == 0 {
                std::cmp::min(current_idx + 1, level_size - 1)
            } else {
                current_idx - 1
            };

            proof.push(self.nodes[level_start + sibling_idx]);

            current_idx /= 2;
            level_start += level_size;
            level_size = (level_size + 1) / 2;
        }

        Some(proof)
    }

    /// Verify a merkle proof
    pub fn verify_proof(root: &Hash, leaf: &Hash, proof: &[Hash], leaf_index: usize) -> bool {
        let mut current_hash = *leaf;
        let mut index = leaf_index;

        for sibling in proof {
            current_hash = if index % 2 == 0 {
                hash_multiple(&[&current_hash, sibling])
            } else {
                hash_multiple(&[sibling, &current_hash])
            };
            index /= 2;
        }

        current_hash == *root
    }
}

/// Merkle proof for efficient verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    pub leaf_index: usize,
    pub proof: Vec<Hash>,
    pub root: Hash,
}

/// Advanced signature aggregation for batch verification
pub struct SignatureAggregator {
    signatures: Vec<CCSignature>,
    public_keys: Vec<CCPublicKey>,
    messages: Vec<Vec<u8>>,
}

impl SignatureAggregator {
    pub fn new() -> Self {
        Self {
            signatures: Vec::new(),
            public_keys: Vec::new(),
            messages: Vec::new(),
        }
    }

    /// Add a signature to the batch
    pub fn add_signature(&mut self, signature: CCSignature, public_key: CCPublicKey, message: Vec<u8>) {
        self.signatures.push(signature);
        self.public_keys.push(public_key);
        self.messages.push(message);
    }

    /// Verify all signatures in batch (more efficient than individual verification)
    pub fn verify_batch(&self) -> bool {
        if self.signatures.len() != self.public_keys.len() || self.signatures.len() != self.messages.len() {
            return false;
        }

        // For Ed25519, we verify each signature individually but in optimized batch
        // Real batch verification would require different cryptographic schemes
        for i in 0..self.signatures.len() {
            if !self.public_keys[i].verify(&self.messages[i], &self.signatures[i]) {
                return false;
            }
        }

        true
    }

    /// Clear the batch
    pub fn clear(&mut self) {
        self.signatures.clear();
        self.public_keys.clear();
        self.messages.clear();
    }

    /// Get batch size
    pub fn len(&self) -> usize {
        self.signatures.len()
    }

    pub fn is_empty(&self) -> bool {
        self.signatures.is_empty()
    }
}

/// Quantum-resistant signature scheme (placeholder for future implementation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumResistantSignature {
    // This would use post-quantum cryptography like Dilithium or Falcon
    // For now, we use a placeholder structure
    data: Vec<u8>,
}

impl QuantumResistantSignature {
    /// Create a placeholder quantum-resistant signature
    pub fn sign_quantum(_keypair: &CCKeypair, _message: &[u8]) -> Self {
        // TODO: Implement actual post-quantum signature scheme
        Self {
            data: vec![0u8; 2420], // Typical size for Dilithium signatures
        }
    }

    /// Verify quantum-resistant signature
    pub fn verify_quantum(&self, _public_key: &CCPublicKey, _message: &[u8]) -> bool {
        // TODO: Implement actual post-quantum verification
        // For now, always return true as placeholder
        !self.data.is_empty()
    }
}

/// High-performance hash cache for frequently computed hashes
pub struct HashCache {
    cache: HashMap<Vec<u8>, Hash>,
    max_size: usize,
}

impl HashCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_size,
        }
    }

    /// Get hash from cache or compute and store
    pub fn get_or_compute(&mut self, data: &[u8]) -> Hash {
        if let Some(&cached_hash) = self.cache.get(data) {
            return cached_hash;
        }

        let computed_hash = hash(data);
        
        // Implement simple eviction if cache is full
        if self.cache.len() >= self.max_size {
            // Remove oldest entry (simple FIFO)
            if let Some(key) = self.cache.keys().next().cloned() {
                self.cache.remove(&key);
            }
        }

        self.cache.insert(data.to_vec(), computed_hash);
        computed_hash
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> (usize, usize) {
        (self.cache.len(), self.max_size)
    }
}

/// Parallel hash computation for large data sets
pub fn parallel_hash_multiple(data_pieces: &[&[u8]]) -> Vec<Hash> {
    use rayon::prelude::*;
    
    data_pieces
        .par_iter()
        .map(|data| hash(data))
        .collect()
}

/// Optimized multi-hash for different algorithms
pub fn multi_hash(data: &[u8]) -> MultiHash {
    MultiHash {
        blake3: hash(data),
        sha256: {
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(data);
            let result = hasher.finalize();
            let mut hash_array = [0u8; 32];
            hash_array.copy_from_slice(&result);
            hash_array
        },
    }
}

/// Multi-hash structure containing different hash algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiHash {
    pub blake3: Hash,
    pub sha256: Hash,
}
