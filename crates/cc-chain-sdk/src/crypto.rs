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
