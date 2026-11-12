// src/app/config.rs
//! Runtime configuration: `config/default.toml` (or `NETCHAIN_CONFIG`) plus environment
//! overrides. Environment variables follow PaaS conventions so the same image runs locally,
//! in Docker and on Railway: `PORT` is the HTTP port, `DATA_DIR` the persistent directory.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use crate::state::ChainParams;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub node: NodeConfig,
    pub http: HttpConfig,
    pub producer: ProducerRuntimeConfig,
    pub faucet: FaucetConfig,
    pub poi: PoiRuntimeConfig,
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

    /// Chain parameters used when this node creates a development genesis.
    pub fn dev_chain_params(&self) -> ChainParams {
        ChainParams {
            block_reward: self.producer.block_reward,
            block_interval_secs: self.producer.block_interval_secs.max(1),
            max_txs_per_block: self.producer.max_txs_per_block,
            stake_weight: self.producer.stake_weight.clamp(0.0, 1.0),
            ..ChainParams::default()
        }
    }

    fn apply_env_overrides(&mut self) {
        let var = |name: &str| std::env::var(name).ok().filter(|v| !v.trim().is_empty());
        let flag = |v: String| matches!(v.trim(), "1" | "true" | "yes");
        let list = |v: String| -> Vec<String> {
            v.split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect()
        };

        if let Some(v) = var("DATA_DIR") {
            self.node.data_dir = v;
        }
        if let Some(v) = var("NETCHAIN_LOG_LEVEL") {
            self.node.log_level = v;
        }
        if let Some(port) = var("NETCHAIN_P2P_PORT").and_then(|v| v.parse().ok()) {
            self.node.p2p_port = port;
        }
        if let Some(v) = var("NETCHAIN_BOOTNODES") {
            self.node.bootnodes = list(v);
        }
        if let Some(v) = var("NETCHAIN_EXTERNAL_ADDRS") {
            self.node.external_addrs = list(v);
        }
        // Railway TCP proxy: advertise the public proxy address to peers.
        if let (Some(domain), Some(port)) = (
            var("RAILWAY_TCP_PROXY_DOMAIN"),
            var("RAILWAY_TCP_PROXY_PORT"),
        ) {
            let addr = format!("/dns4/{domain}/tcp/{port}");
            if !self.node.external_addrs.contains(&addr) {
                self.node.external_addrs.push(addr);
            }
        }
        if let Some(v) = var("NETCHAIN_ENABLE_MDNS") {
            self.node.enable_mdns = flag(v);
        }
        if let Some(v) = var("NETCHAIN_GENESIS") {
            self.node.genesis_file = Some(v);
        }
        if let Some(v) = var("NETCHAIN_GENESIS_JSON") {
            self.node.genesis_json = Some(v);
        }
        if let Some(v) = var("NETCHAIN_GENESIS_URL") {
            self.node.genesis_url = Some(v);
        }
        if let Some(v) = var("NETCHAIN_NODE_KEY") {
            self.node.node_key = Some(Secret(v));
        }

        // HTTP gateway: PORT is the PaaS convention; RPC_PORT is kept for older setups.
        for name in ["RPC_PORT", "PORT", "NETCHAIN_HTTP_PORT"] {
            if let Some(port) = var(name).and_then(|v| v.parse().ok()) {
                self.http.port = port;
            }
        }
        for name in ["NETCHAIN_RPC_BIND_ADDR", "NETCHAIN_HTTP_BIND_ADDR"] {
            if let Some(v) = var(name) {
                self.http.bind_addr = v;
            }
        }

        if let Some(v) = var("NETCHAIN_BLOCK_INTERVAL_SECS").and_then(|v| v.parse().ok()) {
            self.producer.block_interval_secs = v;
        }
        if let Some(v) = var("NETCHAIN_BLOCK_REWARD").and_then(|v| v.parse().ok()) {
            self.producer.block_reward = v;
        }
        if let Some(v) = var("NETCHAIN_MAX_TXS_PER_BLOCK").and_then(|v| v.parse().ok()) {
            self.producer.max_txs_per_block = v;
        }
        if let Some(v) = var("NETCHAIN_STAKE_WEIGHT").and_then(|v| v.parse::<f64>().ok()) {
            self.producer.stake_weight = v.clamp(0.0, 1.0);
        }
        if let Some(v) = var("NETCHAIN_MEMPOOL_TTL_SECS").and_then(|v| v.parse().ok()) {
            self.producer.mempool_ttl_secs = v;
        }

        if let Some(v) = var("NETCHAIN_FAUCET_ENABLED") {
            self.faucet.enabled = flag(v);
        }
        if let Some(v) = var("NETCHAIN_FAUCET_KEY_FILE") {
            self.faucet.key_file = Some(v);
        }
        if let Some(v) = var("NETCHAIN_FAUCET_AMOUNT").and_then(|v| v.parse().ok()) {
            self.faucet.amount = v;
        }
        if let Some(v) = var("NETCHAIN_ATTEST_INTERVAL_SECS").and_then(|v| v.parse().ok()) {
            self.poi.attest_interval_secs = v;
        }
    }
}

