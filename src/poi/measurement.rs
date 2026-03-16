// src/poi/measurement.rs
//
// Real internet speed measurement for Proof of Internet (PoI) consensus.
// Measures download speed, upload speed, latency, and tracks uptime/stability.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio::time::timeout;

/// Configuration for measurement endpoints and parameters
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MeasurementConfig {
    /// URLs for download speed tests (HTTP GET, should return large payloads)
    pub download_endpoints: Vec<String>,
    /// URLs for upload speed tests (HTTP POST, should accept large payloads)
    pub upload_endpoints: Vec<String>,
    /// Timeout for each measurement attempt
    pub timeout_secs: u64,
    /// Number of bytes to download for speed test
    pub download_bytes: usize,
    /// Number of bytes to upload for speed test
    pub upload_bytes: usize,
    /// How many historical measurements to keep for averaging
    pub history_size: usize,
    /// Minimum interval between measurements (seconds)
    pub min_interval_secs: u64,
}

impl Default for MeasurementConfig {
    fn default() -> Self {
        Self {
            // Default to well-known speed test endpoints
            // In production, these should be configurable
            download_endpoints: vec![
                "https://speed.cloudflare.com/__down?bytes=10000000".to_string()
            ],
            upload_endpoints: vec!["https://speed.cloudflare.com/__up".to_string()],
            timeout_secs: 30,
            download_bytes: 10_000_000, // 10 MB
            upload_bytes: 5_000_000,    // 5 MB
            history_size: 10,
            min_interval_secs: 60,
        }
    }
}

/// Result of a single speed measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedResult {
    pub mbps: f64,
    pub bytes_transferred: usize,
    pub duration_ms: u64,
    pub timestamp: u64,
    pub success: bool,
    pub error: Option<String>,
}

/// Result of a latency measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyResult {
    pub target: String,
    pub rtt_ms: f64,
    pub success: bool,
    pub error: Option<String>,
    pub timestamp: u64,
}

/// Aggregated metrics from multiple measurements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    pub upload_mbps: f64,
    pub download_mbps: f64,
    pub latency_ms: f64,
    pub uptime_percent: f64,
    pub stability_percent: f64,
    pub sample_count: usize,
    pub last_updated: u64,
}

impl Default for AggregatedMetrics {
    fn default() -> Self {
        Self {
            upload_mbps: 0.0,
            download_mbps: 0.0,
            latency_ms: 0.0,
            uptime_percent: 100.0,
            stability_percent: 100.0,
            sample_count: 0,
            last_updated: 0,
        }
    }
}

/// Historical measurement data for calculating stability and averages
#[derive(Debug)]
struct MeasurementHistory {
    download_results: VecDeque<SpeedResult>,
    upload_results: VecDeque<SpeedResult>,
    latency_results: VecDeque<LatencyResult>,
    // Track uptime: (timestamp, was_online)
    uptime_checks: VecDeque<(u64, bool)>,
    max_size: usize,
}

impl MeasurementHistory {
    fn new(max_size: usize) -> Self {
        Self {
            download_results: VecDeque::with_capacity(max_size),
            upload_results: VecDeque::with_capacity(max_size),
            latency_results: VecDeque::with_capacity(max_size),
            uptime_checks: VecDeque::with_capacity(max_size * 10), // More uptime samples
            max_size,
        }
    }

    fn add_download(&mut self, result: SpeedResult) {
        if self.download_results.len() >= self.max_size {
            self.download_results.pop_front();
        }
        self.download_results.push_back(result);
    }

    fn add_upload(&mut self, result: SpeedResult) {
        if self.upload_results.len() >= self.max_size {
            self.upload_results.pop_front();
        }
        self.upload_results.push_back(result);
    }

    fn add_latency(&mut self, result: LatencyResult) {
        if self.latency_results.len() >= self.max_size {
            self.latency_results.pop_front();
        }
        self.latency_results.push_back(result);
    }

    fn add_uptime_check(&mut self, timestamp: u64, online: bool) {
        let max_uptime = self.max_size * 10;
        if self.uptime_checks.len() >= max_uptime {
            self.uptime_checks.pop_front();
        }
        self.uptime_checks.push_back((timestamp, online));
    }

