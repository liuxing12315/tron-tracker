// 缓存服务
// 
// 提供 Redis 缓存功能，优化数据库查询性能

use crate::core::{config::Config, models::*};
use anyhow::{Result, anyhow};
use redis::{Client, Connection, Commands, AsyncCommands};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};

/// 缓存键前缀
const CACHE_PREFIX_TRANSACTION: &str = "tx:";
const CACHE_PREFIX_ADDRESS: &str = "addr:";
const CACHE_PREFIX_BLOCK: &str = "block:";
const CACHE_PREFIX_MULTI_ADDR: &str = "multi_addr:";
const CACHE_PREFIX_STATS: &str = "stats:";
const CACHE_PREFIX_API_KEY: &str = "api_key:";

/// 缓存 TTL（秒）
const TTL_TRANSACTION: usize = 3600;        // 1小时
const TTL_ADDRESS: usize = 1800;            // 30分钟
const TTL_BLOCK: usize = 7200;              // 2小时
const TTL_MULTI_ADDR: usize = 900;          // 15分钟
const TTL_STATS: usize = 300;               // 5分钟
const TTL_API_KEY: usize = 1800;            // 30分钟

/// 缓存统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStatistics {
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub hit_rate: f64,
    pub total_keys: u64,
    pub memory_usage_bytes: u64,
    pub uptime_seconds: u64,
}

/// 多地址查询缓存键
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MultiAddressQueryKey {
    addresses: Vec<String>,
    filters: TransactionFilters,
    pagination: PaginationParams,
}

/// 缓存服务
pub struct CacheService {
    config: Config,
    redis_client: Client,
    connection_pool: Arc<RwLock<Vec<redis::aio::Connection>>>,
    statistics: Arc<RwLock<CacheStatistics>>,
    start_time: std::time::Instant,
}

impl CacheService {
    /// 创建新的缓存服务
    pub async fn new(config: Config) -> Result<Self> {
        let redis_url = format!(
            "redis://{}:{}/{}",
            config.redis.host,
            config.redis.port,
            config.redis.database
        );

        let redis_client = Client::open(redis_url)?;

        // 测试连接
        let mut conn = redis_client.get_async_connection().await?;
        let _: String = conn.ping().await?;

        info!("Connected to Redis at {}:{}", config.redis.host, config.redis.port);

        Ok(Self {
            config,
            redis_client,
            connection_pool: Arc::new(RwLock::new(Vec::new())),
            statistics: Arc::new(RwLock::new(CacheStatistics {
                total_requests: 0,
                cache_hits: 0,
                cache_misses: 0,
                hit_rate: 0.0,
                total_keys: 0,
                memory_usage_bytes: 0,
                uptime_seconds: 0,
            })),
            start_time: std::time::Instant::now(),
        })
    }

    /// 获取 Redis 连接
    async fn get_connection(&self) -> Result<redis::aio::Connection> {
        self.redis_client.get_async_connection().await
            .map_err(|e| anyhow!("Failed to get Redis connection: {}", e))
    }

    /// 缓存交易数据
    pub async fn cache_transaction(&self, transaction: &Transaction) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let key = format!("{}{}", CACHE_PREFIX_TRANSACTION, transaction.hash);
        let value = serde_json::to_string(transaction)?;

        let _: () = conn.setex(&key, TTL_TRANSACTION, &value).await?;
        debug!("Cached transaction: {}", transaction.hash);

