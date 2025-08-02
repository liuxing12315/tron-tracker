use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub tron: TronConfig,
    pub api: ApiConfig,
    pub notifications: NotificationConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: u64,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub name: String,
    pub url: String,
    pub api_key: Option<String>,
    pub priority: u8,
    pub timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub rate_limit: RateLimitConfig,
    pub cors: CorsConfig,
    pub pagination: PaginationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationConfig {
    pub default_limit: u32,
    pub max_limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub webhook: WebhookConfig,
    pub websocket: WebSocketConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub timeout: u64,
    pub retry_attempts: u32,
    pub retry_delay: u64,
    pub max_payload_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    pub max_connections: u32,
    pub ping_interval: u64,
    pub message_buffer_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub token_expiry: u64,
    pub api_key_length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronConfig {
    pub nodes: Vec<NodeConfig>,
    pub api_key: Option<String>,
    pub timeout: u64,
    pub batch_size: u32,
    pub scan_interval: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                url: "postgresql://postgres:password@localhost:5432/tron_tracker".to_string(),
                max_connections: 20,
                min_connections: 5,
                acquire_timeout: 30,
            },
            tron: TronConfig {
                nodes: vec![
                    NodeConfig {
                        name: "TronGrid".to_string(),
                        url: "https://api.trongrid.io".to_string(),
                        api_key: None,
                        priority: 1,
                        timeout: 30,
                    },
                    NodeConfig {
                        name: "GetBlock".to_string(),
                        url: "https://go.getblock.io".to_string(),
                        api_key: None,
                        priority: 2,
                        timeout: 30,
                    },
                ],
                api_key: None,
                timeout: 30,
                batch_size: 100,
                scan_interval: 3,
            },
            api: ApiConfig {
                rate_limit: RateLimitConfig {
                    requests_per_minute: 1000,
                    burst_size: 100,
                },
                cors: CorsConfig {
                    allowed_origins: vec!["*".to_string()],
                    allowed_methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string()],
                    allowed_headers: vec!["*".to_string()],
                },
                pagination: PaginationConfig {
                    default_limit: 20,
                    max_limit: 100,
                },
            },
            notifications: NotificationConfig {
                webhook: WebhookConfig {
                    timeout: 30,
                    retry_attempts: 3,
                    retry_delay: 5,
                    max_payload_size: 1024 * 1024, // 1MB
                },
                websocket: WebSocketConfig {
                    max_connections: 10000,
                    ping_interval: 30,
                    message_buffer_size: 1000,
                },
            },
            auth: AuthConfig {
                jwt_secret: "your-secret-key-change-in-production".to_string(),
                token_expiry: 86400, // 24 hours
                api_key_length: 32,
            },
        }
    }
}

impl Config {
    pub async fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        
        // Start with default configuration
        let mut config = Self::default();
        
        // Override with file configuration if exists
        if path.exists() {
            let content = tokio::fs::read_to_string(path).await?;
            let file_config: Config = toml::from_str(&content)?;
            config = file_config;
        }
        
        // Override with environment variables
        config.apply_env_overrides();
        
