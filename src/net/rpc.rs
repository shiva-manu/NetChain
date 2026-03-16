// src/net/rpc.rs
//! HTTP RPC server for wallet CLI communication

use anyhow::Result;
use http_body_util::{BodyExt, Full, LengthLimitError, Limited};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::blockchain::Blockchain;
use crate::mempool::Mempool;
use crate::p2p::P2PService;
use crate::rpc_types::{RpcRequest, RpcResponse};
use crate::state::{ProposalStatus, State};
use crate::transaction::SignedTransaction;

const MAX_RPC_BODY_BYTES: usize = 1024 * 1024; // 1 MiB

/// Chain info response
#[derive(Debug, Serialize, Deserialize)]
pub struct ChainInfo {
    pub height: u64,
    pub latest_block_hash: String,
    pub genesis_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountInfo {
    pub address: String,
    pub balance: u64,
    pub nonce: u64,
    pub staked_balance: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StakingInfo {
    pub address: String,
    pub staked_balance: u64,
    pub total_staked: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProposalInfo {
    pub id: u64,
    pub proposer: String,
    pub title: String,
    pub description: String,
    pub created_at: u64,
    pub expires_at: u64,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub status: ProposalStatus,
    pub voter_count: usize,
}

/// Shared state for RPC handlers
pub struct RpcState {
    pub blockchain: Arc<Mutex<Blockchain>>,
    pub state: Arc<Mutex<State>>,
    pub mempool: Arc<Mutex<Mempool>>,
    pub p2p: Arc<Mutex<P2PService>>,
}

/// Handle RPC request
async fn handle_rpc_request(rpc_state: Arc<RpcState>, request: RpcRequest) -> RpcResponse {
    match request {
        RpcRequest::GetBalance { address } => {
            let state = rpc_state.state.lock().await;
            let balance = state.get_balance(&address);
            RpcResponse::success(serde_json::json!({ "balance": balance }))
        }

        RpcRequest::GetNonce { address } => {
            let state = rpc_state.state.lock().await;
            let nonce = state.get_nonce(&address);
            RpcResponse::success(serde_json::json!({ "nonce": nonce }))
        }

        RpcRequest::SendTransaction { tx_json } => {
            // Parse transaction
            let signed_tx: SignedTransaction = match serde_json::from_str(&tx_json) {
                Ok(tx) => tx,
                Err(e) => return RpcResponse::error(format!("Invalid transaction JSON: {}", e)),
            };

            // Validate and add to mempool
            {
                let state = rpc_state.state.lock().await;
                let mut mempool = rpc_state.mempool.lock().await;

                if let Err(e) = mempool.add_transaction(signed_tx.clone(), &state) {
                    return RpcResponse::error(format!("Transaction rejected: {:?}", e));
                }
            }

            // Broadcast via P2P
            {
                let mut p2p = rpc_state.p2p.lock().await;
                p2p.publish_transaction(tx_json);
            }

            let tx_hash = signed_tx.tx_hash_hex();
            RpcResponse::success(serde_json::json!({
                "tx_hash": tx_hash,
                "message": "Transaction submitted successfully"
            }))
        }

        RpcRequest::GetChainInfo => {
            let bc = rpc_state.blockchain.lock().await;
            let info = ChainInfo {
                height: bc.height(),
                latest_block_hash: bc.last_block().hash.clone(),
                genesis_hash: bc.chain.first().map(|b| b.hash.clone()).unwrap_or_default(),
            };
            RpcResponse::success(info)
        }

        RpcRequest::GetMempoolSize => {
            let mempool = rpc_state.mempool.lock().await;
            RpcResponse::success(serde_json::json!({ "size": mempool.len() }))
        }

        RpcRequest::GetBlock { index } => {
            let bc = rpc_state.blockchain.lock().await;
            match bc.get_block(index) {
                Some(block) => RpcResponse::success(block),
                None => RpcResponse::error(format!("Block {} not found", index)),
            }
        }

        RpcRequest::GetBlocks {
            start_height,
            limit,
        } => {
            let bc = rpc_state.blockchain.lock().await;
            let blocks = bc.blocks_paginated(start_height, limit.min(100));
            RpcResponse::success(blocks)
        }

        RpcRequest::GetAccount { address } => {
            let state = rpc_state.state.lock().await;
            let info = AccountInfo {
                address: address.clone(),
                balance: state.get_balance(&address),
                nonce: state.get_nonce(&address),
                staked_balance: state.get_staked_balance(&address),
            };
            RpcResponse::success(info)
        }

        RpcRequest::GetStakingInfo { address } => {
            let state = rpc_state.state.lock().await;
            let info = StakingInfo {
                address: address.clone(),
                staked_balance: state.get_staked_balance(&address),
                total_staked: state.total_staked(),
            };
            RpcResponse::success(info)
        }

        RpcRequest::GetProposals => {
            let state = rpc_state.state.lock().await;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let proposals: Vec<ProposalInfo> = state
                .list_proposals()
                .into_iter()
                .map(|proposal| ProposalInfo {
                    id: proposal.id,
                    proposer: proposal.proposer.clone(),
                    title: proposal.title.clone(),
                    description: proposal.description.clone(),
                    created_at: proposal.created_at,
                    expires_at: proposal.expires_at,
                    yes_votes: proposal.yes_votes,
                    no_votes: proposal.no_votes,
                    status: state.proposal_status(proposal, now),
                    voter_count: proposal.voters.len(),
                })
                .collect();
            RpcResponse::success(proposals)
        }

        RpcRequest::GetProposal { proposal_id } => {
            let state = rpc_state.state.lock().await;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            match state.get_proposal(proposal_id) {
                Some(proposal) => RpcResponse::success(ProposalInfo {
                    id: proposal.id,
                    proposer: proposal.proposer.clone(),
                    title: proposal.title.clone(),
                    description: proposal.description.clone(),
                    created_at: proposal.created_at,
                    expires_at: proposal.expires_at,
                    yes_votes: proposal.yes_votes,
                    no_votes: proposal.no_votes,
                    status: state.proposal_status(proposal, now),
                    voter_count: proposal.voters.len(),
                }),
                None => RpcResponse::error(format!("Proposal {} not found", proposal_id)),
            }
        }

        RpcRequest::GetChainParams => {
            let state = rpc_state.state.lock().await;
            let params = &state.chain_params;
            RpcResponse::success(serde_json::json!({
                "block_reward": params.block_reward,
                "block_interval_secs": params.block_interval_secs,
                "max_txs_per_block": params.max_txs_per_block,
                "stake_weight": params.stake_weight,
                "proposal_quorum_bps": params.proposal_quorum_bps,
                "proposal_approval_bps": params.proposal_approval_bps,
                "min_proposal_stake": params.min_proposal_stake,
            }))
        }
    }
}

/// HTTP request handler
async fn handle_request(
    rpc_state: Arc<RpcState>,
    req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    // Only accept POST to /rpc
    if req.method() != Method::POST || req.uri().path() != "/rpc" {
        let response = RpcResponse::error("Not found. Use POST /rpc");
        let json = serde_json::to_string(&response).unwrap();
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(json)))
            .unwrap());
    }

