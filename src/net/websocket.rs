// src/net/websocket.rs
//! WebSocket server for real-time event subscriptions.
//! Clients connect and subscribe to topics: new_blocks, new_transactions, proposals.

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tokio::sync::broadcast;
use tracing::{info, warn};

use crate::state::StateEvent;

const MAX_WS_MESSAGE_BYTES: usize = 256 * 1024; // 256 KiB
const MAX_WS_FRAME_BYTES: usize = 64 * 1024; // 64 KiB

/// Topics clients can subscribe to
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WsTopic {
    NewBlocks,
    NewTransactions,
    Proposals,
    Slashing,
    Contracts,
    Tokens,
    Nfts,
}

/// Events broadcast to WebSocket subscribers
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", content = "data")]
pub enum WsEvent {
    #[serde(rename = "new_block")]
    NewBlock {
        index: u64,
        hash: String,
        validator: String,
        tx_count: usize,
        timestamp: String,
    },
    #[serde(rename = "new_transaction")]
    NewTransaction {
        tx_hash: String,
        sender: String,
        receiver: String,
        amount: u64,
        tx_type: String,
    },
    #[serde(rename = "proposal_update")]
    ProposalUpdate {
        proposal_id: u64,
        title: String,
        status: String,
        yes_votes: u64,
        no_votes: u64,
    },
    #[serde(rename = "validator_slashed")]
    ValidatorSlashed {
        validator: String,
        reason: String,
        amount_burned: u64,
        remaining_stake: u64,
    },
    #[serde(rename = "contract_deployed")]
    ContractDeployed {
        contract_address: String,
        deployer: String,
        code_hash: String,
    },
    #[serde(rename = "contract_called")]
    ContractCalled {
        contract_address: String,
        caller: String,
        function: String,
        gas_used: u64,
    },
    #[serde(rename = "token_created")]
    TokenCreated {
        token_id: String,
        creator: String,
        name: String,
        symbol: String,
    },
    #[serde(rename = "token_minted")]
    TokenMinted {
        token_id: String,
        to: String,
        amount: u64,
    },
    #[serde(rename = "token_transferred")]
    TokenTransferred {
        token_id: String,
        from: String,
        to: String,
        amount: u64,
    },
    #[serde(rename = "token_burned")]
    TokenBurned { token_id: String, amount: u64 },
    #[serde(rename = "nft_created")]
    NftCreated {
        nft_id: String,
        collection_id: String,
        creator: String,
        name: String,
    },
    #[serde(rename = "nft_transferred")]
    NftTransferred {
        nft_id: String,
        from: String,
        to: String,
    },
    #[serde(rename = "nft_burned")]
    NftBurned { nft_id: String },
}

impl WsEvent {
    pub fn topic(&self) -> WsTopic {
        match self {
            WsEvent::NewBlock { .. } => WsTopic::NewBlocks,
            WsEvent::NewTransaction { .. } => WsTopic::NewTransactions,
            WsEvent::ProposalUpdate { .. } => WsTopic::Proposals,
            WsEvent::ValidatorSlashed { .. } => WsTopic::Slashing,
            WsEvent::ContractDeployed { .. } | WsEvent::ContractCalled { .. } => WsTopic::Contracts,
            WsEvent::TokenCreated { .. }
            | WsEvent::TokenMinted { .. }
            | WsEvent::TokenTransferred { .. }
            | WsEvent::TokenBurned { .. } => WsTopic::Tokens,
            WsEvent::NftCreated { .. }
            | WsEvent::NftTransferred { .. }
            | WsEvent::NftBurned { .. } => WsTopic::Nfts,
        }
    }
}

/// Client-to-server messages for subscription management
#[derive(Debug, Deserialize)]
#[serde(tag = "action")]
enum ClientMessage {
    #[serde(rename = "subscribe")]
    Subscribe { topics: Vec<WsTopic> },
    #[serde(rename = "unsubscribe")]
    Unsubscribe { topics: Vec<WsTopic> },
    #[serde(rename = "ping")]
    Ping,
}

