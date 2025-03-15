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
      const ws = new WebSocket('ws://127.0.0.1:3001');
      
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
            setGameState({
              position: data.position,
              proofStatus: data.proofStatus,
              processing: data.processing,
              lastBatchSize: data.lastBatchSize,
              trail: data.trail
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
      const message = JSON.stringify({
        type: 'key_press',
        key
      });
      socketRef.current.send(message);
    }
  }, []);
  
  return {
    connected,
    gameState,
    sendKeyPress
  };
}; 