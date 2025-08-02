use anyhow::Result;
use sqlx::PgPool;
use std::time::Duration;
use tracing::{debug, info, error, warn};
use uuid::Uuid;

use crate::core::{
    config::DatabaseConfig,
    models::*,
};
// Remove custom Error alias and use anyhow::Error directly

/// 数据库连接池
#[derive(Debug, Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// 创建新的数据库连接
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        info!("Connecting to database: {}", &config.url[..config.url.find('@').unwrap_or(20)]);

        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(Duration::from_secs(config.acquire_timeout))
            .connect(&config.url)
            .await?;

        info!("Database connected successfully");
        Ok(Self { pool })
    }

    /// 获取数据库连接池
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// 运行数据库迁移
    pub async fn migrate(&self) -> Result<()> {
        info!("Database migration skipped - assuming database is already set up");
        Ok(())
    }

    // ==================== 交易相关操作 ====================

    /// 保存交易记录
    pub async fn save_transaction(&self, transaction: &Transaction) -> Result<()> {
        debug!("Saving transaction: {}", transaction.hash);
        // TODO: 实现实际的SQL插入逻辑
        Ok(())
    }

    /// 根据哈希获取交易
    pub async fn get_transaction_by_hash(&self, hash: &str) -> Result<Option<Transaction>> {
        debug!("Querying transaction by hash: {}", hash);
        // TODO: 实现实际的SQL查询逻辑
        Ok(None)
    }

    /// 查询交易列表
    pub async fn list_transactions(&self, query: &TransactionQuery) -> Result<(Vec<Transaction>, u64)> {
        debug!("Listing transactions with query: {:?}", query);
        // TODO: 实现实际的SQL查询逻辑
        Ok((Vec::new(), 0))
    }

    /// 获取交易列表（别名方法）
    pub async fn get_transactions(&self, filters: &TransactionQuery, _pagination: &Pagination) -> Result<(Vec<Transaction>, u64)> {
        debug!("Getting transactions with filters: {:?}", filters);
        self.list_transactions(filters).await
    }

    /// 多地址批量查询
    pub async fn list_transactions_by_addresses(&self, addresses: &[String], _query: &MultiAddressQuery) -> Result<(Vec<Transaction>, u64)> {
        debug!("Querying transactions for {} addresses", addresses.len());
        // TODO: 实现多地址查询逻辑
        Ok((Vec::new(), 0))
    }

    /// 多地址交易查询（返回MultiAddressQueryResult）
    pub async fn get_multi_address_transactions(
        &self, 
        addresses: &[String], 
        _filters: &TransactionQuery, 
        pagination: &Pagination,
        _group_by_address: bool
    ) -> Result<MultiAddressQueryResult> {
        debug!("Getting multi-address transactions for {} addresses", addresses.len());
        
        let page = pagination.page.unwrap_or(1);
        let limit = pagination.limit.unwrap_or(20);
        
        // TODO: 实现实际的多地址查询逻辑
        Ok(MultiAddressQueryResult {
            transactions: Vec::new(),
            total_count: 0,
            page,
            limit,
            has_more: false,
            address_stats: std::collections::HashMap::new(),
        })
    }

    /// 获取地址交易列表
    pub async fn get_address_transactions(
        &self,
        address: &str,
        filters: &TransactionQuery,
        _pagination: &Pagination,
    ) -> Result<(Vec<Transaction>, u64)> {
        debug!("Getting transactions for address: {}", address);
        // TODO: 实现地址交易查询
        let _ = filters; // 避免未使用警告
        Ok((Vec::new(), 0))
    }

    /// 获取地址交易数量
    pub async fn get_address_transaction_count(
        &self,
        address: &str,
        _filters: &TransactionQuery,
    ) -> Result<u64> {
        debug!("Getting transaction count for address: {}", address);
        // TODO: 实现地址交易计数
        Ok(0)
    }

    /// 搜索交易
    pub async fn search_transactions(
        &self,
        query: &str,
        _pagination: &Pagination,
    ) -> Result<(Vec<Transaction>, u64)> {
        debug!("Searching transactions with query: {}", query);
        // TODO: 实现交易搜索逻辑
        Ok((Vec::new(), 0))
    }

    /// 批量获取地址统计
    pub async fn get_batch_address_statistics(
        &self,
        addresses: &[String],
    ) -> Result<std::collections::HashMap<String, AddressStatistics>> {
        debug!("Getting batch address statistics for {} addresses", addresses.len());
        
        let mut stats = std::collections::HashMap::new();
        for address in addresses {
            let address_stats = self.get_address_statistics(address).await?;
            stats.insert(address.clone(), address_stats);
        }
        
        Ok(stats)
    }

    /// 获取地址统计信息
    pub async fn get_address_statistics(&self, address: &str) -> Result<AddressStatistics> {
        debug!("Getting statistics for address: {}", address);
        
        // TODO: 实现实际的地址统计查询
        Ok(AddressStatistics {
            address: address.to_string(),
            total_transactions: 0,
            successful_transactions: 0,
            sent_transactions: 0,
            received_transactions: 0,
            total_trx_received: "0".to_string(),
            total_usdt_received: "0".to_string(),
            first_transaction: None,
            last_transaction: None,
        })
    }

    /// 获取所有API密钥
    pub async fn get_all_api_keys(&self) -> Result<Vec<ApiKey>> {
        debug!("Getting all API keys");
        
        let query = r#"
            SELECT id, name, key_hash, permissions, enabled,
                   rate_limit, request_count, last_used,
                   expires_at, created_at, updated_at
            FROM api_keys
            ORDER BY created_at DESC
        "#;
        
        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;
        
        let mut api_keys = Vec::new();
        for row in rows {
            use sqlx::Row;
            let permissions_str: Vec<String> = row.get("permissions");
            let permissions = permissions_str.into_iter()
                .filter_map(|s| match s.as_str() {
                    "ReadTransactions" => Some(Permission::ReadTransactions),
                    "ReadAddresses" => Some(Permission::ReadAddresses),
                    "ReadBlocks" => Some(Permission::ReadBlocks),
                    "ManageWebhooks" => Some(Permission::ManageWebhooks),
                    "ManageApiKeys" => Some(Permission::ManageApiKeys),
                    "ManageSystem" => Some(Permission::ManageSystem),
                    _ => None,
                })
                .collect();
            
            api_keys.push(ApiKey {
                id: row.get("id"),
                name: row.get("name"),
                key_hash: row.get("key_hash"),
                permissions,
                enabled: row.get("enabled"),
                rate_limit: row.get("rate_limit"),
                request_count: row.get("request_count"),
                last_used: row.get("last_used"),
                expires_at: row.get("expires_at"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }
        
        debug!("Found {} API keys", api_keys.len());
        Ok(api_keys)
    }

    /// 根据ID获取API密钥
    pub async fn get_api_key_by_id(&self, key_id: &str) -> Result<Option<ApiKey>> {
        debug!("Getting API key by ID: {}", key_id);
        
        let uuid = Uuid::parse_str(key_id)
            .map_err(|e| anyhow::anyhow!("Invalid UUID: {}", e))?;
        
        let query = r#"
            SELECT id, name, key_hash, permissions, enabled,
                   rate_limit, request_count, last_used,
                   expires_at, created_at, updated_at
            FROM api_keys
            WHERE id = $1
        "#;
        
        let row = sqlx::query(query)
            .bind(&uuid)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;
        
        if let Some(row) = row {
            use sqlx::Row;
            let permissions_str: Vec<String> = row.get("permissions");
            let permissions = permissions_str.into_iter()
                .filter_map(|s| match s.as_str() {
                    "ReadTransactions" => Some(Permission::ReadTransactions),
                    "ReadAddresses" => Some(Permission::ReadAddresses),
                    "ReadBlocks" => Some(Permission::ReadBlocks),
                    "ManageWebhooks" => Some(Permission::ManageWebhooks),
                    "ManageApiKeys" => Some(Permission::ManageApiKeys),
                    "ManageSystem" => Some(Permission::ManageSystem),
                    _ => None,
                })
                .collect();
            
            Ok(Some(ApiKey {
                id: row.get("id"),
                name: row.get("name"),
                key_hash: row.get("key_hash"),
                permissions,
                enabled: row.get("enabled"),
                rate_limit: row.get("rate_limit"),
                request_count: row.get("request_count"),
                last_used: row.get("last_used"),
                expires_at: row.get("expires_at"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    /// 更新API密钥
    pub async fn update_api_key(&self, api_key: &ApiKey) -> Result<()> {
        debug!("Updating API key: {}", api_key.name);
        
        let query = r#"
            UPDATE api_keys
            SET name = $2, key_hash = $3, permissions = $4,
                enabled = $5, rate_limit = $6, request_count = $7,
                last_used = $8, expires_at = $9, updated_at = $10
            WHERE id = $1
        "#;
        
        // 将权限枚举转换为字符串数组
        let permissions: Vec<String> = api_key.permissions.iter()
            .map(|p| format!("{:?}", p))
            .collect();
        
        let result = sqlx::query(query)
            .bind(&api_key.id)
            .bind(&api_key.name)
            .bind(&api_key.key_hash)
            .bind(&permissions)
            .bind(api_key.enabled)
            .bind(api_key.rate_limit)
            .bind(api_key.request_count)
            .bind(api_key.last_used)
            .bind(api_key.expires_at)
            .bind(api_key.updated_at)
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;
        
        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("API key not found"));
        }
        
        info!("API key updated successfully: {} ({})", api_key.name, api_key.id);
        Ok(())
    }

    /// 删除API密钥
    pub async fn delete_api_key(&self, key_id: &str) -> Result<()> {
        debug!("Deleting API key: {}", key_id);
        
        let uuid = Uuid::parse_str(key_id)
            .map_err(|e| anyhow::anyhow!("Invalid UUID: {}", e))?;
        
        let query = "DELETE FROM api_keys WHERE id = $1";
        
        let result = sqlx::query(query)
            .bind(&uuid)
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;
        
        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("API key not found"));
        }
        
        info!("API key deleted successfully: {}", key_id);
        Ok(())
    }

    /// 获取API密钥使用统计
    pub async fn get_api_key_usage_stats(&self, key_id: &str) -> Result<serde_json::Value> {
        debug!("Getting API key usage stats: {}", key_id);
        
        let uuid = Uuid::parse_str(key_id)
            .map_err(|e| anyhow::anyhow!("Invalid UUID: {}", e))?;
        
        // 获取基本统计
        let basic_query = r#"
            SELECT request_count, last_used
            FROM api_keys
            WHERE id = $1
        "#;
        
        let basic_row = sqlx::query(basic_query)
            .bind(&uuid)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;
        
        if let Some(row) = basic_row {
            use sqlx::Row;
            let total_requests: i64 = row.get("request_count");
            let last_used: Option<chrono::DateTime<chrono::Utc>> = row.get("last_used");
            
            // 尝试获取今日请求数（如果有日志表）
            let today_query = r#"
                SELECT COUNT(*) as count
                FROM api_key_usage
                WHERE key_id = $1
                AND DATE(created_at) = CURRENT_DATE
            "#;
            
            let requests_today = sqlx::query(today_query)
                .bind(&uuid)
                .fetch_optional(&self.pool)
                .await
                .ok()
                .and_then(|row| row.map(|r| r.get::<i64, _>("count")))
                .unwrap_or(0);
            
            Ok(serde_json::json!({
                "total_requests": total_requests,
                "requests_today": requests_today,
                "last_used": last_used,
                "key_id": key_id
            }))
        } else {
            Err(anyhow::anyhow!("API key not found"))
        }
    }

    /// 更新API密钥使用情况
    pub async fn update_api_key_usage(&self, key_id: &str) -> Result<()> {
        debug!("Updating API key usage: {}", key_id);
        
        let uuid = Uuid::parse_str(key_id)
            .map_err(|e| anyhow::anyhow!("Invalid UUID: {}", e))?;
        
        let query = r#"
            UPDATE api_keys
            SET request_count = request_count + 1,
                last_used = NOW()
            WHERE id = $1
        "#;
        
        sqlx::query(query)
            .bind(&uuid)
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;
        
        Ok(())
    }

    /// 列出Webhook
    pub async fn list_webhooks(&self, include_disabled: bool) -> Result<Vec<Webhook>> {
        debug!("Listing webhooks, include_disabled: {}", include_disabled);
        
        let query = if include_disabled {
            "SELECT id, name, url, secret, enabled, events, filters, success_count, failure_count, last_triggered, created_at, updated_at FROM webhooks ORDER BY created_at DESC"
        } else {
            "SELECT id, name, url, secret, enabled, events, filters, success_count, failure_count, last_triggered, created_at, updated_at FROM webhooks WHERE enabled = true ORDER BY created_at DESC"
        };
        
        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;
        
        let mut webhooks = Vec::new();
        for row in rows {
            use sqlx::Row;
            let events_str: Vec<String> = row.get("events");
            let events = events_str.into_iter()
                .filter_map(|s| match s.as_str() {
                    "transaction" => Some(NotificationEventType::Transaction),
                    "new_address" => Some(NotificationEventType::NewAddress),
                    "system_alert" => Some(NotificationEventType::SystemAlert),
                    _ => None,
                })
                .collect();
            
            let filters_json: serde_json::Value = row.get("filters");
            let filters = serde_json::from_value(filters_json)
                .unwrap_or_default();
            
            webhooks.push(Webhook {
                id: row.get("id"),
                name: row.get("name"),
                url: row.get("url"),
                secret: row.get("secret"),
                enabled: row.get("enabled"),
                events,
                filters,
                retry_count: 0, // Default value since not in database
                timeout: 30000, // Default value since not in database
                success_count: row.get("success_count"),
                failure_count: row.get("failure_count"),
                last_triggered: row.get("last_triggered"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }
        
        debug!("Found {} webhooks", webhooks.len());
        Ok(webhooks)
    }

    // ==================== 区块相关操作 ====================

    /// 保存区块信息
    pub async fn save_block(&self, block: &BlockData) -> Result<()> {
        debug!("Saving block: {}", block.number);
        // TODO: 实现区块保存逻辑
        Ok(())
    }

    /// 获取最后处理的区块号
    pub async fn get_last_processed_block(&self) -> Result<Option<u64>> {
        debug!("Getting last processed block");
        // TODO: 实现最后处理区块查询
        Ok(Some(62800000)) // 默认起始区块
    }

    /// 保存扫描进度
    pub async fn save_scan_progress(&self, block_number: u64) -> Result<()> {
        debug!("Saving scan progress: {}", block_number);
        // TODO: 实现扫描进度保存
        Ok(())
    }

    // ==================== Webhook 相关操作 ====================

    /// 获取所有启用的 Webhook
    pub async fn get_enabled_webhooks(&self) -> Result<Vec<Webhook>> {
        debug!("Getting enabled webhooks");
        // TODO: 实现启用Webhook查询
        Ok(Vec::new())
    }

    /// 保存 Webhook
    pub async fn save_webhook(&self, webhook: &Webhook) -> Result<()> {
        debug!("Saving webhook: {}", webhook.name);
        
        let query = r#"
            INSERT INTO webhooks (
                id, name, url, secret, enabled, events, filters,
                success_count, failure_count, last_triggered,
                created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#;
        
        // 将事件枚举转换为字符串数组
        let events: Vec<String> = webhook.events.iter()
            .map(|e| format!("{:?}", e).to_lowercase())
            .collect();
        
        // 将filters转换为JSON
        let filters_json = serde_json::to_value(&webhook.filters)
            .map_err(|e| anyhow::anyhow!("Invalid filters: {}", e))?;
        
        sqlx::query(query)
            .bind(&webhook.id)
            .bind(&webhook.name)
            .bind(&webhook.url)
            .bind(&webhook.secret)
            .bind(webhook.enabled)
            .bind(&events)
            .bind(&filters_json)
            .bind(webhook.success_count)
            .bind(webhook.failure_count)
            .bind(webhook.last_triggered)
            .bind(webhook.created_at)
            .bind(webhook.updated_at)
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;
        
        info!("Webhook saved successfully: {} ({})", webhook.name, webhook.id);
        Ok(())
    }

    /// 获取 Webhook
    pub async fn get_webhook(&self, id: &str) -> Result<Option<Webhook>> {
        debug!("Getting webhook: {}", id);
        
        let uuid = Uuid::parse_str(id)
            .map_err(|e| anyhow::anyhow!("Invalid UUID: {}", e))?;
        
        let query = "SELECT id, name, url, secret, enabled, events, filters, success_count, failure_count, last_triggered, created_at, updated_at FROM webhooks WHERE id = $1";
        
        let row = sqlx::query(query)
            .bind(&uuid)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;
        
        if let Some(row) = row {
            use sqlx::Row;
            let events_str: Vec<String> = row.get("events");
            let events = events_str.into_iter()
                .filter_map(|s| match s.as_str() {
                    "transaction" => Some(NotificationEventType::Transaction),
                    "new_address" => Some(NotificationEventType::NewAddress),
                    "system_alert" => Some(NotificationEventType::SystemAlert),
                    _ => None,
                })
                .collect();
            
            let filters_json: serde_json::Value = row.get("filters");
            let filters = serde_json::from_value(filters_json)
                .unwrap_or_default();
            
            Ok(Some(Webhook {
                id: row.get("id"),
                name: row.get("name"),
                url: row.get("url"),
                secret: row.get("secret"),
                enabled: row.get("enabled"),
                events,
                filters,
                retry_count: 0, // Default value since not in database
                timeout: 30000, // Default value since not in database
                success_count: row.get("success_count"),
                failure_count: row.get("failure_count"),
                last_triggered: row.get("last_triggered"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    /// 更新 Webhook
    pub async fn update_webhook(&self, webhook: &Webhook) -> Result<()> {
        debug!("Updating webhook: {}", webhook.name);
        
        let query = r#"
            UPDATE webhooks
            SET name = $2, url = $3, secret = $4, enabled = $5,
                events = $6, filters = $7, success_count = $8,
                failure_count = $9, last_triggered = $10, updated_at = $11
            WHERE id = $1
        "#;
        
        // 将事件枚举转换为字符串数组
        let events: Vec<String> = webhook.events.iter()
            .map(|e| format!("{:?}", e).to_lowercase())
            .collect();
        
        // 将filters转换为JSON
        let filters_json = serde_json::to_value(&webhook.filters)
            .map_err(|e| anyhow::anyhow!("Invalid filters: {}", e))?;
        
        let result = sqlx::query(query)
            .bind(&webhook.id)
            .bind(&webhook.name)
            .bind(&webhook.url)
            .bind(&webhook.secret)
            .bind(webhook.enabled)
            .bind(&events)
            .bind(&filters_json)
            .bind(webhook.success_count)
            .bind(webhook.failure_count)
            .bind(webhook.last_triggered)
            .bind(webhook.updated_at)
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;
        
        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Webhook not found"));
        }
        
        info!("Webhook updated successfully: {} ({})", webhook.name, webhook.id);
        Ok(())
    }

    /// 删除 Webhook
    pub async fn delete_webhook(&self, webhook_id: &str) -> Result<()> {
        debug!("Deleting webhook: {}", webhook_id);
        
        let uuid = Uuid::parse_str(webhook_id)
            .map_err(|e| anyhow::anyhow!("Invalid UUID: {}", e))?;
        
        let query = "DELETE FROM webhooks WHERE id = $1";
        
        let result = sqlx::query(query)
            .bind(&uuid)
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;
        
        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Webhook not found"));
        }
        
        info!("Webhook deleted successfully: {}", webhook_id);
        Ok(())
    }


    /// 更新 Webhook 统计
    pub async fn update_webhook_stats(&self, webhook_id: &str, success: bool) -> Result<()> {
        debug!("Updating webhook stats: {} success: {}", webhook_id, success);
        
        let uuid = Uuid::parse_str(webhook_id)
            .map_err(|e| anyhow::anyhow!("Invalid UUID: {}", e))?;
        
        let query = if success {
            "UPDATE webhooks SET success_count = success_count + 1, last_triggered = NOW(), updated_at = NOW() WHERE id = $1"
        } else {
            "UPDATE webhooks SET failure_count = failure_count + 1, updated_at = NOW() WHERE id = $1"
        };
        
        sqlx::query(query)
            .bind(&uuid)
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;
        
        Ok(())
    }

    // ==================== API Key 相关操作 ====================

    /// 保存 API Key
    pub async fn save_api_key(&self, api_key: &ApiKey) -> Result<()> {
        debug!("Saving API key: {}", api_key.name);
        
        let query = r#"
            INSERT INTO api_keys (
                id, name, key_hash, permissions, enabled,
                rate_limit, request_count, last_used,
                expires_at, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#;
        
        // 将权限枚举转换为字符串数组
        let permissions: Vec<String> = api_key.permissions.iter()
            .map(|p| format!("{:?}", p))
            .collect();
        
        sqlx::query(query)
            .bind(&api_key.id)
            .bind(&api_key.name)
            .bind(&api_key.key_hash)
            .bind(&permissions)
            .bind(api_key.enabled)
            .bind(api_key.rate_limit)
            .bind(api_key.request_count)
            .bind(api_key.last_used)
            .bind(api_key.expires_at)
            .bind(api_key.created_at)
            .bind(api_key.updated_at)
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;
        
        info!("API key saved successfully: {} ({})", api_key.name, api_key.id);
        Ok(())
    }

    /// 根据哈希获取 API Key
    pub async fn get_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>> {
        debug!("Getting API key by hash: {}", key_hash);
        
        let query = r#"
            SELECT id, name, key_hash, permissions, enabled,
                   rate_limit, request_count, last_used,
                   expires_at, created_at, updated_at
            FROM api_keys
            WHERE key_hash = $1 AND enabled = true
        "#;
        
        let row = sqlx::query(query)
            .bind(key_hash)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;
        
        if let Some(row) = row {
            use sqlx::Row;
            let permissions_str: Vec<String> = row.get("permissions");
            let permissions = permissions_str.into_iter()
                .filter_map(|s| match s.as_str() {
                    "ReadTransactions" => Some(Permission::ReadTransactions),
                    "ReadAddresses" => Some(Permission::ReadAddresses),
                    "ReadBlocks" => Some(Permission::ReadBlocks),
                    "ManageWebhooks" => Some(Permission::ManageWebhooks),
                    "ManageApiKeys" => Some(Permission::ManageApiKeys),
                    "ManageSystem" => Some(Permission::ManageSystem),
                    _ => None,
                })
                .collect();
            
            Ok(Some(ApiKey {
                id: row.get("id"),
                name: row.get("name"),
                key_hash: row.get("key_hash"),
                permissions,
                enabled: row.get("enabled"),
                rate_limit: row.get("rate_limit"),
                request_count: row.get("request_count"),
                last_used: row.get("last_used"),
                expires_at: row.get("expires_at"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    /// 记录 API Key 使用
    pub async fn record_api_key_usage(&self, key_id: &str, endpoint: &str, ip: &str) -> Result<()> {
        debug!("Recording API key usage: {} {} {}", key_id, endpoint, ip);
        
        let uuid = Uuid::parse_str(key_id)
            .map_err(|e| anyhow::anyhow!("Invalid UUID: {}", e))?;
        
        // 记录到日志表（如果存在）
        let query = r#"
            INSERT INTO api_key_usage (key_id, endpoint, ip_address, response_status, created_at)
            VALUES ($1, $2, $3, 200, NOW())
        "#;
        
        // 尝试插入日志，如果表不存在则忽略
        let _ = sqlx::query(query)
            .bind(&uuid)
            .bind(endpoint)
            .bind(ip)
            .execute(&self.pool)
            .await;
        
        // 同时更新API key的使用计数
        self.update_api_key_usage(key_id).await
    }

    /// 数据库健康检查
    pub async fn health_check(&self) -> Result<bool> {
        debug!("Performing database health check");
        // TODO: 实现数据库健康检查，执行简单查询如 SELECT 1
        Ok(true)
    }

    // ==================== 系统配置相关操作 ====================

    /// 获取系统配置
    pub async fn get_system_config(&self, key: &str) -> Result<Option<serde_json::Value>> {
        debug!("Getting system config: {}", key);
        // TODO: 实现系统配置查询
        Ok(None)
    }

    /// 保存系统配置
    pub async fn save_system_config(&self, key: &str, _value: &serde_json::Value) -> Result<()> {
        debug!("Saving system config: {}", key);
        // TODO: 实现系统配置保存
        Ok(())
    }

    /// 获取交易统计信息
    pub async fn get_transaction_statistics(&self) -> Result<crate::api::handlers::admin::TransactionStats> {
        debug!("Getting transaction statistics");
        // TODO: 实现交易统计查询
        Ok(crate::api::handlers::admin::TransactionStats {
            total_transactions: 0,
            transactions_today: 0,
            transactions_this_week: 0,
            transactions_this_month: 0,
            success_rate: 100.0,
            average_amount: "0".to_string(),
            total_volume: "0".to_string(),
            unique_addresses: 0,
            top_tokens: Vec::new(),
        })
    }

    /// 获取API统计信息
    pub async fn get_api_statistics(&self) -> Result<crate::api::handlers::admin::ApiStats> {
        debug!("Getting API statistics");
        
        // 获取API密钥统计
        let key_stats_query = r#"
            SELECT 
                COUNT(*) as total_keys,
                COUNT(CASE WHEN enabled = true THEN 1 END) as active_keys,
                SUM(request_count) as total_requests
            FROM api_keys
        "#;
        
        let stats_row = sqlx::query(key_stats_query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;
        
        use sqlx::Row;
        let total_api_keys: i64 = stats_row.get("total_keys");
        let active_api_keys: i64 = stats_row.get("active_keys");
        let total_requests: Option<i64> = stats_row.get("total_requests");
        
        // 尝试获取今日请求统计（如果有日志表）
        let today_stats_query = r#"
            SELECT 
                COUNT(*) as requests_today,
                COUNT(CASE WHEN response_status >= 200 AND response_status < 300 THEN 1 END) as successful_requests,
                COUNT(CASE WHEN response_status >= 400 THEN 1 END) as failed_requests,
                AVG(response_time_ms) as avg_response_time
            FROM api_key_usage
            WHERE DATE(created_at) = CURRENT_DATE
        "#;
        
        let today_stats = sqlx::query(today_stats_query)
            .fetch_optional(&self.pool)
            .await
            .ok()
            .flatten();
        
        let (total_requests_today, successful_requests_today, failed_requests_today, average_response_time_ms) = 
            if let Some(row) = today_stats {
                (
                    row.get::<Option<i64>, _>("requests_today").unwrap_or(0),
                    row.get::<Option<i64>, _>("successful_requests").unwrap_or(0),
                    row.get::<Option<i64>, _>("failed_requests").unwrap_or(0),
                    row.get::<Option<f64>, _>("avg_response_time").unwrap_or(0.0),
                )
            } else {
                (0, 0, 0, 0.0)
            };
        
        Ok(crate::api::handlers::admin::ApiStats {
            total_api_keys: total_api_keys as u32,
            active_api_keys: active_api_keys as u32,
            total_requests_today: total_requests_today as u64,
            successful_requests_today: successful_requests_today as u64,
            failed_requests_today: failed_requests_today as u64,
            average_response_time_ms,
            top_endpoints: Vec::new(), // TODO: 从日志表聚合
            rate_limited_requests: 0,  // TODO: 从日志表聚合
        })
    }

    /// 获取性能指标
    pub async fn get_performance_metrics(&self) -> Result<crate::api::handlers::admin::PerformanceMetrics> {
        debug!("Getting performance metrics");
        // TODO: 实现性能指标查询
        Ok(crate::api::handlers::admin::PerformanceMetrics {
            database_connection_pool: crate::api::handlers::admin::PoolStats {
                active_connections: 5,
                idle_connections: 5,
                max_connections: 10,
                total_connections: 10,
            },
            redis_connection_pool: crate::api::handlers::admin::PoolStats {
                active_connections: 2,
                idle_connections: 8,
                max_connections: 10,
                total_connections: 10,
            },
            cache_hit_rate: 85.5,
            average_query_time_ms: 12.3,
            slow_queries_count: 0,
            memory_usage_trend: Vec::new(),
            cpu_usage_trend: Vec::new(),
        })
    }

    /// 获取所有系统配置
    pub async fn get_all_system_config(&self) -> Result<crate::api::handlers::admin::SystemConfig> {
        debug!("Getting system config");
        
        // 查询所有配置项
        let query = "SELECT key, value FROM system_config ORDER BY key";
        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;
        
        // 创建默认配置
        let mut config = crate::api::handlers::admin::SystemConfig {
            scanner_config: crate::api::handlers::admin::ScannerConfig {
                enabled: true,
                scan_interval_ms: 5000,
                batch_size: 10,
                start_block: Some(62800000),
                end_block: None,
                nodes: Vec::new(),
            },
            database_config: crate::api::handlers::admin::DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                database: "tron_tracker".to_string(),
                username: "postgres".to_string(),
                max_connections: 10,
                connection_timeout_ms: 30000,
            },
            cache_config: crate::api::handlers::admin::CacheConfig {
                enabled: true,
                redis_url: "redis://localhost:6379".to_string(),
                max_connections: 10,
                default_ttl_seconds: 3600,
                max_memory_mb: 512,
            },
            api_config: crate::api::handlers::admin::ApiConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                cors_enabled: true,
                rate_limit_enabled: true,
                default_rate_limit: 1000,
                request_timeout_ms: 30000,
            },
            webhook_config: crate::api::handlers::admin::WebhookConfig {
                enabled: true,
                max_retries: 3,
                retry_delay_ms: 1000,
                timeout_ms: 30000,
                max_concurrent_deliveries: 10,
            },
            websocket_config: crate::api::handlers::admin::WebSocketConfig {
                enabled: true,
                port: 8081,
                max_connections: 1000,
                heartbeat_interval_ms: 30000,
                message_buffer_size: 1000,
            },
        };
        
        // 覆盖数据库中存储的配置
        for row in rows {
            use sqlx::Row;
            let key: String = row.get("key");
            let value: serde_json::Value = row.get("value");
            
            match key.as_str() {
                "scanner_config" => {
                    if let Ok(scanner_config) = serde_json::from_value(value) {
                        config.scanner_config = scanner_config;
                    }
                }
                "database_config" => {
                    if let Ok(database_config) = serde_json::from_value(value) {
                        config.database_config = database_config;
                    }
                }
                "cache_config" => {
                    if let Ok(cache_config) = serde_json::from_value(value) {
                        config.cache_config = cache_config;
                    }
                }
                "api_config" => {
                    if let Ok(api_config) = serde_json::from_value(value) {
                        config.api_config = api_config;
                    }
                }
                "webhook_config" => {
                    if let Ok(webhook_config) = serde_json::from_value(value) {
                        config.webhook_config = webhook_config;
                    }
                }
                "websocket_config" => {
                    if let Ok(websocket_config) = serde_json::from_value(value) {
                        config.websocket_config = websocket_config;
                    }
                }
                _ => {} // 忽略未知配置项
            }
        }
        
        Ok(config)
    }

    /// 更新系统配置
    pub async fn update_system_config(&self, config: &crate::api::handlers::admin::SystemConfig) -> Result<()> {
        debug!("Updating system config");
        
        // 使用事务来保证原子性
        let mut tx = self.pool.begin().await
            .map_err(|e| anyhow::anyhow!("Failed to start transaction: {}", e))?;
        
        // 准备配置项映射
        let config_items = vec![
            ("scanner_config", serde_json::to_value(&config.scanner_config)?),
            ("database_config", serde_json::to_value(&config.database_config)?),
            ("cache_config", serde_json::to_value(&config.cache_config)?),
            ("api_config", serde_json::to_value(&config.api_config)?),
            ("webhook_config", serde_json::to_value(&config.webhook_config)?),
            ("websocket_config", serde_json::to_value(&config.websocket_config)?),
        ];
        
        // 更新每个配置项
        for (key, value) in config_items {
            let query = r#"
                INSERT INTO system_config (key, value, updated_by, created_at, updated_at)
                VALUES ($1, $2, 'admin', NOW(), NOW())
                ON CONFLICT (key) 
                DO UPDATE SET 
                    value = EXCLUDED.value,
                    updated_by = EXCLUDED.updated_by,
                    updated_at = NOW()
            "#;
            
            sqlx::query(query)
                .bind(key)
                .bind(&value)
                .execute(&mut *tx)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to update config {}: {}", key, e))?;
        }
        
        // 提交事务
        tx.commit().await
            .map_err(|e| anyhow::anyhow!("Failed to commit transaction: {}", e))?;
        
        info!("System configuration updated successfully");
        Ok(())
    }

    /// 获取配置变更历史
    pub async fn get_config_history(&self, page: u32, limit: u32) -> Result<serde_json::Value> {
        debug!("Getting config history, page: {}, limit: {}", page, limit);
        
        let offset = (page - 1) * limit;
        
        // 查询配置变更历史
        let query = r#"
            SELECT key, value, updated_by, created_at, updated_at,
                   description
            FROM system_config
            ORDER BY updated_at DESC
            LIMIT $1 OFFSET $2
        "#;
        
        let rows = sqlx::query(query)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;
        
        // 获取总数
        let count_query = "SELECT COUNT(*) as total FROM system_config";
        let total_row = sqlx::query(count_query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;
        
        use sqlx::Row;
        let total_count: i64 = total_row.get("total");
        
        let mut history_items = Vec::new();
        for row in rows {
            let key: String = row.get("key");
            let value: serde_json::Value = row.get("value");
            let updated_by: Option<String> = row.get("updated_by");
            let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
            let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");
            let description: Option<String> = row.get("description");
            
            history_items.push(serde_json::json!({
                "key": key,
                "value": value,
                "updated_by": updated_by,
                "created_at": created_at,
                "updated_at": updated_at,
                "description": description
            }));
        }
        
        Ok(serde_json::json!({
            "history": history_items,
            "total_count": total_count,
            "page": page,
            "limit": limit,
            "total_pages": (total_count as f64 / limit as f64).ceil() as u32
        }))
    }

    /// 获取日志
    pub async fn get_logs(
        &self,
        params: &crate::api::handlers::admin::LogQueryParams,
        page: u32,
        limit: u32,
    ) -> Result<(Vec<crate::api::handlers::admin::LogEntry>, u64)> {
        debug!("Getting logs, page: {}, limit: {}", page, limit);
        
        let offset = (page - 1) * limit;
        let mut where_conditions = Vec::new();
        let mut param_count = 0;

        // 构建WHERE条件
        let mut query = r#"
            SELECT id, timestamp, level, module, message, details, trace_id
            FROM system_logs
        "#.to_string();

        if let Some(ref level) = params.level {
            param_count += 1;
            where_conditions.push(format!("level = ${}", param_count));
        }

        if let Some(ref module) = params.module {
            param_count += 1;
            where_conditions.push(format!("module ILIKE ${}", param_count));
        }

        if let Some(start_time) = params.start_time {
            param_count += 1;
            where_conditions.push(format!("timestamp >= ${}", param_count));
        }

        if let Some(end_time) = params.end_time {
            param_count += 1;
            where_conditions.push(format!("timestamp <= ${}", param_count));
        }

        if let Some(ref search) = params.search {
            param_count += 1;
            where_conditions.push(format!("(message ILIKE ${} OR module ILIKE ${})", param_count, param_count));
        }

        if !where_conditions.is_empty() {
            query.push_str(&format!(" WHERE {}", where_conditions.join(" AND ")));
        }

        query.push_str(" ORDER BY timestamp DESC");
        
        // 分页
        param_count += 1;
        query.push_str(&format!(" LIMIT ${}", param_count));
        param_count += 1;
        query.push_str(&format!(" OFFSET ${}", param_count));

        // 构建查询
        let mut sqlx_query = sqlx::query(&query);
        
        // 绑定参数
        if let Some(ref level) = params.level {
            sqlx_query = sqlx_query.bind(level);
        }
        if let Some(ref module) = params.module {
            sqlx_query = sqlx_query.bind(format!("%{}%", module));
        }
        if let Some(start_time) = params.start_time {
            let start_dt = chrono::DateTime::from_timestamp(start_time, 0)
                .unwrap_or_else(chrono::Utc::now);
            sqlx_query = sqlx_query.bind(start_dt);
        }
        if let Some(end_time) = params.end_time {
            let end_dt = chrono::DateTime::from_timestamp(end_time, 0)
                .unwrap_or_else(chrono::Utc::now);
            sqlx_query = sqlx_query.bind(end_dt);
        }
        if let Some(ref search) = params.search {
            sqlx_query = sqlx_query.bind(format!("%{}%", search));
        }
        
        sqlx_query = sqlx_query.bind(limit as i64).bind(offset as i64);

        let rows = sqlx_query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        // 获取总数
        let mut count_query = "SELECT COUNT(*) as total FROM system_logs".to_string();
        if !where_conditions.is_empty() {
            // 重新构建WHERE条件（不包含LIMIT和OFFSET）
            let count_conditions = where_conditions[..where_conditions.len().saturating_sub(2)].to_vec();
            if !count_conditions.is_empty() {
                count_query.push_str(&format!(" WHERE {}", count_conditions.join(" AND ")));
            }
        }

        let mut count_sqlx_query = sqlx::query(&count_query);
        
        // 重新绑定参数（不包含limit和offset）
        if let Some(ref level) = params.level {
            count_sqlx_query = count_sqlx_query.bind(level);
        }
        if let Some(ref module) = params.module {
            count_sqlx_query = count_sqlx_query.bind(format!("%{}%", module));
        }
        if let Some(start_time) = params.start_time {
            let start_dt = chrono::DateTime::from_timestamp(start_time, 0)
                .unwrap_or_else(chrono::Utc::now);
            count_sqlx_query = count_sqlx_query.bind(start_dt);
        }
        if let Some(end_time) = params.end_time {
            let end_dt = chrono::DateTime::from_timestamp(end_time, 0)
                .unwrap_or_else(chrono::Utc::now);
            count_sqlx_query = count_sqlx_query.bind(end_dt);
        }
        if let Some(ref search) = params.search {
            count_sqlx_query = count_sqlx_query.bind(format!("%{}%", search));
        }

        let total_row = count_sqlx_query
            .fetch_one(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        use sqlx::Row;
        let total_count: i64 = total_row.get("total");

        // 构建结果
        let mut logs = Vec::new();
        for row in rows {
            let log_entry = crate::api::handlers::admin::LogEntry {
                id: row.get::<uuid::Uuid, _>("id").to_string(),
                timestamp: row.get("timestamp"),
                level: row.get("level"),
                module: row.get("module"),
                message: row.get("message"),
                details: row.get("details"),
                trace_id: row.get("trace_id"),
            };
            logs.push(log_entry);
        }

        Ok((logs, total_count as u64))
    }

    /// 清空日志
    pub async fn clear_logs(&self) -> Result<u64> {
        debug!("Clearing logs");
        
        // 清空系统日志表
        let result = sqlx::query("DELETE FROM system_logs")
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;
        
        let deleted_count = result.rows_affected();
        info!("Cleared {} log entries", deleted_count);
        
        Ok(deleted_count)
    }

    /// 导出日志
    pub async fn export_logs(&self, params: &crate::api::handlers::admin::LogQueryParams) -> Result<String> {
        debug!("Exporting logs");
        
        let mut where_conditions = Vec::new();
        let mut param_count = 0;

        // 构建WHERE条件
        let mut query = r#"
            SELECT timestamp, level, module, message, details, trace_id
            FROM system_logs
        "#.to_string();

        if let Some(ref level) = params.level {
            param_count += 1;
            where_conditions.push(format!("level = ${}", param_count));
        }

        if let Some(ref module) = params.module {
            param_count += 1;
            where_conditions.push(format!("module ILIKE ${}", param_count));
        }

        if let Some(start_time) = params.start_time {
            param_count += 1;
            where_conditions.push(format!("timestamp >= ${}", param_count));
        }

        if let Some(end_time) = params.end_time {
            param_count += 1;
            where_conditions.push(format!("timestamp <= ${}", param_count));
        }

        if let Some(ref search) = params.search {
            param_count += 1;
            where_conditions.push(format!("(message ILIKE ${} OR module ILIKE ${})", param_count, param_count));
        }

        if !where_conditions.is_empty() {
            query.push_str(&format!(" WHERE {}", where_conditions.join(" AND ")));
        }

        query.push_str(" ORDER BY timestamp DESC LIMIT 10000"); // 限制导出数量避免内存问题

        // 构建查询
        let mut sqlx_query = sqlx::query(&query);
        
        // 绑定参数
        if let Some(ref level) = params.level {
            sqlx_query = sqlx_query.bind(level);
        }
        if let Some(ref module) = params.module {
            sqlx_query = sqlx_query.bind(format!("%{}%", module));
        }
        if let Some(start_time) = params.start_time {
            let start_dt = chrono::DateTime::from_timestamp(start_time, 0)
                .unwrap_or_else(chrono::Utc::now);
            sqlx_query = sqlx_query.bind(start_dt);
        }
        if let Some(end_time) = params.end_time {
            let end_dt = chrono::DateTime::from_timestamp(end_time, 0)
                .unwrap_or_else(chrono::Utc::now);
            sqlx_query = sqlx_query.bind(end_dt);
        }
        if let Some(ref search) = params.search {
            sqlx_query = sqlx_query.bind(format!("%{}%", search));
        }

        let rows = sqlx_query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        // 构建CSV内容
        let mut csv_content = String::new();
        csv_content.push_str("timestamp,level,module,message,details,trace_id\n");

        use sqlx::Row;
        let rows_len = rows.len();
        for row in rows {
            let timestamp: chrono::DateTime<chrono::Utc> = row.get("timestamp");
            let level: String = row.get("level");
            let module: String = row.get("module");
            let message: String = row.get("message");
            let details: Option<serde_json::Value> = row.get("details");
            let trace_id: Option<String> = row.get("trace_id");

            // 转义CSV字段
            let escaped_message = message.replace("\"", "\"\"");
            let details_str = details
                .map(|v| v.to_string().replace("\"", "\"\""))
                .unwrap_or_default();
            let trace_id_str = trace_id.unwrap_or_default();

            csv_content.push_str(&format!(
                "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
                timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
                level,
                module,
                escaped_message,
                details_str,
                trace_id_str
            ));
        }

        info!("Exported {} log entries to CSV", rows_len);
        Ok(csv_content)
    }

    /// 保存系统日志
    pub async fn save_log(
        &self,
        level: &str,
        module: &str,
        message: &str,
        details: Option<serde_json::Value>,
        trace_id: Option<&str>,
    ) -> Result<()> {
        let query = r#"
            INSERT INTO system_logs (level, module, message, details, trace_id)
            VALUES ($1, $2, $3, $4, $5)
        "#;

        sqlx::query(query)
            .bind(level)
            .bind(module)
            .bind(message)
            .bind(details)
            .bind(trace_id)
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to save log: {}", e))?;

        Ok(())
    }

    /// 获取 Webhook 投递日志
    pub async fn get_webhook_delivery_logs(
        &self,
        webhook_id: &str,
        page: u32,
        limit: u32,
    ) -> Result<Vec<crate::api::handlers::webhook::WebhookDeliveryLog>> {
        debug!("Getting webhook delivery logs for webhook: {}", webhook_id);
        
        let offset = (page - 1) * limit;
        
        let query = r#"
            SELECT id, webhook_id, event_type, payload, status_code,
                   response_body, error, attempt, delivered_at
            FROM webhook_delivery_logs
            WHERE webhook_id = $1
            ORDER BY delivered_at DESC
            LIMIT $2 OFFSET $3
        "#;

        let webhook_uuid = uuid::Uuid::parse_str(webhook_id)
            .map_err(|e| anyhow::anyhow!("Invalid webhook ID: {}", e))?;

        let rows = sqlx::query(query)
            .bind(webhook_uuid)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        use sqlx::Row;
        let mut logs = Vec::new();
        for row in rows {
            let log = crate::api::handlers::webhook::WebhookDeliveryLog {
                id: row.get::<uuid::Uuid, _>("id").to_string(),
                webhook_id: row.get::<uuid::Uuid, _>("webhook_id").to_string(),
                event_type: row.get("event_type"),
                payload: row.get("payload"),
                status_code: row.get::<Option<i32>, _>("status_code").map(|v| v as u16),
                response_body: row.get("response_body"),
                error: row.get("error"),
                attempt: row.get("attempt"),
                delivered_at: row.get("delivered_at"),
            };
            logs.push(log);
        }

        Ok(logs)
    }

    /// 保存 Webhook 投递日志
    pub async fn save_webhook_delivery_log(
        &self,
        webhook_id: uuid::Uuid,
        event_type: &str,
        payload: &serde_json::Value,
        status_code: Option<u16>,
        response_body: Option<&str>,
        error: Option<&str>,
        attempt: i32,
    ) -> Result<()> {
        let query = r#"
            INSERT INTO webhook_delivery_logs 
            (webhook_id, event_type, payload, status_code, response_body, error, attempt)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#;

        sqlx::query(query)
            .bind(webhook_id)
            .bind(event_type)
            .bind(payload)
            .bind(status_code.map(|c| c as i32))
            .bind(response_body)
            .bind(error)
            .bind(attempt)
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to save webhook delivery log: {}", e))?;

        Ok(())
    }
}

/// 数据库统计信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct DatabaseStats {
    pub total_transactions: u64,
    pub total_blocks: u64,
    pub total_webhooks: u64,
    pub active_api_keys: u64,
}