    // Read body
    let limited_body = Limited::new(req.into_body(), MAX_RPC_BODY_BYTES);
    let body_bytes = match limited_body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            let too_large = err.downcast_ref::<LengthLimitError>().is_some();
            let response = if too_large {
                RpcResponse::error("Request body too large")
            } else {
                RpcResponse::error(format!("Failed to read request body: {}", err))
            };
            let json = serde_json::to_string(&response).unwrap();
            let status = if too_large {
                StatusCode::PAYLOAD_TOO_LARGE
            } else {
                StatusCode::BAD_REQUEST
            };
            return Ok(Response::builder()
                .status(status)
                .header("Content-Type", "application/json")
                .body(Full::new(Bytes::from(json)))
                .unwrap());
        }
    };

    // Parse RPC request
    let rpc_request: RpcRequest = match serde_json::from_slice(&body_bytes) {
        Ok(req) => req,
        Err(e) => {
            let response = RpcResponse::error(format!("Invalid JSON: {}", e));
            let json = serde_json::to_string(&response).unwrap();
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Content-Type", "application/json")
                .body(Full::new(Bytes::from(json)))
                .unwrap());
        }
    };

    // Handle request
    let response = handle_rpc_request(rpc_state, rpc_request).await;
    let json = serde_json::to_string(&response).unwrap();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Full::new(Bytes::from(json)))
        .unwrap())
}

/// Start the RPC server
pub async fn start_rpc_server(rpc_state: Arc<RpcState>, bind_addr: &str, port: u16) -> Result<()> {
    let addr: SocketAddr = format!("{}:{}", bind_addr, port).parse()?;
    let listener = TcpListener::bind(addr).await?;

    info!(address = %addr, "rpc server listening");

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let rpc_state = rpc_state.clone();

        tokio::spawn(async move {
            let service = service_fn(move |req| {
                let rpc_state = rpc_state.clone();
                async move { handle_request(rpc_state, req).await }
            });

            if let Err(e) = http1::Builder::new().serve_connection(io, service).await {
                error!(error = %e, "rpc connection error");
            }
        });
    }
}
