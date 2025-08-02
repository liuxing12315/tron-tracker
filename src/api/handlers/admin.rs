// 管理后台 API 处理器
// 
// 提供完整的管理后台功能，包括仪表板、系统监控、配置管理等

use crate::core::database::Database;
use crate::services::{
    auth::AuthService,
    cache::CacheService,
    scanner::ScannerService,
    websocket::WebSocketService,
    webhook::WebhookService,
};
use axum::{
    extract::{Query, State, Path},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info};

/// 管理后台应用状态
pub struct AdminAppState {
    pub db: Database,
    pub cache: CacheService,
    pub auth: AuthService,
    pub scanner: ScannerService,
    pub websocket: WebSocketService,
    pub webhook: WebhookService,
}

/// 仪表板统计数据
#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub system_overview: SystemOverview,
    pub transaction_stats: TransactionStats,
    pub websocket_stats: WebSocketStats,
    pub webhook_stats: WebhookStats,
    pub api_stats: ApiStats,
    pub scanner_stats: ScannerStats,
    pub performance_metrics: PerformanceMetrics,
}

/// 系统概览
#[derive(Debug, Serialize)]
pub struct SystemOverview {
    pub uptime_seconds: u64,
    pub version: String,
    pub environment: String,
    pub total_memory_mb: u64,
    pub used_memory_mb: u64,
    pub cpu_usage_percent: f64,
    pub disk_usage_percent: f64,
    pub active_connections: u32,
    pub health_status: String,
}

/// 交易统计
#[derive(Debug, Serialize)]
pub struct TransactionStats {
    pub total_transactions: u64,
    pub transactions_today: u64,
    pub transactions_this_week: u64,
    pub transactions_this_month: u64,
    pub success_rate: f64,
    pub average_amount: String,
    pub total_volume: String,
    pub unique_addresses: u64,
    pub top_tokens: Vec<TokenStats>,
}

/// 代币统计
#[derive(Debug, Serialize)]
pub struct TokenStats {
    pub symbol: String,
    pub transaction_count: u64,
    pub volume: String,
    pub percentage: f64,
}

/// WebSocket 统计
#[derive(Debug, Serialize)]
pub struct WebSocketStats {
    pub active_connections: u32,
    pub total_connections_today: u64,
    pub messages_sent_today: u64,
    pub average_latency_ms: f64,
    pub subscription_count: u64,
    pub connection_by_type: HashMap<String, u32>,
    pub top_subscribed_events: Vec<EventSubscriptionStats>,
}

/// 事件订阅统计
#[derive(Debug, Serialize)]
pub struct EventSubscriptionStats {
    pub event_type: String,
    pub subscriber_count: u32,
    pub messages_sent: u64,
}

/// Webhook 统计
#[derive(Debug, Serialize)]
pub struct WebhookStats {
    pub total_webhooks: u32,
    pub active_webhooks: u32,
    pub total_deliveries_today: u64,
    pub successful_deliveries_today: u64,
    pub failed_deliveries_today: u64,
    pub average_response_time_ms: f64,
    pub success_rate: f64,
    pub retry_rate: f64,
}

/// API 统计
#[derive(Debug, Serialize)]
pub struct ApiStats {
    pub total_api_keys: u32,
    pub active_api_keys: u32,
    pub total_requests_today: u64,
    pub successful_requests_today: u64,
    pub failed_requests_today: u64,
    pub average_response_time_ms: f64,
    pub top_endpoints: Vec<EndpointStats>,
    pub rate_limited_requests: u64,
}

/// 端点统计
#[derive(Debug, Serialize)]
pub struct EndpointStats {
    pub endpoint: String,
    pub method: String,
    pub request_count: u64,
    pub average_response_time_ms: f64,
    pub error_rate: f64,
}

/// 扫描器统计
#[derive(Debug, Serialize)]
pub struct ScannerStats {
    pub current_block: u64,
    pub latest_block: u64,
    pub blocks_behind: u64,
    pub scanning_speed: f64, // 块/分钟
    pub transactions_processed_today: u64,
    pub errors_today: u64,
    pub last_scan_time: chrono::DateTime<chrono::Utc>,
    pub scan_status: String,
}

/// 性能指标
#[derive(Debug, Serialize)]
pub struct PerformanceMetrics {
    pub database_connection_pool: PoolStats,
    pub redis_connection_pool: PoolStats,
    pub cache_hit_rate: f64,
    pub average_query_time_ms: f64,
    pub slow_queries_count: u64,
    pub memory_usage_trend: Vec<MetricPoint>,
    pub cpu_usage_trend: Vec<MetricPoint>,
}

