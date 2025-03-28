use crate::GameState;
use footsteps_core::Outputs;
use footsteps_methods::{FOOTSTEPS_GUEST_ELF, FOOTSTEPS_GUEST_ID};
use futures::StreamExt;
use libp2p::{
    core::upgrade,
    gossipsub::{self, IdentTopic, MessageAuthenticity},
    identity::Keypair,
    mdns::{self, tokio::Behaviour as MdnsBehaviour},
    noise,
    swarm::{NetworkBehaviour, SwarmBuilder, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, Transport,
};
use risc0_zkvm::Receipt;
use serde::{Deserialize, Serialize};
use std::thread;
use std::{
    error::Error,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::mpsc;
use serde_json;

// Message types for our P2P network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum P2PMessage {
    // Player movement with proof
    Proof { player_id: String, receipt: Receipt, ImageID: [u32; 8] },
    // Player joined
    PlayerJoined { player_id: String, name: String },
    // Player left
    PlayerLeft { player_id: String },
    // Node identification with custom data
    NodeInfo { 
        node_id: String, 
        name: String, 
        custom_url: String,
        // Add any other custom fields you want to exchange
    },
}

// Define the network behavior
#[derive(NetworkBehaviour)]
struct GameBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: MdnsBehaviour,
}

// P2P node configuration
pub struct P2PNode {
    pub peer_id: PeerId,
    pub topic: IdentTopic,
    sender: mpsc::Sender<P2PMessage>,
    receiver: mpsc::Receiver<P2PMessage>,
    known_peers: Vec<(String, u16)>, // List of known peers (hostname/IP, port)
    connection_events: mpsc::Sender<String>, // Channel for connection events
    node_name: String,
    custom_url: String, // Custom URL to share with other nodes
}

impl P2PNode {
    // Create a new P2P node
    pub fn new(
        topic_name: &str, 
        known_peers: Vec<(String, u16)>, 
        connection_events: mpsc::Sender<String>,
        node_name: String,
        custom_url: String,
    ) -> Result<Self, Box<dyn Error>> {
        // Create a random keypair for identity
        let id_keys = Keypair::generate_ed25519();
        let peer_id = PeerId::from(id_keys.public());
        println!("Local peer ID: {}", peer_id);

        // Create a channel for sending messages to the P2P network
        let (sender, receiver) = mpsc::channel(100);

        // Create the gossipsub topic
        let topic = IdentTopic::new(topic_name);

        Ok(Self {
            peer_id,
            topic,
            sender,
            receiver,
            known_peers,
            connection_events,
            node_name,
            custom_url,
        })
    }

    // Get a sender for sending messages to the P2P network
    pub fn sender(&self) -> mpsc::Sender<P2PMessage> {
        self.sender.clone()
    }

