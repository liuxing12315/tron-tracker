import apiClient from './api.js';

/**
 * Log management services
 */
export const logService = {
  /**
   * Get system logs with filters and pagination
   */
  async getLogs(filters = {}, pagination = {}) {
    const params = {
      page: pagination.page || 1,
      limit: pagination.limit || 50,
      ...filters
    };
    
    const response = await apiClient.get('/admin/logs', params);
    return response.data || response;
  },

  /**
   * Search logs by query
   */
  async searchLogs(query, filters = {}, pagination = {}) {
    const params = {
      q: query,
      page: pagination.page || 1,
      limit: pagination.limit || 50,
      ...filters
    };
    
    const response = await apiClient.get('/admin/logs/search', params);
    return response.data || response;
  },

  /**
   * Export logs to file
   */
  async exportLogs(filters = {}, format = 'csv') {
    const params = { ...filters, format };
    const response = await apiClient.get('/admin/logs/export', params);
    return response;
  },

  /**
   * Clear logs based on criteria
   */
  async clearLogs(criteria = {}) {
    const response = await apiClient.post('/admin/logs/clear', criteria);
    return response.data || response;
  },

  /**
   * Get log statistics
   */
  async getLogStats(timeRange = '24h') {
    const params = { range: timeRange };
    const response = await apiClient.get('/admin/logs/stats', params);
    return response.data || response;
  },

  /**
   * Get available log levels
   */
  getLogLevels() {
    return [
      { value: 'error', label: 'Error', color: 'red' },
      { value: 'warn', label: 'Warning', color: 'yellow' },
      { value: 'info', label: 'Info', color: 'blue' },
      { value: 'debug', label: 'Debug', color: 'gray' }
    ];
  },

  /**
   * Get available modules
   */
  getLogModules() {
    return [
      { value: 'scanner', label: 'Scanner' },
      { value: 'webhook', label: 'Webhook Service' },
      { value: 'api', label: 'API Server' },
      { value: 'database', label: 'Database' },
      { value: 'websocket', label: 'WebSocket Service' },
      { value: 'auth', label: 'Authentication' },
      { value: 'cache', label: 'Cache Service' },
      { value: 'tron_client', label: 'Tron Client' }
    ];
  },

};

export default logService;