"use client";

import React, { useEffect, useRef, useState } from "react";
import { useGameState } from "../hooks/useGameState";

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

interface Obstacle extends GameObject {}

interface TrailPoint {
  x: number;
  y: number;
  age: number;
  maxAge: number;
}

const GameCanvas: React.FC = () => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const { gameState, movePlayer } = useGameState();
  const [canvasSize, setCanvasSize] = useState({ width: 750, height: 550 });
  const [playerImage, setPlayerImage] = useState<HTMLImageElement | null>(null);
  const [trail, setTrail] = useState<TrailPoint[]>([]);
  
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
      // Always use fixed dimensions of 750x550
      setCanvasSize({
        width: 750,
        height: 550,
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
      Math.pow(gameState.player.x - prevPosRef.current.x, 2) +
      Math.pow(gameState.player.y - prevPosRef.current.y, 2)
    );
    
    if (distMoved > 5) {
      // Add new trail point
      setTrail(prevTrail => [
        ...prevTrail,
        {
          x: gameState.player.x + gameState.player.width / 2,
          y: gameState.player.y + gameState.player.height / 2,
          age: 0,
          maxAge: 30, // How long trail points last
        }
      ]);
      
      // Update previous position
      prevPosRef.current = {
        x: gameState.player.x,
        y: gameState.player.y
      };
    }
    
    // Age and remove old trail points
    setTrail(prevTrail => 
      prevTrail
        .map(point => ({ ...point, age: point.age + 1 }))
        .filter(point => point.age < point.maxAge)
    );
  }, [gameState.player.x, gameState.player.y]);

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
    
    // Draw trail
    drawTrail(ctx, trail);

    // Draw obstacles
    gameState.obstacles.forEach((obstacle: Obstacle) => {
      drawObject(ctx, obstacle);
    });

    // Draw the player
    drawPlayer(ctx, gameState.player, playerImage);

    // Draw game stats
    drawGameStats(ctx, gameState);
  }, [gameState, canvasSize, playerImage, trail]);

  // Handle keyboard input
  useEffect(() => {
    const MOVE_STEP = 50; // Match the step size from GameControls
    
    const handleKeyDown = (e: KeyboardEvent) => {
      switch (e.key) {
        case "ArrowUp":
        case "w":
        case "W":
          movePlayer(0, -MOVE_STEP);
          break;
        case "ArrowDown":
        case "s":
        case "S":
          movePlayer(0, MOVE_STEP);
          break;
        case "ArrowLeft":
        case "a":
        case "A":
          movePlayer(-MOVE_STEP, 0);
          break;
        case "ArrowRight":
        case "d":
        case "D":
          movePlayer(MOVE_STEP, 0);
          break;
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [movePlayer]);

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

  const gridSize = 30; // Smaller grid size
  
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
    ctx.moveTo(0, y);
    ctx.lineTo(width, y);
    ctx.stroke();
  }
  
  // Draw boundary indicators
  ctx.strokeStyle = "#1F2937";
  ctx.lineWidth = 2;
  ctx.strokeRect(0, 0, width, height);
}

function drawTrail(ctx: CanvasRenderingContext2D, trail: TrailPoint[]) {
  trail.forEach(point => {
    const opacity = 1 - point.age / point.maxAge;
    const size = 10 * (1 - point.age / point.maxAge);
    
    ctx.fillStyle = `rgba(59, 130, 246, ${opacity})`;
    ctx.beginPath();
    ctx.arc(point.x, point.y, size, 0, Math.PI * 2);
    ctx.fill();
  });
}

function drawObject(ctx: CanvasRenderingContext2D, object: GameObject) {
  ctx.fillStyle = object.color;
  ctx.fillRect(object.x, object.y, object.width, object.height);
}

function drawPlayer(
  ctx: CanvasRenderingContext2D, 
  player: Player, 
  playerImage: HTMLImageElement | null
) {
  // Draw player shadow
  ctx.fillStyle = "rgba(0, 0, 0, 0.3)";
  ctx.beginPath();
  ctx.ellipse(
    player.x + player.width / 2,
    player.y + player.height + 5,
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
      player.y,
      player.width,
      player.height
    );
  } else {
    // Fallback to rectangle if image not loaded
    ctx.fillStyle = player.color;
    ctx.fillRect(player.x, player.y, player.width, player.height);
    
    // Draw player details (face)
    ctx.fillStyle = "#111827";
    
    // Eyes
    const eyeSize = player.width / 6;
    const eyeY = player.y + player.height / 3;
    
    // Left eye
    ctx.fillRect(
      player.x + player.width / 4 - eyeSize / 2,
      eyeY,
      eyeSize,
      eyeSize
    );
    
    // Right eye
    ctx.fillRect(
      player.x + (player.width * 3) / 4 - eyeSize / 2,
      eyeY,
      eyeSize,
      eyeSize
    );
    
    // Mouth
    ctx.fillRect(
      player.x + player.width / 4,
      player.y + player.height * 0.6,
      player.width / 2,
      player.height / 10
    );
  }
}

function drawGameStats(
  ctx: CanvasRenderingContext2D,
  gameState: { player: Player; obstacles: Obstacle[] }
) {
  ctx.fillStyle = "white";
  ctx.font = "16px Arial";
  ctx.fillText(`X: ${Math.round(gameState.player.x)}`, 10, 20);
  ctx.fillText(`Y: ${Math.round(gameState.player.y)}`, 10, 40);
  ctx.fillText(`Max: 750 x 550`, 10, 60);
}

export default GameCanvas; 