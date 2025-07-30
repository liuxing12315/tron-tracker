// WebSocket 实时通知服务
// 
// 提供 WebSocket 服务器功能，支持实时事件推送和客户端连接管理

use crate::core::{config::Config, models::*};
use crate::services::scanner::TransactionEvent;
use anyhow::{Result, anyhow};
use axum::{
    extract::{
        ws::{WebSocket, Message},
        WebSocketUpgrade, Query, State,
    },
    response::Response,
    http::StatusCode,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock, broadcast};
use tracing::{info, warn, error, debug};
use uuid::Uuid;

/// WebSocket 连接信息
#[derive(Debug, Clone)]
pub struct WebSocketConnection {
    pub id: String,
    pub client_ip: String,
    pub user_agent: Option<String>,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub last_ping: chrono::DateTime<chrono::Utc>,
    pub subscriptions: Vec<String>,
    pub message_count: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

/// WebSocket 订阅信息
#[derive(Debug, Clone, Deserialize)]
pub struct WebSocketSubscription {
    pub event_types: Vec<String>,
    pub addresses: Option<Vec<String>>,
    pub tokens: Option<Vec<String>>,
    pub min_amount: Option<String>,
}

/// WebSocket 消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    /// 订阅事件
    Subscribe {
        subscription: WebSocketSubscription,
    },
    /// 取消订阅
    Unsubscribe {
        subscription_id: String,
    },
    /// 心跳包
    Ping {
        timestamp: i64,
    },
    /// 心跳响应
    Pong {
        timestamp: i64,
    },
    /// 交易通知
    TransactionNotification {
        transaction: Transaction,
        event_type: String,
        subscription_id: String,
    },
    /// 系统通知
    SystemNotification {
        message: String,
        level: String,
        timestamp: i64,
    },
    /// 错误消息
    Error {
        message: String,
        code: String,
    },
    /// 连接确认
    Connected {
        connection_id: String,
        server_time: i64,
    },
}

/// WebSocket 服务状态
#[derive(Debug, Clone)]
pub struct WebSocketServiceState {
    pub total_connections: u32,
    pub active_connections: u32,
    pub total_messages_sent: u64,
    pub total_messages_received: u64,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub uptime_seconds: u64,
}

/// WebSocket 服务
pub struct WebSocketService {
    config: Config,
    connections: Arc<RwLock<HashMap<String, WebSocketConnection>>>,
    subscriptions: Arc<RwLock<HashMap<String, WebSocketSubscription>>>,
    broadcast_sender: broadcast::Sender<WebSocketMessage>,
    state: Arc<RwLock<WebSocketServiceState>>,
    start_time: std::time::Instant,
}

