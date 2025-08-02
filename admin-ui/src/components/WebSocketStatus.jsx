import React from 'react';
import { Wifi, WifiOff, Clock, Users } from 'lucide-react';
import { Badge } from './ui/badge';
import { Button } from './ui/button';
import { useWebSocketContext } from '../contexts/WebSocketContext';

export const WebSocketStatus = ({ className = '' }) => {
  const { 
    connectionStatus, 
    connectionTime, 
    connect, 
    disconnect 
  } = useWebSocketContext();

  const getStatusColor = () => {
    switch (connectionStatus) {
      case 'Connected':
        return 'success';
      case 'Connecting':
        return 'secondary';
      case 'Disconnected':
        return 'outline';
      case 'Error':
        return 'destructive';
      default:
        return 'outline';
    }
  };

  const getStatusIcon = () => {
    switch (connectionStatus) {
      case 'Connected':
        return <Wifi className="w-3 h-3" />;
      case 'Connecting':
        return <Wifi className="w-3 h-3 animate-pulse" />;
      default:
        return <WifiOff className="w-3 h-3" />;
    }
  };

  const formatConnectionTime = () => {
    if (!connectionTime) return null;
    
    const now = new Date();
    const diff = Math.floor((now - connectionTime) / 1000);
    
    if (diff < 60) return `${diff}s`;
    if (diff < 3600) return `${Math.floor(diff / 60)}m`;
    return `${Math.floor(diff / 3600)}h`;
  };

  return (
    <div className={`flex items-center space-x-2 ${className}`}>
      <Badge variant={getStatusColor()} className="flex items-center space-x-1">
        {getStatusIcon()}
        <span className="text-xs">{connectionStatus}</span>
      </Badge>
      
      {connectionTime && (
        <div className="flex items-center space-x-1 text-xs text-gray-500">
          <Clock className="w-3 h-3" />
          <span>{formatConnectionTime()}</span>
        </div>
      )}
      
      {connectionStatus === 'Disconnected' && (
        <Button 
          size="sm" 
          variant="outline" 
          onClick={connect}
          className="text-xs h-6 px-2"
        >
          Reconnect
        </Button>
      )}
      
      {connectionStatus === 'Connected' && (
        <Button 
          size="sm" 
          variant="outline" 
          onClick={disconnect}
          className="text-xs h-6 px-2"
        >
          Disconnect
        </Button>
      )}
    </div>
  );
};

export default WebSocketStatus;