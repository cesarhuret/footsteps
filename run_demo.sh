#!/bin/bash

# Check if port number is provided
if [ $# -lt 1 ]; then
    echo "Usage: $0 <node_name> <app_port> <our_ws_port_number> [our_p2p_port] [their_p2p_address]"
    echo "Example: $0 node1 3000 3001 9000 127.0.0.1:9001"
    exit 1
fi

NODE_NAME=$1  # Create a unique node name based on port
APP_PORT=$2
WS_PORT=$3
P2P_PORT=${4:-9000}  # Default P2P port to 9000 if not provided
THEIR_P2P_ADDRESS=${5:-"127.0.0.1:9001"}

# Function to cleanup background processes on script exit
cleanup() {
    echo "Cleaning up..."
    # Kill any remaining ngrok processes
    pkill -f ngrok
    # Kill the web app
    kill $APP_PID 2>/dev/null
    # Kill the Rust server
    kill $RUST_PID 2>/dev/null
    exit 0
}

# Set up cleanup trap
trap cleanup EXIT

# Check if pnpm is installed
if ! command -v pnpm &> /dev/null; then
    echo "pnpm is not installed. Please install it first."
    exit 1
fi

# Check if we're in the right directory structure
if [ ! -d "app" ]; then
    echo "Error: 'app' directory not found. Please run this script from the project root."
    exit 1
fi

echo "Starting web app on port $APP_PORT..."
cd app

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo "Installing dependencies..."
    pnpm install
fi

# Start the web app
pnpm run dev --port $APP_PORT > ../app.log 2>&1 &
APP_PID=$!

# Wait for the web app to start (check if it's responding)
echo "Waiting for web app to start..."
MAX_ATTEMPTS=30
ATTEMPT=0

while [ $ATTEMPT -lt $MAX_ATTEMPTS ]; do
    if curl -s "http://localhost:$APP_PORT" > /dev/null; then
        echo "Web app is running!"
        break
    fi
    ATTEMPT=$((ATTEMPT + 1))
    sleep 1
    echo -n "."
done

if [ $ATTEMPT -eq $MAX_ATTEMPTS ]; then
    echo "Failed to start web app after $MAX_ATTEMPTS seconds"
    exit 1
fi

cd ..

echo "Starting ngrok..."
# Start ngrok in the background
ngrok http $APP_PORT > ngrok.log 2>&1 &
NGROK_PID=$!

# Wait for ngrok to start and get the URL
echo "Waiting for ngrok to start..."
MAX_ATTEMPTS=30
ATTEMPT=0
NGROK_URL=""

while [ $ATTEMPT -lt $MAX_ATTEMPTS ]; do
    # Try both the API method and log file method to get the URL
    NGROK_URL=$(curl -s http://localhost:4040/api/tunnels | jq -r '.tunnels[0].public_url' 2>/dev/null)
    
    if [ "$NGROK_URL" = "null" ] || [ -z "$NGROK_URL" ]; then
        NGROK_URL=$(grep -o "https://.*\.ngrok-free\.app" ngrok.log 2>/dev/null | head -n 1)
    fi
    
    if [ ! -z "$NGROK_URL" ] && [ "$NGROK_URL" != "null" ]; then
        echo "ngrok URL: $NGROK_URL"
        break
    fi
    
    ATTEMPT=$((ATTEMPT + 1))
    sleep 1
    echo -n "."
done

if [ -z "$NGROK_URL" ] || [ "$NGROK_URL" = "null" ]; then
    echo "Failed to get ngrok URL after $MAX_ATTEMPTS seconds"
    exit 1
fi

# Start the Rust server with the ngrok URL
echo "Starting Rust server..."
RISC0_PROVER=local ./target/release/footsteps "$NODE_NAME" "$WS_PORT" "$P2P_PORT" "$THEIR_P2P_ADDRESS" "$NGROK_URL" &
RUST_PID=$!

# Wait for the Rust server to exit
wait $RUST_PID

# Wait for the web app to exit
wait $APP_PID
