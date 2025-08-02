import { useCallback } from 'react';
import { useApp } from '../contexts/AppContext';

export const useAsyncError = () => {
  const { addNotification } = useApp();

  const throwError = useCallback((error) => {
    throw error;
  }, []);

  const handleError = useCallback((error, context = 'operation') => {
    console.error(`Error in ${context}:`, error);
    
    addNotification({
      type: 'error',
      title: 'Error',
      message: error.message || `An error occurred during ${context}`
    });
  }, [addNotification]);

  return { throwError, handleError };
};

export default useAsyncError;