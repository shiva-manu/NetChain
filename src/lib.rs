//! NetChain library crate.
//!
//! The binaries (`src/main.rs` and `src/bin/wallet.rs`) share core logic through this crate to
//! prevent type drift and reduce duplicated security-critical code.

#[path = "poi/anti_gaming.rs"]
pub mod anti_gaming;
#[path = "chain/block.rs"]
pub mod block;
#[path = "chain/blockchain.rs"]
pub mod blockchain;
#[path = "app/config.rs"]
pub mod config;
#[path = "poi/consensus.rs"]
pub mod consensus;
#[path = "poi/measurement.rs"]
pub mod measurement;
#[path = "node/mempool.rs"]
pub mod mempool;
#[path = "poi/metrics_aggregator.rs"]
pub mod metrics_aggregator;
#[path = "net/monitoring.rs"]
pub mod monitoring;
#[path = "net/p2p.rs"]
pub mod p2p;
#[path = "node/producer.rs"]
pub mod producer;
#[path = "net/rpc.rs"]
pub mod rpc;
#[path = "net/rpc_types.rs"]
pub mod rpc_types;
#[path = "chain/state.rs"]
pub mod state;
#[path = "node/storage.rs"]
pub mod storage;
#[path = "chain/transaction.rs"]
pub mod transaction;
#[path = "wallet/mod.rs"]
pub mod wallet;
#[path = "net/websocket.rs"]
pub mod websocket;
