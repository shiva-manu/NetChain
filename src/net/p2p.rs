// src/net/p2p.rs
//! libp2p networking.
//!
//! * **gossipsub** spreads new blocks and transactions. Topics are namespaced by the genesis
//!   hash so different networks never mix.
//! * **request-response** (JSON) carries direct peer requests: chain sync (blocks from a
//!   height) and bandwidth probes used for Proof-of-Internet measurements.
//! * **ping** continuously measures round-trip time to every connected peer.
//! * Peers are found through configured **bootnodes** (dialled at start and re-dialled while
//!   disconnected), **identify + Kademlia** (peers learn each other's listen addresses and
//!   connect directly, so the network becomes a mesh rather than a star around the bootnode)
//!   and, optionally, **mDNS** on the local network.
//!
//! The libp2p identity is the node key, so a peer's on-chain address can be derived from its
//! PeerId (`node_key::address_from_peer_id`).

use libp2p::futures::StreamExt;
use libp2p::{
    gossipsub::{self, IdentTopic as Topic, MessageAuthenticity},
    identify, identity, kad,
    mdns::{tokio::Behaviour as Mdns, Event as MdnsEvent},
    multiaddr::Protocol,
    noise, ping,
    request_response::{self, json, OutboundRequestId, ProtocolSupport, ResponseChannel},
    swarm::{behaviour::toggle::Toggle, dial_opts::DialOpts, NetworkBehaviour, Swarm, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, StreamProtocol, Transport,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tracing::{debug, info, warn};

use crate::block::Block;

/// Largest gossip message (a full block).
pub const MAX_GOSSIP_BYTES: usize = 4 * 1024 * 1024;
/// Largest request-response payload (a sync batch).
pub const MAX_RESPONSE_BYTES: u64 = 16 * 1024 * 1024;
/// Largest bandwidth probe a peer will serve.
pub const MAX_PROBE_BYTES: usize = 2 * 1024 * 1024;
/// Minimum time between probes served to the same peer.
const PROBE_COOLDOWN: Duration = Duration::from_secs(20);
/// RTT samples kept per peer.
const RTT_SAMPLES: usize = 20;
const REDIAL_INTERVAL: Duration = Duration::from_secs(20);
/// Stop dialling newly discovered peers above this many connections.
const MAX_DISCOVERY_PEERS: usize = 50;

/// Direct request to a single peer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetRequest {
    /// Send me your blocks starting at `from_height`.
    Sync { from_height: u64 },
    /// Send me `size` bytes so I can measure throughput to you.
    Probe { size: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetResponse {
    Blocks(Vec<Block>),
    Probe { payload: String },
    Refused(String),
}

/// Events delivered to the node.
#[derive(Debug)]
pub enum P2PEvent {
    Block {
        json: String,
        source: PeerId,
    },
    Transaction(String),
    SyncRequest {
        peer: PeerId,
        from_height: u64,
        channel: ResponseChannel<NetResponse>,
    },
    SyncBlocks {
        peer: PeerId,
        blocks: Vec<Block>,
    },
    ProbeResult {
        peer: PeerId,
        bytes: usize,
        elapsed: Duration,
    },
    PeerConnected(PeerId),
    PeerDisconnected(PeerId),
}

/// Commands from the node to the network task.
#[derive(Debug)]
pub enum P2PCommand {
    PublishBlock(String),
    PublishTransaction(String),
    /// Ask `peer` (or every connected peer when `None`) for blocks from `from_height`.
    RequestSync {
        peer: Option<PeerId>,
        from_height: u64,
    },
    RespondSync {
        channel: ResponseChannel<NetResponse>,
        blocks: Vec<Block>,
    },
    Probe {
        peer: PeerId,
        size: usize,
    },
}

/// Cheap handle used by the rest of the node to talk to the network task.
#[derive(Clone)]
pub struct P2PHandle {
    command_tx: UnboundedSender<P2PCommand>,
    shared_state: Arc<P2PSharedState>,
    local_peer_id: String,
}

impl P2PHandle {
    pub fn local_peer_id(&self) -> &str {
        &self.local_peer_id
    }

    pub fn shared_state(&self) -> Arc<P2PSharedState> {
        Arc::clone(&self.shared_state)
    }

    fn send(&self, command: P2PCommand) {
        let _ = self.command_tx.send(command);
    }

    pub fn publish_block(&self, block_json: String) {
        self.send(P2PCommand::PublishBlock(block_json));
    }

    pub fn publish_transaction(&self, tx_json: String) {
        self.send(P2PCommand::PublishTransaction(tx_json));
    }

    pub fn request_sync(&self, peer: Option<PeerId>, from_height: u64) {
        self.send(P2PCommand::RequestSync { peer, from_height });
    }

    pub fn respond_sync(&self, channel: ResponseChannel<NetResponse>, blocks: Vec<Block>) {
        self.send(P2PCommand::RespondSync { channel, blocks });
    }

    pub fn probe(&self, peer: PeerId, size: usize) {
        self.send(P2PCommand::Probe { peer, size });
    }
}

/// Network facts readable without talking to the network task.
#[derive(Debug, Default)]
pub struct P2PSharedState {
    pub peer_count: AtomicUsize,
    active_peers: Mutex<HashSet<PeerId>>,
    rtts: Mutex<HashMap<PeerId, VecDeque<Duration>>>,
    pub gossip_messages: AtomicU64,
}

impl P2PSharedState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_peer_count(&self) -> usize {
        self.peer_count.load(Ordering::Relaxed)
    }

    pub fn connected_peers(&self) -> Vec<PeerId> {
        lock(&self.active_peers).iter().copied().collect()
    }

    /// Median of recent ping RTTs to `peer`.
    pub fn median_rtt(&self, peer: &PeerId) -> Option<Duration> {
        let rtts = lock(&self.rtts);
        let samples = rtts.get(peer)?;
        if samples.is_empty() {
            return None;
        }
        let mut sorted: Vec<Duration> = samples.iter().copied().collect();
        sorted.sort();
        Some(sorted[sorted.len() / 2])
    }

    fn register_peer(&self, peer: &PeerId) -> bool {
        let mut peers = lock(&self.active_peers);
        if peers.insert(*peer) {
            self.peer_count.fetch_add(1, Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    fn unregister_peer(&self, peer: &PeerId) -> bool {
        let mut peers = lock(&self.active_peers);
        if peers.remove(peer) {
            self.peer_count.fetch_sub(1, Ordering::Relaxed);
            lock(&self.rtts).remove(peer);
            true
        } else {
            false
        }
    }

    fn record_rtt(&self, peer: &PeerId, rtt: Duration) {
        let mut rtts = lock(&self.rtts);
        let samples = rtts.entry(*peer).or_default();
        if samples.len() >= RTT_SAMPLES {
            samples.pop_front();
        }
        samples.push_back(rtt);
    }
}

/// Skip unspecified listen addresses (0.0.0.0 / ::) that peers cannot dial.
fn is_dialable(addr: &Multiaddr) -> bool {
    !addr.iter().any(|p| match p {
        Protocol::Ip4(ip) => ip.is_unspecified(),
        Protocol::Ip6(ip) => ip.is_unspecified(),
        _ => false,
    })
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "OutEvent")]
pub struct NetBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: Toggle<Mdns>,
    pub ping: ping::Behaviour,
    pub reqres: json::Behaviour<NetRequest, NetResponse>,
    pub identify: identify::Behaviour,
    pub kad: kad::Behaviour<kad::store::MemoryStore>,
}

#[derive(Debug)]
pub enum OutEvent {
    Gossip(gossipsub::Event),
    Mdns(MdnsEvent),
    Ping(ping::Event),
    ReqRes(request_response::Event<NetRequest, NetResponse>),
    Identify(Box<identify::Event>),
    Kad(kad::Event),
}

impl From<identify::Event> for OutEvent {
    fn from(e: identify::Event) -> Self {
        OutEvent::Identify(Box::new(e))
    }
}
impl From<kad::Event> for OutEvent {
    fn from(e: kad::Event) -> Self {
        OutEvent::Kad(e)
    }
}

impl From<gossipsub::Event> for OutEvent {
    fn from(e: gossipsub::Event) -> Self {
        OutEvent::Gossip(e)
    }
}
impl From<MdnsEvent> for OutEvent {
    fn from(e: MdnsEvent) -> Self {
        OutEvent::Mdns(e)
    }
}
impl From<ping::Event> for OutEvent {
    fn from(e: ping::Event) -> Self {
        OutEvent::Ping(e)
    }
}
impl From<request_response::Event<NetRequest, NetResponse>> for OutEvent {
    fn from(e: request_response::Event<NetRequest, NetResponse>) -> Self {
        OutEvent::ReqRes(e)
    }
}

pub struct P2PConfig {
    pub port: u16,
    pub keypair: identity::Keypair,
    pub bootnodes: Vec<Multiaddr>,
    pub enable_mdns: bool,
    /// Genesis hash; used to namespace gossip topics.
    pub network_id: String,
}

pub struct P2PService {
    pub peer_id: PeerId,
    swarm: Swarm<NetBehaviour>,
    block_topic: Topic,
    tx_topic: Topic,
    shared_state: Arc<P2PSharedState>,
    bootnodes: Vec<Multiaddr>,
    pending_sync: HashSet<OutboundRequestId>,
    pending_probes: HashMap<OutboundRequestId, (Instant, usize)>,
    served_probes: HashMap<PeerId, Instant>,
}

impl P2PService {
    pub fn new(
        config: P2PConfig,
    ) -> anyhow::Result<(Self, P2PHandle, UnboundedReceiver<P2PCommand>)> {
        let local_key = config.keypair;
        let peer_id = PeerId::from(local_key.public());

        let transport = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
            .upgrade(libp2p::core::upgrade::Version::V1)
            .authenticate(noise::Config::new(&local_key)?)
            .multiplex(yamux::Config::default())
            .boxed();

        let gossip_config = gossipsub::ConfigBuilder::default()
            .max_transmit_size(MAX_GOSSIP_BYTES)
            .heartbeat_interval(Duration::from_secs(1))
            .build()
            .map_err(|e| anyhow::anyhow!("gossipsub config: {e}"))?;
        let mut gossipsub = gossipsub::Behaviour::new(
            MessageAuthenticity::Signed(local_key.clone()),
            gossip_config,
        )
        .map_err(|e| anyhow::anyhow!(e))?;

        let ns = &config.network_id[..config.network_id.len().min(16)];
        let block_topic = Topic::new(format!("netchain/{ns}/blocks"));
        let tx_topic = Topic::new(format!("netchain/{ns}/transactions"));
        gossipsub.subscribe(&block_topic)?;
        gossipsub.subscribe(&tx_topic)?;

        let mdns = if config.enable_mdns {
            Toggle::from(Some(Mdns::new(Default::default(), peer_id)?))
        } else {
            Toggle::from(None)
        };

        let codec = json::codec::Codec::<NetRequest, NetResponse>::default()
            .set_request_size_maximum(64 * 1024)
            .set_response_size_maximum(MAX_RESPONSE_BYTES);
        let reqres = request_response::Behaviour::with_codec(
            codec,
            [(
                StreamProtocol::new("/netchain/reqres/1.0.0"),
                ProtocolSupport::Full,
            )],
            request_response::Config::default().with_request_timeout(Duration::from_secs(30)),
        );

        let ping = ping::Behaviour::new(ping::Config::new().with_interval(Duration::from_secs(10)));

        let identify = identify::Behaviour::new(identify::Config::new(
            "/netchain/1.0.0".to_string(),
            local_key.public(),
        ));
        let mut kad = kad::Behaviour::with_config(
            peer_id,
            kad::store::MemoryStore::new(peer_id),
            kad::Config::new(StreamProtocol::new("/netchain/kad/1.0.0")),
        );
        // Answer DHT queries even without a confirmed public address (private testnets).
        kad.set_mode(Some(kad::Mode::Server));

        let behaviour = NetBehaviour {
            gossipsub,
            mdns,
            ping,
            reqres,
            identify,
            kad,
        };

        let mut swarm = Swarm::new(
            transport,
            behaviour,
            peer_id,
            libp2p::swarm::Config::with_tokio_executor()
                .with_idle_connection_timeout(Duration::from_secs(3600)),
        );

        swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{}", config.port).parse()?)?;

        let shared_state = Arc::new(P2PSharedState::new());
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let handle = P2PHandle {
            command_tx,
            shared_state: Arc::clone(&shared_state),
            local_peer_id: peer_id.to_string(),
        };

        let service = Self {
            peer_id,
            swarm,
            block_topic,
            tx_topic,
            shared_state,
            bootnodes: config.bootnodes,
            pending_sync: HashSet::new(),
            pending_probes: HashMap::new(),
            served_probes: HashMap::new(),
        };

        Ok((service, handle, command_rx))
    }

    fn dial_bootnodes(&mut self) {
        if !self.bootnodes.is_empty() && self.shared_state.get_peer_count() < self.bootnodes.len() {
            for addr in self.bootnodes.clone() {
                if let Err(e) = self.swarm.dial(addr.clone()) {
                    debug!(%addr, error = %e, "bootnode dial failed");
                }
            }
        }
    }

    pub async fn run(
        &mut self,
        event_sender: mpsc::Sender<P2PEvent>,
        mut command_rx: UnboundedReceiver<P2PCommand>,
    ) {
        self.dial_bootnodes();
        let mut redial = tokio::time::interval(REDIAL_INTERVAL);

        loop {
            tokio::select! {
                swarm_event = self.swarm.select_next_some() => {
                    if let Some(event) = self.handle_swarm_event(swarm_event) {
                        if event_sender.send(event).await.is_err() {
                            return;
                        }
                    }
                }
                Some(command) = command_rx.recv() => {
                    self.handle_command(command);
                }
                _ = redial.tick() => {
                    self.dial_bootnodes();
                    if self.shared_state.get_peer_count() > 0 {
                        let _ = self.swarm.behaviour_mut().kad.bootstrap();
                    }
                }
            }
        }
    }

    fn handle_swarm_event(&mut self, event: SwarmEvent<OutEvent>) -> Option<P2PEvent> {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                info!(%address, peer_id = %self.peer_id, "p2p listening");
                None
            }
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                self.swarm
                    .behaviour_mut()
                    .gossipsub
                    .add_explicit_peer(&peer_id);
                if self.shared_state.register_peer(&peer_id) {
                    info!(peer = %peer_id, addr = %endpoint.get_remote_address(), "peer connected");
                    return Some(P2PEvent::PeerConnected(peer_id));
                }
                None
            }
            SwarmEvent::ConnectionClosed {
                peer_id,
                num_established,
                ..
            } => {
                if num_established == 0 && self.shared_state.unregister_peer(&peer_id) {
                    self.swarm
                        .behaviour_mut()
                        .gossipsub
                        .remove_explicit_peer(&peer_id);
                    info!(peer = %peer_id, "peer disconnected");
                    return Some(P2PEvent::PeerDisconnected(peer_id));
                }
                None
            }
            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                debug!(?peer_id, %error, "outgoing connection failed");
                None
            }
            SwarmEvent::Behaviour(OutEvent::Gossip(gossipsub::Event::Message {
                propagation_source,
                message,
                ..
            })) => {
                self.shared_state
                    .gossip_messages
                    .fetch_add(1, Ordering::Relaxed);
                let data = String::from_utf8(message.data).ok()?;
                if message.topic == self.block_topic.hash() {
                    Some(P2PEvent::Block {
                        json: data,
                        source: propagation_source,
                    })
                } else if message.topic == self.tx_topic.hash() {
                    Some(P2PEvent::Transaction(data))
                } else {
                    None
                }
            }
            SwarmEvent::Behaviour(OutEvent::Mdns(MdnsEvent::Discovered(list))) => {
                for (peer, addr) in list {
                    if !self.swarm.is_connected(&peer) {
                        let _ = self.swarm.dial(addr);
                    }
                }
                None
            }
            SwarmEvent::Behaviour(OutEvent::Ping(ping::Event {
                peer,
                result: Ok(rtt),
                ..
            })) => {
                self.shared_state.record_rtt(&peer, rtt);
                None
            }
            SwarmEvent::Behaviour(OutEvent::ReqRes(event)) => self.handle_reqres(event),
            SwarmEvent::Behaviour(OutEvent::Identify(event)) => {
                if let identify::Event::Received { peer_id, info, .. } = *event {
                    for addr in info.listen_addrs {
                        if is_dialable(&addr) {
                            self.swarm.behaviour_mut().kad.add_address(&peer_id, addr);
                        }
                    }
                }
                None
            }
            SwarmEvent::Behaviour(OutEvent::Kad(kad::Event::RoutingUpdated {
                peer,
                addresses,
                ..
            })) => {
                if peer != self.peer_id
                    && !self.swarm.is_connected(&peer)
                    && self.shared_state.get_peer_count() < MAX_DISCOVERY_PEERS
                {
                    debug!(%peer, "dialling peer discovered via DHT");
                    let opts = DialOpts::peer_id(peer)
                        .addresses(addresses.into_vec())
                        .build();
                    let _ = self.swarm.dial(opts);
                }
                None
            }
            _ => None,
        }
    }

    fn handle_reqres(
        &mut self,
        event: request_response::Event<NetRequest, NetResponse>,
    ) -> Option<P2PEvent> {
        match event {
            request_response::Event::Message { peer, message, .. } => match message {
                request_response::Message::Request {
                    request, channel, ..
                } => match request {
                    NetRequest::Sync { from_height } => Some(P2PEvent::SyncRequest {
                        peer,
                        from_height,
                        channel,
                    }),
                    NetRequest::Probe { size } => {
                        let now = Instant::now();
                        let allowed = self
                            .served_probes
                            .get(&peer)
                            .map(|last| now.duration_since(*last) >= PROBE_COOLDOWN)
                            .unwrap_or(true);
                        let response = if !allowed {
                            NetResponse::Refused("probe rate limited".into())
                        } else {
                            self.served_probes.insert(peer, now);
                            NetResponse::Probe {
                                payload: "n".repeat(size.min(MAX_PROBE_BYTES)),
                            }
                        };
                        let _ = self
                            .swarm
                            .behaviour_mut()
                            .reqres
                            .send_response(channel, response);
                        None
                    }
                },
                request_response::Message::Response {
                    request_id,
                    response,
                } => {
                    if self.pending_sync.remove(&request_id) {
                        if let NetResponse::Blocks(blocks) = response {
                            return Some(P2PEvent::SyncBlocks { peer, blocks });
                        }
                        return None;
                    }
                    if let Some((started, size)) = self.pending_probes.remove(&request_id) {
                        if let NetResponse::Probe { payload } = response {
                            if payload.len() == size {
                                return Some(P2PEvent::ProbeResult {
                                    peer,
                                    bytes: size,
                                    elapsed: started.elapsed(),
                                });
                            }
                        }
                    }
                    None
                }
            },
            request_response::Event::OutboundFailure {
                peer,
                request_id,
                error,
                ..
            } => {
                self.pending_sync.remove(&request_id);
                self.pending_probes.remove(&request_id);
                debug!(%peer, %error, "request failed");
                None
            }
            _ => None,
        }
    }

    fn publish(&mut self, topic: Topic, data: Vec<u8>) {
        if let Err(e) = self.swarm.behaviour_mut().gossipsub.publish(topic, data) {
            // InsufficientPeers is normal for a node without peers.
            debug!(error = %e, "gossip publish failed");
        }
    }

    fn handle_command(&mut self, command: P2PCommand) {
        match command {
            P2PCommand::PublishBlock(json) => {
                let topic = self.block_topic.clone();
                self.publish(topic, json.into_bytes());
            }
            P2PCommand::PublishTransaction(json) => {
                let topic = self.tx_topic.clone();
                self.publish(topic, json.into_bytes());
            }
            P2PCommand::RequestSync { peer, from_height } => {
                let targets = match peer {
                    Some(p) => vec![p],
                    None => self.shared_state.connected_peers(),
                };
                for target in targets {
                    let id = self
                        .swarm
                        .behaviour_mut()
                        .reqres
                        .send_request(&target, NetRequest::Sync { from_height });
                    self.pending_sync.insert(id);
                }
            }
            P2PCommand::RespondSync { channel, blocks } => {
                if self
                    .swarm
                    .behaviour_mut()
                    .reqres
                    .send_response(channel, NetResponse::Blocks(blocks))
                    .is_err()
                {
                    warn!("failed to send sync response (peer gone)");
                }
            }
            P2PCommand::Probe { peer, size } => {
                let size = size.min(MAX_PROBE_BYTES);
                let id = self
                    .swarm
                    .behaviour_mut()
                    .reqres
                    .send_request(&peer, NetRequest::Probe { size });
                self.pending_probes.insert(id, (Instant::now(), size));
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

        state.record_rtt(&peer, Duration::from_millis(30));
        state.record_rtt(&peer, Duration::from_millis(10));
        state.record_rtt(&peer, Duration::from_millis(20));
        assert_eq!(state.median_rtt(&peer), Some(Duration::from_millis(20)));

        assert!(state.unregister_peer(&peer));
        assert!(!state.unregister_peer(&peer));
        assert_eq!(state.get_peer_count(), 0);
        assert_eq!(state.median_rtt(&peer), None);
    }
}
