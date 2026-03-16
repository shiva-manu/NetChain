// src/node/storage.rs
//! Persistent storage module using sled embedded database

use anyhow::{anyhow, Result};
use sled::{Db, Tree};
use std::path::PathBuf;

use crate::block::Block;
use crate::state::{Account, State};

/// Storage keys
const BLOCKS_TREE: &str = "blocks";
const STATE_TREE: &str = "state";
const META_TREE: &str = "meta";
const CHAIN_HEIGHT_KEY: &str = "chain_height";
const FULL_STATE_KEY: &str = "full_state";

/// Persistent storage manager
pub struct Storage {
    db: Db,
    blocks: Tree,
    state: Tree,
    meta: Tree,
}

impl Storage {
    /// Open or create storage at the given path
    pub fn open(path: &PathBuf) -> Result<Self> {
        let db = sled::open(path)?;
        let blocks = db.open_tree(BLOCKS_TREE)?;
        let state = db.open_tree(STATE_TREE)?;
        let meta = db.open_tree(META_TREE)?;

        Ok(Self {
            db,
            blocks,
            state,
            meta,
        })
    }

    /// Get default storage path
    pub fn default_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".netchain")
            .join("data")
    }

    /// Flush all pending writes to disk
    pub fn flush(&self) -> Result<()> {
        self.db.flush()?;
        Ok(())
    }

    // ==================== Block Storage ====================

    /// Save a block to storage
    pub fn save_block(&self, block: &Block) -> Result<()> {
        let key = block.index.to_be_bytes();
        let value = serde_json::to_vec(block)?;
        self.blocks.insert(key, value)?;

        // Update chain height
        self.set_chain_height(block.index)?;

        Ok(())
    }

    /// Load a block by index
    pub fn load_block(&self, index: u64) -> Result<Option<Block>> {
        let key = index.to_be_bytes();
        match self.blocks.get(key)? {
            Some(data) => {
                let block: Block = serde_json::from_slice(&data)?;
                Ok(Some(block))
            }
            None => Ok(None),
        }
    }

    /// Load all blocks from storage
    pub fn load_all_blocks(&self) -> Result<Vec<Block>> {
        let mut blocks = Vec::new();

        for result in self.blocks.iter() {
            let (_, value) = result?;
            let block: Block = serde_json::from_slice(&value)?;
            blocks.push(block);
        }

        // Sort by index
        blocks.sort_by_key(|b| b.index);
        Ok(blocks)
    }

    /// Get the stored chain height
    pub fn get_chain_height(&self) -> Result<Option<u64>> {
        match self.meta.get(CHAIN_HEIGHT_KEY)? {
            Some(data) => {
                let height = u64::from_be_bytes(
                    data.as_ref()
                        .try_into()
                        .map_err(|_| anyhow!("Invalid height data"))?,
                );
                Ok(Some(height))
            }
            None => Ok(None),
        }
    }

    /// Set the chain height
    fn set_chain_height(&self, height: u64) -> Result<()> {
        self.meta.insert(CHAIN_HEIGHT_KEY, &height.to_be_bytes())?;
        Ok(())
    }

    // ==================== State Storage ====================

    /// Save account state
    pub fn save_account(&self, address: &str, account: &Account) -> Result<()> {
        let value = serde_json::to_vec(account)?;
        self.state.insert(address, value)?;
        Ok(())
    }

    /// Load account state
    pub fn load_account(&self, address: &str) -> Result<Option<Account>> {
        match self.state.get(address)? {
            Some(data) => {
                let account: Account = serde_json::from_slice(&data)?;
                Ok(Some(account))
            }
            None => Ok(None),
        }
    }

    /// Load all accounts from storage
    pub fn load_all_accounts(&self) -> Result<Vec<(String, Account)>> {
        let mut accounts = Vec::new();

        for result in self.state.iter() {
            let (key, value) = result?;
            let address = String::from_utf8(key.to_vec())?;
            let account: Account = serde_json::from_slice(&value)?;
            accounts.push((address, account));
        }

        Ok(accounts)
    }

    /// Load full state snapshot if present, otherwise reconstruct from account records.
    pub fn load_state(&self) -> Result<State> {
        match self.meta.get(FULL_STATE_KEY)? {
            Some(data) => Ok(serde_json::from_slice(&data)?),
            None => {
                let account_map: std::collections::HashMap<String, Account> =
                    self.load_all_accounts()?.into_iter().collect();
                Ok(State::from_accounts(account_map))
            }
        }
    }

    /// Save entire state (batch operation)
    pub fn save_state(&self, state: &State) -> Result<()> {
        self.state.clear()?;
        for (address, account) in state.get_accounts() {
            self.save_account(address, account)?;
        }
        self.meta
            .insert(FULL_STATE_KEY, serde_json::to_vec(state)?)?;
        self.flush()?;
        Ok(())
    }

    /// Legacy helper for account-only saves.
    pub fn save_accounts(
        &self,
        accounts: &std::collections::HashMap<String, Account>,
    ) -> Result<()> {
        for (address, account) in accounts {
            self.save_account(address, account)?;
        }
        self.flush()?;
        Ok(())
    }

    // ==================== Utility ====================

    /// Check if storage has any data
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty() && self.state.is_empty()
    }

    /// Clear all data (use with caution!)
    pub fn clear(&self) -> Result<()> {
        self.blocks.clear()?;
        self.state.clear()?;
        self.meta.clear()?;
        Ok(())
    }

    /// Get storage statistics
    pub fn stats(&self) -> StorageStats {
        StorageStats {
            blocks_count: self.blocks.len(),
            accounts_count: self.state.len(),
            size_on_disk: self.db.size_on_disk().unwrap_or(0),
        }
    }
}

