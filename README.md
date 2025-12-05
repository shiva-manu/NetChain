# ⚡ NetChain — A Blockchain Secured by Internet Speed (PoI Consensus)

NetChain is a **next-generation Layer-1 blockchain** that replaces traditional Proof-of-Work (PoW) and Proof-of-Stake (PoS) with a new consensus algorithm called **Proof of Internet (PoI)** — where **validators are selected based on their real internet speed, stability, and uptime**.

NetChain is fast, fair, and energy-efficient. Anyone with a strong internet connection can participate and earn rewards.

---

## 🚀 Features

### ✅ **1. Proof of Internet (PoI) Consensus**
NetChain selects validators based on:
- Upload speed  
- Download speed  
- Latency  
- Uptime  
- Packet stability  

Faster and more stable nodes → higher chances of validating blocks → more rewards.

---

### ✅ **2. Lightweight Rust Implementation**
NetChain is built in **Rust**, giving:
- High performance  
- Memory safety  
- Zero-cost abstractions  
- Modern cryptography  
- Fast execution  

---

### ✅ **3. Simple, Modular Architecture**

Development is divided into stages:

1. **Block & Blockchain layer**  
2. **Hashing and validation**  
3. **Wallets & digital signatures**  
4. **P2P networking (libp2p)**  
5. **Consensus engine (PoI)**  
6. **RPC layer for apps & wallets**  
7. **Testnet → Mainnet launch**

---

## 📁 Project Structure


NetChain/
│
├── src/
│ ├── main.rs # Entry point
# NetChain

An experimental Layer-1 blockchain prototype implemented in Rust. NetChain explores a Proof-of-Internet (PoI) consensus where validators are ranked by network performance (upload/download speed, latency, stability and uptime).

---

**Contents**

- [About](#about)
- [Highlights](#highlights)
- [Quick start](#quick-start)
- [Run a local node](#run-a-local-node)
- [Development](#development)
- [Project layout](#project-layout)
- [Testing](#testing)
- [Contributing](#contributing)
- [License](#license)

---

## About

NetChain replaces energy-intensive PoW and capital-weighted PoS with Proof-of-Internet (PoI) — a concept where nodes gain validation priority by demonstrating superior network performance. The repository is a developer-focused prototype: core blockchain logic, a PoI scoring engine, and networking primitives.

## Highlights

- Proof-of-Internet (PoI) consensus prototype
- Implemented in Rust for performance and safety
- Modular: blocks, chain validation, wallets, P2P networking, consensus

## Quick start

Prerequisites:

- Rust toolchain (install via `rustup`)

Install/update the Rust toolchain:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustup update
```

Build the project:

```bash
cargo build
```

Run the project (default entry):

```bash
cargo run
```

Run tests:

```bash
cargo test
```

Use `cargo run --release` for optimized runs.

## Run a local node

This repo is a prototype; a simple local run starts the node using `cargo run`. If the node accepts flags or a config file, pass them as arguments, for example:

```bash
# example (replace with real flags when implemented)
cargo run -- --listen 0.0.0.0:30333 --fast-sync
```

I can add a sample `config/default.toml` and CLI docs if you want a reproducible local test scenario.

## Development

- Work in `src/` and follow standard Rust conventions.
- Recommended workflow:
   1. Create a feature branch
   2. Add tests for new behavior
   3. Open a PR against `main` with a clear description

Key areas to work on:

- `consensus.rs` — PoI scoring & validator selection
- `network.rs` — P2P transport and discovery (libp2p integration)
- `wallet.rs` — key management and signing

## Project layout

```
netchain/
├── Cargo.toml          # manifest
├── README.md
└── src/
      ├── main.rs        # entry point
      ├── block.rs       # block structure & hashing
      ├── blockchain.rs  # chain logic
      ├── wallet.rs      # keys & signing
      ├── network.rs     # p2p networking
      └── consensus.rs   # PoI engine
```

Files and modules may be added as development continues.

## Testing

Run unit tests with `cargo test`. Add unit tests for core logic (blocks, validation, consensus scoring). Integration tests for networking may require multiple processes or test harnesses.

## Contributing

- Fork and open a pull request against `main`.
- Keep PRs focused and include tests for new behavior.
- If you'd like, I can add a `CONTRIBUTING.md` with a checklist and branch strategy.

## License

No `LICENSE` file detected. Add a `LICENSE` (e.g., MIT or Apache-2.0) to specify repository licensing.

---

If you'd like the README to include diagrams, example payloads, a configuration file, or ready-to-run local test scripts, tell me which and I will add them.
