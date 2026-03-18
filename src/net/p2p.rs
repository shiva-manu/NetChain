// src/net/p2p.rs

use libp2p::futures::StreamExt;

use libp2p::{
    gossipsub::{
        Behaviour as Gossipsub, Config as GossipsubConfig, Event as GossipsubEvent,
        IdentTopic as Topic, MessageAuthenticity,
    },
    identity,
    mdns::{tokio::Behaviour as Mdns, Event as MdnsEvent},
    noise,
    swarm::{Swarm, SwarmEvent},
    tcp, yamux, PeerId, Transport,
};

use libp2p::swarm::derive_prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{self, UnboundedReceiver};

/// Network message types for gossip protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    Block(String),
    Transaction(String),
    /// Chain sync: request blocks from height
    ChainSyncRequest {
        from_height: u64,
    },
    /// Chain sync: response with blocks
    ChainSyncResponse {
        blocks: Vec<String>,
    },

    // ===== Proof of Internet Metric Messages =====
    /// Challenge a peer to prove their internet metrics
    /// The challenger generates a random nonce and requests the peer to prove their bandwidth
    MetricChallenge {
        /// Who issued the challenge
        challenger_id: String,
        /// Who is being challenged
        target_id: String,
        /// Random nonce for this challenge (prevents replay)
        challenge_nonce: String,
        /// Bytes the target should download from challenger (for upload speed proof)
        bytes_to_download: usize,
        /// Timestamp when challenge was issued
        timestamp: u64,
    },

    /// Response to a metric challenge with measured values
    MetricChallengeResponse {
        /// Original challenge nonce
        challenge_nonce: String,
        /// Who responded
        responder_id: String,
        /// Measured download speed (Mbps) during challenge
        download_mbps: f64,
        /// Measured upload speed (Mbps) during challenge
        upload_mbps: f64,
        /// Measured latency to challenger (ms)
        latency_ms: f64,
        /// Bytes actually transferred
        bytes_transferred: usize,
        /// Duration of the test (ms)
        duration_ms: u64,
        /// Timestamp of response
        timestamp: u64,
    },

    /// Attestation from a peer vouching for another peer's metrics
    /// Peers attest to metrics they've personally verified through challenges
    MetricAttestation {
        /// Who is attesting (the verifier)
        attester_id: String,
        /// Who is being attested (the subject)
        subject_id: String,
        /// Attested download speed (Mbps)
        download_mbps: f64,
        /// Attested upload speed (Mbps)
        upload_mbps: f64,
        /// Attested latency (ms)
        latency_ms: f64,
        /// Confidence score (0.0-1.0) based on verification quality
        confidence: f64,
        /// Timestamp of attestation
        timestamp: u64,
        /// Signature over the attestation data (hex-encoded)
        signature: String,
    },

    /// Broadcast self-reported metrics (to be verified by peers)
    MetricAnnouncement {
        /// Node announcing its metrics
        node_id: String,
        /// Claimed download speed (Mbps)
        download_mbps: f64,
        /// Claimed upload speed (Mbps)
        upload_mbps: f64,
        /// Claimed average latency (ms)
        latency_ms: f64,
        /// Claimed uptime percentage
        uptime_percent: f64,
        /// Claimed stability percentage
        stability_percent: f64,
        /// Timestamp
        timestamp: u64,
        /// How many peer attestations this node has received
        attestation_count: usize,
    },
}

/// Commands that can be sent to the P2P service
#[derive(Debug)]
pub enum P2PCommand {
    PublishBlock(String),
    PublishTransaction(String),
    RequestChainSync(u64),
    SendChainSyncResponse(Vec<String>),
    SendMetricChallenge {
        target_id: String,
        challenge_nonce: String,
        bytes_to_download: usize,
    },
    SendMetricResponse {
        challenge_nonce: String,
        download_mbps: f64,
        upload_mbps: f64,
        latency_ms: f64,
        bytes_transferred: usize,
        duration_ms: u64,
    },
    SendMetricAttestation {
        subject_id: String,
        download_mbps: f64,
        upload_mbps: f64,
        latency_ms: f64,
        confidence: f64,
        signature: String,
    },
    AnnounceMetrics {
        download_mbps: f64,
        upload_mbps: f64,
        latency_ms: f64,
        uptime_percent: f64,
        stability_percent: f64,
        attestation_count: usize,
    },
}

/// Handle for sending commands to the P2P service without holding a lock
#[derive(Clone)]
pub struct P2PHandle {
    command_tx: mpsc::UnboundedSender<P2PCommand>,
    shared_state: Arc<P2PSharedState>,
    local_peer_id: String,
}

