import apiClient from './api.js';

/**
 * WebSocket management API services
 */
export const websocketService = {
  /**
   * Get WebSocket connections status
   */
  async getConnections() {
    const response = await apiClient.get('/admin/websocket/connections');
    return response.data || response;
  },

  /**
   * Get WebSocket service statistics
   */
  async getWebSocketStats() {
    const response = await apiClient.get('/admin/websocket/stats');
    return response.data || response;
  },

  /**
   * Disconnect specific WebSocket connection
   */
  async disconnectConnection(connectionId, reason = 'Admin action') {
    const response = await apiClient.post(`/admin/websocket/connections/${connectionId}/disconnect`, {
      reason
    });
    return response.data || response;
  },

  /**
   * Send message to specific connection
   */
  async sendMessage(connectionId, message) {
    const response = await apiClient.post(`/admin/websocket/connections/${connectionId}/message`, {
      message
    });
    return response.data || response;
  },

  /**
   * Broadcast message to all connections
   */
  async broadcastMessage(message, filter = {}) {
    const response = await apiClient.post('/admin/websocket/broadcast', {
      message,
      filter
    });
    return response.data || response;
  },

  /**
   * Get WebSocket configuration
   */
  async getWebSocketConfig() {
    const response = await apiClient.get('/admin/websocket/config');
    return response.data || response;
  },

  /**
   * Update WebSocket configuration
   */
  async updateWebSocketConfig(config) {
    const response = await apiClient.put('/admin/websocket/config', config);
    return response.data || response;
  },

  /**
   * Get connection details
   */
  async getConnectionDetails(connectionId) {
    const response = await apiClient.get(`/admin/websocket/connections/${connectionId}`);
    return response.data || response;
  },

};

export default websocketService;