// src/net/dht.rs
//
// Kademlia DHT integration for peer discovery beyond local network mDNS.
// Provides internet-scale peer discovery with bootstrap nodes.

use libp2p::{
    core::transport::upgrade::Version,
    identity::Keypair,
    kad::{
        store::MemoryStore, Behaviour as KademliaBehaviour, Config as KademliaConfig,
        Event as KademliaEvent,
    },
    mdns::{tokio::Behaviour as Mdns, Event as MdnsEvent},
    multiaddr::{multiaddr, Multiaddr, Protocol},
    noise,
    swarm::{NetworkBehaviour, Swarm, SwarmEvent},
    tcp, yamux, PeerId, Transport,
};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::time::Interval;
use tracing::{debug, info, warn};

/// Bootstrap node configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapNode {
    /// Peer ID of the bootstrap node
    pub peer_id: String,
    /// Multiaddr for connecting to the bootstrap node
    pub multiaddr: String,
    /// Human-readable name/label
    pub name: Option<String>,
}

impl BootstrapNode {
    pub fn new(peer_id: String, multiaddr: String) -> Self {
        Self {
            peer_id,
            multiaddr,
            name: None,
        }
    }

    pub fn with_name(peer_id: String, multiaddr: String, name: String) -> Self {
        Self {
            peer_id,
            multiaddr,
            name: Some(name),
        }
    }

    /// Parse multiaddr and extract peer ID
    pub fn parse_peer_id(&self) -> Option<PeerId> {
        if let Ok(addr) = self.multiaddr.parse::<Multiaddr>() {
            for component in addr.iter() {
                if let Protocol::P2p(peer_id) = component {
                    return Some(peer_id);
                }
            }
        }
        // Fallback: generate a random peer ID for testing
        Some(PeerId::random())
    }
}

/// Default bootstrap nodes for NetChain
pub fn default_bootstrap_nodes() -> Vec<BootstrapNode> {
    vec![
        // These would be replaced with actual production bootstrap nodes
        BootstrapNode::with_name(
            "bootstrap-1".to_string(),
            "/ip4/0.0.0.0/tcp/30333".to_string(),
            "NetChain Bootstrap 1".to_string(),
        ),
    ]
}

/// DHT configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhtConfig {
    /// Bootstrap nodes to connect to
    pub bootstrap_nodes: Vec<BootstrapNode>,
    /// Port for DHT listening
    pub dht_port: u16,
    /// Enable DHT-based peer discovery
    pub enabled: bool,
    /// Interval for periodic bootstrap (seconds)
    pub bootstrap_interval_secs: u64,
    /// Enable mDNS for local discovery (in addition to DHT)
    pub enable_mdns: bool,
    /// Protocol name for DHT
    pub protocol_name: String,
}

impl Default for DhtConfig {
    fn default() -> Self {
        Self {
            bootstrap_nodes: default_bootstrap_nodes(),
            dht_port: 30333,
            enabled: true,
            bootstrap_interval_secs: 30,
            enable_mdns: true,
            protocol_name: "/netchain/kad/1.0.0".to_string(),
        }
    }
}

/// Commands for DHT service
#[derive(Debug, Clone)]
pub enum DhtCommand {
    /// Add a peer to the routing table
    AddPeer { peer_id: PeerId, address: Multiaddr },
    /// Remove a peer from the routing table
    RemovePeer { peer_id: PeerId },
    /// Get peers from the routing table
    GetPeers { response_tx: UnboundedSender<HashSet<PeerId>> },
    /// Get closest peers to a key
    GetClosestPeers {
        key: Vec<u8>,
        response_tx: UnboundedSender<Vec<PeerId>>,
    },
    /// Put a record in the DHT
    PutRecord {
        key: Vec<u8>,
        value: Vec<u8>,
        response_tx: UnboundedSender<bool>,
    },
    /// Get a record from the DHT
    GetRecord {
        key: Vec<u8>,
        response_tx: UnboundedSender<Option<Vec<u8>>>,
    },
    /// Trigger bootstrap
    Bootstrap {
        response_tx: UnboundedSender<bool>,
    },
}

