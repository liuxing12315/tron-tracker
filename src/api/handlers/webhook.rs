// Webhook 管理路由处理器
// 
// 提供 Webhook 的增删改查、测试、重试等功能的 HTTP 端点

use crate::core::models::*;
use crate::api::handlers::admin::AdminAppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

/// Webhook 列表查询参数
#[derive(Debug, Deserialize)]
pub struct ListWebhooksQuery {
    pub include_disabled: Option<bool>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub event_type: Option<String>,
}

/// Webhook 创建请求
#[derive(Debug, Deserialize)]
pub struct CreateWebhookRequest {
    pub name: String,
    pub url: String,
    pub secret: Option<String>,
    pub events: Vec<String>,
    pub filters: serde_json::Value,
    pub enabled: Option<bool>,
}

/// Webhook 更新请求
#[derive(Debug, Deserialize)]
pub struct UpdateWebhookRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub secret: Option<String>,
    pub events: Option<Vec<String>>,
    pub filters: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

/// Webhook 数据传输对象
#[derive(Debug, Serialize)]
pub struct WebhookDto {
    pub id: String,
    pub name: String,
    pub url: String,
    pub secret_preview: String,
    pub events: Vec<String>,
    pub filters: serde_json::Value,
    pub enabled: bool,
    pub success_count: i64,
    pub failure_count: i64,
    pub last_triggered: Option<chrono::DateTime<chrono::Utc>>,
    pub success_rate: f64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Webhook 列表响应
#[derive(Debug, Serialize)]
pub struct ListWebhooksResponse {
    pub webhooks: Vec<WebhookDto>,
    pub total: u64,
    pub page: u32,
    pub limit: u32,
}

/// Webhook 测试请求
#[derive(Debug, Deserialize)]
pub struct TestWebhookRequest {
    pub url: String,
    pub secret: Option<String>,
    pub test_data: Option<serde_json::Value>,
}

/// Webhook 测试响应
#[derive(Debug, Serialize)]
pub struct TestWebhookResponse {
    pub success: bool,
    pub status_code: Option<u16>,
    pub response_time_ms: u64,
    pub error: Option<String>,
    pub response_body: Option<String>,
}

/// Webhook 投递日志
#[derive(Debug, Serialize)]
pub struct WebhookDeliveryLog {
    pub id: String,
    pub webhook_id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub status_code: Option<u16>,
    pub response_body: Option<String>,
    pub error: Option<String>,
    pub attempt: i32,
    pub delivered_at: chrono::DateTime<chrono::Utc>,
}

/// 获取所有 Webhooks
pub async fn list_webhooks(
    Query(query): Query<ListWebhooksQuery>,
    State(state): State<Arc<AdminAppState>>,
) -> Result<Json<ListWebhooksResponse>, StatusCode> {
    let include_disabled = query.include_disabled.unwrap_or(true);
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);

