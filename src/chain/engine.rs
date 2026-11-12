// src/chain/engine.rs
//! Deterministic consensus rules ("Proof of Internet" hybrid).
//!
//! Everything in this module is a pure function of on-chain data (the parent block and the
//! state after the parent block), so every honest node computes exactly the same answer.
//! It is the single source of truth used by block production, gossip import, chain sync and
//! replay from disk.
//!
//! ## Slots
//! Time is divided into slots of `block_interval_secs`. A block's slot is
//! `timestamp / block_interval_secs`, and every block must be in a later slot than its parent.
//! Exactly one validator is eligible to propose in each slot. If that validator is offline
//! the slot stays empty and the next slot has a (most likely) different proposer, so the
//! chain keeps moving as long as some validators are online.
//!
//! ## Proposer selection
//! The proposer of a slot is drawn from the active validator set (stake >=
//! `min_validator_stake`) with probability proportional to its weight:
//!
//! ```text
//! trust  = 40% PoI + 40% uptime + 20% (1 - slashing penalty)        (all in basis points)
//! weight = (1 - stake_weight) * trust + stake_weight * stake / max_stake
//! ```
//!
//! * **PoI** comes from `AttestMetrics` transactions: other validators measure a validator's
//!   round-trip latency and throughput over the p2p network and record it on-chain. The
//!   median of recent attestations from other active validators is used, so a single
//!   dishonest attester cannot move the score.
//! * **Uptime** is derived from the chain itself: producing a block in your slot raises it,
//!   missing your slot lowers it.
//! * **Slashing penalty** comes from recorded slashing events.
//!
//! ## Jailing
//! A validator whose uptime drops below `JAIL_UPTIME_BPS` is jailed: it gets no more slots
//! (so an offline validator stops slowing the chain down) until it sends `Unjail`. The last
//! active validator is never jailed.
//!
//! All arithmetic is integer based (basis points) so results are bit-for-bit identical on
//! every platform.

use chrono::{DateTime, Utc};
use ed25519_dalek::SigningKey;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fmt;

use crate::block::Block;
use crate::state::{ExecutedProposal, SlashReason, State, StateEvent};
use crate::transaction::SignedTransaction;

pub const BPS: u64 = 10_000;
/// Blocks timestamped further than this into the future are rejected.
pub const MAX_FUTURE_DRIFT_SECS: i64 = 15;
/// Upper bound on missed slots accounted per block (bounds work after long outages).
pub const MAX_MISSED_SLOTS_ACCOUNTED: u64 = 64;
/// Uptime EWMA: each produced/missed slot keeps 95% of the old value.
pub const UPTIME_KEEP_PERCENT: u64 = 95;
/// Uptime assigned to a validator that has not been scheduled yet.
pub const INITIAL_UPTIME_BPS: u64 = 7_500;
/// PoI score for validators without enough recent attestations.
pub const UNMEASURED_POI_BPS: u64 = 5_000;
/// Median RTT at or above this scores 0 for latency.
pub const LATENCY_ZERO_SCORE_MS: u64 = 500;
/// Median throughput at or above this scores 100% for bandwidth.
pub const THROUGHPUT_FULL_SCORE_KBPS: u64 = 100_000;
/// A validator whose uptime falls below this is jailed (removed from the active set) until
/// it sends an `Unjail` transaction. From the initial 75% it takes ~8 consecutive missed slots.
pub const JAIL_UPTIME_BPS: u64 = 5_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockError {
    /// Header, hash, merkle root, signature or tx signature problem.
    Structural(String),
    /// Block is not in a later slot than its parent.
    BadSlot {
        parent_slot: u64,
        slot: u64,
    },
    /// Block timestamp is too far in the future.
    FromTheFuture {
        timestamp: i64,
        now: i64,
    },
    /// The active validator set is empty, nobody may propose.
    NoValidators,
    /// Signed by someone other than the slot's scheduled proposer.
    WrongProposer {
        expected: String,
        got: String,
    },
    TooManyTransactions {
        count: usize,
        max: usize,
    },
    InvalidTransaction {
        index: usize,
        reason: String,
    },
}

