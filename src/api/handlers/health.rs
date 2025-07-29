use axum::response::Json;
use serde::Serialize;

use crate::api::{ApiResult, ApiResponse};

#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub version: String,
    pub timestamp: String,
}

/// Health check endpoint
pub async fn health_check() -> ApiResult<HealthStatus> {
    let health = HealthStatus {
        status: "healthy".to_string(),
        version: "2.0.0".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    Ok(Json(ApiResponse::success(health)))
}

