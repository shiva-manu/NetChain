// src/poi/anti_gaming.rs
//
// Anti-gaming protections for Proof of Internet consensus.
// Prevents manipulation of metrics through outlier detection,
// rate limiting, and verification requirements.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// Configuration for anti-gaming protections
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AntiGamingConfig {
    /// Maximum standard deviations from median before flagging as outlier
    pub outlier_threshold_sigma: f64,

    /// Minimum attestations required before metrics are trusted
    pub min_trusted_attestations: usize,

    /// Maximum challenges a node can issue per hour
    pub max_challenges_per_hour: usize,

    /// Maximum challenges a node can receive per hour
    pub max_received_challenges_per_hour: usize,

    /// Hard bounds for metrics (anything outside is rejected)
    pub bounds: MetricBounds,

    /// Number of historical samples to keep for statistics
    pub history_size: usize,

    /// Penalty factor for nodes with suspicious patterns (0.0-1.0)
    pub suspicious_penalty: f64,
}

impl Default for AntiGamingConfig {
    fn default() -> Self {
        Self {
            outlier_threshold_sigma: 3.0,
            min_trusted_attestations: 3,
            max_challenges_per_hour: 60,
            max_received_challenges_per_hour: 120,
            bounds: MetricBounds::default(),
            history_size: 100,
            suspicious_penalty: 0.5,
        }
    }
}

/// Hard bounds for valid metrics
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetricBounds {
    /// Maximum download speed (Mbps)
    pub max_download_mbps: f64,
    /// Maximum upload speed (Mbps)
    pub max_upload_mbps: f64,
    /// Minimum latency (ms) - anything lower is suspicious
    pub min_latency_ms: f64,
    /// Maximum latency (ms) - anything higher is effectively offline
    pub max_latency_ms: f64,
}

impl Default for MetricBounds {
    fn default() -> Self {
        Self {
            max_download_mbps: 10_000.0, // 10 Gbps
            max_upload_mbps: 10_000.0,   // 10 Gbps
            min_latency_ms: 0.1,         // 0.1ms minimum (sub-ms is suspicious)
            max_latency_ms: 10_000.0,    // 10 seconds
        }
    }
}

/// Result of validation check
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    Valid,
    OutOfBounds(String),
    Outlier(String),
    RateLimited(String),
    InsufficientAttestations,
    Suspicious(String),
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        matches!(self, ValidationResult::Valid)
    }
}

/// Challenge rate tracking for a node
#[derive(Debug, Default)]
struct ChallengeTracker {
    /// Timestamps of challenges issued
    issued: VecDeque<u64>,
    /// Timestamps of challenges received
    received: VecDeque<u64>,
}

impl ChallengeTracker {
    fn prune_old(&mut self, current_time: u64, window_secs: u64) {
        let cutoff = current_time.saturating_sub(window_secs);
        while self.issued.front().map_or(false, |&t| t < cutoff) {
            self.issued.pop_front();
        }
        while self.received.front().map_or(false, |&t| t < cutoff) {
            self.received.pop_front();
        }
    }
}

/// Network-wide statistics for outlier detection
#[derive(Debug, Default, Clone)]
pub struct NetworkStats {
    pub download_median: f64,
    pub download_stddev: f64,
    pub upload_median: f64,
    pub upload_stddev: f64,
    pub latency_median: f64,
    pub latency_stddev: f64,
    pub sample_count: usize,
}

/// Anti-gaming protection service
pub struct AntiGamingService {
    config: AntiGamingConfig,
    /// Challenge rate tracking per node
    challenge_trackers: HashMap<String, ChallengeTracker>,
    /// Historical metrics for computing network statistics
    download_history: VecDeque<f64>,
    upload_history: VecDeque<f64>,
    latency_history: VecDeque<f64>,
    /// Cached network stats
    cached_stats: NetworkStats,
    /// Flagged suspicious nodes
    suspicious_nodes: HashMap<String, String>, // node_id -> reason
}

