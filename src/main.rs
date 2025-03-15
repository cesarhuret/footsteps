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

use footsteps_core::Outputs;
use footsteps_methods::{FOOTSTEPS_GUEST_ELF, FOOTSTEPS_GUEST_ID};
use risc0_zkvm::{default_prover, ExecutorEnv};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

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

// Define the same GameMap struct as in the guest code
#[derive(Debug, Serialize, Deserialize)]
struct GameMap {
}

// Current position state shared between Bevy and proof generation thread
struct GameState {
    position_x: f32,
    position_y: f32,
    last_verified_x: f32,  // Last position verified by ZK proof
    last_verified_y: f32,  // Last position verified by ZK proof
    proof_start_x: f32,    // Starting position for the next proof
    proof_start_y: f32,    // Starting position for the next proof
    pending_keys: VecDeque<KeyInput>,
    processing: bool,
    next_process_time: Instant,
    proof_status: String,
    last_batch_size: usize,
    verified_trail: Vec<(f32, f32)>, // Trail verified by ZK proof (excluding final position)
}

fn main() {
    // Print debug information about the environment
    println!("Starting RISC Zero Bevy Demo");
    println!("OS: {}", std::env::consts::OS);
    println!("Environment variables:");
    for (key, value) in std::env::vars() {
        if key.contains("DISPLAY") || key.contains("WAYLAND") || key.contains("XDG") || key.contains("WSL") {
            println!("  {} = {}", key, value);
        }
    }
    
    // Initialize game state
    let game_state = Arc::new(Mutex::new(GameState {
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
    }));
    
    // Clone game state for the proof generation thread
    let proof_game_state = Arc::clone(&game_state);
    
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
                    println!("Proof starting position: ({}, {})", state.proof_start_x, state.proof_start_y);
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
                    println!("Processing batch of {} key presses: {:?}", key_inputs.len(), key_inputs);
                
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
                                state.proof_status = format!("Proof generated in {:.2}s", elapsed.as_secs_f32());
                            }
                            
                        receipt_result.receipt
                    },
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
                            println!("Position reverted to last valid state: ({}, {})", state.last_verified_x, state.last_verified_y);
                        continue;
                    }
                };
                
                println!("Verifying proof...");
                    {
                        let mut state = proof_game_state.lock().unwrap();
                        state.proof_status = "Verifying proof...".to_string();
                    }
                
                // Verify the proof
                if let Err(e) = receipt.verify(FOOTSTEPS_GUEST_ID) {
                    println!("Error verifying proof: {:?}", e);
                    
                    // Mark as no longer processing
                    let mut state = proof_game_state.lock().unwrap();
                    state.processing = false;
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
                        let mut state = proof_game_state.lock().unwrap();
                        state.processing = false;
                            state.proof_status = "Journal decoding failed".to_string();
                        
                        continue;
                    }
                };
                
                // Update game state
                    let mut state: std::sync::MutexGuard<'_, GameState> = proof_game_state.lock().unwrap();
                    
                    // Get the trail length before moving it
                    let trail_len = outputs.trail_positions.len();
                    let trail_summary = format!("{:?}", outputs.trail_positions);
                    
                    // Update the verified trail - make a deep copy to ensure it's a new object
                    state.verified_trail = outputs.trail_positions.clone();
                    
                    state.processing = false;
                    state.proof_status = format!("Proof verified! Trail: {} positions", trail_len);
                    
                    println!("Batch processed! Trail verified with {} positions: {}", 
                             trail_len, trail_summary);
                    
                    // Force immediate update of the trail
                    drop(state); // Release the lock before sleeping
                    
                    // Small delay to ensure the trail update is processed
                    thread::sleep(Duration::from_millis(50));
                }
            }
        }
    });
}
