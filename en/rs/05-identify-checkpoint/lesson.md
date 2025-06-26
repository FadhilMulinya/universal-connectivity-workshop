# Lesson 5: Identify Checkpoint 🏆

Welcome to your second checkpoint! In this lesson, you'll implement the Identify protocol, which allows libp2p peers to exchange information about their capabilities, supported protocols, and network details.

## Learning Objectives

By the end of this lesson, you will:
- Understand the purpose of the Identify protocol in libp2p
- Add Identify behavior to your NetworkBehaviour
- Handle identify events and extract peer information
- Exchange protocol capabilities with remote peers

## Background: The Identify Protocol

The Identify protocol is fundamental to libp2p's peer discovery and capability negotiation. It serves several important purposes:

- **Capability Discovery**: Learn what protocols a peer supports
- **Version Information**: Exchange software version and agent strings
- **Address Discovery**: Learn how peers see your external addresses
- **Protocol Negotiation**: Establish common protocols for communication

When peers connect, they automatically exchange identification information, allowing the network to be self-describing and adaptive.

## Your Task

Building on your QUIC transport implementation from Lesson 4, you need to:

1. **Add Identify Behavior**: Include `identify::Behaviour` in your `NetworkBehaviour` struct
2. **Configure Identify Settings**: Set up identify with the IPFS protocol and Universal Connectivity agent string
3. **Connect to Remote Peer**: Dial the peer specified in `REMOTE_PEER` environment variable
4. **Handle Identify Events**: Process identification events and display peer information

## Step-by-Step Instructions

### Step 1: Update Dependencies

Add identify support to your Cargo.toml features:

```toml
[dependencies]
libp2p = { version = "0.55", features = ["ed25519", "identify", "macros", "noise", "ping", "quic", "tcp", "tokio", "yamux"] }
```

Note the addition of the "identify" feature.

### Step 2: Add Identify Import

Add the identify import to your existing imports:

```rust
use libp2p::{identify, noise, tcp, quic, yamux, Multiaddr, SwarmBuilder};
```

### Step 3: Update Your NetworkBehaviour

Add the identify behavior to your existing behavior struct:

```rust
#[derive(NetworkBehaviour)]
struct Behaviour {
    ping: ping::Behaviour,
    identify: identify::Behaviour,
}
```

### Step 4: Configure Identify in the Swarm Builder

When creating your behavior, configure the identify protocol:

```rust
.with_behaviour(|key| Behaviour {
    ping: ping::Behaviour::new(
        ping::Config::new()
            .with_interval(Duration::from_secs(1))
            .with_timeout(Duration::from_secs(5))
    ),
    identify: identify::Behaviour::new(
        identify::Config::new("/ipfs/id/1.0.0".to_string(), key.public())
            .with_agent_version("universal-connectivity/0.1.0".to_string())
    ),
})?
```

Note that we now use the key parameter in the closure to configure identify.

### Step 5: Handle Identify Events

In your event loop, add handling for identify events alongside your existing ping events:

```rust
SwarmEvent::Behaviour(behaviour_event) => match behaviour_event {
    BehaviourEvent::Ping(ping_event) => {
        match ping_event {
            ping::Event {
                peer,
                result: Ok(ping::Success::Ping { rtt }),
            } => {
                println!("Received a ping from {}, round trip time: {} ms", peer, rtt.as_millis());
            }
            ping::Event {
                peer,
                result: Err(failure),
            } => {
                println!("Ping failed to {}: {:?}", peer, failure);
            }
            _ => {}
        }
    }
    BehaviourEvent::Identify(identify_event) => {
        match identify_event {
            identify::Event::Received { peer_id, info } => {
                println!("Identified peer: {} with protocol version: {}", peer_id, info.protocol_version);
                println!("Peer agent: {}", info.agent_version);
                println!("Peer supports {} protocols", info.protocols.len());
            }
            identify::Event::Sent { peer_id } => {
                println!("Sent identify info to: {}", peer_id);
            }
            identify::Event::Error { peer_id, error } => {
                println!("Identify error with {}: {:?}", peer_id, error);
            }
            _ => {}
        }
    }
}
```

## Testing Your Implementation

1. Set the environment variables:
   ```bash
   export PROJECT_ROOT=/path/to/workshop
   export LESSON_PATH=en/rs/05-identify-checkpoint
   ```

2. Run with Docker Compose:
   ```bash
   docker compose up --build
   ```

3. Check your output:
   ```bash
   python check.py
   ```

## Success Criteria

