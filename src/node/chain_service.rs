// src/node/chain_service.rs
//! Owns the canonical chain and state and applies the consensus engine to everything that
//! changes them: blocks from gossip, chain-sync responses (including fork choice / reorgs),
//! locally produced blocks, and replay from disk at startup.
//!
//! Fork choice: the longest chain of valid blocks wins (each block signed by the scheduled
//! proposer of its slot). Lock order everywhere: blockchain -> state -> mempool -> tx_index.

use anyhow::{bail, Result};
use chrono::{DateTime, TimeZone, Utc};
use ed25519_dalek::SigningKey;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tracing::{debug, info, warn};

use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::engine::{self, BlockError, BlockOutcome};
use crate::genesis::GenesisSpec;
use crate::mempool::{Mempool, MAX_BLOCK_SIZE_BYTES};
use crate::node_key;
use crate::state::State;
use crate::storage::Storage;
use crate::websocket::{self, WsEvent};

/// Maximum blocks per chain-sync response.
pub const SYNC_BATCH: usize = 200;
/// How far behind our tip to start a sync request when we see an unknown block.
pub const SYNC_LOOKBACK: u64 = 32;
/// Cap on blocks buffered for a competing fork that is not yet longer than ours.
pub const MAX_FORK_BUFFER: usize = 20_000;

pub type TxIndex = Arc<Mutex<HashMap<String, (u64, usize)>>>;

#[derive(Debug)]
pub enum GossipImport {
    Imported,
    Known,
    /// A competing block at or below our height; ignored unless its chain grows longer.
    Ignored,
    /// The block does not attach to our tip: we are missing blocks or are on a fork.
    NeedSync {
        from_height: u64,
    },
    Rejected(BlockError),
}

#[derive(Debug, Default)]
pub struct SyncOutcome {
    pub added: u64,
    pub reorged: bool,
    /// Ask peers for blocks starting here next.
    pub request_from: Option<u64>,
}

pub struct ChainService {
    pub genesis: GenesisSpec,
    genesis_state: State,
    pub blockchain: Arc<Mutex<Blockchain>>,
    pub state: Arc<Mutex<State>>,
    pub mempool: Arc<Mutex<Mempool>>,
    pub tx_index: TxIndex,
    storage: Arc<Storage>,
    ws_tx: broadcast::Sender<WsEvent>,
    fork_buffer: Mutex<Vec<Block>>,
}

fn build_tx_index(chain: &[Block]) -> HashMap<String, (u64, usize)> {
    let mut index = HashMap::new();
    for block in chain {
        for (i, tx) in block.transactions.iter().enumerate() {
            index.insert(tx.tx_hash_hex(), (block.index, i));
        }
    }
    index
}

impl ChainService {
    /// Open the chain for `genesis`, replaying and re-validating every stored block.
    pub fn load(
        genesis: GenesisSpec,
        storage: Arc<Storage>,
        ws_tx: broadcast::Sender<WsEvent>,
    ) -> Result<Self> {
        let genesis_block = genesis.genesis_block();
        match storage.genesis_hash()? {
            Some(stored) if stored != genesis_block.hash => bail!(
                "this data directory belongs to a different network (genesis {stored}, \
                 configured genesis {}). Use another DATA_DIR or delete the old one.",
                genesis_block.hash
            ),
            Some(_) => {}
            None => {
                if storage.load_all_blocks()?.iter().any(|b| b.index > 0) {
                    bail!(
                        "this data directory was created by an older NetChain version whose \
                         blocks are unsigned. Delete it (or use another DATA_DIR) to join the \
                         network."
                    );
                }
                storage.set_genesis_hash(&genesis_block.hash)?;
                storage.save_block(&genesis_block)?;
            }
        }

        let genesis_state = genesis.build_state();
        let mut bc = Blockchain::with_genesis(genesis_block);
        let mut state = genesis_state.clone();
        for block in storage
            .load_all_blocks()?
            .into_iter()
            .filter(|b| b.index > 0)
        {
            if block.index != bc.height() + 1 {
                warn!(index = block.index, "gap in stored blocks, truncating");
                storage.truncate_above(bc.height())?;
                break;
            }
            match engine::import_block(bc.last_block(), &state, &block, None) {
                Ok(outcome) => {
                    state = outcome.state;
                    bc.chain.push(block);
                }
                Err(e) => {
                    warn!(index = block.index, error = %e, "invalid stored block, truncating");
                    storage.truncate_above(bc.height())?;
                    break;
                }
            }
        }

        let tx_index = build_tx_index(&bc.chain);
        info!(height = bc.height(), "chain loaded and verified");

        Ok(Self {
            genesis,
            genesis_state,
            blockchain: Arc::new(Mutex::new(bc)),
            state: Arc::new(Mutex::new(state)),
            mempool: Arc::new(Mutex::new(Mempool::new())),
            tx_index: Arc::new(Mutex::new(tx_index)),
            storage,
            ws_tx,
            fork_buffer: Mutex::new(Vec::new()),
        })
    }