/// Storage statistics
#[derive(Debug, Clone)]
pub struct StorageStats {
    pub blocks_count: usize,
    pub accounts_count: usize,
    pub size_on_disk: u64,
}

impl std::fmt::Display for StorageStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Blocks: {}, Accounts: {}, Size: {} KB",
            self.blocks_count,
            self.accounts_count,
            self.size_on_disk / 1024
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_block_storage() {
        let dir = tempdir().unwrap();
        let storage = Storage::open(&dir.path().to_path_buf()).unwrap();

        let block = Block::new(0, Vec::new(), "0".to_string(), "validator".to_string());
        storage.save_block(&block).unwrap();

        let loaded = storage.load_block(0).unwrap().unwrap();
        assert_eq!(loaded.index, block.index);
        assert_eq!(loaded.hash, block.hash);
        assert_eq!(loaded.merkle_root, block.merkle_root);
    }

    #[test]
    fn test_account_storage() {
        let dir = tempdir().unwrap();
        let storage = Storage::open(&dir.path().to_path_buf()).unwrap();

        let account = Account::new(1000);
        storage.save_account("test_address", &account).unwrap();

        let loaded = storage.load_account("test_address").unwrap().unwrap();
        assert_eq!(loaded.balance, 1000);
        assert_eq!(loaded.nonce, 0);
    }

    #[test]
    fn test_full_state_storage() {
        let dir = tempdir().unwrap();
        let storage = Storage::open(&dir.path().to_path_buf()).unwrap();

        let state = State::with_genesis(vec![("alice".to_string(), 1000)]);
        storage.save_state(&state).unwrap();

        let loaded = storage.load_state().unwrap();
        assert_eq!(loaded.get_balance("alice"), 1000);
    }

    #[test]
    fn test_chain_params_persist_across_reopen() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_path_buf();

        let mut state = State::with_genesis(vec![("alice".to_string(), 1000)]);
        state.chain_params.block_reward = 75;
        state.chain_params.block_interval_secs = 8;
        state.chain_params.max_txs_per_block = 250;
        state.chain_params.stake_weight = 0.45;
        state.chain_params.proposal_quorum_bps = 3_000;

        {
            let storage = Storage::open(&path).unwrap();
            storage.save_state(&state).unwrap();
        }

        let reopened = Storage::open(&path).unwrap();
        let loaded = reopened.load_state().unwrap();
        assert_eq!(loaded.chain_params.block_reward, 75);
        assert_eq!(loaded.chain_params.block_interval_secs, 8);
        assert_eq!(loaded.chain_params.max_txs_per_block, 250);
        assert!((loaded.chain_params.stake_weight - 0.45).abs() < f64::EPSILON);
        assert_eq!(loaded.chain_params.proposal_quorum_bps, 3_000);
    }

    #[test]
    fn test_chain_height() {
        let dir = tempdir().unwrap();
        let storage = Storage::open(&dir.path().to_path_buf()).unwrap();

        assert!(storage.get_chain_height().unwrap().is_none());

        let block = Block::new(5, Vec::new(), "abc".to_string(), "validator".to_string());
        storage.save_block(&block).unwrap();

        assert_eq!(storage.get_chain_height().unwrap(), Some(5));
    }
}
