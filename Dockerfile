# NetChain node image. Railway (and any Docker host) builds this file automatically.
FROM rust:1.91-bookworm AS builder
WORKDIR /app

# Build dependencies first so source-only changes reuse this layer.
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src/bin \
    && echo "fn main() {}" > src/main.rs \
    && echo "fn main() {}" > src/bin/wallet.rs \
    && touch src/lib.rs \
    && cargo build --release --bins \
    && rm -rf src target/release/deps/netchain* target/release/deps/libnetchain* \
       target/release/netchain target/release/netchain-wallet

COPY src ./src
RUN cargo build --release --bins

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/netchain /usr/local/bin/netchain
COPY --from=builder /app/target/release/netchain-wallet /usr/local/bin/netchain-wallet
COPY config ./config

# Container defaults: listen on all interfaces (IPv4 + IPv6, needed for Railway private
# networking), keep data on a mounted volume at /data, no mDNS (no LAN in the cloud).
# PORT (HTTP gateway) defaults to 8545 and is overridden by the platform if it sets PORT.
ENV NETCHAIN_CONFIG=/app/config/default.toml \
    NETCHAIN_HTTP_BIND_ADDR=:: \
    NETCHAIN_P2P_PORT=30333 \
    NETCHAIN_ENABLE_MDNS=false \
    DATA_DIR=/data

# 8545: HTTP gateway (RPC, REST, /health, /metrics, /validators, /genesis, /ws)
# 30333: libp2p peer-to-peer (TCP)
EXPOSE 8545 30333

CMD ["netchain", "run"]
