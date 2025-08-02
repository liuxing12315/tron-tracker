import apiClient from './api.js';

/**
 * System statistics and health services
 */
export const systemService = {
  /**
   * Get system statistics for dashboard
   */
  async getSystemStats() {
    const response = await apiClient.get('/system/stats');
    return response.data || response;
  },

  /**
   * Get system health status
   */
  async getHealthStatus() {
    const response = await apiClient.get('/health');
    return response.data || response;
  },

  /**
   * Get detailed system metrics
   */
  async getSystemMetrics() {
    const response = await apiClient.get('/admin/metrics');
    return response.data || response;
  },

  /**
   * Get scanner status and statistics
   */
  async getScannerStats() {
    const response = await apiClient.get('/admin/scanner/stats');
    return response.data || response;
  }
};

export default systemService;