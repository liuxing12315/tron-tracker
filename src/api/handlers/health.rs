use axum::response::Json;
use serde::Serialize;

use crate::api::{ApiResult, ApiResponse};

#[derive(Debug, Serialize, serde::Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check_response() {
        let result = health_check().await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        let health_data = response.0.data.as_ref().unwrap();
        
        assert_eq!(health_data.status, "healthy");
        assert_eq!(health_data.version, "2.0.0");
        assert!(!health_data.timestamp.is_empty());
        
        // Verify timestamp is valid RFC3339 format
        let parsed_time = chrono::DateTime::parse_from_rfc3339(&health_data.timestamp);
        assert!(parsed_time.is_ok());
    }

    #[test]
    fn test_health_status_serialization() {
        let status = HealthStatus {
            status: "ok".to_string(),
            version: "3.0.0".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"status\":\"ok\""));
        assert!(json.contains("\"version\":\"3.0.0\""));
        assert!(json.contains("\"timestamp\":\"2024-01-01T00:00:00Z\""));

        // Test deserialization
        let deserialized: HealthStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status.status, deserialized.status);
        assert_eq!(status.version, deserialized.version);
        assert_eq!(status.timestamp, deserialized.timestamp);
    }

    #[test]
    fn test_health_status_debug_format() {
        let status = HealthStatus {
            status: "healthy".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let debug_output = format!("{:?}", status);
        assert!(debug_output.contains("HealthStatus"));
        assert!(debug_output.contains("healthy"));
    }
}

