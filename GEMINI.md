# NetChain: GEMINI.md

## Project Overview
NetChain is an experimental Layer-1 blockchain prototype implemented in Rust, centered around a hybrid **Proof of Internet (PoI)** consensus model. Validator selection is determined by a composite score of measured network performance (speed, latency, uptime), economic stake, identity confidence, and reputation.

### Core Technologies
- **Backend (Rust)**:
  - `libp2p`: P2P networking, gossipsub, and peer discovery.
  - `tokio`: Asynchronous runtime.
  - `sled`: Embedded key-value database for persistent storage.
  - `serde`: Serialization/deserialization.
  - `ed25519-dalek` & `sha2`: Cryptographic primitives.
  - `tracing`: Structured logging and telemetry.
- **Frontend (TypeScript/React)**:
  - `Vite`: Build tool and dev server.
  - `Tailwind CSS` & `shadcn/ui`: Styling and UI components.
  - `react-router-dom`: Frontend routing.
- **Infrastructure**:
  - `Docker` & `Docker Compose`: Containerization.
  - `Terraform`: Infrastructure as Code (AWS).

---

## Project Structure
```text
/
├── src/                    # Core Rust source code
│   ├── main.rs             # Node entry point (bin)
│   ├── lib.rs              # Shared library logic
│   ├── bin/wallet.rs       # Wallet CLI tool
│   ├── chain/              # Blocks, state transitions, and transactions
│   ├── net/                # Networking: P2P, RPC (JSON-RPC), WebSockets
│   ├── node/               # Mempool, block production, and storage logic
│   ├── poi/                # Consensus engine (Proof of Internet)
│   └── wallet/             # Wallet implementation and crypto helpers
├── website/                # React-based explorer and dashboard
├── config/                 # Runtime configuration (default.toml)
├── scripts/                # Setup and deployment scripts
└── terraform/              # AWS infrastructure configuration
```

---

## Building and Running

### Backend (Rust)
- **Build**: `cargo build`
- **Run Node**: `cargo run --bin netchain`
- **Run Wallet**: `cargo run --bin netchain-wallet`
- **Test**: `cargo test`
- **Format/Lint**: `cargo fmt` / `cargo clippy`

### Frontend (Website)
- **Setup**: `cd website && npm install`
- **Dev Mode**: `npm run dev`
- **Build**: `npm run build`

### Infrastructure
- **Docker**: `docker-compose up --build`
- **Deployment**: See `deploy/deploy.sh` and `terraform/main.tf`.

---

## Development Conventions

### Coding Standards
- **Rust**: Follow standard idiomatic Rust (use `clippy` and `fmt`). Prefer `anyhow` for error handling in application code.
- **Async**: Use `tokio` for all asynchronous operations.
- **Frontend**: Follow React 19 patterns. Use `shadcn/ui` components for consistency. Use Tailwind CSS v4 for styling.

### Networking & RPC
- **P2P**: Uses `libp2p` on port `30333` by default.
- **JSON-RPC**: Exposed on `127.0.0.1:8545`.
- **WebSocket**: Exposed on `127.0.0.1:8546` for real-time events (new blocks, transactions).
- **Monitoring**: Health and Prometheus metrics on `127.0.0.1:9090`.

### Consensus Logic (PoI)
The `src/poi/` directory contains the core scoring logic. Metrics like download/upload speeds and latency are normalized and weighted to select validators.

---

## Key Files for Reference
- `Cargo.toml`: Backend dependencies and workspace configuration.
- `src/main.rs`: Main node initialization and event loop.
- `src/lib.rs`: Library exports and core types.
- `website/package.json`: Frontend dependencies and scripts.
- `config/default.toml`: Default runtime parameters for the node.
- `README.md`: High-level user documentation.