impl AntiGamingService {
    pub fn new(config: AntiGamingConfig) -> Self {
        Self {
            config,
            challenge_trackers: HashMap::new(),
            download_history: VecDeque::new(),
            upload_history: VecDeque::new(),
            latency_history: VecDeque::new(),
            cached_stats: NetworkStats::default(),
            suspicious_nodes: HashMap::new(),
        }
    }

    /// Validate metrics against bounds
    pub fn validate_bounds(
        &self,
        download_mbps: f64,
        upload_mbps: f64,
        latency_ms: f64,
    ) -> ValidationResult {
        let bounds = &self.config.bounds;

        if download_mbps < 0.0 || download_mbps > bounds.max_download_mbps {
            return ValidationResult::OutOfBounds(format!(
                "Download speed {} Mbps out of bounds (0-{})",
                download_mbps, bounds.max_download_mbps
            ));
        }

        if upload_mbps < 0.0 || upload_mbps > bounds.max_upload_mbps {
            return ValidationResult::OutOfBounds(format!(
                "Upload speed {} Mbps out of bounds (0-{})",
                upload_mbps, bounds.max_upload_mbps
            ));
        }

        if latency_ms < bounds.min_latency_ms {
            return ValidationResult::OutOfBounds(format!(
                "Latency {} ms suspiciously low (min {})",
                latency_ms, bounds.min_latency_ms
            ));
        }

        if latency_ms > bounds.max_latency_ms {
            return ValidationResult::OutOfBounds(format!(
                "Latency {} ms too high (max {})",
                latency_ms, bounds.max_latency_ms
            ));
        }

        ValidationResult::Valid
    }

    /// Check if metrics are statistical outliers
    pub fn check_outlier(
        &self,
        download_mbps: f64,
        upload_mbps: f64,
        latency_ms: f64,
    ) -> ValidationResult {
        let stats = &self.cached_stats;

        // Need enough samples for meaningful statistics
        if stats.sample_count < 10 {
            return ValidationResult::Valid;
        }

        let sigma = self.config.outlier_threshold_sigma;

        // Check download
        if stats.download_stddev > 0.0 {
            let z_score = (download_mbps - stats.download_median).abs() / stats.download_stddev;
            if z_score > sigma {
                return ValidationResult::Outlier(format!(
                    "Download speed {} Mbps is {:.1} std devs from median {:.1}",
                    download_mbps, z_score, stats.download_median
                ));
            }
        }

        // Check upload
        if stats.upload_stddev > 0.0 {
            let z_score = (upload_mbps - stats.upload_median).abs() / stats.upload_stddev;
            if z_score > sigma {
                return ValidationResult::Outlier(format!(
                    "Upload speed {} Mbps is {:.1} std devs from median {:.1}",
                    upload_mbps, z_score, stats.upload_median
                ));
            }
        }

        // Check latency
        if stats.latency_stddev > 0.0 {
            let z_score = (latency_ms - stats.latency_median).abs() / stats.latency_stddev;
            if z_score > sigma {
                return ValidationResult::Outlier(format!(
                    "Latency {} ms is {:.1} std devs from median {:.1}",
                    latency_ms, z_score, stats.latency_median
                ));
            }
        }

        ValidationResult::Valid
    }

    /// Check if a node can issue a challenge (rate limiting)
    pub fn can_issue_challenge(&mut self, node_id: &str) -> ValidationResult {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let tracker = self
            .challenge_trackers
            .entry(node_id.to_string())
            .or_default();

        // Prune old entries
        tracker.prune_old(now, 3600);

        if tracker.issued.len() >= self.config.max_challenges_per_hour {
            return ValidationResult::RateLimited(format!(
                "Node {} has exceeded challenge rate limit ({}/hour)",
                node_id, self.config.max_challenges_per_hour
            ));
        }

        ValidationResult::Valid
    }

