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
│ ├── block.rs # Block structure + hashing
│ ├── blockchain.rs # Blockchain logic
│ ├── wallet.rs # Keypairs, signing, verification
│ ├── network.rs # P2P networking
│ └── consensus.rs # Proof of Internet engine
│
├── Cargo.toml
└── README.md



*(Modules will grow as development continues.)*

---

## 🧱 Current Progress

### ✔ Completed
- Basic blockchain (blocks, hashing, chain validation)  
- Genesis block  
- Add & validate new blocks  

### 🔧 In Progress
- Wallets and signed transactions  
- P2P networking layer  
- Speed measurement module  

### ⏳ Coming Soon
- Full PoI consensus  
- NetChain Public Testnet  
- Validator dashboard  
- Block explorer  
- Native token economics (NC token)

---

## 💡 How Proof of Internet Works (Simple)

1. Each validator runs a **NetChain node**.  
2. The node performs continuous internet tests:
   - Upload speed  
   - Download speed  
   - Latency  
   - Packet stability  
3. Nodes submit a **Speed Proof** to the blockchain.  
4. NetChain ranks validators based on their results.  
5. The fastest + most stable nodes produce blocks and earn **NC tokens**.

This prevents:
- Costly mining  
- Rich-only staking systems  
- Centralized networks  

**PoI = Fair validation for everyone with strong internet.**

---

## 🛠 Getting Started (Development Mode)

### 1️⃣ Install Rust
```bash
curl https://sh.rustup.rs -sSf | sh