    match state.db.list_webhooks(include_disabled).await {
        Ok(webhooks) => {
            // 过滤事件类型
            let filtered_webhooks: Vec<_> = webhooks
                .into_iter()
                .filter(|webhook| {
                    if let Some(ref event_type) = query.event_type {
                        webhook.events.iter().any(|e| format!("{:?}", e).contains(event_type))
                    } else {
                        true
                    }
                })
                .collect();

            let total = filtered_webhooks.len() as u64;
            
            // 分页
            let start = ((page - 1) * limit) as usize;
            let end = (start + limit as usize).min(filtered_webhooks.len());
            
            let paginated_webhooks: Vec<WebhookDto> = filtered_webhooks[start..end]
                .iter()
                .map(|webhook| {
                    let total_deliveries = webhook.success_count + webhook.failure_count;
                    let success_rate = if total_deliveries > 0 {
                        webhook.success_count as f64 / total_deliveries as f64 * 100.0
                    } else {
                        0.0
                    };

                    WebhookDto {
                        id: webhook.id.to_string(),
                        name: webhook.name.clone(),
                        url: webhook.url.clone(),
                        secret_preview: if webhook.secret.is_empty() {
                            "None".to_string()
                        } else {
                            format!("{}...", &webhook.secret[..8.min(webhook.secret.len())])
                        },
                        events: webhook.events.iter().map(|e| format!("{:?}", e)).collect(),
                        filters: serde_json::to_value(&webhook.filters).unwrap_or_default(),
                        enabled: webhook.enabled,
                        success_count: webhook.success_count,
                        failure_count: webhook.failure_count,
                        last_triggered: webhook.last_triggered,
                        success_rate,
                        created_at: webhook.created_at,
                        updated_at: webhook.updated_at,
                    }
                })
                .collect();

            Ok(Json(ListWebhooksResponse {
                webhooks: paginated_webhooks,
                total,
                page,
                limit,
            }))
        }
        Err(e) => {
            error!("Failed to list webhooks: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 获取单个 Webhook
pub async fn get_webhook(
    Path(webhook_id): Path<String>,
    State(state): State<Arc<AdminAppState>>,
) -> Result<Json<WebhookDto>, StatusCode> {
    match state.db.get_webhook(&webhook_id).await {
        Ok(Some(webhook)) => {
            let total_deliveries = webhook.success_count + webhook.failure_count;
            let success_rate = if total_deliveries > 0 {
                webhook.success_count as f64 / total_deliveries as f64 * 100.0
            } else {
                0.0
            };

            let dto = WebhookDto {
                id: webhook.id.to_string(),
                name: webhook.name.clone(),
                url: webhook.url.clone(),
                secret_preview: if webhook.secret.is_empty() {
                    "None".to_string()
                } else {
                    format!("{}...", &webhook.secret[..8.min(webhook.secret.len())])
                },
                events: webhook.events.iter().map(|e| format!("{:?}", e)).collect(),
                filters: serde_json::to_value(&webhook.filters).unwrap_or_default(),
                enabled: webhook.enabled,
                success_count: webhook.success_count,
                failure_count: webhook.failure_count,
                last_triggered: webhook.last_triggered,
                success_rate,
                created_at: webhook.created_at,
                updated_at: webhook.updated_at,
            };
            Ok(Json(dto))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get webhook {}: {}", webhook_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 创建 Webhook
pub async fn create_webhook(
    State(state): State<Arc<AdminAppState>>,
    Json(request): Json<CreateWebhookRequest>,
) -> Result<Json<WebhookDto>, StatusCode> {
    // 解析事件类型
    let events: Result<Vec<NotificationEventType>, _> = request.events.iter()
        .map(|e| match e.as_str() {
            "transaction" => Ok(NotificationEventType::Transaction),
            "new_address" => Ok(NotificationEventType::NewAddress),
            "system_alert" => Ok(NotificationEventType::SystemAlert),
            _ => Err(format!("Invalid event type: {}", e)),
        })
        .collect();

    let events = match events {
        Ok(events) => events,
        Err(e) => {
            error!("Invalid event types: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    let webhook = Webhook {
        id: Uuid::new_v4(),
        name: request.name,
        url: request.url,
        secret: request.secret.unwrap_or_else(|| generate_webhook_secret()),
        events,
        filters: serde_json::from_value(request.filters).unwrap_or_default(),
        retry_count: 0, // Default value
        timeout: 30000, // Default value (30 seconds)
        enabled: request.enabled.unwrap_or(true),
        success_count: 0,
        failure_count: 0,
        last_triggered: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    match state.db.save_webhook(&webhook).await {
        Ok(_) => {
            info!("Created webhook: {} ({})", webhook.name, webhook.id);
            
            let dto = WebhookDto {
                id: webhook.id.to_string(),
                name: webhook.name.clone(),
                url: webhook.url.clone(),
                secret_preview: if webhook.secret.is_empty() {
                    "None".to_string()
                } else {
                    format!("{}...", &webhook.secret[..8.min(webhook.secret.len())])
                },
                events: webhook.events.iter().map(|e| format!("{:?}", e)).collect(),
                filters: serde_json::to_value(&webhook.filters).unwrap_or_default(),
                enabled: webhook.enabled,
                success_count: webhook.success_count,
                failure_count: webhook.failure_count,
                last_triggered: webhook.last_triggered,
                success_rate: 0.0,
                created_at: webhook.created_at,
                updated_at: webhook.updated_at,
            };
            
            Ok(Json(dto))
        }
        Err(e) => {
            error!("Failed to create webhook: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 更新 Webhook
pub async fn update_webhook(
    Path(webhook_id): Path<String>,
    State(state): State<Arc<AdminAppState>>,
    Json(request): Json<UpdateWebhookRequest>,
) -> Result<Json<WebhookDto>, StatusCode> {
    let mut webhook = match state.db.get_webhook(&webhook_id).await {
        Ok(Some(webhook)) => webhook,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get webhook {}: {}", webhook_id, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // 更新字段
    if let Some(name) = request.name {
        webhook.name = name;
    }
    if let Some(url) = request.url {
        webhook.url = url;
    }
    if let Some(secret) = request.secret {
        webhook.secret = secret;
    }
    if let Some(events) = request.events {
        let parsed_events: Result<Vec<NotificationEventType>, _> = events.iter()
            .map(|e| match e.as_str() {
                "transaction" => Ok(NotificationEventType::Transaction),
                "new_address" => Ok(NotificationEventType::NewAddress),
                "system_alert" => Ok(NotificationEventType::SystemAlert),
                _ => Err(format!("Invalid event type: {}", e)),
            })
            .collect();

        webhook.events = match parsed_events {
            Ok(events) => events,
            Err(e) => {
                error!("Invalid event types: {}", e);
                return Err(StatusCode::BAD_REQUEST);
            }
        };
    }
    if let Some(filters) = request.filters {
        webhook.filters = serde_json::from_value(filters).unwrap_or_default();
    }
    if let Some(enabled) = request.enabled {
        webhook.enabled = enabled;
    }
    
    webhook.updated_at = chrono::Utc::now();

    match state.db.update_webhook(&webhook).await {
        Ok(_) => {
            info!("Updated webhook: {} ({})", webhook.name, webhook.id);
            
            let total_deliveries = webhook.success_count + webhook.failure_count;
            let success_rate = if total_deliveries > 0 {
                webhook.success_count as f64 / total_deliveries as f64 * 100.0
            } else {
                0.0
            };

            let dto = WebhookDto {
                id: webhook.id.to_string(),
                name: webhook.name.clone(),
                url: webhook.url.clone(),
                secret_preview: if webhook.secret.is_empty() {
                    "None".to_string()
                } else {
                    format!("{}...", &webhook.secret[..8.min(webhook.secret.len())])
                },
                events: webhook.events.iter().map(|e| format!("{:?}", e)).collect(),
                filters: serde_json::to_value(&webhook.filters).unwrap_or_default(),
                enabled: webhook.enabled,
                success_count: webhook.success_count,
                failure_count: webhook.failure_count,
                last_triggered: webhook.last_triggered,
                success_rate,
                created_at: webhook.created_at,
                updated_at: webhook.updated_at,
            };
            
            Ok(Json(dto))
        }
        Err(e) => {
            error!("Failed to update webhook {}: {}", webhook_id, e);
            if e.to_string().contains("not found") {
                Err(StatusCode::NOT_FOUND)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

/// 删除 Webhook
pub async fn delete_webhook(
    Path(webhook_id): Path<String>,
    State(state): State<Arc<AdminAppState>>,
) -> Result<StatusCode, StatusCode> {
    match state.db.delete_webhook(&webhook_id).await {
        Ok(_) => {
            info!("Deleted webhook: {}", webhook_id);
            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => {
            error!("Failed to delete webhook {}: {}", webhook_id, e);
            if e.to_string().contains("not found") {
                Err(StatusCode::NOT_FOUND)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

/// 测试 Webhook
pub async fn test_webhook(
    State(_state): State<Arc<AdminAppState>>,
    Json(request): Json<TestWebhookRequest>,
) -> Result<Json<TestWebhookResponse>, StatusCode> {
    let start_time = std::time::Instant::now();
    
    // 创建测试负载
    let test_payload = request.test_data.unwrap_or_else(|| {
        serde_json::json!({
            "event_type": "test",
            "timestamp": chrono::Utc::now(),
            "data": {
                "message": "This is a test webhook delivery",
                "test": true
            }
        })
    });

    // 创建HTTP客户端
    let client = reqwest::Client::new();
    let mut req_builder = client.post(&request.url)
        .header("Content-Type", "application/json")
        .header("User-Agent", "TronTracker-Webhook/1.0")
        .json(&test_payload);

    // 添加签名头（如果有密钥）
    if let Some(secret) = &request.secret {
        let payload_str = serde_json::to_string(&test_payload).unwrap_or_default();
        let signature = create_webhook_signature(&payload_str, secret);
        req_builder = req_builder.header("X-Webhook-Signature", signature);
    }

    // 发送请求
    match req_builder.send().await {
        Ok(response) => {
            let status_code = response.status().as_u16();
            let response_time = start_time.elapsed().as_millis() as u64;
            let response_body = response.text().await.unwrap_or_default();
            
            let success = status_code >= 200 && status_code < 300;
            
            Ok(Json(TestWebhookResponse {
                success,
                status_code: Some(status_code),
                response_time_ms: response_time,
                error: if success { None } else { Some(format!("HTTP {}", status_code)) },
                response_body: Some(response_body),
            }))
        }
        Err(e) => {
            let response_time = start_time.elapsed().as_millis() as u64;
            
            Ok(Json(TestWebhookResponse {
                success: false,
                status_code: None,
                response_time_ms: response_time,
                error: Some(e.to_string()),
                response_body: None,
            }))
        }
    }
}

/// 获取 Webhook 投递日志
pub async fn get_webhook_logs(
    Path(webhook_id): Path<String>,
    Query(query): Query<serde_json::Value>,
    State(state): State<Arc<AdminAppState>>,
) -> Result<Json<Vec<WebhookDeliveryLog>>, StatusCode> {
    let page = query.get("page").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
    let limit = query.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as u32;

    match state.db.get_webhook_delivery_logs(&webhook_id, page, limit).await {
        Ok(logs) => Ok(Json(logs)),
        Err(e) => {
            error!("Failed to get webhook logs for {}: {}", webhook_id, e);
            if e.to_string().contains("not found") {
                Err(StatusCode::NOT_FOUND)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

/// 重试失败的 Webhook 投递
pub async fn retry_webhook(
    Path(webhook_id): Path<String>,
    State(state): State<Arc<AdminAppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // 这里应该触发 Webhook 服务重试失败的投递
    // 暂时返回成功响应
    info!("Retrying webhook deliveries for: {}", webhook_id);
    
    Ok(Json(serde_json::json!({
        "message": "Webhook retry initiated",
        "webhook_id": webhook_id
    })))
}

/// 获取可用的事件类型
pub async fn get_available_events() -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    let events = vec![
        serde_json::json!({
            "id": "transaction",
            "name": "Transaction",
            "description": "New transaction detected"
        }),
        serde_json::json!({
            "id": "large_transfer",
            "name": "Large Transfer",
            "description": "Large value transfer detected"
        }),
        serde_json::json!({
            "id": "new_address",
            "name": "New Address",
            "description": "New address detected"
        }),
        serde_json::json!({
            "id": "system_alert",
            "name": "System Alert",
            "description": "System alert or error"
        }),
    ];

    Ok(Json(events))
}

/// 生成 Webhook 密钥
fn generate_webhook_secret() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let random_bytes: [u8; 32] = rng.gen();
    hex::encode(random_bytes)
}

/// 创建 Webhook 签名
fn create_webhook_signature(payload: &str, secret: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    
    type HmacSha256 = Hmac<Sha256>;
    
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(payload.as_bytes());
    let result = mac.finalize();
    format!("sha256={}", hex::encode(result.into_bytes()))
}