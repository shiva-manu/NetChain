// src/main.rs

mod block;
mod blockchain;
mod p2p;

use anyhow::Result;
use tokio::sync::mpsc;

use p2p::{P2PService, P2PEvent, NetworkMessage};
use blockchain::Blockchain;
use block::Block;

#[tokio::main]
async fn main() -> Result<()> {
    println!("⚡ Starting NetChain (development mode)");

    // Initialize blockchain
    let mut blockchain = Blockchain::new();
    println!("Genesis block: {:?}", blockchain.last_block());

    // Channel: P2P → main
    let (tx, mut rx) = mpsc::channel(100);

    // Start P2P networking
    let port = 30333;
    let p2p = P2PService::new(port).await?;
    tokio::spawn(p2p.run(tx));

    println!("Node running on port {port}. Waiting for P2P events...\n");

    // Main event loop
    while let Some(event) = rx.recv().await {
        match event {
            P2PEvent::Message(NetworkMessage::Block(block_json)) => {
                println!("📦 Received block data");

                match serde_json::from_str::<Block>(&block_json) {
                    Ok(block) => {
                        match blockchain.validate_and_add_block(block) {
                            Ok(_) => {
                                println!(
                                    "✅ Block accepted. Chain height: {}",
                                    blockchain.chain.len() - 1
                                );
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
