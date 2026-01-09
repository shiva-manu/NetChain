// src/p2p.rs

use libp2p::futures::StreamExt;

use libp2p::{
    PeerId, identity,
    swarm::{Swarm, SwarmEvent},
    gossipsub::{
        Behaviour as Gossipsub,
        Event as GossipsubEvent,
        MessageAuthenticity,
        IdentTopic as Topic,
        Config as GossipsubConfig,
    },
    mdns::{tokio::Behaviour as Mdns, Event as MdnsEvent},
    noise,
    tcp,
    yamux,
    Transport,
};

use libp2p::swarm::derive_prelude::*;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum NetworkMessage {
    Block(String),
    Transaction(String),
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

pub struct P2PService {
    pub peer_id: PeerId,
    pub swarm: Swarm<NetBehaviour>,
    block_topic: Topic,
    tx_topic: Topic,
}

impl P2PService {
    pub async fn new(port: u16) -> anyhow::Result<Self> {
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
        ).map_err(|e| anyhow::anyhow!(e))?;

        let block_topic = Topic::new("blocks");
        let tx_topic = Topic::new("transactions");

        gossipsub.subscribe(&block_topic)?;
        gossipsub.subscribe(&tx_topic)?;

        let mdns = Mdns::new(Default::default(), peer_id)?;

        let behaviour = NetBehaviour { gossipsub, mdns };

        let mut swarm = Swarm::new(
            transport,
            behaviour,
            peer_id,
            libp2p::swarm::Config::with_tokio_executor(),
        );

        swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{port}").parse()?)?;

        Ok(Self {
            peer_id,
            swarm,
            block_topic,
            tx_topic,
        })
    }

    pub async fn run(mut self, sender: mpsc::Sender<P2PEvent>) {
        loop {
            match self.swarm.select_next_some().await {
                SwarmEvent::Behaviour(OutEvent::Gossip(
                    GossipsubEvent::Message { message, .. },
                )) => {
                    let msg = String::from_utf8_lossy(&message.data).to_string();
                    let _ = sender
                        .send(P2PEvent::Message(NetworkMessage::Block(msg)))
                        .await;
                }

                SwarmEvent::Behaviour(OutEvent::Mdns(event)) => match event {
                    MdnsEvent::Discovered(list) => {
                        for (peer, _) in list {
                            let _ = sender.send(P2PEvent::PeerConnected(peer)).await;
                        }
                    }
                    MdnsEvent::Expired(list) => {
                        for (peer, _) in list {
                            let _ = sender.send(P2PEvent::PeerDisconnected(peer)).await;
                        }
                    }
                },

                _ => {}
            }
        }
    }
}
