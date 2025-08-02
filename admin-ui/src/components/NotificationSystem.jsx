import React from 'react';
import { X, CheckCircle, AlertCircle, Info, AlertTriangle } from 'lucide-react';
import { Alert, AlertDescription } from './ui/alert';
import { Button } from './ui/button';
import { useApp } from '../contexts/AppContext';

export const NotificationSystem = () => {
  const { notifications, removeNotification } = useApp();

  const getIcon = (type) => {
    switch (type) {
      case 'success':
        return <CheckCircle className="w-4 h-4 text-green-500" />;
      case 'error':
        return <AlertCircle className="w-4 h-4 text-red-500" />;
      case 'warning':
        return <AlertTriangle className="w-4 h-4 text-yellow-500" />;
      default:
        return <Info className="w-4 h-4 text-blue-500" />;
    }
  };

  const getVariant = (type) => {
    switch (type) {
      case 'error':
        return 'destructive';
      default:
        return 'default';
    }
  };

  if (notifications.length === 0) {
    return null;
  }

  return (
    <div className="fixed top-4 right-4 z-50 space-y-2 max-w-md">
      {notifications.map((notification) => (
        <Alert key={notification.id} variant={getVariant(notification.type)} className="relative">
          <div className="flex items-start space-x-2">
            {getIcon(notification.type)}
            <div className="flex-1">
              {notification.title && (
                <h4 className="text-sm font-medium mb-1">{notification.title}</h4>
              )}
              <AlertDescription className="text-sm">
                {notification.message}
              </AlertDescription>
            </div>
            <Button
              variant="ghost"
              size="sm"
              className="h-auto p-1"
              onClick={() => removeNotification(notification.id)}
            >
              <X className="w-3 h-3" />
            </Button>
          </div>
        </Alert>
      ))}
    </div>
  );
};

export default NotificationSystem;