// src/chain/transaction.rs

//! Transaction module for NetChain
//! - Transaction structure
//! - Signing (Ed25519) and verification
//! - Deterministic canonical serialization for signing (bincode)
//! - Transaction hashing (SHA-256)

//!
//! Usage:
//! - Build a `Transaction` (Without signature), compute hash, then sign using a Keypair
//! - Create a `SignedTransaction` that carries signature + public key
//! - Verify with `SignedTransaction::verify();

use base64::{engine::general_purpose, Engine as _};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

/// The core transcation structure (unsigned).
/// Keep fields small and canonical. We avoid fields that very in serialization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransactionType {
    Transfer,
    Stake,
    Unstake,
    CreateProposal {
        title: String,
        description: String,
        voting_period_secs: u64,
        /// Optional action to execute if the proposal passes
        #[serde(default)]
        action: Option<ProposalAction>,
    },
    VoteProposal {
        proposal_id: u64,
        support: bool,
    },
}

/// Actions that a governance proposal can execute when it passes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProposalAction {
    /// Change the block reward (tokens minted per block)
    ChangeBlockReward(u64),
    /// Change the block production interval in seconds
    ChangeBlockInterval(u64),
    /// Change the maximum transactions per block
    ChangeMaxTxsPerBlock(usize),
    /// Change the stake weight for validator selection (stored as basis points, e.g. 3000 = 0.30)
    ChangeStakeWeight(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Transaction {
    /// Sender address (string representation of public key hash / address)\
    pub sender: String,
    /// Receiver address
    pub receiver: String,
    /// Amount in smallest unit (u64)
    pub amount: u64,
    /// Fee paid to validtors (u64)
    pub fee: u64,
    /// Nonce for replay protection
    pub nonce: u64,
    /// Unix timestamp (seconds) when tx created
    pub timestamp: u64,
    /// Transaction type
    pub tx_type: TransactionType,
    /// Optional memo/data
    pub memo: Option<String>,
}

impl Transaction {
    // Create a new unsigned transaction (timestamp auto-filled)
    pub fn new(
        sender: String,
        receiver: String,
        amount: u64,
        fee: u64,
        nonce: u64,
        memo: Option<String>,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Transaction {
            sender,
            receiver,
            amount,
            fee,
            nonce,
            timestamp,
            tx_type: TransactionType::Transfer,
            memo,
        }
    }

    pub fn stake(sender: String, amount: u64, fee: u64, nonce: u64) -> Self {
        let mut tx = Self::new(
            sender,
            String::new(),
            amount,
            fee,
            nonce,
            Some("stake".into()),
        );
        tx.tx_type = TransactionType::Stake;
        tx
    }

    pub fn unstake(sender: String, amount: u64, fee: u64, nonce: u64) -> Self {
        let mut tx = Self::new(
            sender,
            String::new(),
            amount,
            fee,
            nonce,
            Some("unstake".into()),
        );
        tx.tx_type = TransactionType::Unstake;
        tx
    }

    pub fn create_proposal(
        sender: String,
        fee: u64,
        nonce: u64,
        title: String,
        description: String,
        voting_period_secs: u64,
    ) -> Self {
        let mut tx = Self::new(
            sender,
            String::new(),
            0,
            fee,
            nonce,
            Some("create_proposal".into()),
        );
        tx.tx_type = TransactionType::CreateProposal {
            title,
            description,
            voting_period_secs,
            action: None,
        };
        tx
    }

    /// Create a proposal with an executable action
    pub fn create_proposal_with_action(
        sender: String,
        fee: u64,
        nonce: u64,
        title: String,
        description: String,
        voting_period_secs: u64,
        action: ProposalAction,
    ) -> Self {
        let mut tx = Self::new(
            sender,
            String::new(),
            0,
            fee,
            nonce,
            Some("create_proposal".into()),
        );
        tx.tx_type = TransactionType::CreateProposal {
            title,
            description,
            voting_period_secs,
            action: Some(action),
        };
        tx
    }

    pub fn vote_proposal(
        sender: String,
        fee: u64,
        nonce: u64,
        proposal_id: u64,
        support: bool,
    ) -> Self {
        let mut tx = Self::new(
            sender,
            String::new(),
            0,
            fee,
            nonce,
            Some("vote_proposal".into()),
        );
        tx.tx_type = TransactionType::VoteProposal {
            proposal_id,
            support,
        };
        tx
    }

    /// Produce deterministic bytes for signing / hashing
    /// Uses bincode serialization ( Compact + deterministic)
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let config = bincode::config::standard()
            .with_fixed_int_encoding() // Ensure u64 always takes 8 bytes
            .with_little_endian(); // Explicit byte order
        bincode::serde::encode_to_vec(self, config)
            .expect("bincode serialization should succeed for Transaction")
    }

    /// Compute SHA-256 hash of canonical bytes -> hex string
    pub fn tx_hash_hex(&self) -> String {
        let bytes = self.canonical_bytes();
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let result = hasher.finalize();
        hex::encode(result)
    }
}

