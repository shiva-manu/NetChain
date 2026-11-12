#!/usr/bin/env bash
# Prepare a NetChain network for Railway.
#
#   scripts/railway-setup.sh [--validators N] [--chain-id NAME] [--block-interval SECS]
#
# Creates:
#   config/genesis.json             public genesis (commit and push it; every node loads it)
#   .railway-secrets/nodeK.env      variables for Railway service "nodeK" (contains its
#                                   private key: never commit, paste into Railway only)
#
# Service node1 is the bootnode, holds the faucet funds and serves the public API.
# See RAILWAY.md for the deployment steps.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VALIDATORS=3
CHAIN_ID="netchain-testnet-1"
BLOCK_INTERVAL=5
STAKE=1000
FAUCET_FUNDS=1000000

while [[ $# -gt 0 ]]; do
  case "$1" in
    --validators) VALIDATORS="$2"; shift 2 ;;
    --chain-id) CHAIN_ID="$2"; shift 2 ;;
    --block-interval) BLOCK_INTERVAL="$2"; shift 2 ;;
    *) sed -n '2,13p' "$0"; exit 1 ;;
  esac
done

export PATH="$HOME/.cargo/bin:$PATH"
(cd "$ROOT" && cargo build --quiet --bin netchain)
BIN="$ROOT/target/debug/netchain"
SECRETS="$ROOT/.railway-secrets"

if [[ -d "$SECRETS" ]]; then
  echo "$SECRETS already exists. Delete it first to create a new network." >&2
  exit 1
fi
mkdir -p "$SECRETS"
chmod 700 "$SECRETS"

args=()
for k in $(seq 1 "$VALIDATORS"); do
  out=$("$BIN" keygen 2>/dev/null)
  secret=$(awk '/^secret:/ {print $2}' <<<"$out")
  address=$(awk '/^address:/ {print $2}' <<<"$out")
  args+=(--validator "$address:$STAKE")
  [[ $k == 1 ]] && args+=(--alloc "$address:$FAUCET_FUNDS")

  {
    echo "NETCHAIN_NODE_KEY=$secret"
    echo "NETCHAIN_GENESIS=/app/config/genesis.json"
    echo "PORT=8545"
    if [[ $k != 1 ]]; then
      echo "NETCHAIN_BOOTNODES=/dns/node1.railway.internal/tcp/30333"
      echo "NETCHAIN_FAUCET_ENABLED=false"
    fi
  } >"$SECRETS/node$k.env"
  chmod 600 "$SECRETS/node$k.env"
  echo "node$k  address $address"
done

"$BIN" genesis --chain-id "$CHAIN_ID" --block-interval "$BLOCK_INTERVAL" \
  "${args[@]}" --out "$ROOT/config/genesis.json"

cat <<EOF

Next:
  1. git add config/genesis.json && git commit -m "Add $CHAIN_ID genesis" && git push
  2. In Railway, create services node1..node$VALIDATORS from this repo and paste each
     .railway-secrets/nodeK.env into that service's Variables (Raw Editor).
  3. Follow RAILWAY.md for volumes, the public domain and the TCP proxy.
EOF
