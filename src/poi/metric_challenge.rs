// src/poi/metric_challenge.rs
//
// Peer-to-peer metric challenge/response system for Proof of Internet consensus.
// Implements real bandwidth tests between peers to verify claimed metrics.

use crate::consensus::NodeMetrics;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet, VecDeque};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio::time::timeout;
use tracing::{debug, info, warn};

/// Challenge state for tracking in-flight challenges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeState {
    pub challenge_nonce: String,
    pub target_id: String,
    pub challenger_id: String,
    pub bytes_to_download: usize,
    pub issued_at: u64,
    pub response: Option<ChallengeResponse>,
    pub completed: bool,
}

/// Response to a metric challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResponse {
    pub challenge_nonce: String,
    pub responder_id: String,
    pub download_mbps: f64,
    pub upload_mbps: f64,
    pub latency_ms: f64,
    pub bytes_transferred: usize,
    pub duration_ms: u64,
    pub timestamp: u64,
}

/// Verified challenge result after validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedChallenge {
    pub challenge_nonce: String,
    pub challenger_id: String,
    pub target_id: String,
    pub download_mbps: f64,
    pub upload_mbps: f64,
    pub latency_ms: f64,
    pub confidence: f64,
    pub verified_at: u64,
}

/// Configuration for P2P metric challenges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricChallengeConfig {
    /// Default bytes for download challenges
    pub default_challenge_bytes: usize,
    /// Timeout for challenge completion (seconds)
    pub challenge_timeout_secs: u64,
    /// Minimum interval between challenges to same peer (seconds)
    pub min_challenge_interval_secs: u64,
    /// Maximum concurrent challenges per peer
    pub max_concurrent_challenges: usize,
    /// Challenge success required for attestation
    pub min_challenges_for_attestation: usize,
    /// Enable automatic challenge requests to unverified peers
    pub auto_challenge_enabled: bool,
}

impl Default for MetricChallengeConfig {
    fn default() -> Self {
        Self {
            default_challenge_bytes: 5_000_000, // 5 MB
            challenge_timeout_secs: 60,
            min_challenge_interval_secs: 300, // 5 minutes
            max_concurrent_challenges: 3,
            min_challenges_for_attestation: 2,
            auto_challenge_enabled: true,
        }
    }
}

/// P2P Metric Challenge Service
pub struct MetricChallengeService {
    config: MetricChallengeConfig,
    /// In-flight challenges initiated by us
    outgoing_challenges: Arc<RwLock<HashMap<String, ChallengeState>>>,
    /// Challenges we've received from others
    #[allow(dead_code)]
    incoming_challenges: Arc<RwLock<HashMap<String, ChallengeState>>>,
    /// Completed and verified challenges
    verified_challenges: Arc<RwLock<VecDeque<VerifiedChallenge>>>,
    /// Track challenge timestamps per peer for rate limiting
    last_challenge_to_peer: Arc<RwLock<HashMap<String, u64>>>,
    /// Our signing key for challenge signatures
    signing_key: Arc<SigningKey>,
    /// Our node ID
    node_id: String,
    /// Known peer addresses for direct TCP tests
    peer_addresses: Arc<RwLock<HashMap<String, SocketAddr>>>,
}

