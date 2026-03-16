use anyhow::Result;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::blockchain::Blockchain;
use crate::mempool::Mempool;
use crate::metrics_aggregator::MetricsAggregator;
use crate::p2p::P2PService;
use crate::producer::BlockProducer;

pub struct MonitoringState {
    pub blockchain: Arc<Mutex<Blockchain>>,
    pub mempool: Arc<Mutex<Mempool>>,
    pub p2p: Arc<Mutex<P2PService>>,
    pub producer: Arc<Mutex<BlockProducer>>,
    pub aggregator: Arc<Mutex<MetricsAggregator>>,
    pub started_at: Instant,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    uptime_secs: u64,
    chain_height: u64,
    mempool_size: usize,
    peer_count: usize,
    validator_count: usize,
    aggregator_nodes: usize,
    current_epoch: u64,
}

async fn health_snapshot(state: &MonitoringState) -> HealthResponse {
    let chain_height = state.blockchain.lock().await.height();
    let mempool_size = state.mempool.lock().await.len();
    let peer_count = state.p2p.lock().await.connected_peer_count();
    let validator_count = state.producer.lock().await.validator_count();
    let (aggregator_nodes, current_epoch) = {
        let aggregator = state.aggregator.lock().await;
        (aggregator.node_count(), aggregator.current_epoch())
    };

    HealthResponse {
        status: "ok",
        uptime_secs: state.started_at.elapsed().as_secs(),
        chain_height,
        mempool_size,
        peer_count,
        validator_count,
        aggregator_nodes,
        current_epoch,
    }
}

fn response(status: StatusCode, content_type: &str, body: String) -> Response<Full<Bytes>> {
    Response::builder()
        .status(status)
        .header("Content-Type", content_type)
        .body(Full::new(Bytes::from(body)))
        .unwrap()
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
        (&Method::GET, "/metrics") => {
            let snapshot = health_snapshot(&state).await;
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
# HELP netchain_aggregator_nodes Number of nodes tracked by metrics aggregator\n\
# TYPE netchain_aggregator_nodes gauge\n\
netchain_aggregator_nodes {}\n\
# HELP netchain_current_epoch Current metrics epoch\n\
# TYPE netchain_current_epoch gauge\n\
netchain_current_epoch {}\n\
# HELP netchain_uptime_seconds Process uptime in seconds\n\
# TYPE netchain_uptime_seconds counter\n\
netchain_uptime_seconds {}\n",
                snapshot.chain_height,
                snapshot.mempool_size,
                snapshot.peer_count,
                snapshot.validator_count,
                snapshot.aggregator_nodes,
                snapshot.current_epoch,
                snapshot.uptime_secs,
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
