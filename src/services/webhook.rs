// Webhook 事件投递服务
// 
// 负责处理 Webhook 配置管理和事件投递

use crate::core::{config::Config, database::Database, models::*};
use crate::services::scanner::TransactionEvent;
use anyhow::{Result, anyhow};
use reqwest::{Client, header::{HeaderMap, HeaderValue}};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{sleep, timeout};
use tracing::{info, warn, error, debug};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Webhook 投递任务
#[derive(Debug, Clone)]
pub struct WebhookDeliveryTask {
    pub webhook_id: String,
    pub webhook_url: String,
    pub secret: Option<String>,
    pub payload: Value,
    pub attempt: u32,
    pub max_retries: u32,
    pub timeout_seconds: u32,
}

/// Webhook 投递结果
#[derive(Debug, Clone)]
pub struct WebhookDeliveryResult {
    pub webhook_id: String,
    pub success: bool,
    pub status_code: Option<u16>,
    pub response_body: Option<String>,
    pub error_message: Option<String>,
    pub delivery_time_ms: u64,
    pub attempt: u32,
}

/// Webhook 服务状态
#[derive(Debug, Clone)]
pub struct WebhookServiceState {
    pub total_webhooks: u32,
    pub active_webhooks: u32,
    pub total_deliveries: u64,
    pub successful_deliveries: u64,
    pub failed_deliveries: u64,
    pub pending_deliveries: u32,
    pub average_delivery_time_ms: f64,
}

/// Webhook 服务
#[derive(Clone)]
pub struct WebhookService {
    config: Config,
    db: Database,
    http_client: Client,
    delivery_queue: Arc<RwLock<Vec<WebhookDeliveryTask>>>,
    state: Arc<RwLock<WebhookServiceState>>,
    delivery_sender: mpsc::UnboundedSender<WebhookDeliveryTask>,
    delivery_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<WebhookDeliveryTask>>>>,
}

impl WebhookService {
    /// 创建新的 Webhook 服务
    pub fn new(config: Config, db: Database) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        let (delivery_sender, delivery_receiver) = mpsc::unbounded_channel();