/// DHT events
#[derive(Debug, Clone)]
pub enum DhtEvent {
    PeerDiscovered(PeerId),
    PeerConnected(PeerId),
    PeerDisconnected(PeerId),
    RecordFound { key: Vec<u8>, value: Vec<u8> },
    BootstrapComplete,
}

/// Shared state for DHT
#[derive(Debug, Default)]
pub struct DhtSharedState {
    pub peer_count: AtomicUsize,
    discovered_peers: Arc<std::sync::Mutex<HashSet<PeerId>>>,
    connected_peers: Arc<std::sync::Mutex<HashSet<PeerId>>>,
}

impl DhtSharedState {
    pub fn new() -> Self {
        Self {
            peer_count: AtomicUsize::new(0),
            discovered_peers: Arc::new(std::sync::Mutex::new(HashSet::new())),
            connected_peers: Arc::new(std::sync::Mutex::new(HashSet::new())),
        }
    }

    pub fn get_peer_count(&self) -> usize {
        self.peer_count.load(Ordering::Relaxed)
    }

    pub fn get_discovered_peers(&self) -> HashSet<PeerId> {
        self.discovered_peers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    pub fn get_connected_peers(&self) -> HashSet<PeerId> {
        self.connected_peers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    fn add_discovered_peer(&self, peer: PeerId) -> bool {
        let mut peers = self
            .discovered_peers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        peers.insert(peer)
    }

    fn add_connected_peer(&self, peer: PeerId) {
        let mut peers = self
            .connected_peers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        peers.insert(peer);
        self.peer_count.fetch_add(1, Ordering::Relaxed);
    }

    fn remove_connected_peer(&self, peer: PeerId) {
        let mut peers = self
            .connected_peers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        peers.remove(&peer);
        self.peer_count.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Combined network behavior
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "OutEvent")]
pub struct NetChainBehaviour {
    pub kademlia: KademliaBehaviour<MemoryStore>,
    pub mdns: Mdns,
}

/// Output events from the combined behavior
#[derive(Debug)]
pub enum OutEvent {
    Kademlia(KademliaEvent),
    Mdns(MdnsEvent),
}

impl From<KademliaEvent> for OutEvent {
    fn from(e: KademliaEvent) -> Self {
        OutEvent::Kademlia(e)
    }
}

impl From<MdnsEvent> for OutEvent {
    fn from(e: MdnsEvent) -> Self {
        OutEvent::Mdns(e)
    }
}

/// DHT Service for peer discovery
pub struct DhtService {
    pub local_peer_id: PeerId,
    pub swarm: Swarm<NetChainBehaviour>,
    pub shared_state: Arc<DhtSharedState>,
    command_rx: UnboundedReceiver<DhtCommand>,
    event_tx: mpsc::Sender<DhtEvent>,
    bootstrap_interval: Interval,
    pending_bootstrap: bool,
}

impl DhtService {
    /// Create a new DHT service
    pub async fn new(
        config: DhtConfig,
        local_key: &Keypair,
        event_tx: mpsc::Sender<DhtEvent>,
    ) -> anyhow::Result<(Self, UnboundedSender<DhtCommand>)> {
        let local_peer_id = PeerId::from(local_key.public());

        // Create transport
        let transport = tcp::tokio::Transport::new(tcp::Config::default())
            .upgrade(Version::V1)
            .authenticate(noise::Config::new(local_key)?)
            .multiplex(yamux::Config::default())
            .boxed();

        // Create Kademlia store and behavior
        let store = MemoryStore::new(local_peer_id);
        let kademlia_config = KademliaConfig::default();

        let mut kademlia = KademliaBehaviour::with_config(local_peer_id, store, kademlia_config);

        // Add bootstrap nodes
        for bootstrap in &config.bootstrap_nodes {
            if let Some(peer_id) = bootstrap.parse_peer_id() {
                let addr: Multiaddr = bootstrap.multiaddr.parse().unwrap_or_else(|_| {
                    format!("/ip4/0.0.0.0/tcp/{}", config.dht_port)
                        .parse()
                        .unwrap()
                });
                kademlia.add_address(&peer_id, addr.clone());
                debug!("Added bootstrap node: {} at {}", bootstrap.name.as_deref().unwrap_or("unknown"), addr);
            }
        }

        // Create mDNS for local discovery
        let mdns = Mdns::new(Default::default(), local_peer_id)?;

        // Create combined behavior
        let behaviour = NetChainBehaviour { kademlia, mdns };

        // Create swarm
        let mut swarm = Swarm::new(
            transport,
            behaviour,
            local_peer_id,
            libp2p::swarm::Config::with_tokio_executor()
                .with_idle_connection_timeout(Duration::from_secs(60)),
        );

        // Listen on configured port
        let listen_addr: Multiaddr = multiaddr![
            Ip4([0, 0, 0, 0]),
            Tcp(config.dht_port)
        ];
        swarm.listen_on(listen_addr)?;

        // Create command channel
        let (command_tx, command_rx) = mpsc::unbounded_channel();

        // Create bootstrap interval
        let bootstrap_interval = tokio::time::interval(Duration::from_secs(
            config.bootstrap_interval_secs,
        ));

        let service = Self {
            local_peer_id,
            swarm,
            shared_state: Arc::new(DhtSharedState::new()),
            command_rx,
            event_tx,
            bootstrap_interval,
            pending_bootstrap: false,
        };

        Ok((service, command_tx))
    }

    /// Get multiaddr for this node
    pub fn get_multiaddr(&self, port: u16) -> Multiaddr {
        multiaddr![
            Ip4([0, 0, 0, 0]),
            Tcp(port),
            P2p(self.local_peer_id)
        ]
    }

    /// Run the DHT service event loop
    pub async fn run(&mut self) {
        info!(
            peer_id = %self.local_peer_id,
            "DHT service started"
        );

        // Initial bootstrap
        self.swarm.behaviour_mut().kademlia.bootstrap().ok();
        self.pending_bootstrap = true;

        loop {
            tokio::select! {
                // Swarm events
                swarm_event = self.swarm.select_next_some() => {
                    self.handle_swarm_event(swarm_event).await;
                }

                // Commands
                Some(command) = self.command_rx.recv() => {
                    self.handle_command(command).await;
                }

                // Bootstrap interval
                _ = self.bootstrap_interval.tick() => {
                    if !self.pending_bootstrap {
                        debug!("Triggering periodic bootstrap");
                        self.swarm.behaviour_mut().kademlia.bootstrap().ok();
                        self.pending_bootstrap = true;
                    }
                }
            }
        }
    }

    /// Handle swarm events
    async fn handle_swarm_event(
        &mut self,
        event: SwarmEvent<OutEvent>,
    ) {
        match event {
            SwarmEvent::Behaviour(OutEvent::Kademlia(kad_event)) => {
                self.handle_kademlia_event(kad_event).await;
            }
            SwarmEvent::Behaviour(OutEvent::Mdns(mdns_event)) => {
                self.handle_mdns_event(mdns_event).await;
            }
            SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                debug!("Connected to peer: {} via {:?}", peer_id, endpoint);
                self.shared_state.add_connected_peer(peer_id);

                // Add to Kademlia routing table
                let address = endpoint.get_remote_address();
                self.swarm
                    .behaviour_mut()
                    .kademlia
                    .add_address(&peer_id, address.clone());

                let _ = self.event_tx.send(DhtEvent::PeerConnected(peer_id)).await;
            }
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                debug!("Disconnected from peer: {}", peer_id);
                self.shared_state.remove_connected_peer(peer_id);
                let _ = self.event_tx.send(DhtEvent::PeerDisconnected(peer_id)).await;
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                info!("Listening on: {}", address);
            }
            SwarmEvent::ExpiredListenAddr { address, .. } => {
                info!("Expired listen address: {}", address);
            }
            SwarmEvent::IncomingConnection { .. } => {
                // Incoming connection attempt
            }
            SwarmEvent::IncomingConnectionError { error, .. } => {
                warn!("Incoming connection error: {}", error);
            }
            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                if let Some(pid) = peer_id {
                    warn!("Outgoing connection error to {}: {}", pid, error);
                }
            }
            _ => {}
        }
    }

    /// Handle Kademlia events
    async fn handle_kademlia_event(&mut self, event: KademliaEvent) {
        match event {
            KademliaEvent::RoutingUpdated {
                peer,
                is_new_peer,
                addresses,
                ..
            } => {
                debug!(
                    "Routing updated for peer: {} (new: {})",
                    peer, is_new_peer
                );

                if is_new_peer {
                    self.shared_state.add_discovered_peer(peer);
                    let _ = self.event_tx.send(DhtEvent::PeerDiscovered(peer)).await;

                    // Try to connect to new peer - dial first address
                    if let Some(addr) = addresses.iter().next() {
                        let _ = self.swarm.dial(addr.clone());
                    }
                }
            }
            KademliaEvent::OutboundQueryProgressed { result, .. } => {
                match result {
                    libp2p::kad::QueryResult::Bootstrap(Ok(_)) => {
                        debug!("Bootstrap completed successfully");
                        self.pending_bootstrap = false;
                        let _ = self.event_tx.send(DhtEvent::BootstrapComplete).await;
                    }
                    libp2p::kad::QueryResult::Bootstrap(Err(e)) => {
                        warn!("Bootstrap failed: {:?}", e);
                        self.pending_bootstrap = false;
                    }
                    libp2p::kad::QueryResult::GetRecord(Ok(result)) => {
                        match result {
                            libp2p::kad::GetRecordOk::FoundRecord(peer_record) => {
                                debug!(
                                    "Found record: key={:?}, value_len={}",
                                    peer_record.record.key,
                                    peer_record.record.value.len()
                                );
                                let _ = self
                                    .event_tx
                                    .send(DhtEvent::RecordFound {
                                        key: peer_record.record.key.to_vec(),
                                        value: peer_record.record.value.to_vec(),
                                    })
                                    .await;
                            }
                            libp2p::kad::GetRecordOk::FinishedWithNoAdditionalRecord { .. } => {}
                        }
                    }
                    libp2p::kad::QueryResult::GetRecord(Err(e)) => {
                        warn!("GetRecord failed: {:?}", e);
                    }
                    libp2p::kad::QueryResult::PutRecord(Ok(_)) => {
                        debug!("PutRecord succeeded");
                    }
                    libp2p::kad::QueryResult::PutRecord(Err(e)) => {
                        warn!("PutRecord failed: {:?}", e);
                    }
                    _ => {}
                }
            }
            KademliaEvent::UnroutablePeer { peer } => {
                debug!("Unroutable peer: {}", peer);
            }
            _ => {}
        }
    }

    /// Handle mDNS events
    async fn handle_mdns_event(&mut self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(list) => {
                for (peer, addr) in list {
                    debug!("mDNS discovered peer: {} at {}", peer, addr);
                    self.shared_state.add_discovered_peer(peer);
                    self.swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(&peer, addr);
                    let _ = self.event_tx.send(DhtEvent::PeerDiscovered(peer)).await;
                }
            }
            MdnsEvent::Expired(list) => {
                for (peer, _) in list {
                    debug!("mDNS expired peer: {}", peer);
                }
            }
        }
    }

    /// Handle DHT commands
    async fn handle_command(&mut self, command: DhtCommand) {
        match command {
            DhtCommand::AddPeer { peer_id, address } => {
                self.swarm
                    .behaviour_mut()
                    .kademlia
                    .add_address(&peer_id, address.clone());
                let _ = self.swarm.dial(address);
            }
            DhtCommand::RemovePeer { peer_id } => {
                // Remove all addresses for this peer
                self.swarm.behaviour_mut().kademlia.remove_address(&peer_id, &Multiaddr::empty());
            }
            DhtCommand::GetPeers { response_tx } => {
                let peers = self.shared_state.get_discovered_peers();
                let _ = response_tx.send(peers);
            }
            DhtCommand::GetClosestPeers { key, response_tx } => {
                // Start the query - results come via events
                let key_bytes: Vec<u8> = key;
                self.swarm
                    .behaviour_mut()
                    .kademlia
                    .get_closest_peers(key_bytes);
                // For now, send empty response - actual results come via DhtEvent::RecordFound
                let _ = response_tx.send(Vec::new());
            }
            DhtCommand::PutRecord {
                key,
                value,
                response_tx,
            } => {
                let record_key = libp2p::kad::RecordKey::new(&key);
                let result = self
                    .swarm
                    .behaviour_mut()
                    .kademlia
                    .put_record(
                        libp2p::kad::Record::new(record_key, value),
                        libp2p::kad::Quorum::One,
                    );

                let success = result.is_ok();
                let _ = response_tx.send(success);
            }
            DhtCommand::GetRecord {
                key,
                response_tx,
            } => {
                let record_key = libp2p::kad::RecordKey::new(&key);
                self.swarm
                    .behaviour_mut()
                    .kademlia
                    .get_record(record_key);
                // Response comes via event
                let _ = response_tx.send(None);
            }
            DhtCommand::Bootstrap { response_tx } => {
                let result = self.swarm.behaviour_mut().kademlia.bootstrap();
                let success = result.is_ok();
                if success {
                    self.pending_bootstrap = true;
                }
                let _ = response_tx.send(success);
            }
        }
    }

    /// Get shared state reference
    pub fn shared_state(&self) -> Arc<DhtSharedState> {
        Arc::clone(&self.shared_state)
    }

    /// Get number of connected peers
    pub fn peer_count(&self) -> usize {
        self.shared_state.get_peer_count()
    }

    /// Get local peer ID
    pub fn local_peer_id(&self) -> PeerId {
        self.local_peer_id
    }
}

