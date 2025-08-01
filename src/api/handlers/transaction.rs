// 交易相关的 API 处理器
// 
// 处理交易查询、统计等相关请求，包括多地址批量查询

use crate::core::models::*;
// Removed unused import: CacheService
use axum::{
    extract::{Query, State, Path},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn, error, debug};

// Use the unified AdminAppState
use crate::api::handlers::admin::AdminAppState;

/// 交易查询参数
#[derive(Debug, Deserialize)]
pub struct TransactionQueryParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub token: Option<String>,
    pub status: Option<String>,
    pub min_amount: Option<String>,
    pub max_amount: Option<String>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
}

/// 多地址查询参数
#[derive(Debug, Deserialize)]
pub struct MultiAddressQueryParams {
    pub addresses: String, // 逗号分隔的地址列表
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub token: Option<String>,
    pub status: Option<String>,
    pub min_amount: Option<String>,
    pub max_amount: Option<String>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub group_by_address: Option<bool>, // 是否按地址分组
}

/// 交易列表响应
#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionListResponse {
    pub transactions: Vec<Transaction>,
    pub total_count: u64,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
    pub query_time_ms: u64,
    pub cache_hit: bool,
}

/// 多地址查询响应
// Use the unified MultiAddressQueryResult from models, with additional fields for API response
#[derive(Debug, Serialize, Deserialize)]
pub struct MultiAddressQueryResponse {
    pub transactions: Vec<Transaction>,
    pub total_count: u64,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
    pub has_more: bool,
    pub address_stats: HashMap<String, AddressStatistics>,
    pub addresses_queried: u32,
    pub query_time_ms: u64,
    pub cache_hit: bool,
}

// Use the unified AddressStatistics from models
pub use crate::core::models::AddressStatistics;

