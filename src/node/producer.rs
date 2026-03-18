// src/node/producer.rs
//! Block producer module - creates blocks from mempool transactions using PoI consensus

use chrono::Utc;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use tracing::info;

use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::consensus::{HybridWeights, NodeMetrics, PoiConfig, PoiScorer, Thresholds, Weights};
use crate::mempool::Mempool;
use crate::state::{ExecutedProposal, State};
use crate::transaction::SignedTransaction;
use crate::websocket::WsEvent;

/// Configuration for block production
pub struct ProducerConfig {
    /// Maximum transactions per block
    pub max_txs_per_block: usize,
    /// Block production interval in seconds
    pub block_interval_secs: u64,
    /// This node's ID (peer_id or address)
    pub node_id: String,
    /// Fixed block reward (new tokens minted per block)
    pub block_reward: u64,
}

impl Default for ProducerConfig {
    fn default() -> Self {
        Self {
            max_txs_per_block: 100,
            block_interval_secs: 10,
            node_id: "local_node".to_string(),
            block_reward: 50, // 50 tokens per block (similar to early Bitcoin)
        }
    }
}

/// Block producer - manages block creation and validator selection
pub struct BlockProducer {
    config: ProducerConfig,
    poi_scorer: PoiScorer,
    /// Known validators and their metrics
    validator_pool: HashMap<String, NodeMetrics>,
}

impl BlockProducer {
    /// Create a new block producer with default PoI config
    pub fn new(config: ProducerConfig) -> Self {
        let poi_config = Self::default_poi_config();
        Self {
            config,
            poi_scorer: PoiScorer::new(poi_config),
            validator_pool: HashMap::new(),
        }
    }

    /// Create a new block producer with a custom stake_weight
    pub fn with_stake_weight(config: ProducerConfig, stake_weight: f64) -> Self {
        let mut poi_config = Self::default_poi_config();
        poi_config.stake_weight = stake_weight;
        Self {
            config,
            poi_scorer: PoiScorer::new(poi_config),
            validator_pool: HashMap::new(),
        }
    }

    /// Align attestation normalization with the aggregator's quorum threshold.
    pub fn set_min_attestations(&mut self, min_attestations: usize) {
        self.poi_scorer
            .set_hybrid_min_attestations(min_attestations);
    }

    /// Create default PoI configuration
    fn default_poi_config() -> PoiConfig {
        PoiConfig {
            weights: Weights {
                upload: 0.25,
                download: 0.25,
                latency: 0.20,
                uptime: 0.20,
                stability: 0.10,
            },
            thresholds: Thresholds {
                upload_mbps: 100.0,
                download_mbps: 1000.0,
                latency_ms: 200.0,
                uptime_percent: 100.0,
                stability_percent: 100.0,
            },
            stake_weight: 0.3,
            hybrid: HybridWeights::default(),
        }
    }

    /// Register this node as a validator with given metrics
    pub fn register_self(&mut self, metrics: NodeMetrics) {
        self.validator_pool
            .insert(self.config.node_id.clone(), metrics);
    }

    /// Register or update a peer's metrics
    pub fn register_peer(&mut self, node_id: String, metrics: NodeMetrics) {
        self.validator_pool.insert(node_id, metrics);
    }

    /// Remove a peer from the validator pool
    pub fn remove_peer(&mut self, node_id: &str) {
        self.validator_pool.remove(node_id);
    }

    /// Get a peer's metrics (if registered)
    pub fn get_node_metrics(&self, node_id: &str) -> Option<&NodeMetrics> {
        self.validator_pool.get(node_id)
    }

    /// Update a peer's metrics
    pub fn update_peer_metrics(&mut self, node_id: &str, metrics: NodeMetrics) {
        if self.validator_pool.contains_key(node_id) {
            self.validator_pool.insert(node_id.to_string(), metrics);
        }
    }

    /// Apply a recent slashing penalty to a validator's trust profile.
    pub fn penalize_validator(&mut self, node_id: &str, severity: f64) {
        if let Some(metrics) = self.validator_pool.get_mut(node_id) {
            let severity = severity.clamp(0.0, 1.0);
            metrics.slashing_penalty = (metrics.slashing_penalty + severity).clamp(0.0, 1.0);
            metrics.reputation_score =
                (metrics.reputation_score * (1.0 - severity * 0.5)).clamp(0.0, 1.0);
            metrics.identity_score =
                (metrics.identity_score * (1.0 - severity * 0.25)).clamp(0.0, 1.0);
        }
    }

