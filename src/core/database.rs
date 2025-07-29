use anyhow::Result;
use sqlx::{PgPool, Row};
use std::time::Duration;
use tracing::{info, warn};

use crate::core::config::DatabaseConfig;
use crate::core::models::*;

#[derive(Debug, Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(Duration::from_secs(config.acquire_timeout))
            .connect(&config.url)
            .await?;

        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<()> {
        info!("Running database migrations");
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        info!("Database migrations completed");
        Ok(())
    }

    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1").fetch_one(&self.pool).await?;
        Ok(())
    }

    // Transaction operations
    pub async fn get_transactions(&self, params: &QueryParams) -> Result<(Vec<Transaction>, i64)> {
        let limit = params.limit.unwrap_or(20).min(100);
        let offset = params.offset.unwrap_or(0);

        let count_query = "SELECT COUNT(*) FROM transactions";
        let total: i64 = sqlx::query(count_query)
            .fetch_one(&self.pool)
            .await?
            .get(0);

        let transactions = sqlx::query_as::<_, Transaction>(
            "SELECT * FROM transactions ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok((transactions, total))
    }

    pub async fn get_transaction_by_hash(&self, hash: &str) -> Result<Option<Transaction>> {
        let transaction = sqlx::query_as::<_, Transaction>(
            "SELECT * FROM transactions WHERE hash = $1"
        )
        .bind(hash)
        .fetch_optional(&self.pool)
        .await?;

        Ok(transaction)
    }

    pub async fn insert_transaction(&self, transaction: &Transaction) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO transactions (
                id, hash, block_number, block_hash, transaction_index,
                from_address, to_address, value, token_address, token_symbol,
                token_decimals, gas_used, gas_price, status, timestamp,
                created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17
            ) ON CONFLICT (hash) DO NOTHING
            "#
        )
        .bind(&transaction.id)
        .bind(&transaction.hash)
        .bind(transaction.block_number)
        .bind(&transaction.block_hash)
        .bind(transaction.transaction_index)
        .bind(&transaction.from_address)
        .bind(&transaction.to_address)
        .bind(&transaction.value)
        .bind(&transaction.token_address)
        .bind(&transaction.token_symbol)
        .bind(transaction.token_decimals)
        .bind(transaction.gas_used)
        .bind(&transaction.gas_price)
        .bind(&transaction.status)
        .bind(transaction.timestamp)
        .bind(transaction.created_at)
        .bind(transaction.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Multi-address query operations
    pub async fn get_transactions_by_addresses(
        &self,
        query: &MultiAddressQuery,
    ) -> Result<(Vec<Transaction>, i64)> {
        let limit = query.limit.unwrap_or(20).min(100);
        let offset = query.offset.unwrap_or(0);

        let mut where_conditions = vec!["(from_address = ANY($1) OR to_address = ANY($1))"];
        let mut bind_index = 2;

        let mut query_builder = sqlx::QueryBuilder::new(
            "SELECT * FROM transactions WHERE "
        );
        query_builder.push(&where_conditions.join(" AND "));

        if let Some(start_time) = query.start_time {
            query_builder.push(format!(" AND timestamp >= ${}", bind_index));
            bind_index += 1;
        }

        if let Some(end_time) = query.end_time {
            query_builder.push(format!(" AND timestamp <= ${}", bind_index));
            bind_index += 1;
        }

        query_builder.push(" ORDER BY timestamp DESC");
        query_builder.push(format!(" LIMIT ${} OFFSET ${}", bind_index, bind_index + 1));

        let mut query = query_builder.build_query_as::<Transaction>();
        query = query.bind(&query.addresses);

        if let Some(start_time) = query.start_time {
            query = query.bind(start_time);
        }

        if let Some(end_time) = query.end_time {
            query = query.bind(end_time);
        }

        query = query.bind(limit as i64).bind(offset as i64);

        let transactions = query.fetch_all(&self.pool).await?;

        // Get total count
        let count_query = sqlx::query(
            "SELECT COUNT(*) FROM transactions WHERE (from_address = ANY($1) OR to_address = ANY($1))"
        )
        .bind(&query.addresses);

        let total: i64 = count_query.fetch_one(&self.pool).await?.get(0);

        Ok((transactions, total))
    }

    // Address operations
    pub async fn get_address(&self, address: &str) -> Result<Option<Address>> {
        let addr = sqlx::query_as::<_, Address>(
            "SELECT * FROM addresses WHERE address = $1"
        )
        .bind(address)
        .fetch_optional(&self.pool)
        .await?;

        Ok(addr)
    }

    pub async fn upsert_address(&self, address: &Address) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO addresses (
                id, address, label, balance_trx, balance_usdt,
                transaction_count, first_seen, last_seen, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (address) DO UPDATE SET
                balance_trx = EXCLUDED.balance_trx,
                balance_usdt = EXCLUDED.balance_usdt,
                transaction_count = EXCLUDED.transaction_count,
                last_seen = EXCLUDED.last_seen,
                updated_at = EXCLUDED.updated_at
            "#
        )
        .bind(&address.id)
        .bind(&address.address)
        .bind(&address.label)
        .bind(&address.balance_trx)
        .bind(&address.balance_usdt)
        .bind(address.transaction_count)
        .bind(address.first_seen)
        .bind(address.last_seen)
        .bind(address.created_at)
        .bind(address.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Webhook operations
    pub async fn get_webhooks(&self) -> Result<Vec<Webhook>> {
        let webhooks = sqlx::query_as::<_, Webhook>(
            "SELECT * FROM webhooks ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(webhooks)
    }

    pub async fn get_webhook(&self, id: uuid::Uuid) -> Result<Option<Webhook>> {
        let webhook = sqlx::query_as::<_, Webhook>(
            "SELECT * FROM webhooks WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(webhook)
    }

    pub async fn insert_webhook(&self, webhook: &Webhook) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO webhooks (
                id, name, url, secret, enabled, events, filters,
                success_count, failure_count, last_triggered, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#
        )
        .bind(&webhook.id)
        .bind(&webhook.name)
        .bind(&webhook.url)
        .bind(&webhook.secret)
        .bind(webhook.enabled)
        .bind(&webhook.events)
        .bind(&webhook.filters)
        .bind(webhook.success_count)
        .bind(webhook.failure_count)
        .bind(webhook.last_triggered)
        .bind(webhook.created_at)
        .bind(webhook.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_webhook(&self, webhook: &Webhook) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE webhooks SET
                name = $2, url = $3, secret = $4, enabled = $5,
                events = $6, filters = $7, updated_at = $8
            WHERE id = $1
            "#
        )
        .bind(&webhook.id)
        .bind(&webhook.name)
        .bind(&webhook.url)
        .bind(&webhook.secret)
        .bind(webhook.enabled)
        .bind(&webhook.events)
        .bind(&webhook.filters)
        .bind(webhook.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_webhook(&self, id: uuid::Uuid) -> Result<()> {
        sqlx::query("DELETE FROM webhooks WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // API Key operations
    pub async fn get_api_keys(&self) -> Result<Vec<ApiKey>> {
        let keys = sqlx::query_as::<_, ApiKey>(
            "SELECT * FROM api_keys ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(keys)
    }

    pub async fn get_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>> {
        let key = sqlx::query_as::<_, ApiKey>(
            "SELECT * FROM api_keys WHERE key_hash = $1 AND enabled = true"
        )
        .bind(key_hash)
        .fetch_optional(&self.pool)
        .await?;

        Ok(key)
    }

    pub async fn insert_api_key(&self, api_key: &ApiKey) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO api_keys (
                id, name, key_hash, permissions, enabled, rate_limit,
                request_count, last_used, expires_at, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#
        )
        .bind(&api_key.id)
        .bind(&api_key.name)
        .bind(&api_key.key_hash)
        .bind(&api_key.permissions)
        .bind(api_key.enabled)
        .bind(api_key.rate_limit)
        .bind(api_key.request_count)
        .bind(api_key.last_used)
        .bind(api_key.expires_at)
        .bind(api_key.created_at)
        .bind(api_key.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // System statistics
    pub async fn get_system_stats(&self) -> Result<SystemStats> {
        let total_transactions: i64 = sqlx::query("SELECT COUNT(*) FROM transactions")
            .fetch_one(&self.pool)
            .await?
            .get(0);

        let total_addresses: i64 = sqlx::query("SELECT COUNT(*) FROM addresses")
            .fetch_one(&self.pool)
            .await?
            .get(0);

        let current_block: i64 = sqlx::query("SELECT COALESCE(MAX(number), 0) FROM blocks")
            .fetch_one(&self.pool)
            .await?
            .get(0);

        let active_webhooks: i64 = sqlx::query("SELECT COUNT(*) FROM webhooks WHERE enabled = true")
            .fetch_one(&self.pool)
            .await?
            .get(0);

        // Calculate success rate from recent transactions
        let success_rate: f64 = sqlx::query(
            r#"
            SELECT 
                CASE 
                    WHEN COUNT(*) = 0 THEN 0.0
                    ELSE (COUNT(*) FILTER (WHERE status = 'success')::float / COUNT(*)::float) * 100.0
                END
            FROM transactions 
            WHERE created_at > NOW() - INTERVAL '24 hours'
            "#
        )
        .fetch_one(&self.pool)
        .await?
        .get(0);

        Ok(SystemStats {
            total_transactions,
            total_addresses,
            current_block,
            scan_speed: 20.0, // TODO: Calculate from actual scan data
            active_webhooks,
            websocket_connections: 0, // TODO: Get from WebSocket manager
            api_requests_today: 0, // TODO: Implement request tracking
            success_rate,
            uptime: 0, // TODO: Calculate from service start time
        })
    }
}