impl fmt::Display for BlockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlockError::Structural(e) => write!(f, "{e}"),
            BlockError::BadSlot { parent_slot, slot } => {
                write!(
                    f,
                    "block slot {slot} is not after parent slot {parent_slot}"
                )
            }
            BlockError::FromTheFuture { timestamp, now } => {
                write!(
                    f,
                    "block timestamp {timestamp} is in the future (now {now})"
                )
            }
            BlockError::NoValidators => write!(f, "no active validators"),
            BlockError::WrongProposer { expected, got } => {
                write!(f, "wrong proposer: expected {expected}, got {got}")
            }
            BlockError::TooManyTransactions { count, max } => {
                write!(f, "block has {count} transactions, max is {max}")
            }
            BlockError::InvalidTransaction { index, reason } => {
                write!(f, "transaction #{index} is invalid: {reason}")
            }
        }
    }
}

impl std::error::Error for BlockError {}

/// Result of applying a block on top of its parent state.
#[derive(Debug, Clone)]
pub struct BlockOutcome {
    pub state: State,
    pub events: Vec<StateEvent>,
    pub executed_proposals: Vec<ExecutedProposal>,
}

/// A validator's consensus score, as seen at a given parent block.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ValidatorScore {
    pub address: String,
    pub stake: u64,
    pub poi_bps: u64,
    pub uptime_bps: u64,
    pub slashing_penalty_bps: u64,
    pub trust_bps: u64,
    /// Number of recent attestations from other active validators.
    pub attestations: usize,
    pub median_rtt_ms: Option<u64>,
    pub median_throughput_kbps: Option<u64>,
    pub blocks_produced: u64,
    pub slots_missed: u64,
    /// Relative selection weight (only meaningful compared with other validators).
    pub weight: u128,
    /// Jailed validators appear only in reports, never in selection.
    pub jailed: bool,
}

pub fn timestamp_secs(ts: &DateTime<Utc>) -> i64 {
    ts.timestamp()
}

/// Slot number for a timestamp under a given block interval.
pub fn slot_at(timestamp_secs: i64, interval_secs: u64) -> u64 {
    (timestamp_secs.max(0) as u64) / interval_secs.max(1)
}

/// Start time (unix seconds) of a slot.
pub fn slot_start(slot: u64, interval_secs: u64) -> i64 {
    slot.saturating_mul(interval_secs.max(1)) as i64
}

/// Addresses whose bonded stake meets the validator threshold and that are not jailed, sorted.
pub fn active_validators(state: &State) -> BTreeSet<String> {
    state
        .stakes
        .keys()
        .filter(|addr| state.is_active_validator(addr))
        .cloned()
        .collect()
}

fn lower_median(mut values: Vec<u64>) -> Option<u64> {
    if values.is_empty() {
        return None;
    }
    values.sort_unstable();
    Some(values[(values.len() - 1) / 2])
}

struct PoiMeasurement {
    poi_bps: u64,
    count: usize,
    median_rtt_ms: Option<u64>,
    median_throughput_kbps: Option<u64>,
}

fn poi_for(state: &State, subject: &str, active: &BTreeSet<String>, now: u64) -> PoiMeasurement {
    let max_age = state.chain_params.attestation_max_age_secs;
    let (rtts, kbps): (Vec<u64>, Vec<u64>) = state
        .metric_attestations
        .get(subject)
        .map(|by_attester| {
            by_attester
                .iter()
                .filter(|(attester, att)| {
                    attester.as_str() != subject
                        && active.contains(attester.as_str())
                        && now.saturating_sub(att.at) <= max_age
                })
                .map(|(_, att)| (att.rtt_ms, att.throughput_kbps))
                .unzip()
        })
        .unwrap_or_default();

    let count = rtts.len();
    if count < state.chain_params.min_attestations.max(1) {
        return PoiMeasurement {
            poi_bps: UNMEASURED_POI_BPS,
            count,
            median_rtt_ms: None,
            median_throughput_kbps: None,
        };
    }

    let rtt = lower_median(rtts).unwrap_or(LATENCY_ZERO_SCORE_MS);
    // Lower median of throughput is the conservative choice.
    let throughput = lower_median(kbps).unwrap_or(0);

    let latency_score = if rtt >= LATENCY_ZERO_SCORE_MS {
        0
    } else {
        (LATENCY_ZERO_SCORE_MS - rtt) * BPS / LATENCY_ZERO_SCORE_MS
    };
    let throughput_score =
        throughput.min(THROUGHPUT_FULL_SCORE_KBPS) * BPS / THROUGHPUT_FULL_SCORE_KBPS;

    PoiMeasurement {
        poi_bps: (latency_score + throughput_score) / 2,
        count,
        median_rtt_ms: Some(rtt),
        median_throughput_kbps: Some(throughput),
    }
}

