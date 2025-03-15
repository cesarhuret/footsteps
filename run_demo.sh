#!/bin/bash

# Kill any existing processes on these ports
kill $(lsof -t -i:3001) 2>/dev/null
kill $(lsof -t -i:3002) 2>/dev/null

# Build the application
cargo build

# Run the first node in the background
echo "Starting desktop node on port 3001..."
cargo run -- desktop 3001 &
DESKTOP_PID=$!

# Wait a bit for the first node to start
sleep 2

# Run the second node
echo "Starting laptop node on port 3002..."
cargo run -- laptop 3002

# When the second node is terminated, kill the first node
kill $DESKTOP_PID 