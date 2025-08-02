import { useState, useEffect, useCallback, useRef } from 'react';
import { useApp } from '../contexts/AppContext';

export const useAsync = (asyncFunction, dependencies = [], options = {}) => {
  const {
    immediate = true,
    onSuccess,
    onError,
    successMessage,
    errorMessage,
    loadingKey
  } = options;

  const [data, setData] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const { addNotification, setLoading: setGlobalLoading } = useApp();
  const cancelRef = useRef();

  const execute = useCallback(async (...args) => {
    try {
      setLoading(true);
      setError(null);
      
      if (loadingKey) {
        setGlobalLoading(loadingKey, true);
      }

      // Cancel previous request if it exists
      if (cancelRef.current) {
        cancelRef.current();
      }

      let cancelled = false;
      cancelRef.current = () => {
        cancelled = true;
      };

      const result = await asyncFunction(...args);

      if (!cancelled) {
        setData(result);
        
        if (onSuccess) {
          onSuccess(result);
        }
        
        if (successMessage) {
          addNotification({
            type: 'success',
            message: successMessage
          });
        }
      }

      return result;
    } catch (err) {
      if (!cancelled) {
        setError(err);
        
        if (onError) {
          onError(err);
        } else {
          addNotification({
            type: 'error',
            title: 'Error',
            message: errorMessage || err.message || 'An error occurred'
          });
        }
      }
      throw err;
    } finally {
      if (!cancelled) {
        setLoading(false);
        
        if (loadingKey) {
          setGlobalLoading(loadingKey, false);
        }
      }
    }
  }, dependencies);

  const reset = useCallback(() => {
    setData(null);
    setError(null);
    setLoading(false);
    
    if (loadingKey) {
      setGlobalLoading(loadingKey, false);
    }
  }, [loadingKey, setGlobalLoading]);

  useEffect(() => {
    if (immediate) {
      execute();
    }

    return () => {
      if (cancelRef.current) {
        cancelRef.current();
      }
    };
  }, dependencies);

  return {
    data,
    loading,
    error,
    execute,
    reset
  };
};

export default useAsync;