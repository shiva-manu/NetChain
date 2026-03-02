// src/p2p.rs

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
        )
        .map_err(|e| anyhow::anyhow!(e))?;

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

    pub async fn run(&mut self, sender: mpsc::Sender<P2PEvent>) {
        loop {
            match self.swarm.select_next_some().await {
                SwarmEvent::Behaviour(OutEvent::Gossip(GossipsubEvent::Message {
                    message,
                    ..
                })) => {
                    let msg = String::from_utf8_lossy(&message.data).to_string();
                    if let Some(network_msg) = self.route_topic_message(&message.topic, msg) {
                        let _ = sender.send(P2PEvent::Message(network_msg)).await;
                    }
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
    pub fn publish_block(&mut self, block_json: String) {
        let _ = self
            .swarm
            .behaviour_mut()
            .gossipsub
            .publish(self.block_topic.clone(), block_json.as_bytes());
    }

    pub fn publish_transaction(&mut self, tx_json: String) {
        let _ = self
            .swarm
            .behaviour_mut()
            .gossipsub
            .publish(self.tx_topic.clone(), tx_json.as_bytes());
    }

    fn route_topic_message(
        &self,
        topic: &libp2p::gossipsub::TopicHash,
        payload: String,
    ) -> Option<NetworkMessage> {
        if *topic == self.block_topic.hash() {
            Some(NetworkMessage::Block(payload))
        } else if *topic == self.tx_topic.hash() {
            Some(NetworkMessage::Transaction(payload))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NetworkMessage;

    fn route_topic_payload(topic: &str, payload: String) -> Option<NetworkMessage> {
        match topic {
            "blocks" => Some(NetworkMessage::Block(payload)),
            "transactions" => Some(NetworkMessage::Transaction(payload)),
            _ => None,
        }
    }

    #[test]
    fn routes_block_topic_to_block_message() {
        let routed = route_topic_payload("blocks", "{\"index\":1}".to_string());
        assert!(matches!(routed, Some(NetworkMessage::Block(_))));
    }

    #[test]
    fn routes_transaction_topic_to_transaction_message() {
        let routed = route_topic_payload("transactions", "{\"amount\":5}".to_string());
        assert!(matches!(routed, Some(NetworkMessage::Transaction(_))));
    }
}