fn slashing_penalty_bps(state: &State, address: &str) -> u64 {
    state
        .slashing_records
        .iter()
        .filter(|r| r.validator == address)
        .map(|r| match r.reason {
            SlashReason::InvalidBlockProposal => 3_500,
            SlashReason::MetricFraud => 2_500,
            SlashReason::MissedBlock => 1_000,
        })
        .sum::<u64>()
        .min(BPS)
}

fn stake_weight_bps(state: &State) -> u64 {
    let sw = state.chain_params.stake_weight;
    if sw.is_nan() {
        return 0;
    }
    (sw.clamp(0.0, 1.0) * BPS as f64).round() as u64
}

/// Scores of all active validators, sorted by address. `now` is the parent block time.
pub fn validator_scores(state: &State, now: u64) -> Vec<ValidatorScore> {
    let active = active_validators(state);
    let max_stake = active
        .iter()
        .map(|a| state.get_staked_balance(a))
        .max()
        .unwrap_or(0);
    let sw = stake_weight_bps(state);

    active
        .iter()
        .map(|address| {
            let stake = state.get_staked_balance(address);
            let poi = poi_for(state, address, &active, now);
            let stats = state.validator_stats.get(address);
            let uptime_bps = stats.map(|s| s.uptime_bps).unwrap_or(INITIAL_UPTIME_BPS);
            let slashing = slashing_penalty_bps(state, address);
            let trust_bps = (4 * poi.poi_bps + 4 * uptime_bps + 2 * (BPS - slashing)) / 10;
            let stake_norm = if max_stake == 0 {
                0
            } else {
                ((stake as u128) * (BPS as u128) / (max_stake as u128)) as u64
            };
            let weight = ((BPS - sw) as u128) * (trust_bps as u128)
                + (sw as u128) * (stake_norm as u128)
                + 1;

            ValidatorScore {
                address: address.clone(),
                stake,
                poi_bps: poi.poi_bps,
                uptime_bps,
                slashing_penalty_bps: slashing,
                trust_bps,
                attestations: poi.count,
                median_rtt_ms: poi.median_rtt_ms,
                median_throughput_kbps: poi.median_throughput_kbps,
                blocks_produced: stats.map(|s| s.produced).unwrap_or(0),
                slots_missed: stats.map(|s| s.missed).unwrap_or(0),
                weight,
                jailed: false,
            }
        })
        .collect()
}

/// Active validators (as used for selection) followed by jailed validators with weight 0.
pub fn validator_report(state: &State, now: u64) -> Vec<ValidatorScore> {
    let mut report = validator_scores(state, now);
    let min = state.chain_params.min_validator_stake.max(1);
    for (address, stats) in &state.validator_stats {
        if stats.jailed && state.get_staked_balance(address) >= min {
            report.push(ValidatorScore {
                address: address.clone(),
                stake: state.get_staked_balance(address),
                poi_bps: 0,
                uptime_bps: stats.uptime_bps,
                slashing_penalty_bps: slashing_penalty_bps(state, address),
                trust_bps: 0,
                attestations: 0,
                median_rtt_ms: None,
                median_throughput_kbps: None,
                blocks_produced: stats.produced,
                slots_missed: stats.missed,
                weight: 0,
                jailed: true,
            });
        }
    }
    report
}

/// Randomness for a slot, derived from the parent block hash.
pub fn slot_seed(parent_hash: &str, slot: u64) -> u128 {
    let mut hasher = Sha256::new();
    hasher.update(b"netchain/slot-seed/v1");
    hasher.update(parent_hash.as_bytes());
    hasher.update(slot.to_be_bytes());
    let digest = hasher.finalize();
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    u128::from_be_bytes(bytes)
}

