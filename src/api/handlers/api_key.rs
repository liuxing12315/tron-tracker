// API Key 管理路由处理器
// 
// 提供 API Key 的增删改查、权限管理、使用统计等功能的 HTTP 端点

use crate::services::auth::{AuthService, CreateApiKeyRequest, UpdateApiKeyRequest, ApiPermission};
use crate::api::handlers::admin::AdminAppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

/// API Key 管理应用状态 - 使用统一的AdminAppState

/// API Key 列表查询参数
#[derive(Debug, Deserialize)]
pub struct ListApiKeysQuery {
    pub include_disabled: Option<bool>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

/// API Key 列表响应
#[derive(Debug, Serialize)]
pub struct ListApiKeysResponse {
    pub api_keys: Vec<ApiKeyDto>,
    pub total: u64,
    pub page: u32,
    pub limit: u32,
}

/// API Key 数据传输对象
#[derive(Debug, Serialize)]
pub struct ApiKeyDto {
    pub id: String,
    pub name: String,
    pub key_preview: String,
    pub permissions: Vec<String>,
    pub enabled: bool,
    pub rate_limit: Option<u32>,
    pub request_count: i64,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// 获取所有 API Keys
pub async fn list_api_keys(
    Query(query): Query<ListApiKeysQuery>,
    State(state): State<Arc<AdminAppState>>,
) -> Result<Json<ListApiKeysResponse>, StatusCode> {
    let include_disabled = query.include_disabled.unwrap_or(true);
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);

