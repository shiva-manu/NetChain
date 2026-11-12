#!/usr/bin/env bash
# Run a 3-validator NetChain network on this machine.
#
#   scripts/local-testnet.sh start   # build, create genesis, start 3 nodes
#   scripts/local-testnet.sh status  # height / tip hash / peers of every node
#   scripts/local-testnet.sh stop    # stop all nodes
#   scripts/local-testnet.sh reset   # stop and delete all testnet data
#
# Node N listens on p2p 3040N, RPC 860N, monitoring 960N, websocket 870N.
# Nodes 2 and 3 find node 1 through its bootnode address (mDNS is disabled on purpose,
# so this exercises the same path as nodes on different machines).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
NET_DIR="${NET_DIR:-$ROOT/.testnet}"
BIN="$ROOT/target/debug/netchain"
NODES=(1 2 3)
BLOCK_INTERVAL="${BLOCK_INTERVAL:-4}"

export PATH="$HOME/.cargo/bin:$PATH"

node_env() {
  local n=$1
  echo "DATA_DIR=$NET_DIR/node$n PORT=3040$n RPC_PORT=860$n NETCHAIN_MONITORING_PORT=960$n \
NETCHAIN_WS_PORT=870$n NETCHAIN_GENESIS=$NET_DIR/genesis.json NETCHAIN_ENABLE_MDNS=false \
NETCHAIN_ATTEST_INTERVAL_SECS=${ATTEST_INTERVAL:-20} NETCHAIN_LOG_LEVEL=${LOG_LEVEL:-info}"
}

start_node() {
  local n=$1 bootnodes=""
  if [[ $n != 1 ]]; then
    bootnodes="/ip4/127.0.0.1/tcp/30401"
  fi
  # shellcheck disable=SC2046
  env $(node_env "$n") NETCHAIN_BOOTNODES="$bootnodes" \
    "$BIN" run >"$NET_DIR/node$n.log" 2>&1 &
  echo $! >"$NET_DIR/node$n.pid"
  echo "node$n started (pid $!, rpc http://127.0.0.1:860$n, log $NET_DIR/node$n.log)"
}

cmd_start() {
  (cd "$ROOT" && cargo build --bins)
  mkdir -p "$NET_DIR"
  if [[ ! -f "$NET_DIR/genesis.json" ]]; then
    local args=()
    for n in "${NODES[@]}"; do
      # shellcheck disable=SC2046
      addr=$(env $(node_env "$n") "$BIN" id | awk '/^address:/ {print $2}')
      echo "node$n address $addr"
      args+=(--validator "$addr:1000")
      [[ $n == 1 ]] && args+=(--alloc "$addr:1000000")
    done
    "$BIN" genesis --chain-id netchain-local --block-interval "$BLOCK_INTERVAL" \
      "${args[@]}" --out "$NET_DIR/genesis.json"
  fi
  for n in "${NODES[@]}"; do
    if [[ -f "$NET_DIR/node$n.pid" ]] && kill -0 "$(cat "$NET_DIR/node$n.pid")" 2>/dev/null; then
      echo "node$n already running"
    else
      start_node "$n"
      sleep 1
    fi
  done
}

rpc() {
  curl -s -X POST "http://127.0.0.1:860$1/rpc" -H 'content-type: application/json' -d "$2"
}

cmd_status() {
  for n in "${NODES[@]}"; do
    info=$(rpc "$n" '{"method":"get_chain_info"}' || true)
    peers=$(curl -s "http://127.0.0.1:960$n/health" | sed -n 's/.*"peer_count":\([0-9]*\).*/\1/p' || true)
    height=$(echo "$info" | sed -n 's/.*"height":\([0-9]*\).*/\1/p')
    tip=$(echo "$info" | sed -n 's/.*"latest_block_hash":"\([0-9a-f]\{12\}\).*/\1/p')
    echo "node$n height=${height:-down} tip=${tip:--} peers=${peers:--}"
  done
}

cmd_stop() {
  for n in "${NODES[@]}"; do
    if [[ -f "$NET_DIR/node$n.pid" ]]; then
      kill "$(cat "$NET_DIR/node$n.pid")" 2>/dev/null || true
      rm -f "$NET_DIR/node$n.pid"
      echo "node$n stopped"
    fi
  done
}

case "${1:-}" in
  start) cmd_start ;;
  status) cmd_status ;;
  stop) cmd_stop ;;
  reset) cmd_stop; rm -rf "$NET_DIR"; echo "removed $NET_DIR" ;;
  *) sed -n '2,12p' "$0"; exit 1 ;;
esac