impl P2PHandle {
    pub fn new(
        command_tx: mpsc::UnboundedSender<P2PCommand>,
        shared_state: Arc<P2PSharedState>,
        local_peer_id: String,
    ) -> Self {
        Self {
            command_tx,
            shared_state,
            local_peer_id,
        }
    }

    pub fn local_peer_id(&self) -> &str {
        &self.local_peer_id
    }

    pub fn shared_state(&self) -> Arc<P2PSharedState> {
        Arc::clone(&self.shared_state)
    }

    pub fn publish_block(&self, block_json: String) {
        let _ = self.command_tx.send(P2PCommand::PublishBlock(block_json));
    }

    pub fn publish_transaction(&self, tx_json: String) {
        let _ = self
            .command_tx
            .send(P2PCommand::PublishTransaction(tx_json));
    }

    pub fn request_chain_sync(&self, from_height: u64) {
        let _ = self
            .command_tx
            .send(P2PCommand::RequestChainSync(from_height));
    }

    pub fn send_chain_sync_response(&self, blocks: Vec<String>) {
        let _ = self
            .command_tx
            .send(P2PCommand::SendChainSyncResponse(blocks));
    }

    pub fn send_metric_challenge(
        &self,
        target_id: String,
        challenge_nonce: String,
        bytes_to_download: usize,
    ) {
        let _ = self.command_tx.send(P2PCommand::SendMetricChallenge {
            target_id,
            challenge_nonce,
            bytes_to_download,
        });
    }

    pub fn send_metric_response(
        &self,
        challenge_nonce: String,
        download_mbps: f64,
        upload_mbps: f64,
        latency_ms: f64,
        bytes_transferred: usize,
        duration_ms: u64,
    ) {
        let _ = self.command_tx.send(P2PCommand::SendMetricResponse {
            challenge_nonce,
            download_mbps,
            upload_mbps,
            latency_ms,
            bytes_transferred,
            duration_ms,
        });
    }

    pub fn send_metric_attestation(
        &self,
        subject_id: String,
        download_mbps: f64,
        upload_mbps: f64,
        latency_ms: f64,
        confidence: f64,
        signature: String,
    ) {
        let _ = self.command_tx.send(P2PCommand::SendMetricAttestation {
            subject_id,
            download_mbps,
            upload_mbps,
            latency_ms,
            confidence,
            signature,
        });
    }

    pub fn announce_metrics(
        &self,
        download_mbps: f64,
        upload_mbps: f64,
        latency_ms: f64,
        uptime_percent: f64,
        stability_percent: f64,
        attestation_count: usize,
    ) {
        let _ = self.command_tx.send(P2PCommand::AnnounceMetrics {
            download_mbps,
            upload_mbps,
            latency_ms,
            uptime_percent,
            stability_percent,
            attestation_count,
        });
    }
}

#[derive(Debug)]
pub enum P2PEvent {
    Message(NetworkMessage),
    PeerConnected(PeerId),
    PeerDisconnected(PeerId),
}

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "OutEvent")]
pub struct NetBehaviour {
    pub gossipsub: Gossipsub,
    pub mdns: Mdns,
}

#[derive(Debug)]
pub enum OutEvent {
    Gossip(GossipsubEvent),
    Mdns(MdnsEvent),
}

impl From<GossipsubEvent> for OutEvent {
    fn from(e: GossipsubEvent) -> Self {
        OutEvent::Gossip(e)
    }
}

impl From<MdnsEvent> for OutEvent {
    fn from(e: MdnsEvent) -> Self {
        OutEvent::Mdns(e)
    }
}

/// Shared state that can be read without locking the P2P service.
/// This is updated by the P2P runner and read by monitoring/health endpoints.
#[derive(Debug, Default)]
pub struct P2PSharedState {
    pub peer_count: AtomicUsize,
    active_peers: Mutex<HashSet<String>>,
}

impl P2PSharedState {
    pub fn new() -> Self {
        Self {
            peer_count: AtomicUsize::new(0),
            active_peers: Mutex::new(HashSet::new()),
        }
    }

    pub fn get_peer_count(&self) -> usize {
        self.peer_count.load(Ordering::Relaxed)
    }

