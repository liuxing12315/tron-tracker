import React, { createContext, useContext, useReducer, useEffect } from 'react';
import { systemService } from '../services';

const AppContext = createContext();

export const useApp = () => {
  const context = useContext(AppContext);
  if (!context) {
    throw new Error('useApp must be used within an AppProvider');
  }
  return context;
};

const initialState = {
  systemStats: {
    totalTransactions: 0,
    totalAddresses: 0,
    currentBlock: 0,
    scanSpeed: 0,
    activeWebhooks: 0,
    websocketConnections: 0,
    apiRequestsToday: 0,
    successRate: 0,
    uptime: 0
  },
  transactions: [],
  addresses: [],
  webhooks: [],
  apiKeys: [],
  logs: [],
  websocketConnections: [],
  loading: {
    systemStats: false,
    transactions: false,
    addresses: false,
    webhooks: false,
    apiKeys: false,
    logs: false,
    websockets: false
  },
  errors: {},
  notifications: [],
  filters: {
    transactions: {},
    addresses: {},
    webhooks: {},
    logs: {}
  },
  pagination: {
    transactions: { page: 1, limit: 20, total: 0 },
    addresses: { page: 1, limit: 20, total: 0 },
    webhooks: { page: 1, limit: 20, total: 0 },
    logs: { page: 1, limit: 50, total: 0 }
  }
};

const appReducer = (state, action) => {
  switch (action.type) {
    case 'SET_LOADING':
      return {
        ...state,
        loading: { ...state.loading, [action.key]: action.value }
      };

    case 'SET_ERROR':
      return {
        ...state,
        errors: { ...state.errors, [action.key]: action.error }
      };

    case 'CLEAR_ERROR':
      const { [action.key]: removed, ...remainingErrors } = state.errors;
      return {
        ...state,
        errors: remainingErrors
      };

    case 'SET_SYSTEM_STATS':
      return {
        ...state,
        systemStats: action.stats
      };

    case 'SET_TRANSACTIONS':
      return {
        ...state,
        transactions: action.transactions,
        pagination: {
          ...state.pagination,
          transactions: action.pagination || state.pagination.transactions
        }
      };

    case 'SET_ADDRESSES':
      return {
        ...state,
        addresses: action.addresses,
        pagination: {
          ...state.pagination,
          addresses: action.pagination || state.pagination.addresses
        }
      };

    case 'SET_WEBHOOKS':
      return {
        ...state,
        webhooks: action.webhooks,
        pagination: {
          ...state.pagination,
          webhooks: action.pagination || state.pagination.webhooks
        }
      };

    case 'SET_API_KEYS':
      return {
        ...state,
        apiKeys: action.apiKeys
      };

    case 'SET_LOGS':
      return {
        ...state,
        logs: action.logs,
        pagination: {
          ...state.pagination,
          logs: action.pagination || state.pagination.logs
        }
      };

    case 'SET_WEBSOCKET_CONNECTIONS':
      return {
        ...state,
        websocketConnections: action.connections
      };

    case 'SET_FILTERS':
      return {
        ...state,
        filters: { ...state.filters, [action.key]: action.filters }
      };

    case 'SET_PAGINATION':
      return {
        ...state,
        pagination: { ...state.pagination, [action.key]: action.pagination }
      };

    case 'ADD_NOTIFICATION':
      return {
        ...state,
        notifications: [...state.notifications, action.notification]
      };

    case 'REMOVE_NOTIFICATION':
      return {
        ...state,
        notifications: state.notifications.filter(n => n.id !== action.id)
      };

    case 'UPDATE_TRANSACTION':
      return {
        ...state,
        transactions: state.transactions.map(t => 
          t.hash === action.transaction.hash ? action.transaction : t
        )
      };

    case 'UPDATE_WEBHOOK':
      return {
        ...state,
        webhooks: state.webhooks.map(w => 
          w.id === action.webhook.id ? action.webhook : w
        )
      };

    case 'DELETE_WEBHOOK':
      return {
        ...state,
        webhooks: state.webhooks.filter(w => w.id !== action.webhookId)
      };

    case 'UPDATE_API_KEY':
      return {
        ...state,
        apiKeys: state.apiKeys.map(k => 
          k.id === action.apiKey.id ? action.apiKey : k
        )
      };

    case 'DELETE_API_KEY':
      return {
        ...state,
        apiKeys: state.apiKeys.filter(k => k.id !== action.apiKeyId)
      };

    default:
      return state;
  }
};

