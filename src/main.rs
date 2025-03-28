// Copyright 2024 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod p2p;

use footsteps_core::Outputs;
use footsteps_methods::{FOOTSTEPS_GUEST_ELF, FOOTSTEPS_GUEST_ID};
use risc0_zkvm::{default_prover, ExecutorEnv,  serde::to_vec};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
// Add WebSocket imports
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, oneshot, broadcast};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};

// Define the same KeyInput enum as in the guest code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum KeyInput {
    Up,
    Down,
    Left,
    Right,
    None,
    // Add a new key for testing constraint violations (move by 3 units)
    TestConstraint,
}

// Current position state shared between Bevy and proof generation thread
pub struct GameState {
    position_x: f32,
    position_y: f32,
    last_verified_x: f32, // Last position verified by ZK proof
    last_verified_y: f32, // Last position verified by ZK proof
    proof_start_x: f32,   // Starting position for the next proof
    proof_start_y: f32,   // Starting position for the next proof
    pending_keys: VecDeque<KeyInput>,
    processing: bool,
    next_process_time: Instant,
    proof_status: String,
    last_batch_size: usize,
    verified_trail: Vec<(f32, f32)>, // Trail verified by ZK proof (excluding final position)
}

impl GameState {
    pub fn new() -> Self {
        Self {
            position_x: 0.0,
            position_y: 0.0,
            last_verified_x: 0.0,
            last_verified_y: 0.0,
            proof_start_x: 0.0,
            proof_start_y: 0.0,
            pending_keys: VecDeque::new(),
            processing: false,
            next_process_time: Instant::now() + Duration::from_secs(5),
            proof_status: "Waiting for input".to_string(),
            last_batch_size: 0,
            verified_trail: Vec::new(),
        }
    }
}

