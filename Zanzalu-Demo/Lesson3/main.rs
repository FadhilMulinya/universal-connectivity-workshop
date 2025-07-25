use anyhow::Result;
use futures::StreamExt;
use libp2p::{
    identity, noise, ping, tcp, yamux,
    Multiaddr, SwarmBuilder,
    swarm::{NetworkBehaviour, SwarmEvent}
};
use std::{env, str::FromStr, time::Duration};

#[derive(NetworkBehaviour)]
struct Behaviour {
    ping: ping::Behaviour,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting Universal Connectivity Application...");

    // parse the remote peer addresses from the environment variable
    let mut remote_addrs: Vec<Multiaddr> = Vec::default();
    if let Ok(remote_peers) = env::var("REMOTE_PEERS") {
        remote_addrs = remote_peers
            .split(',')                     
            .map(str::trim)                    
            .filter(|s| !s.is_empty())          
            .map(Multiaddr::from_str)          
            .collect::<Result<Vec<_>, _>>()?;   
    }

    // Generate a new Ed25519 keypair for our local peer
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = identity::PeerId::from(local_key.public());

    println!("Local peer id: {:?}", local_peer_id);

    // Build the Swarm
    let mut swarm = SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|_| Behaviour {
            ping: ping::Behaviour::new(
                ping::Config::new()
                    .with_interval(Duration::from_secs(1))
                    .with_timeout(Duration::from_secs(5))
            ),
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    // Dial all of the remote peer Multiaddrs
    for addr in remote_addrs.into_iter() {
        swarm.dial(addr)?;
    }

    loop {
        tokio::select! {
            Some(event) = swarm.next() => match event {
                SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                    println!("Connected to: {peer_id} via {}", endpoint.get_remote_address());
                }
                SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                    if let Some(err) = cause {
                        println!("Connection to {peer_id} closed with error: {err}");
                    } else {
                        println!("Connection to {peer_id} closed gracefully");
                    }
                }
                SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                    println!("Failed to connect to {:?}: {error}", peer_id);
                }
                SwarmEvent::Behaviour(behaviour_event) => match behaviour_event {
                    BehaviourEvent::Ping(ping_event) => {
                        match ping_event {
                            ping::Event { peer, result: Ok(rtt), .. } => {
                                println!("Received a ping from {peer}, round trip time: {} ms", rtt.as_millis());
                            }
                            ping::Event { peer, result: Err(failure), .. } => {
                                println!("Ping failed to {peer}: {}", failure);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}