    pub async fn height(&self) -> u64 {
        self.blockchain.lock().await.height()
    }

    /// Blocks to send in a chain-sync response.
    pub async fn blocks_for_sync(&self, from_height: u64) -> Vec<Block> {
        let bc = self.blockchain.lock().await;
        bc.get_blocks_range(from_height, SYNC_BATCH)
            .into_iter()
            .cloned()
            .collect()
    }

    fn publish_block_events(&self, block: &Block, outcome_events: &BlockOutcome) {
        let _ = self.ws_tx.send(WsEvent::NewBlock {
            index: block.index,
            hash: block.hash.clone(),
            validator: block.validator.clone(),
            tx_count: block.transactions.len(),
            timestamp: block.timestamp.to_rfc3339(),
        });
        for event in &outcome_events.events {
            let _ = self.ws_tx.send(websocket::from_state_event(event));
        }
        for executed in &outcome_events.executed_proposals {
            let _ = self.ws_tx.send(WsEvent::ProposalUpdate {
                proposal_id: executed.proposal_id,
                title: executed.title.clone(),
                status: "Passed".to_string(),
                yes_votes: executed.yes_votes,
                no_votes: executed.no_votes,
            });
        }
    }

    /// Append a validated block on top of the tip and update everything derived from it.
    async fn commit_extension(
        &self,
        bc: &mut Blockchain,
        state: &mut State,
        block: Block,
        outcome: BlockOutcome,
    ) {
        if let Err(e) = self.storage.save_block(&block) {
            warn!(error = %e, index = block.index, "failed to persist block");
        }
        *state = outcome.state.clone();
        {
            let mut mempool = self.mempool.lock().await;
            mempool.remove_transactions(&block.transactions);
            mempool.prune_invalid(state);
        }
        {
            let mut idx = self.tx_index.lock().await;
            for (i, tx) in block.transactions.iter().enumerate() {
                idx.insert(tx.tx_hash_hex(), (block.index, i));
            }
        }
        self.publish_block_events(&block, &outcome);
        bc.chain.push(block);
    }

    /// Handle a block announced over gossip.
    pub async fn import_gossip_block(&self, block: Block) -> GossipImport {
        let now = Utc::now().timestamp();
        let mut bc = self.blockchain.lock().await;
        if bc.hash_at(block.index) == Some(block.hash.as_str()) {
            return GossipImport::Known;
        }
        let tip = bc.last_block().clone();
        if block.index == tip.index + 1 && block.previous_hash == tip.hash {
            let mut state = self.state.lock().await;
            match engine::import_block(&tip, &state, &block, Some(now)) {
                Ok(outcome) => {
                    info!(
                        height = block.index,
                        txs = block.transactions.len(),
                        proposer = %short(&block.validator),
                        "imported block"
                    );
                    self.commit_extension(&mut bc, &mut state, block, outcome)
                        .await;
                    GossipImport::Imported
                }
                Err(e) => GossipImport::Rejected(e),
            }
        } else if block.index > tip.index {
            GossipImport::NeedSync {
                from_height: tip.index.saturating_sub(SYNC_LOOKBACK).max(1),
            }
        } else {
            GossipImport::Ignored
        }
    }