    /// Calculate average download speed (Mbps) from successful measurements
    fn avg_download_mbps(&self) -> f64 {
        let successful: Vec<f64> = self
            .download_results
            .iter()
            .filter(|r| r.success)
            .map(|r| r.mbps)
            .collect();
        if successful.is_empty() {
            0.0
        } else {
            successful.iter().sum::<f64>() / successful.len() as f64
        }
    }

    /// Calculate average upload speed (Mbps) from successful measurements
    fn avg_upload_mbps(&self) -> f64 {
        let successful: Vec<f64> = self
            .upload_results
            .iter()
            .filter(|r| r.success)
            .map(|r| r.mbps)
            .collect();
        if successful.is_empty() {
            0.0
        } else {
            successful.iter().sum::<f64>() / successful.len() as f64
        }
    }

    /// Calculate average latency (ms) from successful measurements
    fn avg_latency_ms(&self) -> f64 {
        let successful: Vec<f64> = self
            .latency_results
            .iter()
            .filter(|r| r.success)
            .map(|r| r.rtt_ms)
            .collect();
        if successful.is_empty() {
            0.0
        } else {
            successful.iter().sum::<f64>() / successful.len() as f64
        }
    }

    /// Calculate uptime percentage from recent checks
    fn uptime_percent(&self) -> f64 {
        if self.uptime_checks.is_empty() {
            return 100.0; // Assume online if no data
        }
        let online_count = self
            .uptime_checks
            .iter()
            .filter(|(_, online)| *online)
            .count();
        (online_count as f64 / self.uptime_checks.len() as f64) * 100.0
    }

    /// Calculate stability (% of successful measurements)
    fn stability_percent(&self) -> f64 {
        let total =
            self.download_results.len() + self.upload_results.len() + self.latency_results.len();
        if total == 0 {
            return 100.0;
        }
        let successful = self.download_results.iter().filter(|r| r.success).count()
            + self.upload_results.iter().filter(|r| r.success).count()
            + self.latency_results.iter().filter(|r| r.success).count();
        (successful as f64 / total as f64) * 100.0
    }
}

/// Internet speed and quality measurement service
pub struct MeasurementService {
    config: MeasurementConfig,
    history: Arc<RwLock<MeasurementHistory>>,
    last_measurement: Arc<RwLock<Option<Instant>>>,
    http_client: reqwest::Client,
}

impl MeasurementService {
    pub fn new(config: MeasurementConfig) -> Self {
        let history = MeasurementHistory::new(config.history_size);
        Self {
            config,
            history: Arc::new(RwLock::new(history)),
            last_measurement: Arc::new(RwLock::new(None)),
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .unwrap_or_default(),
        }
    }

    /// Get current aggregated metrics
    pub async fn get_metrics(&self) -> AggregatedMetrics {
        let history = self.history.read().await;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        AggregatedMetrics {
            upload_mbps: history.avg_upload_mbps(),
            download_mbps: history.avg_download_mbps(),
            latency_ms: history.avg_latency_ms(),
            uptime_percent: history.uptime_percent(),
            stability_percent: history.stability_percent(),
            sample_count: history.download_results.len()
                + history.upload_results.len()
                + history.latency_results.len(),
            last_updated: now,
        }
    }

    /// Run a complete measurement cycle (download, upload, latency)
    pub async fn run_measurement_cycle(&self) -> AggregatedMetrics {
        // Check minimum interval
        {
            let last = self.last_measurement.read().await;
            if let Some(last_time) = *last {
                if last_time.elapsed() < Duration::from_secs(self.config.min_interval_secs) {
                    return self.get_metrics().await;
                }
            }
        }

        // Update last measurement time
        {
            let mut last = self.last_measurement.write().await;
            *last = Some(Instant::now());
        }

        // Run measurements in parallel
        let download_fut = self.measure_download();
        let upload_fut = self.measure_upload();

        let (download_result, upload_result) = tokio::join!(download_fut, upload_fut);

        // Store results
        {
            let mut history = self.history.write().await;
            history.add_download(download_result);
            history.add_upload(upload_result);

            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            history.add_uptime_check(now, true);
        }

        self.get_metrics().await
    }