export const AppProvider = ({ children }) => {
  const [state, dispatch] = useReducer(appReducer, initialState);

  // System stats auto-refresh
  useEffect(() => {
    const fetchSystemStats = async () => {
      dispatch({ type: 'SET_LOADING', key: 'systemStats', value: true });
      dispatch({ type: 'CLEAR_ERROR', key: 'systemStats' });
      
      try {
        const stats = await systemService.getSystemStats();
        dispatch({ type: 'SET_SYSTEM_STATS', stats });
      } catch (error) {
        dispatch({ type: 'SET_ERROR', key: 'systemStats', error: error.message });
      } finally {
        dispatch({ type: 'SET_LOADING', key: 'systemStats', value: false });
      }
    };

    fetchSystemStats();
    const interval = setInterval(fetchSystemStats, 30000);
    return () => clearInterval(interval);
  }, []);

  // Notification management
  const addNotification = (notification) => {
    const id = Date.now().toString();
    dispatch({ 
      type: 'ADD_NOTIFICATION', 
      notification: { ...notification, id, timestamp: new Date() }
    });

    // Auto-remove after 5 seconds for non-error notifications
    if (notification.type !== 'error') {
      setTimeout(() => {
        dispatch({ type: 'REMOVE_NOTIFICATION', id });
      }, 5000);
    }
  };

  const removeNotification = (id) => {
    dispatch({ type: 'REMOVE_NOTIFICATION', id });
  };

  // Action creators
  const actions = {
    setLoading: (key, value) => dispatch({ type: 'SET_LOADING', key, value }),
    setError: (key, error) => dispatch({ type: 'SET_ERROR', key, error }),
    clearError: (key) => dispatch({ type: 'CLEAR_ERROR', key }),
    setTransactions: (transactions, pagination) => dispatch({ type: 'SET_TRANSACTIONS', transactions, pagination }),
    setAddresses: (addresses, pagination) => dispatch({ type: 'SET_ADDRESSES', addresses, pagination }),
    setWebhooks: (webhooks, pagination) => dispatch({ type: 'SET_WEBHOOKS', webhooks, pagination }),
    setApiKeys: (apiKeys) => dispatch({ type: 'SET_API_KEYS', apiKeys }),
    setLogs: (logs, pagination) => dispatch({ type: 'SET_LOGS', logs, pagination }),
    setWebsocketConnections: (connections) => dispatch({ type: 'SET_WEBSOCKET_CONNECTIONS', connections }),
    setFilters: (key, filters) => dispatch({ type: 'SET_FILTERS', key, filters }),
    setPagination: (key, pagination) => dispatch({ type: 'SET_PAGINATION', key, pagination }),
    updateTransaction: (transaction) => dispatch({ type: 'UPDATE_TRANSACTION', transaction }),
    updateWebhook: (webhook) => dispatch({ type: 'UPDATE_WEBHOOK', webhook }),
    deleteWebhook: (webhookId) => dispatch({ type: 'DELETE_WEBHOOK', webhookId }),
    updateApiKey: (apiKey) => dispatch({ type: 'UPDATE_API_KEY', apiKey }),
    deleteApiKey: (apiKeyId) => dispatch({ type: 'DELETE_API_KEY', apiKeyId }),
    addNotification,
    removeNotification
  };

  const value = {
    ...state,
    ...actions,
    dispatch
  };

  return (
    <AppContext.Provider value={value}>
      {children}
    </AppContext.Provider>
  );
};

export default AppContext;