/// 获取交易列表
pub async fn get_transactions(
    Query(params): Query<TransactionQueryParams>,
    State(state): State<Arc<AdminAppState>>,
) -> Result<Json<TransactionListResponse>, StatusCode> {
    let start_time = std::time::Instant::now();
    
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20).min(100); // 最大限制100

    let filters = TransactionQuery {
        address: None,
        hash: None,
        block_number: None,
        token_address: None,
        token: params.token,
        status: params.status.and_then(|s| match s.as_str() {
            "success" => Some(TransactionStatus::Success),
            "failed" => Some(TransactionStatus::Failed),
            "pending" => Some(TransactionStatus::Pending),
            _ => None,
        }),
        min_amount: params.min_amount,
        max_amount: params.max_amount,
        start_time: params.start_time.map(|t| chrono::DateTime::from_timestamp(t, 0).unwrap()),
        end_time: params.end_time.map(|t| chrono::DateTime::from_timestamp(t, 0).unwrap()),
        limit: Some(limit),
        offset: Some((page - 1) * limit),
        sort_by: None,
        sort_order: None,
    };

    let pagination = Pagination { page: Some(page), limit: Some(limit) };

    // 尝试从缓存获取
    let cache_key = format!("transactions:{}:{}", 
        serde_json::to_string(&filters).unwrap_or_default(),
        serde_json::to_string(&pagination).unwrap_or_default()
    );

    let mut cache_hit = false;
    let result = if let Ok(Some(cached_data)) = state.cache.get_cached_statistics(&cache_key).await {
        cache_hit = true;
        debug!("Cache hit for transactions query");
        
        // 解析缓存数据
        if let Ok(cached_response) = serde_json::from_value::<TransactionListResponse>(cached_data) {
            Ok((cached_response.transactions, cached_response.total_count))
        } else {
            // 缓存数据格式错误，从数据库查询
            state.db.get_transactions(&filters, &pagination).await
        }
    } else {
        // 从数据库查询
        state.db.get_transactions(&filters, &pagination).await
    };

    match result {
        Ok((transactions, total_count)) => {
            let total_pages = (total_count as f64 / limit as f64).ceil() as u32;
            let query_time_ms = start_time.elapsed().as_millis() as u64;
            
            let response = TransactionListResponse {
                transactions,
                total_count,
                page,
                limit,
                total_pages,
                query_time_ms,
                cache_hit,
            };

            // 缓存结果（如果不是从缓存获取的）
            if !cache_hit {
                if let Err(e) = state.cache.cache_statistics(&cache_key, &serde_json::to_value(&response).unwrap()).await {
                    warn!("Failed to cache transactions query result: {}", e);
                }
            }
            
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to get transactions: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 获取单个交易详情
pub async fn get_transaction(
    Path(hash): Path<String>,
    State(state): State<Arc<AdminAppState>>,
) -> Result<Json<Transaction>, StatusCode> {
    // 尝试从缓存获取
    if let Ok(Some(cached_transaction)) = state.cache.get_cached_transaction(&hash).await {
        debug!("Cache hit for transaction: {}", hash);
        return Ok(Json(cached_transaction));
    }

    match state.db.get_transaction_by_hash(&hash).await {
        Ok(Some(transaction)) => {
            // 缓存交易数据
            if let Err(e) = state.cache.cache_transaction(&transaction).await {
                warn!("Failed to cache transaction {}: {}", hash, e);
            }
            
            Ok(Json(transaction))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get transaction {}: {}", hash, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 获取地址交易列表
pub async fn get_address_transactions(
    Path(address): Path<String>,
    Query(params): Query<TransactionQueryParams>,
    State(state): State<Arc<AdminAppState>>,
) -> Result<Json<TransactionListResponse>, StatusCode> {
    let start_time = std::time::Instant::now();
    
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20).min(100);

    let filters = TransactionQuery {
        address: None,
        hash: None,
        block_number: None,
        token_address: None,
        token: params.token,
        status: params.status.and_then(|s| match s.as_str() {
            "success" => Some(TransactionStatus::Success),
            "failed" => Some(TransactionStatus::Failed),
            "pending" => Some(TransactionStatus::Pending),
            _ => None,
        }),
        min_amount: params.min_amount,
        max_amount: params.max_amount,
        start_time: params.start_time.map(|t| chrono::DateTime::from_timestamp(t, 0).unwrap()),
        end_time: params.end_time.map(|t| chrono::DateTime::from_timestamp(t, 0).unwrap()),
        limit: Some(limit),
        offset: Some((page - 1) * limit),
        sort_by: None,
        sort_order: None,
    };

    let pagination = Pagination { page: Some(page), limit: Some(limit) };

    // 尝试从缓存获取
    let mut cache_hit = false;
    let result = if let Ok(Some(cached_transactions)) = state.cache.get_cached_address_transactions(&address, &filters, &pagination).await {
        cache_hit = true;
        debug!("Cache hit for address transactions: {}", address);
        
        // 获取总数（这里简化处理，实际应该也缓存总数）
        match state.db.get_address_transaction_count(&address, &filters).await {
            Ok(count) => Ok((cached_transactions, count)),
            Err(e) => {
                warn!("Failed to get cached transaction count: {}", e);
                state.db.get_address_transactions(&address, &filters, &pagination).await
            }
        }
    } else {
        // 从数据库查询
        state.db.get_address_transactions(&address, &filters, &pagination).await
    };

    match result {
        Ok((transactions, total_count)) => {
            let total_pages = (total_count as f64 / limit as f64).ceil() as u32;
            let query_time_ms = start_time.elapsed().as_millis() as u64;
            
            let response = TransactionListResponse {
                transactions: transactions.clone(),
                total_count,
                page,
                limit,
                total_pages,
                query_time_ms,
                cache_hit,
            };

            // 缓存结果（如果不是从缓存获取的）
            if !cache_hit {
                if let Err(e) = state.cache.cache_address_transactions(&address, &filters, &pagination, &transactions).await {
                    warn!("Failed to cache address transactions: {}", e);
                }
            }
            
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to get address transactions for {}: {}", address, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 多地址批量查询
pub async fn get_multi_address_transactions(
    Query(params): Query<MultiAddressQueryParams>,
    State(state): State<Arc<AdminAppState>>,
) -> Result<Json<MultiAddressQueryResponse>, StatusCode> {
    let start_time = std::time::Instant::now();
    
    // 解析地址列表
    let addresses: Vec<String> = params.addresses
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // 验证地址数量限制
    if addresses.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    if addresses.len() > 100 {
        error!("Too many addresses in multi-address query: {}", addresses.len());
        return Err(StatusCode::BAD_REQUEST);
    }

    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20).min(100);
    let group_by_address = params.group_by_address.unwrap_or(false);

    let filters = TransactionQuery {
        address: None,
        hash: None,
        block_number: None,
        token_address: None,
        token: params.token,
        status: params.status.and_then(|s| match s.as_str() {
            "success" => Some(TransactionStatus::Success),
            "failed" => Some(TransactionStatus::Failed),
            "pending" => Some(TransactionStatus::Pending),
            _ => None,
        }),
        min_amount: params.min_amount,
        max_amount: params.max_amount,
        start_time: params.start_time.map(|t| chrono::DateTime::from_timestamp(t, 0).unwrap()),
        end_time: params.end_time.map(|t| chrono::DateTime::from_timestamp(t, 0).unwrap()),
        limit: Some(limit),
        offset: Some((page - 1) * limit),
        sort_by: None,
        sort_order: None,
    };

    let pagination = Pagination { page: Some(page), limit: Some(limit) };

    info!("Multi-address query for {} addresses", addresses.len());

    // 尝试从缓存获取
    let mut cache_hit = false;
    let result = if let Ok(Some(cached_result)) = state.cache.get_cached_multi_address_query(&addresses, &filters, &pagination).await {
        cache_hit = true;
        debug!("Cache hit for multi-address query: {} addresses", addresses.len());
        Ok(cached_result)
    } else {
        // 从数据库查询
        cache_hit = false;
        state.db.get_multi_address_transactions(&addresses, &filters, &pagination, group_by_address).await
    };

    match result {
        Ok(query_result) => {
            let total_pages = (query_result.total_count as f64 / limit as f64).ceil() as u32;
            let query_time_ms = start_time.elapsed().as_millis() as u64;
            
            let response = MultiAddressQueryResponse {
                transactions: query_result.transactions.clone(),
                total_count: query_result.total_count,
                page,
                limit,
                total_pages,
                has_more: page < total_pages,
                address_stats: query_result.address_stats.clone(),
                addresses_queried: addresses.len() as u32,
                query_time_ms,
                cache_hit,
            };

            // 缓存结果（如果不是从缓存获取的）
            if !cache_hit {
                if let Err(e) = state.cache.cache_multi_address_query(&addresses, &filters, &pagination, &query_result).await {
                    warn!("Failed to cache multi-address query result: {}", e);
                }
            }

            info!("Multi-address query completed: {} transactions, {} ms", 
                  query_result.transactions.len(), query_time_ms);
            
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to execute multi-address query: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 获取地址统计信息
pub async fn get_address_statistics(
    Path(address): Path<String>,
    State(state): State<Arc<AdminAppState>>,
) -> Result<Json<AddressStatistics>, StatusCode> {
    match state.db.get_address_statistics(&address).await {
        Ok(stats) => Ok(Json(stats)),
        Err(e) => {
            error!("Failed to get address statistics for {}: {}", address, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 获取地址信息
pub async fn get_address_info(
    Path(address): Path<String>,
    State(state): State<Arc<AdminAppState>>,
) -> Result<Json<AddressStatistics>, StatusCode> {
    match state.db.get_address_statistics(&address).await {
        Ok(stats) => Ok(Json(stats)),
        Err(e) => {
            error!("Failed to get address info for {}: {}", address, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 批量获取地址统计信息
pub async fn get_batch_address_statistics(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<AdminAppState>>,
) -> Result<Json<HashMap<String, AddressStatistics>>, StatusCode> {
    let addresses_param = params.get("addresses").ok_or(StatusCode::BAD_REQUEST)?;
    
    let addresses: Vec<String> = addresses_param
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if addresses.is_empty() || addresses.len() > 50 {
        return Err(StatusCode::BAD_REQUEST);
    }

    match state.db.get_batch_address_statistics(&addresses).await {
        Ok(stats_map) => Ok(Json(stats_map)),
        Err(e) => {
            error!("Failed to get batch address statistics: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 搜索交易
pub async fn search_transactions(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Arc<AdminAppState>>,
) -> Result<Json<TransactionListResponse>, StatusCode> {
    let start_time = std::time::Instant::now();
    
    let query = params.get("q").ok_or(StatusCode::BAD_REQUEST)?;
    let page = params.get("page").and_then(|p| p.parse().ok()).unwrap_or(1);
    let limit = params.get("limit").and_then(|l| l.parse().ok()).unwrap_or(20).min(100);

    let pagination = Pagination { page: Some(page), limit: Some(limit) };

    match state.db.search_transactions(query, &pagination).await {
        Ok((transactions, total_count)) => {
            let total_pages = (total_count as f64 / limit as f64).ceil() as u32;
            let query_time_ms = start_time.elapsed().as_millis() as u64;
            
            Ok(Json(TransactionListResponse {
                transactions,
                total_count,
                page,
                limit,
                total_pages,
                query_time_ms,
                cache_hit: false, // 搜索结果不缓存
            }))
        }
        Err(e) => {
            error!("Failed to search transactions: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

