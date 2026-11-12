use anyhow::Result;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use serde::Serialize;
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::blockchain::Blockchain;
use crate::engine::{self, ValidatorScore};
use crate::measurement::MeasurementService;
use crate::mempool::Mempool;
use crate::p2p::P2PSharedState;
use crate::state::State;

/// Blocks per informational "epoch" reported by the dashboard.
const EPOCH_LENGTH: u64 = 100;

pub struct MonitoringState {
    pub blockchain: Arc<Mutex<Blockchain>>,
    pub mempool: Arc<Mutex<Mempool>>,
    pub p2p_shared: Arc<P2PSharedState>,
    pub measurement: Arc<MeasurementService>,
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
    /// Active validators (stake >= min_validator_stake).
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

async fn validator_snapshot(state: &MonitoringState) -> Vec<ValidatorScore> {
    let tip_time = state
        .blockchain
        .lock()
        .await
        .last_block()
        .timestamp
        .timestamp();
    let state_guard = state.state.lock().await;
    engine::validator_scores(&state_guard, tip_time.max(0) as u64)
}

async fn health_snapshot(state: &MonitoringState) -> HealthResponse {
    let (chain_height, latest_block_age_secs) = {
        let bc = state.blockchain.lock().await;
        let age = chrono::Utc::now().timestamp() - bc.last_block().timestamp.timestamp();
        (bc.height(), age)
    };
    let mempool_size = state.mempool.lock().await.len();
    let peer_count = state.p2p_shared.get_peer_count();
    let validators = validator_snapshot(state).await;
    let min_attestations = state
        .state
        .lock()
        .await
        .chain_params
        .min_attestations
        .max(1);
    let verified = validators
        .iter()
        .filter(|v| v.attestations >= min_attestations)
        .count();
    let avg = |f: &dyn Fn(&ValidatorScore) -> u64| {
        if validators.is_empty() {
            0.0
        } else {
            validators.iter().map(|v| f(v) as f64).sum::<f64>()
                / (validators.len() as f64 * engine::BPS as f64)
        }
    };
    let slashed_validator_count = {
        let state_guard = state.state.lock().await;
        state_guard
            .slashing_records
            .iter()
            .map(|record| record.validator.clone())
            .collect::<HashSet<_>>()
            .len()
    };

    HealthResponse {
        status: if latest_block_age_secs < 300 {
            "ok"
        } else {
            "stalled"
        },
        consensus_mode: "hybrid-poi",
        chain_id: state.chain_id.clone(),
        node_address: state.node_address.clone(),
        peer_id: state.peer_id.clone(),
        is_validator: validators.iter().any(|v| v.address == state.node_address),
        uptime_secs: state.started_at.elapsed().as_secs(),
        chain_height,
        latest_block_age_secs,
        mempool_size,
        peer_count,
        validator_count: validators.len(),
        verified_validator_count: verified,
        unverified_validator_count: validators.len() - verified,
        aggregator_nodes: peer_count + 1,
        current_epoch: chain_height / EPOCH_LENGTH,
        slashed_validator_count,
        average_reputation: avg(&|v| v.uptime_bps),
        average_identity_score: avg(&|v| v.poi_bps),
    }
}

fn response(status: StatusCode, content_type: &str, body: String) -> Response<Full<Bytes>> {
    let res = Response::builder()
        .status(status)
        .header("Content-Type", content_type)
        .header("Access-Control-Allow-Origin", "*")
        .body(Full::new(Bytes::from(body)))
        .unwrap();
    res
}

async fn handle_request(
    state: Arc<MonitoringState>,
    req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/health") => {
            let snapshot = health_snapshot(&state).await;
            let body = serde_json::to_string(&snapshot).unwrap_or_else(|_| "{}".to_string());
            Ok(response(StatusCode::OK, "application/json", body))
        }
        (&Method::GET, "/validators") => {
            let tip_time = state
                .blockchain
                .lock()
                .await
                .last_block()
                .timestamp
                .timestamp();
            let validators = {
                let state_guard = state.state.lock().await;
                engine::validator_report(&state_guard, tip_time.max(0) as u64)
            };
            let body = serde_json::to_string(&validators).unwrap_or_else(|_| "[]".to_string());
            Ok(response(StatusCode::OK, "application/json", body))
        }
        (&Method::GET, "/metrics") => {
            let snapshot = health_snapshot(&state).await;
            let poi_metrics = state.measurement.get_metrics().await;

            let body = format!(
                "# HELP netchain_chain_height Current blockchain height\n\
# TYPE netchain_chain_height gauge\n\
netchain_chain_height {}\n\
# HELP netchain_mempool_size Number of transactions in mempool\n\
# TYPE netchain_mempool_size gauge\n\
netchain_mempool_size {}\n\
# HELP netchain_peer_count Number of connected peers\n\
# TYPE netchain_peer_count gauge\n\
netchain_peer_count {}\n\
# HELP netchain_validator_count Number of validators in pool\n\
# TYPE netchain_validator_count gauge\n\
netchain_validator_count {}\n\
# HELP netchain_verified_validator_count Number of validators with quorum attestations\n\
# TYPE netchain_verified_validator_count gauge\n\
netchain_verified_validator_count {}\n\
# HELP netchain_unverified_validator_count Number of validators below attestation quorum\n\
# TYPE netchain_unverified_validator_count gauge\n\
netchain_unverified_validator_count {}\n\
# HELP netchain_aggregator_nodes Number of nodes tracked by metrics aggregator\n\
# TYPE netchain_aggregator_nodes gauge\n\
netchain_aggregator_nodes {}\n\
# HELP netchain_current_epoch Current metrics epoch\n\
# TYPE netchain_current_epoch gauge\n\
netchain_current_epoch {}\n\
# HELP netchain_slashed_validator_count Number of validators with slash history\n\
# TYPE netchain_slashed_validator_count gauge\n\
netchain_slashed_validator_count {}\n\
# HELP netchain_average_reputation Average validator reputation score\n\
# TYPE netchain_average_reputation gauge\n\
netchain_average_reputation {}\n\
# HELP netchain_average_identity_score Average validator identity confidence score\n\
# TYPE netchain_average_identity_score gauge\n\
netchain_average_identity_score {}\n\
# HELP netchain_uptime_seconds Process uptime in seconds\n\
# TYPE netchain_uptime_seconds counter\n\
netchain_uptime_seconds {}\n\
# HELP netchain_download_mbps Download speed in Mbps\n\
# TYPE netchain_download_mbps gauge\n\
netchain_download_mbps {}\n\
# HELP netchain_upload_mbps Upload speed in Mbps\n\
# TYPE netchain_upload_mbps gauge\n\
netchain_upload_mbps {}\n\
# HELP netchain_latency_ms Latency in ms\n\
# TYPE netchain_latency_ms gauge\n\
netchain_latency_ms {}\n\
# HELP netchain_uptime_percent Uptime percentage\n\
# TYPE netchain_uptime_percent gauge\n\
netchain_uptime_percent {}\n",
                snapshot.chain_height,
                snapshot.mempool_size,
                snapshot.peer_count,
                snapshot.validator_count,
                snapshot.verified_validator_count,
                snapshot.unverified_validator_count,
                snapshot.aggregator_nodes,
                snapshot.current_epoch,
                snapshot.slashed_validator_count,
                snapshot.average_reputation,
                snapshot.average_identity_score,
                snapshot.uptime_secs,
                poi_metrics.download_mbps,
                poi_metrics.upload_mbps,
                poi_metrics.latency_ms,
                poi_metrics.uptime_percent,
            );
            Ok(response(StatusCode::OK, "text/plain; version=0.0.4", body))
        }
        _ => Ok(response(
            StatusCode::NOT_FOUND,
            "application/json",
            "{\"error\":\"not found\"}".to_string(),
        )),
    }
}

pub async fn start_monitoring_server(
    state: Arc<MonitoringState>,
    bind_addr: &str,
    port: u16,
) -> Result<()> {
    let addr: SocketAddr = format!("{}:{}", bind_addr, port).parse()?;
    let listener = TcpListener::bind(addr).await?;
    info!(address = %addr, "monitoring server listening");

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let state = state.clone();

        tokio::spawn(async move {
            let service = service_fn(move |req| {
                let state = state.clone();
                async move { handle_request(state, req).await }
            });

            if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                error!(error = %err, "monitoring connection error");
            }
        });
    }
}
