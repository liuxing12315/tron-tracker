use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Unified transaction model used across all modules
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Transaction {
    pub id: Uuid,
    pub hash: String,
    pub block_number: u64,
    pub block_hash: String,
    pub transaction_index: u32,
    pub from_address: String,
    pub to_address: String,
    pub value: String,
    pub token_address: Option<String>,
    pub token_symbol: Option<String>,
    pub token_decimals: Option<u32>,
    pub gas_used: Option<u64>,
    pub gas_price: Option<String>,
    pub status: TransactionStatus,
    pub timestamp: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
// #[sqlx(type_name = "transaction_status", rename_all = "lowercase")] // 暂时注释掉sqlx属性
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
// #[sqlx(type_name = "notification_event_type", rename_all = "lowercase")] // 暂时注释掉sqlx属性
pub enum NotificationEventType {
    Transaction,
    LargeTransfer,
    NewAddress,
    SystemAlert,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
// #[sqlx(type_name = "notification_status", rename_all = "lowercase")] // 暂时注释掉sqlx属性
pub enum NotificationStatus {
    Pending,
    Sent,
    Failed,
    Cancelled,
}

/// Webhook filters for event filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookFilters {
    pub addresses: Option<Vec<String>>,
    pub tokens: Option<Vec<String>>,
    pub min_amount: Option<String>,
    pub max_amount: Option<String>,
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
    pub filters: WebhookFilters,
    pub retry_count: u32,
    pub timeout: u32,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketSubscription {
    pub id: String,
    pub events: Vec<String>,
    pub filters: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
// #[sqlx(type_name = "permission", rename_all = "lowercase")] // 暂时注释掉sqlx属性
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
    pub number: u64,
    pub hash: String,
    pub parent_hash: String,
    pub timestamp: DateTime<Utc>,
    pub transaction_count: u32,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

/// Pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

/// Transaction query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionQuery {
    pub address: Option<String>,
    pub hash: Option<String>,
    pub block_number: Option<i64>,
    pub token_address: Option<String>,
    pub token: Option<String>,
    pub status: Option<TransactionStatus>,
    pub min_amount: Option<String>,
    pub max_amount: Option<String>,
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

/// Multi-address query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiAddressQueryResult {
    pub transactions: Vec<Transaction>,
    pub total_count: u64,
    pub page: u32,
    pub limit: u32,
    pub has_more: bool,
    pub address_stats: std::collections::HashMap<String, AddressStatistics>,
}

/// Address statistics (unified definition)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressStatistics {
    pub address: String,
    pub total_transactions: u64,
    pub successful_transactions: u64,
    pub sent_transactions: u64,
    pub received_transactions: u64,
    pub total_trx_received: String,
    pub total_usdt_received: String,
    pub first_transaction: Option<DateTime<Utc>>,
    pub last_transaction: Option<DateTime<Utc>>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_transaction_status_serialization() {
        let pending = TransactionStatus::Pending;
        let success = TransactionStatus::Success;
        let failed = TransactionStatus::Failed;

        assert_eq!(serde_json::to_string(&pending).unwrap(), "\"Pending\"");
        assert_eq!(serde_json::to_string(&success).unwrap(), "\"Success\"");
        assert_eq!(serde_json::to_string(&failed).unwrap(), "\"Failed\"");
    }

    #[test]
    fn test_transaction_status_deserialization() {
        let pending: TransactionStatus = serde_json::from_str("\"Pending\"").unwrap();
        let success: TransactionStatus = serde_json::from_str("\"Success\"").unwrap();
        let failed: TransactionStatus = serde_json::from_str("\"Failed\"").unwrap();

        assert_eq!(pending, TransactionStatus::Pending);
        assert_eq!(success, TransactionStatus::Success);
        assert_eq!(failed, TransactionStatus::Failed);
    }

    #[test]
    fn test_notification_event_type_serialization() {
        let transaction = NotificationEventType::Transaction;
        let large_transfer = NotificationEventType::LargeTransfer;
        let new_address = NotificationEventType::NewAddress;
        let system_alert = NotificationEventType::SystemAlert;

        assert_eq!(serde_json::to_string(&transaction).unwrap(), "\"Transaction\"");
        assert_eq!(serde_json::to_string(&large_transfer).unwrap(), "\"LargeTransfer\"");
        assert_eq!(serde_json::to_string(&new_address).unwrap(), "\"NewAddress\"");
        assert_eq!(serde_json::to_string(&system_alert).unwrap(), "\"SystemAlert\"");
    }

    #[test]
    fn test_permission_serialization() {
        let read_tx = Permission::ReadTransactions;
        let admin = Permission::Admin;

        assert_eq!(serde_json::to_string(&read_tx).unwrap(), "\"ReadTransactions\"");
        assert_eq!(serde_json::to_string(&admin).unwrap(), "\"Admin\"");
    }

    #[test]
    fn test_sort_order_serialization() {
        let asc = SortOrder::Asc;
        let desc = SortOrder::Desc;

        assert_eq!(serde_json::to_string(&asc).unwrap(), "\"asc\"");
        assert_eq!(serde_json::to_string(&desc).unwrap(), "\"desc\"");
    }

    #[test]
    fn test_api_response_new() {
        let data = vec!["test".to_string()];
        let response = ApiResponse::new(data.clone());

        assert_eq!(response.data, data);
        assert!(response.meta.total.is_none());
        assert!(response.meta.limit.is_none());
        assert!(response.meta.offset.is_none());
        assert!(response.meta.has_more.is_none());
    }

    #[test]
    fn test_api_response_with_meta() {
        let data = vec!["test".to_string()];
        let response = ApiResponse::with_meta(data.clone(), 100, 20, 0);

        assert_eq!(response.data, data);
        assert_eq!(response.meta.total, Some(100));
        assert_eq!(response.meta.limit, Some(20));
        assert_eq!(response.meta.offset, Some(0));
        assert_eq!(response.meta.has_more, Some(true));
    }

    #[test]
    fn test_api_response_has_more_calculation() {
        let data = vec!["test".to_string()];
        
        // Test case where there are more items
        let response1 = ApiResponse::with_meta(data.clone(), 100, 20, 0);
        assert_eq!(response1.meta.has_more, Some(true));

        // Test case where we're at the end
        let response2 = ApiResponse::with_meta(data.clone(), 20, 20, 0);
        assert_eq!(response2.meta.has_more, Some(false));

        // Test case where we've reached exactly the end
        let response3 = ApiResponse::with_meta(data.clone(), 40, 20, 20);
        assert_eq!(response3.meta.has_more, Some(false));
    }

    #[test]
    fn test_query_params_default() {
        let params = QueryParams::default();
        
        assert_eq!(params.limit, Some(20));
        assert_eq!(params.offset, Some(0));
        assert_eq!(params.sort_by, Some("created_at".to_string()));
        assert_eq!(params.sort_order, Some(SortOrder::Desc));
        assert!(params.filters.is_none());
    }

    #[test]
    fn test_webhook_filters_serialization() {
        let filters = WebhookFilters {
            addresses: Some(vec!["TRX123".to_string(), "TRX456".to_string()]),
            tokens: Some(vec!["USDT".to_string()]),
            min_amount: Some("1000".to_string()),
            max_amount: Some("10000".to_string()),
        };

        let json = serde_json::to_string(&filters).unwrap();
        let deserialized: WebhookFilters = serde_json::from_str(&json).unwrap();

        assert_eq!(filters.addresses, deserialized.addresses);
        assert_eq!(filters.tokens, deserialized.tokens);
        assert_eq!(filters.min_amount, deserialized.min_amount);
        assert_eq!(filters.max_amount, deserialized.max_amount);
    }

    #[test]
    fn test_multi_address_query_result_serialization() {
        let result = MultiAddressQueryResult {
            transactions: Vec::new(),
            total_count: 100,
            page: 1,
            limit: 20,
            has_more: true,
            address_stats: std::collections::HashMap::new(),
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: MultiAddressQueryResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result.total_count, deserialized.total_count);
        assert_eq!(result.page, deserialized.page);
        assert_eq!(result.limit, deserialized.limit);
        assert_eq!(result.has_more, deserialized.has_more);
    }

    #[test]
    fn test_websocket_message_serialization() {
        let subscribe_msg = WebSocketMessage::Subscribe {
            events: vec!["transaction".to_string()],
            filters: None,
        };

        let json = serde_json::to_string(&subscribe_msg).unwrap();
        let deserialized: WebSocketMessage = serde_json::from_str(&json).unwrap();

        if let WebSocketMessage::Subscribe { events, filters } = deserialized {
            assert_eq!(events, vec!["transaction".to_string()]);
            assert!(filters.is_none());
        } else {
            panic!("Wrong message type deserialized");
        }
    }

    #[test]
    fn test_address_statistics_creation() {
        let stats = AddressStatistics {
            address: "TRX123456".to_string(),
            total_transactions: 100,
            successful_transactions: 95,
            sent_transactions: 50,
            received_transactions: 50,
            total_trx_received: "1000000".to_string(),
            total_usdt_received: "500000".to_string(),
            first_transaction: Some(Utc::now()),
            last_transaction: Some(Utc::now()),
        };

        assert_eq!(stats.address, "TRX123456");
        assert_eq!(stats.total_transactions, 100);
        assert_eq!(stats.successful_transactions, 95);
    }
}

