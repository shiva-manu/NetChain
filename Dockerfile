FROM rust:1.86-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY config ./config

RUN cargo build --release --bins

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/netchain /usr/local/bin/netchain
COPY --from=builder /app/target/release/netchain-wallet /usr/local/bin/netchain-wallet
COPY config ./config

ENV NETCHAIN_CONFIG=/app/config/default.toml
EXPOSE 30333 8545 9090

CMD ["netchain"]