/// Server-to-client control messages
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum ServerControlMessage {
    #[serde(rename = "subscribed")]
    Subscribed { topics: Vec<WsTopic> },
    #[serde(rename = "pong")]
    Pong,
    #[serde(rename = "error")]
    Error { message: String },
}

/// Creates a broadcast channel for WS events. Returns the sender.
/// The sender is cloned into main.rs to broadcast events.
pub fn create_event_channel() -> broadcast::Sender<WsEvent> {
    let (tx, _) = broadcast::channel(256);
    tx
}

/// Answer an HTTP `GET /ws` with a WebSocket upgrade and serve live events on it.
/// Returns `None` if the request is not a valid WebSocket upgrade.
pub fn upgrade(
    mut req: hyper::Request<hyper::body::Incoming>,
    event_tx: broadcast::Sender<WsEvent>,
) -> Option<hyper::Response<http_body_util::Full<hyper::body::Bytes>>> {
    use hyper::header::{
        CONNECTION, SEC_WEBSOCKET_ACCEPT, SEC_WEBSOCKET_KEY, SEC_WEBSOCKET_VERSION, UPGRADE,
    };
    let headers = req.headers();
    let wants_upgrade = headers
        .get(UPGRADE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.eq_ignore_ascii_case("websocket"));
    let version_ok = headers
        .get(SEC_WEBSOCKET_VERSION)
        .is_some_and(|v| v.as_bytes() == b"13");
    let key = headers.get(SEC_WEBSOCKET_KEY)?.as_bytes().to_vec();
    if !wants_upgrade || !version_ok {
        return None;
    }
    let accept = tokio_tungstenite::tungstenite::handshake::derive_accept_key(&key);
    let upgrade = hyper::upgrade::on(&mut req);

    tokio::spawn(async move {
        match upgrade.await {
            Ok(upgraded) => {
                let config = tokio_tungstenite::tungstenite::protocol::WebSocketConfig {
                    max_message_size: Some(MAX_WS_MESSAGE_BYTES),
                    max_frame_size: Some(MAX_WS_FRAME_BYTES),
                    ..Default::default()
                };
                let ws_stream = tokio_tungstenite::WebSocketStream::from_raw_socket(
                    hyper_util::rt::TokioIo::new(upgraded),
                    tokio_tungstenite::tungstenite::protocol::Role::Server,
                    Some(config),
                )
                .await;
                info!("websocket client connected");
                handle_ws_connection(ws_stream, event_tx).await;
                info!("websocket client disconnected");
            }
            Err(e) => warn!(error = %e, "websocket upgrade failed"),
        }
    });

    hyper::Response::builder()
        .status(hyper::StatusCode::SWITCHING_PROTOCOLS)
        .header(CONNECTION, "Upgrade")
        .header(UPGRADE, "websocket")
        .header(SEC_WEBSOCKET_ACCEPT, accept)
        .body(http_body_util::Full::new(hyper::body::Bytes::new()))
        .ok()
}

