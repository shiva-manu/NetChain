//! Shared RPC request/response types.
//!
//! These types are used by both the node RPC server (`src/rpc.rs`) and the wallet CLI
//! (`src/bin/wallet.rs`) to avoid JSON shape drift.

use serde::{Deserialize, Serialize};

/// RPC Request types
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum RpcRequest {
    #[serde(rename = "get_balance")]
    GetBalance { address: String },
    #[serde(rename = "get_nonce")]
    GetNonce { address: String },
    #[serde(rename = "send_transaction")]
    SendTransaction { tx_json: String },
    #[serde(rename = "get_chain_info")]
    GetChainInfo,
    #[serde(rename = "get_mempool_size")]
    GetMempoolSize,
    #[serde(rename = "get_block")]
    GetBlock { index: u64 },
    #[serde(rename = "get_blocks")]
    GetBlocks { start_height: u64, limit: usize },
    #[serde(rename = "get_account")]
    GetAccount { address: String },
    #[serde(rename = "get_staking_info")]
    GetStakingInfo { address: String },
    #[serde(rename = "get_proposals")]
    GetProposals,
    #[serde(rename = "get_proposal")]
    GetProposal { proposal_id: u64 },
    #[serde(rename = "get_chain_params")]
    GetChainParams,
    #[serde(rename = "faucet_tokens")]
    FaucetTokens { address: String },
    #[serde(rename = "get_contract")]
    GetContract { address: String },
    #[serde(rename = "list_contracts")]
    ListContracts,
    #[serde(rename = "get_token")]
    GetToken { token_id: String },
    #[serde(rename = "get_token_balance")]
    GetTokenBalance { address: String, token_id: String },
    #[serde(rename = "list_tokens")]
    ListTokens,
    #[serde(rename = "get_nft")]
    GetNft { nft_id: String },
    #[serde(rename = "get_nft_owner")]
    GetNftOwner { nft_id: String },
}

/// RPC Response types
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum RpcResponse {
    #[serde(rename = "success")]
    Success { data: serde_json::Value },
    #[serde(rename = "error")]
    Error { message: String },
}

impl RpcResponse {
    pub fn success<T: Serialize>(data: T) -> Self {
        RpcResponse::Success {
            data: serde_json::to_value(data).unwrap_or(serde_json::Value::Null),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        RpcResponse::Error {
            message: message.into(),
        }
    }
}