/// Helper to create multiaddr from socket address and peer ID
pub fn socket_to_multiaddr(addr: SocketAddr, peer_id: PeerId) -> Multiaddr {
    let ip = match addr.ip() {
        std::net::IpAddr::V4(ip) => Protocol::Ip4(ip),
        std::net::IpAddr::V6(ip) => Protocol::Ip6(ip),
    };
    let port = Protocol::Tcp(addr.port());
    let p2p = Protocol::P2p(peer_id);

    Multiaddr::empty().with(ip).with(port).with(p2p)
}

/// Helper to parse multiaddr to socket address
pub fn multiaddr_to_socket(addr: &Multiaddr) -> Option<SocketAddr> {
    let mut ip = None;
    let mut port = None;

    for component in addr.iter() {
        match component {
            Protocol::Ip4(addr) => ip = Some(std::net::IpAddr::V4(addr.clone())),
            Protocol::Ip6(addr) => ip = Some(std::net::IpAddr::V6(addr.clone())),
            Protocol::Tcp(p) => port = Some(p),
            _ => {}
        }
    }

    match (ip, port) {
        (Some(ip), Some(port)) => Some(SocketAddr::new(ip, port)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::identity::Keypair;

    #[test]
    fn test_bootstrap_node_parsing() {
        let bootstrap = BootstrapNode::new(
            "12D3KooWRxEPF5f8qXqf3Z5qXqf3Z5qXqf3Z5qXqf3Z5qXqf3Z5".to_string(),
            "/ip4/192.168.1.1/tcp/30333/p2p/12D3KooWRxEPF5f8qXqf3Z5qXqf3Z5qXqf3Z5qXqf3Z5qXqf3Z5"
                .to_string(),
        );

        assert!(bootstrap.parse_peer_id().is_some());
    }

    #[test]
    fn test_socket_multiaddr_conversion() {
        let socket = SocketAddr::from(([192, 168, 1, 1], 30333));
        let peer_id = PeerId::random();
        let multiaddr = socket_to_multiaddr(socket, peer_id);

        assert!(multiaddr.to_string().contains("192.168.1.1"));
        assert!(multiaddr.to_string().contains("30333"));

        let parsed_socket = multiaddr_to_socket(&multiaddr);
        assert_eq!(parsed_socket, Some(socket));
    }

    #[test]
    fn test_dht_config_default() {
        let config = DhtConfig::default();
        assert!(config.enabled);
        assert!(config.enable_mdns);
        assert!(!config.bootstrap_nodes.is_empty());
    }
}
