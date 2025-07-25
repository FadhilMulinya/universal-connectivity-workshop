use anyhow::Result;
use futures::StreamExt;
use libp2p::{identity, noise, ping, tcp, yamux, SwarmBuilder, swarm::NetworkBehaviour};
use std::time::Duration;

    #[derive(NetworkBehaviour)]
    struct Behaviour {
        ping: ping::Behaviour,
    }


#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting universal Connectivity Application..." );

    //Generate a random Ed25519 keypair for our local peer
    let local_key = identity::keypair::generate_ed25519();
    let local_peer_id = local_key.public().public().to_peer_id();

    println!("Your local peer id: {:?}", local_peer_id);


    let swarm  = SwarmBuilder::with_existing_identity(local_key)
         .with_tokio()
         .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default(),
         )?
          .with_behaviour(|_| Behaviour { ping: ping::Behaviour::default() })?
          .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))?
          .build();

        //Lets srun the loop
loop {
      tokio::select! {
          Some(event) = swarm.next() => match event {
              // Handle events here in the future
              _ => {}
          }
      }
  }

}
