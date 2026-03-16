# NetChain

NetChain is an experimental Layer-1 blockchain prototype in Rust built around Proof of Internet (PoI), where validator selection is influenced by measured network performance instead of hashpower or stake alone.

## Highlights

- Rust blockchain prototype with blocks, chain validation, state, wallet, RPC, and libp2p networking
- Proof of Internet scoring based on download speed, upload speed, latency, uptime, and stability
- Peer metric announcements, challenge-response attestations, and anti-gaming validation
- Native staking and governance transaction types with on-chain proposal voting
- Explorer-style RPC reads for blocks, accounts, staking positions, and proposals
- Persistent local storage with sled
- Runtime config file support, structured logging, health checks, and metrics endpoint

## Project Layout

```text
netchain/
├── config/default.toml      # Runtime configuration
├── docker-compose.yml       # Local container orchestration
├── Dockerfile               # Container image build
├── src/
│   ├── main.rs              # Node entry point (bin)
│   ├── lib.rs               # Shared library crate
│   ├── bin/wallet.rs        # Wallet CLI (bin)
│   ├── app/config.rs        # Runtime configuration loader
│   ├── chain/               # Blocks, state, transactions
│   ├── net/                 # P2P, RPC, WebSocket, monitoring
│   ├── node/                # Mempool, producer, storage
│   ├── poi/                 # Proof of Internet + anti-gaming
│   └── wallet/              # Wallet types + crypto helpers
└── .github/workflows/ci.yml # CI pipeline
```

## Quick Start

Prerequisites:

- Rust toolchain

Build:

```bash
cargo build
```

Run the node:

```bash
cargo run --bin netchain
```

Run tests:

```bash
cargo test
```

## Configuration

NetChain loads config from `config/default.toml` by default.

Override the config path:

```bash
NETCHAIN_CONFIG=./config/default.toml cargo run --bin netchain
```

Useful env overrides:

- `DATA_DIR`
- `PORT`
- `RPC_PORT`
- `NETCHAIN_RPC_BIND_ADDR`
- `NETCHAIN_MONITORING_PORT`
- `NETCHAIN_MONITORING_BIND_ADDR`
- `NETCHAIN_WS_PORT`
- `NETCHAIN_WS_BIND_ADDR`
- `NETCHAIN_BLOCK_INTERVAL_SECS`
- `NETCHAIN_BLOCK_REWARD`
- `NETCHAIN_MAX_TXS_PER_BLOCK`
- `NETCHAIN_STAKE_WEIGHT`
- `NETCHAIN_LOG_LEVEL`

## Interfaces

- P2P: `0.0.0.0:30333` by default for node gossip and discovery
- RPC: `127.0.0.1:8545` by default
- Monitoring: `127.0.0.1:9090` by default
- WebSocket: `127.0.0.1:8546` by default

Native transaction types:

- `Transfer`
- `Stake`
- `Unstake`
- `CreateProposal`
- `VoteProposal`

Governance notes:

- Proposal creation requires an existing stake position
- Vote weight equals the voter's currently staked balance
- Proposal status is `Active`, `Passed`, or `Rejected` based on deadline, quorum, and yes-vote approval share
- Passed proposals can update runtime chain parameters without restarting the node
- Supported proposal actions: `ChangeBlockReward`, `ChangeBlockInterval`, `ChangeMaxTxsPerBlock`, `ChangeStakeWeight`
- Proposal actions are validated before entering the mempool

Runtime governance defaults:

- Minimum proposal stake: `100`
- Quorum: `20%` of total staked balance
- Approval threshold: `50.01%` yes votes among participating stake

WebSocket protocol:

- Connect to `ws://127.0.0.1:8546`
- Subscribe with `{"action":"subscribe","topics":["new_blocks","new_transactions","proposals"]}`
- Unsubscribe with `{"action":"unsubscribe","topics":["proposals"]}`
- Ping with `{"action":"ping"}`

Example using `wscat`:

```bash
npx wscat -c ws://127.0.0.1:8546
> {"action":"subscribe","topics":["new_blocks","proposals"]}
```

Example WebSocket event:

```json
{"event":"proposal_update","data":{"proposal_id":1,"title":"Change reward","status":"Passed","yes_votes":500,"no_votes":0}}
```

Monitoring endpoints:

- `GET /health` returns JSON health status
- `GET /metrics` returns Prometheus-style plaintext metrics

Example:

```bash
curl http://127.0.0.1:9090/health
curl http://127.0.0.1:9090/metrics
```

RPC methods:

- `get_balance`
- `get_nonce`
- `send_transaction`
- `get_chain_info`
- `get_mempool_size`
- `get_block`
- `get_blocks`
- `get_account`
- `get_staking_info`
- `get_proposals`
- `get_proposal`

## Docker

Build and run with Docker Compose:

```bash
docker compose up --build
```

The compose file exposes:

- `30333` for P2P
- `8545` for RPC
- `9090` for monitoring

## CI

GitHub Actions runs:

- `cargo fmt --all -- --check`
- `cargo check --all-targets`
- `cargo test --all-targets`

## Development Notes

- Logs are emitted with `tracing`; use `NETCHAIN_LOG_LEVEL=debug` for more detail.
- Runtime metrics and health are intentionally lightweight for local development and early testnet-style deployments.
- The wallet binary is available as `netchain-wallet`.

## License

See `LICENSE`.
