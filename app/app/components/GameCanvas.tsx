"use client";

import React, { useEffect, useRef, useState } from "react";
import { useWebSocket } from "../hooks/useWebSocket";

// Define types for our game objects
interface GameObject {
  x: number;
  y: number;
  width: number;
  height: number;
  color: string;
}

interface Player extends GameObject {
  speed: number;
  targetX: number;
  targetY: number;
}


interface TrailPoint {
  x: number;
  y: number;
  age: number;
  maxAge: number;
}

const GameCanvas: React.FC = () => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const { connected, gameState, sendKeyPress } = useWebSocket();
  const [canvasSize, setCanvasSize] = useState({ width: 750, height: 550 });
  const [playerImage, setPlayerImage] = useState<HTMLImageElement | null>(null);
  const [trail, setTrail] = useState<TrailPoint[]>([]);
  
  // Create a player object from the WebSocket gameState
  const player: Player = {
    x: gameState.position.x,
    y: gameState.position.y,
    targetX: gameState.position.x,
    targetY: gameState.position.y,
    width: 40,
    height: 40,
    color: "#3B82F6", // Blue
    speed: 25 * 5, // Increased speed to match the scaled grid
  };
  
  // Previous position for trail calculation
  const prevPosRef = useRef({ x: 0, y: 0 });

  // Load player image
  useEffect(() => {
    const img = new Image();
    img.src = '/hat.png';
    img.onload = () => {
      setPlayerImage(img);
    };
  }, []);

  // Handle window resize
  useEffect(() => {
    const handleResize = () => {
      // Set canvas size to match grid limits plus some padding
      setCanvasSize({
        width: 750, // Slightly larger than GRID_LIMIT_X (720)
        height: 550, // Slightly larger than GRID_LIMIT_Y (520)
      });
    };

    handleResize();
    window.addEventListener("resize", handleResize);
    return () => window.removeEventListener("resize", handleResize);
  }, []);

  // Update trail
  useEffect(() => {
    // Only add trail points if player has moved significantly
    const distMoved = Math.sqrt(
      Math.pow(player.x - prevPosRef.current.x, 2) +
      Math.pow(player.y - prevPosRef.current.y, 2)
    );
    
    if (distMoved > 5) {
      // Add new trail point
      setTrail(prevTrail => [
        ...prevTrail,
        {
          x: player.x + player.width / 2,
          y: player.y + player.height / 2,
          age: 0,
          maxAge: 30, // How long trail points last
        }
      ]);
      
      // Update previous position
      prevPosRef.current = {
        x: player.x,
        y: player.y
      };
    }
    
    // Age and remove old trail points
    setTrail(prevTrail => 
      prevTrail
        .map(point => ({ ...point, age: point.age + 1 }))
        .filter(point => point.age < point.maxAge)
    );
  }, [player.x, player.y]);

  // Game rendering loop
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    // Set canvas dimensions
    canvas.width = canvasSize.width;
    canvas.height = canvasSize.height;

    // Clear the canvas
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // Draw the game world (grid background)
    drawGameWorld(ctx, canvas.width, canvas.height);
    
    // Draw the player
    drawPlayer(ctx, player, playerImage, canvas.height);

    // Draw game stats
    drawGameStats(ctx, { player, connected, proofStatus: gameState.proofStatus }, canvas.height);
    
    // Draw verified trail from WebSocket if available
    if (gameState.trail && gameState.trail.length > 0) {
      drawVerifiedTrail(ctx, gameState.trail, canvas.height);
    }
  }, [gameState, canvasSize, playerImage, trail, connected, player]);

  // Handle keyboard input
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      switch (e.key) {
        case "ArrowUp":
        case "w":
        case "W":
          sendKeyPress("up");
          break;
        case "ArrowDown":
        case "s":
        case "S":
          sendKeyPress("down");
          break;
        case "ArrowLeft":
        case "a":
        case "A":
          sendKeyPress("left");
          break;
        case "ArrowRight":
        case "d":
        case "D":
          sendKeyPress("right");
          break;
        case "t":
        case "T":
          sendKeyPress("test");
          break;
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [sendKeyPress]);

  return (
    <canvas
      ref={canvasRef}
      className="absolute top-0 left-0 w-full h-full"
    />
  );
};