/// A string that must never be printed (e.g. a private key).
#[derive(Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Secret(pub String);

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NodeConfig {
    pub data_dir: String,
    pub p2p_port: u16,
    pub log_level: String,
    /// Genesis JSON file shared by every node of the network.
    pub genesis_file: Option<String>,
    /// Genesis JSON given inline (handy as a PaaS environment variable).
    pub genesis_json: Option<String>,
    /// URL of a node's `/genesis` endpoint to download the network's genesis from.
    pub genesis_url: Option<String>,
    /// Node key as hex (otherwise `<data_dir>/node_key` is used / created).
    pub node_key: Option<Secret>,
    /// Multiaddrs of peers to connect to, e.g. `/dns4/node.example.com/tcp/30333`.
    pub bootnodes: Vec<String>,
    /// Publicly reachable addresses of this node, advertised to peers.
    pub external_addrs: Vec<String>,
    /// Discover peers on the local network with mDNS.
    pub enable_mdns: bool,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            data_dir: "./data".to_string(),
            p2p_port: 30333,
            log_level: "info".to_string(),
            genesis_file: None,
            genesis_json: None,
            genesis_url: None,
            node_key: None,
            bootnodes: Vec::new(),
            external_addrs: Vec::new(),
            enable_mdns: true,
        }
    }
}

/// One HTTP port serves JSON-RPC (`/rpc`), REST (`/api/v1/*`), monitoring (`/health`,
/// `/metrics`, `/validators`), the genesis file (`/genesis`) and WebSocket events (`/ws`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HttpConfig {
    pub bind_addr: String,
    pub port: u16,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1".to_string(),
            port: 8545,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProducerRuntimeConfig {
    pub max_txs_per_block: usize,
    pub block_interval_secs: u64,
    pub block_reward: u64,
    /// How much stake weight influences validator selection (0.0 = pure PoI, 1.0 = pure PoS)
    pub stake_weight: f64,
    /// Time-to-live for mempool transactions in seconds (default 900 = 15 min)
    pub mempool_ttl_secs: u64,
}

impl Default for ProducerRuntimeConfig {
    fn default() -> Self {
        Self {
            max_txs_per_block: 100,
            block_interval_secs: 15,
            block_reward: 50,
            stake_weight: 0.3,
            mempool_ttl_secs: 900,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FaucetConfig {
    pub enabled: bool,
    /// Key that holds the faucet funds (hex secret, same format as `node_key`).
    /// Defaults to the node key.
    pub key_file: Option<String>,
    pub amount: u64,
    pub fee: u64,
    pub cooldown_secs: u64,
    pub max_per_minute: usize,
}

impl Default for FaucetConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            key_file: None,
            amount: 200,
            fee: 1,
            cooldown_secs: 3_600,
            max_per_minute: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PoiRuntimeConfig {
    /// How often a validator measures its validator peers and records attestations on-chain.
    pub attest_interval_secs: u64,
    /// Bytes requested per throughput probe.
    pub probe_bytes: usize,
    /// Fee paid per attestation transaction.
    pub attestation_fee: u64,
}

impl Default for PoiRuntimeConfig {
    fn default() -> Self {
        Self {
            attest_interval_secs: 120,
            probe_bytes: 1_000_000,
            attestation_fee: 1,
        }
    }
}