        Ok(config)
    }
    
    fn apply_env_overrides(&mut self) {
        // Database overrides
        if let Ok(url) = std::env::var("DATABASE_URL") {
            self.database.url = url;
        }
        if let Ok(max_conn) = std::env::var("DATABASE_MAX_CONNECTIONS") {
            if let Ok(val) = max_conn.parse() {
                self.database.max_connections = val;
            }
        }
        
        
        // Auth overrides
        if let Ok(secret) = std::env::var("JWT_SECRET") {
            self.auth.jwt_secret = secret;
        }
    }
    
    pub async fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        tokio::fs::write(path, content).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        
        // Test database defaults
        assert_eq!(config.database.max_connections, 20);
        assert_eq!(config.database.min_connections, 5);
        assert_eq!(config.database.acquire_timeout, 30);
        
        // Test tron defaults
        assert_eq!(config.tron.batch_size, 100);
        assert_eq!(config.tron.scan_interval, 3);
        
        // Test API defaults
        assert_eq!(config.api.rate_limit.requests_per_minute, 1000);
        assert_eq!(config.api.pagination.default_limit, 20);
        assert_eq!(config.api.pagination.max_limit, 100);
        
        // Test auth defaults
        assert_eq!(config.auth.token_expiry, 86400);
        assert_eq!(config.auth.api_key_length, 32);
    }

    #[test]
    fn test_node_config_creation() {
        let node = NodeConfig {
            name: "TestNode".to_string(),
            url: "https://test.example.com".to_string(),
            api_key: Some("test_key".to_string()),
            priority: 1,
            timeout: 30,
        };

        assert_eq!(node.name, "TestNode");
        assert_eq!(node.url, "https://test.example.com");
        assert_eq!(node.api_key, Some("test_key".to_string()));
        assert_eq!(node.priority, 1);
        assert_eq!(node.timeout, 30);
    }

    #[test]
    fn test_database_config_creation() {
        let db_config = DatabaseConfig {
            url: "postgresql://user:pass@localhost/test".to_string(),
            max_connections: 50,
            min_connections: 10,
            acquire_timeout: 60,
        };

        assert_eq!(db_config.url, "postgresql://user:pass@localhost/test");
        assert_eq!(db_config.max_connections, 50);
        assert_eq!(db_config.min_connections, 10);
        assert_eq!(db_config.acquire_timeout, 60);
    }

    #[test]
    fn test_rate_limit_config() {
        let rate_config = RateLimitConfig {
            requests_per_minute: 500,
            burst_size: 50,
        };

        assert_eq!(rate_config.requests_per_minute, 500);
        assert_eq!(rate_config.burst_size, 50);
    }

    #[test]
    fn test_cors_config() {
        let cors_config = CorsConfig {
            allowed_origins: vec!["http://localhost:3000".to_string()],
            allowed_methods: vec!["GET".to_string(), "POST".to_string()],
            allowed_headers: vec!["Content-Type".to_string()],
        };

        assert_eq!(cors_config.allowed_origins, vec!["http://localhost:3000"]);
        assert_eq!(cors_config.allowed_methods, vec!["GET", "POST"]);
        assert_eq!(cors_config.allowed_headers, vec!["Content-Type"]);
    }

    #[test]
    fn test_webhook_config() {
        let webhook_config = WebhookConfig {
            timeout: 60,
            retry_attempts: 5,
            retry_delay: 10,
            max_payload_size: 2048,
        };

        assert_eq!(webhook_config.timeout, 60);
        assert_eq!(webhook_config.retry_attempts, 5);
        assert_eq!(webhook_config.retry_delay, 10);
        assert_eq!(webhook_config.max_payload_size, 2048);
    }

    #[test]
    fn test_websocket_config() {
        let ws_config = WebSocketConfig {
            max_connections: 5000,
            ping_interval: 60,
            message_buffer_size: 2000,
        };

        assert_eq!(ws_config.max_connections, 5000);
        assert_eq!(ws_config.ping_interval, 60);
        assert_eq!(ws_config.message_buffer_size, 2000);
    }

    #[tokio::test]
    async fn test_config_serialization_deserialization() {
        let config = Config::default();
        
        // Test TOML serialization
        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("[database]"));
        assert!(toml_str.contains("[tron]"));
        
        // Test TOML deserialization
        let deserialized: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.database.max_connections, deserialized.database.max_connections);
        assert_eq!(config.tron.batch_size, deserialized.tron.batch_size);
    }

    #[tokio::test]
    async fn test_config_load_from_nonexistent_file() {
        let temp_path = "/tmp/nonexistent_config.toml";
        
        // Should return default config when file doesn't exist
        let config = Config::load(temp_path).await.unwrap();
        let default_config = Config::default();
        
        assert_eq!(config.database.max_connections, default_config.database.max_connections);
        assert_eq!(config.tron.batch_size, default_config.tron.batch_size);
    }

    #[tokio::test]
    async fn test_config_save_and_load() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path();
        
        // Create a custom config
        let mut config = Config::default();
        config.database.max_connections = 100;
        config.tron.batch_size = 200;
        
        // Save config
        config.save(temp_path).await.unwrap();
        
        // Load config
        let loaded_config = Config::load(temp_path).await.unwrap();
        
        assert_eq!(loaded_config.database.max_connections, 100);
        assert_eq!(loaded_config.tron.batch_size, 200);
    }

    #[test]
    fn test_apply_env_overrides() {
        let mut config = Config::default();
        
        // Set environment variables
        env::set_var("DATABASE_URL", "postgresql://env:pass@localhost/env_db");
        env::set_var("DATABASE_MAX_CONNECTIONS", "50");
        env::set_var("JWT_SECRET", "env_secret_key");
        
        // Apply environment overrides
        config.apply_env_overrides();
        
        assert_eq!(config.database.url, "postgresql://env:pass@localhost/env_db");
        assert_eq!(config.database.max_connections, 50);
        assert_eq!(config.auth.jwt_secret, "env_secret_key");
        
        // Clean up environment variables
        env::remove_var("DATABASE_URL");
        env::remove_var("DATABASE_MAX_CONNECTIONS");
        env::remove_var("JWT_SECRET");
    }

    #[test]
    fn test_apply_env_overrides_invalid_values() {
        let mut config = Config::default();
        let original_max_conn = config.database.max_connections;
        
        // Set invalid environment variable
        env::set_var("DATABASE_MAX_CONNECTIONS", "invalid_number");
        
        // Apply environment overrides
        config.apply_env_overrides();
        
        // Should keep original value when parsing fails
        assert_eq!(config.database.max_connections, original_max_conn);
        
        // Clean up
        env::remove_var("DATABASE_MAX_CONNECTIONS");
    }

    #[test]
    fn test_notification_config() {
        let notification_config = NotificationConfig {
            webhook: WebhookConfig {
                timeout: 45,
                retry_attempts: 4,
                retry_delay: 8,
                max_payload_size: 4096,
            },
            websocket: WebSocketConfig {
                max_connections: 8000,
                ping_interval: 45,
                message_buffer_size: 1500,
            },
        };

        assert_eq!(notification_config.webhook.timeout, 45);
        assert_eq!(notification_config.webhook.retry_attempts, 4);
        assert_eq!(notification_config.websocket.max_connections, 8000);
        assert_eq!(notification_config.websocket.ping_interval, 45);
    }

    #[test]
    fn test_tron_config_multiple_nodes() {
        let tron_config = TronConfig {
            nodes: vec![
                NodeConfig {
                    name: "Primary".to_string(),
                    url: "https://primary.tron.io".to_string(),
                    api_key: Some("primary_key".to_string()),
                    priority: 1,
                    timeout: 30,
                },
                NodeConfig {
                    name: "Backup".to_string(),
                    url: "https://backup.tron.io".to_string(),
                    api_key: None,
                    priority: 2,
                    timeout: 45,
                },
            ],
            api_key: Some("global_key".to_string()),
            timeout: 60,
        };

        assert_eq!(tron_config.nodes.len(), 2);
        assert_eq!(tron_config.nodes[0].name, "Primary");
        assert_eq!(tron_config.nodes[1].name, "Backup");
        assert_eq!(tron_config.api_key, Some("global_key".to_string()));
        assert_eq!(tron_config.timeout, 60);
    }
}

