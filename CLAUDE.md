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
docker compose up --build                # Exposes 30333 (P2P), 8545 (HTTP gateway)
```

## Architecture

### Dual Binary, Shared Library

The project produces two binaries (`netchain` node and `netchain-wallet` CLI) that share core logic through the library crate. The library uses `#[path = ...]` attributes in `src/lib.rs` to re-export modules from subdirectories under flat names — e.g., `netchain::block` maps to `src/chain/block.rs`, `netchain::chain_service` maps to `src/node/chain_service.rs`. This means import paths don't match the filesystem layout.

### Module Map (lib.rs path → filesystem)

| Import path | File |
|---|---|
| `netchain::block` | `src/chain/block.rs` |
| `netchain::blockchain` | `src/chain/blockchain.rs` |
| `netchain::engine` | `src/chain/engine.rs` |
| `netchain::vm` | `src/chain/vm.rs` |
| `netchain::contract` | `src/chain/contract.rs` |
| `netchain::token` | `src/chain/token.rs` |
| `netchain::genesis` | `src/chain/genesis.rs` |
| `netchain::state` | `src/chain/state.rs` |
| `netchain::transaction` | `src/chain/transaction.rs` |
| `netchain::config` | `src/app/config.rs` |
| `netchain::p2p` | `src/net/p2p.rs` |
| `netchain::http` | `src/net/http.rs` |
| `netchain::rpc` | `src/net/rpc.rs` |
| `netchain::websocket` | `src/net/websocket.rs` |
| `netchain::monitoring` | `src/net/monitoring.rs` |
| `netchain::mempool` | `src/node/mempool.rs` |
| `netchain::chain_service` | `src/node/chain_service.rs` |
| `netchain::node_key` | `src/node/node_key.rs` |
| `netchain::storage` | `src/node/storage.rs` |
| `netchain::wallet` | `src/wallet/mod.rs` |

### Hybrid Consensus (Proof of Internet)

`src/chain/engine.rs` is the single source of truth for consensus. Everything there is a pure, integer-only function of the parent block and the post-parent state, so all nodes agree:

- **Slots**: `slot = timestamp / block_interval_secs`; each block must be in a later slot than its parent and no more than 15s in the future.
- **Proposer**: weighted pick over active validators (stake >= `min_validator_stake`, not jailed), seeded by `sha256(parent_hash, slot)`. Weight blends trust (40% PoI, 40% uptime, 20% slashing) with normalized stake via `stake_weight`.
- **PoI**: median of recent `AttestMetrics` txs (RTT + throughput measured peer-to-peer by other validators) stored in `State::metric_attestations`.
- **Uptime/jailing**: `State::validator_stats` EWMA updated per block (produced / missed slots); uptime < 50% jails the validator until it sends `Unjail` (nodes do this automatically).
- **Blocks** are signed by the proposer's node key; `validator` is its address (`hex(sha256(pubkey)[..20])`).

Smart contracts: `CallContract` runs the WASM module through `vm::execute` (wasmi fuel + host-function gas). Failed calls charge the full gas limit and change nothing else. Contract storage lives in `State::contract_storage` (the code is under the protected `__code` key).

### Key Data Flow

1. **Transactions** arrive via RPC or gossip → validated against state + the sender's already-queued txs → `Mempool`
2. **Block production**: every 500ms the node asks `ChainService::try_produce`; if its key owns the current slot it builds, applies and gossips a signed block
3. **Import**: gossiped blocks extending the tip go through `engine::import_block`; anything else triggers a direct request-response sync. `ChainService::import_sync_blocks` extends, buffers a competing fork, or reorgs to a strictly longer fully-validated fork
4. **State** is never persisted separately: startup replays and re-verifies all stored blocks from the genesis spec
5. **Networking** (`src/net/p2p.rs`): gossipsub (topics namespaced by genesis hash), request-response JSON (sync, bandwidth probes), ping (RTT), identify + Kademlia (peer discovery from bootnodes), DNS multiaddrs, optional mDNS. All client-facing HTTP (RPC, REST, monitoring, `/genesis`, WebSocket `/ws`) is served on one port by `src/net/http.rs`
6. **Identity**: `<data_dir>/node_key` is the validator key, block-signing key and libp2p identity; `node_key::address_from_peer_id` maps peers to on-chain addresses
7. **Governance** proposals execute in proposal-id order at block time

### Persistent Storage

Uses `sled` embedded database under `<data_dir>/db`. `Storage` persists blocks plus the network's genesis hash (a data dir from another network or an old unsigned-block version is refused at startup). `<data_dir>` also holds `node_key` and, for dev chains, `genesis.json`.

### Configuration

Loaded from `config/default.toml` by default. Override with `NETCHAIN_CONFIG` env var. Env overrides are documented in the README. Consensus parameters come from the genesis file (`NETCHAIN_GENESIS`); without one the node creates a single-validator dev genesis from the `[producer]` values. `scripts/local-testnet.sh` runs a 3-validator network locally. `PORT` is the HTTP gateway port and `NETCHAIN_P2P_PORT` the libp2p port.

### Deployment

Railway is the deployment target (see `RAILWAY.md`): the root `Dockerfile` is auto-built, each validator is one service with a `/data` volume and a `NETCHAIN_NODE_KEY` variable, all load the committed `config/genesis.json` (generated by `scripts/railway-setup.sh`), and node1 exposes the public domain (port 8545) plus a TCP proxy (30333). There is intentionally no `railway.toml` (deprecated by Railway as of 2026-12-01). The `terraform/`, `deploy/` and AWS docs are legacy.

### Website

React + Vite SPA under `website/`. Pages are route-based: `home`, `dashboard`, `features`, `technology`, `docs`, `get-started`, `faucet`. The dashboard shows live consensus telemetry. Uses a custom `netchain-client.ts` to talk to the node's RPC/WebSocket interfaces.

<!-- code-review-graph MCP tools -->
## MCP Tools: code-review-graph

**IMPORTANT: This project has a knowledge graph. ALWAYS use the
code-review-graph MCP tools BEFORE using Grep/Glob/Read to explore
the codebase.** The graph is faster, cheaper (fewer tokens), and gives
you structural context (callers, dependents, test coverage) that file
scanning cannot.

### When to use graph tools FIRST

- **Exploring code**: `semantic_search_nodes` or `query_graph` instead of Grep
- **Understanding impact**: `get_impact_radius` instead of manually tracing imports
- **Code review**: `detect_changes` + `get_review_context` instead of reading entire files
- **Finding relationships**: `query_graph` with callers_of/callees_of/imports_of/tests_for
- **Architecture questions**: `get_architecture_overview` + `list_communities`

Fall back to Grep/Glob/Read **only** when the graph doesn't cover what you need.

### Key Tools

| Tool | Use when |
|------|----------|
| `detect_changes` | Reviewing code changes — gives risk-scored analysis |
| `get_review_context` | Need source snippets for review — token-efficient |
| `get_impact_radius` | Understanding blast radius of a change |
| `get_affected_flows` | Finding which execution paths are impacted |
| `query_graph` | Tracing callers, callees, imports, tests, dependencies |
| `semantic_search_nodes` | Finding functions/classes by name or keyword |
| `get_architecture_overview` | Understanding high-level codebase structure |
| `refactor_tool` | Planning renames, finding dead code |

### Workflow

1. The graph auto-updates on file changes (via hooks).
2. Use `detect_changes` for code review.
3. Use `get_affected_flows` to understand impact.
4. Use `query_graph` pattern="tests_for" to check coverage.
