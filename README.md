![pixil-frame-0 (2)](https://github.com/user-attachments/assets/62ebd306-8d52-4d97-9c6d-6e498a032862)
# Footsteps

A serverless peer-to-peer multiplayer game using zero-knowledge proofs for decentralized gameplay verification.

## Overview

Footsteps is an innovative multiplayer game that eliminates the need for centralized game servers by leveraging ZK proofs shared across a peer-to-peer network. Players directly interact with each other while validating player moves according to the game logic without revealing their complete position.

Every 5 seconds, the game generates and shares proofs of player movements, revealing only 50% of moves to other players. This creates an exciting gameplay dynamic where players have hints of others' locations without complete visibility.

![Screenshot 2025-03-15 122825](https://github.com/user-attachments/assets/843de1f9-1334-43ba-ae60-e8df58cdb0d7)

## Key Features

- **Serverless Architecture**: No central server required to validate game logic or host lobbies
- **Near Real-Time Multiplayer**: ~3 second delay for proof generation and verification
- **Sybil Resistant**: Integrates Self Protocol for human verification to prevent bots
- **Privacy-Preserving Gameplay**: Only reveals partial player movements (50%)

## Technology Stack

- **Risc0**: Zero-knowledge proof system for game logic verification
- **libp2p**: Peer-to-peer networking for direct player connections
- **Self Protocol**: Human verification system to ensure bot-free lobbies
- **Rust**: Backend server handling p2p connections, WebSocket, and ZK proofs
- **Next.js**: Frontend web application for game visuals

## Architecture

The game client for each player is divided into two parts:

1. **Rust Backend**:
   - libp2p node for peer-to-peer connections
   - WebSocket server for communication with frontend
   - Risc0 prover and verifier for game logic

2. **Next.js Frontend**:
   - Game visuals and controls
   - Self Protocol backend verifier
   - QR code display for verification

## How It Works

1. Players connect to each other via libp2p
2. Self Protocol verifier URLs are exchanged to prove human identity
3. Players move freely in the game environment
4. Every 5 seconds, ZK proofs of movements are generated
5. Proofs are shared with and verified by other players
6. Only 50% of movements are revealed to other players
7. Invalid proofs trigger position rollbacks

## Human Verification
![Screenshot 2025-03-15 122918](https://github.com/user-attachments/assets/4b7e84e3-8426-43c6-a6e6-fb1a9191b664)

Self Protocol integration ensures that players:
- Are verified humans
- Are above 18 years of age
- Are not on the OFAC list

In each lobby, all players have the same verification requirements. New players must prove they meet the criteria before joining, and existing players verify newcomers are valid.

## Unique Innovations

- **Real-time ZK-Based Gameplay**: Unlike turn-based ZK games, Footsteps implements near real-time gameplay with 5-second proof batching
- **Decentralized Player Verification**: Sybil resistance through ZK proofs of identity
- **Privacy-Preserving Movement System**: Partial information revelation creates unique gameplay dynamics

## Future Development

- Implement shooting/flashlight mechanics where players can request proofs to determine if other players were within a flashlight's path
- Further optimize proof generation and verification time
- Expand the game mechanics while maintaining the serverless architecture

## Getting Started
Kinda complicated but

- install the submodules
- build risc0
- install the /app dependencies
- build the rust server using
  
  ```RISC0_PROVER=local RUSTFLAGS="-C target-cpu=native" cargo build --release --features cuda,prove```

- run the demo script using

  ```
  # first client 
  ./run_demo.sh desktop 3003 3004 9001 
  # second client
  ./run_demo.sh laptop 3002 3001 9000 <first_client_ip>:9001
  ```
  ![image](https://github.com/user-attachments/assets/f0584cbb-6475-4fb6-a171-10aeb6057d35)


## Requirements

- Rust
- Node.js
- Self Protocol account for verification