impl WebSocketService {
    /// 创建新的 WebSocket 服务
    pub fn new(config: Config) -> Self {
        let (broadcast_sender, _) = broadcast::channel(1000);
        
        Self {
            config,
            connections: Arc::new(RwLock::new(HashMap::new())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            broadcast_sender,
            state: Arc::new(RwLock::new(WebSocketServiceState {
                total_connections: 0,
                active_connections: 0,
                total_messages_sent: 0,
                total_messages_received: 0,
                total_bytes_sent: 0,
                total_bytes_received: 0,
                uptime_seconds: 0,
            })),
            start_time: std::time::Instant::now(),
        }
    }

    /// 处理 WebSocket 升级请求
    pub async fn handle_websocket_upgrade(
        ws: WebSocketUpgrade,
        Query(params): Query<HashMap<String, String>>,
        State(service): State<Arc<WebSocketService>>,
    ) -> Response {
        let client_ip = params.get("ip").cloned().unwrap_or_else(|| "unknown".to_string());
        let user_agent = params.get("user_agent").cloned();

        ws.on_upgrade(move |socket| {
            service.handle_websocket_connection(socket, client_ip, user_agent)
        })
    }

    /// 处理 WebSocket 连接
    async fn handle_websocket_connection(
        self: Arc<Self>,
        socket: WebSocket,
        client_ip: String,
        user_agent: Option<String>,
    ) {
        let connection_id = Uuid::new_v4().to_string();
        let connection = WebSocketConnection {
            id: connection_id.clone(),
            client_ip: client_ip.clone(),
            user_agent: user_agent.clone(),
            connected_at: chrono::Utc::now(),
            last_ping: chrono::Utc::now(),
            subscriptions: Vec::new(),
            message_count: 0,
            bytes_sent: 0,
            bytes_received: 0,
        };

        // 添加连接到管理器
        {
            let mut connections = self.connections.write().await;
            connections.insert(connection_id.clone(), connection);
            
            let mut state = self.state.write().await;
            state.total_connections += 1;
            state.active_connections += 1;
        }

        info!("WebSocket connection established: {} from {}", connection_id, client_ip);

        // 发送连接确认消息
        let connected_msg = WebSocketMessage::Connected {
            connection_id: connection_id.clone(),
            server_time: chrono::Utc::now().timestamp(),
        };

        // 分离发送和接收流
        let (mut sender, mut receiver) = socket.split();

        // 发送连接确认
        if let Ok(msg_text) = serde_json::to_string(&connected_msg) {
            if let Err(e) = sender.send(Message::Text(msg_text)).await {
                warn!("Failed to send connection confirmation: {}", e);
                return;
            }
        }

        // 创建广播接收器
        let mut broadcast_receiver = self.broadcast_sender.subscribe();

        // 创建心跳定时器
        let mut heartbeat_interval = tokio::time::interval(
            std::time::Duration::from_secs(self.config.websocket.heartbeat_interval)
        );

        // 处理消息循环
        loop {
            tokio::select! {
                // 处理客户端消息
                msg = receiver.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            if let Err(e) = self.handle_client_message(&connection_id, &text).await {
                                warn!("Error handling client message: {}", e);
                                break;
                            }
                        }
                        Some(Ok(Message::Binary(_))) => {
                            warn!("Binary messages not supported");
                        }
                        Some(Ok(Message::Close(_))) => {
                            info!("Client {} closed connection", connection_id);
                            break;
                        }
                        Some(Err(e)) => {
                            warn!("WebSocket error for {}: {}", connection_id, e);
                            break;
                        }
                        None => {
                            debug!("WebSocket stream ended for {}", connection_id);
                            break;
                        }
                    }
                }
                
                // 处理广播消息
                broadcast_msg = broadcast_receiver.recv() => {
                    match broadcast_msg {
                        Ok(msg) => {
                            if self.should_send_to_connection(&connection_id, &msg).await {
                                if let Ok(msg_text) = serde_json::to_string(&msg) {
                                    if let Err(e) = sender.send(Message::Text(msg_text)).await {
                                        warn!("Failed to send broadcast message: {}", e);
                                        break;
                                    }
                                    
                                    // 更新统计信息
                                    self.update_connection_stats(&connection_id, 0, msg_text.len() as u64).await;
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Broadcast receiver error: {}", e);
                        }
                    }
                }
                
                // 心跳检查
                _ = heartbeat_interval.tick() => {
                    let ping_msg = WebSocketMessage::Ping {
                        timestamp: chrono::Utc::now().timestamp(),
                    };
                    
                    if let Ok(msg_text) = serde_json::to_string(&ping_msg) {
                        if let Err(e) = sender.send(Message::Text(msg_text)).await {
                            warn!("Failed to send ping: {}", e);
                            break;
                        }
                    }
                }
            }
        }

        // 清理连接
        self.cleanup_connection(&connection_id).await;
    }

    /// 处理客户端消息
    async fn handle_client_message(&self, connection_id: &str, message: &str) -> Result<()> {
        debug!("Received message from {}: {}", connection_id, message);

        // 更新接收统计
        self.update_connection_stats(connection_id, message.len() as u64, 0).await;

        let ws_message: WebSocketMessage = serde_json::from_str(message)?;

        match ws_message {
            WebSocketMessage::Subscribe { subscription } => {
                self.handle_subscription(connection_id, subscription).await?;
            }
            WebSocketMessage::Unsubscribe { subscription_id } => {
                self.handle_unsubscription(connection_id, &subscription_id).await?;
            }
            WebSocketMessage::Pong { timestamp: _ } => {
                // 更新最后 ping 时间
                if let Some(mut connection) = self.connections.write().await.get_mut(connection_id) {
                    connection.last_ping = chrono::Utc::now();
                }
            }
            _ => {
                warn!("Unexpected message type from client: {}", connection_id);
            }
        }

        Ok(())
    }

