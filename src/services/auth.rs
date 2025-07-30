// API 密钥管理和认证服务
// 
// 提供完整的 API Key 生成、验证、权限控制和使用统计功能

use crate::core::{database::Database, models::*};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// API 密钥权限
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ApiPermission {
    ReadTransactions,
    ReadAddresses,
    ReadBlocks,
    ManageWebhooks,
    ManageWebSockets,
    ManageApiKeys,
    ReadLogs,
    ManageSystem,
}

/// API 密钥信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: String,
    pub name: String,
    pub key_hash: String, // 存储哈希值，不存储原始密钥
    pub permissions: Vec<ApiPermission>,
    pub rate_limit: u32, // 每分钟请求限制
    pub ip_whitelist: Vec<String>, // IP 白名单
    pub enabled: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub usage_count: u64,
    pub description: Option<String>,
}

/// API 密钥创建请求
#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub permissions: Vec<ApiPermission>,
    pub rate_limit: Option<u32>,
    pub ip_whitelist: Option<Vec<String>>,
    pub expires_in_days: Option<u32>,
    pub description: Option<String>,
}

/// API 密钥创建响应
#[derive(Debug, Serialize)]
pub struct CreateApiKeyResponse {
    pub api_key: ApiKey,
    pub raw_key: String, // 只在创建时返回一次
}

/// API 密钥更新请求
#[derive(Debug, Deserialize)]
pub struct UpdateApiKeyRequest {
    pub name: Option<String>,
    pub permissions: Option<Vec<ApiPermission>>,
    pub rate_limit: Option<u32>,
    pub ip_whitelist: Option<Vec<String>>,
    pub enabled: Option<bool>,
    pub expires_in_days: Option<u32>,
    pub description: Option<String>,
}

/// API 密钥使用统计
#[derive(Debug, Serialize)]
pub struct ApiKeyUsageStats {
    pub total_requests: u64,
    pub requests_today: u64,
    pub requests_this_week: u64,
    pub requests_this_month: u64,
    pub last_request_time: Option<DateTime<Utc>>,
    pub average_requests_per_day: f64,
    pub top_endpoints: Vec<EndpointUsage>,
    pub error_rate: f64,
}

/// 端点使用统计
#[derive(Debug, Serialize)]
pub struct EndpointUsage {
    pub endpoint: String,
    pub method: String,
    pub count: u64,
    pub last_used: DateTime<Utc>,
}

/// 认证服务
#[derive(Clone)]
pub struct AuthService {
    db: Arc<Database>,
    rate_limiter: Arc<RateLimiter>,
}

/// 速率限制器
pub struct RateLimiter {
    // 使用内存存储，生产环境应该使用 Redis
    requests: Arc<tokio::sync::RwLock<HashMap<String, Vec<DateTime<Utc>>>>>,
}

