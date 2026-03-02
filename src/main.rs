// src/main.rs

mod block;
mod blockchain;
mod p2p;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{sleep, Duration};

use block::Block;
use blockchain::Blockchain;
use p2p::{NetworkMessage, P2PEvent, P2PService};

#[tokio::main]
async fn main() -> Result<()> {
    println!("⚡ Starting NetChain (development mode)");

    // Shared blockchain state
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));
    {
        let bc = blockchain.lock().await;
        println!("Genesis block: {:?}", bc.last_block());
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
    tokio::spawn(async move {
        sleep(Duration::from_secs(10)).await;

        println!("⛏️ Creating new block locally...");

        let block = {
            let mut bc = blockchain_broadcast.lock().await;
            bc.add_block("Hello from NetChain".to_string())
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
