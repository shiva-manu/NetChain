// src/net/rpc.rs
//! HTTP RPC server for wallet CLI communication

use anyhow::Result;
use http_body_util::{BodyExt, Full, LengthLimitError, Limited};
use hyper::body::Bytes;
use hyper::header::{
    HeaderValue, ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS,
    ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_TYPE,
};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::blockchain::Blockchain;
use crate::mempool::Mempool;
use crate::p2p::P2PHandle;
use crate::rest_types::{RestError, RestResponse};
use crate::rpc_types::{RpcRequest, RpcResponse};
use crate::state::{ProposalStatus, State};
use crate::transaction::SignedTransaction;
use std::collections::HashMap;

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
    pub p2p: P2PHandle,
    /// Transaction index: tx_hash -> (block_height, tx_index_in_block)
    pub tx_index: Arc<Mutex<HashMap<String, (u64, usize)>>>,
}

fn json_response(status: StatusCode, body: String) -> Response<Full<Bytes>> {
    Response::builder()
        .status(status)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"))
        .header(
            ACCESS_CONTROL_ALLOW_METHODS,
            HeaderValue::from_static("GET, POST, OPTIONS"),
        )
        .header(
            ACCESS_CONTROL_ALLOW_HEADERS,
            HeaderValue::from_static("content-type"),
        )
        .body(Full::new(Bytes::from(body)))
        .unwrap()
}

/// Parse query string into key-value pairs.
fn parse_query(query: &str) -> HashMap<String, String> {
    query
        .split('&')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next()?.to_string();
            let value = parts.next().unwrap_or("").to_string();
            Some((key, value))
        })
        .collect()
}

/// Build a transaction index from the current blockchain.
pub async fn build_tx_index(blockchain: &Blockchain) -> HashMap<String, (u64, usize)> {
    let mut index = HashMap::new();
    for block in &blockchain.chain {
        for (tx_idx, signed_tx) in block.transactions.iter().enumerate() {
            let tx_hash = signed_tx.tx_hash_hex();
            index.insert(tx_hash, (block.index, tx_idx));
        }
    }
    index
}