    /// Record that a node issued a challenge
    pub fn record_challenge_issued(&mut self, node_id: &str) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let tracker = self
            .challenge_trackers
            .entry(node_id.to_string())
            .or_default();
        tracker.issued.push_back(now);
    }

    /// Check if a node can receive a challenge (rate limiting)
    pub fn can_receive_challenge(&mut self, node_id: &str) -> ValidationResult {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let tracker = self
            .challenge_trackers
            .entry(node_id.to_string())
            .or_default();

        tracker.prune_old(now, 3600);

        if tracker.received.len() >= self.config.max_received_challenges_per_hour {
            return ValidationResult::RateLimited(format!(
                "Node {} has exceeded received challenge limit ({}/hour)",
                node_id, self.config.max_received_challenges_per_hour
            ));
        }

        ValidationResult::Valid
    }

    /// Record that a node received a challenge
    pub fn record_challenge_received(&mut self, node_id: &str) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let tracker = self
            .challenge_trackers
            .entry(node_id.to_string())
            .or_default();
        tracker.received.push_back(now);
    }

    /// Add metrics sample to history for statistics
    pub fn add_metric_sample(&mut self, download_mbps: f64, upload_mbps: f64, latency_ms: f64) {
        let max_size = self.config.history_size;

        if self.download_history.len() >= max_size {
            self.download_history.pop_front();
        }
        self.download_history.push_back(download_mbps);

        if self.upload_history.len() >= max_size {
            self.upload_history.pop_front();
        }
        self.upload_history.push_back(upload_mbps);

        if self.latency_history.len() >= max_size {
            self.latency_history.pop_front();
        }
        self.latency_history.push_back(latency_ms);

        // Recompute statistics
        self.recompute_stats();
    }

    /// Recompute network statistics
    fn recompute_stats(&mut self) {
        self.cached_stats = NetworkStats {
            download_median: median(&self.download_history),
            download_stddev: stddev(&self.download_history),
            upload_median: median(&self.upload_history),
            upload_stddev: stddev(&self.upload_history),
            latency_median: median(&self.latency_history),
            latency_stddev: stddev(&self.latency_history),
            sample_count: self.download_history.len(),
        };
    }

    /// Get current network statistics
    pub fn get_network_stats(&self) -> &NetworkStats {
        &self.cached_stats
    }

    /// Flag a node as suspicious
    pub fn flag_suspicious(&mut self, node_id: &str, reason: &str) {
        self.suspicious_nodes
            .insert(node_id.to_string(), reason.to_string());
    }

    /// Check if a node is flagged as suspicious
    pub fn is_suspicious(&self, node_id: &str) -> Option<&String> {
        self.suspicious_nodes.get(node_id)
    }

    /// Clear suspicious flag for a node
    pub fn clear_suspicious(&mut self, node_id: &str) {
        self.suspicious_nodes.remove(node_id);
    }

    /// Get penalty factor for a node (1.0 = no penalty, lower = penalized)
    pub fn get_penalty_factor(&self, node_id: &str) -> f64 {
        if self.suspicious_nodes.contains_key(node_id) {
            self.config.suspicious_penalty
        } else {
            1.0
        }
    }

    /// Full validation of metrics
    pub fn validate_metrics(
        &mut self,
        node_id: &str,
        download_mbps: f64,
        upload_mbps: f64,
        latency_ms: f64,
        attestation_count: usize,
    ) -> ValidationResult {
        // Check bounds first
        let bounds_result = self.validate_bounds(download_mbps, upload_mbps, latency_ms);
        if !bounds_result.is_valid() {
            self.flag_suspicious(node_id, "Out of bounds metrics");
            return bounds_result;
        }

        // Check attestations
        if attestation_count < self.config.min_trusted_attestations {
            return ValidationResult::InsufficientAttestations;
        }

        // Check for outliers
        let outlier_result = self.check_outlier(download_mbps, upload_mbps, latency_ms);
        if !outlier_result.is_valid() {
            // Don't immediately flag - could be legitimate exceptional performance
            return outlier_result;
        }

        // Check if previously flagged
        if let Some(reason) = self.suspicious_nodes.get(node_id) {
            return ValidationResult::Suspicious(reason.clone());
        }

        ValidationResult::Valid
    }
}

