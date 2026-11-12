//! NetChain library crate.
//!
//! The binaries (`src/main.rs` and `src/bin/wallet.rs`) share core logic through this crate to
//! prevent type drift and reduce duplicated security-critical code.

#[path = "chain/block.rs"]
pub mod block;
#[path = "chain/blockchain.rs"]
pub mod blockchain;
#[path = "node/chain_service.rs"]
pub mod chain_service;
#[path = "app/config.rs"]
pub mod config;
#[path = "chain/contract.rs"]
pub mod contract;
#[path = "chain/engine.rs"]
pub mod engine;
#[path = "chain/genesis.rs"]
pub mod genesis;
#[path = "net/http.rs"]
pub mod http;
#[path = "node/mempool.rs"]
pub mod mempool;
#[path = "net/monitoring.rs"]
pub mod monitoring;
#[path = "node/node_key.rs"]
pub mod node_key;
#[path = "net/p2p.rs"]
pub mod p2p;
#[path = "net/rest_types.rs"]
pub mod rest_types;
#[path = "net/rpc.rs"]
pub mod rpc;
#[path = "net/rpc_types.rs"]
pub mod rpc_types;
#[path = "chain/state.rs"]
pub mod state;
#[path = "node/storage.rs"]
pub mod storage;
#[path = "chain/token.rs"]
pub mod token;
#[path = "chain/transaction.rs"]
pub mod transaction;
#[path = "chain/vm.rs"]
pub mod vm;
#[path = "wallet/mod.rs"]
pub mod wallet;
#[path = "net/websocket.rs"]
pub mod websocket;