        Ok(())
    }

    /// 获取缓存的交易数据
    pub async fn get_cached_transaction(&self, hash: &str) -> Result<Option<Transaction>> {
        self.update_request_stats().await;

        let mut conn = self.get_connection().await?;
        let key = format!("{}{}", CACHE_PREFIX_TRANSACTION, hash);

        match conn.get::<_, Option<String>>(&key).await? {
            Some(value) => {
                self.update_hit_stats().await;
                let transaction: Transaction = serde_json::from_str(&value)?;
                debug!("Cache hit for transaction: {}", hash);
                Ok(Some(transaction))
            }
            None => {
                self.update_miss_stats().await;
                debug!("Cache miss for transaction: {}", hash);
                Ok(None)
            }
        }
    }

    /// 缓存地址交易列表
    pub async fn cache_address_transactions(
        &self,
        address: &str,
        filters: &TransactionFilters,
        pagination: &PaginationParams,
        transactions: &[Transaction],
    ) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let cache_key = self.generate_address_cache_key(address, filters, pagination);
        let value = serde_json::to_string(transactions)?;

        let _: () = conn.setex(&cache_key, TTL_ADDRESS, &value).await?;
        debug!("Cached address transactions: {} ({})", address, transactions.len());

        Ok(())
    }

    /// 获取缓存的地址交易列表
    pub async fn get_cached_address_transactions(
        &self,
        address: &str,
        filters: &TransactionFilters,
        pagination: &PaginationParams,
    ) -> Result<Option<Vec<Transaction>>> {
        self.update_request_stats().await;

        let mut conn = self.get_connection().await?;
        let cache_key = self.generate_address_cache_key(address, filters, pagination);

        match conn.get::<_, Option<String>>(&cache_key).await? {
            Some(value) => {
                self.update_hit_stats().await;
                let transactions: Vec<Transaction> = serde_json::from_str(&value)?;
                debug!("Cache hit for address transactions: {} ({})", address, transactions.len());
                Ok(Some(transactions))
            }
            None => {
                self.update_miss_stats().await;
                debug!("Cache miss for address transactions: {}", address);
                Ok(None)
            }
        }
    }

    /// 缓存多地址查询结果
    pub async fn cache_multi_address_query(
        &self,
        addresses: &[String],
        filters: &TransactionFilters,
        pagination: &PaginationParams,
        result: &MultiAddressQueryResult,
    ) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let cache_key = self.generate_multi_address_cache_key(addresses, filters, pagination);
        let value = serde_json::to_string(result)?;

        let _: () = conn.setex(&cache_key, TTL_MULTI_ADDR, &value).await?;
        debug!("Cached multi-address query: {} addresses", addresses.len());

        Ok(())
    }

    /// 获取缓存的多地址查询结果
    pub async fn get_cached_multi_address_query(
        &self,
        addresses: &[String],
        filters: &TransactionFilters,
        pagination: &PaginationParams,
    ) -> Result<Option<MultiAddressQueryResult>> {
        self.update_request_stats().await;

        let mut conn = self.get_connection().await?;
        let cache_key = self.generate_multi_address_cache_key(addresses, filters, pagination);

        match conn.get::<_, Option<String>>(&cache_key).await? {
            Some(value) => {
                self.update_hit_stats().await;
                let result: MultiAddressQueryResult = serde_json::from_str(&value)?;
                debug!("Cache hit for multi-address query: {} addresses", addresses.len());
                Ok(Some(result))
            }
            None => {
                self.update_miss_stats().await;
                debug!("Cache miss for multi-address query: {} addresses", addresses.len());
                Ok(None)
            }
        }
    }

    /// 缓存区块数据
    pub async fn cache_block(&self, block: &Block) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let key = format!("{}{}", CACHE_PREFIX_BLOCK, block.number);
        let value = serde_json::to_string(block)?;

        let _: () = conn.setex(&key, TTL_BLOCK, &value).await?;
        debug!("Cached block: {}", block.number);

        Ok(())
    }

    /// 获取缓存的区块数据
    pub async fn get_cached_block(&self, block_number: u64) -> Result<Option<Block>> {
        self.update_request_stats().await;

        let mut conn = self.get_connection().await?;
        let key = format!("{}{}", CACHE_PREFIX_BLOCK, block_number);

        match conn.get::<_, Option<String>>(&key).await? {
            Some(value) => {
                self.update_hit_stats().await;
                let block: Block = serde_json::from_str(&value)?;
                debug!("Cache hit for block: {}", block_number);
                Ok(Some(block))
            }
            None => {
                self.update_miss_stats().await;
                debug!("Cache miss for block: {}", block_number);
                Ok(None)
            }
        }
    }

    /// 缓存统计数据
    pub async fn cache_statistics(&self, stats_type: &str, data: &serde_json::Value) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let key = format!("{}{}", CACHE_PREFIX_STATS, stats_type);
        let value = serde_json::to_string(data)?;

        let _: () = conn.setex(&key, TTL_STATS, &value).await?;
        debug!("Cached statistics: {}", stats_type);

        Ok(())
    }

    /// 获取缓存的统计数据
    pub async fn get_cached_statistics(&self, stats_type: &str) -> Result<Option<serde_json::Value>> {
        self.update_request_stats().await;

        let mut conn = self.get_connection().await?;
        let key = format!("{}{}", CACHE_PREFIX_STATS, stats_type);

        match conn.get::<_, Option<String>>(&key).await? {
            Some(value) => {
                self.update_hit_stats().await;
                let data: serde_json::Value = serde_json::from_str(&value)?;
                debug!("Cache hit for statistics: {}", stats_type);
                Ok(Some(data))
            }
            None => {
                self.update_miss_stats().await;
                debug!("Cache miss for statistics: {}", stats_type);
                Ok(None)
            }
        }
    }

    /// 缓存 API 密钥信息
    pub async fn cache_api_key(&self, api_key: &ApiKey) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let key = format!("{}{}", CACHE_PREFIX_API_KEY, api_key.key);
        let value = serde_json::to_string(api_key)?;

        let _: () = conn.setex(&key, TTL_API_KEY, &value).await?;
        debug!("Cached API key: {}", api_key.id);

        Ok(())
    }

    /// 获取缓存的 API 密钥信息
    pub async fn get_cached_api_key(&self, key: &str) -> Result<Option<ApiKey>> {
        self.update_request_stats().await;

        let mut conn = self.get_connection().await?;
        let cache_key = format!("{}{}", CACHE_PREFIX_API_KEY, key);

        match conn.get::<_, Option<String>>(&cache_key).await? {
            Some(value) => {
                self.update_hit_stats().await;
                let api_key: ApiKey = serde_json::from_str(&value)?;
                debug!("Cache hit for API key: {}", api_key.id);
                Ok(Some(api_key))
            }
            None => {
                self.update_miss_stats().await;
                debug!("Cache miss for API key");
                Ok(None)
            }
        }
    }

    /// 使缓存失效
    pub async fn invalidate_cache(&self, pattern: &str) -> Result<u64> {
        let mut conn = self.get_connection().await?;
        
        // 获取匹配的键
        let keys: Vec<String> = conn.keys(pattern).await?;
        
        if !keys.is_empty() {
            let deleted: u64 = conn.del(&keys).await?;
            info!("Invalidated {} cache entries matching pattern: {}", deleted, pattern);
            Ok(deleted)
        } else {
            Ok(0)
        }
    }

    /// 清空所有缓存
    pub async fn clear_all_cache(&self) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let _: () = conn.flushdb().await?;
        
        info!("Cleared all cache");
        
        // 重置统计信息
        {
            let mut stats = self.statistics.write().await;
            *stats = CacheStatistics {
                total_requests: 0,
                cache_hits: 0,
                cache_misses: 0,
                hit_rate: 0.0,
                total_keys: 0,
                memory_usage_bytes: 0,
                uptime_seconds: self.start_time.elapsed().as_secs(),
            };
        }

        Ok(())
    }

    /// 获取缓存统计信息
    pub async fn get_cache_statistics(&self) -> Result<CacheStatistics> {
        let mut conn = self.get_connection().await?;
        
        // 获取 Redis 信息
        let info: String = conn.info("memory").await?;
        let memory_usage = self.parse_memory_usage(&info);
        
        let total_keys: u64 = conn.dbsize().await?;

        let mut stats = self.statistics.write().await;
        stats.total_keys = total_keys;
        stats.memory_usage_bytes = memory_usage;
        stats.uptime_seconds = self.start_time.elapsed().as_secs();
        
        // 计算命中率
        if stats.total_requests > 0 {
            stats.hit_rate = (stats.cache_hits as f64 / stats.total_requests as f64) * 100.0;
        }

        Ok(stats.clone())
    }

    /// 生成地址缓存键
    fn generate_address_cache_key(
        &self,
        address: &str,
        filters: &TransactionFilters,
        pagination: &PaginationParams,
    ) -> String {
        let filters_hash = self.hash_filters(filters);
        let pagination_hash = self.hash_pagination(pagination);
        
        format!("{}{}:{}:{}", CACHE_PREFIX_ADDRESS, address, filters_hash, pagination_hash)
    }

    /// 生成多地址缓存键
    fn generate_multi_address_cache_key(
        &self,
        addresses: &[String],
        filters: &TransactionFilters,
        pagination: &PaginationParams,
    ) -> String {
        let mut sorted_addresses = addresses.to_vec();
        sorted_addresses.sort();
        
        let addresses_hash = self.hash_string(&sorted_addresses.join(","));
        let filters_hash = self.hash_filters(filters);
        let pagination_hash = self.hash_pagination(pagination);
        
        format!("{}{}:{}:{}", CACHE_PREFIX_MULTI_ADDR, addresses_hash, filters_hash, pagination_hash)
    }

    /// 计算过滤条件哈希
    fn hash_filters(&self, filters: &TransactionFilters) -> String {
        let filter_str = format!(
            "{}:{}:{}:{}:{}:{}",
            filters.token.as_deref().unwrap_or(""),
            filters.status.as_ref().map(|s| format!("{:?}", s)).unwrap_or_default(),
            filters.min_amount.as_deref().unwrap_or(""),
            filters.max_amount.as_deref().unwrap_or(""),
            filters.start_time.map(|t| t.timestamp()).unwrap_or(0),
            filters.end_time.map(|t| t.timestamp()).unwrap_or(0),
        );
        
        self.hash_string(&filter_str)
    }

    /// 计算分页参数哈希
    fn hash_pagination(&self, pagination: &PaginationParams) -> String {
        let pagination_str = format!("{}:{}", pagination.page, pagination.limit);
        self.hash_string(&pagination_str)
    }

    /// 计算字符串哈希
    fn hash_string(&self, input: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// 解析内存使用量
    fn parse_memory_usage(&self, info: &str) -> u64 {
        for line in info.lines() {
            if line.starts_with("used_memory:") {
                if let Some(value) = line.split(':').nth(1) {
                    return value.parse().unwrap_or(0);
                }
            }
        }
        0
    }

    /// 更新请求统计
    async fn update_request_stats(&self) {
        let mut stats = self.statistics.write().await;
        stats.total_requests += 1;
    }

    /// 更新命中统计
    async fn update_hit_stats(&self) {
        let mut stats = self.statistics.write().await;
        stats.cache_hits += 1;
    }

    /// 更新未命中统计
    async fn update_miss_stats(&self) {
        let mut stats = self.statistics.write().await;
        stats.cache_misses += 1;
    }

    /// 健康检查
    pub async fn health_check(&self) -> Result<bool> {
        let mut conn = self.get_connection().await?;
        let _: String = conn.ping().await?;
        Ok(true)
    }

    /// 预热缓存
    pub async fn warm_up_cache(&self, addresses: &[String]) -> Result<()> {
        info!("Warming up cache for {} addresses", addresses.len());
        
        // 这里可以预加载常用的查询结果
        for address in addresses {
            debug!("Warming up cache for address: {}", address);
            // 预加载逻辑...
        }
        
        info!("Cache warm-up completed");
        Ok(())
    }
}

/// 多地址查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiAddressQueryResult {
    pub transactions: Vec<Transaction>,
    pub total_count: u64,
    pub address_stats: HashMap<String, AddressStatistics>,
    pub query_time_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Config;

    #[tokio::test]
    async fn test_cache_service_creation() {
        let config = Config::default();
        
        // 只有在 Redis 可用时才运行测试
        if let Ok(service) = CacheService::new(config).await {
            assert!(service.health_check().await.unwrap());
        }
    }

    #[test]
    fn test_cache_key_generation() {
        let config = Config::default();
        let service = CacheService::new(config).await.unwrap();

        let filters = TransactionFilters::default();
        let pagination = PaginationParams { page: 1, limit: 20 };

        let key1 = service.generate_address_cache_key("addr1", &filters, &pagination);
        let key2 = service.generate_address_cache_key("addr2", &filters, &pagination);

        assert_ne!(key1, key2);
        assert!(key1.starts_with(CACHE_PREFIX_ADDRESS));
    }

    #[test]
    fn test_hash_functions() {
        let config = Config::default();
        let service = CacheService::new(config).await.unwrap();

        let hash1 = service.hash_string("test");
        let hash2 = service.hash_string("test");
        let hash3 = service.hash_string("different");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}

