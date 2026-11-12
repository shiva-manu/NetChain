// src/chain/blockchain.rs
//! The canonical block list plus structural (stateless) block checks.
//!
//! Consensus rules that need state (proposer schedule, slot timing, transaction execution)
//! live in `engine`; fork choice lives in the node's chain service.

use crate::block::Block;
use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
}

impl Default for Blockchain {
    fn default() -> Self {
        Self::new()
    }
}

impl Blockchain {
    /// A chain with a fixed test genesis block. Real nodes use `with_genesis`.
    pub fn new() -> Self {
        let ts = Utc
            .timestamp_opt(0, 0)
            .single()
            .expect("valid genesis timestamp");
        Self::with_genesis(Block::genesis("0".to_string(), ts))
    }

    pub fn with_genesis(genesis: Block) -> Self {
        Blockchain {
            chain: vec![genesis],
        }
    }

    pub fn genesis(&self) -> &Block {
        &self.chain[0]
    }

    pub fn last_block(&self) -> &Block {
        self.chain.last().expect("chain always has genesis")
    }

    pub fn height(&self) -> u64 {
        self.last_block().index
    }

    pub fn get_block(&self, index: u64) -> Option<&Block> {
        self.chain.get(index as usize)
    }

    pub fn blocks_paginated(&self, start_height: u64, limit: usize) -> Vec<&Block> {
        self.chain
            .iter()
            .skip(start_height as usize)
            .take(limit)
            .collect()
    }

    /// Blocks in `[from_height, from_height + limit)`.
    pub fn get_blocks_range(&self, from_height: u64, limit: usize) -> Vec<&Block> {
        self.blocks_paginated(from_height, limit)
    }

    /// Hash of our block at `index`, if we have it.
    pub fn hash_at(&self, index: u64) -> Option<&str> {
        self.get_block(index).map(|b| b.hash.as_str())
    }
}

/// Stateless checks of `block` as the child of `parent`: linkage, hash, Merkle root,
/// proposer signature, transaction signatures and size.
pub fn validate_block_structure(parent: &Block, block: &Block) -> Result<(), String> {
    if block.index != parent.index + 1 {
        return Err(format!(
            "Invalid index: expected {}, got {}",
            parent.index + 1,
            block.index
        ));
    }

    if block.previous_hash != parent.hash {
        return Err("Invalid previous hash".into());
    }

    if block.timestamp < parent.timestamp {
        return Err("Invalid timestamp: must be >= previous block timestamp".into());
    }

    if block.timestamp.timestamp() < 0 {
        return Err("Invalid timestamp: before unix epoch".into());
    }

    if block.hash != block.compute_hash() {
        return Err("Invalid block hash".into());
    }

    if !block.verify_merkle_root() {
        return Err("Invalid merkle root: does not match transactions".into());
    }

    block
        .verify_signature()
        .map_err(|e| format!("Invalid block signature: {e}"))?;

    for tx in &block.transactions {
        if let Err(e) = tx.verify() {
            return Err(format!("Block contains invalid transaction: {}", e));
        }
    }

    let config = bincode::config::standard()
        .with_fixed_int_encoding()
        .with_little_endian();
    let tx_total_bytes: usize = block
        .transactions
        .iter()
        .map(|tx| {
            bincode::serde::encode_to_vec(tx, config)
                .map(|v| v.len())
                .unwrap_or(usize::MAX / 2)
        })
        .sum();
    if tx_total_bytes > crate::mempool::MAX_BLOCK_SIZE_BYTES {
        return Err(format!(
            "Block too large: {} bytes exceeds maximum {} bytes",
            tx_total_bytes,
            crate::mempool::MAX_BLOCK_SIZE_BYTES
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{
        generate_ed25519_keypair, pubkey_to_address_hex, SignedTransaction, Transaction,
    };
    use chrono::Duration;

    fn signed_child(parent: &Block, key: &ed25519_dalek::SigningKey) -> Block {
        Block::new_signed(
            parent.index + 1,
            vec![],
            parent.hash.clone(),
            parent.timestamp + Duration::seconds(15),
            key,
        )
    }

    #[test]
    fn test_genesis_block() {
        let bc = Blockchain::new();
        assert_eq!(bc.chain.len(), 1);
        assert_eq!(bc.last_block().index, 0);
    }

    #[test]
    fn test_accepts_signed_child_with_transactions() {
        let key = generate_ed25519_keypair();
        let mut bc = Blockchain::new();
        let addr = pubkey_to_address_hex(&key.verifying_key());
        let tx = Transaction::new(addr, "receiver".into(), 100, 1, 0, None);
        let signed = SignedTransaction::sign_with_keypair(&tx, &key);
        let block = Block::new_signed(
            1,
            vec![signed],
            bc.last_block().hash.clone(),
            bc.last_block().timestamp + Duration::seconds(15),
            &key,
        );
        validate_block_structure(bc.last_block(), &block).unwrap();
        bc.chain.push(block);
        assert_eq!(bc.height(), 1);
    }

    #[test]
    fn test_rejects_invalid_index_link_and_signature() {
        let key = generate_ed25519_keypair();
        let bc = Blockchain::new();

        let mut wrong_index = signed_child(bc.last_block(), &key);
        wrong_index.index = 5;
        assert!(validate_block_structure(bc.last_block(), &wrong_index).is_err());

        let wrong_parent = Block::new_signed(
            1,
            vec![],
            "deadbeef".into(),
            bc.last_block().timestamp,
            &key,
        );
        assert!(validate_block_structure(bc.last_block(), &wrong_parent).is_err());

        let unsigned = Block::new_unsigned(
            1,
            vec![],
            bc.last_block().hash.clone(),
            "validator".into(),
            bc.last_block().timestamp,
        );
        assert!(validate_block_structure(bc.last_block(), &unsigned).is_err());
    }

    #[test]
    fn test_rejects_tampered_transactions() {
        let key = generate_ed25519_keypair();
        let bc = Blockchain::new();
        let addr = pubkey_to_address_hex(&key.verifying_key());
        let tx = Transaction::new(addr, "receiver".into(), 100, 1, 0, None);
        let signed = SignedTransaction::sign_with_keypair(&tx, &key);
        let mut block = Block::new_signed(
            1,
            vec![signed],
            bc.last_block().hash.clone(),
            bc.last_block().timestamp,
            &key,
        );
        block.transactions[0].tx.amount = 1_000_000;
        assert!(validate_block_structure(bc.last_block(), &block).is_err());
    }
}