/// Weighted pick among scored validators.
pub fn pick_proposer(scores: &[ValidatorScore], seed: u128) -> Option<String> {
    let total: u128 = scores.iter().map(|s| s.weight).sum();
    if total == 0 {
        return None;
    }
    let mut pick = seed % total;
    for score in scores {
        if pick < score.weight {
            return Some(score.address.clone());
        }
        pick -= score.weight;
    }
    scores.last().map(|s| s.address.clone())
}

/// Scheduled proposer for `slot` on top of `parent` (whose post-state is `parent_state`).
pub fn proposer_for_slot(parent: &Block, parent_state: &State, slot: u64) -> Option<String> {
    let now = timestamp_secs(&parent.timestamp).max(0) as u64;
    let scores = validator_scores(parent_state, now);
    pick_proposer(&scores, slot_seed(&parent.hash, slot))
}

/// Validate a block against its parent without executing transactions.
/// `now_secs` enables the future-timestamp check (use `None` when replaying stored blocks).
pub fn check_block(
    parent: &Block,
    parent_state: &State,
    block: &Block,
    now_secs: Option<i64>,
) -> Result<(), BlockError> {
    crate::blockchain::validate_block_structure(parent, block).map_err(BlockError::Structural)?;

    let interval = parent_state.chain_params.block_interval_secs;
    let parent_slot = slot_at(timestamp_secs(&parent.timestamp), interval);
    let block_ts = timestamp_secs(&block.timestamp);
    let slot = slot_at(block_ts, interval);
    if slot <= parent_slot {
        return Err(BlockError::BadSlot { parent_slot, slot });
    }
    if let Some(now) = now_secs {
        if block_ts > now + MAX_FUTURE_DRIFT_SECS {
            return Err(BlockError::FromTheFuture {
                timestamp: block_ts,
                now,
            });
        }
    }

    let max = parent_state.chain_params.max_txs_per_block;
    if block.transactions.len() > max {
        return Err(BlockError::TooManyTransactions {
            count: block.transactions.len(),
            max,
        });
    }

    let expected = proposer_for_slot(parent, parent_state, slot).ok_or(BlockError::NoValidators)?;
    if expected != block.validator {
        return Err(BlockError::WrongProposer {
            expected,
            got: block.validator.clone(),
        });
    }
    Ok(())
}

/// Update on-chain liveness stats: the proposer produced, the skipped slots' proposers missed.
fn record_liveness(parent: &Block, parent_state: &State, block: &Block, next: &mut State) {
    let interval = parent_state.chain_params.block_interval_secs;
    let parent_slot = slot_at(timestamp_secs(&parent.timestamp), interval);
    let slot = slot_at(timestamp_secs(&block.timestamp), interval);

    let first_missed = (parent_slot + 1).max(slot.saturating_sub(MAX_MISSED_SLOTS_ACCOUNTED));
    if first_missed < slot {
        let now = timestamp_secs(&parent.timestamp).max(0) as u64;
        let scores = validator_scores(parent_state, now);
        for missed_slot in first_missed..slot {
            if let Some(addr) = pick_proposer(&scores, slot_seed(&parent.hash, missed_slot)) {
                let stats = next.validator_stats.entry(addr.clone()).or_default();
                stats.missed += 1;
                stats.uptime_bps = stats.uptime_bps * UPTIME_KEEP_PERCENT / 100;
                let should_jail = !stats.jailed && stats.uptime_bps < JAIL_UPTIME_BPS;
                // Never jail the last active validator: the chain would halt.
                if should_jail && active_validators(next).len() > 1 {
                    if let Some(stats) = next.validator_stats.get_mut(&addr) {
                        stats.jailed = true;
                    }
                }
            }
        }
    }

    let stats = next
        .validator_stats
        .entry(block.validator.clone())
        .or_default();
    stats.produced += 1;
    stats.uptime_bps =
        stats.uptime_bps * UPTIME_KEEP_PERCENT / 100 + BPS * (100 - UPTIME_KEEP_PERCENT) / 100;
    stats.last_slot = slot;
}