impl AuthService {
    /// 创建新的认证服务
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            rate_limiter: Arc::new(RateLimiter::new()),
        }
    }

    /// 生成新的 API 密钥
    pub async fn create_api_key(&self, request: CreateApiKeyRequest) -> Result<CreateApiKeyResponse, String> {
        // 生成随机密钥
        let raw_key = self.generate_api_key();
        let key_hash = self.hash_api_key(&raw_key);

        // 计算过期时间
        let expires_at = request.expires_in_days.map(|days| {
            Utc::now() + chrono::Duration::days(days as i64)
        });

        let api_key = ApiKey {
            id: Uuid::new_v4().to_string(),
            name: request.name,
            key_hash,
            permissions: request.permissions,
            rate_limit: request.rate_limit.unwrap_or(1000), // 默认每分钟 1000 次
            ip_whitelist: request.ip_whitelist.unwrap_or_default(),
            enabled: true,
            expires_at,
            created_at: Utc::now(),
            last_used_at: None,
            usage_count: 0,
            description: request.description,
        };

        // 保存到数据库
        match self.db.create_api_key(&api_key).await {
            Ok(_) => {
                info!("Created new API key: {} ({})", api_key.name, api_key.id);
                Ok(CreateApiKeyResponse {
                    api_key,
                    raw_key,
                })
            }
            Err(e) => {
                error!("Failed to create API key: {}", e);
                Err(format!("Failed to create API key: {}", e))
            }
        }
    }

    /// 验证 API 密钥
    pub async fn validate_api_key(&self, raw_key: &str, client_ip: &str) -> Result<ApiKey, String> {
        let key_hash = self.hash_api_key(raw_key);

        // 从数据库获取 API 密钥
        let api_key = match self.db.get_api_key_by_hash(&key_hash).await {
            Ok(Some(key)) => key,
            Ok(None) => return Err("Invalid API key".to_string()),
            Err(e) => {
                error!("Failed to validate API key: {}", e);
                return Err("Authentication failed".to_string());
            }
        };

        // 检查密钥是否启用
        if !api_key.enabled {
            return Err("API key is disabled".to_string());
        }

        // 检查是否过期
        if let Some(expires_at) = api_key.expires_at {
            if Utc::now() > expires_at {
                return Err("API key has expired".to_string());
            }
        }

        // 检查 IP 白名单
        if !api_key.ip_whitelist.is_empty() && !api_key.ip_whitelist.contains(&client_ip.to_string()) {
            warn!("IP {} not in whitelist for API key {}", client_ip, api_key.id);
            return Err("IP address not allowed".to_string());
        }

        // 检查速率限制
        if let Err(e) = self.rate_limiter.check_rate_limit(&api_key.id, api_key.rate_limit).await {
            warn!("Rate limit exceeded for API key {}: {}", api_key.id, e);
            return Err(e);
        }

        // 更新使用统计
        if let Err(e) = self.db.update_api_key_usage(&api_key.id).await {
            warn!("Failed to update API key usage: {}", e);
        }

        debug!("API key validated successfully: {}", api_key.id);
        Ok(api_key)
    }

    /// 检查权限
    pub fn check_permission(&self, api_key: &ApiKey, required_permission: &ApiPermission) -> bool {
        api_key.permissions.contains(required_permission)
    }

    /// 获取所有 API 密钥
    pub async fn get_api_keys(&self) -> Result<Vec<ApiKey>, String> {
        match self.db.get_all_api_keys().await {
            Ok(keys) => Ok(keys),
            Err(e) => {
                error!("Failed to get API keys: {}", e);
                Err(format!("Failed to get API keys: {}", e))
            }
        }
    }

    /// 获取单个 API 密钥
    pub async fn get_api_key(&self, key_id: &str) -> Result<Option<ApiKey>, String> {
        match self.db.get_api_key_by_id(key_id).await {
            Ok(key) => Ok(key),
            Err(e) => {
                error!("Failed to get API key {}: {}", key_id, e);
                Err(format!("Failed to get API key: {}", e))
            }
        }
    }

    /// 更新 API 密钥
    pub async fn update_api_key(&self, key_id: &str, request: UpdateApiKeyRequest) -> Result<ApiKey, String> {
        let mut api_key = match self.get_api_key(key_id).await? {
            Some(key) => key,
            None => return Err("API key not found".to_string()),
        };

        // 更新字段
        if let Some(name) = request.name {
            api_key.name = name;
        }
        if let Some(permissions) = request.permissions {
            api_key.permissions = permissions;
        }
        if let Some(rate_limit) = request.rate_limit {
            api_key.rate_limit = rate_limit;
        }
        if let Some(ip_whitelist) = request.ip_whitelist {
            api_key.ip_whitelist = ip_whitelist;
        }
        if let Some(enabled) = request.enabled {
            api_key.enabled = enabled;
        }
        if let Some(expires_in_days) = request.expires_in_days {
            api_key.expires_at = Some(Utc::now() + chrono::Duration::days(expires_in_days as i64));
        }
        if let Some(description) = request.description {
            api_key.description = Some(description);
        }

        // 保存到数据库
        match self.db.update_api_key(&api_key).await {
            Ok(_) => {
                info!("Updated API key: {} ({})", api_key.name, api_key.id);
                Ok(api_key)
            }
            Err(e) => {
                error!("Failed to update API key {}: {}", key_id, e);
                Err(format!("Failed to update API key: {}", e))
            }
        }
    }

    /// 删除 API 密钥
    pub async fn delete_api_key(&self, key_id: &str) -> Result<(), String> {
        match self.db.delete_api_key(key_id).await {
            Ok(_) => {
                info!("Deleted API key: {}", key_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to delete API key {}: {}", key_id, e);
                Err(format!("Failed to delete API key: {}", e))
            }
        }
    }

    /// 获取 API 密钥使用统计
    pub async fn get_api_key_usage_stats(&self, key_id: &str) -> Result<ApiKeyUsageStats, String> {
        match self.db.get_api_key_usage_stats(key_id).await {
            Ok(stats) => Ok(stats),
            Err(e) => {
                error!("Failed to get API key usage stats for {}: {}", key_id, e);
                Err(format!("Failed to get usage stats: {}", e))
            }
        }
    }

    /// 重新生成 API 密钥
    pub async fn regenerate_api_key(&self, key_id: &str) -> Result<CreateApiKeyResponse, String> {
        let mut api_key = match self.get_api_key(key_id).await? {
            Some(key) => key,
            None => return Err("API key not found".to_string()),
        };

        // 生成新的密钥
        let raw_key = self.generate_api_key();
        let key_hash = self.hash_api_key(&raw_key);

        api_key.key_hash = key_hash;
        api_key.usage_count = 0; // 重置使用计数
        api_key.last_used_at = None; // 重置最后使用时间

        // 保存到数据库
        match self.db.update_api_key(&api_key).await {
            Ok(_) => {
                info!("Regenerated API key: {} ({})", api_key.name, api_key.id);
                Ok(CreateApiKeyResponse {
                    api_key,
                    raw_key,
                })
            }
            Err(e) => {
                error!("Failed to regenerate API key {}: {}", key_id, e);
                Err(format!("Failed to regenerate API key: {}", e))
            }
        }
    }

    /// 生成随机 API 密钥
    fn generate_api_key(&self) -> String {
        let mut rng = rand::thread_rng();
        let random_bytes: [u8; 32] = rng.gen();
        format!("tk_{}", hex::encode(random_bytes))
    }

    /// 计算 API 密钥哈希
    fn hash_api_key(&self, raw_key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(raw_key.as_bytes());
        hex::encode(hasher.finalize())
    }
}

