// 本地内存缓存服务
// 
// 提供高性能的本地内存缓存，使用 moka 库实现
// 避免了 Redis 的网络开销和运维复杂性

use crate::core::{config::Config, models::*};
use anyhow::Result;
use moka::future::Cache;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{info, debug};

/// 缓存键前缀
const CACHE_PREFIX_TRANSACTION: &str = "tx:";
const CACHE_PREFIX_ADDRESS: &str = "addr:";
const CACHE_PREFIX_BLOCK: &str = "block:";
const CACHE_PREFIX_MULTI_ADDR: &str = "multi_addr:";
const CACHE_PREFIX_STATS: &str = "stats:";
const CACHE_PREFIX_API_KEY: &str = "api_key:";

/// 缓存 TTL（秒）
const TTL_TRANSACTION: u64 = 3600;        // 1小时
const TTL_ADDRESS: u64 = 1800;            // 30分钟
const TTL_BLOCK: u64 = 7200;              // 2小时
const TTL_MULTI_ADDR: u64 = 900;          // 15分钟
const TTL_STATS: u64 = 300;               // 5分钟
const TTL_API_KEY: u64 = 1800;            // 30分钟

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
    filters: TransactionQuery,
    pagination: Pagination,
}

/// 本地缓存服务
#[derive(Clone)]
pub struct LocalCacheService {
    config: Config,
    // 使用 String 作为值类型，所有数据序列化为 JSON
    cache: Arc<Cache<String, String>>,
    statistics: Arc<RwLock<CacheStatistics>>,
    start_time: std::time::Instant,
}