    fn register_peer(&self, peer: &PeerId) -> bool {
        let mut peers = self
            .active_peers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if peers.insert(peer.to_string()) {
            self.peer_count.fetch_add(1, Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    fn unregister_peer(&self, peer: &PeerId) -> bool {
        let mut peers = self
            .active_peers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if peers.remove(&peer.to_string()) {
            self.peer_count.fetch_sub(1, Ordering::Relaxed);
            true
        } else {
            false
        }
    }
}

pub struct P2PService {
    pub peer_id: PeerId,
    pub swarm: Swarm<NetBehaviour>,
    block_topic: Topic,
    tx_topic: Topic,
    sync_topic: Topic,
    metrics_topic: Topic,
    shared_state: Arc<P2PSharedState>,
}

impl P2PService {
    pub async fn new(
        port: u16,
    ) -> anyhow::Result<(Self, P2PHandle, UnboundedReceiver<P2PCommand>)> {
        let local_key = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(local_key.public());

        let transport = tcp::tokio::Transport::new(tcp::Config::default())
            .upgrade(libp2p::core::upgrade::Version::V1)
            .authenticate(noise::Config::new(&local_key)?)
            .multiplex(yamux::Config::default())
            .boxed();

        let mut gossipsub = Gossipsub::new(
            MessageAuthenticity::Signed(local_key),
            GossipsubConfig::default(),
        )
        .map_err(|e| anyhow::anyhow!(e))?;

        let block_topic = Topic::new("blocks");
        let tx_topic = Topic::new("transactions");
        let sync_topic = Topic::new("sync");
        let metrics_topic = Topic::new("metrics");

        gossipsub.subscribe(&block_topic)?;
        gossipsub.subscribe(&tx_topic)?;
        gossipsub.subscribe(&sync_topic)?;
        gossipsub.subscribe(&metrics_topic)?;

        let mdns = Mdns::new(Default::default(), peer_id)?;

        let behaviour = NetBehaviour { gossipsub, mdns };

        let mut swarm = Swarm::new(
            transport,
            behaviour,
            peer_id,
            libp2p::swarm::Config::with_tokio_executor(),
        );

        swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{port}").parse()?)?;

        let shared_state = Arc::new(P2PSharedState::new());
        let (command_tx, command_rx) = mpsc::unbounded_channel();

        let handle = P2PHandle::new(
            command_tx.clone(),
            Arc::clone(&shared_state),
            peer_id.to_string(),
        );

        let service = Self {
            peer_id,
            swarm,
            block_topic,
            tx_topic,
            sync_topic,
            metrics_topic,
            shared_state,
        };

        Ok((service, handle, command_rx))
    }

    /// Get a clone of the shared state Arc for use by monitoring endpoints.
    /// This allows reading peer count without locking the P2P service.
    pub fn shared_state(&self) -> Arc<P2PSharedState> {
        Arc::clone(&self.shared_state)
    }

    pub async fn run(
        &mut self,
        event_sender: mpsc::Sender<P2PEvent>,
        mut command_rx: mpsc::UnboundedReceiver<P2PCommand>,
    ) {
        loop {
            tokio::select! {
                swarm_event = self.swarm.select_next_some() => {
                    match swarm_event {
                        SwarmEvent::Behaviour(OutEvent::Gossip(GossipsubEvent::Message {
                            message,
                            ..
                        })) => {
                            let topic = message.topic.as_str();
                            let data = String::from_utf8_lossy(&message.data).to_string();

                            let network_msg = match topic {
                                "blocks" => NetworkMessage::Block(data),
                                "transactions" => NetworkMessage::Transaction(data),
                                "sync" | "metrics" => {
                                    // Try to parse as structured message (sync or metrics)
                                    if let Ok(msg) = serde_json::from_str::<NetworkMessage>(&data) {
                                        msg
                                    } else {
                                        continue;
                                    }
                                }
                                _ => continue,
                            };

                            let _ = event_sender.send(P2PEvent::Message(network_msg)).await;
                        }

                        SwarmEvent::Behaviour(OutEvent::Mdns(event)) => match event {
                            MdnsEvent::Discovered(list) => {
                                for (peer, _) in list {
                                    self.swarm
                                        .behaviour_mut()
                                        .gossipsub
                                        .add_explicit_peer(&peer);
                                    if self.shared_state.register_peer(&peer) {
                                        let _ =
                                            event_sender.send(P2PEvent::PeerConnected(peer)).await;
                                    }
                                }
                            }
                            MdnsEvent::Expired(list) => {
                                for (peer, _) in list {
                                    self.swarm
                                        .behaviour_mut()
                                        .gossipsub
                                        .remove_explicit_peer(&peer);
                                    if self.shared_state.unregister_peer(&peer) {
                                        let _ = event_sender
                                            .send(P2PEvent::PeerDisconnected(peer))
                                            .await;
                                    }
                                }
                            }
                        },

                        _ => {}
                    }
                }

                Some(command) = command_rx.recv() => {
                    self.handle_command(command);
                }
            }
        }
    }

    fn handle_command(&mut self, command: P2PCommand) {
        match command {
            P2PCommand::PublishBlock(block_json) => {
                let _ = self
                    .swarm
                    .behaviour_mut()
                    .gossipsub
                    .publish(self.block_topic.clone(), block_json.as_bytes());
            }
            P2PCommand::PublishTransaction(tx_json) => {
                let _ = self
                    .swarm
                    .behaviour_mut()
                    .gossipsub
                    .publish(self.tx_topic.clone(), tx_json.as_bytes());
            }
            P2PCommand::RequestChainSync(from_height) => {
                let msg = NetworkMessage::ChainSyncRequest { from_height };
                if let Ok(json) = serde_json::to_string(&msg) {
                    let _ = self
                        .swarm
                        .behaviour_mut()
                        .gossipsub
                        .publish(self.sync_topic.clone(), json.as_bytes());
                }
            }
            P2PCommand::SendChainSyncResponse(blocks) => {
                let msg = NetworkMessage::ChainSyncResponse { blocks };
                if let Ok(json) = serde_json::to_string(&msg) {
                    let _ = self
                        .swarm
                        .behaviour_mut()
                        .gossipsub
                        .publish(self.sync_topic.clone(), json.as_bytes());
                }
            }
            P2PCommand::SendMetricChallenge {
                target_id,
                challenge_nonce,
                bytes_to_download,
            } => {
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                let msg = NetworkMessage::MetricChallenge {
                    challenger_id: self.peer_id.to_string(),
                    target_id,
                    challenge_nonce,
                    bytes_to_download,
                    timestamp,
                };

                if let Ok(json) = serde_json::to_string(&msg) {
                    let _ = self
                        .swarm
                        .behaviour_mut()
                        .gossipsub
                        .publish(self.metrics_topic.clone(), json.as_bytes());
                }
            }
            P2PCommand::SendMetricResponse {
                challenge_nonce,
                download_mbps,
                upload_mbps,
                latency_ms,
                bytes_transferred,
                duration_ms,
            } => {
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                let msg = NetworkMessage::MetricChallengeResponse {
                    challenge_nonce,
                    responder_id: self.peer_id.to_string(),
                    download_mbps,
                    upload_mbps,
                    latency_ms,
                    bytes_transferred,
                    duration_ms,
                    timestamp,
                };

                if let Ok(json) = serde_json::to_string(&msg) {
                    let _ = self
                        .swarm
                        .behaviour_mut()
                        .gossipsub
                        .publish(self.metrics_topic.clone(), json.as_bytes());
                }
            }
            P2PCommand::SendMetricAttestation {
                subject_id,
                download_mbps,
                upload_mbps,
                latency_ms,
                confidence,
                signature,
            } => {
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                let msg = NetworkMessage::MetricAttestation {
                    attester_id: self.peer_id.to_string(),
                    subject_id,
                    download_mbps,
                    upload_mbps,
                    latency_ms,
                    confidence,
                    timestamp,
                    signature,
                };

                if let Ok(json) = serde_json::to_string(&msg) {
                    let _ = self
                        .swarm
                        .behaviour_mut()
                        .gossipsub
                        .publish(self.metrics_topic.clone(), json.as_bytes());
                }
            }
            P2PCommand::AnnounceMetrics {
                download_mbps,
                upload_mbps,
                latency_ms,
                uptime_percent,
                stability_percent,
                attestation_count,
            } => {
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                let msg = NetworkMessage::MetricAnnouncement {
                    node_id: self.peer_id.to_string(),
                    download_mbps,
                    upload_mbps,
                    latency_ms,
                    uptime_percent,
                    stability_percent,
                    timestamp,
                    attestation_count,
                };

                if let Ok(json) = serde_json::to_string(&msg) {
                    let _ = self
                        .swarm
                        .behaviour_mut()
                        .gossipsub
                        .publish(self.metrics_topic.clone(), json.as_bytes());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_state_deduplicates_peers() {
        let state = P2PSharedState::new();
        let peer = PeerId::from(identity::Keypair::generate_ed25519().public());

        assert!(state.register_peer(&peer));
        assert!(!state.register_peer(&peer));
        assert_eq!(state.get_peer_count(), 1);

        assert!(state.unregister_peer(&peer));
        assert!(!state.unregister_peer(&peer));
        assert_eq!(state.get_peer_count(), 0);
    }
}
