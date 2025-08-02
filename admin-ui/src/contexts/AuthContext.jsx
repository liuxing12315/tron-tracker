import React, { createContext, useContext, useState, useEffect } from 'react';
import { apiKeyService } from '../services';

const AuthContext = createContext();

export const useAuth = () => {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
};

export const AuthProvider = ({ children }) => {
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [apiKey, setApiKey] = useState(null);
  const [permissions, setPermissions] = useState([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    checkAuth();
  }, []);

  const checkAuth = async () => {
    try {
      const savedApiKey = localStorage.getItem('api_key');
      if (savedApiKey) {
        const result = await apiKeyService.testApiKey(savedApiKey);
        if (result.valid) {
          setApiKey(savedApiKey);
          setIsAuthenticated(true);
          // Get permissions from API key details
          const keyDetails = await apiKeyService.getApiKey(savedApiKey);
          setPermissions(keyDetails.permissions || []);
        } else {
          localStorage.removeItem('api_key');
        }
      }
    } catch (error) {
      localStorage.removeItem('api_key');
    } finally {
      setLoading(false);
    }
  };

  const login = async (key) => {
    try {
      const result = await apiKeyService.testApiKey(key);
      if (result.valid) {
        localStorage.setItem('api_key', key);
        setApiKey(key);
        setIsAuthenticated(true);
        
        // Get permissions
        const keyDetails = await apiKeyService.getApiKey(key);
        setPermissions(keyDetails.permissions || []);
        
        return { success: true };
      } else {
        return { success: false, error: 'Invalid API key' };
      }
    } catch (error) {
      return { success: false, error: error.message };
    }
  };

  const logout = () => {
    localStorage.removeItem('api_key');
    setApiKey(null);
    setIsAuthenticated(false);
    setPermissions([]);
  };

  const hasPermission = (permission) => {
    return permissions.includes('admin') || permissions.includes(permission);
  };

  const value = {
    isAuthenticated,
    apiKey,
    permissions,
    loading,
    login,
    logout,
    hasPermission,
    checkAuth
  };

  return (
    <AuthContext.Provider value={value}>
      {children}
    </AuthContext.Provider>
  );
};

export default AuthContext;