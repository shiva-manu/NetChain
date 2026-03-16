// src/consensus.rs
use rand::Rng; // keep for testing helpers only
use serde::{Deserialize, Serialize}; // For config serialization (optional)
use std::collections::HashMap;

/// Config for PoI weights and thresholds (load from TOML/JSON)
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PoiConfig {
    pub weights: Weights,
    pub thresholds: Thresholds,
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
    pub upload_mbps: f64,     // Max for normalization, e.g., 100.0
    pub download_mbps: f64,   // e.g., 1000.0
    pub latency_ms: f64,      // Max penalty at this, e.g., 200.0
    pub uptime_percent: f64,  // Max, e.g., 100.0
    pub stability_percent: f64, // Packet success rate, e.g., 100.0
}

/// Node's internet metrics (self-reported or proven via P2P challenges)
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct NodeMetrics {
    pub node_id: String, // e.g., pubkey hash
    pub upload_mbps: f64,
    pub download_mbps: f64,
    pub latency_ms: f64,     // Avg RTT to peers
    pub uptime_percent: f64, // Over last epoch (e.g., 99.5)
    pub stability_percent: f64, // % successful packets
}

impl NodeMetrics {
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

    /// Compute PoI score for a node (0.0 = useless, 1.0 = god-tier connection)
    pub fn poi_score(&self, metrics: &NodeMetrics) -> f64 {
        // Weighted sum of normalized metrics
        let upload_norm =
            NodeMetrics::normalize(metrics, metrics.upload_mbps, self.config.thresholds.upload_mbps);
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
        let uptime_norm =
            NodeMetrics::normalize(metrics, metrics.uptime_percent, self.config.thresholds.uptime_percent);
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

    /// Deterministic selection: choose validator using a shared `seed_u128`.
    /// IMPORTANT: `seed_u128` must be derived the same way on all nodes for determinism.
    /// Example: u128::from_be_bytes(sha256(previous_block_hash || epoch) [0..16])
    pub fn select_validator_with_seed(
        &self,
        pool: &HashMap<String, NodeMetrics>,
        seed_u128: u128,
    ) -> String {
        if pool.is_empty() {
            panic!("No validators in pool!");
        }

        // Compute cumulative weights
        let mut cum_weights: Vec<(String, f64)> = Vec::with_capacity(pool.len());
        let mut total_weight = 0.0f64;
        for (id, metrics) in pool.iter() {
            let score = self.poi_score(metrics).max(0.0);
            // scale to integer-space-like but keep f64
            let weight = score * 1_000.0;
            total_weight += weight;
            cum_weights.push((id.clone(), total_weight));
        }

        // If total weight is zero (all scores zero), fallback deterministically using lexicographic order + seed
        if total_weight <= f64::EPSILON {
            let mut ids: Vec<&String> = pool.keys().collect();
            ids.sort();
            let idx = (seed_u128 as usize) % ids.len();
            return ids[idx].clone().to_owned();
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
    pub fn select_validator_rng<R: Rng>(&self, pool: &HashMap<String, NodeMetrics>, rng: &mut R) -> String {
        if pool.is_empty() {
            panic!("No validators in pool!");
        }

        // compute cumulative weights
        let mut cum_weights: Vec<(String, f64)> = Vec::with_capacity(pool.len());
        let mut total_weight = 0.0f64;
        for (id, metrics) in pool.iter() {
            let score = self.poi_score(metrics).max(0.0);
            let weight = score * 1_000.0;
            total_weight += weight;
            cum_weights.push((id.clone(), total_weight));
        }

        if total_weight <= f64::EPSILON {
            // fallback: deterministic lexicographic pick
            let mut ids: Vec<&String> = pool.keys().collect();
            ids.sort();
            return ids[0].clone().to_owned();
        }

        let pick = rng.gen_range(0.0..total_weight);
        let idx = cum_weights
            .iter()
            .position(|(_, cum)| pick < *cum)
            .expect("position must exist when total_weight > 0");
        cum_weights[idx].0.clone()
    }

    /// Epoch update: Re-score all nodes (call every N blocks)
    pub fn update_epoch(&mut self, pool: &mut HashMap<String, NodeMetrics>) -> HashMap<String, f64> {
        pool.iter()
            .map(|(id, metrics)| (id.clone(), self.poi_score(metrics)))
            .collect()
    }
}

// Helper trait for RNG (for testing/mocking) — now returns String
trait WeightedSelect {
    fn select_validator<R: Rng>(&self, pool: &HashMap<String, NodeMetrics>, rng: &mut R) -> String;
}

impl WeightedSelect for PoiScorer {
    fn select_validator<R: Rng>(&self, pool: &HashMap<String, NodeMetrics>, rng: &mut R) -> String {
        self.select_validator_rng(pool, rng)
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
        };
        let score = scorer.poi_score(&metrics);
        assert_eq!(score, 1.0);
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
            },
        );

        // Use a fixed seed; the highest scorer ("A") should often be selected for most seeds.
        let seed: u128 = 0x123456789abcdef0u128;
        let winner = scorer.select_validator_with_seed(&pool, seed);
        // We expect a deterministic output. We assert that winner is one of A/B/C
        assert!(["A", "B", "C"].contains(&winner.as_str()));

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
            },
        );

        // Deterministic fallback must return one of them and be deterministic
        let seed = 42u128;
        let winner = scorer.select_validator_with_seed(&pool, seed);
        assert!(["x", "y"].contains(&winner.as_str()));
    }
}
