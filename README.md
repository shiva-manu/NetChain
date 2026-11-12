# NetChain

NetChain is an experimental Layer-1 blockchain in Rust built around a hybrid **Proof of Internet** consensus: the chance that a validator proposes the next block depends on its stake, its on-chain uptime, and network performance (latency and throughput) that **other validators measure and record on-chain**.

Nodes run as a real peer-to-peer network: blocks are signed, every node independently verifies who was allowed to propose each block, nodes discover each other across the internet, and a node that restarts re-verifies its chain from disk and catches up from peers.

## How consensus works

- **Slots.** Time is split into slots of `block_interval_secs`. Exactly one validator may propose in each slot. It is chosen deterministically from the parent block hash, so every node agrees on who it is without communicating. If that validator is offline the slot stays empty and the next slot has a different proposer.
- **Validators.** Any address with at least `min_validator_stake` bonded (via a `Stake` transaction) is a validator. Each node has a persistent key (`<data_dir>/node_key`) that is its validator address, signs its blocks, and is its libp2p identity.
- **Selection weight.** All scores are in basis points and computed with integer math, so every node gets identical results:
  ```text
  trust  = 40% PoI + 40% uptime + 20% (1 - slashing penalty)
  weight = (1 - stake_weight) * trust + stake_weight * stake / max_stake
  ```
- **Proof of Internet.**
  - Every validator node pings its validator peers continuously and periodically downloads a probe payload from each of them.
  - It records the measured round-trip time and throughput on-chain with an `AttestMetrics` transaction.
  - A validator's PoI score is the median of recent attestations from *other* active validators. It cannot measure itself, and a single dishonest attester cannot move the median.
  - With fewer than `min_attestations` recent attestations the score stays at a neutral 50%.
- **Uptime and jailing.** Producing a block in your slot raises your on-chain uptime and missing your slot lowers it. Below 50% uptime a validator is **jailed**: it gets no slots, so an offline validator stops slowing the chain down. A node that comes back online submits `Unjail` automatically. The last active validator is never jailed.
- **Fork choice.** The longest chain of valid blocks wins. Every block is checked for its proposer signature, slot schedule and timestamp, then all its transactions are executed. A node switches to a longer fork only after validating it completely.
- **Genesis.** All starting conditions (balances, validator stakes, chain parameters, time) live in one genesis JSON file. Its hash is committed in block 0, so nodes with different genesis files never mix.

All of these rules live in `src/chain/engine.rs`.

## Project layout

```text
src/
├── main.rs              # Node binary: `netchain run | id | genesis`
├── bin/wallet.rs        # Wallet CLI
├── chain/
│   ├── engine.rs        # Consensus rules (slots, proposer selection, PoI, uptime, jailing)
│   ├── genesis.rs       # Genesis spec (JSON) -> genesis block + state
│   ├── block.rs         # Signed blocks, Merkle root
│   ├── blockchain.rs    # Block list + stateless block checks
│   ├── state.rs         # Accounts, staking, governance, tokens, attestations, validator stats
│   └── transaction.rs   # Signed transactions
├── node/
│   ├── chain_service.rs # Block import, sync, fork choice/reorg, block production, persistence
│   ├── node_key.rs      # Persistent node identity
│   ├── mempool.rs       # Fee-ordered mempool (several queued txs per sender)
│   └── storage.rs       # sled block store
├── net/
│   ├── p2p.rs           # libp2p: gossipsub, request-response sync/probes, ping, identify+Kademlia, mDNS
│   ├── rpc.rs           # JSON-RPC + REST API, faucet
│   ├── websocket.rs     # Live events
│   └── monitoring.rs    # /health, /metrics, /validators
├── poi/                 # Speed-test measurement (informational) and earlier off-chain PoI experiments
└── wallet/              # Encrypted keystore
```

## Quick start: a single-node dev chain

```bash
cargo run --bin netchain
```

With no genesis configured, the node creates `./data/genesis.json`: a private network where this node is the only validator and holds 1,000,000 NCN for its faucet.

```bash
cargo run --bin netchain-wallet -- new --name alice
cargo run --bin netchain-wallet -- faucet alice
cargo run --bin netchain-wallet -- balance alice
cargo run --bin netchain-wallet -- send --from alice --to <address> --amount 10
cargo run --bin netchain-wallet -- validators
```

## A local 3-validator network

```bash
scripts/local-testnet.sh start    # build, write genesis, start 3 nodes
scripts/local-testnet.sh status   # height / tip / peers per node
scripts/local-testnet.sh stop
scripts/local-testnet.sh reset    # delete .testnet/
```

Node N serves RPC on `127.0.0.1:860N`. Nodes 2 and 3 only know node 1's address and discover each other through the DHT. Try stopping a node: the chain keeps going, the stopped validator is jailed after a few missed slots, and when you restart it, it catches up and unjails itself.

## Running a network across machines

1. On each validator machine, create the node key and note the address:
   ```bash
   netchain id
   # address: 3f2c...   peer_id: 12D3KooW...
   ```
