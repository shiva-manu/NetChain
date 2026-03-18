// src/poi/consensus.rs
use rand::Rng; // keep for testing helpers only
use serde::{Deserialize, Serialize}; // For config serialization (optional)
use std::collections::HashMap;

/// Config for PoI weights and thresholds (load from TOML/JSON)
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PoiConfig {
    pub weights: Weights,
    pub thresholds: Thresholds,
    /// How much stake influences validator selection vs hybrid trust score (0.0 = pure trust, 1.0 = pure stake)
    #[serde(default = "default_stake_weight")]
    pub stake_weight: f64,
    /// Weights for the hybrid trust score.
    #[serde(default)]
    pub hybrid: HybridWeights,
}

fn default_stake_weight() -> f64 {
    0.3
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct HybridWeights {
    /// PoI contribution to the trust score.
    pub poi: f64,
    /// Long-term reputation contribution.
    pub reputation: f64,
    /// Identity confidence contribution.
    pub identity: f64,
    /// Multi-party attestation volume contribution.
    pub attestation: f64,
    /// Recent slashing penalty contribution.
    pub slashing: f64,
    /// Minimum attestation count used to normalize the attestation factor.
    pub min_attestations: usize,
}

impl Default for HybridWeights {
    fn default() -> Self {
        Self {
            poi: 0.35,
            reputation: 0.25,
            identity: 0.15,
            attestation: 0.15,
            slashing: 0.10,
            min_attestations: 3,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Weights {
    pub upload: f64,    // e.g., 0.25
    pub download: f64,  // e.g., 0.25
    pub latency: f64,   // e.g., 0.20 (lower latency = higher score)
    pub uptime: f64,    // e.g., 0.20
    pub stability: f64, // e.g., 0.10
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Thresholds {
    pub upload_mbps: f64,       // Max for normalization, e.g., 100.0
    pub download_mbps: f64,     // e.g., 1000.0
    pub latency_ms: f64,        // Max penalty at this, e.g., 200.0
    pub uptime_percent: f64,    // Max, e.g., 100.0
    pub stability_percent: f64, // Packet success rate, e.g., 100.0
}

/// Node's internet metrics (self-reported or proven via P2P challenges)
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct NodeMetrics {
    pub node_id: String, // e.g., pubkey hash
    pub upload_mbps: f64,
    pub download_mbps: f64,
    pub latency_ms: f64,        // Avg RTT to peers
    pub uptime_percent: f64,    // Over last epoch (e.g., 99.5)
    pub stability_percent: f64, // % successful packets
    pub identity_score: f64,
    pub reputation_score: f64,
    pub attestation_count: usize,
    pub unique_attester_count: usize,
    pub slashing_penalty: f64,
}

impl NodeMetrics {
    pub fn with_baseline(
        node_id: String,
        upload_mbps: f64,
        download_mbps: f64,
        latency_ms: f64,
        uptime_percent: f64,
        stability_percent: f64,
    ) -> Self {
        Self {
            node_id,
            upload_mbps,
            download_mbps,
            latency_ms,
            uptime_percent,
            stability_percent,
            ..Self::default()
        }
    }

    /// Normalize a value: (val / max) clamped to [0.0, 1.0]
    fn normalize(_self: &Self, val: f64, max: f64) -> f64 {
        if max <= 0.0 {
            return 0.0;
        }
        (val / max).clamp(0.0, 1.0)
    }

    /// Inverted normalize for penalties (e.g., latency: higher = worse)
    fn invert_normalize(_self: &Self, val: f64, max: f64) -> f64 {
        1.0 - NodeMetrics::normalize(_self, val, max)
    }

    fn attestation_ratio(&self, min_attestations: usize) -> f64 {
        if min_attestations == 0 {
            return 1.0;
        }

        (self.unique_attester_count as f64 / min_attestations as f64).clamp(0.0, 1.0)
    }
}

impl Default for NodeMetrics {
    fn default() -> Self {
        Self {
            node_id: String::new(),
            upload_mbps: 0.0,
            download_mbps: 0.0,
            latency_ms: 0.0,
            uptime_percent: 0.0,
            stability_percent: 0.0,
            identity_score: 0.5,
            reputation_score: 0.5,
            attestation_count: 0,
            unique_attester_count: 0,
            slashing_penalty: 0.0,
        }
    }
}

/// PoI Scorer: Main engine for computing importance scores
#[derive(Debug, Clone)]
pub struct PoiScorer {
    config: PoiConfig,
}

impl PoiScorer {
    pub fn new(config: PoiConfig) -> Self {
        Self { config }
    }

    pub fn set_stake_weight(&mut self, stake_weight: f64) {
        self.config.stake_weight = stake_weight.clamp(0.0, 1.0);
    }

    pub fn set_hybrid_min_attestations(&mut self, min_attestations: usize) {
        self.config.hybrid.min_attestations = min_attestations.max(1);
    }

    pub fn set_hybrid_weights(&mut self, hybrid: HybridWeights) {
        self.config.hybrid = hybrid;
    }

    pub fn hybrid_weights(&self) -> &HybridWeights {
        &self.config.hybrid
    }

    /// Compute PoI score for a node (0.0 = useless, 1.0 = god-tier connection)
    pub fn poi_score(&self, metrics: &NodeMetrics) -> f64 {
        // Weighted sum of normalized metrics
        let upload_norm = NodeMetrics::normalize(
            metrics,
            metrics.upload_mbps,
            self.config.thresholds.upload_mbps,
        );
        let download_norm = NodeMetrics::normalize(
            metrics,
            metrics.download_mbps,
            self.config.thresholds.download_mbps,
        );
        let latency_norm = NodeMetrics::invert_normalize(
            metrics,
            metrics.latency_ms,
            self.config.thresholds.latency_ms,
        );
        let uptime_norm = NodeMetrics::normalize(
            metrics,
            metrics.uptime_percent,
            self.config.thresholds.uptime_percent,
        );
        let stability_norm = NodeMetrics::normalize(
            metrics,
            metrics.stability_percent,
            self.config.thresholds.stability_percent,
        );

        let score = self.config.weights.upload * upload_norm
            + self.config.weights.download * download_norm
            + self.config.weights.latency * latency_norm
            + self.config.weights.uptime * uptime_norm
            + self.config.weights.stability * stability_norm;

        // Clamp to 0..=1 and return
        score.clamp(0.0, 1.0)
    }

    /// Compute the hybrid trust score from PoI, reputation, identity, attestations, and slashing.
    fn hybrid_score(&self, metrics: &NodeMetrics) -> f64 {
        let hybrid = &self.config.hybrid;
        let total_weight =
            hybrid.poi + hybrid.reputation + hybrid.identity + hybrid.attestation + hybrid.slashing;

        if total_weight <= f64::EPSILON {
            return self.poi_score(metrics);
        }

        let poi = self.poi_score(metrics);
        let reputation = metrics.reputation_score.clamp(0.0, 1.0);
        let identity = metrics.identity_score.clamp(0.0, 1.0);
        let attestation = metrics.attestation_ratio(hybrid.min_attestations);
        let slashing = (1.0 - metrics.slashing_penalty).clamp(0.0, 1.0);

        (hybrid.poi * poi
            + hybrid.reputation * reputation
            + hybrid.identity * identity
            + hybrid.attestation * attestation
            + hybrid.slashing * slashing)
            / total_weight
    }

    /// Compute combined weight for a validator: blends hybrid trust score with normalized stake.
    /// combined = (1 - stake_weight) * trust_score + stake_weight * (stake / max_stake)
    /// If no one has stake, falls back to pure trust.
    pub fn combined_weight(&self, metrics: &NodeMetrics, stake: u64, max_stake: u64) -> f64 {
        let trust = self.hybrid_score(metrics);
        let stake_norm = if max_stake > 0 {
            (stake as f64) / (max_stake as f64)
        } else {
            0.0
        };
        let sw = self.config.stake_weight.clamp(0.0, 1.0);
        ((1.0 - sw) * trust + sw * stake_norm).clamp(0.0, 1.0)
    }

    /// Deterministic selection: choose validator using a shared `seed_u128`.
    /// Accepts optional stake data to blend stake weight into selection.
    /// IMPORTANT: `seed_u128` must be derived the same way on all nodes for determinism.
    /// Example: u128::from_be_bytes(sha256(previous_block_hash || epoch) [0..16])
    pub fn select_validator_with_seed(
        &self,
        pool: &HashMap<String, NodeMetrics>,
        seed_u128: u128,
    ) -> String {
        self.select_validator_with_seed_and_stakes(pool, seed_u128, &HashMap::new())
    }

    /// Deterministic selection that factors in stake weights.
    /// `stakes` maps node_id -> staked amount. Nodes not in the map are treated as 0 stake.
    pub fn select_validator_with_seed_and_stakes(
        &self,
        pool: &HashMap<String, NodeMetrics>,
        seed_u128: u128,
        stakes: &HashMap<String, u64>,
    ) -> String {
        if pool.is_empty() {
            panic!("No validators in pool!");
        }

        // CRITICAL: Sort by node_id for deterministic iteration across all nodes.
        let mut sorted_entries: Vec<(&String, &NodeMetrics)> = pool.iter().collect();
        sorted_entries.sort_by(|(a, _), (b, _)| a.cmp(b));

        // Find max stake for normalization
        let max_stake = stakes.values().copied().max().unwrap_or(0);

        // Compute cumulative weights in deterministic (sorted) order
        let mut cum_weights: Vec<(String, f64)> = Vec::with_capacity(pool.len());
        let mut total_weight = 0.0f64;
        for (id, metrics) in &sorted_entries {
            let stake = stakes.get(*id).copied().unwrap_or(0);
            let score = self.combined_weight(metrics, stake, max_stake);
            let weight = score * 1_000.0;
            total_weight += weight;
            cum_weights.push(((*id).clone(), total_weight));
        }

        // If total weight is zero (all scores zero), fallback deterministically using sorted order + seed
        if total_weight <= f64::EPSILON {
            let idx = (seed_u128 as usize) % sorted_entries.len();
            return sorted_entries[idx].0.clone();
        }

        // Convert seed to fractional in [0,1)
        let seed_frac = (seed_u128 as f64) / (u128::MAX as f64);
        let pick = seed_frac * total_weight;

        // Find first cumulative weight greater than pick
        let idx = cum_weights
            .iter()
            .position(|(_, cum)| pick < *cum)
            .expect("position must exist when total_weight > 0");

        cum_weights[idx].0.clone()
    }

    /// Non-deterministic RNG helper (ONLY for local tests). For consensus use deterministic seed.
    pub fn select_validator_rng<R: Rng>(
        &self,
        pool: &HashMap<String, NodeMetrics>,
        rng: &mut R,
    ) -> String {
        if pool.is_empty() {
            panic!("No validators in pool!");
        }

        // Sort for consistent behavior even in test helper
        let mut sorted_entries: Vec<(&String, &NodeMetrics)> = pool.iter().collect();
        sorted_entries.sort_by(|(a, _), (b, _)| a.cmp(b));

        let mut cum_weights: Vec<(String, f64)> = Vec::with_capacity(pool.len());
        let mut total_weight = 0.0f64;
        for (id, metrics) in &sorted_entries {
            let score = self.hybrid_score(metrics).max(0.0);
            let weight = score * 1_000.0;
            total_weight += weight;
            cum_weights.push(((*id).clone(), total_weight));
        }

        if total_weight <= f64::EPSILON {
            return sorted_entries[0].0.clone();
        }

        let pick = rng.gen_range(0.0..total_weight);
        let idx = cum_weights
            .iter()
            .position(|(_, cum)| pick < *cum)
            .expect("position must exist when total_weight > 0");
        cum_weights[idx].0.clone()
    }

    /// Epoch update: Re-score all nodes (call every N blocks)
    pub fn update_epoch(
        &mut self,
        pool: &mut HashMap<String, NodeMetrics>,
    ) -> HashMap<String, f64> {
        pool.iter()
            .map(|(id, metrics)| (id.clone(), self.hybrid_score(metrics)))
            .collect()
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    fn build_test_config() -> PoiConfig {
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
            stake_weight: 0.0, // Pure PoI for backward-compatible tests
            hybrid: HybridWeights {
                poi: 1.0,
                reputation: 0.0,
                identity: 0.0,
                attestation: 0.0,
                slashing: 0.0,
                min_attestations: 3,
            },
        }
    }

    #[test]
    fn test_poi_score_perfect_node() {
        let config = build_test_config();
        let scorer = PoiScorer::new(config);
        let metrics = NodeMetrics {
            node_id: "test".to_string(),
            upload_mbps: 100.0,
            download_mbps: 1000.0,
            latency_ms: 0.0,
            uptime_percent: 100.0,
            stability_percent: 100.0,
            ..Default::default()
        };
        let score = scorer.poi_score(&metrics);
        // Use approximate comparison due to floating point precision
        assert!((score - 1.0).abs() < f64::EPSILON * 10.0);
    }

    #[test]
    fn test_select_validator_deterministic() {
        let config = build_test_config();
        let scorer = PoiScorer::new(config);
        let mut pool: HashMap<String, NodeMetrics> = HashMap::new();

        // Node A: Best
        pool.insert(
            "A".to_string(),
            NodeMetrics {
                node_id: "A".to_string(),
                upload_mbps: 90.0,
                download_mbps: 900.0,
                latency_ms: 5.0,
                uptime_percent: 99.9,
                stability_percent: 99.9,
                ..Default::default()
            },
        );

        // Node B: Medium
        pool.insert(
            "B".to_string(),
            NodeMetrics {
                node_id: "B".to_string(),
                upload_mbps: 40.0,
                download_mbps: 400.0,
                latency_ms: 50.0,
                uptime_percent: 98.0,
                stability_percent: 97.0,
                ..Default::default()
            },
        );

        // Node C: Poor
        pool.insert(
            "C".to_string(),
            NodeMetrics {
                node_id: "C".to_string(),
                upload_mbps: 1.0,
                download_mbps: 10.0,
                latency_ms: 180.0,
                uptime_percent: 80.0,
                stability_percent: 70.0,
                ..Default::default()
            },
        );

        // Use a fixed seed; the result must be deterministic across calls
        let seed: u128 = 0x123456789abcdef0u128;
        let winner1 = scorer.select_validator_with_seed(&pool, seed);
        let winner2 = scorer.select_validator_with_seed(&pool, seed);
        assert_eq!(
            winner1, winner2,
            "Same seed must always produce same validator"
        );

        // Different seeds may produce different validators (but result must be valid)
        let winner3 = scorer.select_validator_with_seed(&pool, 0u128);
        assert!(["A", "B", "C"].contains(&winner3.as_str()));

        // Run 100 times with different seeds to verify consistency (no panics, always valid)
        for seed in 0u128..100 {
            let w = scorer.select_validator_with_seed(&pool, seed);
            assert!(["A", "B", "C"].contains(&w.as_str()));
        }

        // Also test rng helper (local only)
        let mut rng = thread_rng();
        let w2 = scorer.select_validator_rng(&pool, &mut rng);
        assert!(["A", "B", "C"].contains(&w2.as_str()));
    }

    #[test]
    fn test_select_validator_all_zero_weights() {
        let mut config = build_test_config();
        // set thresholds such that metrics normalize to 0 (make thresholds tiny)
        config.thresholds.upload_mbps = 0.0001;
        config.thresholds.download_mbps = 0.0001;
        config.thresholds.latency_ms = 0.0001;
        config.thresholds.uptime_percent = 0.0001;
        config.thresholds.stability_percent = 0.0001;
        config.stake_weight = 0.0;

        let scorer = PoiScorer::new(config);
        let mut pool: HashMap<String, NodeMetrics> = HashMap::new();
        pool.insert(
            "x".to_string(),
            NodeMetrics {
                node_id: "x".to_string(),
                upload_mbps: 0.0,
                download_mbps: 0.0,
                latency_ms: 0.0,
                uptime_percent: 0.0,
                stability_percent: 0.0,
                ..Default::default()
            },
        );
        pool.insert(
            "y".to_string(),
            NodeMetrics {
                node_id: "y".to_string(),
                upload_mbps: 0.0,
                download_mbps: 0.0,
                latency_ms: 0.0,
                uptime_percent: 0.0,
                stability_percent: 0.0,
                ..Default::default()
            },
        );

        // Deterministic fallback must return one of them and be deterministic
        let seed = 42u128;
        let winner = scorer.select_validator_with_seed(&pool, seed);
        assert!(["x", "y"].contains(&winner.as_str()));
    }

    #[test]
    fn test_stake_weighted_selection_favors_higher_stake() {
        let mut config = build_test_config();
        config.stake_weight = 1.0; // Pure stake-based selection

        let scorer = PoiScorer::new(config);
        let mut pool: HashMap<String, NodeMetrics> = HashMap::new();

        // All nodes have identical PoI metrics
        for id in &["A", "B", "C"] {
            pool.insert(
                id.to_string(),
                NodeMetrics {
                    node_id: id.to_string(),
                    upload_mbps: 50.0,
                    download_mbps: 500.0,
                    latency_ms: 30.0,
                    uptime_percent: 99.0,
                    stability_percent: 98.0,
                    ..Default::default()
                },
            );
        }

        let mut stakes = HashMap::new();
        stakes.insert("A".to_string(), 100u64);
        stakes.insert("B".to_string(), 1000u64); // B has 10x more stake
        stakes.insert("C".to_string(), 10u64);

        // Run 1000 selections with different seeds and count wins
        let mut wins: HashMap<String, u32> = HashMap::new();
        for i in 0u128..1000 {
            // Spread seeds across the full u128 range for proper coverage
            let seed = i.wrapping_mul(u128::MAX / 1000);
            let winner = scorer.select_validator_with_seed_and_stakes(&pool, seed, &stakes);
            *wins.entry(winner).or_insert(0) += 1;
        }

        // B should win significantly more often than A or C due to higher stake
        let b_wins = *wins.get("B").unwrap_or(&0);
        let a_wins = *wins.get("A").unwrap_or(&0);
        let c_wins = *wins.get("C").unwrap_or(&0);
        assert!(
            b_wins > a_wins && b_wins > c_wins,
            "B (stake=1000) should win most: A={}, B={}, C={}",
            a_wins,
            b_wins,
            c_wins,
        );
    }

    #[test]
    fn test_combined_weight_blending() {
        let mut config = build_test_config();
        config.stake_weight = 0.5; // Equal blend

        let scorer = PoiScorer::new(config);

        // Perfect PoI metrics, no stake
        let metrics = NodeMetrics {
            node_id: "test".to_string(),
            upload_mbps: 100.0,
            download_mbps: 1000.0,
            latency_ms: 0.0,
            uptime_percent: 100.0,
            stability_percent: 100.0,
            ..Default::default()
        };

        // With 0 stake and max_stake > 0, stake component is 0
        let w1 = scorer.combined_weight(&metrics, 0, 1000);
        assert!(
            (w1 - 0.5).abs() < 0.01,
            "Expected ~0.5 (poi=1.0*0.5 + stake=0*0.5), got {}",
            w1
        );

        // With max stake, both components are 1.0
        let w2 = scorer.combined_weight(&metrics, 1000, 1000);
        assert!((w2 - 1.0).abs() < 0.01, "Expected ~1.0, got {}", w2);

        // With half stake
        let w3 = scorer.combined_weight(&metrics, 500, 1000);
        assert!(
            (w3 - 0.75).abs() < 0.01,
            "Expected ~0.75 (0.5 + 0.25), got {}",
            w3
        );
    }

    #[test]
    fn test_attestation_ratio_uses_unique_attesters() {
        let metrics = NodeMetrics {
            node_id: "test".to_string(),
            attestation_count: 6,
            unique_attester_count: 2,
            ..Default::default()
        };

        assert!((metrics.attestation_ratio(3) - (2.0 / 3.0)).abs() < f64::EPSILON * 10.0);
    }

    #[test]
    fn test_stake_weighted_selection_deterministic() {
        let mut config = build_test_config();
        config.stake_weight = 0.5;

        let scorer = PoiScorer::new(config);
        let mut pool: HashMap<String, NodeMetrics> = HashMap::new();
        pool.insert(
            "A".to_string(),
            NodeMetrics {
                node_id: "A".to_string(),
                upload_mbps: 50.0,
                download_mbps: 500.0,
                latency_ms: 30.0,
                uptime_percent: 99.0,
                stability_percent: 98.0,
                ..Default::default()
            },
        );
        pool.insert(
            "B".to_string(),
            NodeMetrics {
                node_id: "B".to_string(),
                upload_mbps: 50.0,
                download_mbps: 500.0,
                latency_ms: 30.0,
                uptime_percent: 99.0,
                stability_percent: 98.0,
                ..Default::default()
            },
        );

        let mut stakes = HashMap::new();
        stakes.insert("A".to_string(), 500u64);
        stakes.insert("B".to_string(), 200u64);

        let seed = 0xdeadbeef_u128;
        let w1 = scorer.select_validator_with_seed_and_stakes(&pool, seed, &stakes);
        let w2 = scorer.select_validator_with_seed_and_stakes(&pool, seed, &stakes);
        assert_eq!(w1, w2, "Same seed + stakes must produce same result");
    }
}
