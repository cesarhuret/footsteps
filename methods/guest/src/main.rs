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

use risc0_zkvm::guest::env;

use footsteps_core::Outputs;
use bevy_ecs::{prelude::*, world::World};
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Copy)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Component, Clone, Copy)]
struct Velocity {
    x: f32,
    y: f32,
}


#[derive(StageLabel)]
pub struct UpdateLabel;

// This system moves each entity with a Position and Velocity component
// Modified to ensure movement is exactly 1 block at a time
fn movement(mut param_set: ParamSet<(
    Query<(&mut Position, &Velocity)>,
)>) {
    // Then process movement
    for (mut position, velocity) in &mut param_set.p0() {
        // Check for constraint violation (movement must be exactly 1 block)
        if velocity.x.abs() > 1.1 || velocity.y.abs() > 1.1 {
            // This will cause the proof to fail
            // env::log(&format!("CONSTRAINT VIOLATION: Movement must be exactly 1 block at a time. Attempted: ({}, {})", velocity.x, velocity.y));
            // Use panic! instead of env::fail() to abort execution
            panic!("CONSTRAINT VIOLATION: Movement must be exactly 1 block at a time");
        }
        
        // Normalize movement to exactly 1 block
        if velocity.x != 0.0 || velocity.y != 0.0 {
            // Get direction
            let dx = if velocity.x > 0.0 { 1.0 } else if velocity.x < 0.0 { -1.0 } else { 0.0 };
            let dy = if velocity.y > 0.0 { 1.0 } else if velocity.y < 0.0 { -1.0 } else { 0.0 };
            
            // Calculate new position
            let new_x = position.x + dx;
            let new_y = position.y + dy;
            
            position.x = new_x;
            position.y = new_y;
        }
    }
}

// Define key input enum
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


fn main() {
    // Read key inputs
    let key_inputs: Vec<KeyInput> = env::read();
    
    // Read the current position from the host
    let (start_x, start_y): (f32, f32) = env::read();
    
    let mut world = World::new();
    
    // Spawn player
    let entity = world
        .spawn((Position { x: start_x, y: start_y }, Velocity { x: 0.0, y: 0.0 }))
        .id();

    let mut schedule = Schedule::default();

    schedule.add_stage(
        UpdateLabel,
        SystemStage::single_threaded()
            .with_system(movement)
    );
    
    // Track all positions for the movement trail
    let mut all_positions = Vec::with_capacity(key_inputs.len() + 1);
    
    // Add starting position
    all_positions.push(Position { x: start_x, y: start_y });
    
    // Process each key input
    for key in key_inputs {
        // Update velocity based on key input
        {
            let mut entity_mut = world.entity_mut(entity);
            let mut velocity = entity_mut.get_mut::<Velocity>().unwrap();
            
            // Reset velocity
            velocity.x = 0.0;
            velocity.y = 0.0;
            
            // Set velocity based on key input
            match key {
                KeyInput::Up => velocity.y = 1.0,
                KeyInput::Down => velocity.y = -1.0,
                KeyInput::Left => velocity.x = -1.0,
                KeyInput::Right => velocity.x = 1.0,
                KeyInput::TestConstraint => {
                    // Try to move by 3 units (should violate constraints)
                    velocity.x = 3.0;
                    env::log("Attempting to move by 3 units (should violate constraints and cause panic)");
                    env::log("This will trigger the constraint check in the movement system");
                },
                KeyInput::None => (), // No movement
            }
        }
        
        // Run a single timestep
        schedule.run(&mut world);
        
        // Record position after movement
        let entity_ref = world.entity(entity);
        let position = entity_ref.get::<Position>().unwrap();
        
        // Add current position to all positions if we moved
        let velocity = entity_ref.get::<Velocity>().unwrap();
        if velocity.x != 0.0 || velocity.y != 0.0 {
            all_positions.push((*position).clone());
        }
    }
    
    // Select the middle sequence of the trail
    let trail_positions = if all_positions.len() <= 1 {
        // If there's only the starting position or no movement, return empty trail
        Vec::new()
    } else if all_positions.len() <= 4 {
        // If there are 2-4 positions (including start), return all except the last one
        all_positions[0..all_positions.len()-1].iter().map(|p| (p.x, p.y)).collect()
    } else {
        // For longer trails, select the middle 50% of the trail

        let middle_index = all_positions.len() / 2;
        let start_index = middle_index / 2;
        let end_index = middle_index + start_index;
        
        // Ensure we don't go out of bounds
        let start_index = start_index.max(0);
        let end_index = end_index.min(all_positions.len() - 1);
        
        // Extract the middle sequence
        all_positions[start_index..end_index].iter().map(|p| (p.x, p.y)).collect()
    };
    
    // Output only the selected trail, not the final position
    {
        let out = Outputs {
            trail_positions,
        };
        env::commit(&out);
    }
}