    /// Handle a chain-sync response: extend, buffer a competing fork, or reorganize.
    pub async fn import_sync_blocks(&self, mut blocks: Vec<Block>) -> SyncOutcome {
        let now = Utc::now().timestamp();
        blocks.sort_by_key(|b| b.index);
        blocks.dedup_by_key(|b| b.index);
        if blocks.is_empty() {
            return SyncOutcome::default();
        }
        if blocks
            .windows(2)
            .any(|w| w[1].previous_hash != w[0].hash || w[1].index != w[0].index + 1)
        {
            warn!("sync response is not a contiguous chain segment, ignoring");
            return SyncOutcome::default();
        }
        let full = blocks.len() >= SYNC_BATCH;

        let mut bc = self.blockchain.lock().await;

        // Continue a buffered fork segment if this response extends it.
        {
            let mut buffer = self.fork_buffer.lock().await;
            if let Some(last) = buffer.last() {
                if last.hash == blocks[0].previous_hash && last.index + 1 == blocks[0].index {
                    let mut merged = std::mem::take(&mut *buffer);
                    merged.extend(blocks);
                    blocks = merged;
                }
            }
        }

        if blocks[0].index == 0 {
            if blocks[0].hash != bc.genesis().hash {
                warn!("peer is on a different network (genesis mismatch), ignoring");
                return SyncOutcome::default();
            }
            blocks.remove(0);
            if blocks.is_empty() {
                return SyncOutcome::default();
            }
        }

        // Does the segment attach to a block we have?
        let anchor = blocks[0].index - 1;
        if bc.hash_at(anchor) != Some(blocks[0].previous_hash.as_str()) {
            let from = blocks[0].index.saturating_sub(SYNC_BATCH as u64).max(1);
            debug!(
                from,
                "sync segment does not attach to our chain, requesting earlier blocks"
            );
            return SyncOutcome {
                request_from: Some(from),
                ..Default::default()
            };
        }

        let last_index = blocks.last().map(|b| b.index).unwrap_or(0);
        let known = blocks
            .iter()
            .take_while(|b| bc.hash_at(b.index) == Some(b.hash.as_str()))
            .count();
        let new_blocks: Vec<Block> = blocks.split_off(known);
        if new_blocks.is_empty() {
            return SyncOutcome {
                request_from: full.then_some(last_index + 1),
                ..Default::default()
            };
        }

        let fork_point = new_blocks[0].index - 1;
        let our_height = bc.height();
        let candidate_height = new_blocks.last().map(|b| b.index).unwrap_or(0);

        // Plain extension of our chain.
        if fork_point == our_height {
            let mut state = self.state.lock().await;
            let mut added = 0u64;
            for block in new_blocks {
                let tip = bc.last_block().clone();
                match engine::import_block(&tip, &state, &block, Some(now)) {
                    Ok(outcome) => {
                        self.commit_extension(&mut bc, &mut state, block, outcome)
                            .await;
                        added += 1;
                    }
                    Err(e) => {
                        warn!(index = block.index, error = %e, "rejected synced block");
                        break;
                    }
                }
            }
            if added > 0 {
                info!(added, height = bc.height(), "synced blocks");
            }
            return SyncOutcome {
                added,
                reorged: false,
                request_from: (full && added > 0).then_some(bc.height() + 1),
            };
        }

        // Competing fork that is not (yet) longer than ours.
        if candidate_height <= our_height {
            if full {
                let mut buffer = self.fork_buffer.lock().await;
                if new_blocks.len() <= MAX_FORK_BUFFER {
                    *buffer = new_blocks;
                }
                return SyncOutcome {
                    request_from: Some(last_index + 1),
                    ..Default::default()
                };
            }
            return SyncOutcome::default();
        }

        // Longer fork: validate it fully from the fork point, then switch.
        let mut fork_state =
            match engine::replay_chain(&self.genesis_state, &bc.chain[..=fork_point as usize]) {
                Ok(state) => state,
                Err((index, e)) => {
                    warn!(index, error = %e, "own chain failed replay during reorg, aborting");
                    return SyncOutcome::default();
                }
            };
        let mut parent = bc.chain[fork_point as usize].clone();
        let mut accepted: Vec<(Block, BlockOutcome)> = Vec::new();
        for block in new_blocks {
            match engine::import_block(&parent, &fork_state, &block, Some(now)) {
                Ok(outcome) => {
                    fork_state = outcome.state.clone();
                    parent = block.clone();
                    accepted.push((block, outcome));
                }
                Err(e) => {
                    warn!(index = block.index, error = %e, "rejected block on competing fork");
                    break;
                }
            }
        }
        let new_height = fork_point + accepted.len() as u64;
        if new_height <= our_height {
            return SyncOutcome::default();
        }

        let mut state = self.state.lock().await;
        let orphaned = bc.chain.split_off(fork_point as usize + 1);
        if let Err(e) = self.storage.truncate_above(fork_point) {
            warn!(error = %e, "failed to truncate stored blocks during reorg");
        }
        for (block, outcome) in &accepted {
            if let Err(e) = self.storage.save_block(block) {
                warn!(error = %e, index = block.index, "failed to persist block");
            }
            self.publish_block_events(block, outcome);
            bc.chain.push(block.clone());
        }
        *state = fork_state;

        {
            let mut idx = self.tx_index.lock().await;
            *idx = build_tx_index(&bc.chain);
        }
        {
            // Return transactions from orphaned blocks to the mempool when still valid.
            let included: HashSet<String> = accepted
                .iter()
                .flat_map(|(b, _)| b.transactions.iter().map(|t| t.tx_hash_hex()))
                .collect();
            let mut mempool = self.mempool.lock().await;
            for (block, _) in &accepted {
                mempool.remove_transactions(&block.transactions);
            }
            mempool.prune_invalid(&state);
            for tx in orphaned.iter().flat_map(|b| b.transactions.iter()) {
                if !included.contains(&tx.tx_hash_hex()) {
                    let _ = mempool.add_transaction(tx.clone(), &state);
                }
            }
        }

        info!(
            fork_point,
            replaced = orphaned.len(),
            new_height,
            "chain reorganized to longer fork"
        );
        SyncOutcome {
            added: accepted.len() as u64,
            reorged: true,
            request_from: full.then_some(new_height + 1),
        }
    }

