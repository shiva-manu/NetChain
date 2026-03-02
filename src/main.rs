// src/main.rs

mod block;
mod blockchain;
mod p2p;
mod state;
mod storage;
mod transaction;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{sleep, Duration};

use block::Block;
use blockchain::Blockchain;
use p2p::{NetworkMessage, P2PEvent, P2PService};
use state::State;
use storage::Storage;

const SNAPSHOT_INTERVAL: usize = 5;

#[tokio::main]
async fn main() -> Result<()> {
    println!("⚡ Starting NetChain (development mode)");

    let storage = Storage::new("data");

    let chain = match storage.load_chain() {
        Ok(Some(chain)) => {
            println!("💾 Loaded persisted chain with height {}", chain.len() - 1);
            Blockchain::from_chain(chain).map_err(anyhow::Error::msg)?
        }
        Ok(None) => {
            println!("🆕 No persisted chain found. Initializing genesis chain.");
            Blockchain::new()
        }
        Err(e) => {
            println!("⚠️ Failed to load persisted chain ({e}). Initializing genesis chain.");
            Blockchain::new()
        }
    };

    let state = match storage.load_state_snapshot() {
        Ok(Some((state, height))) => {
            println!("💾 Loaded state snapshot at height {height}");
            state
        }
        Ok(None) => {
            println!("🆕 No persisted state snapshot found. Starting empty state.");
            State::new()
        }
        Err(e) => {
            println!("⚠️ Failed to load state snapshot ({e}). Starting empty state.");
            State::new()
        }
    };

    let blockchain = Arc::new(Mutex::new(chain));
    let state = Arc::new(Mutex::new(state));

    {
        let bc = blockchain.lock().await;
        println!("Current tip: {:?}", bc.last_block());
    }

    // Channel: P2P → main
    let (tx, mut rx) = mpsc::channel(100);

    // Start P2P networking
    let port = 30333;
    let p2p = Arc::new(Mutex::new(P2PService::new(port).await?));

    let p2p_runner = p2p.clone();
    tokio::spawn(async move {
        let mut p2p = p2p_runner.lock().await;
        p2p.run(tx).await;
    });

    println!("Node running on port {port}. Waiting for P2P events...\n");

    // 🔥 TEMPORARY: create & broadcast a block after 10 seconds
    let p2p_broadcast = p2p.clone();
    let blockchain_broadcast = blockchain.clone();
    let state_broadcast = state.clone();
    let storage_broadcast = storage.clone();
    tokio::spawn(async move {
        sleep(Duration::from_secs(10)).await;

        println!("⛏️ Creating new block locally...");

        let block = {
            let mut bc = blockchain_broadcast.lock().await;
            let block = bc.add_block("Hello from NetChain".to_string());

            if let Err(e) = storage_broadcast.persist_chain(&bc.chain) {
                println!("⚠️ Failed to persist chain after local block: {e}");
            }

            if (bc.chain.len() - 1) % SNAPSHOT_INTERVAL == 0 {
                let state_guard = state_broadcast.lock().await;
                if let Err(e) =
                    storage_broadcast.persist_state_snapshot(&state_guard, bc.chain.len() - 1)
                {
                    println!("⚠️ Failed to persist state snapshot: {e}");
                }
            }

            block
            bc.add_block(vec![])
        };

        let json = serde_json::to_string(&block).unwrap();

        let mut p2p = p2p_broadcast.lock().await;
        p2p.publish_block(json);

        println!("📡 Block broadcasted");
    });

    // Main event loop
    while let Some(event) = rx.recv().await {
        match event {
            P2PEvent::Message(NetworkMessage::Block(block_json)) => {
                println!("📦 Received block data");

                match serde_json::from_str::<Block>(&block_json) {
                    Ok(block) => {
                        let mut bc = blockchain.lock().await;
                        match bc.validate_and_add_block(block) {
                            Ok(_) => {
                                println!("✅ Block accepted. Chain height: {}", bc.chain.len() - 1);

                                if let Err(e) = storage.persist_chain(&bc.chain) {
                                    println!("⚠️ Failed to persist chain: {e}");
                                }

                                if (bc.chain.len() - 1) % SNAPSHOT_INTERVAL == 0 {
                                    let state_guard = state.lock().await;
                                    if let Err(e) = storage
                                        .persist_state_snapshot(&state_guard, bc.chain.len() - 1)
                                    {
                                        println!("⚠️ Failed to persist state snapshot: {e}");
                                    }
                                }
                            }
                            Err(e) => {
                                println!("❌ Block rejected: {e}");
                            }
                        }
                    }
                    Err(e) => {
                        println!("❌ Failed to deserialize block: {e}");
                    }
                }
            }

            P2PEvent::Message(NetworkMessage::Transaction(tx)) => {
                println!("💸 Received transaction (not yet handled): {tx}");
            }

            P2PEvent::PeerConnected(peer) => {
                println!("🔗 Peer connected: {peer}");
            }

            P2PEvent::PeerDisconnected(peer) => {
                println!("❌ Peer disconnected: {peer}");
            }
        }
    }

    Ok(())
}