    /// Measure download speed
    pub async fn measure_download(&self) -> SpeedResult {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if self.config.download_endpoints.is_empty() {
            return SpeedResult {
                mbps: 0.0,
                bytes_transferred: 0,
                duration_ms: 0,
                timestamp: now,
                success: false,
                error: Some("No download endpoints configured".to_string()),
            };
        }

        // Try each endpoint until one succeeds
        for endpoint in &self.config.download_endpoints {
            let result = self.download_from_endpoint(endpoint).await;
            if result.success {
                return result;
            }
        }

        SpeedResult {
            mbps: 0.0,
            bytes_transferred: 0,
            duration_ms: 0,
            timestamp: now,
            success: false,
            error: Some("All download endpoints failed".to_string()),
        }
    }

    async fn download_from_endpoint(&self, url: &str) -> SpeedResult {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let start = Instant::now();
        let timeout_duration = Duration::from_secs(self.config.timeout_secs);

        match timeout(timeout_duration, self.http_client.get(url).send()).await {
            Ok(Ok(response)) => {
                if !response.status().is_success() {
                    return SpeedResult {
                        mbps: 0.0,
                        bytes_transferred: 0,
                        duration_ms: 0,
                        timestamp: now,
                        success: false,
                        error: Some(format!("HTTP error: {}", response.status())),
                    };
                }

                match timeout(timeout_duration, response.bytes()).await {
                    Ok(Ok(bytes)) => {
                        let duration = start.elapsed();
                        let bytes_len = bytes.len();
                        let duration_ms = duration.as_millis() as u64;
                        let mbps = if duration_ms > 0 {
                            (bytes_len as f64 * 8.0) / (duration_ms as f64 / 1000.0) / 1_000_000.0
                        } else {
                            0.0
                        };

                        SpeedResult {
                            mbps,
                            bytes_transferred: bytes_len,
                            duration_ms,
                            timestamp: now,
                            success: true,
                            error: None,
                        }
                    }
                    Ok(Err(e)) => SpeedResult {
                        mbps: 0.0,
                        bytes_transferred: 0,
                        duration_ms: 0,
                        timestamp: now,
                        success: false,
                        error: Some(format!("Read error: {}", e)),
                    },
                    Err(_) => SpeedResult {
                        mbps: 0.0,
                        bytes_transferred: 0,
                        duration_ms: 0,
                        timestamp: now,
                        success: false,
                        error: Some("Read timeout".to_string()),
                    },
                }
            }
            Ok(Err(e)) => SpeedResult {
                mbps: 0.0,
                bytes_transferred: 0,
                duration_ms: 0,
                timestamp: now,
                success: false,
                error: Some(format!("Request error: {}", e)),
            },
            Err(_) => SpeedResult {
                mbps: 0.0,
                bytes_transferred: 0,
                duration_ms: 0,
                timestamp: now,
                success: false,
                error: Some("Connection timeout".to_string()),
            },
        }
    }

    /// Measure upload speed
    pub async fn measure_upload(&self) -> SpeedResult {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if self.config.upload_endpoints.is_empty() {
            return SpeedResult {
                mbps: 0.0,
                bytes_transferred: 0,
                duration_ms: 0,
                timestamp: now,
                success: false,
                error: Some("No upload endpoints configured".to_string()),
            };
        }

        // Try each endpoint until one succeeds
        for endpoint in &self.config.upload_endpoints {
            let result = self.upload_to_endpoint(endpoint).await;
            if result.success {
                return result;
            }
        }

        SpeedResult {
            mbps: 0.0,
            bytes_transferred: 0,
            duration_ms: 0,
            timestamp: now,
            success: false,
            error: Some("All upload endpoints failed".to_string()),
        }
    }

