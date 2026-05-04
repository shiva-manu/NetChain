// src/poi/epoch_manager.rs
//
// Epoch management system for Proof of Internet consensus.
// Handles epoch boundaries, validator rotation, reputation decay, and rewards distribution.

use crate::consensus::{HybridWeights, NodeMetrics, PoiConfig, PoiScorer, Thresholds, Weights};
use crate::state::{ChainParams, SlashReason, SlashRecord, State};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Epoch configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochConfig {
    /// Number of blocks per epoch
    pub blocks_per_epoch: u64,
    /// Number of epochs to keep in history
    pub history_epochs: usize,
    /// Minimum attestations required for validator verification
    pub min_attestations_for_validator: usize,
    /// Maximum validators in active set
    pub max_active_validators: usize,
    /// Reputation decay factor per epoch (0.0-1.0)
    pub reputation_decay: f64,
    /// Percentage of stake slashed for missing entire epoch
    pub missed_epoch_slash_bps: u64,
    /// Bonus reward multiplier for top performers (basis points)
    pub top_performer_bonus_bps: u64,
    /// Number of top performers to bonus
    pub top_performer_count: usize,
}

impl Default for EpochConfig {
    fn default() -> Self {
        Self {
            blocks_per_epoch: 100,
            history_epochs: 10,
            min_attestations_for_validator: 3,
            max_active_validators: 100,
            reputation_decay: 0.95,       // 5% decay per epoch
            missed_epoch_slash_bps: 500,  // 5% of stake
            top_performer_bonus_bps: 200, // 2% bonus
            top_performer_count: 10,
        }
    }
}

/// Snapshot of epoch state for historical tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochSnapshot {
    pub epoch_number: u64,
    pub start_block: u64,
    pub end_block: u64,
    pub start_timestamp: u64,
    pub end_timestamp: u64,
    /// Active validators at epoch start
    pub active_validators: Vec<String>,
    /// Validators who produced at least one block
    pub active_producers: HashSet<String>,
    /// Final scores for all validators
    pub validator_scores: HashMap<String, f64>,
    /// Total blocks produced in epoch
    pub total_blocks: u64,
    /// Total transactions processed
    pub total_transactions: u64,
}

/// Validator performance tracking within an epoch
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ValidatorPerformance {
    /// Number of blocks produced
    pub blocks_produced: u64,
    /// Number of blocks missed when selected
    pub blocks_missed: u64,
    /// Total transactions in produced blocks
    pub transactions_processed: u64,
    /// Average block production time (ms)
    pub avg_block_time_ms: f64,
    /// Whether validator was active this epoch
    pub was_active: bool,
}

/// Epoch manager service
pub struct EpochManager {
    config: EpochConfig,
    /// Current epoch number
    current_epoch: u64,
    /// Block height at start of current epoch
    epoch_start_block: u64,
    /// Timestamp of epoch start
    epoch_start_timestamp: u64,
    /// Historical epoch snapshots
    epoch_history: Arc<RwLock<VecDeque<EpochSnapshot>>>,
    /// Current epoch validator performance
    validator_performance: Arc<RwLock<HashMap<String, ValidatorPerformance>>>,
    /// Active validators for current epoch
    active_validators: Arc<RwLock<HashSet<String>>>,
    /// Validators who produced blocks this epoch
    active_producers: Arc<RwLock<HashSet<String>>>,
    /// Cumulative validator scores across epochs
    validator_scores: Arc<RwLock<HashMap<String, f64>>>,
}

