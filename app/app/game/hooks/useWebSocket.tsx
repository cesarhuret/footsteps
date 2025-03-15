"use client";

import { useState, useEffect, useCallback, useRef } from 'react';

interface WebSocketState {
  position: {
    x: number;
    y: number;
  };
  proofStatus: string;
  processing: boolean;
  lastBatchSize: number;
  trail: [number, number][];
}

// Constants for grid scaling
const GRID_SCALE_FACTOR = 50; // Each game grid unit = 20 ZK proof grid units

// Grid limits (in game coordinates)
const GRID_LIMIT_X = 720;
const GRID_LIMIT_Y = 520;

export const useWebSocket = () => {
  const [connected, setConnected] = useState(false);
  const [gameState, setGameState] = useState<WebSocketState>({
    position: { x: 0, y: 0 },
    proofStatus: "Connecting...",
    processing: false,
    lastBatchSize: 0,
    trail: []
  });
  
  const socketRef = useRef<WebSocket | null>(null);
  
  // Initialize WebSocket connection
  useEffect(() => {
    const connectWebSocket = () => {
      // Use environment variable with fallback to specific IP
      const wsUrl = process.env.NEXT_PUBLIC_WS_URL || 'ws://172.21.137.205:3001';
      const ws = new WebSocket(wsUrl);

      console.log('Connecting to WebSocket at:', wsUrl);


      ws.onopen = () => {
        console.log('WebSocket connected');
        setConnected(true);
      };
      
      ws.onclose = () => {
        console.log('WebSocket disconnected');
        setConnected(false);
        
        // Try to reconnect after a delay
        setTimeout(connectWebSocket, 2000);
      };
      
      ws.onerror = (error) => {
        console.error('WebSocket error:', error);
      };
      
      ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          
          if (data.type === 'state_update') {

            console.log("state_update", data);
            // Scale the position coordinates from ZK proof grid to game grid
            // and enforce grid limits
            const scaledX = data.position.x * GRID_SCALE_FACTOR;
            // Note: The server is still using the original coordinate system (0,0 at top-left)
            // We need to keep the same coordinate system for internal state
            const scaledY = data.position.y * GRID_SCALE_FACTOR;
            
            // Log trail data for debugging
            if (data.trail) {
              console.log(`Received trail data with ${data.trail.length} points`);
            }
            
            const scaledTrail = data.trail ? data.trail.map(([x, y]: [number, number]) => [
              Math.min(GRID_LIMIT_X, Math.max(0, x * GRID_SCALE_FACTOR)),
              Math.min(GRID_LIMIT_Y, Math.max(0, y * GRID_SCALE_FACTOR))
            ] as [number, number]) : [];
            
            // Log the scaled trail
            if (scaledTrail.length > 0) {
              console.log(`Scaled trail has ${scaledTrail.length} points. First point: ${scaledTrail[0]}`);
            }
            
            setGameState({
              position: {
                x: Math.min(GRID_LIMIT_X, Math.max(0, scaledX)),
                y: Math.min(GRID_LIMIT_Y, Math.max(0, scaledY))
              },
              proofStatus: data.proofStatus,
              processing: data.processing,
              lastBatchSize: data.lastBatchSize,
              // Scale the trail coordinates as well
              trail: scaledTrail
            });
          }
        } catch (error) {
          console.error('Error parsing WebSocket message:', error);
        }
      };
      
      socketRef.current = ws;
    };
    
    connectWebSocket();
    
    // Clean up WebSocket connection on unmount
    return () => {
      if (socketRef.current) {
        socketRef.current.close();
      }
    };
  }, []);
  
  // Function to send key presses to the server
  const sendKeyPress = useCallback((key: 'up' | 'down' | 'left' | 'right' | 'test') => {
    if (socketRef.current && socketRef.current.readyState === WebSocket.OPEN) {
      // The server still expects the original key commands
      // The scaling is handled when receiving the updated position
      const message = JSON.stringify({
        type: 'key_press',
        key
      });
      socketRef.current.send(message);
      
      // For a more responsive feel, we can update the local state immediately
      // This is just a visual update, the actual position will be corrected when the server responds
      if (!gameState.processing) {
        
        const newPosition = { ...gameState.position };
       
        setGameState(prev => ({
          ...prev,
          position: newPosition
        }));
      }
    }
  }, [gameState]);
  
  return {
    connected,
    gameState,
    sendKeyPress
  };
}; 