    async fn upload_to_endpoint(&self, url: &str) -> SpeedResult {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Generate random data to upload
        let data: Vec<u8> = (0..self.config.upload_bytes)
            .map(|i| (i % 256) as u8)
            .collect();
        let data_len = data.len();

        let start = Instant::now();
        let timeout_duration = Duration::from_secs(self.config.timeout_secs);

        match timeout(
            timeout_duration,
            self.http_client
                .post(url)
                .header("Content-Type", "application/octet-stream")
                .body(data)
                .send(),
        )
        .await
        {
            Ok(Ok(response)) => {
                let duration = start.elapsed();
                let duration_ms = duration.as_millis() as u64;
                let mbps = if duration_ms > 0 {
                    (data_len as f64 * 8.0) / (duration_ms as f64 / 1000.0) / 1_000_000.0
                } else {
                    0.0
                };

                SpeedResult {
                    mbps,
                    bytes_transferred: data_len,
                    duration_ms,
                    timestamp: now,
                    success: response.status().is_success(),
                    error: if response.status().is_success() {
                        None
                    } else {
                        Some(format!("HTTP error: {}", response.status()))
                    },
                }
            }
            Ok(Err(e)) => SpeedResult {
                mbps: 0.0,
                bytes_transferred: 0,
                duration_ms: 0,
                timestamp: now,
                success: false,
                error: Some(format!("Request error: {}", e)),
            },
            Err(_) => SpeedResult {
                mbps: 0.0,
                bytes_transferred: 0,
                duration_ms: 0,
                timestamp: now,
                success: false,
                error: Some("Connection timeout".to_string()),
            },
        }
    }

    /// Measure TCP latency to a peer (RTT)
    pub async fn measure_latency(&self, addr: SocketAddr) -> LatencyResult {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let timeout_duration = Duration::from_secs(5);
        let start = Instant::now();

        match timeout(timeout_duration, TcpStream::connect(addr)).await {
            Ok(Ok(_stream)) => {
                let rtt = start.elapsed();
                let result = LatencyResult {
                    target: addr.to_string(),
                    rtt_ms: rtt.as_secs_f64() * 1000.0,
                    success: true,
                    error: None,
                    timestamp: now,
                };

                // Store result
                let mut history = self.history.write().await;
                history.add_latency(result.clone());

                result
            }
            Ok(Err(e)) => {
                let result = LatencyResult {
                    target: addr.to_string(),
                    rtt_ms: 0.0,
                    success: false,
                    error: Some(format!("Connection failed: {}", e)),
                    timestamp: now,
                };

                let mut history = self.history.write().await;
                history.add_latency(result.clone());

                result
            }
            Err(_) => {
                let result = LatencyResult {
                    target: addr.to_string(),
                    rtt_ms: 0.0,
                    success: false,
                    error: Some("Connection timeout".to_string()),
                    timestamp: now,
                };

                let mut history = self.history.write().await;
                history.add_latency(result.clone());

                result
            }
        }
    }

    /// Measure latency to multiple peers and return average
    pub async fn measure_peer_latencies(&self, addrs: &[SocketAddr]) -> f64 {
        if addrs.is_empty() {
            return 0.0;
        }

        let mut results = Vec::new();
        for addr in addrs {
            let result = self.measure_latency(*addr).await;
            if result.success {
                results.push(result.rtt_ms);
            }
        }

        if results.is_empty() {
            0.0
        } else {
            results.iter().sum::<f64>() / results.len() as f64
        }
    }

    /// Record an uptime check (called periodically)
    pub async fn record_uptime(&self, online: bool) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut history = self.history.write().await;
        history.add_uptime_check(now, online);
    }
}

/// Peer-to-peer bandwidth measurement (for challenge-response verification)
pub struct P2PMeasurement;