    /// Get the number of known validators
    pub fn validator_count(&self) -> usize {
        self.validator_pool.len()
    }

    /// Compute deterministic seed from previous block hash and block height
    pub fn compute_seed(previous_hash: &str, height: u64) -> u128 {
        let mut hasher = Sha256::new();
        hasher.update(previous_hash.as_bytes());
        hasher.update(height.to_be_bytes());
        let result = hasher.finalize();

        // Take first 16 bytes as u128
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(&result[0..16]);
        u128::from_be_bytes(bytes)
    }

    /// Select the validator for the next block, factoring in stake weights
    pub fn select_validator(
        &self,
        previous_hash: &str,
        height: u64,
        stakes: &HashMap<String, u64>,
    ) -> Option<String> {
        if self.validator_pool.is_empty() {
            return None;
        }

        let seed = Self::compute_seed(previous_hash, height);
        Some(self.poi_scorer.select_validator_with_seed_and_stakes(
            &self.validator_pool,
            seed,
            stakes,
        ))
    }

    /// Check if this node is the selected validator for the next block
    pub fn is_my_turn(
        &self,
        previous_hash: &str,
        height: u64,
        stakes: &HashMap<String, u64>,
    ) -> bool {
        match self.select_validator(previous_hash, height, stakes) {
            Some(validator) => validator == self.config.node_id,
            None => {
                // If no validators registered, allow local block production
                true
            }
        }
    }

    /// Get this node's PoI score
    pub fn get_my_score(&self) -> Option<f64> {
        self.validator_pool
            .get(&self.config.node_id)
            .map(|m| self.poi_scorer.poi_score(m))
    }

    /// Create default node metrics for testing
    pub fn default_node_metrics(node_id: String) -> NodeMetrics {
        NodeMetrics::with_baseline(node_id, 100.0, 1000.0, 10.0, 99.9, 99.9)
    }

