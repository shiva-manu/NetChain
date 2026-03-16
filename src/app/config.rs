use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::anti_gaming::AntiGamingConfig;
use crate::measurement::MeasurementConfig;
use crate::metrics_aggregator::AggregatorConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub node: NodeConfig,
    pub rpc: ServerConfig,
    pub monitoring: MonitoringConfig,
    pub websocket: WebSocketConfig,
    pub producer: ProducerRuntimeConfig,
    pub measurement: MeasurementConfig,
    pub aggregator: AggregatorConfig,
    pub anti_gaming: AntiGamingConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            node: NodeConfig::default(),
            rpc: ServerConfig::default_rpc(),
            monitoring: MonitoringConfig::default(),
            websocket: WebSocketConfig::default(),
            producer: ProducerRuntimeConfig::default(),
            measurement: MeasurementConfig::default(),
            aggregator: AggregatorConfig::default(),
            anti_gaming: AntiGamingConfig::default(),
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let config_path = std::env::var("NETCHAIN_CONFIG")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("config/default.toml"));

        let mut config = if config_path.exists() {
            Self::load_from_file(&config_path)?
        } else {
            Self::default()
        };

        config.apply_env_overrides();
        Ok(config)
    }

    pub fn load_from_file(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("failed to read config file {}", path.display()))?;
        toml::from_str(&contents)
            .with_context(|| format!("failed to parse config file {}", path.display()))
    }

    pub fn data_dir(&self) -> PathBuf {
        PathBuf::from(&self.node.data_dir)
    }

    fn apply_env_overrides(&mut self) {
        if let Ok(value) = std::env::var("DATA_DIR") {
            self.node.data_dir = value;
        }
        if let Ok(value) = std::env::var("PORT") {
            if let Ok(port) = value.parse() {
                self.node.p2p_port = port;
            }
        }
        if let Ok(value) = std::env::var("RPC_PORT") {
            if let Ok(port) = value.parse() {
                self.rpc.port = port;
            }
        }
        if let Ok(value) = std::env::var("NETCHAIN_RPC_BIND_ADDR") {
            self.rpc.bind_addr = value;
        }
        if let Ok(value) = std::env::var("NETCHAIN_MONITORING_PORT") {
            if let Ok(port) = value.parse() {
                self.monitoring.port = port;
            }
        }
        if let Ok(value) = std::env::var("NETCHAIN_MONITORING_BIND_ADDR") {
            self.monitoring.bind_addr = value;
        }
        if let Ok(value) = std::env::var("NETCHAIN_LOG_LEVEL") {
            self.node.log_level = value;
        }
        if let Ok(value) = std::env::var("NETCHAIN_WS_PORT") {
            if let Ok(port) = value.parse() {
                self.websocket.port = port;
            }
        }
        if let Ok(value) = std::env::var("NETCHAIN_WS_BIND_ADDR") {
            self.websocket.bind_addr = value;
        }
        if let Ok(value) = std::env::var("NETCHAIN_BLOCK_INTERVAL_SECS") {
            if let Ok(parsed) = value.parse() {
                self.producer.block_interval_secs = parsed;
            }
        }
        if let Ok(value) = std::env::var("NETCHAIN_BLOCK_REWARD") {
            if let Ok(parsed) = value.parse() {
                self.producer.block_reward = parsed;
            }
        }
        if let Ok(value) = std::env::var("NETCHAIN_MAX_TXS_PER_BLOCK") {
            if let Ok(parsed) = value.parse() {
                self.producer.max_txs_per_block = parsed;
            }
        }
        if let Ok(value) = std::env::var("NETCHAIN_STAKE_WEIGHT") {
            if let Ok(parsed) = value.parse::<f64>() {
                self.producer.stake_weight = parsed.clamp(0.0, 1.0);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NodeConfig {
    pub data_dir: String,
    pub p2p_port: u16,
    pub log_level: String,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            data_dir: "./data".to_string(),
            p2p_port: 30333,
            log_level: "info".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    pub bind_addr: String,
    pub port: u16,
}

impl ServerConfig {
    fn default_rpc() -> Self {
        Self {
            bind_addr: "127.0.0.1".to_string(),
            port: 8545,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self::default_rpc()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MonitoringConfig {
    pub bind_addr: String,
    pub port: u16,
    pub enabled: bool,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1".to_string(),
            port: 9090,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WebSocketConfig {
    pub bind_addr: String,
    pub port: u16,
    pub enabled: bool,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1".to_string(),
            port: 8546,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProducerRuntimeConfig {
    pub max_txs_per_block: usize,
    pub block_interval_secs: u64,
    pub block_reward: u64,
    pub metric_measurement_interval_secs: u64,
    /// How much stake weight influences validator selection (0.0 = pure PoI, 1.0 = pure PoS)
    pub stake_weight: f64,
}

impl Default for ProducerRuntimeConfig {
    fn default() -> Self {
        Self {
            max_txs_per_block: 100,
            block_interval_secs: 15,
            block_reward: 50,
            metric_measurement_interval_secs: 120,
            stake_weight: 0.3,
        }
    }
}
