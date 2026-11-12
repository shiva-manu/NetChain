// src/net/monitoring.rs
//! Monitoring endpoints served by the HTTP gateway:
//! `/health` (JSON), `/metrics` (Prometheus text) and `/validators` (JSON).

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Response, StatusCode};
use serde::Serialize;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

use crate::blockchain::Blockchain;
use crate::engine::{self, ValidatorScore};
use crate::mempool::Mempool;
use crate::p2p::P2PSharedState;
use crate::state::State;

/// Blocks per informational "epoch" reported by the dashboard.
const EPOCH_LENGTH: u64 = 100;
/// `/health` reports "stalled" when the newest block is older than this.
const STALLED_AFTER_SECS: i64 = 300;

pub struct MonitoringState {
    pub blockchain: Arc<Mutex<Blockchain>>,
    pub mempool: Arc<Mutex<Mempool>>,
    pub p2p_shared: Arc<P2PSharedState>,
    pub state: Arc<Mutex<State>>,
    pub started_at: Instant,
    pub chain_id: String,
    pub node_address: String,
    pub peer_id: String,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    consensus_mode: &'static str,
    chain_id: String,
    node_address: String,
    peer_id: String,
    is_validator: bool,
    uptime_secs: u64,
    chain_height: u64,
    latest_block_age_secs: i64,
    mempool_size: usize,
    peer_count: usize,
    /// Active validators (stake >= min_validator_stake, not jailed).
    validator_count: usize,
    /// Active validators with enough recent on-chain PoI attestations.
    verified_validator_count: usize,
    unverified_validator_count: usize,
    aggregator_nodes: usize,
    current_epoch: u64,
    slashed_validator_count: usize,
    /// Average on-chain uptime score (0..1).
    average_reputation: f64,
    /// Average on-chain PoI score (0..1).
    average_identity_score: f64,
}

/// Active validators followed by jailed ones, scored at the current tip.
async fn validator_report(state: &MonitoringState) -> Vec<ValidatorScore> {
    let tip_time = state
        .blockchain
        .lock()
        .await
        .last_block()
        .timestamp
        .timestamp();
    let state_guard = state.state.lock().await;
    engine::validator_report(&state_guard, tip_time.max(0) as u64)
}

async fn health_snapshot(state: &MonitoringState, validators: &[ValidatorScore]) -> HealthResponse {
    let (chain_height, latest_block_age_secs) = {
        let bc = state.blockchain.lock().await;
        let age = chrono::Utc::now().timestamp() - bc.last_block().timestamp.timestamp();
        (bc.height(), age)
    };
    let mempool_size = state.mempool.lock().await.len();
    let peer_count = state.p2p_shared.get_peer_count();
    let (min_attestations, slashed_validator_count) = {
        let state_guard = state.state.lock().await;
        let slashed = state_guard
            .slashing_records
            .iter()
            .map(|record| record.validator.clone())
            .collect::<HashSet<_>>()
            .len();
        (state_guard.chain_params.min_attestations.max(1), slashed)
    };

    let active: Vec<&ValidatorScore> = validators.iter().filter(|v| !v.jailed).collect();
    let verified = active
        .iter()
        .filter(|v| v.attestations >= min_attestations)
        .count();
    let avg = |f: fn(&ValidatorScore) -> u64| {
        if active.is_empty() {
            0.0
        } else {
            active.iter().map(|v| f(v) as f64).sum::<f64>()
                / (active.len() as f64 * engine::BPS as f64)
        }
    };

    HealthResponse {
        status: if latest_block_age_secs < STALLED_AFTER_SECS {
            "ok"
        } else {
            "stalled"
        },
        consensus_mode: "hybrid-poi",
        chain_id: state.chain_id.clone(),
        node_address: state.node_address.clone(),
        peer_id: state.peer_id.clone(),
        is_validator: active.iter().any(|v| v.address == state.node_address),
        uptime_secs: state.started_at.elapsed().as_secs(),
        chain_height,
        latest_block_age_secs,
        mempool_size,
        peer_count,
        validator_count: active.len(),
        verified_validator_count: verified,
        unverified_validator_count: active.len() - verified,
        aggregator_nodes: peer_count + 1,
        current_epoch: chain_height / EPOCH_LENGTH,
        slashed_validator_count,
        average_reputation: avg(|v| v.uptime_bps),
        average_identity_score: avg(|v| v.poi_bps),
    }
}