    /// If `key` is the scheduled proposer for the current slot, build, apply and return a
    /// new block (the caller broadcasts it).
    pub async fn try_produce(&self, key: &SigningKey, now: DateTime<Utc>) -> Option<Block> {
        let my_address = node_key::address_of(key);
        let mut bc = self.blockchain.lock().await;
        let mut state = self.state.lock().await;
        let tip = bc.last_block().clone();
        let interval = state.chain_params.block_interval_secs.max(1);
        let now_secs = now.timestamp();
        let slot = engine::slot_at(now_secs, interval);
        if slot <= engine::slot_at(tip.timestamp.timestamp(), interval) {
            return None;
        }
        if engine::proposer_for_slot(&tip, &state, slot)? != my_address {
            return None;
        }

        let candidates = {
            let mempool = self.mempool.lock().await;
            mempool.select_for_block(
                state.chain_params.max_txs_per_block,
                &state,
                now_secs.max(0) as u64,
                MAX_BLOCK_SIZE_BYTES,
            )
        };
        let timestamp = Utc.timestamp_opt(now_secs, 0).single()?;
        match engine::build_block(&tip, &state, candidates, key, timestamp) {
            Ok((block, outcome)) => {
                info!(
                    height = block.index,
                    slot,
                    txs = block.transactions.len(),
                    "produced block"
                );
                self.commit_extension(&mut bc, &mut state, block.clone(), outcome)
                    .await;
                Some(block)
            }
            Err(e) => {
                warn!(error = %e, "failed to build block");
                None
            }
        }
    }
}

