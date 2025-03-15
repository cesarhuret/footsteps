"use client";

import React from "react";
import { useWebSocket } from "../hooks/useWebSocket";

const GameControls: React.FC = () => {
  const { sendKeyPress } = useWebSocket();

  // Handle touch controls for mobile devices
  const handleButtonPress = (direction: 'up' | 'down' | 'left' | 'right' | 'test') => {
    sendKeyPress(direction);
  };

  return (
    <div className="absolute bottom-4 right-4 md:hidden">
      <div className="grid grid-cols-3 gap-2">
        {/* Top row */}
        <div className="col-start-2">
          <button
            className="w-16 h-16 bg-gray-800 rounded-full flex items-center justify-center text-white text-2xl"
            onClick={() => handleButtonPress('up')}
            aria-label="Move Up"
          >
            ↑
          </button>
        </div>

        {/* Middle row */}
        <div>
          <button
            className="w-16 h-16 bg-gray-800 rounded-full flex items-center justify-center text-white text-2xl"
            onClick={() => handleButtonPress('left')}
            aria-label="Move Left"
          >
            ←
          </button>
        </div>
        <div>{/* Empty cell */}</div>
        <div>
          <button
            className="w-16 h-16 bg-gray-800 rounded-full flex items-center justify-center text-white text-2xl"
            onClick={() => handleButtonPress('right')}
            aria-label="Move Right"
          >
            →
          </button>
        </div>

        {/* Bottom row */}
        <div className="col-start-2">
          <button
            className="w-16 h-16 bg-gray-800 rounded-full flex items-center justify-center text-white text-2xl"
            onClick={() => handleButtonPress('down')}
            aria-label="Move Down"
          >
            ↓
          </button>
        </div>
      </div>
      
      {/* Test constraint button */}
      <div className="mt-2">
        <button
          className="w-full h-12 bg-red-800 rounded-lg flex items-center justify-center text-white"
          onClick={() => handleButtonPress('test')}
          aria-label="Test Constraint"
        >
          Test Constraint
        </button>
      </div>
    </div>
  );
};

export default GameControls; 