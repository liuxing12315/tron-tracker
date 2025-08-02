import { useState, useEffect, useRef, useCallback } from 'react';
import { useApp } from '../contexts/AppContext';

export const useWebSocket = (url, options = {}) => {
  const {
    onOpen,
    onMessage,
    onClose,
    onError,
    autoReconnect = true,
    reconnectInterval = 3000,
    maxReconnectAttempts = 5,
    protocols = []
  } = options;

  const [connectionStatus, setConnectionStatus] = useState('Disconnected');
  const [lastMessage, setLastMessage] = useState(null);
  const [connectionTime, setConnectionTime] = useState(null);
  const { addNotification } = useApp();
  
  const websocketRef = useRef(null);
  const reconnectTimeoutRef = useRef(null);
  const reconnectAttemptsRef = useRef(0);
  const urlRef = useRef(url);

  // Update URL ref when URL changes
  useEffect(() => {
    urlRef.current = url;
  }, [url]);

  const connect = useCallback(() => {
    if (websocketRef.current?.readyState === WebSocket.OPEN) {
      return;
    }

    try {
      const fullUrl = urlRef.current.startsWith('ws') 
        ? urlRef.current 
        : `${window.location.protocol === 'https:' ? 'wss:' : 'ws:'}//${window.location.host}${urlRef.current}`;
      
      websocketRef.current = new WebSocket(fullUrl, protocols);
      setConnectionStatus('Connecting');

      websocketRef.current.onopen = (event) => {
        setConnectionStatus('Connected');
        setConnectionTime(new Date());
        reconnectAttemptsRef.current = 0;
        
        addNotification({
          type: 'success',
          message: 'WebSocket connected'
        });

        if (onOpen) {
          onOpen(event);
        }
      };

      websocketRef.current.onmessage = (event) => {
        let data;
        try {
          data = JSON.parse(event.data);
        } catch (error) {
          data = event.data;
        }
        
        setLastMessage({
          data,
          timestamp: new Date(),
          raw: event.data
        });

        if (onMessage) {
          onMessage(data, event);
        }
      };

      websocketRef.current.onclose = (event) => {
        setConnectionStatus('Disconnected');
        setConnectionTime(null);

        if (onClose) {
          onClose(event);
        }

        // Auto-reconnect logic
        if (autoReconnect && reconnectAttemptsRef.current < maxReconnectAttempts) {
          reconnectAttemptsRef.current += 1;
          
          addNotification({
            type: 'warning',
            message: `WebSocket disconnected. Reconnecting... (${reconnectAttemptsRef.current}/${maxReconnectAttempts})`
          });

          reconnectTimeoutRef.current = setTimeout(() => {
            connect();
          }, reconnectInterval);
        } else if (reconnectAttemptsRef.current >= maxReconnectAttempts) {
          addNotification({
            type: 'error',
            title: 'Connection Failed',
            message: 'Maximum reconnection attempts reached'
          });
        }
      };

      websocketRef.current.onerror = (event) => {
        setConnectionStatus('Error');
        
        addNotification({
          type: 'error',
          title: 'WebSocket Error',
          message: 'Connection error occurred'
        });

        if (onError) {
          onError(event);
        }
      };

    } catch (error) {
      setConnectionStatus('Error');
      addNotification({
        type: 'error',
        title: 'Connection Error',
        message: error.message
      });
    }
  }, [autoReconnect, maxReconnectAttempts, reconnectInterval, onOpen, onMessage, onClose, onError, protocols, addNotification]);

  const disconnect = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
    }
    
    if (websocketRef.current) {
      websocketRef.current.close();
    }
    
    setConnectionStatus('Disconnected');
    setConnectionTime(null);
  }, []);

  const sendMessage = useCallback((message) => {
    if (websocketRef.current?.readyState === WebSocket.OPEN) {
      const messageString = typeof message === 'string' ? message : JSON.stringify(message);
      websocketRef.current.send(messageString);
      return true;
    } else {
      addNotification({
        type: 'error',
        message: 'WebSocket is not connected'
      });
      return false;
    }
  }, [addNotification]);

  const sendJsonMessage = useCallback((object) => {
    return sendMessage(JSON.stringify(object));
  }, [sendMessage]);

  // Connect on mount, disconnect on unmount
  useEffect(() => {
    if (url) {
      connect();
    }

    return () => {
      disconnect();
    };
  }, [url, connect, disconnect]);

  return {
    sendMessage,
    sendJsonMessage,
    lastMessage,
    connectionStatus,
    connectionTime,
    connect,
    disconnect,
    websocket: websocketRef.current
  };
};

export default useWebSocket;