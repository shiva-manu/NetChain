# ⚡ SpeedChain — A Blockchain Secured by Internet Speed (PoI Consensus)

SpeedChain is a **next-generation Layer-1 blockchain** that replaces traditional Proof-of-Work (PoW) and Proof-of-Stake (PoS) with a new consensus algorithm called **Proof of Internet (PoI)** — where **validators are selected based on their real internet speed, stability, and uptime**.

SpeedChain is fast, fair, and energy-efficient. Anyone with a strong internet connection can participate and earn rewards.

---

## 🚀 Features

### ✅ **1. Proof of Internet (PoI) Consensus**
SpeedChain selects validators based on:
- Upload speed
- Download speed
- Latency
- Uptime
- Packet stability

Faster and more stable nodes → higher chances of validating blocks → more rewards.

---

### ✅ **2. Lightweight Rust Implementation**
SpeedChain is built in **Rust**, giving:
- High performance  
- Memory safety  
- Zero-cost abstractions  
- Modern cryptography support  

---

### ✅ **3. Simple, Modular Architecture**
The project is divided into stages:

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


## 🧱 Current Progress

### Completed:
✔ Basic blockchain (blocks, hashing, chain validation)  
✔ Genesis block  
✔ Add & validate new blocks  

### In Progress:
🔄 Wallets and signed transactions  
🔄 P2P networking layer  
🔄 Speed measurement module  

### Coming Soon:
⏳ Full PoI consensus  
⏳ SpeedChain Testnet  
⏳ Validator dashboard  
⏳ Explorer  
⏳ Native token economics (SC token)

---

## 💡 How Proof of Internet Works (Simple)

1. Each validator runs a SpeedChain node.  
2. Node performs continuous internet tests:
   - Upload  
   - Download  
   - Latency  
   - Packet loss  
3. Node submits a "Speed Proof" to the network.  
4. The validator selection algorithm ranks nodes.  
5. Fastest + most stable nodes produce blocks and earn **SC tokens**.  

This prevents:
- Expensive mining  
- Rich-controlled staking  
- Centralization  

PoI gives **fair access to everyone with strong internet**.

---

## 🛠 Getting Started (Development Mode)

### 1️⃣ Install Rust
```bash
curl https://sh.rustup.rs -sSf | sh