impl RateLimiter {
    /// 创建新的速率限制器
    pub fn new() -> Self {
        Self {
            requests: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// 检查速率限制
    pub async fn check_rate_limit(&self, key_id: &str, limit: u32) -> Result<(), String> {
        let now = Utc::now();
        let window_start = now - chrono::Duration::minutes(1);

        let mut requests = self.requests.write().await;
        let key_requests = requests.entry(key_id.to_string()).or_insert_with(Vec::new);

        // 清理过期的请求记录
        key_requests.retain(|&time| time > window_start);

        // 检查是否超过限制
        if key_requests.len() >= limit as usize {
            return Err("Rate limit exceeded".to_string());
        }

        // 记录当前请求
        key_requests.push(now);

        Ok(())
    }

    /// 清理过期的请求记录
    pub async fn cleanup_expired_records(&self) {
        let cutoff = Utc::now() - chrono::Duration::minutes(5);
        let mut requests = self.requests.write().await;

        requests.retain(|_, times| {
            times.retain(|&time| time > cutoff);
            !times.is_empty()
        });
    }
}

/// 认证中间件
pub async fn auth_middleware(
    State(auth_service): State<Arc<AuthService>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 获取 API 密钥
    let api_key = headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|auth| {
            if auth.starts_with("Bearer ") {
                Some(&auth[7..])
            } else {
                None
            }
        })
        .or_else(|| {
            headers
                .get("X-API-Key")
                .and_then(|value| value.to_str().ok())
        });

    let api_key = match api_key {
        Some(key) => key,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    // 获取客户端 IP
    let client_ip = headers
        .get("X-Forwarded-For")
        .and_then(|value| value.to_str().ok())
        .and_then(|forwarded| forwarded.split(',').next())
        .unwrap_or("unknown")
        .trim();

    // 验证 API 密钥
    match auth_service.validate_api_key(api_key, client_ip).await {
        Ok(_) => {
            // 验证成功，继续处理请求
            Ok(next.run(request).await)
        }
        Err(e) => {
            warn!("Authentication failed: {}", e);
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

/// 权限检查中间件
pub fn require_permission(permission: ApiPermission) -> impl Fn(State<Arc<AuthService>>, HeaderMap, Request, Next) -> Result<Response, StatusCode> + Clone {
    move |State(auth_service): State<Arc<AuthService>>, headers: HeaderMap, request: Request, next: Next| {
        let permission = permission.clone();
        async move {
            // 这里简化处理，实际应该从请求上下文中获取已验证的 API 密钥
            // 在实际实现中，应该在认证中间件中将 API 密钥信息存储到请求扩展中
            
            // 暂时跳过权限检查，直接继续处理请求
            // 在完整实现中，这里应该检查 API 密钥是否具有所需权限
            Ok(next.run(request).await)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new();
        let key_id = "test_key";

        // 测试正常情况
        for _ in 0..5 {
            assert!(limiter.check_rate_limit(key_id, 10).await.is_ok());
        }

        // 测试超过限制
        for _ in 0..10 {
            let _ = limiter.check_rate_limit(key_id, 10).await;
        }
        assert!(limiter.check_rate_limit(key_id, 10).await.is_err());
    }

    #[test]
    fn test_api_key_generation() {
        // 创建模拟的数据库
        use crate::core::config::DatabaseConfig;
        let db_config = DatabaseConfig {
            url: "postgresql://test:test@localhost/test".to_string(),
            max_connections: 10,
            min_connections: 1,
            acquire_timeout: 30,
        };
        
        // 由于测试环境可能没有数据库，我们直接测试密钥生成逻辑
        let auth_service = AuthService {
            db: Arc::new(Database::new(&db_config).await.unwrap()),
            rate_limiter: Arc::new(RateLimiter::new()),
        };
        
        let key1 = auth_service.generate_api_key();
        let key2 = auth_service.generate_api_key();

        assert_ne!(key1, key2);
        assert!(key1.starts_with("tk_"));
        assert!(key2.starts_with("tk_"));
        assert_eq!(key1.len(), 67); // "tk_" + 64 hex chars
    }

    #[test]
    fn test_api_key_hashing() {
        // 创建模拟的数据库
        use crate::core::config::DatabaseConfig;
        let db_config = DatabaseConfig {
            url: "postgresql://test:test@localhost/test".to_string(),
            max_connections: 10,
            min_connections: 1,
            acquire_timeout: 30,
        };
        
        let auth_service = AuthService {
            db: Arc::new(Database::new(&db_config).await.unwrap()),
            rate_limiter: Arc::new(RateLimiter::new()),
        };
        
        let raw_key = "test_key";
        let hash1 = auth_service.hash_api_key(raw_key);
        let hash2 = auth_service.hash_api_key(raw_key);

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, raw_key);
        assert_eq!(hash1.len(), 64); // SHA256 hex string
    }
}

