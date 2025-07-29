use axum::{
    extract::{Query, Path, State},
    response::Json,
};
use serde_json::Value;

use crate::api::{ApiState, ApiResult, ApiResponse, TransactionQuery, MultiAddressQuery, PaginationInfo};
use crate::core::models::Transaction;

/// List transactions with optional filtering
pub async fn list_transactions(
    State(state): State<ApiState>,
    Query(query): Query<TransactionQuery>,
) -> ApiResult<Vec<Transaction>> {
    let page = query.pagination.page.unwrap_or(1);
    let limit = query.pagination.limit.unwrap_or(20);
    
    // Validate pagination parameters
    if page == 0 || limit == 0 || limit > 100 {
        return Ok(Json(ApiResponse::error("Invalid pagination parameters")));
    }
    
    match state.db.get_transactions(&query, page, limit).await {
        Ok((transactions, total)) => {
            let total_pages = (total as f64 / limit as f64).ceil() as u32;
            let pagination = PaginationInfo {
                page,
                limit,
                total,
                total_pages,
            };
            Ok(Json(ApiResponse::success_with_pagination(transactions, pagination)))
        }
        Err(e) => Ok(Json(ApiResponse::error(&format!("Database error: {}", e)))),
    }
}

/// Get a specific transaction by hash
pub async fn get_transaction(
    State(state): State<ApiState>,
    Path(hash): Path<String>,
) -> ApiResult<Transaction> {
    // Validate transaction hash format
    if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Ok(Json(ApiResponse::error("Invalid transaction hash format")));
    }
    
    match state.db.get_transaction_by_hash(&hash).await {
        Ok(Some(transaction)) => Ok(Json(ApiResponse::success(transaction))),
        Ok(None) => Ok(Json(ApiResponse::error("Transaction not found"))),
        Err(e) => Ok(Json(ApiResponse::error(&format!("Database error: {}", e)))),
    }
}

/// Multi-address query endpoint
pub async fn multi_address_query(
    State(state): State<ApiState>,
    Json(query): Json<MultiAddressQuery>,
) -> ApiResult<Vec<Transaction>> {
    // Validate addresses
    if query.addresses.is_empty() {
        return Ok(Json(ApiResponse::error("At least one address is required")));
    }
    
    if query.addresses.len() > 100 {
        return Ok(Json(ApiResponse::error("Maximum 100 addresses allowed")));
    }
    
    // Validate address formats
    for address in &query.addresses {
        if !is_valid_tron_address(address) {
            return Ok(Json(ApiResponse::error(&format!("Invalid address format: {}", address))));
        }
    }
    
    let page = query.pagination.page.unwrap_or(1);
    let limit = query.pagination.limit.unwrap_or(20);
    
    match state.db.get_transactions_by_addresses(&query.addresses, &query, page, limit).await {
        Ok((transactions, total)) => {
            let total_pages = (total as f64 / limit as f64).ceil() as u32;
            let pagination = PaginationInfo {
                page,
                limit,
                total,
                total_pages,
            };
            Ok(Json(ApiResponse::success_with_pagination(transactions, pagination)))
        }
        Err(e) => Ok(Json(ApiResponse::error(&format!("Database error: {}", e)))),
    }
}

/// Validate Tron address format
fn is_valid_tron_address(address: &str) -> bool {
    // Basic Tron address validation
    // Tron addresses start with 'T' and are 34 characters long
    address.len() == 34 && address.starts_with('T') && address.chars().all(|c| c.is_ascii_alphanumeric())
}

