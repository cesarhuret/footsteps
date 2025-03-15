"use client";

import React from "react";
import { useGameState } from "../hooks/useGameState";

const GameControls: React.FC = () => {
  const { movePlayer } = useGameState();

  // Handle touch controls for mobile devices
  const handleButtonPress = (dx: number, dy: number) => {
    movePlayer(dx, dy);
  };

  // Movement step size
  const MOVE_STEP = 50;

  return (
    <div className="absolute bottom-4 right-4 md:hidden">
      <div className="grid grid-cols-3 gap-2">
        {/* Top row */}
        <div className="col-start-2">
          <button
            className="w-16 h-16 bg-gray-800 rounded-full flex items-center justify-center text-white text-2xl"
            onClick={() => handleButtonPress(0, -MOVE_STEP)}
            aria-label="Move Up"
          >
            ↑
          </button>
        </div>

        {/* Middle row */}
        <div>
          <button
            className="w-16 h-16 bg-gray-800 rounded-full flex items-center justify-center text-white text-2xl"
            onClick={() => handleButtonPress(-MOVE_STEP, 0)}
            aria-label="Move Left"
          >
            ←
          </button>
        </div>
        <div>{/* Empty cell */}</div>
        <div>
          <button
            className="w-16 h-16 bg-gray-800 rounded-full flex items-center justify-center text-white text-2xl"
            onClick={() => handleButtonPress(MOVE_STEP, 0)}
            aria-label="Move Right"
          >
            →
          </button>
        </div>

        {/* Bottom row */}
        <div className="col-start-2">
          <button
            className="w-16 h-16 bg-gray-800 rounded-full flex items-center justify-center text-white text-2xl"
            onClick={() => handleButtonPress(0, MOVE_STEP)}
            aria-label="Move Down"
          >
            ↓
          </button>
        </div>
      </div>
    </div>
  );
};

export default GameControls; 