use axum::{
    // Removed unused import: State
    http::StatusCode,
    response::{Json, IntoResponse},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
// Removed unused import: HashMap
use std::sync::Arc;
use tower_http::cors::CorsLayer;

// Removed unused imports: Config, models::*, Database

pub mod handlers;

use handlers::{health, transaction, dashboard};

// Use AdminAppState as the unified state type
pub use crate::api::handlers::admin::AdminAppState as AppState;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health::health_check))
        
        // Transaction endpoints
        .route("/api/v1/transactions", get(transaction::get_transactions))
        .route("/api/v1/transactions/:hash", get(transaction::get_transaction))
        // .route("/api/v1/transactions/multi-address", post(transaction::multi_address_query)) // 暂时注释掉缺失的函数
        
        // Address endpoints - 暂时注释掉缺失的模块
        // .route("/api/v1/addresses/:address", get(address::get_address_info))
        // .route("/api/v1/addresses/:address/transactions", get(address::get_address_transactions))
        
        // Webhook endpoints - 暂时注释掉缺失的模块
        // .route("/api/v1/webhooks", get(webhook::list_webhooks))
        // .route("/api/v1/webhooks", post(webhook::create_webhook))
        // .route("/api/v1/webhooks/:id", get(webhook::get_webhook))
        // .route("/api/v1/webhooks/:id", put(webhook::update_webhook))
        // .route("/api/v1/webhooks/:id", delete(webhook::delete_webhook))
        
        // WebSocket management endpoints - 暂时注释掉缺失的模块
        // .route("/api/v1/websockets/connections", get(websocket::list_connections))
        // .route("/api/v1/websockets/stats", get(websocket::get_stats))
        
        // API Key management endpoints - 暂时注释掉缺失的模块
        // .route("/api/v1/api-keys", get(api_key::list_api_keys))
        // .route("/api/v1/api-keys", post(api_key::create_api_key))
        // .route("/api/v1/api-keys/:id", delete(api_key::delete_api_key))
        
        // System configuration endpoints - 暂时注释掉缺失的模块
        // .route("/api/v1/config", get(config::get_config))
        // .route("/api/v1/config", put(config::update_config))
        
        // Logs endpoints - 暂时注释掉缺失的模块
        // .route("/api/v1/logs", get(logs::get_logs))
        
        // Dashboard stats
        .route("/api/v1/dashboard/stats", get(dashboard::get_stats))
        
        // WebSocket upgrade endpoint - 暂时注释掉缺失的函数
        // .route("/ws", get(websocket::websocket_handler))
        
        .layer(CorsLayer::permissive())
        .with_state(state)
}

// Common query parameters
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

impl Default for PaginationQuery {
    fn default() -> Self {
        Self {
            page: Some(1),
            limit: Some(20),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TransactionQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub status: Option<String>,
    pub token: Option<String>,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub min_amount: Option<String>,
    pub max_amount: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MultiAddressQuery {
    pub addresses: Vec<String>,
    #[serde(flatten)]
    pub pagination: PaginationQuery,
    pub status: Option<String>,
    pub token: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

// Common response types
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub pagination: Option<PaginationInfo>,
}

#[derive(Debug, Serialize)]
pub struct PaginationInfo {
    pub page: u32,
    pub limit: u32,
    pub total: u64,
    pub total_pages: u32,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            pagination: None,
        }
    }
    
    pub fn success_with_pagination(data: T, pagination: PaginationInfo) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            pagination: Some(pagination),
        }
    }
    
    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.to_string()),
            pagination: None,
        }
    }
}

// Error handling
pub type ApiResult<T> = Result<Json<ApiResponse<T>>, ApiError>;

#[derive(Debug)]
pub enum ApiError {
    DatabaseError(String),
    ValidationError(String),
    NotFound(String),
    InternalError(String),
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            ApiError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };
        
        (status, Json(ApiResponse::<()>::error(&message))).into_response()
    }
}

