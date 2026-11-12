// src/net/http.rs
//! Single HTTP port for everything a client needs, so the node runs behind one PaaS domain
//! (e.g. Railway) or one reverse-proxy route:
//!
//! | Path | Purpose |
//! |---|---|
//! | `POST /rpc` | JSON-RPC (wallet, website) |
//! | `GET /api/v1/*` | REST explorer API |
//! | `GET /health`, `/metrics`, `/validators` | Monitoring |
//! | `GET /genesis` | This network's genesis file (for new nodes joining) |
//! | `GET /ws` | WebSocket live events |

use anyhow::{Context, Result};
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tracing::{debug, info};

use crate::monitoring::{self, MonitoringState};
use crate::rpc::{self, RpcState};
use crate::websocket::{self, WsEvent};

pub struct Gateway {
    pub rpc: Arc<RpcState>,
    pub monitoring: Arc<MonitoringState>,
    pub ws_tx: broadcast::Sender<WsEvent>,
    /// Pretty-printed genesis spec served at `/genesis`.
    pub genesis_json: String,
}

async fn route(
    gateway: Arc<Gateway>,
    req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    if req.method() == Method::GET {
        let path = req.uri().path().to_string();
        if let Some(response) = monitoring::handle(&gateway.monitoring, &path).await {
            return Ok(response);
        }
        match path.as_str() {
            "/genesis" => {
                return Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .header("Access-Control-Allow-Origin", "*")
                    .body(Full::new(Bytes::from(gateway.genesis_json.clone())))
                    .expect("static response parts are valid"));
            }
            "/ws" => {
                return Ok(
                    websocket::upgrade(req, gateway.ws_tx.clone()).unwrap_or_else(|| {
                        Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(Full::new(Bytes::from("expected a WebSocket upgrade")))
                            .expect("static response parts are valid")
                    }),
                );
            }
            "/" => {
                return Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(Full::new(Bytes::from(
                        r#"{"service":"netchain","endpoints":["POST /rpc","GET /api/v1/*","GET /health","GET /metrics","GET /validators","GET /genesis","GET /ws"]}"#,
                    )))
                    .expect("static response parts are valid"));
            }
            _ => {}
        }
    }
    rpc::handle_request(gateway.rpc.clone(), req).await
}

/// Serve the gateway forever on `bind_addr:port` (`bind_addr` may be `::` for all interfaces).
pub async fn serve(gateway: Arc<Gateway>, bind_addr: &str, port: u16) -> Result<()> {
    let ip: IpAddr = bind_addr
        .parse()
        .with_context(|| format!("invalid HTTP bind address {bind_addr:?}"))?;
    let addr = SocketAddr::new(ip, port);
    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind HTTP gateway on {addr}"))?;
    info!(address = %addr, "http gateway listening (rpc, api, health, metrics, ws)");

    loop {
        let (stream, _) = listener.accept().await?;
        let gateway = gateway.clone();
        tokio::spawn(async move {
            let service = service_fn(move |req| route(gateway.clone(), req));
            if let Err(e) = http1::Builder::new()
                .serve_connection(TokioIo::new(stream), service)
                .with_upgrades()
                .await
            {
                debug!(error = %e, "http connection closed with error");
            }
        });
    }
}
