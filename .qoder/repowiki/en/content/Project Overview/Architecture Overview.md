# Architecture Overview

<cite>
**Referenced Files in This Document**
- [src/main.rs](file://src/main.rs)
- [src/blockchain.rs](file://src/blockchain.rs)
- [src/block.rs](file://src/block.rs)
- [src/state.rs](file://src/state.rs)
- [src/mempool.rs](file://src/mempool.rs)
- [src/transaction.rs](file://src/transaction.rs)
- [src/p2p.rs](file://src/p2p.rs)
- [src/consensus.rs](file://src/consensus.rs)
- [Cargo.toml](file://Cargo.toml)
- [README.md](file://README.md)
</cite>

## Table of Contents
1. [Introduction](#introduction)
2. [Project Structure](#project-structure)
3. [Core Components](#core-components)
4. [Architecture Overview](#architecture-overview)
5. [Detailed Component Analysis](#detailed-component-analysis)
6. [Dependency Analysis](#dependency-analysis)
7. [Performance Considerations](#performance-considerations)
8. [Troubleshooting Guide](#troubleshooting-guide)
9. [Conclusion](#conclusion)

## Introduction

NetChain is a next-generation Layer-1 blockchain prototype that implements a novel Proof-of-Internet (PoI) consensus mechanism. Unlike traditional Proof-of-Work or Proof-of-Stake systems, NetChain selects validators based on their real-world internet performance characteristics including upload/download speeds, latency, uptime, and packet stability.

The system follows a lightweight, modular architecture built in Rust with a focus on performance, memory safety, and modern cryptographic primitives. The implementation demonstrates a clean separation of concerns across blockchain management, state handling, memory pool operations, and P2P networking layers.

## Project Structure

The NetChain project follows a clean, modular structure organized around core blockchain components:

```mermaid
graph TB
subgraph "Application Root"
MAIN[src/main.rs<br/>Entry Point]
README[README.md<br/>Documentation]
CARGO[Cargo.toml<br/>Dependencies]
end
subgraph "Core Modules"
BLOCK[block.rs<br/>Block Structure]
BLOCKCHAIN[blockchain.rs<br/>Blockchain Logic]
STATE[state.rs<br/>State Management]
MEMPOOL[mempool.rs<br/>Transaction Pool]
TRANSACTION[transaction.rs<br/>Transaction System]
end
subgraph "Networking & Consensus"
P2P[p2p.rs<br/>P2P Service]
CONSENSUS[consensus.rs<br/>PoI Engine]
end
MAIN --> BLOCKCHAIN
MAIN --> STATE
MAIN --> MEMPOOL
MAIN --> P2P
MAIN --> CONSENSUS
BLOCKCHAIN --> BLOCK
STATE --> TRANSACTION
MEMPOOL --> TRANSACTION
P2P --> BLOCK
P2P --> TRANSACTION
CONSENSUS --> STATE
```

**Diagram sources**
- [src/main.rs](file://src/main.rs#L1-L145)
- [src/blockchain.rs](file://src/blockchain.rs#L1-L89)
- [src/state.rs](file://src/state.rs#L1-L183)
- [src/mempool.rs](file://src/mempool.rs#L1-L159)
- [src/p2p.rs](file://src/p2p.rs#L1-L150)
- [src/consensus.rs](file://src/consensus.rs#L1-L334)

**Section sources**
- [src/main.rs](file://src/main.rs#L1-L145)
- [Cargo.toml](file://Cargo.toml#L1-L47)

## Core Components

NetChain implements a comprehensive blockchain architecture with clearly defined component boundaries:

### Blockchain Core
The blockchain module provides fundamental ledger functionality including block creation, validation, and chain integrity maintenance. It manages the immutable chain of blocks and ensures cryptographic consistency through hash verification.

### State Management
The state system maintains account balances, nonces, and handles transaction validation and application. It provides atomic state transitions and enforces business rules for transaction processing.

### Memory Pool
The mempool component manages unconfirmed transactions with duplicate detection, nonce ordering, and fee-based prioritization. It serves as the staging area for transactions awaiting block inclusion.

### P2P Networking
Built on libp2p, the networking layer provides decentralized peer discovery, gossip-based message propagation, and secure encrypted communications using Noise protocol and Yamux multiplexing.

### Consensus Engine
The PoI consensus module implements a sophisticated scoring system that evaluates network performance metrics to determine validator selection probabilities, promoting nodes with superior internet connectivity.

**Section sources**
- [src/blockchain.rs](file://src/blockchain.rs#L1-L89)
- [src/state.rs](file://src/state.rs#L1-L183)
- [src/mempool.rs](file://src/mempool.rs#L1-L159)
- [src/p2p.rs](file://src/p2p.rs#L1-L150)
- [src/consensus.rs](file://src/consensus.rs#L1-L334)

## Architecture Overview

NetChain employs an event-driven architecture pattern that enables asynchronous communication between components through message passing channels. The system operates on a central event loop that coordinates blockchain operations, state updates, and network synchronization.

```mermaid
sequenceDiagram
participant Main as Main Thread
participant P2P as P2P Service
participant BC as Blockchain
participant State as State Manager
participant Mempool as Mempool
participant Consensus as PoI Engine
Main->>P2P : Initialize P2P Service
Main->>BC : Create Blockchain Instance
Main->>State : Initialize State
Main->>Mempool : Setup Transaction Pool
P2P->>Main : P2PEvent : : PeerConnected
P2P->>Main : P2PEvent : : Message(Block)
Main->>BC : validate_and_add_block()
BC-->>Main : Validation Result
P2P->>Main : P2PEvent : : Message(Transaction)
Main->>State : validate_transaction()
State-->>Main : Validation Result
Main->>Mempool : add_transaction()
Mempool-->>Main : Confirmation
Main->>Consensus : Update Metrics (when applicable)
Consensus-->>Main : Validator Selection (future)
```

**Diagram sources**
- [src/main.rs](file://src/main.rs#L23-L145)
- [src/p2p.rs](file://src/p2p.rs#L113-L141)
- [src/blockchain.rs](file://src/blockchain.rs#L40-L64)
- [src/state.rs](file://src/state.rs#L69-L95)
- [src/mempool.rs](file://src/mempool.rs#L42-L77)

The architecture follows a producer-consumer pattern where the P2P service acts as the primary event producer, generating messages that drive the blockchain's state transitions. The main thread serves as the central coordinator, orchestrating operations across all subsystems through a structured event handling mechanism.

**Section sources**
- [src/main.rs](file://src/main.rs#L44-L141)
- [src/p2p.rs](file://src/p2p.rs#L113-L141)

## Detailed Component Analysis

### Blockchain Management System

The blockchain component provides the foundational ledger functionality with robust validation mechanisms:

```mermaid
classDiagram
class Blockchain {
+Vec~Block~ chain
+new() Blockchain
+last_block() Block
+add_block(data : String) Block
+validate_and_add_block(block : Block) Result
+is_valid() bool
}
class Block {
+u64 index
+DateTime~Utc~ timestamp
+String data
+String previous_hash
+String hash
+new(index, data, previous_hash) Block
+calculate_hash() String
}
Blockchain --> Block : "manages"
```

**Diagram sources**
- [src/blockchain.rs](file://src/blockchain.rs#L5-L37)
- [src/block.rs](file://src/block.rs#L5-L25)

The blockchain implements a simple but effective validation pipeline that checks block indices, cryptographic chaining through hash verification, and maintains chain integrity through comprehensive validation routines.

**Section sources**
- [src/blockchain.rs](file://src/blockchain.rs#L10-L64)
- [src/block.rs](file://src/block.rs#L14-L46)

### State Management Architecture

The state system implements a ledger-based approach to account management with atomic transaction processing:

```mermaid
classDiagram
class State {
+HashMap~String,Account~ accounts
+new() State
+with_genesis(genesis : Vec) State
+get_balance(address : String) u64
+get_nonce(address : String) u64
+validate_transaction(tx : SignedTransaction) Result
+apply_transaction(tx : SignedTransaction) Result
+apply_transactions(txs : [SignedTransaction]) Result
}
class Account {
+u64 balance
+u64 nonce
+new(balance : u64) Account
}
class SignedTransaction {
+Transaction tx
+String signature
+String pubkey
+verify() Result
+tx_hash_hex() String
}
class Transaction {
+String sender
+String receiver
+u64 amount
+u64 fee
+u64 nonce
+u64 timestamp
+Option~String~ memo
+canonical_bytes() Vec~u8~
+tx_hash_hex() String
}
State --> Account : "manages"
State --> SignedTransaction : "validates"
SignedTransaction --> Transaction : "wraps"
```

**Diagram sources**
- [src/state.rs](file://src/state.rs#L16-L128)
- [src/transaction.rs](file://src/transaction.rs#L23-L144)

The state management system enforces strict validation rules including cryptographic signature verification, balance calculations, nonce sequencing, and transaction atomicity through comprehensive error handling.

**Section sources**
- [src/state.rs](file://src/state.rs#L36-L128)
- [src/transaction.rs](file://src/transaction.rs#L23-L144)

### Memory Pool Operations

The mempool component implements sophisticated transaction queuing with duplicate prevention and nonce ordering:

```mermaid
flowchart TD
Start([Add Transaction]) --> HashCheck["Check Transaction Hash"]
HashCheck --> Duplicate{"Duplicate Found?"}
Duplicate --> |Yes| ReturnDup["Return DuplicateTransaction Error"]
Duplicate --> |No| ValidateState["Validate Against State"]
ValidateState --> StateValid{"State Valid?"}
StateValid --> |No| ReturnInvalid["Return InvalidTransaction Error"]
StateValid --> |Yes| CheckNonce["Check Sender Nonce Queue"]
CheckNonce --> NonceValid{"Nonce Order Valid?"}
NonceValid --> |No| ReturnNonce["Return NonceTooLow Error"]
NonceValid --> |Yes| InsertTx["Insert Into Mempool"]
InsertTx --> UpdateQueues["Update Sender Queues"]
UpdateQueues --> Complete([Transaction Added])
ReturnDup --> Complete
ReturnInvalid --> Complete
ReturnNonce --> Complete
```

**Diagram sources**
- [src/mempool.rs](file://src/mempool.rs#L42-L77)

The mempool ensures transaction integrity through multiple validation layers and maintains optimal ordering for block production through fee-based prioritization.

**Section sources**
- [src/mempool.rs](file://src/mempool.rs#L26-L113)

### P2P Networking Infrastructure

The networking layer leverages libp2p for decentralized peer-to-peer communication with comprehensive message routing:

```mermaid
graph TB
subgraph "P2P Service"
SWARM[libp2p Swarm]
BEHAVIOR[NetBehaviour]
GOSSIPSUB[Gossipsub]
MDNS[Multicast DNS]
end
subgraph "Topics"
BLOCK_TOPIC["blocks Topic"]
TX_TOPIC["transactions Topic"]
end
subgraph "Event Flow"
EVENT[SwarmEvent]
MESSAGE[NetworkMessage]
P2PEVENT[P2PEvent]
end
SWARM --> BEHAVIOR
BEHAVIOR --> GOSSIPSUB
BEHAVIOR --> MDNS
GOSSIPSUB --> BLOCK_TOPIC
GOSSIPSUB --> TX_TOPIC
SWARM --> EVENT
EVENT --> MESSAGE
MESSAGE --> P2PEVENT
```

**Diagram sources**
- [src/p2p.rs](file://src/p2p.rs#L38-L111)

The P2P system provides secure, encrypted communication channels with automatic peer discovery and efficient message propagation through gossip protocols.

**Section sources**
- [src/p2p.rs](file://src/p2p.rs#L63-L149)

### PoI Consensus Engine

The PoI consensus module implements a sophisticated scoring system for validator selection based on network performance metrics:

```mermaid
flowchart TD
Metrics[NodeMetrics] --> Normalize[Normalize Metrics]
Normalize --> WeightCalc[Weighted Sum Calculation]
WeightCalc --> Score[Final PoI Score 0.0-1.0]
Score --> ValidatorSelection[Validator Selection]
subgraph "Normalization Functions"
Normal[Normal: val/max]
Invert[Invert: 1-(val/max)]
end
Metrics --> Normal
Metrics --> Invert
Normal --> WeightCalc
Invert --> WeightCalc
subgraph "Selection Methods"
Seed[Seed-based Selection]
RNG[RNG-based Selection]
end
WeightCalc --> Seed
WeightCalc --> RNG
```

**Diagram sources**
- [src/consensus.rs](file://src/consensus.rs#L63-L182)

The consensus engine provides both deterministic and probabilistic validator selection mechanisms, ensuring fair and transparent governance based on network contribution rather than capital ownership.

**Section sources**
- [src/consensus.rs](file://src/consensus.rs#L57-L182)

## Dependency Analysis

NetChain's dependency graph reflects a clean, layered architecture with minimal coupling between components:

```mermaid
graph TB
subgraph "External Dependencies"
TOKIO[Tokio Runtime]
LIBP2P[libp2p Networking]
ED25519[Ed25519 Cryptography]
SERDE[Serialization]
SHA256[SHA-256 Hashing]
end
subgraph "Internal Modules"
MAIN[main.rs]
BLOCKCHAIN[blockchain.rs]
STATE[state.rs]
MEMPOOL[mempool.rs]
P2P[p2p.rs]
CONSENSUS[consensus.rs]
TRANSACTION[transaction.rs]
BLOCK[block.rs]
end
MAIN --> BLOCKCHAIN
MAIN --> STATE
MAIN --> MEMPOOL
MAIN --> P2P
MAIN --> CONSENSUS
BLOCKCHAIN --> BLOCK
STATE --> TRANSACTION
MEMPOOL --> TRANSACTION
P2P --> BLOCK
P2P --> TRANSACTION
MAIN --> TOKIO
P2P --> LIBP2P
STATE --> ED25519
TRANSACTION --> ED25519
BLOCK --> SHA256
STATE --> SERDE
TRANSACTION --> SERDE
```

**Diagram sources**
- [Cargo.toml](file://Cargo.toml#L6-L47)
- [src/main.rs](file://src/main.rs#L11-L21)

The dependency analysis reveals a well-structured system where external dependencies are primarily used for specific functionalities: Tokio for async operations, libp2p for networking, Ed25519 for cryptography, and various serialization libraries for data interchange.

**Section sources**
- [Cargo.toml](file://Cargo.toml#L6-L47)

## Performance Considerations

NetChain's architecture incorporates several performance optimization strategies:

### Asynchronous Processing
The system leverages Tokio's async runtime to handle concurrent operations efficiently, enabling non-blocking I/O operations and parallel processing of network events and blockchain operations.

### Memory Efficiency
Components use optimized data structures including HashMap for transaction indexing, VecDeque for nonce-ordered queues, and compact binary serialization for efficient storage and transmission.

### Network Optimization
The libp2p integration provides efficient peer discovery, message routing, and connection management with built-in encryption and compression capabilities.

### Cryptographic Efficiency
Ed25519 signatures offer excellent performance characteristics with small key sizes and fast verification times, suitable for high-throughput blockchain operations.

## Troubleshooting Guide

Common issues and their resolution strategies:

### Network Connectivity Problems
- **Symptom**: Peers not connecting or messages not propagating
- **Solution**: Verify port availability, firewall settings, and libp2p transport configuration

### Transaction Validation Failures
- **Symptom**: Transactions rejected with validation errors
- **Solution**: Check signature verification, nonce sequencing, and account balance sufficient funds

### Blockchain Synchronization Issues
- **Symptom**: Chain validation failures or fork conflicts
- **Solution**: Verify block hash calculations, index continuity, and previous hash matching

### Memory Pool Errors
- **Symptom**: Duplicate transaction rejections or nonce ordering problems
- **Solution**: Review transaction hash uniqueness and sender queue management

**Section sources**
- [src/state.rs](file://src/state.rs#L6-L14)
- [src/mempool.rs](file://src/mempool.rs#L5-L11)
- [src/blockchain.rs](file://src/blockchain.rs#L40-L64)

## Conclusion

NetChain represents a forward-thinking approach to blockchain architecture that prioritizes performance, modularity, and practical utility. The system's event-driven design, combined with Rust's memory safety guarantees and libp2p's robust networking capabilities, creates a foundation for scalable, efficient blockchain operations.

The PoI consensus mechanism introduces a novel approach to validator selection that aligns economic incentives with network contribution, potentially reducing energy consumption while maintaining security and decentralization. The modular architecture ensures that each component can be independently developed, tested, and optimized while maintaining clear interfaces and well-defined responsibilities.

This architectural approach positions NetChain as a promising candidate for next-generation blockchain infrastructure, demonstrating how modern technologies and innovative consensus mechanisms can work together to create more sustainable and efficient distributed systems.