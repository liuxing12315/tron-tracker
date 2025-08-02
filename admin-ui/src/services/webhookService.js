import apiClient from './api.js';

/**
 * Webhook management API services
 */
export const webhookService = {
  /**
   * Get all webhooks
   */
  async getWebhooks(includeDisabled = true) {
    const params = { include_disabled: includeDisabled };
    const response = await apiClient.get('/webhooks', params);
    return response.data || response;
  },

  /**
   * Get single webhook by ID
   */
  async getWebhook(webhookId) {
    const response = await apiClient.get(`/webhooks/${webhookId}`);
    return response.data || response;
  },

  /**
   * Create new webhook
   */
  async createWebhook(webhookData) {
    const response = await apiClient.post('/webhooks', webhookData);
    return response.data || response;
  },

  /**
   * Update existing webhook
   */
  async updateWebhook(webhookId, webhookData) {
    const response = await apiClient.put(`/webhooks/${webhookId}`, webhookData);
    return response.data || response;
  },

  /**
   * Delete webhook
   */
  async deleteWebhook(webhookId) {
    await apiClient.delete(`/webhooks/${webhookId}`);
    return true;
  },

  /**
   * Test webhook endpoint
   */
  async testWebhook(webhookUrl, secret = null) {
    const data = { url: webhookUrl };
    if (secret) {
      data.secret = secret;
    }
    
    const response = await apiClient.post('/webhooks/test', data);
    return response.data || response;
  },

  /**
   * Get webhook delivery logs
   */
  async getWebhookLogs(webhookId, pagination = {}) {
    const params = {
      page: pagination.page || 1,
      limit: pagination.limit || 50,
    };
    
    const response = await apiClient.get(`/webhooks/${webhookId}/logs`, params);
    return response.data || response;
  },

  /**
   * Retry failed webhook deliveries
   */
  async retryWebhook(webhookId, deliveryId = null) {
    const endpoint = deliveryId 
      ? `/webhooks/${webhookId}/retry/${deliveryId}`
      : `/webhooks/${webhookId}/retry`;
      
    const response = await apiClient.post(endpoint);
    return response.data || response;
  },

  /**
   * Get webhook statistics
   */
  async getWebhookStats(webhookId) {
    const response = await apiClient.get(`/webhooks/${webhookId}/stats`);
    return response.data || response;
  }
};

export default webhookService;