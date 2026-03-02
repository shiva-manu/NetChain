use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::state::{Account, State};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const CHAIN_FILE: &str = "chain.json";
const STATE_FILE: &str = "state_snapshot.bin";

#[derive(Debug, Clone)]
pub struct Storage {
    base_dir: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedBlockRecord {
    block: Block,
    checksum: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedStateSnapshot {
    height: usize,
    accounts: HashMap<String, Account>,
}

impl Storage {
    pub fn new(base_dir: impl AsRef<Path>) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
        }
    }

    pub fn persist_chain(&self, chain: &[Block]) -> Result<(), String> {
        fs::create_dir_all(&self.base_dir).map_err(|e| format!("failed creating base dir: {e}"))?;

        let records: Vec<PersistedBlockRecord> = chain
            .iter()
            .cloned()
            .map(|block| PersistedBlockRecord {
                checksum: block_checksum(&block),
                block,
            })
            .collect();

        let bytes = serde_json::to_vec_pretty(&records)
            .map_err(|e| format!("failed serializing chain: {e}"))?;

        fs::write(self.chain_path(), bytes).map_err(|e| format!("failed writing chain file: {e}"))
    }

    pub fn load_chain(&self) -> Result<Option<Vec<Block>>, String> {
        let path = self.chain_path();
        if !path.exists() {
            return Ok(None);
        }

        let bytes = fs::read(path).map_err(|e| format!("failed reading chain file: {e}"))?;
        let records: Vec<PersistedBlockRecord> =
            serde_json::from_slice(&bytes).map_err(|e| format!("failed parsing chain: {e}"))?;

        let mut chain = Vec::with_capacity(records.len());
        for record in records {
            let actual = block_checksum(&record.block);
            if record.checksum != actual {
                return Err("persisted block checksum mismatch".into());
            }
            chain.push(record.block);
        }

        Blockchain::from_chain(chain.clone())
            .map_err(|e| format!("loaded chain failed validation: {e}"))?;

        Ok(Some(chain))
    }

    pub fn persist_state_snapshot(&self, state: &State, height: usize) -> Result<(), String> {
        fs::create_dir_all(&self.base_dir).map_err(|e| format!("failed creating base dir: {e}"))?;

        let snapshot = PersistedStateSnapshot {
            height,
            accounts: state.snapshot_accounts(),
        };

        let bytes = bincode::serde::encode_to_vec(&snapshot, bincode::config::standard())
            .map_err(|e| format!("failed serializing snapshot: {e}"))?;

        fs::write(self.state_path(), bytes).map_err(|e| format!("failed writing snapshot: {e}"))
    }

    pub fn load_state_snapshot(&self) -> Result<Option<(State, usize)>, String> {
        let path = self.state_path();
        if !path.exists() {
            return Ok(None);
        }

        let bytes = fs::read(path).map_err(|e| format!("failed reading snapshot: {e}"))?;
        let (snapshot, _): (PersistedStateSnapshot, usize) =
            bincode::serde::decode_from_slice(&bytes, bincode::config::standard())
                .map_err(|e| format!("failed decoding snapshot: {e}"))?;

        Ok(Some((
            State::from_accounts(snapshot.accounts),
            snapshot.height,
        )))
    }

    pub fn chain_path(&self) -> PathBuf {
        self.base_dir.join(CHAIN_FILE)
    }

    pub fn state_path(&self) -> PathBuf {
        self.base_dir.join(STATE_FILE)
    }
}

fn block_checksum(block: &Block) -> String {
    let bytes = serde_json::to_vec(block).expect("block serialization should succeed");
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_dir(name: &str) -> PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("netchain_{name}_{ts}"))
    }

    #[test]
    fn round_trip_save_load_chain_and_state() {
        let dir = test_dir("round_trip");
        let storage = Storage::new(&dir);

        let mut bc = Blockchain::new();
        bc.add_block("one".into());
        bc.add_block("two".into());

        let state = State::with_genesis(vec![("alice".to_string(), 42)]);

        storage.persist_chain(&bc.chain).expect("persist chain");
        storage
            .persist_state_snapshot(&state, bc.chain.len() - 1)
            .expect("persist state");

        let loaded_chain = storage
            .load_chain()
            .expect("load chain")
            .expect("chain exists");
        let (loaded_state, height) = storage
            .load_state_snapshot()
            .expect("load state")
            .expect("state exists");

        assert_eq!(loaded_chain.len(), bc.chain.len());
        assert_eq!(
            loaded_chain.last().unwrap().hash,
            bc.chain.last().unwrap().hash
        );
        assert_eq!(height, bc.chain.len() - 1);
        assert_eq!(loaded_state.get_balance("alice"), 42);
    }

    #[test]
    fn rejects_corrupted_chain_data() {
        let dir = test_dir("checksum");
        let storage = Storage::new(&dir);

        let mut bc = Blockchain::new();
        bc.add_block("valid".into());
        storage.persist_chain(&bc.chain).expect("persist chain");

        let mut bytes = fs::read(storage.chain_path()).expect("read chain file");
        bytes.push(b'!');
        fs::write(storage.chain_path(), bytes).expect("corrupt chain file");

        assert!(storage.load_chain().is_err());
    }

    #[test]
    fn restart_continuity_uses_loaded_chain() {
        let dir = test_dir("restart");
        let storage = Storage::new(&dir);

        let mut first = Blockchain::new();
        first.add_block("before restart".into());
        storage.persist_chain(&first.chain).expect("persist chain");

        let loaded = storage
            .load_chain()
            .expect("load chain")
            .expect("chain exists");
        let mut restarted = Blockchain::from_chain(loaded).expect("valid loaded chain");

        restarted.add_block("after restart".into());
        assert_eq!(restarted.chain.len(), 3);
        assert_eq!(restarted.last_block().index, 2);
    }
}