impl LocalCacheService {
    /// 创建新的本地缓存服务
    pub async fn new(config: Config) -> Result<Self> {
        // 配置缓存
        let cache = Cache::builder()
            // 最大缓存条目数，可以根据内存大小调整
            .max_capacity(100_000)
            // 基于时间的过期策略
            .time_to_live(Duration::from_secs(3600))
            // 空闲时间过期
            .time_to_idle(Duration::from_secs(1800))
            .build();

        info!("Initialized local cache service with max capacity: 100,000 items");

        Ok(Self {
            config,
            cache: Arc::new(cache),
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

    /// 创建禁用的缓存服务（用于错误处理）
    pub fn new_disabled() -> Self {
        Self {
            config: Config::default(),
            cache: Arc::new(Cache::new(0)),
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
        }
    }

    /// 缓存交易数据
    pub async fn cache_transaction(&self, transaction: &Transaction) -> Result<()> {
        let key = format!("{}{}", CACHE_PREFIX_TRANSACTION, transaction.hash);
        let value = serde_json::to_string(transaction)?;

        self.cache.insert(key, value).await;
        debug!("Cached transaction: {}", transaction.hash);

        Ok(())
    }

    /// 获取缓存的交易数据
    pub async fn get_cached_transaction(&self, hash: &str) -> Result<Option<Transaction>> {
        self.update_request_stats().await;

        let key = format!("{}{}", CACHE_PREFIX_TRANSACTION, hash);

        match self.cache.get(&key).await {
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
        filters: &TransactionQuery,
        pagination: &Pagination,
        transactions: &[Transaction],
    ) -> Result<()> {
        let cache_key = self.generate_address_cache_key(address, filters, pagination);
        let value = serde_json::to_string(transactions)?;

        self.cache.insert(cache_key, value).await;
        debug!("Cached address transactions: {} ({})", address, transactions.len());

        Ok(())
    }

    /// 获取缓存的地址交易列表
    pub async fn get_cached_address_transactions(
        &self,
        address: &str,
        filters: &TransactionQuery,
        pagination: &Pagination,
    ) -> Result<Option<Vec<Transaction>>> {
        self.update_request_stats().await;

        let cache_key = self.generate_address_cache_key(address, filters, pagination);

        match self.cache.get(&cache_key).await {
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
        filters: &TransactionQuery,
        pagination: &Pagination,
        result: &MultiAddressQueryResult,
    ) -> Result<()> {
        let cache_key = self.generate_multi_address_cache_key(addresses, filters, pagination);
        let value = serde_json::to_string(result)?;

        self.cache.insert(cache_key, value).await;
        debug!("Cached multi-address query: {} addresses", addresses.len());

        Ok(())
    }

    /// 获取缓存的多地址查询结果
    pub async fn get_cached_multi_address_query(
        &self,
        addresses: &[String],
        filters: &TransactionQuery,
        pagination: &Pagination,
    ) -> Result<Option<MultiAddressQueryResult>> {
        self.update_request_stats().await;

        let cache_key = self.generate_multi_address_cache_key(addresses, filters, pagination);

        match self.cache.get(&cache_key).await {
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
        let key = format!("{}{}", CACHE_PREFIX_BLOCK, block.number);
        let value = serde_json::to_string(block)?;

        self.cache.insert(key, value).await;
        debug!("Cached block: {}", block.number);

        Ok(())
    }

    /// 获取缓存的区块数据
    pub async fn get_cached_block(&self, block_number: u64) -> Result<Option<Block>> {
        self.update_request_stats().await;

        let key = format!("{}{}", CACHE_PREFIX_BLOCK, block_number);

        match self.cache.get(&key).await {
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
        let key = format!("{}{}", CACHE_PREFIX_STATS, stats_type);
        let value = serde_json::to_string(data)?;

        self.cache.insert(key, value).await;
        debug!("Cached statistics: {}", stats_type);

        Ok(())
    }

    /// 获取缓存的统计数据
    pub async fn get_cached_statistics(&self, stats_type: &str) -> Result<Option<serde_json::Value>> {
        self.update_request_stats().await;

        let key = format!("{}{}", CACHE_PREFIX_STATS, stats_type);

        match self.cache.get(&key).await {
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
        let key = format!("{}{}", CACHE_PREFIX_API_KEY, api_key.key_hash);
        let value = serde_json::to_string(api_key)?;

        self.cache.insert(key, value).await;
        debug!("Cached API key: {}", api_key.id);

        Ok(())
    }

    /// 获取缓存的 API 密钥信息
    pub async fn get_cached_api_key(&self, key: &str) -> Result<Option<ApiKey>> {
        self.update_request_stats().await;

        let cache_key = format!("{}{}", CACHE_PREFIX_API_KEY, key);

        match self.cache.get(&cache_key).await {
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
        // moka 不支持模式匹配，需要手动实现
        // 这里简化处理，只支持前缀匹配
        let mut count = 0;
        
        // 由于 moka 不提供遍历所有键的方法，我们需要维护一个键集合
        // 或者使用其他策略。这里我们简化处理，只提供清空功能
        if pattern == "*" {
            self.cache.invalidate_all();
            count = self.cache.entry_count() as u64;
            info!("Invalidated all cache entries");
        } else {
            info!("Pattern-based invalidation not supported in local cache, use clear_all instead");
        }
        
        Ok(count)
    }

    /// 清空所有缓存
    pub async fn clear_all(&self) -> Result<()> {
        self.clear_all_cache().await
    }

    /// 清空所有缓存
    pub async fn clear_all_cache(&self) -> Result<()> {
        self.cache.invalidate_all();
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
        let mut stats = self.statistics.write().await;
        
        // 更新缓存条目数
        stats.total_keys = self.cache.entry_count() as u64;
        stats.uptime_seconds = self.start_time.elapsed().as_secs();
        
        // 计算命中率
        if stats.total_requests > 0 {
            stats.hit_rate = (stats.cache_hits as f64 / stats.total_requests as f64) * 100.0;
        }
        
        // 估算内存使用（粗略估计）
        // 假设每个条目平均 1KB
        stats.memory_usage_bytes = stats.total_keys * 1024;

        Ok(stats.clone())
    }

    /// 生成地址缓存键
    fn generate_address_cache_key(
        &self,
        address: &str,
        filters: &TransactionQuery,
        pagination: &Pagination,
    ) -> String {
        let filters_hash = self.hash_filters(filters);
        let pagination_hash = self.hash_pagination(pagination);
        
        format!("{}{}:{}:{}", CACHE_PREFIX_ADDRESS, address, filters_hash, pagination_hash)
    }

    /// 生成多地址缓存键
    fn generate_multi_address_cache_key(
        &self,
        addresses: &[String],
        filters: &TransactionQuery,
        pagination: &Pagination,
    ) -> String {
        let mut sorted_addresses = addresses.to_vec();
        sorted_addresses.sort();
        
        let addresses_hash = self.hash_string(&sorted_addresses.join(","));
        let filters_hash = self.hash_filters(filters);
        let pagination_hash = self.hash_pagination(pagination);
        
        format!("{}{}:{}:{}", CACHE_PREFIX_MULTI_ADDR, addresses_hash, filters_hash, pagination_hash)
    }

    /// 计算过滤条件哈希
    fn hash_filters(&self, filters: &TransactionQuery) -> String {
        let filter_str = format!(
            "{}:{}:{}:{}:{}:{}",
            filters.token_address.as_deref().unwrap_or(""),
            "",  // status字段暂时不使用
            "",  // min_amount字段暂时不使用  
            "",  // max_amount字段暂时不使用
            filters.start_time.map(|t| t.timestamp()).unwrap_or(0),
            filters.end_time.map(|t| t.timestamp()).unwrap_or(0),
        );
        
        self.hash_string(&filter_str)
    }

    /// 计算分页参数哈希
    fn hash_pagination(&self, pagination: &Pagination) -> String {
        let pagination_str = format!("{}:{}", 
            pagination.page.unwrap_or(1), 
            pagination.limit.unwrap_or(20));
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
        // 本地缓存始终健康
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

    /// 获取统计信息
    pub async fn get_statistics(&self) -> Result<serde_json::Value> {
        let stats = self.statistics.read().await;
        Ok(serde_json::json!({
            "total_requests": stats.total_requests,
            "cache_hits": stats.cache_hits,
            "cache_misses": stats.cache_misses,
            "hit_rate": stats.hit_rate,
            "total_keys": self.cache.entry_count(),
            "memory_usage_bytes": stats.memory_usage_bytes,
            "uptime_seconds": self.start_time.elapsed().as_secs()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_local_cache_service() {
        let config = Config::default();
        let service = LocalCacheService::new(config).await.unwrap();
        
        // 测试健康检查
        assert!(service.health_check().await.unwrap());
        
        // 测试缓存和获取交易
        let transaction = Transaction {
            id: uuid::Uuid::new_v4(),
            hash: "test_hash".to_string(),
            block_number: 12345,
            block_hash: "test_block_hash".to_string(),
            transaction_index: 0,
            from_address: "from".to_string(),
            to_address: "to".to_string(),
            value: "1000".to_string(),
            token_address: None,
            token_symbol: None,
            token_decimals: None,
            gas_used: None,
            gas_price: None,
            status: TransactionStatus::Success,
            timestamp: chrono::Utc::now(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        // 缓存交易
        service.cache_transaction(&transaction).await.unwrap();
        
        // 获取缓存的交易
        let cached = service.get_cached_transaction("test_hash").await.unwrap();
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().hash, "test_hash");
        
        // 测试缓存未命中
        let miss = service.get_cached_transaction("non_existent").await.unwrap();
        assert!(miss.is_none());
        
        // 测试统计信息
        let stats = service.get_cache_statistics().await.unwrap();
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.hit_rate, 50.0);
    }

    #[test]
    fn test_cache_key_generation() {
        let config = Config::default();
        let service = LocalCacheService {
            config: config.clone(),
            cache: Arc::new(Cache::new(100)),
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
        };

        // 测试缓存键生成
        let filters = TransactionQuery::default();
        let pagination = Pagination { 
            page: Some(1), 
            limit: Some(20) 
        };

        let key1 = service.generate_address_cache_key("addr1", &filters, &pagination);
        let key2 = service.generate_address_cache_key("addr2", &filters, &pagination);

        assert_ne!(key1, key2);
        assert!(key1.starts_with(CACHE_PREFIX_ADDRESS));
    }
}