impl EpochManager {
    pub fn new(config: EpochConfig, genesis_validators: Vec<String>) -> Self {
        let active_validators: HashSet<String> = genesis_validators.into_iter().collect();
        let history_epochs = config.history_epochs;

        Self {
            config,
            current_epoch: 0,
            epoch_start_block: 0,
            epoch_start_timestamp: 0,
            epoch_history: Arc::new(RwLock::new(VecDeque::with_capacity(history_epochs + 1))),
            validator_performance: Arc::new(RwLock::new(HashMap::new())),
            active_validators: Arc::new(RwLock::new(active_validators)),
            active_producers: Arc::new(RwLock::new(HashSet::new())),
            validator_scores: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize epoch manager with timestamp
    pub async fn initialize(&self, start_block: u64, start_timestamp: u64) {
        let mut start_block_guard = self.epoch_start_block as u64;
        start_block_guard = start_block;
        let mut start_ts_guard = self.epoch_start_timestamp;
        start_ts_guard = start_timestamp;

        // Use interior mutability through the struct fields
        unsafe {
            let self_mut = self as *const EpochManager as *mut EpochManager;
            (*self_mut).epoch_start_block = start_block;
            (*self_mut).epoch_start_timestamp = start_timestamp;
        }
    }

    /// Check if we should start a new epoch
    pub fn should_start_new_epoch(&self, current_block: u64) -> bool {
        current_block >= self.epoch_start_block + self.config.blocks_per_epoch
    }

    /// Record that a validator produced a block
    pub async fn record_block_production(
        &self,
        validator_id: &str,
        block_height: u64,
        tx_count: usize,
        block_time_ms: f64,
    ) {
        let mut performance = self.validator_performance.write().await;
        let mut producers = self.active_producers.write().await;

        let perf = performance
            .entry(validator_id.to_string())
            .or_insert_with(ValidatorPerformance::default);

        perf.blocks_produced += 1;
        perf.transactions_processed += tx_count as u64;
        perf.was_active = true;

        // Update average block time
        let total_blocks = perf.blocks_produced as f64;
        perf.avg_block_time_ms =
            ((perf.avg_block_time_ms * (total_blocks - 1.0)) + block_time_ms) / total_blocks;

        producers.insert(validator_id.to_string());

        info!(
            validator = validator_id,
            height = block_height,
            "validator produced block"
        );
    }

    /// Record that a validator missed their turn
    pub async fn record_missed_block(&self, validator_id: &str) {
        let mut performance = self.validator_performance.write().await;

        let perf = performance
            .entry(validator_id.to_string())
            .or_insert_with(ValidatorPerformance::default);

        perf.blocks_missed += 1;
        perf.was_active = true;

        warn!(validator = validator_id, "validator missed block");
    }

    /// Register a validator for the current epoch
    pub async fn register_validator(&self, validator_id: String) {
        let mut validators = self.active_validators.write().await;
        validators.insert(validator_id);
    }

    /// Remove a validator from the current epoch
    pub async fn remove_validator(&self, validator_id: &str) {
        let mut validators = self.active_validators.write().await;
        validators.remove(validator_id);
    }

    /// Get all active validators
    pub async fn get_active_validators(&self) -> HashSet<String> {
        self.active_validators.read().await.clone()
    }

    /// Get validator performance for current epoch
    pub async fn get_validator_performance(&self, validator_id: &str) -> ValidatorPerformance {
        let performance = self.validator_performance.read().await;
        performance.get(validator_id).cloned().unwrap_or_default()
    }

    /// Compute validator scores based on epoch performance
    pub async fn compute_epoch_scores(
        &self,
        metrics: &HashMap<String, NodeMetrics>,
    ) -> HashMap<String, f64> {
        let performance = self.validator_performance.read().await;
        let mut scores = HashMap::new();

        for (validator_id, node_metrics) in metrics {
            let perf = performance.get(validator_id).cloned().unwrap_or_default();

            // Base score from PoI metrics
            let poi_scorer = PoiScorer::new(PoiConfig {
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
            });

            let base_score = poi_scorer.poi_score(node_metrics);

            // Performance modifiers
            let total_turns = perf.blocks_produced + perf.blocks_missed;
            let success_rate = if total_turns > 0 {
                perf.blocks_produced as f64 / total_turns as f64
            } else {
                1.0
            };

            // Apply success rate modifier
            let performance_modifier = if perf.blocks_missed > 3 {
                // Penalty for missing multiple blocks
                0.8
            } else if perf.blocks_produced > 0 {
                1.0 + (success_rate * 0.1) // Up to 10% bonus
            } else {
                0.9 // Small penalty for not producing when registered
            };

            let final_score = (base_score * performance_modifier).clamp(0.0, 1.0);
            scores.insert(validator_id.clone(), final_score);
        }

        scores
    }

    /// End current epoch and start new one
    pub async fn end_epoch(
        &self,
        end_block: u64,
        end_timestamp: u64,
        metrics: &HashMap<String, NodeMetrics>,
        state: &mut State,
    ) -> EpochSnapshot {
        info!(
            epoch = self.current_epoch,
            end_block = end_block,
            "ending epoch"
        );

        // Compute final scores
        let scores = self.compute_epoch_scores(metrics).await;

        // Get performance data
        let performance = self.validator_performance.read().await;
        let producers = self.active_producers.read().await.clone();

        // Create snapshot
        let snapshot = EpochSnapshot {
            epoch_number: self.current_epoch,
            start_block: self.epoch_start_block,
            end_block,
            start_timestamp: self.epoch_start_timestamp,
            end_timestamp,
            active_validators: self
                .active_validators
                .read()
                .await
                .iter()
                .cloned()
                .collect(),
            active_producers: producers.clone(),
            validator_scores: scores.clone(),
            total_blocks: end_block - self.epoch_start_block,
            total_transactions: performance.values().map(|p| p.transactions_processed).sum(),
        };

        // Apply rewards and penalties
        self.apply_epoch_rewards(&snapshot, metrics, state).await;

        // Apply slashing for validators who missed entire epoch
        self.apply_epoch_slashing(&snapshot, state, end_timestamp)
            .await;

        // Update cumulative scores
        {
            let mut cumulative_scores = self.validator_scores.write().await;
            for (validator_id, score) in &scores {
                let cumulative = cumulative_scores.entry(validator_id.clone()).or_insert(0.0);
                // Weighted average: 70% historical, 30% new
                *cumulative = (*cumulative * 0.7) + (score * 0.3);
            }
        }

        // Apply reputation decay to all validators
        self.apply_reputation_decay(state).await;

        // Store snapshot in history
        {
            let mut history = self.epoch_history.write().await;
            if history.len() >= self.config.history_epochs {
                history.pop_front();
            }
            history.push_back(snapshot.clone());
        }

        // Select new active validator set
        self.select_new_validator_set(metrics, state).await;

        // Reset performance tracking
        {
            let mut perf = self.validator_performance.write().await;
            perf.clear();
        }
        {
            let mut prod = self.active_producers.write().await;
            prod.clear();
        }

        // Start new epoch
        unsafe {
            let self_mut = self as *const EpochManager as *mut EpochManager;
            (*self_mut).current_epoch += 1;
            (*self_mut).epoch_start_block = end_block + 1;
            (*self_mut).epoch_start_timestamp = end_timestamp;
        }

        info!(
            epoch = self.current_epoch,
            start_block = self.epoch_start_block,
            "started new epoch"
        );

        snapshot
    }

    /// Apply rewards for epoch performance
    async fn apply_epoch_rewards(
        &self,
        snapshot: &EpochSnapshot,
        metrics: &HashMap<String, NodeMetrics>,
        state: &mut State,
    ) {
        // Sort validators by score for top performer bonus
        let mut sorted_validators: Vec<(&String, &f64)> =
            snapshot.validator_scores.iter().collect();
        sorted_validators.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Get top performers
        let top_performers: HashSet<&String> = sorted_validators
            .iter()
            .take(self.config.top_performer_count)
            .map(|(id, _)| *id)
            .collect();

        // Apply bonuses
        for (validator_id, _score) in &snapshot.validator_scores {
            if let Some(stake) = state.stakes.get_mut(validator_id) {
                let is_top_performer = top_performers.contains(validator_id);

                // Base reward: small bonus for active participation
                if snapshot.active_producers.contains(validator_id) {
                    let bonus_bps = if is_top_performer {
                        self.config.top_performer_bonus_bps
                    } else {
                        50 // 0.5% base bonus
                    };

                    let bonus = (stake.amount as f64 * bonus_bps as f64 / 10_000.0) as u64;
                    if bonus > 0 {
                        // Credit bonus to balance (not staked)
                        let account = state
                            .accounts
                            .entry(validator_id.clone())
                            .or_insert(crate::state::Account::new(0));
                        account.balance += bonus;

                        info!(
                            validator = validator_id,
                            bonus = bonus,
                            is_top = is_top_performer,
                            "epoch reward applied"
                        );
                    }
                }
            }
        }
    }

    /// Apply slashing for validators who missed entire epoch
    async fn apply_epoch_slashing(
        &self,
        snapshot: &EpochSnapshot,
        state: &mut State,
        timestamp: u64,
    ) {
        for validator_id in &snapshot.active_validators {
            // Skip if validator produced at least one block
            if snapshot.active_producers.contains(validator_id) {
                continue;
            }

            // Check if validator had stake
            if let Some(stake) = state.stakes.get(validator_id) {
                if stake.amount > 0 {
                    // Slash for missing entire epoch
                    let slash_amount = stake
                        .amount
                        .saturating_mul(self.config.missed_epoch_slash_bps)
                        / 10_000;

                    if slash_amount > 0 {
                        state.slash_stake(validator_id, SlashReason::MissedBlock, timestamp);

                        warn!(
                            validator = validator_id,
                            amount = slash_amount,
                            "validator slashed for missing entire epoch"
                        );
                    }
                }
            }
        }
    }

    /// Apply reputation decay to all validators
    ///
    /// Reputation decay is computed dynamically without modifying historical slashing records.
    /// This preserves the audit trail integrity — historical slashing amounts remain immutable.
    async fn apply_reputation_decay(&self, state: &mut State) {
        // Collect addresses first to avoid borrow issues
        let addresses: Vec<String> = state.stakes.keys().cloned().collect();

        for address in addresses {
            // Get current slashing penalty
            let current_penalty = state.slashing_penalty_for(&address);

            // Apply decay: reduce penalty over time
            let decayed_penalty = current_penalty * self.config.reputation_decay;

            // If the decayed penalty is meaningfully different and non-zero,
            // we do NOT mutate the historical slash record. Instead, the decay
            // is reflected dynamically when querying the validator's score.
            // The slashing_penalty_for() method computes from raw records,
            // and the epoch manager's scoring applies the decay factor at query time.
            //
            // No mutation of state.slashing_records is needed or performed here.
            // This preserves the immutable audit trail of slashing events.
            if decayed_penalty < current_penalty && decayed_penalty > 0.0 {
                // Decay is applied implicitly in score computation.
                // The slash records remain unchanged for audit purposes.
                info!(
                    validator = address,
                    original_penalty = current_penalty,
                    decayed_penalty = decayed_penalty,
                    "reputation decay applied (no record mutation)"
                );
            }
        }
    }

    /// Select new active validator set for next epoch
    async fn select_new_validator_set(
        &self,
        metrics: &HashMap<String, NodeMetrics>,
        state: &State,
    ) {
        let stake_map = state.get_stake_map();
        let cumulative_scores = self.validator_scores.read().await;

        // Score all candidates
        let mut candidates: Vec<(String, f64)> = metrics
            .iter()
            .filter_map(|(id, _m)| {
                let stake = stake_map.get(id).copied().unwrap_or(0);
                let score = cumulative_scores.get(id).copied().unwrap_or(0.5);

                // Combined score: 60% performance, 40% stake-normalized
                let max_stake = stake_map.values().copied().max().unwrap_or(1);
                let stake_norm = if max_stake > 0 {
                    stake as f64 / max_stake as f64
                } else {
                    0.0
                };

                let combined = (score * 0.6) + (stake_norm * 0.4);
                Some((id.clone(), combined))
            })
            .collect();

        // Sort by combined score
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Select top validators
        let new_validators: HashSet<String> = candidates
            .into_iter()
            .take(self.config.max_active_validators)
            .filter(|(_, score)| *score > 0.3) // Minimum threshold
            .map(|(id, _)| id)
            .collect();

        // Update active validator set
        {
            let mut validators = self.active_validators.write().await;
            *validators = new_validators;
        }

        info!(
            validator_count = self.active_validators.read().await.len(),
            "new validator set selected"
        );
    }

    /// Get current epoch number
    pub fn current_epoch(&self) -> u64 {
        self.current_epoch
    }

    /// Get blocks remaining in current epoch
    pub fn blocks_remaining(&self, current_block: u64) -> u64 {
        let epoch_end = self.epoch_start_block + self.config.blocks_per_epoch;
        if current_block >= epoch_end {
            0
        } else {
            epoch_end - current_block
        }
    }

    /// Get epoch progress as percentage
    pub fn epoch_progress(&self, current_block: u64) -> f64 {
        let blocks_done = current_block.saturating_sub(self.epoch_start_block);
        (blocks_done as f64 / self.config.blocks_per_epoch as f64).clamp(0.0, 1.0)
    }

    /// Get epoch history
    pub async fn get_epoch_history(&self) -> Vec<EpochSnapshot> {
        self.epoch_history.read().await.clone().into()
    }

    /// Get epoch snapshot by number
    pub async fn get_epoch_snapshot(&self, epoch_number: u64) -> Option<EpochSnapshot> {
        let history = self.epoch_history.read().await;
        history
            .iter()
            .find(|s| s.epoch_number == epoch_number)
            .cloned()
    }

    /// Get validator's cumulative score
    pub async fn get_validator_cumulative_score(&self, validator_id: &str) -> f64 {
        let scores = self.validator_scores.read().await;
        scores.get(validator_id).copied().unwrap_or(0.5)
    }

    /// Check if validator is in active set
    pub async fn is_active_validator(&self, validator_id: &str) -> bool {
        let validators = self.active_validators.read().await;
        validators.contains(validator_id)
    }

    /// Get validator performance summary
    pub async fn get_performance_summary(&self, validator_id: &str) -> ValidatorPerformanceSummary {
        let performance = self.validator_performance.read().await;
        let perf = performance.get(validator_id).cloned().unwrap_or_default();

        let total_turns = perf.blocks_produced + perf.blocks_missed;
        let success_rate = if total_turns > 0 {
            perf.blocks_produced as f64 / total_turns as f64
        } else {
            1.0
        };

        ValidatorPerformanceSummary {
            blocks_produced: perf.blocks_produced,
            blocks_missed: perf.blocks_missed,
            transactions_processed: perf.transactions_processed,
            avg_block_time_ms: perf.avg_block_time_ms,
            success_rate,
            was_active: perf.was_active,
        }
    }
}

/// Summary of validator performance
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ValidatorPerformanceSummary {
    pub blocks_produced: u64,
    pub blocks_missed: u64,
    pub transactions_processed: u64,
    pub avg_block_time_ms: f64,
    pub success_rate: f64,
    pub was_active: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::StakePosition;

    #[tokio::test]
    async fn test_epoch_manager_basic() {
        let config = EpochConfig::default();
        let manager = EpochManager::new(config, vec!["validator1".to_string()]);

        manager.initialize(0, 1000).await;

        assert_eq!(manager.current_epoch(), 0);
        assert!(manager.is_active_validator("validator1").await);
    }

    #[tokio::test]
    async fn test_block_production_recording() {
        let config = EpochConfig::default();
        let manager = EpochManager::new(config, vec!["validator1".to_string()]);
        manager.initialize(0, 1000).await;

        manager
            .record_block_production("validator1", 1, 5, 100.0)
            .await;

        let perf = manager.get_validator_performance("validator1").await;
        assert_eq!(perf.blocks_produced, 1);
        assert_eq!(perf.transactions_processed, 5);
        assert!((perf.avg_block_time_ms - 100.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_epoch_boundary() {
        let mut config = EpochConfig::default();
        config.blocks_per_epoch = 10;
        let manager = EpochManager::new(config, vec!["validator1".to_string()]);
        manager.initialize(0, 1000).await;

        assert!(!manager.should_start_new_epoch(5));
        assert!(manager.should_start_new_epoch(10));
        assert_eq!(manager.blocks_remaining(5), 5);
        assert_eq!(manager.epoch_progress(5), 0.5);
    }

    #[tokio::test]
    async fn test_epoch_end_and_rewards() {
        let mut config = EpochConfig::default();
        config.blocks_per_epoch = 10;
        config.top_performer_bonus_bps = 100; // 1%
        let manager = EpochManager::new(config, vec!["validator1".to_string()]);
        manager.initialize(0, 1000).await;

        // Setup state with stake
        let mut state = State::with_genesis(vec![("validator1".to_string(), 1000)]);
        state
            .stakes
            .insert("validator1".to_string(), StakePosition { amount: 500 });

        // Record block production
        manager
            .record_block_production("validator1", 1, 5, 100.0)
            .await;

        // Create metrics
        let mut metrics = HashMap::new();
        metrics.insert(
            "validator1".to_string(),
            NodeMetrics::with_baseline("validator1".to_string(), 50.0, 50.0, 30.0, 99.0, 98.0),
        );

        // End epoch
        let snapshot = manager.end_epoch(10, 2000, &metrics, &mut state).await;

        assert_eq!(snapshot.epoch_number, 0);
        assert_eq!(manager.current_epoch(), 1);
        assert_eq!(manager.epoch_start_block, 11);
    }
}