        Self {
            config,
            db,
            http_client,
            delivery_queue: Arc::new(RwLock::new(Vec::new())),
            state: Arc::new(RwLock::new(WebhookServiceState {
                total_webhooks: 0,
                active_webhooks: 0,
                total_deliveries: 0,
                successful_deliveries: 0,
                failed_deliveries: 0,
                pending_deliveries: 0,
                average_delivery_time_ms: 0.0,
            })),
            delivery_sender,
            delivery_receiver: Arc::new(RwLock::new(Some(delivery_receiver))),
        }
    }

    /// 启动 Webhook 服务
    pub async fn start(&self) -> Result<()> {
        info!("Starting Webhook service...");

        // 启动投递处理器
        let receiver = {
            let mut receiver_guard = self.delivery_receiver.write().await;
            receiver_guard.take()
        };

        if let Some(receiver) = receiver {
            let service = Arc::new(self.clone());
            tokio::spawn(async move {
                service.delivery_processor(receiver).await;
            });
        }

        // 更新 Webhook 统计
        self.update_webhook_statistics().await?;

        info!("Webhook service started");
        Ok(())
    }

    /// 处理交易事件
    pub async fn handle_transaction_event(&self, event: TransactionEvent) -> Result<()> {
        debug!("Processing transaction event for webhooks: {:?}", event);

        // 获取所有启用的 Webhook
        let webhooks = self.db.get_enabled_webhooks().await?;

        for webhook in webhooks {
            if self.should_trigger_webhook(&webhook, &event) {
                let payload = self.create_webhook_payload(&webhook, &event)?;
                
                let delivery_task = WebhookDeliveryTask {
                    webhook_id: webhook.id.clone(),
                    webhook_url: webhook.url.clone(),
                    secret: webhook.secret.clone(),
                    payload,
                    attempt: 1,
                    max_retries: webhook.retry_count,
                    timeout_seconds: webhook.timeout,
                };

                // 发送到投递队列
                if let Err(e) = self.delivery_sender.send(delivery_task) {
                    error!("Failed to queue webhook delivery: {}", e);
                }
            }
        }

        Ok(())
    }

    /// 检查是否应该触发 Webhook
    fn should_trigger_webhook(&self, webhook: &Webhook, event: &TransactionEvent) -> bool {
        // 检查事件类型
        if !webhook.events.contains(&event.event_type) {
            return false;
        }

        // 检查过滤条件
        if let Some(ref addresses) = webhook.filters.addresses {
            let matches_address = addresses.iter().any(|addr| {
                addr == &event.transaction.from_address || addr == &event.transaction.to_address
            });
            if !matches_address {
                return false;
            }
        }

        if let Some(ref tokens) = webhook.filters.tokens {
            let default_token = "TRX".to_string();
            let token_symbol = event.transaction.token_symbol.as_ref().unwrap_or(&default_token);
            if !tokens.contains(token_symbol) {
                return false;
            }
        }

        if let Some(ref min_amount) = webhook.filters.min_amount {
            if let (Ok(tx_amount), Ok(min_amt)) = (
                event.transaction.value.parse::<f64>(),
                min_amount.parse::<f64>()
            ) {
                if tx_amount < min_amt {
                    return false;
                }
            }
        }

        true
    }

    /// 创建 Webhook 载荷
    fn create_webhook_payload(&self, webhook: &Webhook, event: &TransactionEvent) -> Result<Value> {
        let payload = json!({
            "webhook_id": webhook.id,
            "event_type": event.event_type,
            "timestamp": chrono::Utc::now().timestamp(),
            "data": {
                "transaction": {
                    "hash": event.transaction.hash,
                    "block_number": event.transaction.block_number,
                    "from_address": event.transaction.from_address,
                    "to_address": event.transaction.to_address,
                    "value": event.transaction.value,
                    "token_symbol": event.transaction.token_symbol,
                    "status": match event.transaction.status {
                        TransactionStatus::Success => "success",
                        TransactionStatus::Failed => "failed",
                        TransactionStatus::Pending => "pending",
                    },
                    "timestamp": event.transaction.timestamp.timestamp(),
                    "gas_used": event.transaction.gas_used,
                    "gas_price": event.transaction.gas_price,
                    "token_address": event.transaction.token_address,
                    "token_decimals": event.transaction.token_decimals,
                }
            }
        });

        Ok(payload)
    }

    /// 投递处理器
    async fn delivery_processor(self: Arc<Self>, mut receiver: mpsc::UnboundedReceiver<WebhookDeliveryTask>) {
        info!("Webhook delivery processor started");

        while let Some(task) = receiver.recv().await {
            let service = Arc::clone(&self);
            
            // 异步处理每个投递任务
            tokio::spawn(async move {
                let result = service.deliver_webhook(task.clone()).await;
                service.handle_delivery_result(task, result).await;
            });
        }

        info!("Webhook delivery processor stopped");
    }

    /// 投递 Webhook
    async fn deliver_webhook(&self, task: WebhookDeliveryTask) -> WebhookDeliveryResult {
        let start_time = std::time::Instant::now();
        
        debug!("Delivering webhook {} to {}", task.webhook_id, task.webhook_url);

        // 准备请求头
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        headers.insert("User-Agent", HeaderValue::from_static("TronTracker-Webhook/2.0"));

        // 生成签名
        if let Some(ref secret) = task.secret {
            if let Ok(signature) = self.generate_signature(&task.payload, secret) {
                if let Ok(sig_header) = HeaderValue::from_str(&format!("sha256={}", signature)) {
                    headers.insert("X-Webhook-Signature", sig_header);
                }
            }
        }

        // 添加自定义头
        if let Ok(timestamp_header) = HeaderValue::from_str(&chrono::Utc::now().timestamp().to_string()) {
            headers.insert("X-Webhook-Timestamp", timestamp_header);
        }

        if let Ok(attempt_header) = HeaderValue::from_str(&task.attempt.to_string()) {
            headers.insert("X-Webhook-Attempt", attempt_header);
        }

        // 发送请求
        let request_future = self.http_client
            .post(&task.webhook_url)
            .headers(headers)
            .json(&task.payload)
            .send();

        let delivery_time_ms = start_time.elapsed().as_millis() as u64;

        match timeout(Duration::from_secs(task.timeout_seconds as u64), request_future).await {
            Ok(Ok(response)) => {
                let status_code = response.status().as_u16();
                let success = response.status().is_success();

                match response.text().await {
                    Ok(body) => {
                        debug!("Webhook delivery completed: {} - {}", status_code, success);
                        WebhookDeliveryResult {
                            webhook_id: task.webhook_id,
                            success,
                            status_code: Some(status_code),
                            response_body: Some(body),
                            error_message: None,
                            delivery_time_ms,
                            attempt: task.attempt,
                        }
                    }
                    Err(e) => {
                        warn!("Failed to read response body: {}", e);
                        WebhookDeliveryResult {
                            webhook_id: task.webhook_id,
                            success: false,
                            status_code: Some(status_code),
                            response_body: None,
                            error_message: Some(e.to_string()),
                            delivery_time_ms,
                            attempt: task.attempt,
                        }
                    }
                }
            }
            Ok(Err(e)) => {
                warn!("Webhook delivery failed: {}", e);
                WebhookDeliveryResult {
                    webhook_id: task.webhook_id,
                    success: false,
                    status_code: None,
                    response_body: None,
                    error_message: Some(e.to_string()),
                    delivery_time_ms,
                    attempt: task.attempt,
                }
            }
            Err(_) => {
                warn!("Webhook delivery timeout after {}s", task.timeout_seconds);
                WebhookDeliveryResult {
                    webhook_id: task.webhook_id,
                    success: false,
                    status_code: None,
                    response_body: None,
                    error_message: Some("Request timeout".to_string()),
                    delivery_time_ms,
                    attempt: task.attempt,
                }
            }
        }
    }

    /// 处理投递结果
    async fn handle_delivery_result(&self, task: WebhookDeliveryTask, result: WebhookDeliveryResult) {
        // 更新数据库统计
        if let Err(e) = self.db.update_webhook_stats(&task.webhook_id, result.success).await {
            warn!("Failed to update webhook stats: {}", e);
        }

        // 更新服务统计
        {
            let mut state = self.state.write().await;
            state.total_deliveries += 1;
            if result.success {
                state.successful_deliveries += 1;
            } else {
                state.failed_deliveries += 1;
            }

            // 更新平均投递时间
            let total_time = state.average_delivery_time_ms * (state.total_deliveries - 1) as f64;
            state.average_delivery_time_ms = (total_time + result.delivery_time_ms as f64) / state.total_deliveries as f64;
        }

        // 如果失败且还有重试次数，安排重试
        if !result.success && task.attempt < task.max_retries {
            let retry_delay = self.calculate_retry_delay(task.attempt);
            
            info!("Scheduling webhook retry {} for {} in {}s", 
                  task.attempt + 1, task.webhook_id, retry_delay);

            let retry_task = WebhookDeliveryTask {
                attempt: task.attempt + 1,
                ..task
            };

            let sender = self.delivery_sender.clone();
            tokio::spawn(async move {
                sleep(Duration::from_secs(retry_delay)).await;
                if let Err(e) = sender.send(retry_task) {
                    error!("Failed to schedule webhook retry: {}", e);
                }
            });
        } else if !result.success {
            error!("Webhook delivery failed permanently after {} attempts: {}", 
                   task.attempt, task.webhook_id);
        }

        info!("Webhook delivery result: {} - {} (attempt {})", 
              task.webhook_id, if result.success { "success" } else { "failed" }, task.attempt);
    }

    /// 计算重试延迟（指数退避）
    fn calculate_retry_delay(&self, attempt: u32) -> u64 {
        let base_delay = 2u64;
        let max_delay = 300; // 5分钟
        
        let delay = base_delay.pow(attempt).min(max_delay);
        delay
    }

    /// 生成 Webhook 签名
    fn generate_signature(&self, payload: &Value, secret: &str) -> Result<String> {
        let payload_str = serde_json::to_string(payload)?;
        
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .map_err(|e| anyhow!("Invalid secret key: {}", e))?;
        
        mac.update(payload_str.as_bytes());
        let result = mac.finalize();
        
        Ok(hex::encode(result.into_bytes()))
    }

    /// 验证 Webhook 签名
    pub fn verify_signature(&self, payload: &str, signature: &str, secret: &str) -> Result<bool> {
        let expected_signature = {
            let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
                .map_err(|e| anyhow!("Invalid secret key: {}", e))?;
            
            mac.update(payload.as_bytes());
            let result = mac.finalize();
            hex::encode(result.into_bytes())
        };

        // 移除 "sha256=" 前缀（如果存在）
        let signature = signature.strip_prefix("sha256=").unwrap_or(signature);
        
        Ok(signature == expected_signature)
    }

    /// 手动触发 Webhook
    pub async fn trigger_webhook_manually(&self, webhook_id: &str, payload: Value) -> Result<()> {
        // 获取 Webhook 配置
        let webhooks = self.db.list_webhooks(false).await?;
        let webhook = webhooks.iter()
            .find(|w| w.id == webhook_id)
            .ok_or_else(|| anyhow!("Webhook not found: {}", webhook_id))?;

        let delivery_task = WebhookDeliveryTask {
            webhook_id: webhook.id.clone(),
            webhook_url: webhook.url.clone(),
            secret: webhook.secret.clone(),
            payload,
            attempt: 1,
            max_retries: webhook.retry_count,
            timeout_seconds: webhook.timeout,
        };

        // 发送到投递队列
        self.delivery_sender.send(delivery_task)?;

        info!("Manual webhook trigger queued: {}", webhook_id);
        Ok(())
    }

    /// 测试 Webhook 连接
    pub async fn test_webhook(&self, webhook_url: &str, secret: Option<String>) -> Result<WebhookDeliveryResult> {
        let test_payload = json!({
            "event_type": "test",
            "timestamp": chrono::Utc::now().timestamp(),
            "data": {
                "message": "This is a test webhook from TronTracker"
            }
        });

        let test_task = WebhookDeliveryTask {
            webhook_id: "test".to_string(),
            webhook_url: webhook_url.to_string(),
            secret,
            payload: test_payload,
            attempt: 1,
            max_retries: 0,
            timeout_seconds: 30,
        };

        Ok(self.deliver_webhook(test_task).await)
    }

    /// 获取服务状态
    pub async fn get_service_state(&self) -> WebhookServiceState {
        self.state.read().await.clone()
    }

    /// 更新 Webhook 统计信息
    async fn update_webhook_statistics(&self) -> Result<()> {
        let webhooks = self.db.list_webhooks(false).await?;
        
        let mut state = self.state.write().await;
        state.total_webhooks = webhooks.len() as u32;
        state.active_webhooks = webhooks.iter().filter(|w| w.enabled).count() as u32;

        Ok(())
    }

    /// 获取投递队列状态
    pub async fn get_queue_status(&self) -> WebhookQueueStatus {
        let queue = self.delivery_queue.read().await;
        let state = self.state.read().await;

        WebhookQueueStatus {
            pending_deliveries: queue.len() as u32,
            total_deliveries: state.total_deliveries,
            successful_deliveries: state.successful_deliveries,
            failed_deliveries: state.failed_deliveries,
            success_rate: if state.total_deliveries > 0 {
                (state.successful_deliveries as f64 / state.total_deliveries as f64) * 100.0
            } else {
                0.0
            },
            average_delivery_time_ms: state.average_delivery_time_ms,
        }
    }

    /// 清空投递队列
    pub async fn clear_queue(&self) -> Result<u32> {
        let mut queue = self.delivery_queue.write().await;
        let count = queue.len() as u32;
        queue.clear();
        
        info!("Cleared {} pending webhook deliveries", count);
        Ok(count)
    }

    /// 重新投递失败的 Webhook
    pub async fn retry_failed_webhooks(&self, webhook_id: Option<String>) -> Result<u32> {
        // 这里应该从数据库查询失败的投递记录并重新投递
        // 为了简化，返回模拟数据
        info!("Retrying failed webhooks for: {:?}", webhook_id);
        Ok(0)
    }

    /// 启动投递工作器
    pub async fn start_delivery_worker(&self) -> Result<()> {
        info!("Starting webhook delivery worker...");
        self.start().await
    }

    /// 停止 Webhook 服务
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping webhook service...");
        
        // 清空投递队列
        let cleared_count = self.clear_queue().await?;
        info!("Cleared {} pending deliveries", cleared_count);
        
        info!("Webhook service stopped");
        Ok(())
    }
}