// Helper functions for drawing
function drawGameWorld(
  ctx: CanvasRenderingContext2D,
  width: number,
  height: number
) {
  // Draw grid background
  ctx.fillStyle = "#000";
  ctx.fillRect(0, 0, width, height);

  // Draw grid lines
  ctx.strokeStyle = "#1F2937";
  ctx.lineWidth = 1;

  // Grid size should match the scaling factor
  const GRID_SCALE_FACTOR = 50; // Must match the value in useWebSocket.tsx
  const gridSize = GRID_SCALE_FACTOR; // Grid size matches the scaling factor
  
  // Grid limits
  const GRID_LIMIT_X = 750;
  const GRID_LIMIT_Y = 550;
  
  // Vertical lines
  for (let x = 0; x <= width; x += gridSize) {
    ctx.beginPath();
    ctx.moveTo(x, 0);
    ctx.lineTo(x, height);
    ctx.stroke();
  }
  
  // Horizontal lines
  for (let y = 0; y <= height; y += gridSize) {
    ctx.beginPath();
    ctx.moveTo(0, height - y);
    ctx.lineTo(width, height - y);
    ctx.stroke();
  }
  
  // Draw boundary indicators
  ctx.strokeStyle = "#1F2937";
  ctx.lineWidth = 2;
  ctx.strokeRect(0, 0, width, height);
  
  // Draw grid limit boundary
  ctx.strokeStyle = "#FF5555";
  ctx.lineWidth = 2;
  ctx.strokeRect(0, height - GRID_LIMIT_Y, GRID_LIMIT_X, GRID_LIMIT_Y);
}



// Draw the verified trail from the ZK proof
function drawVerifiedTrail(ctx: CanvasRenderingContext2D, trail: [number, number][], canvasHeight: number) {
  
  trail.forEach((point, index) => {
    const opacity = (index / (trail.length * 1.5)) + 0.6;
    ctx.fillStyle = `rgba(0, 255, 0, ${opacity})`;
    ctx.beginPath();
    ctx.arc(point[0] + 20, canvasHeight - point[1] - 20, 10 * opacity, 0, Math.PI * 2);
    ctx.fill();

    // disappear after 30 frames
  });
}

function drawObject(ctx: CanvasRenderingContext2D, object: GameObject, canvasHeight: number) {
  ctx.fillStyle = object.color;
  ctx.fillRect(object.x, canvasHeight - object.y - object.height, object.width, object.height);
}

function drawPlayer(
  ctx: CanvasRenderingContext2D, 
  player: Player, 
  playerImage: HTMLImageElement | null,
  canvasHeight: number
) {
  // Draw player shadow
  ctx.fillStyle = "rgba(0, 0, 0, 0.3)";
  ctx.beginPath();
  ctx.ellipse(
    player.x + player.width / 2,
    canvasHeight - player.y - player.height + 5,
    player.width / 2,
    player.width / 4,
    0,
    0,
    Math.PI * 2
  );
  ctx.fill();

  if (playerImage) {
    // Draw the player image
    ctx.drawImage(
      playerImage,
      player.x,
      canvasHeight - player.y - player.height,
      player.width,
      player.height
    );
  } else {
    // Fallback to a colored rectangle if image isn't loaded
    drawObject(ctx, player, canvasHeight);
  }
}

function drawGameStats(
  ctx: CanvasRenderingContext2D,
  gameState: { player: Player, connected: boolean, proofStatus: string },
  canvasHeight: number
) {
  // Constants
  const GRID_SCALE_FACTOR = 50; // Must match the value in useWebSocket.tsx
  const GRID_LIMIT_X = 720;
  const GRID_LIMIT_Y = 520;
  const ZK_LIMIT_X = GRID_LIMIT_X / GRID_SCALE_FACTOR;
  const ZK_LIMIT_Y = GRID_LIMIT_Y / GRID_SCALE_FACTOR;
  
  ctx.fillStyle = "#FFF";
  ctx.font = "14px Arial";
  ctx.textAlign = "left";
  
  // Draw game position (scaled)
  ctx.fillText(
    `Game Position: (${Math.round(gameState.player.x)}, ${Math.round(gameState.player.y)}) / (${GRID_LIMIT_X}, ${GRID_LIMIT_Y})`,
    10,
    20
  );
  
  // Draw ZK grid position (unscaled)
  ctx.fillText(
    `ZK Grid Position: (${Math.round(gameState.player.x / GRID_SCALE_FACTOR)}, ${Math.round(gameState.player.y / GRID_SCALE_FACTOR)}) / (${ZK_LIMIT_X}, ${ZK_LIMIT_Y})`,
    10,
    40
  );
  
  // Draw connection status
  ctx.fillStyle = gameState.connected ? "#4ADE80" : "#EF4444";
  ctx.fillText(
    `Connection: ${gameState.connected ? "Connected" : "Disconnected"}`,
    10,
    60
  );
  
  // Draw proof status
  ctx.fillStyle = "#FFF";
  ctx.fillText(
    `Proof Status: ${gameState.proofStatus}`,
    10,
    80
  );
}

export default GameCanvas; 