/// Handle REST API requests.
async fn handle_rest_request(
    rpc_state: Arc<RpcState>,
    method: &Method,
    path: &str,
    query: Option<&str>,
) -> Response<Full<Bytes>> {
    // Parse optional pagination params
    let params: HashMap<String, String> = query.map(parse_query).unwrap_or_default();
    let get_limit_offset = |params: &HashMap<String, String>| -> (u64, u64) {
        let limit = params
            .get("limit")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(25)
            .min(100);
        let offset = params
            .get("offset")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);
        (limit, offset)
    };

    if method != Method::GET {
        let err = RestError::bad_request("Only GET is supported for REST API");
        let json = serde_json::to_string(&err).unwrap();
        return json_response(StatusCode::METHOD_NOT_ALLOWED, json);
    }

    let response_json = match path {
        "/api/v1/status" => {
            let bc = rpc_state.blockchain.lock().await;
            let mempool = rpc_state.mempool.lock().await;
            let state_lock = rpc_state.state.lock().await;
            let peer_count = rpc_state
                .p2p
                .shared_state()
                .peer_count
                .load(std::sync::atomic::Ordering::Relaxed);
            serde_json::json!(RestResponse {
                data: serde_json::json!({
                    "height": bc.height(),
                    "latest_block_hash": bc.last_block().hash,
                    "mempool_size": mempool.len(),
                    "peer_count": peer_count,
                    "total_staked": state_lock.total_staked(),
                }),
                total: None,
                limit: None,
                offset: None,
            })
        }

        p if p.starts_with("/api/v1/blocks/") && p != "/api/v1/blocks/" => {
            let height_str = &p["/api/v1/blocks/".len()..];
            match height_str.parse::<u64>() {
                Ok(height) => {
                    let bc = rpc_state.blockchain.lock().await;
                    match bc.get_block(height) {
                        Some(block) => serde_json::json!(RestResponse {
                            data: block,
                            total: None,
                            limit: None,
                            offset: None,
                        }),
                        None => {
                            let err = RestError::not_found(format!("Block {} not found", height));
                            let json = serde_json::to_string(&err).unwrap();
                            return json_response(StatusCode::NOT_FOUND, json);
                        }
                    }
                }
                Err(_) => {
                    let err = RestError::bad_request("Invalid block height");
                    let json = serde_json::to_string(&err).unwrap();
                    return json_response(StatusCode::BAD_REQUEST, json);
                }
            }
        }

        "/api/v1/blocks" => {
            let (limit, offset) = get_limit_offset(&params);
            let bc = rpc_state.blockchain.lock().await;
            let total = bc.height() + 1;
            let blocks = bc.blocks_paginated(offset, limit as usize);
            serde_json::json!(RestResponse {
                data: blocks,
                total: Some(total),
                limit: Some(limit),
                offset: Some(offset),
            })
        }

        p if p.starts_with("/api/v1/transactions/") => {
            let tx_hash = &p["/api/v1/transactions/".len()..];
            let tx_index = rpc_state.tx_index.lock().await;
            match tx_index.get(tx_hash) {
                Some(&(block_height, tx_idx)) => {
                    let bc = rpc_state.blockchain.lock().await;
                    match bc.get_block(block_height) {
                        Some(block) => {
                            if let Some(signed_tx) = block.transactions.get(tx_idx) {
                                serde_json::json!(RestResponse {
                                    data: serde_json::json!({
                                        "hash": signed_tx.tx_hash_hex(),
                                        "block_height": block_height,
                                        "block_hash": block.hash,
                                        "index_in_block": tx_idx,
                                        "sender": signed_tx.tx.sender,
                                        "receiver": signed_tx.tx.receiver,
                                        "amount": signed_tx.tx.amount,
                                        "fee": signed_tx.tx.fee,
                                        "nonce": signed_tx.tx.nonce,
                                        "timestamp": signed_tx.tx.timestamp,
                                        "tx_type": signed_tx.tx.tx_type,
                                        "memo": signed_tx.tx.memo,
                                    }),
                                    total: None,
                                    limit: None,
                                    offset: None,
                                })
                            } else {
                                let err = RestError::not_found("Transaction index out of bounds");
                                let json = serde_json::to_string(&err).unwrap();
                                return json_response(StatusCode::NOT_FOUND, json);
                            }
                        }
                        None => {
                            let err = RestError::not_found("Block not found for transaction");
                            let json = serde_json::to_string(&err).unwrap();
                            return json_response(StatusCode::NOT_FOUND, json);
                        }
                    }
                }
                None => {
                    let err = RestError::not_found(format!("Transaction {} not found", tx_hash));
                    let json = serde_json::to_string(&err).unwrap();
                    return json_response(StatusCode::NOT_FOUND, json);
                }
            }
        }

        p if p.starts_with("/api/v1/accounts/") => {
            let remainder = &p["/api/v1/accounts/".len()..];
            // Check if it ends with /transactions
            if let Some(addr) = remainder.strip_suffix("/transactions") {
                let state_lock = rpc_state.state.lock().await;
                let balance = state_lock.get_balance(addr);
                let nonce = state_lock.get_nonce(addr);
                let staked = state_lock.get_staked_balance(addr);
                // Return account info for now; transaction history requires index
                serde_json::json!(RestResponse {
                    data: serde_json::json!({
                        "address": addr,
                        "balance": balance,
                        "nonce": nonce,
                        "staked_balance": staked,
                        "transactions": [],
                        "note": "Transaction history index not yet implemented"
                    }),
                    total: None,
                    limit: None,
                    offset: None,
                })
            } else {
                let state_lock = rpc_state.state.lock().await;
                serde_json::json!(RestResponse {
                    data: serde_json::json!({
                        "address": remainder,
                        "balance": state_lock.get_balance(remainder),
                        "nonce": state_lock.get_nonce(remainder),
                        "staked_balance": state_lock.get_staked_balance(remainder),
                    }),
                    total: None,
                    limit: None,
                    offset: None,
                })
            }
        }

        p if p.starts_with("/api/v1/proposals/") => {
            let id_str = &p["/api/v1/proposals/".len()..];
            match id_str.parse::<u64>() {
                Ok(id) => {
                    let state_lock = rpc_state.state.lock().await;
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    match state_lock.get_proposal(id) {
                        Some(proposal) => serde_json::json!(RestResponse {
                            data: ProposalInfo {
                                id: proposal.id,
                                proposer: proposal.proposer.clone(),
                                title: proposal.title.clone(),
                                description: proposal.description.clone(),
                                created_at: proposal.created_at,
                                expires_at: proposal.expires_at,
                                yes_votes: proposal.yes_votes,
                                no_votes: proposal.no_votes,
                                status: state_lock.proposal_status(proposal, now),
                                voter_count: proposal.voters.len(),
                            },
                            total: None,
                            limit: None,
                            offset: None,
                        }),
                        None => {
                            let err = RestError::not_found(format!("Proposal {} not found", id));
                            let json = serde_json::to_string(&err).unwrap();
                            return json_response(StatusCode::NOT_FOUND, json);
                        }
                    }
                }
                Err(_) => {
                    let err = RestError::bad_request("Invalid proposal ID");
                    let json = serde_json::to_string(&err).unwrap();
                    return json_response(StatusCode::BAD_REQUEST, json);
                }
            }
        }

        "/api/v1/proposals" => {
            let state_lock = rpc_state.state.lock().await;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let proposals: Vec<ProposalInfo> = state_lock
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
                    status: state_lock.proposal_status(proposal, now),
                    voter_count: proposal.voters.len(),
                })
                .collect();
            serde_json::json!(RestResponse {
                data: proposals,
                total: None,
                limit: None,
                offset: None,
            })
        }

        "/api/v1/staking" => {
            let state_lock = rpc_state.state.lock().await;
            serde_json::json!(RestResponse {
                data: serde_json::json!({
                    "total_staked": state_lock.total_staked(),
                    "stake_map": state_lock.get_stake_map(),
                }),
                total: None,
                limit: None,
                offset: None,
            })
        }

        "/api/v1/chain-params" => {
            let state_lock = rpc_state.state.lock().await;
            let params = &state_lock.chain_params;
            serde_json::json!(RestResponse {
                data: serde_json::json!({
                    "block_reward": params.block_reward,
                    "block_interval_secs": params.block_interval_secs,
                    "max_txs_per_block": params.max_txs_per_block,
                    "stake_weight": params.stake_weight,
                    "proposal_quorum_bps": params.proposal_quorum_bps,
                    "proposal_approval_bps": params.proposal_approval_bps,
                    "min_proposal_stake": params.min_proposal_stake,
                }),
                total: None,
                limit: None,
                offset: None,
            })
        }

        "/api/v1/tokens" => {
            let state_lock = rpc_state.state.lock().await;
            let tokens: Vec<_> = state_lock.token_registry.tokens.values().collect();
            serde_json::json!(RestResponse {
                data: tokens,
                total: None,
                limit: None,
                offset: None,
            })
        }

        p if p.starts_with("/api/v1/tokens/") => {
            let remainder = &p["/api/v1/tokens/".len()..];
            // Try to parse "TOKEN_ID/balances/ADDRESS"
            if let Some((token_id, addr_part)) = remainder.split_once("/balances/") {
                let state_lock = rpc_state.state.lock().await;
                let balance = state_lock
                    .token_registry
                    .get_token_balance(addr_part, token_id);
                serde_json::json!(RestResponse {
                    data: serde_json::json!({
                        "address": addr_part,
                        "token_id": token_id,
                        "balance": balance,
                    }),
                    total: None,
                    limit: None,
                    offset: None,
                })
            } else {
                // Just a token ID
                let state_lock = rpc_state.state.lock().await;
                match state_lock.token_registry.tokens.get(remainder) {
                    Some(token) => serde_json::json!(RestResponse {
                        data: token,
                        total: None,
                        limit: None,
                        offset: None,
                    }),
                    None => {
                        let err = RestError::not_found(format!("Token {} not found", remainder));
                        let json = serde_json::to_string(&err).unwrap();
                        return json_response(StatusCode::NOT_FOUND, json);
                    }
                }
            }
        }

        "/api/v1/nfts" => {
            let state_lock = rpc_state.state.lock().await;
            let nfts: Vec<_> = state_lock.token_registry.nfts.values().collect();
            serde_json::json!(RestResponse {
                data: nfts,
                total: None,
                limit: None,
                offset: None,
            })
        }

        p if p.starts_with("/api/v1/nfts/") => {
            let nft_id = &p["/api/v1/nfts/".len()..];
            let state_lock = rpc_state.state.lock().await;
            match state_lock.token_registry.nfts.get(nft_id) {
                Some(nft) => serde_json::json!(RestResponse {
                    data: nft,
                    total: None,
                    limit: None,
                    offset: None,
                }),
                None => {
                    let err = RestError::not_found(format!("NFT {} not found", nft_id));
                    let json = serde_json::to_string(&err).unwrap();
                    return json_response(StatusCode::NOT_FOUND, json);
                }
            }
        }

        _ => {
            let err = RestError::not_found("Endpoint not found");
            let json = serde_json::to_string(&err).unwrap();
            return json_response(StatusCode::NOT_FOUND, json);
        }
    };

    json_response(StatusCode::OK, response_json.to_string())
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
            let tx_json = serde_json::to_string(&signed_tx).unwrap_or(tx_json);
            rpc_state.p2p.publish_transaction(tx_json);

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

        RpcRequest::FaucetTokens { address } => {
            if address.is_empty() {
                return RpcResponse::error("Invalid address");
            }

            const FAUCET_AMOUNT: u64 = 10;
            const FAUCET_SOURCE: &str = "genesis_account";

            let mut state = rpc_state.state.lock().await;

            // Ensure source has enough balance
            let source_balance = state
                .accounts
                .get(FAUCET_SOURCE)
                .map(|a| a.balance)
                .unwrap_or(0);
            if source_balance < FAUCET_AMOUNT {
                return RpcResponse::error("Faucet is empty");
            }

            // Debit source
            if let Some(source) = state.accounts.get_mut(FAUCET_SOURCE) {
                source.balance -= FAUCET_AMOUNT;
            }

            // Credit recipient
            let recipient = state
                .accounts
                .entry(address.clone())
                .or_insert_with(|| crate::state::Account::new(0));
            recipient.balance += FAUCET_AMOUNT;

            // Generate a synthetic tx hash
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            let hash_input = format!("faucet:{}:{}:{}", address, FAUCET_AMOUNT, now);
            let hash_bytes = sha2::Sha256::digest(hash_input.as_bytes());
            let tx_hash = hex::encode(hash_bytes);

            RpcResponse::success(serde_json::json!({
                "tx_hash": tx_hash,
                "amount": FAUCET_AMOUNT,
                "recipient": address,
                "message": format!("{} NCN sent to {}", FAUCET_AMOUNT, address),
            }))
        }

        RpcRequest::GetContract { address } => {
            let state = rpc_state.state.lock().await;
            match state.contracts.get(&address) {
                Some(contract) => RpcResponse::success(contract),
                None => RpcResponse::error(format!("Contract {} not found", address)),
            }
        }

        RpcRequest::ListContracts => {
            let state = rpc_state.state.lock().await;
            let contracts: Vec<_> = state.contracts.values().collect();
            RpcResponse::success(contracts)
        }

        RpcRequest::GetToken { token_id } => {
            let state = rpc_state.state.lock().await;
            match state.token_registry.tokens.get(&token_id) {
                Some(token) => RpcResponse::success(token),
                None => RpcResponse::error(format!("Token {} not found", token_id)),
            }
        }

        RpcRequest::GetTokenBalance { address, token_id } => {
            let state = rpc_state.state.lock().await;
            let balance = state.token_registry.get_token_balance(&address, &token_id);
            RpcResponse::success(serde_json::json!({
                "address": address,
                "token_id": token_id,
                "balance": balance,
            }))
        }

        RpcRequest::ListTokens => {
            let state = rpc_state.state.lock().await;
            let tokens: Vec<_> = state.token_registry.tokens.values().collect();
            RpcResponse::success(tokens)
        }

        RpcRequest::GetNft { nft_id } => {
            let state = rpc_state.state.lock().await;
            match state.token_registry.nfts.get(&nft_id) {
                Some(nft) => RpcResponse::success(nft),
                None => RpcResponse::error(format!("NFT {} not found", nft_id)),
            }
        }

        RpcRequest::GetNftOwner { nft_id } => {
            let state = rpc_state.state.lock().await;
            match state.token_registry.nfts.get(&nft_id) {
                Some(nft) => RpcResponse::success(serde_json::json!({
                    "nft_id": nft_id,
                    "owner": nft.owner,
                })),
                None => RpcResponse::error(format!("NFT {} not found", nft_id)),
            }
        }
    }
}

