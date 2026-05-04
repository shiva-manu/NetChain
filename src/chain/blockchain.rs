// src/chain/blockchain.rs

use crate::block::Block;
use crate::transaction::SignedTransaction;
use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};

/// Result of a chain sync operation
#[derive(Debug)]
pub struct SyncResult {
    /// Number of blocks added to the chain
    pub added: u64,
    /// Whether a chain reorganization occurred (blocks were replaced)
    pub reorged: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
}

impl Blockchain {
    pub fn new() -> Self {
        let mut bc = Blockchain { chain: Vec::new() };
        bc.chain.push(Self::genesis_block());
        bc
    }

    fn genesis_block() -> Block {
        // Deterministic genesis timestamp so all nodes share the same genesis hash.
        let ts = Utc
            .timestamp_opt(0, 0)
            .single()
            .expect("valid genesis timestamp");
        Block::new_at(
            0,
            Vec::new(), // no transactions in genesis
            "0".to_string(),
            "genesis".to_string(),
            ts,
            String::new(), // genesis has no VRF proof
        )
    }

    pub fn last_block(&self) -> &Block {
        self.chain
            .last()
            .expect("Blockchain must have at least one block")
    }

    /// Used by local block producer / validator
    pub fn add_block(&mut self, transactions: Vec<SignedTransaction>, validator: String) -> Block {
        let last = self.last_block();
        let new_block = Block::new(last.index + 1, transactions, last.hash.clone(), validator);
        self.chain.push(new_block.clone());
        new_block
    }