    match state.auth.get_api_keys().await {
        Ok(api_keys) => {
            // 过滤禁用的密钥
            let filtered_keys: Vec<_> = api_keys
                .into_iter()
                .filter(|key| include_disabled || key.enabled)
                .collect();

            let total = filtered_keys.len() as u64;
            
            // 分页
            let start = ((page - 1) * limit) as usize;
            let end = (start + limit as usize).min(filtered_keys.len());
            
            let paginated_keys: Vec<ApiKeyDto> = filtered_keys[start..end]
                .iter()
                .map(|key| ApiKeyDto {
                    id: key.id.to_string(),
                    name: key.name.clone(),
                    key_preview: format!("{}...", &key.key_hash[..16]), // 显示哈希的前16个字符
                    permissions: key.permissions.iter().map(|p| format!("{:?}", p)).collect(),
                    enabled: key.enabled,
                    rate_limit: key.rate_limit.map(|r| r as u32),
                    request_count: key.request_count,
                    last_used: key.last_used,
                    expires_at: key.expires_at,
                    created_at: key.created_at,
                    updated_at: key.updated_at,
                })
                .collect();

            Ok(Json(ListApiKeysResponse {
                api_keys: paginated_keys,
                total,
                page,
                limit,
            }))
        }
        Err(e) => {
            error!("Failed to list API keys: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 获取单个 API Key
pub async fn get_api_key(
    Path(key_id): Path<String>,
    State(state): State<Arc<AdminAppState>>,
) -> Result<Json<ApiKeyDto>, StatusCode> {
    match state.auth.get_api_key(&key_id).await {
        Ok(Some(key)) => {
            let dto = ApiKeyDto {
                id: key.id.to_string(),
                name: key.name.clone(),
                key_preview: format!("{}...", &key.key_hash[..16]),
                permissions: key.permissions.iter().map(|p| format!("{:?}", p)).collect(),
                enabled: key.enabled,
                rate_limit: key.rate_limit.map(|r| r as u32),
                request_count: key.request_count,
                last_used: key.last_used,
                expires_at: key.expires_at,
                created_at: key.created_at,
                updated_at: key.updated_at,
            };
            Ok(Json(dto))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get API key {}: {}", key_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 创建 API Key
pub async fn create_api_key(
    State(state): State<Arc<AdminAppState>>,
    Json(request): Json<CreateApiKeyRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.auth.create_api_key(request).await {
        Ok(response) => {
            info!("Created API key: {}", response.api_key.name);
            Ok(Json(serde_json::json!({
                "id": response.api_key.id.to_string(),
                "name": response.api_key.name,
                "key": response.raw_key, // 只在创建时返回完整密钥
                "permissions": response.api_key.permissions.iter().map(|p| format!("{:?}", p)).collect::<Vec<_>>(),
                "rate_limit": response.api_key.rate_limit,
                "expires_at": response.api_key.expires_at,
                "created_at": response.api_key.created_at,
            })))
        }
        Err(e) => {
            error!("Failed to create API key: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 更新 API Key
pub async fn update_api_key(
    Path(key_id): Path<String>,
    State(state): State<Arc<AdminAppState>>,
    Json(request): Json<UpdateApiKeyRequest>,
) -> Result<Json<ApiKeyDto>, StatusCode> {
    match state.auth.update_api_key(&key_id, request).await {
        Ok(key) => {
            info!("Updated API key: {} ({})", key.name, key.id);
            let dto = ApiKeyDto {
                id: key.id.to_string(),
                name: key.name.clone(),
                key_preview: format!("{}...", &key.key_hash[..16]),
                permissions: key.permissions.iter().map(|p| format!("{:?}", p)).collect(),
                enabled: key.enabled,
                rate_limit: key.rate_limit.map(|r| r as u32),
                request_count: key.request_count,
                last_used: key.last_used,
                expires_at: key.expires_at,
                created_at: key.created_at,
                updated_at: key.updated_at,
            };
            Ok(Json(dto))
        }
        Err(e) => {
            error!("Failed to update API key {}: {}", key_id, e);
            if e.contains("not found") {
                Err(StatusCode::NOT_FOUND)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

/// 删除 API Key
pub async fn delete_api_key(
    Path(key_id): Path<String>,
    State(state): State<Arc<AdminAppState>>,
) -> Result<StatusCode, StatusCode> {
    match state.auth.delete_api_key(&key_id).await {
        Ok(_) => {
            info!("Deleted API key: {}", key_id);
            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => {
            error!("Failed to delete API key {}: {}", key_id, e);
            if e.contains("not found") {
                Err(StatusCode::NOT_FOUND)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

/// 切换 API Key 启用/禁用状态
pub async fn toggle_api_key(
    Path(key_id): Path<String>,
    State(state): State<Arc<AdminAppState>>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<ApiKeyDto>, StatusCode> {
    let enabled = body.get("enabled")
        .and_then(|v| v.as_bool())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let request = UpdateApiKeyRequest {
        enabled: Some(enabled),
        ..Default::default()
    };

    match state.auth.update_api_key(&key_id, request).await {
        Ok(key) => {
            info!("Toggled API key {} to {}", key_id, if enabled { "enabled" } else { "disabled" });
            let dto = ApiKeyDto {
                id: key.id.to_string(),
                name: key.name.clone(),
                key_preview: format!("{}...", &key.key_hash[..16]),
                permissions: key.permissions.iter().map(|p| format!("{:?}", p)).collect(),
                enabled: key.enabled,
                rate_limit: key.rate_limit.map(|r| r as u32),
                request_count: key.request_count,
                last_used: key.last_used,
                expires_at: key.expires_at,
                created_at: key.created_at,
                updated_at: key.updated_at,
            };
            Ok(Json(dto))
        }
        Err(e) => {
            error!("Failed to toggle API key {}: {}", key_id, e);
            if e.contains("not found") {
                Err(StatusCode::NOT_FOUND)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

/// 重新生成 API Key
pub async fn regenerate_api_key(
    Path(key_id): Path<String>,
    State(state): State<Arc<AdminAppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.auth.regenerate_api_key(&key_id).await {
        Ok(response) => {
            info!("Regenerated API key: {}", response.api_key.name);
            Ok(Json(serde_json::json!({
                "id": response.api_key.id.to_string(),
                "name": response.api_key.name,
                "key": response.raw_key, // 只在重新生成时返回完整密钥
                "permissions": response.api_key.permissions.iter().map(|p| format!("{:?}", p)).collect::<Vec<_>>(),
                "rate_limit": response.api_key.rate_limit,
                "expires_at": response.api_key.expires_at,
                "created_at": response.api_key.created_at,
            })))
        }
        Err(e) => {
            error!("Failed to regenerate API key {}: {}", key_id, e);
            if e.contains("not found") {
                Err(StatusCode::NOT_FOUND)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

/// 获取 API Key 使用统计
pub async fn get_api_key_usage(
    Path(key_id): Path<String>,
    Query(query): Query<serde_json::Value>,
    State(state): State<Arc<AdminAppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let time_range = query.get("range")
        .and_then(|v| v.as_str())
        .unwrap_or("7d");

    match state.auth.get_api_key_usage_stats(&key_id).await {
        Ok(stats) => {
            Ok(Json(serde_json::json!({
                "key_id": key_id,
                "time_range": time_range,
                "total_requests": stats.total_requests,
                "requests_today": stats.requests_today,
                "requests_this_week": stats.requests_this_week,
                "requests_this_month": stats.requests_this_month,
                "last_request_time": stats.last_request_time,
                "average_requests_per_day": stats.average_requests_per_day,
                "top_endpoints": stats.top_endpoints,
                "error_rate": stats.error_rate,
            })))
        }
        Err(e) => {
            error!("Failed to get API key usage stats for {}: {}", key_id, e);
            if e.contains("not found") {
                Err(StatusCode::NOT_FOUND)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

/// 测试 API Key
pub async fn test_api_key(
    State(state): State<Arc<AdminAppState>>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let api_key = body.get("api_key")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    // 尝试验证 API Key
    match state.auth.validate_api_key(api_key, "test").await {
        Ok(key) => {
            Ok(Json(serde_json::json!({
                "valid": true,
                "name": key.name,
                "permissions": key.permissions.iter().map(|p| format!("{:?}", p)).collect::<Vec<_>>(),
                "rate_limit": key.rate_limit,
                "enabled": key.enabled,
                "expires_at": key.expires_at,
            })))
        }
        Err(e) => {
            Ok(Json(serde_json::json!({
                "valid": false,
                "error": e
            })))
        }
    }
}

/// 获取可用权限列表
pub async fn get_available_permissions() -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    let permissions = vec![
        serde_json::json!({
            "id": "read_transactions",
            "name": "Read Transactions",
            "description": "View transaction data and search transactions"
        }),
        serde_json::json!({
            "id": "read_addresses",
            "name": "Read Addresses",
            "description": "View address information and statistics"
        }),
        serde_json::json!({
            "id": "read_blocks",
            "name": "Read Blocks",
            "description": "View block information and data"
        }),
        serde_json::json!({
            "id": "manage_webhooks",
            "name": "Manage Webhooks",
            "description": "Create, update, and delete webhooks"
        }),
        serde_json::json!({
            "id": "manage_websockets",
            "name": "Manage WebSockets",
            "description": "Manage WebSocket connections and subscriptions"
        }),
        serde_json::json!({
            "id": "manage_api_keys",
            "name": "Manage API Keys",
            "description": "Create and manage other API keys"
        }),
        serde_json::json!({
            "id": "read_logs",
            "name": "Read Logs",
            "description": "View system logs and activity"
        }),
        serde_json::json!({
            "id": "manage_system",
            "name": "Manage System",
            "description": "Access system configuration and management"
        }),
    ];

    Ok(Json(permissions))
}

// 为 UpdateApiKeyRequest 实现 Default trait
impl Default for UpdateApiKeyRequest {
    fn default() -> Self {
        Self {
            name: None,
            permissions: None,
            rate_limit: None,
            ip_whitelist: None,
            enabled: None,
            expires_in_days: None,
            description: None,
        }
    }
}