/// HTTP request handler
async fn handle_request(
    rpc_state: Arc<RpcState>,
    req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let path = req.uri().path().to_string();
    let method = req.method().clone();
    let query = req.uri().query().map(|q| q.to_string());

    // Handle CORS preflight for all paths
    if method == Method::OPTIONS {
        return Ok(Response::builder()
            .status(StatusCode::NO_CONTENT)
            .header(ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"))
            .header(
                ACCESS_CONTROL_ALLOW_METHODS,
                HeaderValue::from_static("GET, POST, OPTIONS"),
            )
            .header(
                ACCESS_CONTROL_ALLOW_HEADERS,
                HeaderValue::from_static("content-type"),
            )
            .body(Full::new(Bytes::new()))
            .unwrap());
    }

    // Route REST API requests
    if path.starts_with("/api/v1/") {
        return Ok(handle_rest_request(rpc_state, &method, &path, query.as_deref()).await);
    }

    // Only accept POST to /rpc
    if method != Method::POST || path != "/rpc" {
        let response = RpcResponse::error("Not found. Use POST /rpc or GET /api/v1/*");
        let json = serde_json::to_string(&response).unwrap();
        return Ok(json_response(StatusCode::NOT_FOUND, json));
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
            return Ok(json_response(status, json));
        }
    };

    // Parse RPC request
    let rpc_request: RpcRequest = match serde_json::from_slice(&body_bytes) {
        Ok(req) => req,
        Err(e) => {
            let response = RpcResponse::error(format!("Invalid JSON: {}", e));
            let json = serde_json::to_string(&response).unwrap();
            return Ok(json_response(StatusCode::BAD_REQUEST, json));
        }
    };

    // Handle request
    let response = handle_rpc_request(rpc_state, rpc_request).await;
    let json = serde_json::to_string(&response).unwrap();

    Ok(json_response(StatusCode::OK, json))
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
