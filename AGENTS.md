# AGENTS.md — NetChain

## Project Overview

NetChain is a Layer-1 blockchain node written in **Rust 2021** with a **Proof of Internet (PoI)** consensus mechanism. Validator selection is weighted by measured internet performance metrics (download, upload, latency, uptime, stability) blended with stake. The repo also contains a React/TypeScript marketing website under `website/` (separate sub-project with its own git history).

## Repository Structure

```
src/
├── main.rs                    # Node binary entry point, event loop, server startup
├── lib.rs                     # Library crate — flat re-exports all modules via #[path]
├── app/config.rs              # TOML + env config loading
├── bin/wallet.rs              # Wallet CLI binary (clap derive)
├── chain/                     # Core blockchain: block, blockchain, state, transaction
├── net/                       # Networking: p2p (libp2p), rpc (hyper), websocket, monitoring
├── node/                      # Node services: mempool, block producer, sled storage
├── poi/                       # PoI: consensus scoring, measurement, aggregator, anti-gaming
└── wallet/mod.rs              # Wallet crypto, encrypted key storage
```

Two binaries: `netchain` (node) and `netchain-wallet` (CLI wallet).

## Build / Lint / Test Commands

```bash
# Build
cargo build                          # Debug build
cargo build --release                # Release build

# Format
cargo fmt --all                      # Format all code
cargo fmt --all -- --check           # Check formatting (CI uses this)

# Lint / check
cargo check --all-targets            # Type-check without building

# Test — all tests
cargo test --all-targets             # Run all tests (CI uses this)

# Test — single test by name
cargo test test_empty_merkle_root    # Run one test by name substring
cargo test block::tests              # Run all tests in a module
cargo test -- --exact test_name      # Exact match

# Test — single file's tests
cargo test --lib block               # Tests in modules matching "block"

# Run
cargo run --bin netchain             # Run the node
cargo run --bin netchain-wallet      # Run the wallet CLI
```

CI pipeline (`.github/workflows/ci.yml`) runs on push to `main` and all PRs:
1. `cargo fmt --all -- --check`
2. `cargo check --all-targets`
3. `cargo test --all-targets`

## Code Style Guidelines

### Imports

Organized in this order, separated by blank lines:
1. External crate imports (`use anyhow::Result;`, `use serde::{Deserialize, Serialize};`)
2. Standard library imports (`use std::collections::HashMap;`)
3. Internal crate imports (`use crate::transaction::SignedTransaction;`)

Use nested imports for multiple items from one crate: `use serde::{Deserialize, Serialize};`

### Formatting

- Standard `rustfmt` defaults — no `.rustfmt.toml` overrides
- 4-space indentation
- Trailing commas in multi-line structs, function args, and match arms
- Section separators in larger files use comment banners: `// ==================== Section ====================`

### Naming Conventions

- **Structs/Enums**: `PascalCase` — `Block`, `SignedTransaction`, `StateError`, `TransactionType`
- **Functions/Methods**: `snake_case` — `compute_merkle_root`, `validate_next_block`
- **Constants**: `SCREAMING_SNAKE_CASE` — `BLOCKS_TREE`, `MAX_RPC_BODY_BYTES`, `WALLET_FORMAT_VERSION`
- **Modules**: `snake_case` — `anti_gaming`, `metrics_aggregator`, `rpc_types`
- **Type parameters**: single uppercase letters (`T`) or descriptive when needed

### Error Handling

Two patterns depending on context:

1. **`anyhow::Result`** — for infrastructure/IO operations (storage, config, networking, main)
2. **Domain-specific error enums** — for business logic validation (no `thiserror`), e.g. `StateError`, `MemPoolError`

- `?` operator for propagation with `anyhow::Result`
- `Result<T, SpecificError>` for domain functions (e.g., `Result<(), StateError>`)
- `unwrap()` / `expect()` only in tests and infallible cases
- Errors are logged with `tracing::error!` or `tracing::warn!` at the call site

### Types and Derives

Common derive sets:
- Data structs: `#[derive(Debug, Clone, Serialize, Deserialize)]`
- Enums needing equality: add `PartialEq, Eq`
- Error enums: `#[derive(Debug, Clone)]` (no Serialize)
- Config structs: `#[derive(Clone, Debug, Deserialize, Serialize)]` with `#[serde(default)]`

Serde attributes used extensively: `#[serde(default)]`, `#[serde(default = "fn_name")]`, `#[serde(skip)]`

### Documentation

- Module-level doc comments use `//!` at the top of each file
- Each file starts with a path comment: `// src/chain/block.rs`
- Public functions and structs have `///` doc comments
- Important struct fields have inline `///` documentation
- No rustdoc tests; documentation is descriptive

### Module Organization

- `lib.rs` uses `#[path = "..."]` to flatten all modules at the crate root (no nested `mod` re-exports)
- Modules are organized by domain: `chain/`, `net/`, `node/`, `poi/`, `app/`, `wallet/`
- One module uses `mod.rs` style (`wallet/mod.rs`); the rest use named files
- Test modules are always inline `#[cfg(test)] mod tests { ... }` at the bottom of each file

### Test Patterns

- All tests are inline `mod tests` blocks (no separate test files)
- Tests use `use super::*;` to import parent module items
- Helper functions (e.g., `make_signed_tx`) are defined inside test modules
- Assertions: `assert!`, `assert_eq!`, `assert_ne!` — no external assertion crates
- `tempfile::tempdir()` for storage tests needing temporary directories
- Dev dependency: `tempfile = "3.10"` only

### Async Patterns

- Runtime: `tokio` with `rt-multi-thread` and `macros` features
- Entry point: `#[tokio::main]` in `main.rs`
- Shared state: `Arc<Mutex<T>>` (using `tokio::sync::Mutex`)
- Channels: `tokio::sync::mpsc` for inter-task communication
- Spawned tasks use `tokio::spawn` for background work

### Logging

- Uses `tracing` crate: `info!`, `warn!`, `error!`
- Structured fields: `info!(path = %data_dir.display(), "using data directory")`
- Configured via `tracing_subscriber` with `EnvFilter` (reads `RUST_LOG` env var)
- Default log level set in `config/default.toml` (`log_level = "info"`)

### Serialization & Cryptography

- JSON (`serde_json`) for RPC/wallet/WebSocket; Bincode (`bincode`) for canonical tx serialization
- TOML (`toml`) for config; Hex (`hex`) for hashes/keys/addresses; Base64 for encrypted storage
- SHA-256 (`sha2`) for hashing; Ed25519 (`ed25519-dalek`) for signatures
- AES-256-GCM (`aes-gcm`) + Argon2id KDF for wallet encryption; `zeroize` for sensitive data

## Website (Secondary)

Located in `website/` — a standalone React/Vite/TypeScript project with its own git repo.

```bash
cd website
npm install              # Install dependencies (uses legacy-peer-deps)
npm run dev              # Dev server
npm run build            # Type-check + production build
npm run lint             # ESLint
```

Stack: React 19, Vite 8, TypeScript ~5.9, Tailwind CSS 4, shadcn/ui (base-nova style).
Path alias: `@/*` maps to `src/*`. No tests configured for the website.