/// Execute a block on top of `parent_state`. Assumes `check_block` passed.
pub fn apply_block(
    parent: &Block,
    parent_state: &State,
    block: &Block,
) -> Result<BlockOutcome, BlockError> {
    let block_time = timestamp_secs(&block.timestamp).max(0) as u64;
    let mut next = parent_state.clone();
    let mut events = Vec::new();

    for (index, tx) in block.transactions.iter().enumerate() {
        match next.apply_transaction_at(tx, block_time) {
            Ok(Some(event)) => events.push(event),
            Ok(None) => {}
            Err(e) => {
                return Err(BlockError::InvalidTransaction {
                    index,
                    reason: format!("{e:?}"),
                })
            }
        }
    }

    let reward = next.chain_params.block_reward;
    next.apply_block_rewards(&block.validator, &block.transactions, reward);
    record_liveness(parent, parent_state, block, &mut next);
    let executed_proposals = next.execute_passed_proposals_at(block_time);

    Ok(BlockOutcome {
        state: next,
        events,
        executed_proposals,
    })
}

/// Full validation + execution of a block received from the network or disk.
pub fn import_block(
    parent: &Block,
    parent_state: &State,
    block: &Block,
    now_secs: Option<i64>,
) -> Result<BlockOutcome, BlockError> {
    check_block(parent, parent_state, block, now_secs)?;
    apply_block(parent, parent_state, block)
}

/// Build and sign a block for `slot`. Transactions that fail to apply are dropped.
pub fn build_block(
    parent: &Block,
    parent_state: &State,
    candidate_txs: Vec<SignedTransaction>,
    key: &SigningKey,
    timestamp: DateTime<Utc>,
) -> Result<(Block, BlockOutcome), BlockError> {
    let block_time = timestamp_secs(&timestamp).max(0) as u64;

    // Keep only transactions that apply cleanly in order.
    let mut scratch = parent_state.clone();
    let max = parent_state.chain_params.max_txs_per_block;
    let mut txs = Vec::with_capacity(candidate_txs.len().min(max));
    for tx in candidate_txs {
        if txs.len() >= max {
            break;
        }
        if scratch.apply_transaction_at(&tx, block_time).is_ok() {
            txs.push(tx);
        }
    }

    let block = Block::new_signed(parent.index + 1, txs, parent.hash.clone(), timestamp, key);
    let outcome = import_block(parent, parent_state, &block, None)?;
    Ok((block, outcome))
}