fn short(address: &str) -> &str {
    &address[..address.len().min(10)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genesis::{GenesisAllocation, GenesisValidator};
    use crate::state::ChainParams;
    use crate::transaction::{generate_ed25519_keypair, SignedTransaction, Transaction};

    fn service(spec: &GenesisSpec) -> (ChainService, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let storage = Arc::new(Storage::open(&dir.path().to_path_buf()).unwrap());
        let (ws_tx, _) = broadcast::channel(64);
        (
            ChainService::load(spec.clone(), storage, ws_tx).unwrap(),
            dir,
        )
    }

    fn spec_for(keys: &[SigningKey]) -> GenesisSpec {
        let params = ChainParams {
            block_interval_secs: 1,
            ..ChainParams::default()
        };
        GenesisSpec {
            chain_id: "svc-test".into(),
            timestamp: Utc::now().timestamp() - 10_000,
            allocations: keys
                .iter()
                .map(|k| GenesisAllocation {
                    address: node_key::address_of(k),
                    balance: 10_000,
                })
                .collect(),
            validators: keys
                .iter()
                .map(|k| GenesisValidator {
                    address: node_key::address_of(k),
                    stake: 1_000,
                })
                .collect(),
            params,
        }
    }

    /// Produce `n` blocks on `svc` using whichever key is scheduled, starting after `start`.
    async fn produce_n(
        svc: &ChainService,
        keys: &[SigningKey],
        start: i64,
        n: usize,
    ) -> Vec<Block> {
        let mut out = Vec::new();
        let mut t = start;
        while out.len() < n {
            t += 1;
            let now = Utc.timestamp_opt(t, 0).unwrap();
            for key in keys {
                if let Some(b) = svc.try_produce(key, now).await {
                    out.push(b);
                    break;
                }
            }
        }
        out
    }

    #[tokio::test]
    async fn produces_and_syncs_between_nodes() {
        let keys = vec![generate_ed25519_keypair(), generate_ed25519_keypair()];
        let spec = spec_for(&keys);
        let (a, _da) = service(&spec);
        let (b, _db) = service(&spec);

        let start = spec.timestamp + 10;
        let blocks = produce_n(&a, &keys, start, 5).await;
        assert_eq!(a.height().await, 5);

        // Gossip of block 5 alone needs a sync; then sync brings b up to date.
        assert!(matches!(
            b.import_gossip_block(blocks[4].clone()).await,
            GossipImport::NeedSync { .. }
        ));
        let outcome = b.import_sync_blocks(a.blocks_for_sync(1).await).await;
        assert_eq!(outcome.added, 5);
        assert_eq!(b.height().await, 5);
        assert_eq!(
            a.blockchain.lock().await.last_block().hash,
            b.blockchain.lock().await.last_block().hash
        );
    }

    #[tokio::test]
    async fn reorgs_to_longer_fork_and_keeps_shorter_chain_otherwise() {
        let keys = vec![generate_ed25519_keypair(), generate_ed25519_keypair()];
        let spec = spec_for(&keys);
        let (a, _da) = service(&spec);
        let (b, _db) = service(&spec);
        let start = spec.timestamp + 10;

        produce_n(&a, &keys, start, 3).await;
        // b builds a different, longer history in later slots.
        produce_n(&b, &keys, start + 1_000, 6).await;

        // a sees b's longer chain and switches.
        let outcome = a.import_sync_blocks(b.blocks_for_sync(1).await).await;
        assert!(outcome.reorged);
        assert_eq!(a.height().await, 6);

        // b ignores a shorter fork.
        let (c, _dc) = service(&spec);
        produce_n(&c, &keys, start + 5_000, 2).await;
        let outcome = b.import_sync_blocks(c.blocks_for_sync(1).await).await;
        assert!(!outcome.reorged);
        assert_eq!(b.height().await, 6);
    }

    #[tokio::test]
    async fn reload_from_storage_restores_chain_and_state() {
        let keys = vec![generate_ed25519_keypair()];
        let spec = spec_for(&keys);
        let dir = tempfile::tempdir().unwrap();
        let receiver = "a".repeat(40);
        {
            let storage = Arc::new(Storage::open(&dir.path().to_path_buf()).unwrap());
            let (ws_tx, _) = broadcast::channel(64);
            let svc = ChainService::load(spec.clone(), storage, ws_tx).unwrap();
            let sender = node_key::address_of(&keys[0]);
            let tx = Transaction::new(sender, receiver.clone(), 42, 1, 0, None);
            let signed = SignedTransaction::sign_with_keypair(&tx, &keys[0]);
            {
                let state = svc.state.lock().await;
                svc.mempool
                    .lock()
                    .await
                    .add_transaction(signed, &state)
                    .unwrap();
            }
            produce_n(&svc, &keys, Utc::now().timestamp() - 100, 3).await;
            assert_eq!(svc.state.lock().await.get_balance(&receiver), 42);
        }
        let storage = Arc::new(Storage::open(&dir.path().to_path_buf()).unwrap());
        let (ws_tx, _) = broadcast::channel(64);
        let svc = ChainService::load(spec.clone(), storage, ws_tx).unwrap();
        assert_eq!(svc.height().await, 3);
        assert_eq!(svc.state.lock().await.get_balance(&receiver), 42);

        // A different genesis must refuse this data directory.
        drop(svc);
        let mut other = spec.clone();
        other.chain_id = "other".into();
        let storage = Arc::new(Storage::open(&dir.path().to_path_buf()).unwrap());
        let (ws_tx, _) = broadcast::channel(64);
        assert!(ChainService::load(other, storage, ws_tx).is_err());
    }
}
