"use client";

import React, { useState } from "react";
import GameCanvas from "./components/GameCanvas";
import GameControls from "./components/GameControls";
import { Inter } from 'next/font/google';
import Link from "next/link";


const inter = Inter({
  subsets: ['latin'],
  variable: '--font-inter',
})


export default function GamePage() {
  const [showInstructions, setShowInstructions] = useState(true);

  return (
    <div className={`flex flex-col items-center justify-center min-h-screen bg-black p-4 ${inter.variable} font-sans`}>
      <div className="w-full max-w-4xl">
        <div className="flex justify-between items-center mb-6">
          <h1 className="text-4xl font-bold text-white">Footsteps</h1>
          <Link 
            href="/" 
            className="px-4 py-2 bg-gray-800 text-white rounded-lg hover:bg-gray-700 transition-colors"
          >
            Back to Home
          </Link>
        </div>

        <div className="relative w-[750px] h-[550px] mx-auto bg-black rounded-lg overflow-hidden shadow-2xl border border-gray-800">
          <GameCanvas />
          <GameControls />
          
          {showInstructions && (
            <div className="absolute inset-0 bg-black bg-opacity-80 flex items-center justify-center p-8">
              <div className="bg-gray-900 p-6 rounded-lg max-w-md">
                <h2 className="text-2xl font-bold text-white mb-4">How to Play</h2>
                <ul className="text-gray-300 space-y-2 mb-6">
                  <li>• Use <span className="text-blue-400">WASD</span> or <span className="text-blue-400">Arrow Keys</span> to move</li>
                  <li>• Explore the 750x550 grid world</li>
                  <li>• On mobile, use the on-screen controls</li>
                  <li>• Watch the trail effect as you move around</li>
                  <li>• Stay within the red boundary</li>
                </ul>
                <button 
                  className="w-full py-3 bg-blue-600 text-white font-medium rounded-lg hover:bg-blue-700 transition-colors"
                  onClick={() => setShowInstructions(false)}
                >
                  Start Playing
                </button>
              </div>
            </div>
          )}
        </div>
        
        <div className="mt-6 flex justify-between">
          <div className="text-gray-300">
            <p>Use arrow keys or WASD to move the player around. Game area: 750x550</p>
          </div>
          <button 
            className="px-4 py-2 bg-gray-800 text-white rounded-lg hover:bg-gray-700 transition-colors"
            onClick={() => setShowInstructions(true)}
          >
            Show Instructions
          </button>
        </div>
      </div>
    </div>
  );
} 