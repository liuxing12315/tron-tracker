// TRX Tracker Unified - 主程序入口
// 
// 统一的 TRX/USDT 交易历史查询系统
// 集成区块链扫描、API 服务、WebSocket 通知、Webhook 投递和管理后台

use std::sync::Arc;
use tokio::signal;
use tracing::{info, error, warn};
use tracing_subscriber;

mod core;
mod api;
mod services;

use crate::core::{config::Config, database::Database};
use crate::services::{
    auth::AuthService,
    cache::CacheService,
    scanner::ScannerService,
    websocket::WebSocketService,
    webhook::WebhookService,
};
use crate::api::handlers::admin::AdminAppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();
    
    info!("🚀 Starting TRX Tracker Unified v2.0.0");
    
    // 加载配置
    let config = match Config::load() {
        Ok(config) => {
            info!("✅ Configuration loaded successfully");
            Arc::new(config)
        }
        Err(e) => {
            error!("❌ Failed to load configuration: {}", e);
            return Err(e.into());
        }
    };
    
    // 初始化数据库
    let database = match Database::new(&config.database.url) {
        Ok(db) => {
            info!("✅ Database connection established");
            Arc::new(db)
        }
        Err(e) => {
            error!("❌ Failed to connect to database: {}", e);
            return Err(e.into());
        }
    };
    
    // 运行数据库迁移
    if let Err(e) = database.migrate().await {
        error!("❌ Database migration failed: {}", e);
        return Err(e.into());
    }
    info!("✅ Database migrations completed");
    
    // 初始化缓存服务
    let cache_service = match CacheService::new(&config.cache.redis_url).await {
        Ok(cache) => {
            info!("✅ Cache service initialized");
            Arc::new(cache)
        }
        Err(e) => {
            warn!("⚠️ Cache service initialization failed: {}, continuing without cache", e);
            Arc::new(CacheService::disabled())
        }
    };
    
    // 初始化认证服务
    let auth_service = Arc::new(AuthService::new(database.clone()));
    info!("✅ Authentication service initialized");
    
    // 初始化 Webhook 服务
    let webhook_service = Arc::new(WebhookService::new(
        database.clone(),
        config.webhook.clone(),
    ));
    info!("✅ Webhook service initialized");
    
    // 初始化 WebSocket 服务
    let websocket_service = Arc::new(WebSocketService::new(
        config.websocket.clone(),
    ));
    info!("✅ WebSocket service initialized");
    
    // 初始化扫描器服务
    let scanner_service = Arc::new(ScannerService::new(
        database.clone(),
        cache_service.clone(),
        webhook_service.clone(),
        websocket_service.clone(),
        config.scanner.clone(),
    ));
    info!("✅ Scanner service initialized");
    
    // 创建应用状态
    let app_state = Arc::new(AdminAppState {
        db: (*database).clone(),
        cache: (*cache_service).clone(),
        auth: (*auth_service).clone(),
        scanner: (*scanner_service).clone(),
        websocket: (*websocket_service).clone(),
        webhook: (*webhook_service).clone(),
    });
    
    // 启动各种服务
    let mut handles = Vec::new();
    
    // 启动扫描器服务
    if config.scanner.enabled {
        let scanner_handle = {
            let scanner = scanner_service.clone();
            tokio::spawn(async move {
                if let Err(e) = scanner.start().await {
                    error!("❌ Scanner service failed: {}", e);
                }
            })
        };
        handles.push(scanner_handle);
        info!("🔍 Scanner service started");
    }
    
    // 启动 WebSocket 服务器
    if config.websocket.enabled {
        let websocket_handle = {
            let websocket = websocket_service.clone();
            let port = config.websocket.port;
            tokio::spawn(async move {
                if let Err(e) = websocket.start_server(port).await {
                    error!("❌ WebSocket server failed: {}", e);
                }
            })
        };
        handles.push(websocket_handle);
        info!("🔌 WebSocket server started on port {}", config.websocket.port);
    }
    
    // 启动 Webhook 投递服务
    if config.webhook.enabled {
        let webhook_handle = {
            let webhook = webhook_service.clone();
            tokio::spawn(async move {
                if let Err(e) = webhook.start_delivery_worker().await {
                    error!("❌ Webhook delivery service failed: {}", e);
                }
            })
        };
        handles.push(webhook_handle);
        info!("🪝 Webhook delivery service started");
    }
    
    // 启动 API 服务器
    let api_handle = {
        let state = app_state.clone();
        let host = config.api.host.clone();
        let port = config.api.port;
        tokio::spawn(async move {
            if let Err(e) = start_api_server(state, &host, port).await {
                error!("❌ API server failed: {}", e);
            }
        })
    };
    handles.push(api_handle);
    info!("🌐 API server started on {}:{}", config.api.host, config.api.port);
    
    info!("🎉 TRX Tracker Unified started successfully!");
    info!("📊 API: http://{}:{}", config.api.host, config.api.port);
    info!("📡 WebSocket: ws://{}:{}", config.api.host, config.websocket.port);
    
    // 等待关闭信号
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("🛑 Received Ctrl+C, shutting down...");
        }
    }
    
    info!("🔄 Stopping all services...");
    
    // 停止扫描器
    if let Err(e) = scanner_service.stop().await {
        warn!("⚠️ Failed to stop scanner service: {}", e);
    }
    
    // 停止 WebSocket 服务
    if let Err(e) = websocket_service.stop().await {
        warn!("⚠️ Failed to stop WebSocket service: {}", e);
    }
    
    // 停止 Webhook 服务
    if let Err(e) = webhook_service.stop().await {
        warn!("⚠️ Failed to stop Webhook service: {}", e);
    }
    
    // 等待所有任务完成
    for handle in handles {
        if let Err(e) = handle.await {
            warn!("⚠️ Task failed to complete: {}", e);
        }
    }
    
    info!("✅ TRX Tracker Unified stopped successfully");
    
    Ok(())
}

