// src/chain/block.rs
//! Block structure with Merkle root commitment over transactions

use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::transaction::{pubkey_to_address_hex, SignedTransaction};

/// Proposer name used by the genesis block (which is not signed).
pub const GENESIS_VALIDATOR: &str = "genesis";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub index: u64,
    pub timestamp: DateTime<Utc>,
    /// Merkle root of transaction hashes (hex). Empty string if no transactions.
    pub merkle_root: String,
    /// The transactions included in this block.
    pub transactions: Vec<SignedTransaction>,
    /// On-chain address of the proposer (hex(SHA-256(pubkey)[0..20])).
    pub validator: String,
    /// Proposer's Ed25519 public key (hex). Must hash to `validator`.
    #[serde(default)]
    pub proposer_pubkey: String,
    /// For block 0: SHA-256 of the genesis spec. Otherwise the parent block hash.
    pub previous_hash: String,
    pub hash: String,
    /// Proposer's Ed25519 signature over `hash` (hex).
    #[serde(default)]
    pub signature: String,
}

impl Block {
    /// Genesis block committing to the genesis spec hash.
    pub fn genesis(spec_hash: String, timestamp: DateTime<Utc>) -> Self {
        Self::new_unsigned(
            0,
            Vec::new(),
            spec_hash,
            GENESIS_VALIDATOR.to_string(),
            timestamp,
        )
    }

    /// Create an unsigned block (genesis and tests only; the network rejects unsigned blocks).
    pub fn new(
        index: u64,
        transactions: Vec<SignedTransaction>,
        previous_hash: String,
        validator: String,
    ) -> Self {
        Self::new_unsigned(index, transactions, previous_hash, validator, Utc::now())
    }

    /// Create an unsigned block at a specific timestamp.
    pub fn new_unsigned(
        index: u64,
        transactions: Vec<SignedTransaction>,
        previous_hash: String,
        validator: String,
        timestamp: DateTime<Utc>,
    ) -> Self {
        let merkle_root = compute_merkle_root(&transactions);
        let hash = Self::calculate_hash(
            index,
            &timestamp,
            &merkle_root,
            transactions.len(),
            &previous_hash,
            &validator,
            "",
        );
        Self {
            index,
            timestamp,
            merkle_root,
            transactions,
            validator,
            proposer_pubkey: String::new(),
            previous_hash,
            hash,
            signature: String::new(),
        }
    }

    /// Create a block signed by `key`; the proposer address is derived from the key.
    pub fn new_signed(
        index: u64,
        transactions: Vec<SignedTransaction>,
        previous_hash: String,
        timestamp: DateTime<Utc>,
        key: &SigningKey,
    ) -> Self {
        let verifying = key.verifying_key();
        let validator = pubkey_to_address_hex(&verifying);
        let proposer_pubkey = hex::encode(verifying.to_bytes());
        let merkle_root = compute_merkle_root(&transactions);
        let hash = Self::calculate_hash(
            index,
            &timestamp,
            &merkle_root,
            transactions.len(),
            &previous_hash,
            &validator,
            &proposer_pubkey,
        );
        let signature = hex::encode(key.sign(hash.as_bytes()).to_bytes());
        Self {
            index,
            timestamp,
            merkle_root,
            transactions,
            validator,
            proposer_pubkey,
            previous_hash,
            hash,
            signature,
        }
    }

    /// Hash of the header fields. Transactions are committed via the Merkle root and count
    /// (the count prevents the duplicated-last-leaf Merkle mutation).
    pub fn calculate_hash(
        index: u64,
        timestamp: &DateTime<Utc>,
        merkle_root: &str,
        tx_count: usize,
        previous_hash: &str,
        validator: &str,
        proposer_pubkey: &str,
    ) -> String {
        let payload = serde_json::json!({
            "index": index,
            "timestamp": timestamp.to_rfc3339(),
            "merkle_root": merkle_root,
            "tx_count": tx_count,
            "previous_hash": previous_hash,
            "validator": validator,
            "proposer_pubkey": proposer_pubkey,
        })
        .to_string();

        let mut hasher = Sha256::new();
        hasher.update(payload.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Recompute this block's hash from its header fields.
    pub fn compute_hash(&self) -> String {
        Self::calculate_hash(
            self.index,
            &self.timestamp,
            &self.merkle_root,
            self.transactions.len(),
            &self.previous_hash,
            &self.validator,
            &self.proposer_pubkey,
        )
    }

    /// Verify that the merkle_root matches the actual transactions in this block.
    pub fn verify_merkle_root(&self) -> bool {
        let computed = compute_merkle_root(&self.transactions);
        computed == self.merkle_root
    }

    /// Verify the proposer signature: the public key must derive `validator` and must have
    /// signed this block's hash.
    pub fn verify_signature(&self) -> Result<(), String> {
        let pubkey_bytes: [u8; 32] = hex::decode(&self.proposer_pubkey)
            .map_err(|_| "proposer_pubkey is not hex".to_string())?
            .try_into()
            .map_err(|_| "proposer_pubkey must be 32 bytes".to_string())?;
        let verifying = VerifyingKey::from_bytes(&pubkey_bytes)
            .map_err(|e| format!("invalid proposer_pubkey: {e}"))?;
        if pubkey_to_address_hex(&verifying) != self.validator {
            return Err("proposer_pubkey does not match validator address".into());
        }
        let sig_bytes: [u8; 64] = hex::decode(&self.signature)
            .map_err(|_| "signature is not hex".to_string())?
            .try_into()
            .map_err(|_| "signature must be 64 bytes".to_string())?;
        verifying
            .verify_strict(self.hash.as_bytes(), &Signature::from_bytes(&sig_bytes))
            .map_err(|_| "invalid block signature".to_string())
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
        let h1 = Block::calculate_hash(1, &ts, "merkle", 0, "prev", "val", "pk");
        let h2 = Block::calculate_hash(1, &ts, "merkle", 0, "prev", "val", "pk");
        assert_eq!(h1, h2);

        // Changing any field changes the hash
        let h3 = Block::calculate_hash(2, &ts, "merkle", 0, "prev", "val", "pk");
        assert_ne!(h1, h3);
        let h4 = Block::calculate_hash(1, &ts, "merkle", 1, "prev", "val", "pk");
        assert_ne!(h1, h4);
    }

    #[test]
    fn test_signed_block_verifies_and_detects_tampering() {
        let key = generate_ed25519_keypair();
        let block = Block::new_signed(
            1,
            vec![make_signed_tx(&key, 0)],
            "prev".into(),
            Utc::now(),
            &key,
        );
        assert_eq!(block.validator, pubkey_to_address_hex(&key.verifying_key()));
        assert!(block.verify_signature().is_ok());

        let mut forged = block.clone();
        forged.validator = "f".repeat(40);
        assert!(forged.verify_signature().is_err());

        let mut resigned = block.clone();
        let other = generate_ed25519_keypair();
        resigned.signature = hex::encode(other.sign(block.hash.as_bytes()).to_bytes());
        assert!(resigned.verify_signature().is_err());

        let unsigned = Block::new(1, vec![], "prev".into(), "val".into());
        assert!(unsigned.verify_signature().is_err());
    }
}