    /// 处理订阅请求
    async fn handle_subscription(&self, connection_id: &str, subscription: WebSocketSubscription) -> Result<()> {
        let subscription_id = Uuid::new_v4().to_string();
        
        // 保存订阅信息
        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.insert(subscription_id.clone(), subscription);
        }

        // 更新连接的订阅列表
        {
            let mut connections = self.connections.write().await;
            if let Some(connection) = connections.get_mut(connection_id) {
                connection.subscriptions.push(subscription_id.clone());
            }
        }

        info!("Client {} subscribed with ID: {}", connection_id, subscription_id);
        Ok(())
    }

    /// 处理取消订阅请求
    async fn handle_unsubscription(&self, connection_id: &str, subscription_id: &str) -> Result<()> {
        // 移除订阅信息
        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.remove(subscription_id);
        }

        // 更新连接的订阅列表
        {
            let mut connections = self.connections.write().await;
            if let Some(connection) = connections.get_mut(connection_id) {
                connection.subscriptions.retain(|id| id != subscription_id);
            }
        }

        info!("Client {} unsubscribed from: {}", connection_id, subscription_id);
        Ok(())
    }

    /// 检查是否应该向连接发送消息
    async fn should_send_to_connection(&self, connection_id: &str, message: &WebSocketMessage) -> bool {
        match message {
            WebSocketMessage::TransactionNotification { subscription_id, .. } => {
                // 检查连接是否有对应的订阅
                let connections = self.connections.read().await;
                if let Some(connection) = connections.get(connection_id) {
                    connection.subscriptions.contains(subscription_id)
                } else {
                    false
                }
            }
            WebSocketMessage::SystemNotification { .. } => {
                // 系统通知发送给所有连接
                true
            }
            _ => false,
        }
    }

    /// 更新连接统计信息
    async fn update_connection_stats(&self, connection_id: &str, bytes_received: u64, bytes_sent: u64) {
        let mut connections = self.connections.write().await;
        if let Some(connection) = connections.get_mut(connection_id) {
            connection.message_count += 1;
            connection.bytes_received += bytes_received;
            connection.bytes_sent += bytes_sent;
        }

        // 更新全局统计
        let mut state = self.state.write().await;
        state.total_messages_sent += if bytes_sent > 0 { 1 } else { 0 };
        state.total_messages_received += if bytes_received > 0 { 1 } else { 0 };
        state.total_bytes_sent += bytes_sent;
        state.total_bytes_received += bytes_received;
    }

    /// 清理连接
    async fn cleanup_connection(&self, connection_id: &str) {
        info!("Cleaning up connection: {}", connection_id);

        // 移除连接
        {
            let mut connections = self.connections.write().await;
            connections.remove(connection_id);
        }

        // 移除相关订阅
        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.retain(|_, _| true); // 这里应该根据连接ID清理订阅
        }

        // 更新统计
        {
            let mut state = self.state.write().await;
            state.active_connections = state.active_connections.saturating_sub(1);
        }
    }

    /// 处理交易事件
    pub async fn handle_transaction_event(&self, event: TransactionEvent) -> Result<()> {
        debug!("Processing transaction event: {:?}", event);

        // 查找匹配的订阅
        let subscriptions = self.subscriptions.read().await;
        for (subscription_id, subscription) in subscriptions.iter() {
            if self.matches_subscription(&event.transaction, subscription) {
                let notification = WebSocketMessage::TransactionNotification {
                    transaction: event.transaction.clone(),
                    event_type: event.event_type.clone(),
                    subscription_id: subscription_id.clone(),
                };

                // 广播通知
                if let Err(e) = self.broadcast_sender.send(notification) {
                    warn!("Failed to broadcast transaction notification: {}", e);
                }
            }
        }

        Ok(())
    }

    /// 检查交易是否匹配订阅条件
    fn matches_subscription(&self, transaction: &Transaction, subscription: &WebSocketSubscription) -> bool {
        // 检查地址过滤
        if let Some(ref addresses) = subscription.addresses {
            let matches_address = addresses.iter().any(|addr| {
                addr == &transaction.from_address || addr == &transaction.to_address
            });
            if !matches_address {
                return false;
            }
        }

        // 检查代币过滤
        if let Some(ref tokens) = subscription.tokens {
            if !tokens.contains(&transaction.token) {
                return false;
            }
        }

        // 检查最小金额过滤
        if let Some(ref min_amount) = subscription.min_amount {
            if let (Ok(tx_amount), Ok(min_amt)) = (
                transaction.amount.parse::<f64>(),
                min_amount.parse::<f64>()
            ) {
                if tx_amount < min_amt {
                    return false;
                }
            }
        }

        true
    }

    /// 发送系统通知
    pub async fn send_system_notification(&self, message: String, level: String) -> Result<()> {
        let notification = WebSocketMessage::SystemNotification {
            message,
            level,
            timestamp: chrono::Utc::now().timestamp(),
        };

        if let Err(e) = self.broadcast_sender.send(notification) {
            warn!("Failed to send system notification: {}", e);
        }

        Ok(())
    }

    /// 获取服务状态
    pub async fn get_service_state(&self) -> WebSocketServiceState {
        let mut state = self.state.read().await.clone();
        state.uptime_seconds = self.start_time.elapsed().as_secs();
        state
    }

    /// 获取连接列表
    pub async fn get_connections(&self) -> Vec<WebSocketConnection> {
        let connections = self.connections.read().await;
        connections.values().cloned().collect()
    }

    /// 获取连接统计信息
    pub async fn get_connection_statistics(&self) -> WebSocketStatistics {
        let connections = self.connections.read().await;
        let state = self.state.read().await;

        let total_subscriptions: usize = connections.values()
            .map(|conn| conn.subscriptions.len())
            .sum();

        WebSocketStatistics {
            total_connections: state.total_connections,
            active_connections: state.active_connections,
            total_subscriptions: total_subscriptions as u32,
            total_messages_sent: state.total_messages_sent,
            total_messages_received: state.total_messages_received,
            total_bytes_sent: state.total_bytes_sent,
            total_bytes_received: state.total_bytes_received,
            uptime_seconds: self.start_time.elapsed().as_secs(),
            average_latency_ms: 0.0, // TODO: 实现延迟计算
        }
    }

    /// 断开指定连接
    pub async fn disconnect_connection(&self, connection_id: &str) -> Result<()> {
        // 这里应该发送关闭信号给指定连接
        // 由于架构限制，这里只是移除连接记录
        self.cleanup_connection(connection_id).await;
        Ok(())
    }

    /// 获取消息历史（简化实现）
    pub async fn get_message_history(&self, limit: Option<u32>) -> Vec<WebSocketMessageHistory> {
        // 这里应该从数据库或内存缓存中获取消息历史
        // 为了简化，返回模拟数据
        vec![]
    }
}

