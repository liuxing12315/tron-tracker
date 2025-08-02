import apiClient from './api.js';

/**
 * Transaction-related API services
 */
export const transactionService = {
  /**
   * Get transactions with filters and pagination
   */
  async getTransactions(filters = {}, pagination = {}) {
    const params = {
      page: pagination.page || 1,
      limit: pagination.limit || 20,
      ...filters
    };
    
    const response = await apiClient.get('/transactions', params);
    return response.data || response;
  },

  /**
   * Get single transaction by hash
   */
  async getTransactionByHash(hash) {
    const response = await apiClient.get(`/transactions/${hash}`);
    return response.data || response;
  },

  /**
   * Search transactions by multiple criteria
   */
  async searchTransactions(query, filters = {}, pagination = {}) {
    const params = {
      q: query,
      page: pagination.page || 1,
      limit: pagination.limit || 20,
      ...filters
    };
    
    const response = await apiClient.get('/transactions/search', params);
    return response.data || response;
  },

  /**
   * Get transactions for multiple addresses
   */
  async getMultiAddressTransactions(addresses, filters = {}, pagination = {}) {
    const data = {
      addresses,
      ...filters,
      page: pagination.page || 1,
      limit: pagination.limit || 20,
    };
    
    const response = await apiClient.post('/transactions/multi-address', data);
    return response.data || response;
  },

  /**
   * Get address statistics
   */
  async getAddressStatistics(address) {
    const response = await apiClient.get(`/addresses/${address}/stats`);
    return response.data || response;
  },

  /**
   * Get batch address statistics
   */
  async getBatchAddressStatistics(addresses) {
    const response = await apiClient.post('/addresses/batch-stats', { addresses });
    return response.data || response;
  },

  /**
   * Export transactions to CSV/JSON
   */
  async exportTransactions(filters = {}, format = 'csv') {
    const params = { ...filters, format };
    const response = await apiClient.get('/transactions/export', params);
    return response;
  }
};

export default transactionService;