/// SignedTransaction:include the serialized Transaction plus the signature and public key
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedTransaction {
    pub tx: Transaction,
    /// Signature encoded as base64
    pub signature: String,
    /// Public key encoded as base64(ed25519 public key bytes)
    pub pubkey: String,
}

impl SignedTransaction {
    /// Construct a SignedTransaction from a transaction and an ed25519 signing key
    pub fn sign_with_keypair(tx: &Transaction, signing_key: &SigningKey) -> Self {
        let msg = tx.canonical_bytes();
        let sig: Signature = signing_key.sign(&msg);
        SignedTransaction {
            tx: tx.clone(),
            signature: general_purpose::STANDARD.encode(sig.to_bytes()),
            pubkey: general_purpose::STANDARD.encode(signing_key.verifying_key().to_bytes()),
        }
    }

    /// Verify signature and pubkey match the transaction
    pub fn verify(&self) -> Result<(), String> {
        // decode signature & pubkey
        let sig_bytes = general_purpose::STANDARD
            .decode(&self.signature)
            .map_err(|e| format!("Invalid signature base64: {}", e))?;
        let pk_bytes = general_purpose::STANDARD
            .decode(&self.pubkey)
            .map_err(|e| format!("Invalid pubkey base64: {}", e))?;

        let sig_array: [u8; 64] = sig_bytes
            .as_slice()
            .try_into()
            .map_err(|_| "Invalid signature length")?;
        let pk_array: [u8; 32] = pk_bytes
            .as_slice()
            .try_into()
            .map_err(|_| "Invalid pubkey length")?;

        let signature = Signature::from_bytes(&sig_array);
        let public_key = VerifyingKey::from_bytes(&pk_array)
            .map_err(|e| format!("Invalid pubkey bytes: {}", e))?;

        // Verify that the claimed sender address matches the public key used for signing.
        // The address is derived as hex(SHA-256(pubkey)[0..20]).
        let derived_addr = pubkey_to_address_hex(&public_key);
        if derived_addr != self.tx.sender {
            return Err(format!(
                "Sender address mismatch: tx.sender={} but signing pubkey derives={}",
                self.tx.sender, derived_addr
            ));
        }

        // verify signature
        let msg = self.tx.canonical_bytes();
        public_key
            .verify(&msg, &signature)
            .map_err(|e| format!("signature verification failed: {}", e))?;
        Ok(())
    }

    /// Get SHA-256 tx hash (hex) from inner transaction
    pub fn tx_hash_hex(&self) -> String {
        self.tx.tx_hash_hex()
    }
}

/// Helper: generate an Ed25519 signing key
pub fn generate_ed25519_keypair() -> SigningKey {
    SigningKey::generate(&mut OsRng)
}

///OPTIONAL: helper to produce an address string from public key bytes
///Here we use SHA-256 of public and hex encode first 20 bytes (like an address)
pub fn pubkey_to_address_hex(pubkey: &VerifyingKey) -> String {
    let mut hasher = Sha256::new();
    hasher.update(pubkey.to_bytes());
    let res = hasher.finalize();
    // take first 20 bytes and hex encode (40 hex chars)
    hex::encode(&res[0..20])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tx_sign_and_verify_flow() {
        // generate signing key
        let signing_key = generate_ed25519_keypair();

        // derive address from pubkey
        let addr = pubkey_to_address_hex(&signing_key.verifying_key());

        //build tx
        let tx = Transaction::new(
            addr.clone(),
            "receiver_address_example".to_string(),
            1_000u64,
            10u64,
            0u64,
            Some("test payment".to_string()),
        );

        //Sign
        let signed = SignedTransaction::sign_with_keypair(&tx, &signing_key);

        // quick sanity: pubkey encoded should match
        let pk_decoded = general_purpose::STANDARD.decode(&signed.pubkey).unwrap();
        assert_eq!(
            pk_decoded.as_slice(),
            signing_key.verifying_key().to_bytes().as_slice()
        );

        // verify
        let res = signed.verify();
        assert!(res.is_ok());

        // tx hash stable
        let h1 = tx.tx_hash_hex();
        let h2 = signed.tx_hash_hex();
        assert_eq!(h1, h2);

        // changing tx should make verification fail
        let mut bad = signed.clone();
        bad.tx.amount = 999999;
        assert!(bad.verify().is_err());
    }

    #[test]
    fn address_derivation_and_consistency() {
        let signing_key = generate_ed25519_keypair();
        let addr = pubkey_to_address_hex(&signing_key.verifying_key());
        assert_eq!(addr.len(), 40); // 20 bytes -> 40 chars
    }

    #[test]
    fn tx_verify_rejects_sender_mismatch() {
        let signing_key = generate_ed25519_keypair();

        // Build tx with a FAKE sender address (not derived from signing key)
        let tx = Transaction::new(
            "0000000000000000000000000000000000000000".to_string(),
            "receiver_address_example".to_string(),
            1_000u64,
            10u64,
            0u64,
            None,
        );

        let signed = SignedTransaction::sign_with_keypair(&tx, &signing_key);

        // Verification should fail because sender != derived address from pubkey
        let res = signed.verify();
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Sender address mismatch"));
    }
}
