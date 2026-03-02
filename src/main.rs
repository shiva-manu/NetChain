// src/main.rs

mod block;
mod blockchain;
mod mempool;
mod p2p;
mod state;
mod transaction;

use anyhow::Result;
use mempool::Mempool;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{interval, Duration};
use transaction::SignedTransaction;

use block::Block;
use blockchain::Blockchain;
use p2p::{NetworkMessage, P2PEvent, P2PService};

#[tokio::main]
async fn main() -> Result<()> {
    println!("⚡ Starting NetChain (development mode)");

    // Shared blockchain state
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));
    let mempool = Arc::new(Mutex::new(Mempool::new()));
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

    let proposer_enabled = std::env::var("NETCHAIN_ENABLE_PROPOSER")
        .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false);
    let proposer_interval_secs = std::env::var("NETCHAIN_PROPOSER_INTERVAL_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .filter(|secs| (2..=5).contains(secs))
        .unwrap_or(3);

    if proposer_enabled {
        let p2p_proposer = p2p.clone();
        let blockchain_proposer = blockchain.clone();
        let mempool_proposer = mempool.clone();

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(proposer_interval_secs));
            let max_txs = 100usize;

            println!(
                "⛏️ Proposer enabled (interval={}s, max_txs={max_txs})",
                proposer_interval_secs
            );

            loop {
                ticker.tick().await;
                println!("⏱️ Proposer tick");

                let selected_txs = {
                    let mempool = mempool_proposer.lock().await;
                    mempool.select_for_block(max_txs)
                };

                println!("📥 Selected {} tx(s) for proposal", selected_txs.len());

                if selected_txs.is_empty() {
                    println!("⏭️ Skipping proposal: mempool is empty");
                    continue;
                }

                let payload = selected_txs
                    .iter()
                    .map(SignedTransaction::tx_hash_hex)
                    .collect::<Vec<_>>()
                    .join(",");

                let maybe_block = {
                    let bc = blockchain_proposer.lock().await;
                    let last = bc.last_block();
                    let candidate_block =
                        Block::new(last.index + 1, format!("txs:{payload}"), last.hash.clone());

                    if candidate_block.previous_hash != last.hash {
                        println!("❌ Proposal failed local validation: invalid previous hash");
                        None
                    } else {
                        Some(candidate_block)
                    }
                };

                let Some(block) = maybe_block else {
                    continue;
                };

                {
                    let mut bc = blockchain_proposer.lock().await;
                    if let Err(e) = bc.validate_and_add_block(block.clone()) {
                        println!("❌ Failed to append proposed block locally: {e}");
                        continue;
                    }
                }

                {
                    let mut mempool = mempool_proposer.lock().await;
                    mempool.remove_transactions(&selected_txs);
                }

                match serde_json::to_string(&block) {
                    Ok(json) => {
                        let mut p2p = p2p_proposer.lock().await;
                        p2p.publish_block(json);
                        println!("✅ Proposal succeeded: block {} broadcasted", block.index);
                    }
                    Err(e) => {
                        println!("❌ Proposal failed serialization: {e}");
                    }
                }
            }
        });
    } else {
        println!("⏸️ Proposer disabled (set NETCHAIN_ENABLE_PROPOSER=true to enable)");
    }

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