fn metrics_text(health: &HealthResponse, me: Option<&ValidatorScore>) -> String {
    let mut out = String::new();
    let mut gauge = |name: &str, help: &str, value: String| {
        out.push_str(&format!(
            "# HELP {name} {help}\n# TYPE {name} gauge\n{name} {value}\n"
        ));
    };
    gauge(
        "netchain_chain_height",
        "Current blockchain height",
        health.chain_height.to_string(),
    );
    gauge(
        "netchain_mempool_size",
        "Transactions in the mempool",
        health.mempool_size.to_string(),
    );
    gauge(
        "netchain_peer_count",
        "Connected peers",
        health.peer_count.to_string(),
    );
    gauge(
        "netchain_validator_count",
        "Active validators",
        health.validator_count.to_string(),
    );
    gauge(
        "netchain_verified_validator_count",
        "Active validators with enough PoI attestations",
        health.verified_validator_count.to_string(),
    );
    gauge(
        "netchain_unverified_validator_count",
        "Active validators below the attestation threshold",
        health.unverified_validator_count.to_string(),
    );
    gauge(
        "netchain_aggregator_nodes",
        "Known nodes (peers + self)",
        health.aggregator_nodes.to_string(),
    );
    gauge(
        "netchain_current_epoch",
        "Informational epoch (height / 100)",
        health.current_epoch.to_string(),
    );
    gauge(
        "netchain_slashed_validator_count",
        "Validators with slash history",
        health.slashed_validator_count.to_string(),
    );
    gauge(
        "netchain_average_reputation",
        "Average validator uptime (0..1)",
        health.average_reputation.to_string(),
    );
    gauge(
        "netchain_average_identity_score",
        "Average validator PoI score (0..1)",
        health.average_identity_score.to_string(),
    );
    gauge(
        "netchain_latest_block_age_seconds",
        "Age of the newest block",
        health.latest_block_age_secs.to_string(),
    );
    gauge(
        "netchain_uptime_seconds",
        "Process uptime in seconds",
        health.uptime_secs.to_string(),
    );

    // This node's network performance as measured by other validators and recorded on-chain.
    if let Some(me) = me {
        if let Some(kbps) = me.median_throughput_kbps {
            gauge(
                "netchain_upload_mbps",
                "Throughput peers measured downloading from this node (Mbps)",
                format!("{:.3}", kbps as f64 / 1000.0),
            );
        }
        if let Some(rtt) = me.median_rtt_ms {
            gauge(
                "netchain_latency_ms",
                "Round-trip time peers measured to this node (ms)",
                rtt.to_string(),
            );
        }
        gauge(
            "netchain_uptime_percent",
            "This validator's on-chain uptime (%)",
            format!("{:.2}", me.uptime_bps as f64 / 100.0),
        );
    }
    out
}

fn response(status: StatusCode, content_type: &str, body: String) -> Response<Full<Bytes>> {
    Response::builder()
        .status(status)
        .header("Content-Type", content_type)
        .header("Access-Control-Allow-Origin", "*")
        .body(Full::new(Bytes::from(body)))
        .expect("static response parts are valid")
}

/// Handle a monitoring path, or return `None` if `path` is not a monitoring endpoint.
pub async fn handle(state: &MonitoringState, path: &str) -> Option<Response<Full<Bytes>>> {
    match path {
        "/health" => {
            let validators = validator_report(state).await;
            let health = health_snapshot(state, &validators).await;
            let body = serde_json::to_string(&health).unwrap_or_else(|_| "{}".to_string());
            Some(response(StatusCode::OK, "application/json", body))
        }
        "/validators" => {
            let validators = validator_report(state).await;
            let body = serde_json::to_string(&validators).unwrap_or_else(|_| "[]".to_string());
            Some(response(StatusCode::OK, "application/json", body))
        }
        "/metrics" => {
            let validators = validator_report(state).await;
            let health = health_snapshot(state, &validators).await;
            let me = validators.iter().find(|v| v.address == state.node_address);
            Some(response(
                StatusCode::OK,
                "text/plain; version=0.0.4",
                metrics_text(&health, me),
            ))
        }
        _ => None,
    }
}