impl P2PMeasurement {
    /// Measure download speed from a peer by requesting data
    pub async fn measure_download_from_peer(
        peer_addr: SocketAddr,
        bytes_to_request: usize,
        timeout_secs: u64,
    ) -> SpeedResult {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let timeout_duration = Duration::from_secs(timeout_secs);
        let start = Instant::now();

        match timeout(timeout_duration, TcpStream::connect(peer_addr)).await {
            Ok(Ok(mut stream)) => {
                // Send request for data
                let request = format!("SPEEDTEST_REQUEST:{}", bytes_to_request);
                if let Err(e) = stream.write_all(request.as_bytes()).await {
                    return SpeedResult {
                        mbps: 0.0,
                        bytes_transferred: 0,
                        duration_ms: 0,
                        timestamp: now,
                        success: false,
                        error: Some(format!("Write failed: {}", e)),
                    };
                }

                // Read response data
                let mut buffer = vec![0u8; bytes_to_request];
                let mut total_read = 0;

                loop {
                    match timeout(timeout_duration, stream.read(&mut buffer[total_read..])).await {
                        Ok(Ok(0)) => break, // EOF
                        Ok(Ok(n)) => {
                            total_read += n;
                            if total_read >= bytes_to_request {
                                break;
                            }
                        }
                        Ok(Err(e)) => {
                            return SpeedResult {
                                mbps: 0.0,
                                bytes_transferred: total_read,
                                duration_ms: start.elapsed().as_millis() as u64,
                                timestamp: now,
                                success: false,
                                error: Some(format!("Read failed: {}", e)),
                            };
                        }
                        Err(_) => {
                            return SpeedResult {
                                mbps: 0.0,
                                bytes_transferred: total_read,
                                duration_ms: start.elapsed().as_millis() as u64,
                                timestamp: now,
                                success: false,
                                error: Some("Read timeout".to_string()),
                            };
                        }
                    }
                }

                let duration = start.elapsed();
                let duration_ms = duration.as_millis() as u64;
                let mbps = if duration_ms > 0 {
                    (total_read as f64 * 8.0) / (duration_ms as f64 / 1000.0) / 1_000_000.0
                } else {
                    0.0
                };

                SpeedResult {
                    mbps,
                    bytes_transferred: total_read,
                    duration_ms,
                    timestamp: now,
                    success: true,
                    error: None,
                }
            }
            Ok(Err(e)) => SpeedResult {
                mbps: 0.0,
                bytes_transferred: 0,
                duration_ms: 0,
                timestamp: now,
                success: false,
                error: Some(format!("Connection failed: {}", e)),
            },
            Err(_) => SpeedResult {
                mbps: 0.0,
                bytes_transferred: 0,
                duration_ms: 0,
                timestamp: now,
                success: false,
                error: Some("Connection timeout".to_string()),
            },
        }
    }

    /// Handle incoming speed test request from a peer
    pub async fn handle_speedtest_request(
        stream: &mut TcpStream,
        bytes_requested: usize,
    ) -> Result<(), std::io::Error> {
        // Generate and send test data
        let data: Vec<u8> = (0..bytes_requested).map(|i| (i % 256) as u8).collect();
        stream.write_all(&data).await?;
        stream.flush().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_measurement_config_default() {
        let config = MeasurementConfig::default();
        assert!(!config.download_endpoints.is_empty());
        assert!(!config.upload_endpoints.is_empty());
        assert!(config.timeout_secs > 0);
    }

    #[test]
    fn test_speed_result_calculation() {
        // 10 MB in 1 second = 80 Mbps
        let bytes = 10_000_000;
        let duration_ms = 1000;
        let mbps = (bytes as f64 * 8.0) / (duration_ms as f64 / 1000.0) / 1_000_000.0;
        assert!((mbps - 80.0).abs() < 0.01);
    }

    #[test]
    fn test_measurement_history() {
        let mut history = MeasurementHistory::new(3);

        // Add some download results
        for i in 0..5 {
            history.add_download(SpeedResult {
                mbps: (i + 1) as f64 * 10.0,
                bytes_transferred: 1000,
                duration_ms: 100,
                timestamp: i as u64,
                success: true,
                error: None,
            });
        }

        // Should only keep last 3
        assert_eq!(history.download_results.len(), 3);

        // Average should be (30 + 40 + 50) / 3 = 40
        assert!((history.avg_download_mbps() - 40.0).abs() < 0.01);
    }

    #[test]
    fn test_stability_calculation() {
        let mut history = MeasurementHistory::new(10);

        // Add 8 successful and 2 failed
        for i in 0..10 {
            history.add_download(SpeedResult {
                mbps: 50.0,
                bytes_transferred: 1000,
                duration_ms: 100,
                timestamp: i as u64,
                success: i < 8,
                error: if i >= 8 {
                    Some("Failed".to_string())
                } else {
                    None
                },
            });
        }

        // Stability should be 80%
        assert!((history.stability_percent() - 80.0).abs() < 0.01);
    }

    #[test]
    fn test_uptime_calculation() {
        let mut history = MeasurementHistory::new(10);

        // Add 9 online and 1 offline
        for i in 0..10 {
            history.add_uptime_check(i as u64, i < 9);
        }

        // Uptime should be 90%
        assert!((history.uptime_percent() - 90.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_measurement_service_creation() {
        let config = MeasurementConfig::default();
        let service = MeasurementService::new(config);

        let metrics = service.get_metrics().await;
        assert_eq!(metrics.sample_count, 0);
        assert_eq!(metrics.uptime_percent, 100.0);
    }
}