// 实现 Clone trait 以支持 Arc
impl Clone for WebhookService {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            db: self.db.clone(),
            http_client: self.http_client.clone(),
            delivery_queue: Arc::clone(&self.delivery_queue),
            state: Arc::clone(&self.state),
            delivery_sender: self.delivery_sender.clone(),
            delivery_receiver: Arc::clone(&self.delivery_receiver),
        }
    }
}

/// Webhook 队列状态
#[derive(Debug, Clone, serde::Serialize)]
pub struct WebhookQueueStatus {
    pub pending_deliveries: u32,
    pub total_deliveries: u64,
    pub successful_deliveries: u64,
    pub failed_deliveries: u64,
    pub success_rate: f64,
    pub average_delivery_time_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Config;

    #[tokio::test]
    async fn test_webhook_service_creation() {
        let config = Config::default();
        let db = Database::new(&config.database).await.unwrap();
        
        let service = WebhookService::new(config, db);
        let state = service.get_service_state().await;
        
        assert_eq!(state.total_deliveries, 0);
    }

    #[test]
    fn test_signature_generation() {
        let config = Config::default();
        let db = Database::new(&config.database).await.unwrap();
        let service = WebhookService::new(config, db);

        let payload = json!({"test": "data"});
        let secret = "test_secret";

        let signature = service.generate_signature(&payload, secret).unwrap();
        assert!(!signature.is_empty());

        // 验证签名
        let payload_str = serde_json::to_string(&payload).unwrap();
        let is_valid = service.verify_signature(&payload_str, &signature, secret).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_retry_delay_calculation() {
        let config = Config::default();
        let db = Database::new(&config.database).await.unwrap();
        let service = WebhookService::new(config, db);

        assert_eq!(service.calculate_retry_delay(1), 2);
        assert_eq!(service.calculate_retry_delay(2), 4);
        assert_eq!(service.calculate_retry_delay(3), 8);
        assert_eq!(service.calculate_retry_delay(10), 300); // 最大延迟
    }
}

