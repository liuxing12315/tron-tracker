import React, { createContext, useContext, useEffect } from 'react';
import { useWebSocket } from '../hooks/useWebSocket';
import { useAuth } from './AuthContext';
import { useApp } from './AppContext';

const WebSocketContext = createContext();

export const useWebSocketContext = () => {
  const context = useContext(WebSocketContext);
  if (!context) {
    throw new Error('useWebSocketContext must be used within a WebSocketProvider');
  }
  return context;
};

export const WebSocketProvider = ({ children }) => {
  const { isAuthenticated, apiKey } = useAuth();
  const { 
    updateTransaction, 
    setWebsocketConnections, 
    addNotification,
    systemStats,
    setSystemStats 
  } = useApp();

  // Build WebSocket URL with authentication
  const wsUrl = isAuthenticated && apiKey 
    ? `/ws?api_key=${encodeURIComponent(apiKey)}`
    : null;

  const {
    sendMessage,
    sendJsonMessage,
    lastMessage,
    connectionStatus,
    connectionTime,
    connect,
    disconnect
  } = useWebSocket(wsUrl, {
    onMessage: (data) => {
      handleWebSocketMessage(data);
    },
    onOpen: () => {
      // Subscribe to relevant channels
      subscribeToChannels();
    },
    autoReconnect: true,
    reconnectInterval: 3000,
    maxReconnectAttempts: 5
  });

  const handleWebSocketMessage = (data) => {
    try {
      switch (data.type) {
        case 'transaction_update':
          updateTransaction(data.transaction);
          addNotification({
            type: 'info',
            title: 'Transaction Update',
            message: `Transaction ${data.transaction.hash.substring(0, 8)}... updated`
          });
          break;

        case 'new_transaction':
          addNotification({
            type: 'success',
            title: 'New Transaction',
            message: `New transaction detected: ${data.transaction.hash.substring(0, 8)}...`
          });
          break;

        case 'webhook_status':
          addNotification({
            type: data.status === 'success' ? 'success' : 'error',
            title: 'Webhook Status',
            message: `Webhook ${data.webhook_id}: ${data.status}`
          });
          break;

        case 'scanner_status':
          addNotification({
            type: 'info',
            title: 'Scanner Update',
            message: `Scanner ${data.status}: Block ${data.block_number}`
          });
          break;

        case 'system_stats':
          setSystemStats(data.stats);
          break;

        case 'websocket_connections':
          setWebsocketConnections(data.connections);
          break;

        case 'error':
          addNotification({
            type: 'error',
            title: 'WebSocket Error',
            message: data.message
          });
          break;

        case 'pong':
          // Handle heartbeat response
          break;

        default:
          console.log('Unknown WebSocket message type:', data.type);
      }
    } catch (error) {
      console.error('Error handling WebSocket message:', error);
    }
  };

  const subscribeToChannels = () => {
    // Subscribe to system events
    sendJsonMessage({
      type: 'subscribe',
      channels: [
        'transactions',
        'webhooks',
        'scanner',
        'system_stats',
        'websocket_connections'
      ]
    });
  };

  const unsubscribeFromChannels = () => {
    sendJsonMessage({
      type: 'unsubscribe',
      channels: ['all']
    });
  };

  // Subscribe to specific transaction hash
  const subscribeToTransaction = (hash) => {
    sendJsonMessage({
      type: 'subscribe_transaction',
      hash
    });
  };

  // Subscribe to specific address
  const subscribeToAddress = (address) => {
    sendJsonMessage({
      type: 'subscribe_address',
      address
    });
  };

  // Send heartbeat to keep connection alive
  useEffect(() => {
    if (connectionStatus === 'Connected') {
      const interval = setInterval(() => {
        sendJsonMessage({ type: 'ping' });
      }, 30000); // Send ping every 30 seconds

      return () => clearInterval(interval);
    }
  }, [connectionStatus, sendJsonMessage]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (connectionStatus === 'Connected') {
        unsubscribeFromChannels();
      }
    };
  }, []);

  const value = {
    sendMessage,
    sendJsonMessage,
    lastMessage,
    connectionStatus,
    connectionTime,
    connect,
    disconnect,
    subscribeToTransaction,
    subscribeToAddress,
    subscribeToChannels,
    unsubscribeFromChannels
  };

  return (
    <WebSocketContext.Provider value={value}>
      {children}
    </WebSocketContext.Provider>
  );
};

export default WebSocketContext;