2. On one machine, write the genesis file with every validator's address and stake, plus a funded faucet account, then copy the same file to every node:
   ```bash
   netchain genesis --chain-id netchain-testnet-1 \
     --validator <ADDR_1>:1000 --validator <ADDR_2>:1000 --validator <ADDR_3>:1000 \
     --alloc <ADDR_1>:1000000 --block-interval 5 --out genesis.json
   ```
3. Start every node with the same genesis and at least one bootnode:
   ```bash
   NETCHAIN_GENESIS=./genesis.json \
   NETCHAIN_BOOTNODES=/ip4/<NODE_1_PUBLIC_IP>/tcp/30333 \
   netchain run
   ```
   Open TCP port 30333 (P2P) on every node. RPC (8545), WebSocket (8546) and monitoring (9090) bind to localhost unless you set `NETCHAIN_RPC_BIND_ADDR=0.0.0.0` (and the matching variables for WebSocket and monitoring).

New validators can join later without a new genesis:
1. Start a node with the network's genesis file and a bootnode.
2. Fund its address (`netchain id` shows it) from the faucet or a transfer.
3. Import its key with `netchain-wallet import-key --name mynode --key-file <data_dir>/node_key`.
4. Stake at least `min_validator_stake` (100 by default) with `netchain-wallet stake --from mynode --amount 100`.

## Configuration

Loaded from `config/default.toml` (override the path with `NETCHAIN_CONFIG`). Environment overrides:

| Variable | Meaning |
|---|---|
| `DATA_DIR` | Data directory (node key, genesis for dev chains, block database) |
| `NETCHAIN_GENESIS` | Genesis file of the network to join |
| `NETCHAIN_BOOTNODES` | Comma-separated multiaddrs, e.g. `/ip4/1.2.3.4/tcp/30333` |
| `NETCHAIN_ENABLE_MDNS` | Local-network discovery (`true`/`false`) |
| `PORT`, `RPC_PORT`, `NETCHAIN_WS_PORT`, `NETCHAIN_MONITORING_PORT` | Ports |
| `NETCHAIN_RPC_BIND_ADDR`, `NETCHAIN_WS_BIND_ADDR`, `NETCHAIN_MONITORING_BIND_ADDR` | Bind addresses |
| `NETCHAIN_FAUCET_ENABLED`, `NETCHAIN_FAUCET_KEY_FILE`, `NETCHAIN_FAUCET_AMOUNT` | Faucet |
| `NETCHAIN_ATTEST_INTERVAL_SECS` | How often validators measure each other |
| `NETCHAIN_BLOCK_INTERVAL_SECS`, `NETCHAIN_BLOCK_REWARD`, `NETCHAIN_MAX_TXS_PER_BLOCK`, `NETCHAIN_STAKE_WEIGHT` | Chain parameters for a newly created **dev** genesis only (a shared network takes them from its genesis file) |
| `NETCHAIN_LOG_LEVEL` | Log level |

## Interfaces

- **P2P** `0.0.0.0:30333`
- **RPC** `127.0.0.1:8545`: `POST /rpc` with `{"method": ..., "params": {...}}`
  - Methods: `get_balance`, `get_nonce` (next usable nonce, including pending transactions), `send_transaction`, `get_chain_info`, `get_mempool_size`, `get_block`, `get_blocks`, `get_account`, `get_staking_info`, `get_proposals`, `get_proposal`, `get_chain_params`, `get_validators`, `get_node_info`, `faucet_tokens`, plus contract and token queries.
  - REST reads: `GET /api/v1/{status,blocks,transactions,accounts,proposals,staking,chain-params,tokens,nfts}`.
- **WebSocket** `ws://127.0.0.1:8546`: subscribe with `{"action":"subscribe","topics":["new_blocks","new_transactions","proposals","slashing"]}`.
- **Monitoring** `127.0.0.1:9090`:
  - `/health`: height, peers, validator counts, whether this node is a validator, and age of the latest block.
  - `/validators`: per-validator stake, PoI, uptime, attestations and jailed status.
  - `/metrics`: Prometheus format.

Native transactions: `Transfer`, `Stake`, `Unstake`, `CreateProposal`, `VoteProposal`, `AttestMetrics`, `Unjail`, token and NFT operations, contract deploy/call.

Governance: a passed proposal can change the block reward, block interval, max transactions per block and stake weight. Proposals execute deterministically in id order.

## Docker

```bash
docker compose up --build
```

## Development

```bash
cargo test --all-targets        # unit + multi-node chain-service tests
cargo fmt --all -- --check
cargo clippy --all-targets
```

## Known limitations

This is a testnet-grade system, not production money infrastructure:

- **Finality is probabilistic** (longest chain). There is no BFT finality gadget yet, so a deep reorg is possible in principle.
- **Smart contract calls are not executed yet.** `CallContract` charges gas but does not run the WASM VM.
- **Equivocation is not slashed.** Double-signing in one slot is not punished. Signed blocks make evidence possible, but there is no evidence transaction yet.
- **Colluding attesters can skew PoI.** A colluding majority of validators can inflate each other's scores; the median only protects against a minority.
- **Replay is the only state store.** State is rebuilt by replaying blocks at startup and during a reorg. That is simple and robust, but slow for very long chains; state snapshots would fix it.
- **No external audit.** The code has not been audited.

## License

See `LICENSE`.