/// 连接池统计
#[derive(Debug, Serialize)]
pub struct PoolStats {
    pub active_connections: u32,
    pub idle_connections: u32,
    pub max_connections: u32,
    pub total_connections: u32,
}

/// 指标数据点
#[derive(Debug, Serialize)]
pub struct MetricPoint {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub value: f64,
}

/// 系统配置
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemConfig {
    pub scanner_config: ScannerConfig,
    pub database_config: DatabaseConfig,
    pub cache_config: CacheConfig,
    pub api_config: ApiConfig,
    pub webhook_config: WebhookConfig,
    pub websocket_config: WebSocketConfig,
}

/// 扫描器配置
#[derive(Debug, Serialize, Deserialize)]
pub struct ScannerConfig {
    pub enabled: bool,
    pub scan_interval_ms: u64,
    pub batch_size: u32,
    pub start_block: Option<u64>,
    pub end_block: Option<u64>,
    pub nodes: Vec<NodeConfig>,
}

/// 节点配置
#[derive(Debug, Serialize, Deserialize)]
pub struct NodeConfig {
    pub name: String,
    pub url: String,
    pub api_key: Option<String>,
    pub priority: u32,
    pub enabled: bool,
    pub timeout_ms: u64,
}

/// 数据库配置
#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub max_connections: u32,
    pub connection_timeout_ms: u64,
}

/// 缓存配置
#[derive(Debug, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub redis_url: String,
    pub max_connections: u32,
    pub default_ttl_seconds: u64,
    pub max_memory_mb: u64,
}

/// API 配置
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub cors_enabled: bool,
    pub rate_limit_enabled: bool,
    pub default_rate_limit: u32,
    pub request_timeout_ms: u64,
}

/// Webhook 配置
#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub enabled: bool,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub timeout_ms: u64,
    pub max_concurrent_deliveries: u32,
}

/// WebSocket 配置
#[derive(Debug, Serialize, Deserialize)]
pub struct WebSocketConfig {
    pub enabled: bool,
    pub port: u16,
    pub max_connections: u32,
    pub heartbeat_interval_ms: u64,
    pub message_buffer_size: u32,
}

/// 日志查询参数
#[derive(Debug, Deserialize)]
pub struct LogQueryParams {
    pub level: Option<String>,
    pub module: Option<String>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub search: Option<String>,
}

/// 日志条目
#[derive(Debug, Serialize)]
pub struct LogEntry {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: String,
    pub module: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub trace_id: Option<String>,
}