/// Replay a chain from genesis (`blocks[0]` must be the genesis block) and return the state
/// after the last block. Fails with the index of the first invalid block.
pub fn replay_chain(genesis_state: &State, blocks: &[Block]) -> Result<State, (u64, BlockError)> {
    let mut state = genesis_state.clone();
    for window in blocks.windows(2) {
        let (parent, block) = (&window[0], &window[1]);
        state = import_block(parent, &state, block, None)
            .map_err(|e| (block.index, e))?
            .state;
    }
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genesis::{GenesisAllocation, GenesisSpec, GenesisValidator};
    use crate::state::ChainParams;
    use crate::transaction::{generate_ed25519_keypair, pubkey_to_address_hex, Transaction};
    use chrono::TimeZone;

    struct Net {
        keys: Vec<SigningKey>,
        spec: GenesisSpec,
    }

    fn addr(key: &SigningKey) -> String {
        pubkey_to_address_hex(&key.verifying_key())
    }

    fn net(validators: usize) -> Net {
        let keys: Vec<SigningKey> = (0..validators)
            .map(|_| generate_ed25519_keypair())
            .collect();
        let spec = GenesisSpec {
            chain_id: "test".into(),
            timestamp: 1_700_000_000,
            allocations: keys
                .iter()
                .map(|k| GenesisAllocation {
                    address: addr(k),
                    balance: 10_000,
                })
                .collect(),
            validators: keys
                .iter()
                .map(|k| GenesisValidator {
                    address: addr(k),
                    stake: 1_000,
                })
                .collect(),
            params: ChainParams::default(),
        };
        spec.validate().unwrap();
        Net { keys, spec }
    }

    impl Net {
        fn key_for(&self, address: &str) -> &SigningKey {
            self.keys.iter().find(|k| addr(k) == address).unwrap()
        }

        /// Produce the next block in the first slot after `parent` whose proposer we control.
        fn next_block(
            &self,
            parent: &Block,
            state: &State,
            txs: Vec<SignedTransaction>,
        ) -> (Block, BlockOutcome) {
            let interval = state.chain_params.block_interval_secs;
            let slot = slot_at(parent.timestamp.timestamp(), interval) + 1;
            let proposer = proposer_for_slot(parent, state, slot).unwrap();
            let ts = Utc.timestamp_opt(slot_start(slot, interval), 0).unwrap();
            build_block(parent, state, txs, self.key_for(&proposer), ts).unwrap()
        }
    }

    #[test]
    fn selection_is_deterministic_and_covers_validators() {
        let n = net(4);
        let state = n.spec.build_state();
        let genesis = n.spec.genesis_block();
        let mut seen = BTreeSet::new();
        for slot in 0..200 {
            let a = proposer_for_slot(&genesis, &state, slot).unwrap();
            let b = proposer_for_slot(&genesis, &state, slot).unwrap();
            assert_eq!(a, b);
            seen.insert(a);
        }
        assert_eq!(seen.len(), 4, "every validator should get slots");
    }

    #[test]
    fn chain_of_signed_blocks_imports_and_replays() {
        let n = net(3);
        let genesis = n.spec.genesis_block();
        let mut blocks = vec![genesis.clone()];
        let mut state = n.spec.build_state();

        let sender = &n.keys[0];
        let tx = Transaction::new(addr(sender), "f".repeat(40), 25, 1, 0, None);
        let signed = SignedTransaction::sign_with_keypair(&tx, sender);

        for i in 0..5 {
            let txs = if i == 0 { vec![signed.clone()] } else { vec![] };
            let (block, outcome) = n.next_block(blocks.last().unwrap(), &state, txs);
            state = outcome.state;
            blocks.push(block);
        }
        assert_eq!(state.get_balance(&"f".repeat(40)), 25);

        let last = replay_chain(&n.spec.build_state(), &blocks).unwrap();
        assert_eq!(last.get_balance(&"f".repeat(40)), 25);
        let produced: u64 = last.validator_stats.values().map(|s| s.produced).sum();
        assert_eq!(produced, 5);
    }

    #[test]
    fn rejects_block_from_wrong_proposer() {
        let n = net(3);
        let genesis = n.spec.genesis_block();
        let state = n.spec.build_state();
        let interval = state.chain_params.block_interval_secs;
        let slot = slot_at(genesis.timestamp.timestamp(), interval) + 1;
        let proposer = proposer_for_slot(&genesis, &state, slot).unwrap();
        let wrong = n.keys.iter().find(|k| addr(k) != proposer).unwrap();
        let ts = Utc.timestamp_opt(slot_start(slot, interval), 0).unwrap();
        let block = Block::new_signed(1, vec![], genesis.hash.clone(), ts, wrong);
        assert!(matches!(
            import_block(&genesis, &state, &block, None),
            Err(BlockError::WrongProposer { .. })
        ));
    }

    #[test]
    fn rejects_tampered_and_unsigned_blocks() {
        let n = net(1);
        let genesis = n.spec.genesis_block();
        let state = n.spec.build_state();
        let (mut block, _) = n.next_block(&genesis, &state, vec![]);
        block.validator = "e".repeat(40);
        assert!(matches!(
            import_block(&genesis, &state, &block, None),
            Err(BlockError::Structural(_))
        ));

        let (mut block, _) = n.next_block(&genesis, &state, vec![]);
        block.signature.clear();
        assert!(matches!(
            import_block(&genesis, &state, &block, None),
            Err(BlockError::Structural(_))
        ));
    }

    #[test]
    fn rejects_same_slot_and_future_blocks() {
        let n = net(1);
        let genesis = n.spec.genesis_block();
        let state = n.spec.build_state();
        let block = Block::new_signed(
            1,
            vec![],
            genesis.hash.clone(),
            genesis.timestamp,
            &n.keys[0],
        );
        assert!(matches!(
            import_block(&genesis, &state, &block, None),
            Err(BlockError::BadSlot { .. })
        ));

        let (block, _) = n.next_block(&genesis, &state, vec![]);
        let now = genesis.timestamp.timestamp();
        assert!(matches!(
            import_block(&genesis, &state, &block, Some(now - 100)),
            Err(BlockError::FromTheFuture { .. })
        ));
    }

    #[test]
    fn missed_slots_reduce_uptime() {
        let n = net(2);
        let genesis = n.spec.genesis_block();
        let state = n.spec.build_state();
        let interval = state.chain_params.block_interval_secs;
        let base = slot_at(genesis.timestamp.timestamp(), interval);

        // Find a slot whose proposer differs from slot base+1's proposer and skip to it.
        let first = proposer_for_slot(&genesis, &state, base + 1).unwrap();
        let (skip_to, proposer) = (base + 2..base + 100)
            .map(|s| (s, proposer_for_slot(&genesis, &state, s).unwrap()))
            .find(|(_, p)| *p != first)
            .unwrap();
        let ts = Utc.timestamp_opt(slot_start(skip_to, interval), 0).unwrap();
        let (_, outcome) = build_block(&genesis, &state, vec![], n.key_for(&proposer), ts).unwrap();
        let stats = &outcome.state.validator_stats[&first];
        assert!(stats.missed >= 1);
        assert!(stats.uptime_bps < INITIAL_UPTIME_BPS);
    }

    #[test]
    fn offline_validator_gets_jailed_and_can_unjail() {
        let n = net(3);
        let genesis = n.spec.genesis_block();
        let mut state = n.spec.build_state();
        let offline = addr(&n.keys[2]);
        let interval = state.chain_params.block_interval_secs;
        let mut parent = genesis.clone();
        let mut slot = slot_at(genesis.timestamp.timestamp(), interval);

        // Run until the offline validator is jailed: its slots are simply skipped.
        for _ in 0..500 {
            slot += 1;
            let proposer = proposer_for_slot(&parent, &state, slot).unwrap();
            if proposer == offline {
                continue;
            }
            let ts = Utc.timestamp_opt(slot_start(slot, interval), 0).unwrap();
            let (block, outcome) =
                build_block(&parent, &state, vec![], n.key_for(&proposer), ts).unwrap();
            state = outcome.state;
            parent = block;
            if state.is_jailed(&offline) {
                break;
            }
        }
        assert!(state.is_jailed(&offline));
        assert!(!active_validators(&state).contains(&offline));

        // Once jailed it is never scheduled.
        for s in slot + 1..slot + 200 {
            assert_ne!(proposer_for_slot(&parent, &state, s).unwrap(), offline);
        }

        // Unjail brings it back.
        let tx = Transaction::unjail(offline.clone(), 1, 0);
        let signed = SignedTransaction::sign_with_keypair(&tx, &n.keys[2]);
        state.apply_transaction_at(&signed, 1).unwrap();
        assert!(active_validators(&state).contains(&offline));
    }

    #[test]
    fn last_validator_is_never_jailed() {
        let n = net(1);
        let genesis = n.spec.genesis_block();
        let state = n.spec.build_state();
        let interval = state.chain_params.block_interval_secs;
        // Skip 64 slots: the only validator misses all of them but stays active.
        let slot = slot_at(genesis.timestamp.timestamp(), interval) + 65;
        let ts = Utc.timestamp_opt(slot_start(slot, interval), 0).unwrap();
        let (_, outcome) = build_block(&genesis, &state, vec![], &n.keys[0], ts).unwrap();
        assert!(!outcome.state.is_jailed(&addr(&n.keys[0])));
    }

    #[test]
    fn attestations_drive_poi_score() {
        let n = net(3);
        let mut state = n.spec.build_state();
        let subject = addr(&n.keys[0]);
        for (i, attester) in n.keys[1..].iter().enumerate() {
            state
                .metric_attestations
                .entry(subject.clone())
                .or_default()
                .insert(
                    addr(attester),
                    crate::state::MetricAttestation {
                        rtt_ms: 50 + i as u64 * 10,
                        throughput_kbps: 100_000,
                        at: 1_700_000_000,
                    },
                );
        }
        let scores = validator_scores(&state, 1_700_000_100);
        let s = scores.iter().find(|s| s.address == subject).unwrap();
        assert_eq!(s.attestations, 2);
        assert_eq!(s.median_rtt_ms, Some(50));
        assert!(s.poi_bps > UNMEASURED_POI_BPS);

        // Stale attestations no longer count.
        let scores = validator_scores(&state, 1_700_000_000 + 100_000);
        let s = scores.iter().find(|s| s.address == subject).unwrap();
        assert_eq!(s.poi_bps, UNMEASURED_POI_BPS);
    }
}
