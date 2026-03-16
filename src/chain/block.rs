// src/chain/block.rs
//! Block structure with Merkle root commitment over transactions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::transaction::SignedTransaction;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: DateTime<Utc>,
    /// Merkle root of transaction hashes (hex). Empty string if no transactions.
    pub merkle_root: String,
    /// The transactions included in this block.
    pub transactions: Vec<SignedTransaction>,
    /// Validator/producer address that created this block.
    pub validator: String,
    pub previous_hash: String,
    pub hash: String,
}

impl Block {
    /// Create a new block with structured transactions and Merkle root.
    pub fn new(
        index: u64,
        transactions: Vec<SignedTransaction>,
        previous_hash: String,
        validator: String,
    ) -> Self {
        Self::new_at(index, transactions, previous_hash, validator, Utc::now())
    }

    /// Create a new block at a specific timestamp.
    ///
    /// This is useful for deterministic block processing where transaction validation
    /// must use the same "now" as the block timestamp.
    pub fn new_at(
        index: u64,
        transactions: Vec<SignedTransaction>,
        previous_hash: String,
        validator: String,
        timestamp: DateTime<Utc>,
    ) -> Self {
        let merkle_root = compute_merkle_root(&transactions);
        let hash =
            Self::calculate_hash(index, &timestamp, &merkle_root, &previous_hash, &validator);
        Self {
            index,
            timestamp,
            merkle_root,
            transactions,
            validator,
            previous_hash,
            hash,
        }
    }

    /// Recalculate block hash from header fields (does NOT include raw tx data, only merkle root).
    pub fn calculate_hash(
        index: u64,
        timestamp: &DateTime<Utc>,
        merkle_root: &str,
        previous_hash: &str,
        validator: &str,
    ) -> String {
        let payload = serde_json::json!({
            "index": index,
            "timestamp": timestamp.to_rfc3339(),
            "merkle_root": merkle_root,
            "previous_hash": previous_hash,
            "validator": validator,
        })
        .to_string();

        let mut hasher = Sha256::new();
        hasher.update(payload.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Verify that the merkle_root matches the actual transactions in this block.
    pub fn verify_merkle_root(&self) -> bool {
        let computed = compute_merkle_root(&self.transactions);
        computed == self.merkle_root
    }
}

/// Compute the Merkle root of a list of signed transactions.
///
/// Uses a standard binary Merkle tree:
/// - Leaf nodes are SHA-256 hashes of each transaction's canonical hash.
/// - If the number of nodes at a level is odd, the last node is duplicated.
/// - An empty transaction list produces an empty string.
pub fn compute_merkle_root(transactions: &[SignedTransaction]) -> String {
    if transactions.is_empty() {
        return String::new();
    }

    // Leaf hashes: SHA-256 of each tx hash (double-hash for Merkle leaves)
    let mut hashes: Vec<[u8; 32]> = transactions
        .iter()
        .map(|tx| {
            let tx_hash = tx.tx_hash_hex();
            let mut hasher = Sha256::new();
            hasher.update(tx_hash.as_bytes());
            hasher.finalize().into()
        })
        .collect();

    // Build tree bottom-up
    while hashes.len() > 1 {
        let mut next_level = Vec::new();
        let mut i = 0;
        while i < hashes.len() {
            let left = &hashes[i];
            // If odd number of nodes, duplicate the last one
            let right = if i + 1 < hashes.len() {
                &hashes[i + 1]
            } else {
                &hashes[i]
            };

            let mut hasher = Sha256::new();
            hasher.update(left);
            hasher.update(right);
            next_level.push(hasher.finalize().into());

            i += 2;
        }
        hashes = next_level;
    }

    hex::encode(hashes[0])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{
        generate_ed25519_keypair, pubkey_to_address_hex, SignedTransaction, Transaction,
    };

    fn make_signed_tx(sender_key: &ed25519_dalek::SigningKey, nonce: u64) -> SignedTransaction {
        let addr = pubkey_to_address_hex(&sender_key.verifying_key());
        let tx = Transaction::new(addr, "receiver".to_string(), 100, 1, nonce, None);
        SignedTransaction::sign_with_keypair(&tx, sender_key)
    }

    #[test]
    fn test_empty_merkle_root() {
        let root = compute_merkle_root(&[]);
        assert_eq!(root, "");
    }

    #[test]
    fn test_single_tx_merkle_root() {
        let key = generate_ed25519_keypair();
        let tx = make_signed_tx(&key, 0);
        let root = compute_merkle_root(&[tx.clone()]);
        assert!(!root.is_empty());
        assert_eq!(root.len(), 64); // SHA-256 hex = 64 chars

        // Same tx should always produce same root
        let root2 = compute_merkle_root(&[tx]);
        assert_eq!(root, root2);
    }

    #[test]
    fn test_merkle_root_changes_with_different_txs() {
        let key = generate_ed25519_keypair();
        let tx1 = make_signed_tx(&key, 0);
        let tx2 = make_signed_tx(&key, 1);

        let root_a = compute_merkle_root(&[tx1.clone()]);
        let root_b = compute_merkle_root(&[tx2.clone()]);
        let root_ab = compute_merkle_root(&[tx1, tx2]);

        assert_ne!(root_a, root_b);
        assert_ne!(root_a, root_ab);
        assert_ne!(root_b, root_ab);
    }

    #[test]
    fn test_block_verify_merkle_root() {
        let key = generate_ed25519_keypair();
        let tx = make_signed_tx(&key, 0);
        let block = Block::new(1, vec![tx], "0000".to_string(), "validator1".to_string());

        assert!(block.verify_merkle_root());
    }

    #[test]
    fn test_block_hash_determinism() {
        // Two blocks with same params at same timestamp should have same hash
        let ts = Utc::now();
        let h1 = Block::calculate_hash(1, &ts, "merkle", "prev", "val");
        let h2 = Block::calculate_hash(1, &ts, "merkle", "prev", "val");
        assert_eq!(h1, h2);

        // Changing any field changes the hash
        let h3 = Block::calculate_hash(2, &ts, "merkle", "prev", "val");
        assert_ne!(h1, h3);
    }
}
