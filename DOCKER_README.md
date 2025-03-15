# Docker Setup for Footsteps Game

This README provides instructions for running the Footsteps game using Docker and Docker Compose.

## Prerequisites

- [Docker](https://docs.docker.com/get-docker/)
- [Docker Compose](https://docs.docker.com/compose/install/)

## Project Structure

The project consists of two main components:

1. **Rust Backend**: A Rust server that handles game logic, P2P networking, and ZK proof generation/verification.
2. **Next.js Frontend**: A web-based UI for the game.

## Docker Configuration

The project includes the following Docker-related files:

- `Dockerfile.rust`: Builds the Rust backend server
- `Dockerfile.nextjs`: Builds the Next.js frontend
- `docker-compose.yml`: Orchestrates the services

## Running the Application

### Start the Services

```bash
docker-compose up -d
```

This will start:
- A Rust server node on port 3001
- A desktop node on port 3002
- The Next.js app on port 3000

### Access the Application

- Open your browser and navigate to `http://localhost:3000` to access the Next.js frontend.
- The frontend will automatically connect to the Rust backend via WebSocket.

### View Logs

```bash
# View logs for all services
docker-compose logs -f

# View logs for a specific service
docker-compose logs -f rust-server
docker-compose logs -f desktop-node
docker-compose logs -f nextjs-app
```

### Stop the Services

```bash
docker-compose down
```

## Testing P2P Functionality

The Docker Compose setup includes two Rust nodes:

1. `rust-server`: The main server node
2. `desktop-node`: A secondary node for testing P2P functionality

To test the P2P functionality:
1. Open two browser windows
2. In the first window, connect to `http://localhost:3000` (connects to rust-server)
3. In the second window, connect to `http://localhost:3002` directly (connects to desktop-node)
4. Actions performed in one window should be reflected in the other through the P2P network

## Customization

You can modify the Docker Compose configuration to:
- Change port mappings
- Add environment variables
- Configure volumes for persistent data
- Scale services

## Troubleshooting

If you encounter issues:

1. Check the logs using `docker-compose logs`
2. Ensure all required ports are available
3. Try rebuilding the images with `docker-compose build --no-cache`
4. Restart the services with `docker-compose restart` 