    /// Validate a candidate block against our current tip (does not mutate the chain).
    pub fn validate_next_block(&self, block: &Block) -> Result<(), String> {
        let last = self.last_block();

        if block.index != last.index + 1 {
            return Err(format!(
                "Invalid index: expected {}, got {}",
                last.index + 1,
                block.index
            ));
        }

        if block.previous_hash != last.hash {
            return Err("Invalid previous hash".into());
        }

        // Enforce non-decreasing timestamps for deterministic time-based rules.
        if block.timestamp < last.timestamp {
            return Err("Invalid timestamp: must be >= previous block timestamp".into());
        }

        if block.timestamp.timestamp() < 0 {
            return Err("Invalid timestamp: before unix epoch".into());
        }

        // Verify the block hash (includes VRF proof in the hash)
        let recalculated = Block::calculate_hash(
            block.index,
            &block.timestamp,
            &block.merkle_root,
            &block.previous_hash,
            &block.validator,
            &block.vrf_proof,
        );

        if block.hash != recalculated {
            return Err("Invalid block hash".into());
        }

        // Verify the Merkle root matches the included transactions
        if !block.verify_merkle_root() {
            return Err("Invalid merkle root: does not match transactions".into());
        }

        // Verify VRF proof (validator's signature on previous block hash)
        if !block.verify_vrf_proof() {
            return Err("Invalid VRF proof: validator signature verification failed".into());
        }

        // Verify all transaction signatures
        for tx in &block.transactions {
            if let Err(e) = tx.verify() {
                return Err(format!("Block contains invalid transaction: {}", e));
            }
        }

        // Enforce maximum block size (serialized transactions)
        let config = bincode::config::standard()
            .with_fixed_int_encoding()
            .with_little_endian();
        let tx_total_bytes: usize = block
            .transactions
            .iter()
            .map(|tx| {
                bincode::serde::encode_to_vec(tx, config)
                    .unwrap_or_default()
                    .len()
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

    /// Used when receiving blocks from P2P -- validates before adding
    pub fn validate_and_add_block(&mut self, block: Block) -> Result<(), String> {
        self.validate_next_block(&block)?;
        self.chain.push(block);
        Ok(())
    }

    pub fn is_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current = &self.chain[i];
            let previous = &self.chain[i - 1];

            if current.previous_hash != previous.hash {
                return false;
            }

            if current.timestamp < previous.timestamp {
                return false;
            }

            let recalculated = Block::calculate_hash(
                current.index,
                &current.timestamp,
                &current.merkle_root,
                &current.previous_hash,
                &current.validator,
                &current.vrf_proof,
            );

            if current.hash != recalculated {
                return false;
            }

            if !current.verify_merkle_root() {
                return false;
            }

            for tx in &current.transactions {
                if tx.verify().is_err() {
                    return false;
                }
            }
        }
        true
    }

    /// Get the current chain height (index of last block)
    pub fn height(&self) -> u64 {
        self.last_block().index
    }

    pub fn get_block(&self, index: u64) -> Option<&Block> {
        self.chain.iter().find(|block| block.index == index)
    }

    pub fn blocks_paginated(&self, start_height: u64, limit: usize) -> Vec<&Block> {
        self.chain
            .iter()
            .filter(|block| block.index >= start_height)
            .take(limit)
            .collect()
    }

    /// Get blocks from a given height for chain sync
    pub fn get_blocks_from(&self, from_height: u64) -> Vec<&Block> {
        self.chain
            .iter()
            .filter(|b| b.index >= from_height)
            .collect()
    }

    /// Sync chain from received blocks.
    ///
    /// Implements the "longest valid chain" fork choice rule:
    /// 1. If the incoming blocks simply extend our chain, append them.
    /// 2. If the incoming chain is longer and forks from a common ancestor,
    ///    validate the entire fork and switch to it (chain reorganization).
    /// 3. If the incoming chain is shorter or equal, ignore it.
    ///
    /// Returns the number of blocks added, or an error if the chain is invalid.
    pub fn sync_from_blocks(&mut self, blocks: Vec<Block>) -> Result<SyncResult, String> {
        if blocks.is_empty() {
            return Ok(SyncResult {
                added: 0,
                reorged: false,
            });
        }

        // Sort incoming blocks by index
        let mut sorted_blocks = blocks;
        sorted_blocks.sort_by_key(|b| b.index);

        let incoming_max_height = sorted_blocks.last().unwrap().index;
        let our_height = self.height();

        // Case 1: Incoming chain doesn't extend beyond ours -- skip
        if incoming_max_height <= our_height {
            return Ok(SyncResult {
                added: 0,
                reorged: false,
            });
        }

        // Case 2: Simple extension -- incoming blocks continue from our tip
        // Try to find blocks that extend our chain directly
        let extension_blocks: Vec<Block> = sorted_blocks
            .iter()
            .filter(|b| b.index > our_height)
            .cloned()
            .collect();

        if !extension_blocks.is_empty() {
            // Check if the first extension block links to our current tip
            let first_ext = &extension_blocks[0];
            if first_ext.index == our_height + 1
                && first_ext.previous_hash == self.last_block().hash
            {
                // Simple append
                let mut added = 0u64;
                for block in extension_blocks {
                    self.validate_and_add_block(block)?;
                    added += 1;
                }
                return Ok(SyncResult {
                    added,
                    reorged: false,
                });
            }
        }

        // Case 3: Fork detected -- incoming chain diverges from ours.
        // Find the common ancestor (the fork point).
        let fork_point = self.find_fork_point(&sorted_blocks);

        // Build the candidate chain: our chain up to fork_point + incoming blocks from fork_point
        let candidate_blocks: Vec<Block> = sorted_blocks
            .into_iter()
            .filter(|b| b.index > fork_point)
            .collect();

        if candidate_blocks.is_empty() {
            return Ok(SyncResult {
                added: 0,
                reorged: false,
            });
        }

        // The candidate chain must be strictly longer than our current chain
        let candidate_tip = candidate_blocks.last().unwrap().index;
        if candidate_tip <= our_height {
            return Ok(SyncResult {
                added: 0,
                reorged: false,
            });
        }

        // Validate the candidate fork chain internally
        // First block of the fork must link to one of our blocks at fork_point
        let anchor_hash = if fork_point < self.chain.len() as u64 {
            self.chain[fork_point as usize].hash.clone()
        } else {
            return Err("Fork point beyond our chain".into());
        };

        if candidate_blocks[0].previous_hash != anchor_hash {
            return Err("Fork chain does not link to a known block".into());
        }

        // Validate each block in the candidate fork
        let anchor_ts = &self.chain[fork_point as usize].timestamp;
        Self::validate_chain_segment(&candidate_blocks, fork_point, &anchor_hash, anchor_ts)?;

        // Perform the reorganization: truncate our chain at fork point and append the new fork
        let reorged_count = self.chain.len() as u64 - fork_point - 1;
        self.chain.truncate((fork_point + 1) as usize);

        for block in &candidate_blocks {
            self.chain.push(block.clone());
        }

        Ok(SyncResult {
            added: candidate_blocks.len() as u64,
            reorged: reorged_count > 0,
        })
    }

    /// Find the highest block index that exists in both our chain and the incoming blocks.
    fn find_fork_point(&self, incoming_blocks: &[Block]) -> u64 {
        // Build a map of incoming block hashes by index
        let incoming_by_index: std::collections::HashMap<u64, &Block> =
            incoming_blocks.iter().map(|b| (b.index, b)).collect();

        // Walk backwards from our tip to find where chains agree
        for i in (0..self.chain.len()).rev() {
            let our_block = &self.chain[i];
            if let Some(incoming) = incoming_by_index.get(&our_block.index) {
                if incoming.hash == our_block.hash {
                    return our_block.index;
                }
            }
        }

        // If no match found, the genesis blocks must match (index 0)
        0
    }

    /// Validate a chain segment (sequence of blocks) given the hash of the block preceding it.
    fn validate_chain_segment(
        blocks: &[Block],
        prev_index: u64,
        prev_hash: &str,
        prev_timestamp: &chrono::DateTime<Utc>,
    ) -> Result<(), String> {
        let mut expected_prev_hash = prev_hash.to_string();
        let mut expected_prev_timestamp = prev_timestamp.clone();
        let mut expected_index = prev_index;

        for (i, block) in blocks.iter().enumerate() {
            if block.index != expected_index + 1 {
                return Err(format!(
                    "Invalid index at segment position {} (expected #{}, got #{})",
                    i,
                    expected_index + 1,
                    block.index
                ));
            }

            if block.previous_hash != expected_prev_hash {
                return Err(format!(
                    "Invalid previous_hash at segment position {} (block #{})",
                    i, block.index
                ));
            }

            if block.timestamp < expected_prev_timestamp {
                return Err(format!(
                    "Invalid timestamp at segment position {} (block #{})",
                    i, block.index
                ));
            }

            if block.timestamp.timestamp() < 0 {
                return Err(format!(
                    "Invalid timestamp before unix epoch (block #{})",
                    block.index
                ));
            }

            let recalculated = Block::calculate_hash(
                block.index,
                &block.timestamp,
                &block.merkle_root,
                &block.previous_hash,
                &block.validator,
                &block.vrf_proof,
            );

            if block.hash != recalculated {
                return Err(format!(
                    "Invalid block hash at segment position {} (block #{})",
                    i, block.index
                ));
            }

            if !block.verify_merkle_root() {
                return Err(format!(
                    "Invalid merkle root at segment position {} (block #{})",
                    i, block.index
                ));
            }

            for tx in &block.transactions {
                if let Err(e) = tx.verify() {
                    return Err(format!(
                        "Invalid transaction in block #{}: {}",
                        block.index, e
                    ));
                }
            }

            expected_prev_hash = block.hash.clone();
            expected_prev_timestamp = block.timestamp;
            expected_index = block.index;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{
        generate_ed25519_keypair, pubkey_to_address_hex, SignedTransaction, Transaction,
    };

    #[test]
    fn test_genesis_block() {
        let bc = Blockchain::new();
        assert_eq!(bc.chain.len(), 1);
        assert_eq!(bc.chain[0].index, 0);
        assert!(bc.chain[0].transactions.is_empty());
        assert!(bc.is_valid());
    }

    #[test]
    fn test_add_block_with_transactions() {
        let mut bc = Blockchain::new();
        let key = generate_ed25519_keypair();
        let addr = pubkey_to_address_hex(&key.verifying_key());

        let tx = Transaction::new(addr, "receiver".to_string(), 100, 1, 0, None);
        let signed = SignedTransaction::sign_with_keypair(&tx, &key);

        let block = bc.add_block(vec![signed], "validator1".to_string());
        assert_eq!(block.index, 1);
        assert_eq!(block.transactions.len(), 1);
        assert!(block.verify_merkle_root());
        assert!(bc.is_valid());
    }

    #[test]
    fn test_validate_and_add_block() {
        let mut bc = Blockchain::new();
        let prev_hash = bc.last_block().hash.clone();

        let block = Block::new(1, Vec::new(), prev_hash, "validator1".to_string());

        assert!(bc.validate_and_add_block(block).is_ok());
        assert_eq!(bc.height(), 1);
    }

    #[test]
    fn test_reject_invalid_index() {
        let mut bc = Blockchain::new();
        let prev_hash = bc.last_block().hash.clone();

        // Block with wrong index (5 instead of 1)
        let block = Block::new(5, Vec::new(), prev_hash, "validator1".to_string());

        assert!(bc.validate_and_add_block(block).is_err());
    }

    #[test]
    fn test_sync_simple_extension() {
        let mut bc = Blockchain::new();
        let genesis_hash = bc.last_block().hash.clone();

        // Build blocks externally that extend the chain
        let block1 = Block::new(1, Vec::new(), genesis_hash, "val".to_string());
        let block2 = Block::new(2, Vec::new(), block1.hash.clone(), "val".to_string());

        let result = bc.sync_from_blocks(vec![block1, block2]).unwrap();
        assert_eq!(result.added, 2);
        assert!(!result.reorged);
        assert_eq!(bc.height(), 2);
    }

    #[test]
    fn test_sync_ignores_shorter_chain() {
        let mut bc = Blockchain::new();

        // Add two blocks locally
        bc.add_block(Vec::new(), "val".to_string());
        bc.add_block(Vec::new(), "val".to_string());
        assert_eq!(bc.height(), 2);

        // Try to sync with a chain that only has block 1 (shorter)
        let genesis_hash = bc.chain[0].hash.clone();
        let alt_block1 = Block::new(1, Vec::new(), genesis_hash, "other".to_string());

        let result = bc.sync_from_blocks(vec![alt_block1]).unwrap();
        assert_eq!(result.added, 0);
        assert!(!result.reorged);
        assert_eq!(bc.height(), 2); // unchanged
    }

    #[test]
    fn test_sync_fork_reorg() {
        let mut bc = Blockchain::new();
        let genesis_hash = bc.chain[0].hash.clone();

        // Add 1 block locally
        bc.add_block(Vec::new(), "local_val".to_string());
        assert_eq!(bc.height(), 1);

        // Build an alternative chain from genesis that is longer (3 blocks)
        let fork1 = Block::new(1, Vec::new(), genesis_hash.clone(), "fork_val".to_string());
        let fork2 = Block::new(2, Vec::new(), fork1.hash.clone(), "fork_val".to_string());
        let fork3 = Block::new(3, Vec::new(), fork2.hash.clone(), "fork_val".to_string());

        // Include the genesis block in the sync set so fork point can be found
        let genesis_clone = bc.chain[0].clone();
        let result = bc
            .sync_from_blocks(vec![genesis_clone, fork1, fork2, fork3])
            .unwrap();

        assert_eq!(result.added, 3);
        assert!(result.reorged);
        assert_eq!(bc.height(), 3);
        assert_eq!(bc.chain[1].validator, "fork_val"); // Our local block was replaced
    }

    #[test]
    fn test_sync_rejects_non_contiguous_segment() {
        let mut bc = Blockchain::new();
        let genesis_hash = bc.chain[0].hash.clone();

        let block1 = Block::new(1, Vec::new(), genesis_hash.clone(), "fork_val".to_string());
        let block3 = Block::new(3, Vec::new(), block1.hash.clone(), "fork_val".to_string());

        let err = bc
            .sync_from_blocks(vec![bc.chain[0].clone(), block1, block3])
            .unwrap_err();

        assert!(err.contains("Invalid index"));
    }
}