/// 日志查询响应
#[derive(Debug, Serialize)]
pub struct LogQueryResponse {
    pub logs: Vec<LogEntry>,
    pub total_count: u64,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

/// 获取仪表板统计数据
pub async fn get_dashboard_stats(
    State(state): State<Arc<AdminAppState>>,
) -> Result<Json<DashboardStats>, StatusCode> {
    // 并行获取各种统计数据
    let (
        system_overview,
        transaction_stats,
        websocket_stats,
        webhook_stats,
        api_stats,
        scanner_stats,
        performance_metrics,
    ) = tokio::try_join!(
        get_system_overview(&state),
        get_transaction_stats(&state),
        get_websocket_stats(&state),
        get_webhook_stats(&state),
        get_api_stats(&state),
        get_scanner_stats(&state),
        get_performance_metrics(&state),
    ).map_err(|e| {
        error!("Failed to get dashboard stats: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(DashboardStats {
        system_overview,
        transaction_stats,
        websocket_stats,
        webhook_stats,
        api_stats,
        scanner_stats,
        performance_metrics,
    }))
}

/// 获取系统概览
async fn get_system_overview(_state: &AdminAppState) -> Result<SystemOverview, String> {
    // 这里应该实际获取系统指标，现在返回模拟数据
    Ok(SystemOverview {
        uptime_seconds: 86400, // 1 天
        version: "2.0.0".to_string(),
        environment: "production".to_string(),
        total_memory_mb: 8192,
        used_memory_mb: 2048,
        cpu_usage_percent: 15.2,
        disk_usage_percent: 42.1,
        active_connections: 156,
        health_status: "healthy".to_string(),
    })
}

/// 获取交易统计
async fn get_transaction_stats(state: &AdminAppState) -> Result<TransactionStats, String> {
    match state.db.get_transaction_statistics().await {
        Ok(stats) => Ok(stats),
        Err(e) => {
            error!("Failed to get transaction stats: {}", e);
            Err(e.to_string())
        }
    }
}

/// 获取 WebSocket 统计
async fn get_websocket_stats(state: &AdminAppState) -> Result<WebSocketStats, String> {
    match state.websocket.get_statistics().await {
        Ok(stats) => Ok(stats),
        Err(e) => {
            error!("Failed to get WebSocket stats: {}", e);
            Err(e.to_string())
        }
    }
}

/// 获取 Webhook 统计
async fn get_webhook_stats(state: &AdminAppState) -> Result<WebhookStats, String> {
    match state.webhook.get_statistics().await {
        Ok(stats) => Ok(stats),
        Err(e) => {
            error!("Failed to get Webhook stats: {}", e);
            Err(e.to_string())
        }
    }
}

/// 获取 API 统计
async fn get_api_stats(state: &AdminAppState) -> Result<ApiStats, String> {
    match state.db.get_api_statistics().await {
        Ok(stats) => Ok(stats),
        Err(e) => {
            error!("Failed to get API stats: {}", e);
            Err(e.to_string())
        }
    }
}

/// 获取扫描器统计
async fn get_scanner_stats(state: &AdminAppState) -> Result<ScannerStats, String> {
    match state.scanner.get_statistics().await {
        Ok(stats) => Ok(stats),
        Err(e) => {
            error!("Failed to get scanner stats: {}", e);
            Err(e.to_string())
        }
    }
}

/// 获取性能指标
async fn get_performance_metrics(state: &AdminAppState) -> Result<PerformanceMetrics, String> {
    match state.db.get_performance_metrics().await {
        Ok(metrics) => Ok(metrics),
        Err(e) => {
            error!("Failed to get performance metrics: {}", e);
            Err(e.to_string())
        }
    }
}

/// 获取系统配置
pub async fn get_system_config(
    State(state): State<Arc<AdminAppState>>,
) -> Result<Json<SystemConfig>, StatusCode> {
    match state.db.get_all_system_config().await {
        Ok(config) => Ok(Json(config)),
        Err(e) => {
            error!("Failed to get system config: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 更新系统配置
pub async fn update_system_config(
    State(state): State<Arc<AdminAppState>>,
    Json(config): Json<SystemConfig>,
) -> Result<Json<SystemConfig>, StatusCode> {
    // 验证配置
    if let Err(validation_error) = validate_system_config(&config) {
        error!("Configuration validation failed: {}", validation_error);
        return Err(StatusCode::BAD_REQUEST);
    }
    
    match state.db.update_system_config(&config).await {
        Ok(_) => {
            info!("System configuration updated successfully");
            Ok(Json(config))
        }
        Err(e) => {
            error!("Failed to update system config: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 验证系统配置
fn validate_system_config(config: &SystemConfig) -> Result<(), String> {
    // 验证扫描器配置
    if config.scanner_config.scan_interval_ms < 1000 {
        return Err("Scanner interval must be at least 1000ms".to_string());
    }
    if config.scanner_config.batch_size < 1 || config.scanner_config.batch_size > 1000 {
        return Err("Scanner batch size must be between 1 and 1000".to_string());
    }
    
    // 验证数据库配置
    if config.database_config.port == 0 || config.database_config.port > 65535 {
        return Err("Database port must be between 1 and 65535".to_string());
    }
    if config.database_config.max_connections == 0 || config.database_config.max_connections > 1000 {
        return Err("Database max connections must be between 1 and 1000".to_string());
    }
    if config.database_config.connection_timeout_ms < 1000 {
        return Err("Database connection timeout must be at least 1000ms".to_string());
    }
    
    // 验证缓存配置
    if config.cache_config.enabled && config.cache_config.redis_url.is_empty() {
        return Err("Redis URL is required when cache is enabled".to_string());
    }
    if config.cache_config.max_connections == 0 || config.cache_config.max_connections > 1000 {
        return Err("Cache max connections must be between 1 and 1000".to_string());
    }
    if config.cache_config.default_ttl_seconds == 0 {
        return Err("Cache TTL must be greater than 0".to_string());
    }
    
    // 验证API配置
    if config.api_config.port == 0 || config.api_config.port > 65535 {
        return Err("API port must be between 1 and 65535".to_string());
    }
    if config.api_config.default_rate_limit == 0 {
        return Err("API rate limit must be greater than 0".to_string());
    }
    if config.api_config.request_timeout_ms < 1000 {
        return Err("API request timeout must be at least 1000ms".to_string());
    }
    
    // 验证Webhook配置
    if config.webhook_config.enabled {
        if config.webhook_config.max_retries > 10 {
            return Err("Webhook max retries must not exceed 10".to_string());
        }
        if config.webhook_config.retry_delay_ms < 100 {
            return Err("Webhook retry delay must be at least 100ms".to_string());
        }
        if config.webhook_config.timeout_ms < 1000 {
            return Err("Webhook timeout must be at least 1000ms".to_string());
        }
        if config.webhook_config.max_concurrent_deliveries == 0 || config.webhook_config.max_concurrent_deliveries > 1000 {
            return Err("Webhook max concurrent deliveries must be between 1 and 1000".to_string());
        }
    }
    
    // 验证WebSocket配置
    if config.websocket_config.enabled {
        if config.websocket_config.port == 0 || config.websocket_config.port > 65535 {
            return Err("WebSocket port must be between 1 and 65535".to_string());
        }
        if config.websocket_config.max_connections == 0 || config.websocket_config.max_connections > 100000 {
            return Err("WebSocket max connections must be between 1 and 100000".to_string());
        }
        if config.websocket_config.heartbeat_interval_ms < 5000 {
            return Err("WebSocket heartbeat interval must be at least 5000ms".to_string());
        }
        if config.websocket_config.message_buffer_size == 0 || config.websocket_config.message_buffer_size > 10000 {
            return Err("WebSocket message buffer size must be between 1 and 10000".to_string());
        }
    }
    
    // 验证节点配置
    for node in &config.scanner_config.nodes {
        if node.url.is_empty() {
            return Err("Node URL cannot be empty".to_string());
        }
        if !node.url.starts_with("http://") && !node.url.starts_with("https://") {
            return Err("Node URL must start with http:// or https://".to_string());
        }
        if node.timeout_ms < 1000 {
            return Err("Node timeout must be at least 1000ms".to_string());
        }
    }
    
    Ok(())
}

/// 验证系统配置但不保存
pub async fn validate_config(
    Json(config): Json<SystemConfig>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match validate_system_config(&config) {
        Ok(_) => {
            Ok(Json(serde_json::json!({
                "valid": true,
                "message": "Configuration is valid"
            })))
        }
        Err(validation_error) => {
            Ok(Json(serde_json::json!({
                "valid": false,
                "error": validation_error
            })))
        }
    }
}

/// 重置系统配置为默认值
pub async fn reset_system_config(
    State(state): State<Arc<AdminAppState>>,
) -> Result<Json<SystemConfig>, StatusCode> {
    // 创建默认配置
    let default_config = SystemConfig {
        scanner_config: ScannerConfig {
            enabled: true,
            scan_interval_ms: 5000,
            batch_size: 10,
            start_block: Some(62800000),
            end_block: None,
            nodes: vec![
                NodeConfig {
                    name: "TronGrid".to_string(),
                    url: "https://api.trongrid.io".to_string(),
                    api_key: None,
                    priority: 1,
                    enabled: true,
                    timeout_ms: 30000,
                },
            ],
        },
        database_config: DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            database: "tron_tracker".to_string(),
            username: "postgres".to_string(),
            max_connections: 10,
            connection_timeout_ms: 30000,
        },
        cache_config: CacheConfig {
            enabled: true,
            redis_url: "redis://localhost:6379".to_string(),
            max_connections: 10,
            default_ttl_seconds: 3600,
            max_memory_mb: 512,
        },
        api_config: ApiConfig {
            host: "0.0.0.0".to_string(),
            port: 8080,
            cors_enabled: true,
            rate_limit_enabled: true,
            default_rate_limit: 1000,
            request_timeout_ms: 30000,
        },
        webhook_config: WebhookConfig {
            enabled: true,
            max_retries: 3,
            retry_delay_ms: 1000,
            timeout_ms: 30000,
            max_concurrent_deliveries: 10,
        },
        websocket_config: WebSocketConfig {
            enabled: true,
            port: 8081,
            max_connections: 1000,
            heartbeat_interval_ms: 30000,
            message_buffer_size: 1000,
        },
    };
    
    match state.db.update_system_config(&default_config).await {
        Ok(_) => {
            info!("System configuration reset to defaults");
            Ok(Json(default_config))
        }
        Err(e) => {
            error!("Failed to reset system config: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 获取配置变更历史
pub async fn get_config_history(
    State(state): State<Arc<AdminAppState>>,
    Query(query): Query<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let page = query.get("page").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
    let limit = query.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as u32;
    
    match state.db.get_config_history(page, limit).await {
        Ok(history) => Ok(Json(history)),
        Err(e) => {
            error!("Failed to get config history: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 获取日志
pub async fn get_logs(
    Query(params): Query<LogQueryParams>,
    State(state): State<Arc<AdminAppState>>,
) -> Result<Json<LogQueryResponse>, StatusCode> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(50).min(1000);

    match state.db.get_logs(&params, page, limit).await {
        Ok((logs, total_count)) => {
            let total_pages = (total_count as f64 / limit as f64).ceil() as u32;
            
            Ok(Json(LogQueryResponse {
                logs,
                total_count,
                page,
                limit,
                total_pages,
            }))
        }
        Err(e) => {
            error!("Failed to get logs: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 清空日志
pub async fn clear_logs(
    State(state): State<Arc<AdminAppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.db.clear_logs().await {
        Ok(deleted_count) => {
            info!("Cleared {} log entries", deleted_count);
            Ok(Json(serde_json::json!({
                "message": "Logs cleared successfully",
                "deleted_count": deleted_count
            })))
        }
        Err(e) => {
            error!("Failed to clear logs: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 导出日志
pub async fn export_logs(
    Query(params): Query<LogQueryParams>,
    State(state): State<Arc<AdminAppState>>,
) -> Result<String, StatusCode> {
    match state.db.export_logs(&params).await {
        Ok(csv_data) => Ok(csv_data),
        Err(e) => {
            error!("Failed to export logs: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 重启扫描器
pub async fn restart_scanner(
    State(state): State<Arc<AdminAppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.scanner.restart().await {
        Ok(_) => {
            info!("Scanner restarted successfully");
            Ok(Json(serde_json::json!({
                "message": "Scanner restarted successfully"
            })))
        }
        Err(e) => {
            error!("Failed to restart scanner: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 停止扫描器
pub async fn stop_scanner(
    State(state): State<Arc<AdminAppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state.scanner.stop().await;
    info!("Scanner stopped successfully");
    Ok(Json(serde_json::json!({
        "message": "Scanner stopped successfully"
    })))
}

/// 手动扫描指定区块
pub async fn scan_block(
    Path(block_number): Path<u64>,
    State(state): State<Arc<AdminAppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.scanner.scan_block_admin(block_number).await {
        Ok(result) => {
            info!("Block {} scanned successfully", block_number);
            Ok(Json(serde_json::json!({
                "message": "Block scanned successfully",
                "block_number": block_number,
                "transactions_found": result.transactions_count,
                "processing_time_ms": result.processing_time_ms
            })))
        }
        Err(e) => {
            error!("Failed to scan block {}: {}", block_number, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 清空缓存
pub async fn clear_cache(
    State(state): State<Arc<AdminAppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.cache.clear_all().await {
        Ok(_) => {
            info!("Cache cleared successfully");
            Ok(Json(serde_json::json!({
                "message": "Cache cleared successfully"
            })))
        }
        Err(e) => {
            error!("Failed to clear cache: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 获取缓存统计
pub async fn get_cache_stats(
    State(state): State<Arc<AdminAppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.cache.get_statistics().await {
        Ok(stats) => Ok(Json(stats)),
        Err(e) => {
            error!("Failed to get cache stats: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 获取日志统计
pub async fn get_log_stats(
    State(state): State<Arc<AdminAppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // 这里应该实现真实的日志统计查询
    // 暂时返回模拟数据
    let stats = serde_json::json!({
        "total_count": 1250,
        "error_count": 15,
        "warn_count": 45,
        "info_count": 890,
        "debug_count": 300,
        "last_24h": {
            "total": 1250,
            "errors": 15,
            "warnings": 45
        }
    });
    
    Ok(Json(stats))
}

/// 健康检查
pub async fn health_check(
    State(state): State<Arc<AdminAppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut health_status = HashMap::new();
    
    // 检查数据库连接
    health_status.insert("database", match state.db.health_check().await {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    });
    
    // 检查缓存连接
    health_status.insert("cache", match state.cache.health_check().await {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    });
    
    // 检查扫描器状态
    health_status.insert("scanner", match state.scanner.health_check().await {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    });
    
    let overall_status = if health_status.values().all(|&status| status == "healthy") {
        "healthy"
    } else {
        "unhealthy"
    };
    
    Ok(Json(serde_json::json!({
        "status": overall_status,
        "timestamp": chrono::Utc::now(),
        "version": "2.0.0",
        "components": health_status
    })))
}

