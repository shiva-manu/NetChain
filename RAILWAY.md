# Deploying NetChain on Railway

Railway builds the repository's `Dockerfile` automatically. A NetChain node needs four things from it:

| Need | Railway feature |
|---|---|
| Persistent chain data | A **volume** mounted at `/data` |
| HTTP API for the wallet and website (`/rpc`, `/api/v1`, `/health`, `/ws`, ...) | A **public domain** targeting port `8545` |
| Peer-to-peer traffic between nodes | **Private networking** (`nodeK.railway.internal`), plus a **TCP proxy** on port `30333` for nodes outside Railway |
| Node identity and network | **Variables**: `NETCHAIN_NODE_KEY`, `NETCHAIN_GENESIS`, `NETCHAIN_BOOTNODES` |

> Keep every validator service at **1 replica**. Two replicas share one key and would sign competing blocks.

## Option A: one node in five minutes

This runs a private development network where the node is the only validator and holds the faucet funds.

1. In Railway: **New Project → Deploy from GitHub repo →** your NetChain repo.
2. **Service → Settings → Volumes**: add a volume mounted at `/data`.
3. **Settings → Networking → Generate Domain**, target port `8545`.
4. Open `https://<your-domain>/health`.

On first start the node writes its key and a dev genesis into `/data` and keeps them across redeploys. Others can join it later; see [Joining from outside Railway](#joining-from-outside-railway).

## Option B: a 3-validator network

### 1. Generate keys and genesis (locally)

```bash
scripts/railway-setup.sh                       # 3 validators, 5-second blocks
# or: scripts/railway-setup.sh --validators 4 --chain-id netchain-testnet-1 --block-interval 5
git add config/genesis.json
git commit -m "Add testnet genesis"
git push
```

The script writes two things:
- `config/genesis.json` (public, so commit it).
- `.railway-secrets/node1.env` … `node3.env`. Each contains one validator's private key, so never commit them. The folder is gitignored.

### 2. Create the services

Create three services from the same GitHub repo and name them exactly **`node1`**, **`node2`**, **`node3`**. Nodes 2 and 3 find node 1 at `node1.railway.internal`.

For **each** service:

1. **Variables → Raw Editor**: paste the contents of the matching `.railway-secrets/nodeK.env`.
2. **Settings → Volumes**: add a volume mounted at `/data`.
3. **Settings → Deploy**: set the healthcheck path to `/health`.

For **node1** only:

4. **Settings → Networking → Generate Domain**, target port `8545`. This is your public API.
5. **Settings → Networking → TCP Proxy**, port `30333`, so validators outside Railway can connect. With the Railway CLI this is `railway tcp-proxy create --port 30333 --service node1`. The node detects the proxy address from `RAILWAY_TCP_PROXY_DOMAIN` and `RAILWAY_TCP_PROXY_PORT` and advertises it to peers.

### 3. Check it

```bash
curl https://<node1-domain>/health        # status, height, peers, validator count
curl https://<node1-domain>/validators    # stake, PoI, uptime per validator
cargo run --bin netchain-wallet -- --rpc https://<node1-domain> validators
cargo run --bin netchain-wallet -- --rpc https://<node1-domain> new --name alice
cargo run --bin netchain-wallet -- --rpc https://<node1-domain> faucet alice
```

Within a few minutes each validator should show 2 attestations: the other two nodes measure it over Railway's private network.

## Point the website at it

In the website's hosting (e.g. Vercel), set:

```
VITE_NETCHAIN_RPC_URL=https://<node1-domain>
VITE_NETCHAIN_MONITORING_URL=https://<node1-domain>
VITE_NETCHAIN_WS_URL=wss://<node1-domain>/ws
```

To keep `api.netchain.me`, add it as a custom domain on node1 in Railway and point its CNAME record at the Railway domain.

## Joining from outside Railway

Anyone can run a node that syncs from your network:

```bash
NETCHAIN_GENESIS_URL=https://<node1-domain>/genesis \
NETCHAIN_BOOTNODES=/dns4/<tcp-proxy-domain>/tcp/<tcp-proxy-port> \
cargo run --release --bin netchain
```

To become a validator:
1. Fund the node's address (`netchain id` shows it) using the faucet.
2. Import the key: `netchain-wallet import-key --name mynode --key-file data/node_key`.
3. Stake at least 100: `netchain-wallet --rpc https://<node1-domain> stake --from mynode --amount 100`.

## Operating

- **Upgrades:** pushing to the deployed branch redeploys. Chain data stays on the volume, and the node re-verifies it at startup before syncing.
- **Downtime:**
  - A validator that is down for a while is jailed, so the chain keeps full speed without it. It unjails itself automatically when it comes back.
  - If *every* validator is down, nobody is penalized and the chain resumes when they return.
- **Starting a new network:** delete the volumes, run `scripts/railway-setup.sh` again (delete `.railway-secrets/` first), push the new `config/genesis.json`, and paste the new variables. A node refuses to start on a volume from a different genesis, and the error says why.
- **Logs:** look for `NetChain node started`, `peer connected`, `produced block` and `submitted PoI attestation`.

## Variables reference

| Variable | Meaning |
|---|---|
| `NETCHAIN_NODE_KEY` | Validator key (hex). If unset, a key is created in `/data/node_key` and kept on the volume. |
| `NETCHAIN_GENESIS` | Path of the genesis file (`/app/config/genesis.json` in the image). |
| `NETCHAIN_GENESIS_JSON` / `NETCHAIN_GENESIS_URL` | Alternative genesis sources: inline JSON, or a node's `/genesis` URL. |
| `NETCHAIN_BOOTNODES` | Comma-separated multiaddrs, e.g. `/dns/node1.railway.internal/tcp/30333`. |
| `NETCHAIN_EXTERNAL_ADDRS` | Extra public addresses to advertise (the Railway TCP proxy is added automatically). |
| `PORT` | HTTP gateway port (default `8545`). |
| `NETCHAIN_FAUCET_ENABLED` | `false` on nodes without faucet funds. |
| `NETCHAIN_LOG_LEVEL` | `info` (default), `debug`, ... |

## Why there is no `railway.toml`

Railway's config-as-code files (`railway.toml` / `railway.json`) are deprecated and stop being read on 2026-12-01. Their replacement, `.railway/railway.ts`, is applied with `railway config apply` rather than on deploy. Everything NetChain needs is the auto-detected `Dockerfile` plus the dashboard settings above.