/// 启动 API 服务器
async fn start_api_server(
    state: Arc<AdminAppState>,
    host: &str,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    use axum::{
        routing::{get, post, put, delete},
        Router,
    };
    use tower::ServiceBuilder;
    use tower_http::cors::{CorsLayer, Any};
    use std::time::Duration;
    
    // 创建路由
    let app = Router::new()
        // 健康检查
        .route("/health", get(crate::api::handlers::admin::health_check))
        
        // 仪表板 API
        .route("/api/admin/dashboard/stats", get(crate::api::handlers::admin::get_dashboard_stats))
        
        // 系统配置 API
        .route("/api/admin/config", get(crate::api::handlers::admin::get_system_config))
        .route("/api/admin/config", put(crate::api::handlers::admin::update_system_config))
        
        // 日志管理 API
        .route("/api/admin/logs", get(crate::api::handlers::admin::get_logs))
        .route("/api/admin/logs", delete(crate::api::handlers::admin::clear_logs))
        .route("/api/admin/logs/export", get(crate::api::handlers::admin::export_logs))
        
        // 扫描器控制 API
        .route("/api/admin/scanner/restart", post(crate::api::handlers::admin::restart_scanner))
        .route("/api/admin/scanner/stop", post(crate::api::handlers::admin::stop_scanner))
        .route("/api/admin/scanner/scan/:block_number", post(crate::api::handlers::admin::scan_block))
        
        // 缓存管理 API
        .route("/api/admin/cache/stats", get(crate::api::handlers::admin::get_cache_stats))
        .route("/api/admin/cache", delete(crate::api::handlers::admin::clear_cache))
        
        // 交易查询 API
        .route("/api/v1/transactions", get(crate::api::handlers::transaction::get_transactions))
        .route("/api/v1/transactions/multi-address", post(crate::api::handlers::transaction::get_multi_address_transactions))
        .route("/api/v1/transactions/:hash", get(crate::api::handlers::transaction::get_transaction))
        
        // 地址查询 API
        .route("/api/v1/addresses/:address", get(crate::api::handlers::transaction::get_address_info))
        .route("/api/v1/addresses/:address/transactions", get(crate::api::handlers::transaction::get_address_transactions))
        
        // 添加中间件
        .layer(
            ServiceBuilder::new()
                .timeout(Duration::from_secs(30))
                .layer(CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any))
                .into_inner()
        )
        .with_state(state);
    
    // 启动服务器
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port)).await?;
    info!("🌐 API server listening on {}:{}", host, port);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

