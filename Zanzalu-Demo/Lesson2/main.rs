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

    /
    let mut remote_addrs: Vec<Multiaddr> = Vec::default();
    if let Ok(remote_peers) = env::var("REMOTE_PEERS") {
        remote_addrs = remote_peers
            .split(',')                         
            .map(str::trim)                     
            .filter(|s| !s.is_empty())          
            .map(Multiaddr::from_str)           
            .collect::<Result<Vec<_>, _>>()?;   
    }
     
    // Generate a random Ed25519 keypair for our local peer
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = local_key.public().to_peer_id();
    
    println!("Local peer id: {local_peer_id}", local_peer_id);
    
    // Build the Swarm
    let mut swarm = SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|_| Behaviour { ping: ping::Behaviour::default() })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    // Dial all of the remote peer Multiaddrs
    for addr in remote_addrs.into_iter() {
        swarm.dial(addr)?;
    }
    
    // Run the network event loop
    loop {
        match swarm.select_next_some().await {
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
                println!("Failed to connect to {peer_id:?}: {error}");
            }
            _ => {}
        }
    }
}