/// WebSocket 统计信息
#[derive(Debug, Clone, Serialize)]
pub struct WebSocketStatistics {
    pub total_connections: u32,
    pub active_connections: u32,
    pub total_subscriptions: u32,
    pub total_messages_sent: u64,
    pub total_messages_received: u64,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub uptime_seconds: u64,
    pub average_latency_ms: f64,
}

/// WebSocket 消息历史
#[derive(Debug, Clone, Serialize)]
pub struct WebSocketMessageHistory {
    pub id: String,
    pub connection_id: String,
    pub message_type: String,
    pub direction: String, // "inbound" or "outbound"
    pub size_bytes: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub content_preview: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Config;

    #[tokio::test]
    async fn test_websocket_service_creation() {
        let config = Config::default();
        let service = WebSocketService::new(config);
        
        let state = service.get_service_state().await;
        assert_eq!(state.active_connections, 0);
    }

    #[test]
    fn test_subscription_matching() {
        let config = Config::default();
        let service = WebSocketService::new(config);

        let transaction = Transaction {
            hash: "test_hash".to_string(),
            block_number: 12345,
            from_address: "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t".to_string(),
            to_address: "TLPpXqSYwxPwaQypYiTWkQAG2J1ePgnNK6".to_string(),
            amount: "1000.0".to_string(),
            token: "USDT".to_string(),
            status: TransactionStatus::Success,
            timestamp: chrono::Utc::now(),
            gas_used: Some(21000),
            gas_price: Some(20),
            contract_address: None,
            token_symbol: None,
            token_decimals: None,
        };

        let subscription = WebSocketSubscription {
            event_types: vec!["transaction".to_string()],
            addresses: Some(vec!["TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t".to_string()]),
            tokens: Some(vec!["USDT".to_string()]),
            min_amount: Some("500.0".to_string()),
        };

        assert!(service.matches_subscription(&transaction, &subscription));
    }
}

