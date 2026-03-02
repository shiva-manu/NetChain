mod block;
mod blockchain;
mod consensus;
mod mempool;
mod p2p;
mod state;
mod transaction;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{interval, Duration};

use block::Block;
use blockchain::Blockchain;
use mempool::Mempool;
use p2p::{NetworkMessage, P2PEvent, P2PService};
use transaction::SignedTransaction;

#[tokio::main]
async fn main() -> Result<()> {
    println!("⚡ Starting NetChain (development mode)");

    let blockchain = Arc::new(Mutex::new(Blockchain::new()));
    let mempool = Arc::new(Mutex::new(Mempool::new()));

    {
        let bc = blockchain.lock().await;
        println!("Genesis block: {:?}", bc.last_block());
    }

    let (tx, mut rx) = mpsc::channel(100);

    let port = 30333;
    let p2p = Arc::new(Mutex::new(P2PService::new(port).await?));

    let p2p_runner = p2p.clone();
    tokio::spawn(async move {
        let mut p2p = p2p_runner.lock().await;
        p2p.run(tx).await;
    });

    println!("Node running on port {port}. Waiting for P2P events...\n");

    let proposer_enabled = std::env::var("NETCHAIN_PROPOSER")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if proposer_enabled {
        println!("⛏️ Proposer loop enabled (NETCHAIN_PROPOSER)");
        let p2p_proposer = p2p.clone();
        let blockchain_proposer = blockchain.clone();
        let mempool_proposer = mempool.clone();

        tokio::spawn(async move {
            let mut tick = interval(Duration::from_secs(5));
            loop {
                tick.tick().await;

                let maybe_block = {
                    let mut bc = blockchain_proposer.lock().await;
                    let mut mp = mempool_proposer.lock().await;
                    let selected = mp.select_for_block(&bc.state, 100);

                    if selected.is_empty() {
                        println!("⏭️ Proposer tick: no pending transactions");
                        None
                    } else {
                        match bc.add_block(selected.clone()) {
                            Ok(block) => {
                                mp.remove_transactions(&selected);
                                println!(
                                    "✅ Proposed block #{} with {} txs",
                                    block.index,
                                    block.transactions.len()
                                );
                                Some(block)
                            }
                            Err(err) => {
                                println!("❌ Failed to propose block: {err}");
                                None
                            }
                        }
                    }
                };

                if let Some(block) = maybe_block {
                    if let Ok(json) = serde_json::to_string(&block) {
                        let mut p2p = p2p_proposer.lock().await;
                        p2p.publish_block(json);
                        println!("📡 Broadcasted proposed block");
                    }
                }
            }
        });
    } else {
        println!("ℹ️ Proposer loop disabled (set NETCHAIN_PROPOSER=1 to enable)");
    }

    while let Some(event) = rx.recv().await {
        match event {
            P2PEvent::Message(NetworkMessage::Block(block_json)) => {
                match serde_json::from_str::<Block>(&block_json) {
                    Ok(block) => {
                        let accepted = {
                            let mut bc = blockchain.lock().await;
                            bc.validate_and_add_block(block.clone()).is_ok()
                        };

                        if accepted {
                            let mut mp = mempool.lock().await;
                            mp.remove_transactions(&block.transactions);
                            let height = { blockchain.lock().await.chain.len() - 1 };
                            println!("✅ Block accepted. Chain height: {}", height);
                        } else {
                            println!("❌ Block rejected");
                        }
                    }
                    Err(e) => {
                        println!("❌ Failed to deserialize block: {e}");
                    }
                }
            }
            P2PEvent::Message(NetworkMessage::Transaction(tx_json)) => {
                match serde_json::from_str::<SignedTransaction>(&tx_json) {
                    Ok(tx) => {
                        let state_snapshot = {
                            let bc = blockchain.lock().await;
                            bc.state.clone()
                        };

                        let mut mp = mempool.lock().await;
                        match mp.add_transaction(tx, &state_snapshot) {
                            Ok(_) => println!("💸 Transaction accepted into mempool"),
                            Err(e) => println!("❌ Transaction rejected: {e:?}"),
                        }
                    }
                    Err(e) => println!("❌ Failed to deserialize transaction: {e}"),
                }
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