    // Start the P2P node
    pub async fn start(
        mut self,
        game_state: Arc<Mutex<GameState>>,
        listen_port: u16,
    ) -> Result<(), Box<dyn Error>> {
        // Create a simple TCP transport
        let transport = tcp::tokio::Transport::new(tcp::Config::default())
            .upgrade(upgrade::Version::V1)
            .authenticate(noise::Config::new(&Keypair::generate_ed25519())?)
            .multiplex(yamux::Config::default())
            .boxed();

        // Create the gossipsub behavior
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .max_transmit_size(1024 * 1024)
            .build()?;

        let mut gossipsub = gossipsub::Behaviour::new(
            MessageAuthenticity::Signed(Keypair::generate_ed25519()),
            gossipsub_config,
        )?;

        // Subscribe to the topic
        gossipsub.subscribe(&self.topic)?;

        // Create the mdns behavior for local peer discovery
        let mdns = MdnsBehaviour::new(mdns::Config::default(), self.peer_id)?;

        // Build the swarm
        let mut swarm = SwarmBuilder::with_tokio_executor(
            transport,
            GameBehaviour { gossipsub, mdns },
            self.peer_id,
        )
        .build();

        // Listen on all interfaces and the specified port
        let listen_addr = format!("/ip4/0.0.0.0/tcp/{}", listen_port);
        println!("Attempting to listen on {}", listen_addr);
        swarm.listen_on(listen_addr.parse()?)?;

        // Connect to known peers
        for (peer_host, peer_port) in &self.known_peers {
            let peer_addr = format!("/ip4/{}/tcp/{}", peer_host, peer_port);
            println!("Attempting to connect to peer at {}", peer_addr);

            match peer_addr.parse::<Multiaddr>() {
                Ok(addr) => {
                    if let Err(e) = swarm.dial(addr.clone()) {
                        eprintln!("Failed to dial {}: {:?}", addr, e);
                    } else {
                        println!("Dialing peer at {}", addr);
                    }
                }
                Err(e) => eprintln!("Invalid multiaddr {}: {:?}", peer_addr, e),
            }
        }

        // Remove the periodic node info interval
        // let mut node_info_interval = tokio::time::interval(Duration::from_secs(10));
        
        // Flag to track if we should try sending node info
        let mut try_node_info = true; // Start with true to send node info once at startup
        let mut retry_timer = tokio::time::interval(Duration::from_secs(3));
        
        // Event loop
        loop {
            tokio::select! {
                // Remove the periodic node info broadcast
                // _ = node_info_interval.tick() => {
                //     // Periodically broadcast our node info
                //     self.broadcast_node_info(&mut swarm);
                // }
                _ = retry_timer.tick(), if try_node_info => {
                    // Try to send node info after startup or new connection
                    println!("Trying to send node info...");
                    if self.broadcast_node_info(&mut swarm) {
                        // If successful, reset the flag
                        try_node_info = false;
                        println!("Successfully sent node info");
                    } else {
                        println!("Failed to send node info, will retry in 3 seconds");
                    }
                }
                event = swarm.select_next_some() => {
                    match event {
                        SwarmEvent::NewListenAddr { address, .. } => {
                            println!("Listening on {}", address);
                        }
                        SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                            let addr = endpoint.get_remote_address();
                            println!("Connection established with {} via {}", peer_id, addr);
                            
                            // Send connection event to the main thread with more detailed information
                            let event_data = serde_json::json!({
                                "peer_id": peer_id.to_string(),
                                "address": addr.to_string(),
                                "is_incoming": endpoint.is_listener(),
                                "timestamp": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                            });
                            
                            let event_msg = serde_json::to_string(&event_data).unwrap_or_else(|_| 
                                format!("Connected to peer: {} via {}", peer_id, addr)
                            );
                            
                            if let Err(e) = self.connection_events.send(event_msg).await {
                                eprintln!("Failed to send connection event: {:?}", e);
                            }
                            
                            // Set flag to try sending node info after new connection
                            try_node_info = true;
                        }
                        SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                            if let Some(id) = peer_id {
                                eprintln!("Failed to connect to peer {}: {:?}", id, error);
                            } else {
                                eprintln!("Failed to connect to peer: {:?}", error);
                            }
                        }
                        SwarmEvent::Behaviour(behaviour) => match behaviour {
                            GameBehaviourEvent::Mdns(mdns::Event::Discovered(list)) => {
                                for (peer_id, multiaddr) in list {
                                    println!("mDNS discovered peer: {} at {}", peer_id, multiaddr);
                                    swarm.dial(multiaddr)?;
                                }
                            }
                            GameBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                                propagation_source: peer_id,
                                message_id: _,
                                message,
                            }) => {
                                println!("Received proof from {}", peer_id);

                                // Try to parse the message
                                if let Ok(p2p_msg) = serde_json::from_slice::<P2PMessage>(&message.data) {
                                    match &p2p_msg {
                                        P2PMessage::Proof { player_id, receipt, ImageID } => {
                                            println!("Proof from {}. ImageID: {:?}", player_id, ImageID);

                                            // Here you would verify the proof and update the game state
                                            // For now, just update the position if it's not from this node
                                            if !player_id.starts_with(&self.node_name) {
                                                // Only update if it's from another player
                                                // In a real implementation, you'd verify the proof first

                                                println!("Verifying proof...");
                                                {
                                                    let mut state = game_state.lock().unwrap();
                                                    state.proof_status = "Verifying proof...".to_string();
                                                }

                                                // Verify the proof
                                                if let Err(e) = receipt.verify(*ImageID) {
                                                    println!("Error verifying proof: {:?}", e);

                                                    // Mark as no longer processing
                                                    let mut state = game_state.lock().unwrap();
                                                    state.proof_status = "Proof verification failed".to_string();

                                                    continue;
                                                }

                                                println!("Proof verified successfully!");

                                                // Extract the outputs
                                                let outputs: Outputs = match receipt.journal.decode() {
                                                    Ok(outputs) => outputs,
                                                    Err(e) => {
                                                        println!("Error decoding journal: {:?}", e);

                                                        // Mark as no longer processing
                                                        let mut state = game_state.lock().unwrap();
                                                        state.proof_status = "Journal decoding failed".to_string();

                                                        continue;
                                                    }
                                                };

                                                // Update game state
                                                let mut state: std::sync::MutexGuard<'_, GameState> = game_state.lock().unwrap();

                                                // Get the trail length before moving it
                                                let trail_len = outputs.trail_positions.len();
                                                let trail_summary = format!("{:?}", outputs.trail_positions);

                                                // Update the verified trail - make a deep copy to ensure it's a new object
                                                state.verified_trail = outputs.trail_positions.clone();
                                                state.proof_status = format!("Proof verified! Trail: {} positions", trail_len);

                                                println!("Batch processed! Trail verified with {} positions: {}",
                                                        trail_len, trail_summary);

                                                // Force immediate update of the trail
                                                drop(state); // Release the lock before sleeping

                                                // Small delay to ensure the trail update is processed
                                                thread::sleep(Duration::from_millis(50));
                                            }
                                        }
                                        P2PMessage::PlayerJoined { player_id, name } => {
                                            println!("Player joined: {} ({})", name, player_id);
                                        }
                                        P2PMessage::PlayerLeft { player_id } => {
                                            println!("Player left: {}", player_id);
                                        }
                                        P2PMessage::NodeInfo { node_id, name, custom_url } => {
                                            println!("Received node info from {}: name={}, url={}", node_id, name, custom_url);
                                            
                                            // Send the node info to the main thread
                                            let node_info_data = serde_json::json!({
                                                "type": "node_info",
                                                "peer_id": node_id,
                                                "name": name,
                                                "custom_url": custom_url,
                                            });
                                            
                                            let node_info_msg = serde_json::to_string(&node_info_data).unwrap_or_else(|_| 
                                                format!("Node info: {} ({}), URL: {}", name, node_id, custom_url)
                                            );
                                            
                                            if let Err(e) = self.connection_events.send(node_info_msg).await {
                                                eprintln!("Failed to send node info event: {:?}", e);
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
                Some(msg) = self.receiver.recv() => {
                    // Received a message to send to the P2P network
                    println!("Sending message to P2P network");

                    // Serialize and publish the message
                    match serde_json::to_vec(&msg) {
                        Ok(data) => {
                            if let Err(e) = swarm.behaviour_mut().gossipsub.publish(self.topic.clone(), data) {
                                eprintln!("Error publishing message: {:?}", e);
                            }
                        }
                        Err(e) => {
                            eprintln!("Error serializing message: {:?}", e);
                        }
                    }
                }
            }
        }
    }
    
    // Helper method to broadcast node info
    fn broadcast_node_info(&self, swarm: &mut libp2p::swarm::Swarm<GameBehaviour>) -> bool {
        let node_info = P2PMessage::NodeInfo {
            node_id: self.peer_id.to_string(),
            name: self.node_name.clone(),
            custom_url: self.custom_url.clone(),
        };
        
        // Serialize and publish the node info message
        match serde_json::to_vec(&node_info) {
            Ok(data) => {
                match swarm.behaviour_mut().gossipsub.publish(self.topic.clone(), data) {
                    Ok(_) => {
                        println!("Successfully sent node info to peers");
                        true
                    }
                    Err(e) => {
                        // This is expected to fail sometimes when there aren't enough peers
                        if e.to_string().contains("InsufficientPeers") {
                            println!("Not enough peers to publish node info yet (this is normal during startup)");
                        } else {
                            eprintln!("Error publishing node info: {:?}", e);
                        }
                        false
                    }
                }
            }
            Err(e) => {
                eprintln!("Error serializing node info: {:?}", e);
                false
            }
        }
    }
}

// Helper function to start a P2P node
pub async fn start_p2p_node(
    node_name: String,
    game_state: Arc<Mutex<GameState>>,
    p2p_port: u16,
    known_peers: Vec<(String, u16)>,
    custom_url: String,
) -> Result<(mpsc::Sender<P2PMessage>, mpsc::Receiver<String>), Box<dyn Error>> {
    // Create a channel for connection events
    let (connection_tx, connection_rx) = mpsc::channel::<String>(100);
    
    // Create a new P2P node
    let node = P2PNode::new("footsteps-game", known_peers, connection_tx, node_name, custom_url)?;

    // Get a sender for sending messages to the P2P network
    let sender = node.sender();

    // Start the node in a separate task
    tokio::spawn(async move {
        if let Err(e) = node.start(game_state, p2p_port).await {
            eprintln!("Error starting P2P node: {:?}", e);
        }
    });

    Ok((sender, connection_rx))
}
