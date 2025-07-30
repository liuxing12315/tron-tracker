// 数据库访问层
// 
// 提供统一的数据库操作接口，支持所有业务功能

use anyhow::{Result, anyhow};
use sqlx::{PgPool, Row, postgres::PgPoolOptions};
use std::time::Duration;
use tracing::{info, warn, error, debug};
use crate::core::config::DatabaseConfig;
use crate::core::models::*;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// 创建新的数据库连接
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        info!("Connecting to database: {}", config.url);

        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(Duration::from_secs(config.acquire_timeout))
            .connect(&config.url)
            .await?;

        // 运行数据库迁移
        sqlx::migrate!("./migrations").run(&pool).await?;

        info!("Database connected successfully");
        Ok(Self { pool })
    }

    /// 获取数据库连接池
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    // ==================== 交易相关操作 ====================

    /// 保存交易记录
    pub async fn save_transaction(&self, transaction: &Transaction) -> Result<()> {
        let status_str = match transaction.status {
            TransactionStatus::Success => "success",
            TransactionStatus::Failed => "failed",
            TransactionStatus::Pending => "pending",
        };

        sqlx::query!(
            r#"
            INSERT INTO transactions (
                hash, block_number, from_address, to_address, amount, token,
                status, timestamp, gas_used, gas_price, contract_address,
                token_symbol, token_decimals
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ON CONFLICT (hash) DO UPDATE SET
                status = EXCLUDED.status,
                gas_used = EXCLUDED.gas_used,
                gas_price = EXCLUDED.gas_price
            "#,
            transaction.hash,
            transaction.block_number as i64,
            transaction.from_address,
            transaction.to_address,
            transaction.amount,
            transaction.token,
            status_str,
            transaction.timestamp,
            transaction.gas_used,
            transaction.gas_price,
            transaction.contract_address,
            transaction.token_symbol,
            transaction.token_decimals
        )
        .execute(&self.pool)
        .await?;

        debug!("Saved transaction: {}", transaction.hash);
        Ok(())
    }

    /// 根据哈希获取交易
    pub async fn get_transaction_by_hash(&self, hash: &str) -> Result<Option<Transaction>> {
        let row = sqlx::query!(
            "SELECT * FROM transactions WHERE hash = $1",
            hash
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(self.row_to_transaction(row)?))
        } else {
            Ok(None)
        }
    }

    /// 查询交易列表
    pub async fn list_transactions(&self, query: &TransactionQuery) -> Result<(Vec<Transaction>, u64)> {
        let mut sql = String::from("SELECT * FROM transactions WHERE 1=1");
        let mut count_sql = String::from("SELECT COUNT(*) FROM transactions WHERE 1=1");
        let mut params: Vec<Box<dyn sqlx::Encode<sqlx::Postgres> + Send + Sync>> = Vec::new();
        let mut param_index = 1;

        // 构建查询条件
        if let Some(ref address) = query.address {
            sql.push_str(&format!(" AND (from_address = ${} OR to_address = ${})", param_index, param_index + 1));
            count_sql.push_str(&format!(" AND (from_address = ${} OR to_address = ${})", param_index, param_index + 1));
            params.push(Box::new(address.clone()));
            params.push(Box::new(address.clone()));
            param_index += 2;
        }

        if let Some(ref token) = query.token {
            sql.push_str(&format!(" AND token = ${}", param_index));
            count_sql.push_str(&format!(" AND token = ${}", param_index));
            params.push(Box::new(token.clone()));
            param_index += 1;
        }

        if let Some(ref status) = query.status {
            let status_str = match status {
                TransactionStatus::Success => "success",
                TransactionStatus::Failed => "failed",
                TransactionStatus::Pending => "pending",
            };
            sql.push_str(&format!(" AND status = ${}", param_index));
            count_sql.push_str(&format!(" AND status = ${}", param_index));
            params.push(Box::new(status_str.to_string()));
            param_index += 1;
        }

        if let Some(start_time) = query.start_time {
            sql.push_str(&format!(" AND timestamp >= ${}", param_index));
            count_sql.push_str(&format!(" AND timestamp >= ${}", param_index));
            params.push(Box::new(start_time));
            param_index += 1;
        }

        if let Some(end_time) = query.end_time {
            sql.push_str(&format!(" AND timestamp <= ${}", param_index));
            count_sql.push_str(&format!(" AND timestamp <= ${}", param_index));
            params.push(Box::new(end_time));
            param_index += 1;
        }

        // 添加排序和分页
        sql.push_str(" ORDER BY timestamp DESC");
        
        let page = query.pagination.page.unwrap_or(1);
        let limit = query.pagination.limit.unwrap_or(20);
        let offset = (page - 1) * limit;

        sql.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

        // 执行查询（这里简化处理，实际应该使用参数化查询）
        let transactions = self.execute_transaction_query(&sql).await?;
        let total = self.execute_count_query(&count_sql).await?;

        Ok((transactions, total))
    }

    /// 多地址批量查询
    pub async fn list_transactions_by_addresses(&self, addresses: &[String], query: &MultiAddressQuery) -> Result<(Vec<Transaction>, u64)> {
        if addresses.is_empty() {
            return Ok((Vec::new(), 0));
        }

        let mut sql = String::from("SELECT * FROM transactions WHERE (");
        let mut count_sql = String::from("SELECT COUNT(*) FROM transactions WHERE (");
        
        // 构建地址条件
        let address_conditions: Vec<String> = addresses.iter()
            .enumerate()
            .map(|(i, _)| format!("from_address = ${} OR to_address = ${}", i * 2 + 1, i * 2 + 2))
            .collect();
        
        let address_clause = address_conditions.join(" OR ");
        sql.push_str(&address_clause);
        sql.push(')');
        count_sql.push_str(&address_clause);
        count_sql.push(')');

        // 添加其他过滤条件
        let mut param_index = addresses.len() * 2 + 1;

        if let Some(ref token) = query.token {
            sql.push_str(&format!(" AND token = ${}", param_index));
            count_sql.push_str(&format!(" AND token = ${}", param_index));
            param_index += 1;
        }

        if let Some(ref status) = query.status {
            let status_str = match status {
                TransactionStatus::Success => "success",
                TransactionStatus::Failed => "failed",
                TransactionStatus::Pending => "pending",
            };
            sql.push_str(&format!(" AND status = ${}", param_index));
            count_sql.push_str(&format!(" AND status = ${}", param_index));
            param_index += 1;
        }

        if let Some(start_time) = query.start_time {
            sql.push_str(&format!(" AND timestamp >= ${}", param_index));
            count_sql.push_str(&format!(" AND timestamp >= ${}", param_index));
            param_index += 1;
        }

        if let Some(end_time) = query.end_time {
            sql.push_str(&format!(" AND timestamp <= ${}", param_index));
            count_sql.push_str(&format!(" AND timestamp <= ${}", param_index));
        }

        // 添加排序和分页
        sql.push_str(" ORDER BY timestamp DESC");
        
        let page = query.pagination.page.unwrap_or(1);
        let limit = query.pagination.limit.unwrap_or(20);
        let offset = (page - 1) * limit;

        sql.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

        // 执行查询（简化处理）
        let transactions = self.execute_multi_address_query(&sql, addresses).await?;
        let total = self.execute_multi_address_count_query(&count_sql, addresses).await?;

        Ok((transactions, total))
    }

    /// 获取地址统计信息
    pub async fn get_address_statistics(&self, address: &str) -> Result<AddressStatistics> {
        let stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total_transactions,
                COUNT(CASE WHEN status = 'success' THEN 1 END) as successful_transactions,
                COUNT(CASE WHEN from_address = $1 THEN 1 END) as sent_transactions,
                COUNT(CASE WHEN to_address = $1 THEN 1 END) as received_transactions,
                COALESCE(SUM(CASE WHEN to_address = $1 AND token = 'TRX' THEN amount::numeric ELSE 0 END), 0) as total_trx_received,
                COALESCE(SUM(CASE WHEN to_address = $1 AND token = 'USDT' THEN amount::numeric ELSE 0 END), 0) as total_usdt_received,
                MIN(timestamp) as first_transaction,
                MAX(timestamp) as last_transaction
            FROM transactions 
            WHERE from_address = $1 OR to_address = $1
            "#,
            address
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(AddressStatistics {
            address: address.to_string(),
            total_transactions: stats.total_transactions.unwrap_or(0) as u64,
            successful_transactions: stats.successful_transactions.unwrap_or(0) as u64,
            sent_transactions: stats.sent_transactions.unwrap_or(0) as u64,
            received_transactions: stats.received_transactions.unwrap_or(0) as u64,
            total_trx_received: stats.total_trx_received.unwrap_or(0.into()).to_string(),
            total_usdt_received: stats.total_usdt_received.unwrap_or(0.into()).to_string(),
            first_transaction: stats.first_transaction,
            last_transaction: stats.last_transaction,
        })
    }

    // ==================== 区块相关操作 ====================

    /// 保存区块信息
    pub async fn save_block(&self, block: &BlockData) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO blocks (number, hash, parent_hash, timestamp, transaction_count)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (number) DO UPDATE SET
                hash = EXCLUDED.hash,
                parent_hash = EXCLUDED.parent_hash,
                timestamp = EXCLUDED.timestamp,
                transaction_count = EXCLUDED.transaction_count
            "#,
            block.number as i64,
            block.hash,
            block.parent_hash,
            chrono::DateTime::from_timestamp(block.timestamp as i64, 0).unwrap_or_else(|| chrono::Utc::now()),
            block.transaction_count as i32
        )
        .execute(&self.pool)
        .await?;

        debug!("Saved block: {}", block.number);
        Ok(())
    }

    /// 获取最后处理的区块号
    pub async fn get_last_processed_block(&self) -> Result<Option<u64>> {
        let row = sqlx::query!(
            "SELECT MAX(number) as max_block FROM blocks"
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.max_block.map(|n| n as u64))
    }

    /// 保存扫描进度
    pub async fn save_scan_progress(&self, block_number: u64) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO scan_progress (id, last_block, updated_at)
            VALUES (1, $1, NOW())
            ON CONFLICT (id) DO UPDATE SET
                last_block = EXCLUDED.last_block,
                updated_at = EXCLUDED.updated_at
            "#,
            block_number as i64
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // ==================== Webhook 相关操作 ====================

    /// 保存 Webhook 配置
    pub async fn save_webhook(&self, webhook: &Webhook) -> Result<()> {
        let events_json = serde_json::to_value(&webhook.events)?;
        let filters_json = serde_json::to_value(&webhook.filters)?;

        sqlx::query!(
            r#"
            INSERT INTO webhooks (
                id, name, url, secret, events, filters, enabled, 
                retry_count, timeout, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                url = EXCLUDED.url,
                secret = EXCLUDED.secret,
                events = EXCLUDED.events,
                filters = EXCLUDED.filters,
                enabled = EXCLUDED.enabled,
                retry_count = EXCLUDED.retry_count,
                timeout = EXCLUDED.timeout,
                updated_at = EXCLUDED.updated_at
            "#,
            webhook.id,
            webhook.name,
            webhook.url,
            webhook.secret,
            events_json,
            filters_json,
            webhook.enabled,
            webhook.retry_count as i32,
            webhook.timeout as i32,
            webhook.created_at,
            webhook.updated_at
        )
        .execute(&self.pool)
        .await?;

        debug!("Saved webhook: {}", webhook.id);
        Ok(())
    }

    /// 获取所有启用的 Webhook
    pub async fn get_enabled_webhooks(&self) -> Result<Vec<Webhook>> {
        let rows = sqlx::query!(
            "SELECT * FROM webhooks WHERE enabled = true ORDER BY created_at"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut webhooks = Vec::new();
        for row in rows {
            webhooks.push(Webhook {
                id: row.id,
                name: row.name,
                url: row.url,
                secret: row.secret,
                events: serde_json::from_value(row.events).unwrap_or_default(),
                filters: serde_json::from_value(row.filters).unwrap_or_default(),
                enabled: row.enabled,
                retry_count: row.retry_count as u32,
                timeout: row.timeout as u32,
                created_at: row.created_at,
                updated_at: row.updated_at,
                statistics: None, // 统计信息需要单独查询
            });
        }

        Ok(webhooks)
    }

    /// 获取 Webhook 列表
    pub async fn list_webhooks(&self, enabled_only: bool) -> Result<Vec<Webhook>> {
        let sql = if enabled_only {
            "SELECT * FROM webhooks WHERE enabled = true ORDER BY created_at"
        } else {
            "SELECT * FROM webhooks ORDER BY created_at"
        };

        let rows = sqlx::query(sql)
            .fetch_all(&self.pool)
            .await?;

        let mut webhooks = Vec::new();
        for row in rows {
            let webhook = Webhook {
                id: row.get("id"),
                name: row.get("name"),
                url: row.get("url"),
                secret: row.get("secret"),
                events: serde_json::from_value(row.get("events")).unwrap_or_default(),
                filters: serde_json::from_value(row.get("filters")).unwrap_or_default(),
                enabled: row.get("enabled"),
                retry_count: row.get::<i32, _>("retry_count") as u32,
                timeout: row.get::<i32, _>("timeout") as u32,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                statistics: None,
            };
            webhooks.push(webhook);
        }

        Ok(webhooks)
    }

    /// 删除 Webhook
    pub async fn delete_webhook(&self, webhook_id: &str) -> Result<()> {
        sqlx::query!(
            "DELETE FROM webhooks WHERE id = $1",
            webhook_id
        )
        .execute(&self.pool)
        .await?;

        debug!("Deleted webhook: {}", webhook_id);
        Ok(())
    }

    /// 更新 Webhook 统计信息
    pub async fn update_webhook_stats(&self, webhook_id: &str, success: bool) -> Result<()> {
        if success {
            sqlx::query!(
                r#"
                INSERT INTO webhook_stats (webhook_id, success_count, last_success)
                VALUES ($1, 1, NOW())
                ON CONFLICT (webhook_id) DO UPDATE SET
                    success_count = webhook_stats.success_count + 1,
                    last_success = NOW()
                "#,
                webhook_id
            )
            .execute(&self.pool)
            .await?;
        } else {
            sqlx::query!(
                r#"
                INSERT INTO webhook_stats (webhook_id, failure_count, last_failure)
                VALUES ($1, 1, NOW())
                ON CONFLICT (webhook_id) DO UPDATE SET
                    failure_count = webhook_stats.failure_count + 1,
                    last_failure = NOW()
                "#,
                webhook_id
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    // ==================== API Key 相关操作 ====================

    /// 保存 API Key
    pub async fn save_api_key(&self, api_key: &ApiKey) -> Result<()> {
        let permissions_json = serde_json::to_value(&api_key.permissions)?;

        sqlx::query!(
            r#"
            INSERT INTO api_keys (
                id, name, key_hash, permissions, enabled, rate_limit,
                ip_whitelist, expires_at, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                permissions = EXCLUDED.permissions,
                enabled = EXCLUDED.enabled,
                rate_limit = EXCLUDED.rate_limit,
                ip_whitelist = EXCLUDED.ip_whitelist,
                expires_at = EXCLUDED.expires_at,
                updated_at = EXCLUDED.updated_at
            "#,
            api_key.id,
            api_key.name,
            api_key.key_hash,
            permissions_json,
            api_key.enabled,
            api_key.rate_limit as i32,
            &api_key.ip_whitelist,
            api_key.expires_at,
            api_key.created_at,
            api_key.updated_at
        )
        .execute(&self.pool)
        .await?;

        debug!("Saved API key: {}", api_key.id);
        Ok(())
    }

    /// 根据 key hash 获取 API Key
    pub async fn get_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>> {
        let row = sqlx::query!(
            "SELECT * FROM api_keys WHERE key_hash = $1 AND enabled = true",
            key_hash
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(ApiKey {
                id: row.id,
                name: row.name,
                key_hash: row.key_hash,
                permissions: serde_json::from_value(row.permissions).unwrap_or_default(),
                enabled: row.enabled,
                rate_limit: row.rate_limit as u32,
                ip_whitelist: row.ip_whitelist,
                expires_at: row.expires_at,
                created_at: row.created_at,
                updated_at: row.updated_at,
                last_used: None, // 需要从使用记录中查询
                usage_count: 0,  // 需要从使用记录中查询
            }))
        } else {
            Ok(None)
        }
    }

    /// 记录 API Key 使用
    pub async fn record_api_key_usage(&self, key_id: &str, endpoint: &str, ip: &str) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO api_key_usage (key_id, endpoint, ip_address, used_at)
            VALUES ($1, $2, $3, NOW())
            "#,
            key_id,
            endpoint,
            ip
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // ==================== 辅助方法 ====================

    /// 将数据库行转换为 Transaction 对象
    fn row_to_transaction(&self, row: sqlx::postgres::PgRow) -> Result<Transaction> {
        let status = match row.get::<String, _>("status").as_str() {
            "success" => TransactionStatus::Success,
            "failed" => TransactionStatus::Failed,
            "pending" => TransactionStatus::Pending,
            _ => TransactionStatus::Pending,
        };

        Ok(Transaction {
            hash: row.get("hash"),
            block_number: row.get::<i64, _>("block_number") as u64,
            from_address: row.get("from_address"),
            to_address: row.get("to_address"),
            amount: row.get("amount"),
            token: row.get("token"),
            status,
            timestamp: row.get("timestamp"),
            gas_used: row.get("gas_used"),
            gas_price: row.get("gas_price"),
            contract_address: row.get("contract_address"),
            token_symbol: row.get("token_symbol"),
            token_decimals: row.get("token_decimals"),
        })
    }

    /// 执行交易查询（简化实现）
    async fn execute_transaction_query(&self, sql: &str) -> Result<Vec<Transaction>> {
        let rows = sqlx::query(sql)
            .fetch_all(&self.pool)
            .await?;

        let mut transactions = Vec::new();
        for row in rows {
            transactions.push(self.row_to_transaction(row)?);
        }

        Ok(transactions)
    }

    /// 执行计数查询
    async fn execute_count_query(&self, sql: &str) -> Result<u64> {
        let row = sqlx::query(sql)
            .fetch_one(&self.pool)
            .await?;

        Ok(row.get::<i64, _>(0) as u64)
    }

    /// 执行多地址查询（简化实现）
    async fn execute_multi_address_query(&self, sql: &str, addresses: &[String]) -> Result<Vec<Transaction>> {
        // 这里应该使用参数化查询，为了简化先这样实现
        let mut query = sqlx::query(sql);
        for address in addresses {
            query = query.bind(address).bind(address);
        }

        let rows = query.fetch_all(&self.pool).await?;

        let mut transactions = Vec::new();
        for row in rows {
            transactions.push(self.row_to_transaction(row)?);
        }

        Ok(transactions)
    }

    /// 执行多地址计数查询
    async fn execute_multi_address_count_query(&self, sql: &str, addresses: &[String]) -> Result<u64> {
        let mut query = sqlx::query(sql);
        for address in addresses {
            query = query.bind(address).bind(address);
        }

        let row = query.fetch_one(&self.pool).await?;
        Ok(row.get::<i64, _>(0) as u64)
    }

    /// 健康检查
    pub async fn health_check(&self) -> Result<bool> {
        match sqlx::query("SELECT 1").fetch_one(&self.pool).await {
            Ok(_) => Ok(true),
            Err(e) => {
                warn!("Database health check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// 获取数据库统计信息
    pub async fn get_database_stats(&self) -> Result<DatabaseStats> {
        let stats = sqlx::query!(
            r#"
            SELECT 
                (SELECT COUNT(*) FROM transactions) as total_transactions,
                (SELECT COUNT(*) FROM blocks) as total_blocks,
                (SELECT COUNT(*) FROM webhooks) as total_webhooks,
                (SELECT COUNT(*) FROM api_keys WHERE enabled = true) as active_api_keys
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(DatabaseStats {
            total_transactions: stats.total_transactions.unwrap_or(0) as u64,
            total_blocks: stats.total_blocks.unwrap_or(0) as u64,
            total_webhooks: stats.total_webhooks.unwrap_or(0) as u64,
            active_api_keys: stats.active_api_keys.unwrap_or(0) as u64,
        })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::DatabaseConfig;

    #[tokio::test]
    async fn test_database_connection() {
        let config = DatabaseConfig {
            url: "postgresql://test:test@localhost/test".to_string(),
            max_connections: 10,
            min_connections: 1,
            acquire_timeout: 30,
        };

        // 这个测试需要实际的数据库连接
        // 在实际环境中可以启用
        /*
        let db = Database::new(&config).await;
        assert!(db.is_ok());
        */
    }
}

