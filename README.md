# Footsteps

A serverless peer-to-peer multiplayer game using zero-knowledge proofs for decentralized gameplay verification.

## Overview

Footsteps is an innovative multiplayer game that eliminates the need for centralized game servers by leveraging ZK proofs shared across a peer-to-peer network. Players directly interact with each other while validating player moves according to the game logic without revealing their complete position.

Every 5 seconds, the game generates and shares proofs of player movements, revealing only 50% of moves to other players. This creates an exciting gameplay dynamic where players have hints of others' locations without complete visibility.

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

## Requirements

- Rust
- Node.js
- Self Protocol account for verification