Your implementation should:
- ✅ Display the startup message and local peer ID
- ✅ Successfully dial the remote peer
- ✅ Establish a connection
- ✅ Send and receive ping messages
- ✅ Exchange identify information
- ✅ Display peer protocol version and agent string

## Hints

## Hint - Behavior Configuration

When using identify, you need to pass the keypair to the behavior configuration closure:

```rust
.with_behaviour(|key| Behaviour {
    // Use 'key' parameter here for identify configuration
    identify: identify::Behaviour::new(/* config using key.public() */),
    // Other behaviors...
})?
```

## Hint - Event Handling Structure

Both ping and identify events are nested within `SwarmEvent::Behaviour`:

```rust
SwarmEvent::Behaviour(behaviour_event) => match behaviour_event {
    BehaviourEvent::Ping(ping_event) => { /* handle ping */ }
    BehaviourEvent::Identify(identify_event) => { /* handle identify */ }
}
```

## Hint - Complete Solution

Here's the complete working solution:

```rust
use anyhow::Result;
use futures::StreamExt;
use libp2p::identity;
use libp2p::{identify, noise, tcp, quic, yamux, Multiaddr, SwarmBuilder};
use libp2p::{
    ping,
    swarm::{NetworkBehaviour, SwarmEvent},
};
use std::env;
use std::time::Duration;

#[derive(NetworkBehaviour)]
struct Behaviour {
    ping: ping::Behaviour,
    identify: identify::Behaviour,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting Universal Connectivity Application...");

    let remote_peer = env::var("REMOTE_PEER")?;
    let remote_addr: Multiaddr = remote_peer.parse()?;

    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = identity::PeerId::from(local_key.public());
    println!("Local peer id: {local_peer_id}");

    let mut swarm = SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_quic()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| Behaviour {
            ping: ping::Behaviour::new(
                ping::Config::new()
                    .with_interval(Duration::from_secs(1))
                    .with_timeout(Duration::from_secs(5))
            ),
            identify: identify::Behaviour::new(
                identify::Config::new("/ipfs/id/1.0.0".to_string(), key.public())
                    .with_agent_version("universal-connectivity/0.1.0".to_string())
            ),
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    println!("Dialing: {}", remote_addr);
    swarm.dial(remote_addr)?;

    loop {
        tokio::select! {
            Some(event) = swarm.next() => match event {
                SwarmEvent::ConnectionEstablished { peer_id, connection_id, endpoint, .. } => {
                    println!("Connected to: {peer_id} via {}", endpoint.get_remote_address());
                }
                SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                    if let Some(err) = cause {
                        println!("Connection to {peer_id} closed with error: {err}");
                    } else {
                        println!("Connection to {peer_id} closed gracefully");
                    }
                }
                SwarmEvent::Behaviour(behaviour_event) => match behaviour_event {
                    BehaviourEvent::Ping(ping_event) => {
                        match ping_event {
                            ping::Event {
                                peer,
                                result: Ok(ping::Success::Ping { rtt }),
                            } => {
                                println!("Received a ping from {}, round trip time: {} ms", peer, rtt.as_millis());
                            }
                            ping::Event {
                                peer,
                                result: Err(failure),
                            } => {
                                println!("Ping failed to {}: {:?}", peer, failure);
                            }
                            _ => {}
                        }
                    }
                    BehaviourEvent::Identify(identify_event) => {
                        match identify_event {
                            identify::Event::Received { peer_id, info } => {
                                println!("Identified peer: {} with protocol version: {}", peer_id, info.protocol_version);
                                println!("Peer agent: {}", info.agent_version);
                                println!("Peer supports {} protocols", info.protocols.len());
                            }
                            identify::Event::Sent { peer_id } => {
                                println!("Sent identify info to: {}", peer_id);
                            }
                            identify::Event::Error { peer_id, error } => {
                                println!("Identify error with {}: {:?}", peer_id, error);
                            }
                            _ => {}
                        }
                    }
                }
                SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                    println!("Failed to connect to {peer_id:?}: {error}");
                }
                _ => {}
            }
        }
    }
}
```

## What's Next?

Congratulations! You've reached your second checkpoint 🎉

You now have a libp2p node that can:
- Support multiple transports (TCP and QUIC)
- Measure connectivity with ping
- Exchange peer capabilities with identify

Key concepts you've learned:
- **Identify Protocol**: Exchanging peer capabilities and metadata
- **Protocol Negotiation**: How peers learn about each other's supported protocols
- **Agent Strings**: Identifying software versions in the network
- **Capability Discovery**: Building adaptive peer-to-peer networks

In the next lesson, you'll implement Gossipsub for publish-subscribe messaging, allowing peers to communicate through topic-based channels!