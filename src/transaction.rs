// src/transaction.rs

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

use base64::{engine::general_purpose,Engine as _};
use bincode;
use ed25519_dalek::{Keypair,PublicKey,Signature,Signer,Verifier};
use rand::rngs::OsRng;
use serde::{Deserialize,Serialize};
use sha2::{Digest,Sha256};
use std::time::{SystemTime,UNIX_EPOCH};

/// The core transcation structure (unsigned).
/// Keep fields small and canonical. We avoid fields that very in serialization
#[derive(Debug,Clone,Serialize,Deserialize,PartialEq,Eq)]
pub struct Transaction{
    /// Sender address (string representation of public key hash / address)\
    pub sender:String,
    /// Receiver address
    pub receiver:String,
    /// Amount in smallest unit (u64)
    pub amount:u64,
    /// Fee paid to validtors (u64)
    pub fee:u64,
    /// Nonce for replay protection
    pub nonce:u64,
    /// Unix timestamp (seconds) when tx created
    pub timestamp:u64,
    /// Optional memo/data
    pub memo:Option<String>,
}

impl Transaction{
    // Create a new unsigned transaction (timestamp auto-filled)
    pub fn new(sender:String,receiver:String,amount:u64,fee:u64,nonce:u64,memo:Option<String>)->Self{
        let timestamp=SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
        Transaction{
            sender,
            receiver,
            amount,
            fee,
            nonce,
            timestamp,
            memo
        }
    }

    /// Produce deterministic bytes for signing / hashing
    /// Uses bincode serialization ( Compact + deterministic)
    pub fn canonical_bytes(&self)->Vec<u8>{
        // We rely on bincode default options which are deterministic for primitive types
        // Avoid Option<> variants chainging ordering by serializing the struct as-is.
        bincode::DefaultOptions::new()
        .with_fixint_encoding()      // Ensure u64 always takes 8 bytes
        .with_little_endian()        // Explicit byte order
        .serialize(self)
        .expect("bincode serialization should succed for Transaction")
    }

    /// Compute SHA-256 hash of canonical bytes -> hex string
    pub fn tx_hash_hex(&self)->String{
        let bytes=self.canonical_bytes();
        let mut hasher=Sha256::new();
        hasher.update(&bytes);
        let result=hasher.finalize();
        hex::encode(result)
    }
}

/// SignedTransaction:include the serialized Transaction plus the signature and public key
#[derive(Debug,Clone,Serialize,Deserialize,PartialEq,Eq)]
pub struct SignedTransaction{
    pub tx:Transaction,
    /// Signature encoded as base64
    pub signature:String,
    /// Public key encoded as base64(ed25519 public key bytes)
    pub pubkey:String,
}

impl SignedTransaction{
    /// Construct a SignedTransaction from a transaction and an ed25519 keypair
    pub fn sign_with_keypair(tx:&Transaction,keypair:&Keypair)->Self{
        let msg=tx.canonical_bytes();
        let sig:Signature=keypair.sign(&msg);
        SignedTransaction{
            tx:tx.clone(),
            signature:general_purpose::STANDARD.encode(sig.to_bytes()),
            pubkey:general_purpose::STANDARD.encode(keypair.public.to_bytes())
        }
    }

    /// Verify signature and pubkey match the transaction
    pub fn verify(&self)->Result<(),String>{
        // decode signature & pubkey
        let sig_bytes=general_purpose::STANDARD
        .decode(&self.signature)
        .map_err(|e| format!("Invalid signature base64: {}",e))?;
        let pk_bytes=general_purpose::STANDARD
        .decode(&self.pubkey)
        .map_err(|e| format!("Invalid pubkey base64: {}",e))?;

        let signature=Signature::from_bytes(&sig_bytes).map_err(|e| format!("Invalid signature bytes: {}"))?;
        let public_key=PublicKey::from_bytes(&pk_bytes).map_err(|e| format!("Invalid pubkey bytes: {}"))?;

        // Verify that the claimed sender address matches public key (Optional mapping)
        // NOTE: Here we assume sender is hex(pubkey_hash) or base64(pubkey).The address schema is up to you
        // We'll skip strict address match; projects usually derive an address from pubkey (e.g., hash).
        // If your sender field is designed as the hex of pubkey hash, verify accordingly.
        // Example check (optional):
        // let derived_addr = pubkey_to_address(&public_key);
        // if derived_addr != self.tx.sender { return Err("sender mismatch".into()); }

        // verify signature
        let msg=self.tx.canonical_bytes();
        public_key
        .verify(&msg,&signature)
        .map_err(|e| format!("signature verification failed: {}",e))?;
        Ok(())
    }

    /// Get SHA-256 tx hash (hex) from inner transaction
    pub fn tx_hash_hex(&self)->String{
        self.tx.tx_hash_hex()
    }
}

/// Helper: generate an Ed25519 keypair (keypair contains both secret & public)
pub fn generate_ed25519_keypair()->Keypair{
    let mut cspring=OsRng{};
    Keypair::generate(&mut cspring)
}

///OPTIONAL: helper to produce an address string from public key bytes
///Here we use SHA-256 of public and hex encode first 20 bytes (like an address)
pub fn pubkey_to_address_hex(pubkey:&PublicKey)->String{
    let mut hasher=Sha256::new();
    hasher.update(pubkey.to_bytes());
    let res=hasher.finalize();
    // take first 20 bytes and hex encode (40 hex chars)
    hex::encode(&res[0..20])
}

#[cfg(test)]
mod tests{
    use super::*;
    use ed25519_dalek::Keypair;
    use rand::rngs::OsRng;

    #[test]
    fn tx_sign_and_verify_flow(){
        // generate keypair
        let mut cspring=OsRng{};
        let keypair:Keypair=Keypair::generate(&mut cspring);

        // derive address from pubkey
        let addr=pubkey_to_address_hex(&keypair.public);

        //build tx
        let tx=Transaction::new(
            addr.clone(),
            "receiver_address_example".to_string(),
            1_000u64,
            10u64,
            0u64,
            Some("test payment".to_string()),
        );

        //Sign
        let signed=SignedTransaction::sign_with_keypair(&tx,&keypair);

        // quick sanity: pubkey encoded should match
        let pk_decoded=general_purpose::STANDARD.decode(&signed.pubkey).unwrap();
        assert_eq!(pk_decoded,keypair.public.to_bytes());

        // verify
        let res=signed.verify();
        assert!(res.is_ok());

        // tx hash stable
        let h1=tx.tx_hash_hex();
        let h2=signed.tx_hash_hex();
        assert_eq!(h1,h2);

        // changing tx should make verification fail
        let mut bad=signed.clone();
        bad.tx.amount=999999;
        assert!(bad.verify().is_err());
    }

    #[test]
    fn address_derivation_and_consistency(){
        let keypair=generate_ed25519_keypair();
        let addr=pubkey_to_address_hex(&keypair.public);
        assert_eq!(addr.len(),40);  // 20 bytes -> 40 chars
    }
}