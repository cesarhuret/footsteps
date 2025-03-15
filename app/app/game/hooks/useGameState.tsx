"use client";

import { useState, useCallback, useEffect, useRef } from "react";

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
  targetX: number; // Target X position for interpolation
  targetY: number; // Target Y position for interpolation
}

interface GameState {
  player: Player;
  gameArea: {
    width: number;
    height: number;
  };
}

export const useGameState = () => {
  // Initialize game state
  const [gameState, setGameState] = useState<GameState>({
    player: {
      x: 100,
      y: 100,
      targetX: 100, // Initialize target position same as current
      targetY: 100,
      width: 40,
      height: 40,
      color: "#3B82F6", // Blue
      speed: 25,
    },
    gameArea: {
      width: 750, // Fixed width for the game area
      height: 550, // Fixed height for the game area
    },
  });

  // Animation frame ID for cleanup
  const animationFrameRef = useRef<number | null>(null);

  // Update game area dimensions based on window size, but capped at 750x550
  useEffect(() => {
    const updateGameArea = () => {
      setGameState((prevState) => ({
        ...prevState,
        gameArea: {
          // Always use fixed dimensions of 750x550
          width: 750,
          height: 550,
        },
      }));
    };

    updateGameArea();
    window.addEventListener("resize", updateGameArea);
    return () => window.removeEventListener("resize", updateGameArea);
  }, []);

  // Move player function - now sets target position
  const movePlayer = useCallback((dx: number, dy: number) => {
    setGameState((prevState) => {
      // Calculate new target position
      const newTargetX = prevState.player.targetX + dx;
      const newTargetY = prevState.player.targetY + dy;

      // Create a temporary player object with the new target position
      const tempPlayer = {
        ...prevState.player,
        targetX: newTargetX,
        targetY: newTargetY,
      };

      // Check boundary collisions - ensure player stays within the game area
      // Allow player to move right up to the edge (not beyond)
      const isOutOfBounds =
        newTargetX < 0 ||
        newTargetX + prevState.player.width > prevState.gameArea.width ||
        newTargetY < 0 ||
        newTargetY + prevState.player.height > prevState.gameArea.height;

      // If out of bounds, adjust the target position to be at the boundary
      if (isOutOfBounds) {
        const adjustedTargetX = Math.max(
          0,
          Math.min(
            newTargetX,
            prevState.gameArea.width - prevState.player.width
          )
        );
        const adjustedTargetY = Math.max(
          0,
          Math.min(
            newTargetY,
            prevState.gameArea.height - prevState.player.height
          )
        );

        return {
          ...prevState,
          player: {
            ...prevState.player,
            targetX: adjustedTargetX,
            targetY: adjustedTargetY,
          },
        };
      }

      return {
        ...prevState,
        player: {
          ...prevState.player,
          targetX: newTargetX,
          targetY: newTargetY,
        },
      };
    });
  }, []);

  // Interpolation animation loop
  useEffect(() => {
    const interpolationSpeed = 0.1; // Adjust for faster/slower interpolation (0-1)

    const updatePlayerPosition = () => {
      setGameState((prevState) => {
        // Calculate distance between current and target positions
        const dx = prevState.player.targetX - prevState.player.x;
        const dy = prevState.player.targetY - prevState.player.y;

        // If we're very close to the target, snap to it
        if (Math.abs(dx) < 0.1 && Math.abs(dy) < 0.1) {
          return {
            ...prevState,
            player: {
              ...prevState.player,
              x: prevState.player.targetX,
              y: prevState.player.targetY,
            },
          };
        }

        // Otherwise, move a percentage of the way there
        return {
          ...prevState,
          player: {
            ...prevState.player,
            x: prevState.player.x + dx * interpolationSpeed,
            y: prevState.player.y + dy * interpolationSpeed,
          },
        };
      });

      // Continue animation loop
      animationFrameRef.current = requestAnimationFrame(updatePlayerPosition);
    };

    // Start animation loop
    animationFrameRef.current = requestAnimationFrame(updatePlayerPosition);

    // Cleanup animation loop on unmount
    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    };
  }, []);

  return {
    gameState,
    movePlayer,
  };
};