    /// Produce a block from mempool transactions
    pub async fn produce_block(
        &mut self,
        blockchain: &Arc<Mutex<Blockchain>>,
        mempool: &Arc<Mutex<Mempool>>,
        state: &Arc<Mutex<State>>,
        ws_tx: &broadcast::Sender<WsEvent>,
    ) -> Option<(Block, Vec<SignedTransaction>, Vec<ExecutedProposal>)> {
        let mut bc = blockchain.lock().await;
        let last_block = bc.last_block();
        let next_height = last_block.index + 1;

        let block_time = Utc::now();
        let block_time_secs: u64 = match block_time.timestamp().try_into() {
            Ok(v) => v,
            Err(_) => return None,
        };

        // Read stake data and chain params from state for stake-weighted selection
        let mut state_guard = state.lock().await;
        let stakes = state_guard.get_stake_map();
        let chain_params = state_guard.chain_params.clone();

        // Keep producer stake-weight in sync with on-chain governance params.
        self.poi_scorer.set_stake_weight(chain_params.stake_weight);

        // Check if it's our turn to produce
        if !self.is_my_turn(&last_block.hash, next_height, &stakes) {
            return None;
        }

        // Select transactions from mempool (use configured max from chain params).
        // Selection validates sequential execution at `block_time_secs`.
        let state_snapshot = state_guard.clone();
        let mempool_guard = mempool.lock().await;
        let selected_txs = mempool_guard.select_for_block(
            chain_params.max_txs_per_block,
            &state_snapshot,
            block_time_secs,
        );
        drop(mempool_guard);

        // Create a deterministic block header (timestamp must match validation time).
        let new_block = Block::new_at(
            next_height,
            selected_txs.clone(),
            last_block.hash.clone(),
            self.config.node_id.clone(),
            block_time,
        );

        if bc.validate_next_block(&new_block).is_err() {
            return None;
        }

        // Apply transactions to state and distribute rewards
        for tx in &selected_txs {
            if state_guard
                .apply_transaction_at(tx, block_time_secs)
                .is_err()
            {
                return None;
            }
        }
        // Credit validator with fees + block reward
        state_guard.apply_block_rewards(
            &self.config.node_id,
            &selected_txs,
            chain_params.block_reward,
        );

        // Execute any passed proposals that have expired
        let executed_proposals = state_guard.execute_passed_proposals_at(block_time_secs);
        if !executed_proposals.is_empty() {
            info!(
                proposals_executed = executed_proposals.len(),
                "executed governance proposals"
            );
        }

        // Commit block to chain after state transition succeeds.
        bc.chain.push(new_block.clone());

        for executed in &executed_proposals {
            let _ = ws_tx.send(WsEvent::ProposalUpdate {
                proposal_id: executed.proposal_id,
                title: executed.title.clone(),
                status: "Passed".to_string(),
                yes_votes: executed.yes_votes,
                no_votes: executed.no_votes,
            });
        }

        Some((new_block, selected_txs, executed_proposals))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{GovernanceProposal, ProposalStatus, StakePosition};
    use crate::transaction::ProposalAction;
    use crate::websocket::create_event_channel;

    #[test]
    fn test_seed_computation() {
        let seed1 = BlockProducer::compute_seed("abc123", 1);
        let seed2 = BlockProducer::compute_seed("abc123", 1);
        let seed3 = BlockProducer::compute_seed("abc123", 2);

        // Same inputs = same seed
        assert_eq!(seed1, seed2);
        // Different height = different seed
        assert_ne!(seed1, seed3);
    }

    #[test]
    fn test_validator_registration() {
        let config = ProducerConfig {
            node_id: "node1".to_string(),
            ..Default::default()
        };
        let mut producer = BlockProducer::new(config);

        assert_eq!(producer.validator_count(), 0);

        let metrics = BlockProducer::default_node_metrics("node1".to_string());
        producer.register_self(metrics);

        assert_eq!(producer.validator_count(), 1);
        assert!(producer.get_my_score().is_some());
    }

    #[test]
    fn test_is_my_turn_single_validator() {
        let config = ProducerConfig {
            node_id: "only_node".to_string(),
            ..Default::default()
        };
        let mut producer = BlockProducer::new(config);

        let metrics = BlockProducer::default_node_metrics("only_node".to_string());
        producer.register_self(metrics);

        // With only one validator, it's always their turn
        let stakes = HashMap::new();
        assert!(producer.is_my_turn("genesis_hash", 1, &stakes));
    }

    #[tokio::test]
    async fn test_produce_block_executes_proposal_and_broadcasts_event() {
        let config = ProducerConfig {
            node_id: "validator_1".to_string(),
            ..Default::default()
        };
        let mut producer = BlockProducer::new(config);
        producer.register_self(BlockProducer::default_node_metrics(
            "validator_1".to_string(),
        ));

        let blockchain = Arc::new(Mutex::new(Blockchain::new()));
        let mempool = Arc::new(Mutex::new(Mempool::new()));
        let mut state = State::with_genesis(vec![("alice".to_string(), 1_000)]);
        state
            .stakes
            .insert("alice".to_string(), StakePosition { amount: 500 });
        state.proposals.insert(
            1,
            GovernanceProposal {
                id: 1,
                proposer: "alice".to_string(),
                title: "Raise rewards".to_string(),
                description: "Increase validator rewards".to_string(),
                created_at: 1,
                expires_at: 2,
                yes_votes: 500,
                no_votes: 0,
                voters: HashMap::from([("alice".to_string(), true)]),
                action: Some(ProposalAction::ChangeBlockReward(100)),
            },
        );
        let state = Arc::new(Mutex::new(state));

        let ws_tx = create_event_channel();
        let mut ws_rx = ws_tx.subscribe();

        let produced = producer
            .produce_block(&blockchain, &mempool, &state, &ws_tx)
            .await
            .expect("single validator should produce a block");

        assert_eq!(produced.2.len(), 1);
        assert_eq!(produced.2[0].proposal_id, 1);

        let state_guard = state.lock().await;
        assert_eq!(state_guard.chain_params.block_reward, 100);
        assert!(state_guard.get_proposal(1).is_none());
        drop(state_guard);

        let event = ws_rx.try_recv().expect("expected proposal update event");
        match event {
            WsEvent::ProposalUpdate {
                proposal_id,
                title,
                status,
                yes_votes,
                no_votes,
            } => {
                assert_eq!(proposal_id, 1);
                assert_eq!(title, "Raise rewards");
                assert_eq!(status, format!("{:?}", ProposalStatus::Passed));
                assert_eq!(yes_votes, 500);
                assert_eq!(no_votes, 0);
            }
            other => panic!("unexpected event: {:?}", other),
        }
    }
}