impl MetricChallengeService {
    pub fn new(config: MetricChallengeConfig, signing_key: SigningKey, node_id: String) -> Self {
        Self {
            config,
            outgoing_challenges: Arc::new(RwLock::new(HashMap::new())),
            incoming_challenges: Arc::new(RwLock::new(HashMap::new())),
            verified_challenges: Arc::new(RwLock::new(VecDeque::with_capacity(100))),
            last_challenge_to_peer: Arc::new(RwLock::new(HashMap::new())),
            signing_key: Arc::new(signing_key),
            node_id,
            peer_addresses: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a peer's address for direct TCP connections
    pub async fn register_peer_address(&self, peer_id: String, addr: SocketAddr) {
        let mut peers = self.peer_addresses.write().await;
        peers.insert(peer_id.clone(), addr);
        debug!("Registered peer {} at {}", peer_id, addr);
    }

    /// Get a peer's address
    pub async fn get_peer_address(&self, peer_id: &str) -> Option<SocketAddr> {
        let peers = self.peer_addresses.read().await;
        peers.get(peer_id).copied()
    }

    /// Generate a random nonce for challenge identification
    pub fn generate_nonce() -> String {
        let mut bytes = [0u8; 32];
        RngCore::fill_bytes(&mut OsRng, &mut bytes);
        hex::encode(bytes)
    }

    /// Check if we can challenge a peer (rate limiting)
    pub async fn can_challenge_peer(&self, peer_id: &str) -> bool {
        let last_times = self.last_challenge_to_peer.read().await;
        if let Some(&last_time) = last_times.get(peer_id) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            if now.saturating_sub(last_time) < self.config.min_challenge_interval_secs {
                return false;
            }
        }

        // Check concurrent challenge limit
        let outgoing = self.outgoing_challenges.read().await;
        let active_count = outgoing
            .values()
            .filter(|c| c.target_id == peer_id && !c.completed)
            .count();

        active_count < self.config.max_concurrent_challenges
    }

    /// Create a new challenge to send to a peer
    pub async fn create_challenge(&self, target_id: String) -> Option<ChallengeState> {
        if !self.can_challenge_peer(&target_id).await {
            return None;
        }

        let nonce = Self::generate_nonce();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let challenge = ChallengeState {
            challenge_nonce: nonce.clone(),
            target_id: target_id.clone(),
            challenger_id: self.node_id.clone(),
            bytes_to_download: self.config.default_challenge_bytes,
            issued_at: now,
            response: None,
            completed: false,
        };

        // Record challenge time for rate limiting
        {
            let mut last_times = self.last_challenge_to_peer.write().await;
            last_times.insert(target_id, now);
        }

        // Store outgoing challenge
        {
            let mut outgoing = self.outgoing_challenges.write().await;
            outgoing.insert(nonce.clone(), challenge.clone());
        }

        Some(challenge)
    }

    /// Execute a P2P bandwidth challenge (download test)
    pub async fn execute_download_challenge(
        &self,
        peer_addr: SocketAddr,
        bytes_to_download: usize,
    ) -> Result<(f64, usize, u64), String> {
        let timeout_duration = Duration::from_secs(self.config.challenge_timeout_secs);
        let start = Instant::now();

        // Connect to peer
        let stream = timeout(timeout_duration, TcpStream::connect(peer_addr))
            .await
            .map_err(|_| "Connection timeout")?
            .map_err(|e| format!("Connection failed: {}", e))?;

        // Send challenge request header
        let request = format!("CHALLENGE:{}\n", bytes_to_download);
        let mut stream = stream;
        timeout(timeout_duration, stream.write_all(request.as_bytes()))
            .await
            .map_err(|_| "Write timeout")?
            .map_err(|e| format!("Write failed: {}", e))?;

        // Read response data
        let mut buffer = vec![0u8; bytes_to_download];
        let mut total_read = 0;

        while total_read < bytes_to_download {
            let read = timeout(
                timeout_duration,
                stream.read(&mut buffer[total_read..]),
            )
            .await
            .map_err(|_| "Read timeout")?
            .map_err(|e| format!("Read failed: {}", e))?;

            if read == 0 {
                break; // EOF
            }
            total_read += read;
        }

        let duration = start.elapsed();
        let duration_ms = duration.as_millis() as u64;

        // Calculate Mbps
        let mbps = if duration_ms > 0 {
            (total_read as f64 * 8.0) / (duration_ms as f64 / 1000.0) / 1_000_000.0
        } else {
            0.0
        };

        Ok((mbps, total_read, duration_ms))
    }

    /// Execute upload challenge (send data to peer)
    pub async fn execute_upload_challenge(
        &self,
        peer_addr: SocketAddr,
        bytes_to_upload: usize,
    ) -> Result<(f64, usize, u64), String> {
        let timeout_duration = Duration::from_secs(self.config.challenge_timeout_secs);
        let start = Instant::now();

        // Connect to peer
        let stream = timeout(timeout_duration, TcpStream::connect(peer_addr))
            .await
            .map_err(|_| "Connection timeout")?
            .map_err(|e| format!("Connection failed: {}", e))?;

        // Send upload request header
        let request = format!("UPLOAD:{}\n", bytes_to_upload);
        let mut stream = stream;
        timeout(timeout_duration, stream.write_all(request.as_bytes()))
            .await
            .map_err(|_| "Write timeout")?
            .map_err(|e| format!("Write failed: {}", e))?;

        // Generate and send test data
        let data: Vec<u8> = (0..bytes_to_upload).map(|i| (i % 256) as u8).collect();
        timeout(timeout_duration, stream.write_all(&data))
            .await
            .map_err(|_| "Upload timeout")?
            .map_err(|e| format!("Upload failed: {}", e))?;

        // Wait for acknowledgment
        let mut ack = [0u8; 2];
        timeout(timeout_duration, stream.read_exact(&mut ack))
            .await
            .map_err(|_| "Ack timeout")?
            .map_err(|e| format!("Ack failed: {}", e))?;

        let duration = start.elapsed();
        let duration_ms = duration.as_millis() as u64;

        // Calculate Mbps
        let mbps = if duration_ms > 0 {
            (bytes_to_upload as f64 * 8.0) / (duration_ms as f64 / 1000.0) / 1_000_000.0
        } else {
            0.0
        };

        Ok((mbps, bytes_to_upload, duration_ms))
    }

    /// Measure TCP latency (RTT) to a peer
    pub async fn measure_latency(&self, peer_addr: SocketAddr) -> Result<f64, String> {
        let timeout_duration = Duration::from_secs(5);
        let start = Instant::now();

        // Connect and immediately close for RTT measurement
        let mut stream = timeout(timeout_duration, TcpStream::connect(peer_addr))
            .await
            .map_err(|_| "Connection timeout")?
            .map_err(|e| format!("Connection failed: {}", e))?;

        // Send ping
        let ping = b"PING\n";
        timeout(timeout_duration, stream.write_all(ping))
            .await
            .map_err(|_| "Ping write timeout")?
            .map_err(|e| format!("Ping write failed: {}", e))?;

        // Read pong
        let mut pong = [0u8; 5];
        timeout(timeout_duration, stream.read_exact(&mut pong))
            .await
            .map_err(|_| "Pong read timeout")?
            .map_err(|e| format!("Pong read failed: {}", e))?;

        let rtt = start.elapsed();
        Ok(rtt.as_secs_f64() * 1000.0) // Convert to ms
    }

    /// Handle incoming challenge request from a peer (server side)
    pub async fn handle_incoming_challenge(
        &self,
        mut stream: TcpStream,
        bytes_to_send: usize,
    ) -> Result<(usize, u64), String> {
        let start = Instant::now();

        // Generate test data
        let data: Vec<u8> = (0..bytes_to_send).map(|i| (i % 256) as u8).collect();

        // Send data
        timeout(
            Duration::from_secs(self.config.challenge_timeout_secs),
            stream.write_all(&data),
        )
        .await
        .map_err(|_| "Challenge send timeout")?
        .map_err(|e| format!("Challenge send failed: {}", e))?;

        let duration = start.elapsed();
        Ok((bytes_to_send, duration.as_millis() as u64))
    }

    /// Handle incoming upload request (server side)
    pub async fn handle_incoming_upload(
        &self,
        mut stream: TcpStream,
        expected_bytes: usize,
    ) -> Result<(usize, u64), String> {
        let start = Instant::now();

        // Read uploaded data
        let mut buffer = vec![0u8; expected_bytes];
        let mut total_read = 0;

        while total_read < expected_bytes {
            let read = timeout(
                Duration::from_secs(self.config.challenge_timeout_secs),
                stream.read(&mut buffer[total_read..]),
            )
            .await
            .map_err(|_| "Upload receive timeout")?
            .map_err(|e| format!("Upload receive failed: {}", e))?;

            if read == 0 {
                break;
            }
            total_read += read;
        }

        // Send acknowledgment
        let _ = stream.write_all(b"OK\n").await;

        let duration = start.elapsed();
        Ok((total_read, duration.as_millis() as u64))
    }

    /// Process a challenge response and create verified result
    pub async fn process_challenge_response(
        &self,
        response: ChallengeResponse,
    ) -> Option<VerifiedChallenge> {
        let mut outgoing = self.outgoing_challenges.write().await;

        // Find matching challenge
        let challenge = outgoing.get_mut(&response.challenge_nonce)?;

        // Validate response matches challenge
        if challenge.target_id != response.responder_id {
            warn!(
                "Challenge response mismatch: expected {}, got {}",
                challenge.target_id, response.responder_id
            );
            return None;
        }

        // Mark challenge as completed
        challenge.response = Some(response.clone());
        challenge.completed = true;

        // Create verified challenge with confidence based on data transferred
        let confidence = if response.bytes_transferred >= self.config.default_challenge_bytes {
            1.0
        } else {
            response.bytes_transferred as f64 / self.config.default_challenge_bytes as f64
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let verified = VerifiedChallenge {
            challenge_nonce: response.challenge_nonce.clone(),
            challenger_id: challenge.challenger_id.clone(),
            target_id: challenge.target_id.clone(),
            download_mbps: response.download_mbps,
            upload_mbps: response.upload_mbps,
            latency_ms: response.latency_ms,
            confidence,
            verified_at: now,
        };

        // Store verified challenge
        let mut verified_list = self.verified_challenges.write().await;
        verified_list.push_back(verified.clone());

        // Keep only recent challenges
        while verified_list.len() > 100 {
            verified_list.pop_front();
        }

        info!(
            "Verified challenge for {}: download={} Mbps, upload={} Mbps, latency={} ms",
            verified.target_id, verified.download_mbps, verified.upload_mbps, verified.latency_ms
        );

        Some(verified)
    }

    /// Get recent verified challenges for a target
    pub async fn get_verified_challenges(&self, target_id: &str) -> Vec<VerifiedChallenge> {
        let verified = self.verified_challenges.read().await;
        verified
            .iter()
            .filter(|v| v.target_id == target_id)
            .cloned()
            .collect()
    }

    /// Compute aggregated metrics from verified challenges
    pub async fn compute_verified_metrics(&self, target_id: &str) -> Option<NodeMetrics> {
        let challenges = self.get_verified_challenges(target_id).await;

        if challenges.is_empty() {
            return None;
        }

        let avg_download = challenges.iter().map(|c| c.download_mbps).sum::<f64>() / challenges.len() as f64;
        let avg_upload = challenges.iter().map(|c| c.upload_mbps).sum::<f64>() / challenges.len() as f64;
        let avg_latency = challenges.iter().map(|c| c.latency_ms).sum::<f64>() / challenges.len() as f64;
        let avg_confidence = challenges.iter().map(|c| c.confidence).sum::<f64>() / challenges.len() as f64;

        Some(NodeMetrics {
            node_id: target_id.to_string(),
            download_mbps: avg_download,
            upload_mbps: avg_upload,
            latency_ms: avg_latency,
            uptime_percent: 99.0, // Would need separate tracking
            stability_percent: avg_confidence * 100.0,
            identity_score: 0.5,
            reputation_score: 0.5,
            attestation_count: challenges.len(),
            unique_attester_count: 1, // This challenger
            slashing_penalty: 0.0,
        })
    }

    /// Sign challenge data for attestation
    pub fn sign_challenge_data(&self, data: &[u8]) -> Signature {
        self.signing_key.sign(data)
    }

    /// Verify challenge signature
    pub fn verify_challenge_signature(
        &self,
        data: &[u8],
        signature: &Signature,
        public_key: &VerifyingKey,
    ) -> Result<(), String> {
        public_key
            .verify(data, signature)
            .map_err(|e| format!("Signature verification failed: {}", e))
    }

    /// Get count of active (incomplete) outgoing challenges
    pub async fn active_challenge_count(&self) -> usize {
        let outgoing = self.outgoing_challenges.read().await;
        outgoing.values().filter(|c| !c.completed).count()
    }

    /// Clean up expired challenges
    pub async fn cleanup_expired_challenges(&self) -> usize {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut removed = 0;

        // Clean outgoing challenges
        {
            let mut outgoing = self.outgoing_challenges.write().await;
            let to_remove: Vec<String> = outgoing
                .iter()
                .filter(|(_, c)| {
                    !c.completed && now.saturating_sub(c.issued_at) > self.config.challenge_timeout_secs
                })
                .map(|(k, _)| k.clone())
                .collect();

            for key in to_remove {
                outgoing.remove(&key);
                removed += 1;
            }
        }

        // Clean old verified challenges (older than 1 hour)
        {
            let mut verified = self.verified_challenges.write().await;
            let cutoff = now.saturating_sub(3600);
            while let Some(front) = verified.front() {
                if front.verified_at < cutoff {
                    verified.pop_front();
                    removed += 1;
                } else {
                    break;
                }
            }
        }

        if removed > 0 {
            debug!("Cleaned up {} expired challenges", removed);
        }

        removed
    }
}

/// Helper for computing hash of challenge data
pub fn compute_challenge_hash(nonce: &str, target_id: &str, bytes: usize) -> String {
    let mut hasher = Sha256::new();
    hasher.update(nonce.as_bytes());
    hasher.update(target_id.as_bytes());
    hasher.update(bytes.to_be_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nonce_generation() {
        let nonce1 = MetricChallengeService::generate_nonce();
        let nonce2 = MetricChallengeService::generate_nonce();
        assert_ne!(nonce1, nonce2);
        assert_eq!(nonce1.len(), 64); // 32 bytes = 64 hex chars
    }

    #[test]
    fn test_challenge_hash_determinism() {
        let hash1 = compute_challenge_hash("nonce1", "target1", 1000);
        let hash2 = compute_challenge_hash("nonce1", "target1", 1000);
        let hash3 = compute_challenge_hash("nonce2", "target1", 1000);

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[tokio::test]
    async fn test_challenge_service_creation() {
        let config = MetricChallengeConfig::default();
        let signing_key = SigningKey::generate(&mut OsRng);
        let service = MetricChallengeService::new(config, signing_key, "test_node".to_string());

        assert_eq!(service.active_challenge_count().await, 0);

        // Create a challenge
        let challenge = service
            .create_challenge("peer1".to_string())
            .await
            .expect("Should create challenge");

        assert_eq!(challenge.target_id, "peer1");
        assert_eq!(service.active_challenge_count().await, 1);
    }

    #[tokio::test]
    async fn test_challenge_rate_limiting() {
        let mut config = MetricChallengeConfig::default();
        config.min_challenge_interval_secs = 60;
        let signing_key = SigningKey::generate(&mut OsRng);
        let service = MetricChallengeService::new(config, signing_key, "test_node".to_string());

        // First challenge should succeed
        assert!(service.can_challenge_peer("peer1").await);
        let _ = service.create_challenge("peer1".to_string()).await;

        // Second challenge should be rate limited
        assert!(!service.can_challenge_peer("peer1").await);
    }

    #[tokio::test]
    async fn test_challenge_cleanup() {
        let mut config = MetricChallengeConfig::default();
        config.challenge_timeout_secs = 1; // Very short for testing
        let signing_key = SigningKey::generate(&mut OsRng);
        let service = MetricChallengeService::new(config, signing_key, "test_node".to_string());

        // Create a challenge
        let _ = service
            .create_challenge("peer1".to_string())
            .await;

        assert_eq!(service.active_challenge_count().await, 1);

        // Wait for timeout
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Cleanup should remove expired challenge
        let removed = service.cleanup_expired_challenges().await;
        assert_eq!(removed, 1);
        assert_eq!(service.active_challenge_count().await, 0);
    }
}