/// Compute median of a collection
fn median(values: &VecDeque<f64>) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let mut sorted: Vec<f64> = values.iter().copied().collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let len = sorted.len();
    if len % 2 == 0 {
        (sorted[len / 2 - 1] + sorted[len / 2]) / 2.0
    } else {
        sorted[len / 2]
    }
}

/// Compute standard deviation of a collection
fn stddev(values: &VecDeque<f64>) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }

    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance =
        values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64;

    variance.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_bounds() {
        let config = AntiGamingConfig::default();
        let service = AntiGamingService::new(config);

        // Valid metrics
        assert!(service.validate_bounds(100.0, 50.0, 20.0).is_valid());

        // Out of bounds download
        assert!(!service.validate_bounds(100_000.0, 50.0, 20.0).is_valid());

        // Negative speed
        assert!(!service.validate_bounds(-1.0, 50.0, 20.0).is_valid());

        // Too low latency (suspicious)
        assert!(!service.validate_bounds(100.0, 50.0, 0.01).is_valid());

        // Too high latency
        assert!(!service.validate_bounds(100.0, 50.0, 50_000.0).is_valid());
    }

    #[test]
    fn test_outlier_detection() {
        let config = AntiGamingConfig::default();
        let mut service = AntiGamingService::new(config);

        // Add varied samples to get meaningful statistics
        for i in 0..20 {
            // Vary samples slightly around a mean
            let download = 100.0 + (i as f64 - 10.0) * 2.0; // 80-120 range
            let upload = 50.0 + (i as f64 - 10.0); // 40-60 range
            let latency = 20.0 + (i as f64 - 10.0) * 0.5; // 15-25 range
            service.add_metric_sample(download, upload, latency);
        }

        // Normal value should pass
        assert!(service.check_outlier(100.0, 50.0, 20.0).is_valid());

        // Extreme outlier (way beyond 3 sigma) should fail
        // With stddev ~12 for download, 10000 is ~825 sigma away
        assert!(!service.check_outlier(10000.0, 50.0, 20.0).is_valid());
    }

    #[test]
    fn test_rate_limiting() {
        let mut config = AntiGamingConfig::default();
        config.max_challenges_per_hour = 5;
        let mut service = AntiGamingService::new(config);

        // First 5 should succeed
        for i in 0..5 {
            assert!(
                service.can_issue_challenge("node1").is_valid(),
                "Challenge {} should succeed",
                i
            );
            service.record_challenge_issued("node1");
        }

        // 6th should fail
        assert!(!service.can_issue_challenge("node1").is_valid());
    }

    #[test]
    fn test_suspicious_flagging() {
        let config = AntiGamingConfig::default();
        let mut service = AntiGamingService::new(config);

        assert!(service.is_suspicious("node1").is_none());

        service.flag_suspicious("node1", "Testing");
        assert!(service.is_suspicious("node1").is_some());
        assert!(service.get_penalty_factor("node1") < 1.0);

        service.clear_suspicious("node1");
        assert!(service.is_suspicious("node1").is_none());
        assert_eq!(service.get_penalty_factor("node1"), 1.0);
    }

    #[test]
    fn test_median_and_stddev() {
        let mut values: VecDeque<f64> = VecDeque::new();
        values.extend([1.0, 2.0, 3.0, 4.0, 5.0]);

        assert!((median(&values) - 3.0).abs() < 0.01);

        // Standard deviation of 1,2,3,4,5 is ~1.58
        let std = stddev(&values);
        assert!(std > 1.5 && std < 1.7);
    }

    #[test]
    fn test_full_validation() {
        let mut config = AntiGamingConfig::default();
        config.min_trusted_attestations = 3;
        let mut service = AntiGamingService::new(config);

        // Add some baseline samples
        for _ in 0..20 {
            service.add_metric_sample(100.0, 50.0, 20.0);
        }

        // Should fail without enough attestations
        let result = service.validate_metrics("node1", 100.0, 50.0, 20.0, 1);
        assert_eq!(result, ValidationResult::InsufficientAttestations);

        // Should pass with enough attestations
        let result = service.validate_metrics("node1", 100.0, 50.0, 20.0, 5);
        assert!(result.is_valid());
    }
}
