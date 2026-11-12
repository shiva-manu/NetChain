// src/node/storage.rs
//! Persistent block store (sled).
//!
//! Only blocks and the network's genesis hash are stored. Chain state is never persisted:
//! it is rebuilt by replaying and re-verifying the stored blocks at startup, so it can never
//! drift from the chain.

use anyhow::Result;
use sled::{Db, Tree};
use std::path::Path;

use crate::block::Block;

const BLOCKS_TREE: &str = "blocks";
const META_TREE: &str = "meta";
const GENESIS_HASH_KEY: &str = "genesis_hash";

pub struct Storage {
    db: Db,
    blocks: Tree,
    meta: Tree,
}

impl Storage {
    /// Open or create storage at the given path.
    pub fn open(path: &Path) -> Result<Self> {
        let db = sled::open(path)?;
        let blocks = db.open_tree(BLOCKS_TREE)?;
        let meta = db.open_tree(META_TREE)?;
        Ok(Self { db, blocks, meta })
    }

    /// Flush all pending writes to disk.
    pub fn flush(&self) -> Result<()> {
        self.db.flush()?;
        Ok(())
    }

    pub fn save_block(&self, block: &Block) -> Result<()> {
        self.blocks
            .insert(block.index.to_be_bytes(), serde_json::to_vec(block)?)?;
        Ok(())
    }

    /// All stored blocks, ordered by index.
    pub fn load_all_blocks(&self) -> Result<Vec<Block>> {
        // Keys are big-endian indices, so sled iterates them in order.
        self.blocks
            .iter()
            .map(|entry| {
                let (_, value) = entry?;
                Ok(serde_json::from_slice(&value)?)
            })
            .collect()
    }

    /// Delete every stored block with index > `height` (used after a chain reorganization).
    pub fn truncate_above(&self, height: u64) -> Result<()> {
        let start = (height + 1).to_be_bytes();
        let keys: Vec<sled::IVec> = self
            .blocks
            .range(start..)
            .keys()
            .collect::<std::result::Result<_, _>>()?;
        for key in keys {
            self.blocks.remove(key)?;
        }
        Ok(())
    }

    /// Genesis hash of the network this data directory belongs to.
    pub fn genesis_hash(&self) -> Result<Option<String>> {
        Ok(self
            .meta
            .get(GENESIS_HASH_KEY)?
            .map(|v| String::from_utf8_lossy(&v).to_string()))
    }

    pub fn set_genesis_hash(&self, hash: &str) -> Result<()> {
        self.meta.insert(GENESIS_HASH_KEY, hash.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn block(index: u64) -> Block {
        Block::new(
            index,
            Vec::new(),
            format!("prev{index}"),
            "validator".into(),
        )
    }

    #[test]
    fn blocks_round_trip_in_index_order() {
        let dir = tempdir().unwrap();
        let storage = Storage::open(dir.path()).unwrap();
        for i in [2, 0, 1, 10] {
            storage.save_block(&block(i)).unwrap();
        }
        let indices: Vec<u64> = storage
            .load_all_blocks()
            .unwrap()
            .iter()
            .map(|b| b.index)
            .collect();
        assert_eq!(indices, vec![0, 1, 2, 10]);
    }

    #[test]
    fn truncate_and_genesis_hash() {
        let dir = tempdir().unwrap();
        let storage = Storage::open(dir.path()).unwrap();
        for i in 0..5 {
            storage.save_block(&block(i)).unwrap();
        }
        storage.truncate_above(2).unwrap();
        assert_eq!(storage.load_all_blocks().unwrap().len(), 3);

        assert_eq!(storage.genesis_hash().unwrap(), None);
        storage.set_genesis_hash("abc").unwrap();
        assert_eq!(storage.genesis_hash().unwrap().as_deref(), Some("abc"));
    }
}
