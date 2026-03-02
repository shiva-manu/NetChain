//! Transaction module for NetChain
//! - Transaction structure
//! - Signing (Ed25519) and verification
//! - Canonical serialization for signing/hashing
//! - Transaction hashing (SHA-256)

use base64::{engine::general_purpose, Engine as _};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Transaction {
    pub sender: String,
    pub receiver: String,
    pub amount: u64,
    pub fee: u64,
    pub nonce: u64,
    pub timestamp: u64,
    pub memo: Option<String>,
}

#[derive(Debug, Clone)]
pub struct KeyPair {
    signing_key: SigningKey,
    pub public: VerifyingKey,
}

impl Transaction {
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

        Self {
            sender,
            receiver,
            amount,
            fee,
            nonce,
            timestamp,
            memo,
        }
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("transaction serialization should succeed")
    }

    pub fn tx_hash_hex(&self) -> String {
        let bytes = self.canonical_bytes();
        bytes_to_hex(&Sha256::digest(bytes))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedTransaction {
    pub tx: Transaction,
    pub signature: String,
    pub pubkey: String,
}

impl SignedTransaction {
    pub fn sign_with_keypair(tx: &Transaction, keypair: &KeyPair) -> Self {
        let msg = tx.canonical_bytes();
        let signature: Signature = keypair.signing_key.sign(&msg);

        Self {
            tx: tx.clone(),
            signature: general_purpose::STANDARD.encode(signature.to_bytes()),
            pubkey: general_purpose::STANDARD.encode(keypair.public.to_bytes()),
        }
    }

    pub fn verify(&self) -> Result<(), String> {
        let msg = self.tx.canonical_bytes();

        let sig_bytes = general_purpose::STANDARD
            .decode(&self.signature)
            .map_err(|e| format!("invalid signature base64: {e}"))?;
        let pk_bytes = general_purpose::STANDARD
            .decode(&self.pubkey)
            .map_err(|e| format!("invalid pubkey base64: {e}"))?;

        let signature = Signature::try_from(sig_bytes.as_slice())
            .map_err(|e| format!("invalid signature bytes: {e}"))?;
        let public_key = VerifyingKey::try_from(pk_bytes.as_slice())
            .map_err(|e| format!("invalid pubkey bytes: {e}"))?;

        public_key
            .verify(&msg, &signature)
            .map_err(|e| format!("signature verification failed: {e}"))?;

        Ok(())
    }

    pub fn tx_hash_hex(&self) -> String {
        self.tx.tx_hash_hex()
    }
}

pub fn generate_ed25519_keypair() -> KeyPair {
    let signing_key = SigningKey::generate(&mut OsRng);
    let public = signing_key.verifying_key();
    KeyPair {
        signing_key,
        public,
    }
}

pub fn pubkey_to_address_hex(pubkey: &VerifyingKey) -> String {
    let hash = Sha256::digest(pubkey.to_bytes());
    bytes_to_hex(&hash[..20])
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tx_sign_and_verify_roundtrip() {
        let keypair = generate_ed25519_keypair();
        let addr = pubkey_to_address_hex(&keypair.public);

        let tx = Transaction::new(
            addr,
            "receiver_address_example".to_string(),
            1_000,
            10,
            0,
            Some("test payment".to_string()),
        );

        let signed = SignedTransaction::sign_with_keypair(&tx, &keypair);
        assert!(signed.verify().is_ok());
        assert_eq!(tx.tx_hash_hex(), signed.tx_hash_hex());

        let mut bad = signed.clone();
        bad.tx.amount = 999_999;
        assert!(bad.verify().is_err());
    }

    #[test]
    fn address_derivation_and_consistency() {
        let keypair = generate_ed25519_keypair();
        let addr = pubkey_to_address_hex(&keypair.public);
        assert_eq!(addr.len(), 40);
    }
}
