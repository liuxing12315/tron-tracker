use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Unified transaction model used across all modules
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Transaction {
    pub id: Uuid,
    pub hash: String,
    pub block_number: i64,
    pub block_hash: String,
    pub transaction_index: i32,
    pub from_address: String,
    pub to_address: String,
    pub value: String,
    pub token_address: Option<String>,
    pub token_symbol: Option<String>,
    pub token_decimals: Option<i32>,
    pub gas_used: Option<i64>,
    pub gas_price: Option<String>,
    pub status: TransactionStatus,
    pub timestamp: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "transaction_status", rename_all = "lowercase")]
pub enum TransactionStatus {
    Pending,
    Success,
    Failed,
}

/// Unified address model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Address {
    pub id: Uuid,
    pub address: String,
    pub label: Option<String>,
    pub balance_trx: String,
    pub balance_usdt: String,
    pub transaction_count: i64,
    pub first_seen: Option<DateTime<Utc>>,
    pub last_seen: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Unified notification model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Notification {
    pub id: Uuid,
    pub event_type: NotificationEventType,
    pub target_address: String,
    pub transaction_id: Option<Uuid>,
    pub webhook_id: Option<Uuid>,
    pub payload: serde_json::Value,
    pub status: NotificationStatus,
    pub attempts: i32,
    pub last_attempt: Option<DateTime<Utc>>,
    pub next_attempt: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "notification_event_type", rename_all = "lowercase")]
pub enum NotificationEventType {
    Transaction,
    LargeTransfer,
    NewAddress,
    SystemAlert,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "notification_status", rename_all = "lowercase")]
pub enum NotificationStatus {
    Pending,
    Sent,
    Failed,
    Cancelled,
}

/// Webhook configuration model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Webhook {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    pub secret: String,
    pub enabled: bool,
    pub events: Vec<NotificationEventType>,
    pub filters: serde_json::Value,
    pub success_count: i64,
    pub failure_count: i64,
    pub last_triggered: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// WebSocket connection model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConnection {
    pub id: Uuid,
    pub client_ip: String,
    pub user_agent: Option<String>,
    pub subscriptions: Vec<String>,
    pub connected_at: DateTime<Utc>,
    pub last_ping: DateTime<Utc>,
    pub messages_sent: i64,
    pub messages_received: i64,
}

/// API key model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApiKey {
    pub id: Uuid,
    pub name: String,
    pub key_hash: String,
    pub permissions: Vec<Permission>,
    pub enabled: bool,
    pub rate_limit: Option<i32>,
    pub request_count: i64,
    pub last_used: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "permission", rename_all = "lowercase")]
pub enum Permission {
    ReadTransactions,
    ReadAddresses,
    ReadBlocks,
    ManageWebhooks,
    ManageApiKeys,
    ManageSystem,
    Admin,
}

/// System configuration model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SystemConfig {
    pub id: Uuid,
    pub key: String,
    pub value: serde_json::Value,
    pub description: Option<String>,
    pub updated_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Block model for tracking sync progress
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Block {
    pub id: Uuid,
    pub number: i64,
    pub hash: String,
    pub parent_hash: String,
    pub timestamp: DateTime<Utc>,
    pub transaction_count: i32,
    pub processed: bool,
    pub created_at: DateTime<Utc>,
}

/// Block data from blockchain API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockData {
    pub number: u64,
    pub hash: String,
    pub parent_hash: String,
    pub timestamp: u64,
    pub transaction_count: u32,
    pub transactions: Vec<Transaction>,
}

/// Unified query parameters for pagination and filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryParams {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub sort_by: Option<String>,
    pub sort_order: Option<SortOrder>,
    pub filters: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    Desc,
}

/// Unified response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub data: T,
    pub meta: ResponseMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMeta {
    pub total: Option<i64>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub has_more: Option<bool>,
}

/// Multi-address query request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiAddressQuery {
    pub addresses: Vec<String>,
    pub include_tokens: Option<bool>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// Transaction query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionQuery {
    pub address: Option<String>,
    pub hash: Option<String>,
    pub block_number: Option<i64>,
    pub token_address: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub sort_by: Option<String>,
    pub sort_order: Option<SortOrder>,
}

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WebSocketMessage {
    Subscribe { events: Vec<String>, filters: Option<serde_json::Value> },
    Unsubscribe { events: Vec<String> },
    Ping,
    Pong,
    Notification(Notification),
    Error { code: String, message: String },
}

/// System statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    pub total_transactions: i64,
    pub total_addresses: i64,
    pub current_block: i64,
    pub scan_speed: f64,
    pub active_webhooks: i64,
    pub websocket_connections: i64,
    pub api_requests_today: i64,
    pub success_rate: f64,
    pub uptime: i64,
}

impl Default for QueryParams {
    fn default() -> Self {
        Self {
            limit: Some(20),
            offset: Some(0),
            sort_by: Some("created_at".to_string()),
            sort_order: Some(SortOrder::Desc),
            filters: None,
        }
    }
}

impl<T> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            meta: ResponseMeta {
                total: None,
                limit: None,
                offset: None,
                has_more: None,
            },
        }
    }
    
    pub fn with_meta(data: T, total: i64, limit: u32, offset: u32) -> Self {
        Self {
            data,
            meta: ResponseMeta {
                total: Some(total),
                limit: Some(limit),
                offset: Some(offset),
                has_more: Some(total > (offset + limit) as i64),
            },
        }
    }
}

