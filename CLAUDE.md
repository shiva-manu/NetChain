# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Build
```bash
cargo build                              # Build all binaries
cargo build --bin netchain-wallet        # Build wallet binary only
```

### Test
```bash
cargo test                               # Run all tests
cargo test --all-targets                 # Match CI: includes all targets
cargo test -- --nocapture                # Run with debug output
```

### Lint (CI runs all three)
```bash
cargo fmt --all -- --check               # Formatting check
cargo check --all-targets                # Build check
cargo clippy -- -D clippy::pedantic      # Strict lints
```

### Website
```bash
cd website && npm install && npm run dev # Dev server
cd website && npm run build              # Production build
```

### Docker
```bash
docker compose up --build                # Exposes 30333 (P2P), 8545 (RPC), 9090 (monitoring)
```

## Architecture

### Dual Binary, Shared Library

The project produces two binaries (`netchain` node and `netchain-wallet` CLI) that share core logic through the library crate. The library uses `#[path = ...]` attributes in `src/lib.rs` to re-export modules from subdirectories under flat names — e.g., `netchain::block` maps to `src/chain/block.rs`, `netchain::consensus` maps to `src/poi/consensus.rs`. This means import paths don't match the filesystem layout.

### Module Map (lib.rs path → filesystem)

| Import path | File |
|---|---|
| `netchain::block` | `src/chain/block.rs` |
| `netchain::blockchain` | `src/chain/blockchain.rs` |
| `netchain::state` | `src/chain/state.rs` |
| `netchain::transaction` | `src/chain/transaction.rs` |
| `netchain::config` | `src/app/config.rs` |
| `netchain::consensus` | `src/poi/consensus.rs` |
| `netchain::measurement` | `src/poi/measurement.rs` |
| `netchain::metrics_aggregator` | `src/poi/metrics_aggregator.rs` |
| `netchain::anti_gaming` | `src/poi/anti_gaming.rs` |
| `netchain::epoch_manager` | `src/poi/epoch_manager.rs` |
| `netchain::metric_challenge` | `src/poi/metric_challenge.rs` |
| `netchain::p2p` | `src/net/p2p.rs` |
| `netchain::rpc` | `src/net/rpc.rs` |
| `netchain::websocket` | `src/net/websocket.rs` |
| `netchain::monitoring` | `src/net/monitoring.rs` |
| `netchain::dht` | `src/net/dht.rs` |
| `netchain::mempool` | `src/node/mempool.rs` |
| `netchain::producer` | `src/node/producer.rs` |
| `netchain::storage` | `src/node/storage.rs` |
| `netchain::wallet` | `src/wallet/mod.rs` |

### Hybrid Consensus (Proof of Internet)

Validator selection is a composite score blending multiple signals:

- **PoI metrics**: download speed, upload speed, latency, uptime, stability (measured via Cloudflare endpoints)
- **Stake weight**: economic weight (configurable 0.0–1.0 via `stake_weight`)
- **Identity & reputation**: long-running node stability rewards
- **Attestations**: peer-verified metric challenges (minimum 3 by default)
- **Slashing penalties**: persistent trust penalties for invalid blocks, metric fraud, or missed blocks

The `PoiScorer` in `src/poi/consensus.rs` computes the final validator score. `BlockProducer` in `src/node/producer.rs` uses it to select the next block producer.

### Key Data Flow

1. **Transactions** arrive via RPC (`src/net/rpc.rs`) → validated and stored in `Mempool`
2. **Block production** runs on a timer (`block_interval_secs`): `BlockProducer` selects validator via PoI scorer, drains mempool, produces block
3. **P2P gossip** (`src/net/p2p.rs` via libp2p/gossipsub) broadcasts blocks and transactions between nodes
4. **State transitions** applied atomically per block in `State` (`src/chain/state.rs`); invalid txs reject the whole block and slash the proposing validator
5. **Governance** proposals and votes are on-chain transactions; passed proposals update runtime parameters (block reward, interval, etc.) without restart
6. **Metric challenges**: peers challenge each other's bandwidth claims via P2P; responses are attested and stored in `MetricsAggregator`
7. **Epochs**: `EpochManager` tracks block production per epoch, applies reputation decay, and can slash validators who miss their epoch quota

### Persistent Storage

Uses `sled` embedded database. `Storage` (`src/node/storage.rs`) persists blocks and state. The `data/` directory (configurable via `DATA_DIR`) holds the sled database.

### Configuration

Loaded from `config/default.toml` by default. Override with `NETCHAIN_CONFIG` env var. All config sections have env var overrides documented in the README (e.g., `NETCHAIN_BLOCK_INTERVAL_SECS`, `NETCHAIN_STAKE_WEIGHT`).

### Website

React + Vite SPA under `website/`. Pages are route-based: `home`, `dashboard`, `features`, `technology`, `docs`, `get-started`, `faucet`. The dashboard shows live consensus telemetry. Uses a custom `netchain-client.ts` to talk to the node's RPC/WebSocket interfaces.
