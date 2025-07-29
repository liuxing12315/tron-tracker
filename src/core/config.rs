use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub blockchain: BlockchainConfig,
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
pub struct RedisConfig {
    pub url: String,
    pub max_connections: u32,
    pub default_ttl: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    pub nodes: Vec<NodeConfig>,
    pub start_block: Option<u64>,
    pub batch_size: u32,
    pub scan_interval: u64,
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

impl Default for Config {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                url: "postgresql://postgres:password@localhost:5432/tron_tracker".to_string(),
                max_connections: 20,
                min_connections: 5,
                acquire_timeout: 30,
            },
            redis: RedisConfig {
                url: "redis://localhost:6379".to_string(),
                max_connections: 10,
                default_ttl: 300,
            },
            blockchain: BlockchainConfig {
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
                start_block: None,
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
        
        // Redis overrides
        if let Ok(url) = std::env::var("REDIS_URL") {
            self.redis.url = url;
        }
        
        // Blockchain overrides
        if let Ok(start_block) = std::env::var("BLOCKCHAIN_START_BLOCK") {
            if let Ok(val) = start_block.parse() {
                self.blockchain.start_block = Some(val);
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