async fn handle_ws_connection<S>(
    ws_stream: tokio_tungstenite::WebSocketStream<S>,
    event_tx: broadcast::Sender<WsEvent>,
) where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let (mut ws_sink, mut ws_source) = ws_stream.split();
    let mut event_rx = event_tx.subscribe();
    let mut subscribed_topics: HashSet<WsTopic> = HashSet::new();

    // Send welcome message
    let welcome = serde_json::json!({
        "type": "welcome",
        "message": "Connected to NetChain WebSocket. Send {\"action\":\"subscribe\",\"topics\":[\"new_blocks\",\"new_transactions\",\"proposals\",\"slashing\"]} to subscribe."
    });
    if let Err(e) = ws_sink
        .send(tokio_tungstenite::tungstenite::Message::Text(
            welcome.to_string(),
        ))
        .await
    {
        warn!(error = %e, "failed to send welcome");
        return;
    }

    loop {
        tokio::select! {
            // Incoming client messages
            msg = ws_source.next() => {
                match msg {
                    Some(Ok(tokio_tungstenite::tungstenite::Message::Text(text))) => {
                        match serde_json::from_str::<ClientMessage>(&text) {
                            Ok(ClientMessage::Subscribe { topics }) => {
                                for topic in &topics {
                                    subscribed_topics.insert(topic.clone());
                                }
                                let response = ServerControlMessage::Subscribed {
                                    topics: subscribed_topics.iter().cloned().collect(),
                                };
                                let json = serde_json::to_string(&response).unwrap();
                                if ws_sink.send(tokio_tungstenite::tungstenite::Message::Text(json)).await.is_err() {
                                    break;
                                }
                            }
                            Ok(ClientMessage::Unsubscribe { topics }) => {
                                for topic in &topics {
                                    subscribed_topics.remove(topic);
                                }
                                let response = ServerControlMessage::Subscribed {
                                    topics: subscribed_topics.iter().cloned().collect(),
                                };
                                let json = serde_json::to_string(&response).unwrap();
                                if ws_sink.send(tokio_tungstenite::tungstenite::Message::Text(json)).await.is_err() {
                                    break;
                                }
                            }
                            Ok(ClientMessage::Ping) => {
                                let response = ServerControlMessage::Pong;
                                let json = serde_json::to_string(&response).unwrap();
                                if ws_sink.send(tokio_tungstenite::tungstenite::Message::Text(json)).await.is_err() {
                                    break;
                                }
                            }
                            Err(e) => {
                                let response = ServerControlMessage::Error {
                                    message: format!("Invalid message: {}", e),
                                };
                                let json = serde_json::to_string(&response).unwrap();
                                let _ = ws_sink.send(tokio_tungstenite::tungstenite::Message::Text(json)).await;
                            }
                        }
                    }
                    Some(Ok(tokio_tungstenite::tungstenite::Message::Close(_))) | None => {
                        break;
                    }
                    Some(Ok(tokio_tungstenite::tungstenite::Message::Ping(data))) => {
                        let _ = ws_sink.send(tokio_tungstenite::tungstenite::Message::Pong(data)).await;
                    }
                    Some(Ok(_)) => {
                        // Ignore other message types (binary, pong, frame)
                    }
                    Some(Err(e)) => {
                        warn!(error = %e, "websocket receive error");
                        break;
                    }
                }
            }

            // Broadcast events from the chain
            event = event_rx.recv() => {
                match event {
                    Ok(ws_event) => {
                        if subscribed_topics.contains(&ws_event.topic()) {
                            let json = serde_json::to_string(&ws_event).unwrap();
                            if ws_sink.send(tokio_tungstenite::tungstenite::Message::Text(json)).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!(skipped = n, "websocket client lagged, skipped events");
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }
        }
    }
}

/// Convert a chain state event into the WebSocket event broadcast to subscribers.
pub fn from_state_event(event: &StateEvent) -> WsEvent {
    match event {
        StateEvent::ProposalCreated {
            proposal_id,
            title,
            proposer: _,
        } => WsEvent::ProposalUpdate {
            proposal_id: *proposal_id,
            title: title.clone(),
            status: "Created".to_string(),
            yes_votes: 0,
            no_votes: 0,
        },
        StateEvent::VoteCast {
            proposal_id,
            title,
            voter: _,
            support: _,
            yes_votes,
            no_votes,
        } => WsEvent::ProposalUpdate {
            proposal_id: *proposal_id,
            title: title.clone(),
            status: "VoteReceived".to_string(),
            yes_votes: *yes_votes,
            no_votes: *no_votes,
        },
        StateEvent::ContractDeployed {
            contract_address,
            deployer,
        } => WsEvent::ContractDeployed {
            contract_address: contract_address.clone(),
            deployer: deployer.clone(),
            code_hash: String::new(),
        },
        StateEvent::ContractCalled {
            contract_address,
            caller,
            function,
            gas_used,
        } => WsEvent::ContractCalled {
            contract_address: contract_address.clone(),
            caller: caller.clone(),
            function: function.clone(),
            gas_used: *gas_used,
        },
        StateEvent::TokenCreated {
            token_id,
            creator,
            name,
            symbol,
        } => WsEvent::TokenCreated {
            token_id: token_id.clone(),
            creator: creator.clone(),
            name: name.clone(),
            symbol: symbol.clone(),
        },
        StateEvent::TokenMinted {
            token_id,
            to,
            amount,
        } => WsEvent::TokenMinted {
            token_id: token_id.clone(),
            to: to.clone(),
            amount: *amount,
        },
        StateEvent::TokenTransferred {
            token_id,
            from,
            to,
            amount,
        } => WsEvent::TokenTransferred {
            token_id: token_id.clone(),
            from: from.clone(),
            to: to.clone(),
            amount: *amount,
        },
        StateEvent::TokenBurned {
            token_id,
            from: _,
            amount,
        } => WsEvent::TokenBurned {
            token_id: token_id.clone(),
            amount: *amount,
        },
        StateEvent::NftCreated {
            nft_id,
            collection_id,
            owner,
        } => WsEvent::NftCreated {
            nft_id: nft_id.clone(),
            collection_id: collection_id.clone(),
            creator: owner.clone(),
            name: String::new(),
        },
        StateEvent::NftTransferred { nft_id, from, to } => WsEvent::NftTransferred {
            nft_id: nft_id.clone(),
            from: from.clone(),
            to: to.clone(),
        },
        StateEvent::NftBurned { nft_id, owner: _ } => WsEvent::NftBurned {
            nft_id: nft_id.clone(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_event_topics() {
        let block_event = WsEvent::NewBlock {
            index: 1,
            hash: "abc".to_string(),
            validator: "node1".to_string(),
            tx_count: 5,
            timestamp: "2025-01-01T00:00:00Z".to_string(),
        };
        assert_eq!(block_event.topic(), WsTopic::NewBlocks);

        let tx_event = WsEvent::NewTransaction {
            tx_hash: "def".to_string(),
            sender: "alice".to_string(),
            receiver: "bob".to_string(),
            amount: 100,
            tx_type: "Transfer".to_string(),
        };
        assert_eq!(tx_event.topic(), WsTopic::NewTransactions);

        let proposal_event = WsEvent::ProposalUpdate {
            proposal_id: 1,
            title: "Test".to_string(),
            status: "Active".to_string(),
            yes_votes: 10,
            no_votes: 5,
        };
        assert_eq!(proposal_event.topic(), WsTopic::Proposals);
    }

    #[test]
    fn test_ws_event_serialization() {
        let event = WsEvent::NewBlock {
            index: 42,
            hash: "0xabc123".to_string(),
            validator: "node_42".to_string(),
            tx_count: 3,
            timestamp: "2025-06-15T12:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"event\":\"new_block\""));
        assert!(json.contains("\"index\":42"));
    }

    #[test]
    fn test_client_message_deserialization() {
        let subscribe_json = r#"{"action":"subscribe","topics":["new_blocks","proposals"]}"#;
        let msg: ClientMessage = serde_json::from_str(subscribe_json).unwrap();
        match msg {
            ClientMessage::Subscribe { topics } => {
                assert_eq!(topics.len(), 2);
                assert!(topics.contains(&WsTopic::NewBlocks));
                assert!(topics.contains(&WsTopic::Proposals));
            }
            _ => panic!("Expected Subscribe"),
        }

        let ping_json = r#"{"action":"ping"}"#;
        let msg: ClientMessage = serde_json::from_str(ping_json).unwrap();
        assert!(matches!(msg, ClientMessage::Ping));
    }

    #[test]
    fn test_broadcast_channel() {
        let tx = create_event_channel();
        let mut rx1 = tx.subscribe();
        let mut rx2 = tx.subscribe();

        let event = WsEvent::NewBlock {
            index: 1,
            hash: "test".to_string(),
            validator: "v1".to_string(),
            tx_count: 0,
            timestamp: "now".to_string(),
        };

        // Send event
        tx.send(event.clone()).unwrap();

        // Both receivers get the event
        let e1 = rx1.try_recv().unwrap();
        let e2 = rx2.try_recv().unwrap();

        // Verify the events match
        let json1 = serde_json::to_string(&e1).unwrap();
        let json2 = serde_json::to_string(&e2).unwrap();
        assert_eq!(json1, json2);
    }
}
