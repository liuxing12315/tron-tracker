import apiClient from './api.js';

/**
 * API Key management services
 */
export const apiKeyService = {
  /**
   * Get all API keys
   */
  async getApiKeys() {
    const response = await apiClient.get('/v1/api-keys');
    return response.data || response;
  },

  /**
   * Get single API key by ID
   */
  async getApiKey(keyId) {
    const response = await apiClient.get(`/v1/api-keys/${keyId}`);
    return response.data || response;
  },

  /**
   * Create new API key
   */
  async createApiKey(keyData) {
    const response = await apiClient.post('/v1/api-keys', keyData);
    return response.data || response;
  },

  /**
   * Update existing API key
   */
  async updateApiKey(keyId, keyData) {
    const response = await apiClient.put(`/v1/api-keys/${keyId}`, keyData);
    return response.data || response;
  },

  /**
   * Delete API key
   */
  async deleteApiKey(keyId) {
    await apiClient.delete(`/v1/api-keys/${keyId}`);
    return true;
  },

  /**
   * Regenerate API key
   */
  async regenerateApiKey(keyId) {
    const response = await apiClient.post(`/v1/api-keys/${keyId}/regenerate`);
    return response.data || response;
  },

  /**
   * Get API key usage statistics
   */
  async getApiKeyUsage(keyId, timeRange = '7d') {
    const params = { range: timeRange };
    const response = await apiClient.get(`/v1/api-keys/${keyId}/usage`, params);
    return response.data || response;
  },

  /**
   * Enable/disable API key
   */
  async toggleApiKey(keyId, enabled) {
    const response = await apiClient.put(`/v1/api-keys/${keyId}`, { enabled });
    return response.data || response;
  },

  /**
   * Get API key permissions list
   */
  getAvailablePermissions() {
    return [
      {
        id: 'read_transactions',
        name: 'Read Transactions',
        description: 'View transaction data and search transactions'
      },
      {
        id: 'read_addresses',
        name: 'Read Addresses',
        description: 'View address information and statistics'
      },
      {
        id: 'read_blocks',
        name: 'Read Blocks',
        description: 'View block information and data'
      },
      {
        id: 'manage_webhooks',
        name: 'Manage Webhooks',
        description: 'Create, update, and delete webhooks'
      },
      {
        id: 'manage_api_keys',
        name: 'Manage API Keys',
        description: 'Create and manage other API keys'
      },
      {
        id: 'manage_system',
        name: 'Manage System',
        description: 'Access system configuration and management'
      },
      {
        id: 'admin',
        name: 'Administrator',
        description: 'Full access to all system features'
      }
    ];
  },

  /**
   * Test API key
   */
  async testApiKey(apiKey) {
    const response = await apiClient.post('/v1/api-keys/test', { api_key: apiKey });
    return response.data || response;
  }
};

export default apiKeyService;