// Function to handle a WebSocket connection
async fn handle_connection(
    ws_stream: TcpStream,
    game_state: Arc<Mutex<GameState>>,
    node_name: String,
    connection_events: broadcast::Receiver<String>,
) {
    println!(
        "New WebSocket connection: {}",
        ws_stream.peer_addr().unwrap()
    );

    let ws_stream = match accept_async(ws_stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("Error accepting WebSocket: {:?}", e);
            return;
        }
    };

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Send initial game state
    let initial_state = {
        let state = game_state.lock().unwrap();
        json!({
            "type": "state_update",
            "position": {
                "x": state.position_x,
                "y": state.position_y
            },
            "proofStatus": state.proof_status,
            "processing": state.processing,
            "lastBatchSize": state.last_batch_size,
            "trail": state.verified_trail,
            "nodeName": node_name,
        })
    };

    if let Err(e) = ws_sender
        .send(Message::Text(initial_state.to_string()))
        .await
    {
        eprintln!("Error sending initial state: {:?}", e);
        return;
    }

    // Clone game state for the state update task
    let update_game_state = Arc::clone(&game_state);
    let update_node_name = node_name.clone();
    let mut connection_events_clone = connection_events.resubscribe();

    // Spawn a task to periodically send state updates and connection events
    let update_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(100));

        // Keep track of the last sent state to detect changes
        let mut last_sent_state = {
            let state = update_game_state.lock().unwrap();
            json!({
                "position": {
                    "x": state.position_x,
                    "y": state.position_y
                },
                "proofStatus": state.proof_status.clone(),
                "processing": state.processing,
                "lastBatchSize": state.last_batch_size,
                "trail": state.verified_trail.clone(),
                "nodeName": update_node_name.clone(),
            })
        };

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    // Get current state
                    let current_state = {
                        let state = update_game_state.lock().unwrap();
                        json!({
                            "position": {
                                "x": state.position_x,
                                "y": state.position_y
                            },
                            "proofStatus": state.proof_status.clone(),
                            "processing": state.processing,
                            "lastBatchSize": state.last_batch_size,
                            "trail": state.verified_trail.clone(),
                            "nodeName": update_node_name.clone(),
                        })
                    };

                    // Check if state has changed
                    if current_state != last_sent_state {
                        // State has changed, send update
                        let state_json = json!({
                            "type": "state_update",
                            "position": current_state["position"],
                            "proofStatus": current_state["proofStatus"],
                            "processing": current_state["processing"],
                            "lastBatchSize": current_state["lastBatchSize"],
                            "trail": current_state["trail"],
                            "nodeName": current_state["nodeName"],
                        });

                        if let Err(e) = ws_sender.send(Message::Text(state_json.to_string())).await {
                            eprintln!("Error sending state update: {:?}", e);
                            break;
                        }

                        // Update last sent state
                        last_sent_state = current_state;
                    }
                }
                Ok(event) = connection_events_clone.recv() => {
                    // Parse the event message
                    if let Ok(event_json) = serde_json::from_str::<serde_json::Value>(&event) {
                        // Check if it's a node info event
                        if event_json["type"] == "node_info" {
                            // It's a node info event, forward it as is
                            if let Err(e) = ws_sender.send(Message::Text(event)).await {
                                eprintln!("Error sending node info event: {:?}", e);
                                break;
                            }
                        } else {
                            // It's a regular connection event
                            let event_json = json!({
                                "type": "p2p_connection",
                                "message": event
                            });

                            if let Err(e) = ws_sender.send(Message::Text(event_json.to_string())).await {
                                eprintln!("Error sending connection event: {:?}", e);
                                break;
                            }
                        }
                    } else {
                        // Couldn't parse as JSON, send as a regular connection event
                        let event_json = json!({
                            "type": "p2p_connection",
                            "message": event
                        });

                        if let Err(e) = ws_sender.send(Message::Text(event_json.to_string())).await {
                            eprintln!("Error sending connection event: {:?}", e);
                            break;
                        }
                    }
                }
            }
        }
    });

    // Process incoming messages
    while let Some(result) = ws_receiver.next().await {
        match result {
            Ok(msg) => {
                if let Message::Text(text) = msg {
                    println!("Received message: {}", text);

                    // Parse the message as JSON
                    if let Ok(json) = serde_json::from_str::<Value>(&text) {
                        if let Some(msg_type) = json["type"].as_str() {
                            match msg_type {
                                "key_press" => {
                                    if let Some(key_str) = json["key"].as_str() {
                                        let key = match key_str {
                                            "up" => KeyInput::Up,
                                            "down" => KeyInput::Down,
                                            "left" => KeyInput::Left,
                                            "right" => KeyInput::Right,
                                            "test" => KeyInput::TestConstraint,
                                            _ => KeyInput::None,
                                        };

                                        // Add the key to the pending keys queue
                                        {
                                            let mut state = game_state.lock().unwrap();
                                            state.pending_keys.push_back(key);

                                            // Update player position immediately for responsive UI
                                            let (dx, dy) = match key {
                                                KeyInput::Up => (0.0, 1.0),
                                                KeyInput::Down => (0.0, -1.0),
                                                KeyInput::Left => (-1.0, 0.0),
                                                KeyInput::Right => (1.0, 0.0),
                                                KeyInput::TestConstraint => (3.0, 3.0),
                                                KeyInput::None => (0.0, 0.0),
                                            };

                                            state.position_x += dx;
                                            state.position_y += dy;
                                        }
                                    }
                                }
                                _ => println!("Unknown message type: {}", msg_type),
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("Error receiving message: {:?}", e);
                break;
            }
        }
    }

    // Cancel the update task when the connection is closed
    update_task.abort();
    println!("WebSocket connection closed");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    println!("Welcome to Footsteps!");

    // Get node name from command line
    let node_name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "node".to_string());
    let ws_port = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "3001".to_string())
        .parse::<u16>()?;
    let p2p_port = std::env::args()
        .nth(3)
        .unwrap_or_else(|| "9000".to_string())
        .parse::<u16>()?;
    
    // Parse known peers from command line (format: host1:port1,host2:port2,...)
    let peers_arg = std::env::args().nth(4).unwrap_or_else(|| String::new());
    
    // Get custom URL from command line (for sharing with other nodes)
    let custom_url = std::env::args().nth(5).unwrap_or_else(|| String::new());
    
    let known_peers: Vec<(String, u16)> = if !peers_arg.is_empty() {
        peers_arg
            .split(',')
            .filter_map(|peer_str| {
                let parts: Vec<&str> = peer_str.split(':').collect();
                if parts.len() == 2 {
                    if let Ok(port) = parts[1].parse::<u16>() {
                        Some((parts[0].to_string(), port))
                    } else {
                        eprintln!("Invalid port in peer specification: {}", peer_str);
                        None
                    }
                } else {
                    eprintln!("Invalid peer specification: {}", peer_str);
                    None
                }
            })
            .collect()
    } else {
        Vec::new()
    };

    println!(
        "Starting {} node with WebSocket port {}, P2P port {}, and custom URL: {}",
        node_name, ws_port, p2p_port, if custom_url.is_empty() { "none" } else { &custom_url }
    );
    if !known_peers.is_empty() {
        println!("Known peers:");
        for (host, port) in &known_peers {
            println!("  {}:{}", host, port);
        }
    } else {
        println!("No known peers specified. Only local discovery will be used.");
    }

    // Initialize game state
    let game_state = Arc::new(Mutex::new(GameState::new()));

    // Start the P2P node
    let (p2p_sender, p2p_connection_rx) = p2p::start_p2p_node(
        node_name.clone(),
        Arc::clone(&game_state),
        p2p_port,
        known_peers,
        custom_url,
    )
    .await?;

    // Create a channel for sending proofs from the thread to the main async context
    let (proof_tx, mut proof_rx) = mpsc::channel::<p2p::P2PMessage>(100);
 
    // Clone p2p_sender for the async task
    let p2p_sender_clone = p2p_sender.clone();

    // Clone game state for the proof generation thread
    let proof_game_state = Arc::clone(&game_state);
    let proof_node_name = node_name.clone();

    // Spawn a thread to handle periodic proof generation
    thread::spawn(move || {
        loop {
            // Sleep for a short time to prevent CPU hogging
            thread::sleep(Duration::from_millis(100));

            // Check if it's time to process
            let now = Instant::now();
            let should_update_timer = {
                let state = proof_game_state.lock().unwrap();
                now >= state.next_process_time
            };

            if should_update_timer {
                // Update next process time
                {
                    let mut state = proof_game_state.lock().unwrap();
                    state.next_process_time = Instant::now() + Duration::from_secs(5);
                }

                // Check if there are pending key presses to process
                let (should_process, key_inputs, current_position) = {
                    let mut state = proof_game_state.lock().unwrap();

                    // Only process if there are pending keys and we're not already processing
                    let should_process = !state.pending_keys.is_empty() && !state.processing;

                    if should_process {
                        // Mark as processing to prevent concurrent processing
                        state.processing = true;
                        state.proof_status = "Generating proof...".to_string();

                        // Get all pending key presses
                        let keys: Vec<KeyInput> = state.pending_keys.drain(..).collect();
                        state.last_batch_size = keys.len();

                        // Use the proof_start position as the starting point for verification
                        // This is the position before any pending keys were applied
                        println!(
                            "Proof starting position: ({}, {})",
                            state.proof_start_x, state.proof_start_y
                        );
                        let position = (state.proof_start_x, state.proof_start_y);

                        // After processing this batch, the next proof should start from the current position
                        state.proof_start_x = state.position_x;
                        state.proof_start_y = state.position_y;

                        (true, keys, position)
                    } else {
                        (false, Vec::new(), (0.0, 0.0))
                    }
                };

                // Process the batch of key presses if needed
                if should_process {
                    println!(
                        "Processing batch of {} key presses: {:?}",
                        key_inputs.len(),
                        key_inputs
                    );

                    // Create the execution environment with the key inputs, current position, and game map
                    let env = ExecutorEnv::builder()
                        .write(&key_inputs)
                        .unwrap()
                        .write(&current_position)
                        .unwrap()
                        .build()
                        .unwrap();

                    // Get the prover inside the thread
                    let prover = default_prover();

                    println!("Generating proof for batch (this may take a while)...");
                    let start_time = Instant::now();

                    // Generate the proof
                    let receipt = match prover.prove(env, FOOTSTEPS_GUEST_ELF) {
                        Ok(receipt_result) => {
                            let elapsed = start_time.elapsed();
                            println!("Proof generated in {:.2} seconds", elapsed.as_secs_f32());

                            {
                                let mut state = proof_game_state.lock().unwrap();
                                state.processing = false;
                                state.proof_status =
                                    format!("Proof generated in {:.2}s", elapsed.as_secs_f32());
                            }

                            receipt_result.receipt
                        }
                        Err(e) => {
                            println!("Error generating proof: {:?}", e);
                            println!("This may be due to a constraint violation in one of the key presses.");

                            // Mark as no longer processing
                            let mut state = proof_game_state.lock().unwrap();
                            state.processing = false;
                            state.proof_status = "Proof failed: Constraint violation".to_string();

                            // Revert position to the last valid state (the last verified position)
                            state.position_x = state.last_verified_x;
                            state.position_y = state.last_verified_y;

                            // Don't update position for failed proofs
                            println!(
                                "Position reverted to last valid state: ({}, {})",
                                state.last_verified_x, state.last_verified_y
                            );
                            continue;
                        }
                    };

                    // send the proof to the p2p network
                    // for other players to verify
                    let p2p_msg = p2p::P2PMessage::Proof {
                        player_id: format!("{}-player", proof_node_name),
                        receipt: receipt,
                        ImageID: FOOTSTEPS_GUEST_ID,
                    };


                    // let serialized = to_vec(&receipt).unwrap();
                    // println!("Receipt size: {} bytes", serialized.len());
                    // println!(
                    //     "Receipt size: {:.2} MB",
                    //     serialized.len() as f64 / 1_048_576.0
                    // );

                    // Send the proof to the main async context
                    if let Err(e) = proof_tx.blocking_send(p2p_msg) {
                        eprintln!("Error sending proof to main context: {:?}", e);
                    }
                }
            }
        }
    });

    // Create a broadcast channel for connection events
    let (connection_tx, _) = broadcast::channel::<String>(100);

    tokio::spawn(async move {
        while let Some(proof_msg) = proof_rx.recv().await {
            if let Err(e) = p2p_sender_clone.send(proof_msg).await {
                eprintln!("Error sending to p2p: {:?}", e);

            }
        }
    });

    // Spawn a task to forward P2P connection events to the broadcast channel
    let connection_tx_clone = connection_tx.clone();
    tokio::spawn(async move {
        let mut p2p_connection_rx = p2p_connection_rx;
        while let Some(event) = p2p_connection_rx.recv().await {
            println!("P2P connection event: {}", event);
            if let Err(e) = connection_tx_clone.send(event) {
                eprintln!("Error broadcasting connection event: {:?}", e);
            }
        }
    });

    // Set up the WebSocket server
    let addr = format!("0.0.0.0:{}", ws_port);
    let listener = TcpListener::bind(&addr).await?;
    println!("WebSocket server listening on: {}", addr);
    println!("Connect your Next.js app to ws://<ip>:{}", ws_port);

    // Accept and handle WebSocket connections
    while let Ok((stream, _)) = listener.accept().await {
        let game_state_clone = Arc::clone(&game_state);
        let node_name_clone = node_name.clone();
        let connection_events = connection_tx.subscribe();

        tokio::spawn(async move {
            handle_connection(stream, game_state_clone, node_name_clone, connection_events).await;
        });
    }

    Ok(())
}
