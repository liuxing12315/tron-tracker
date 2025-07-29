# TRX Tracker - Unified System Architecture

## Overview

TRX Tracker has been completely redesigned with a unified architecture that eliminates redundancy, improves maintainability, and follows the "convention over configuration" principle. This document outlines the new architectural design and the rationale behind key decisions.

## Design Principles

### 1. Unified Codebase
- **Single Binary**: All functionality consolidated into one Rust binary
- **Shared Core**: Common data models, database access, and configuration
- **Modular Services**: Clear separation of concerns without code duplication

### 2. Convention Over Configuration
- **Sensible Defaults**: Zero-configuration startup for development
- **Environment Override**: Production settings via environment variables
- **Single Config File**: One TOML file for all system settings

### 3. Simplified Deployment
- **Container Ready**: Docker and Kubernetes deployment support
- **Horizontal Scaling**: Stateless design for easy scaling
- **Health Monitoring**: Built-in health checks and metrics

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        TRX Tracker                             │
│                      (Single Binary)                           │
├─────────────────────────────────────────────────────────────────┤
│                     Presentation Layer                         │
├─────────────────┬─────────────────┬─────────────────────────────┤
│   Admin UI      │   REST API      │   WebSocket Server         │
│   (React SPA)   │   (Axum)        │   (Axum + WebSocket)       │
└─────────────────┴─────────────────┴─────────────────────────────┘
├─────────────────────────────────────────────────────────────────┤
│                      Service Layer                             │
├─────────────────┬─────────────────┬─────────────────────────────┤
│   Scanner       │   Webhook       │   Notification              │
│   Service       │   Service       │   Service                   │
└─────────────────┴─────────────────┴─────────────────────────────┘
├─────────────────────────────────────────────────────────────────┤
│                       Core Layer                               │
├─────────────────┬─────────────────┬─────────────────────────────┤
│   Models        │   Database      │   Configuration             │
│   (Shared)      │   (Unified)     │   (Single Source)           │
└─────────────────┴─────────────────┴─────────────────────────────┘
├─────────────────────────────────────────────────────────────────┤
│                    Infrastructure                              │
├─────────────────┬─────────────────┬─────────────────────────────┤
│   PostgreSQL    │   Redis         │   Tron Network              │
│   (Primary DB)  │   (Cache)       │   (Blockchain)              │
└─────────────────┴─────────────────┴─────────────────────────────┘
```

## Core Components

### 1. Unified Configuration System

**Location**: `src/core/config.rs`

The configuration system provides a single source of truth for all system settings:

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub blockchain: BlockchainConfig,
    pub tron: TronConfig,
    pub webhook: WebhookConfig,
    pub websocket: WebSocketConfig,
}
```

**Key Features**:
- **Default Values**: Sensible defaults for all settings
- **Environment Override**: Any setting can be overridden via environment variables
- **Validation**: Configuration validation at startup
- **Hot Reload**: Runtime configuration updates for non-critical settings

### 2. Shared Data Models

**Location**: `src/core/models.rs`

All components share the same data structures, eliminating inconsistencies:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub hash: String,
    pub block_number: u64,
    pub from_address: String,
    pub to_address: String,
    pub amount: String,
    pub token: String,
    pub status: TransactionStatus,
    pub timestamp: DateTime<Utc>,
    pub gas_used: u64,
    pub gas_price: u64,
}
```

**Benefits**:
- **Type Safety**: Compile-time guarantees of data consistency
- **Single Source**: One definition used across all modules
- **Serialization**: Automatic JSON/database serialization

### 3. Unified Database Layer

**Location**: `src/core/database.rs`

A single database abstraction layer serves all components:

```rust
impl Database {
    // Transaction operations
    pub async fn get_transactions(&self, query: &TransactionQuery, page: u32, limit: u32) -> Result<(Vec<Transaction>, u64)>;
    pub async fn get_transactions_by_addresses(&self, addresses: &[String], query: &MultiAddressQuery, page: u32, limit: u32) -> Result<(Vec<Transaction>, u64)>;
    
    // Webhook operations
    pub async fn create_webhook(&self, webhook: &CreateWebhookRequest) -> Result<Webhook>;
    pub async fn update_webhook(&self, id: &str, webhook: &UpdateWebhookRequest) -> Result<Webhook>;
    
    // WebSocket operations
    pub async fn get_websocket_connections(&self) -> Result<Vec<WebSocketConnection>>;
    pub async fn get_websocket_stats(&self) -> Result<WebSocketStats>;
}
```

**Features**:
- **Connection Pooling**: Efficient connection management
- **Query Optimization**: Prepared statements and indexing
- **Transaction Support**: ACID compliance for critical operations
- **Caching Integration**: Automatic Redis caching for frequently accessed data

## Service Architecture

### 1. Scanner Service

**Purpose**: Monitors the Tron blockchain for new transactions

**Key Features**:
- **Multi-Node Support**: Automatic failover between Tron nodes
- **Batch Processing**: Efficient bulk transaction processing
- **State Management**: Persistent scanning state across restarts
- **Error Recovery**: Automatic retry and error handling

**Implementation**:
```rust
pub struct Scanner {
    config: Config,
    db: Database,
    tron_client: TronClient,
    current_block: Arc<AtomicU64>,
}

impl Scanner {
    pub async fn scan_blocks(&self) -> Result<()> {
        let start_block = self.get_last_scanned_block().await?;
        let latest_block = self.tron_client.get_latest_block().await?;
        
        for block_num in start_block..=latest_block {
            let transactions = self.scan_block(block_num).await?;
            self.process_transactions(transactions).await?;
            self.update_scan_progress(block_num).await?;
        }
        
        Ok(())
    }
}
```

### 2. Webhook Service

**Purpose**: Delivers real-time notifications to external endpoints

**Key Features**:
- **Reliable Delivery**: Retry logic with exponential backoff
- **Signature Verification**: HMAC-SHA256 request signing
- **Rate Limiting**: Configurable delivery rate limits
- **Dead Letter Queue**: Failed delivery handling

**Implementation**:
```rust
pub struct WebhookService {
    config: Config,
    db: Database,
    http_client: reqwest::Client,
    delivery_queue: Arc<Mutex<VecDeque<WebhookDelivery>>>,
}

impl WebhookService {
    pub async fn deliver_webhook(&self, webhook: &Webhook, payload: &serde_json::Value) -> Result<()> {
        let signature = self.generate_signature(&webhook.secret, payload)?;
        
        let response = self.http_client
            .post(&webhook.url)
            .header("X-Webhook-Signature", signature)
            .json(payload)
            .timeout(Duration::from_secs(webhook.timeout))
            .send()
            .await?;
            
        if response.status().is_success() {
            self.record_success(&webhook.id).await?;
        } else {
            self.schedule_retry(&webhook.id, payload.clone()).await?;
        }
        
        Ok(())
    }
}
```

### 3. WebSocket Service

**Purpose**: Provides real-time streaming connections for clients

**Key Features**:
- **Connection Management**: Efficient connection pooling and cleanup
- **Subscription Filtering**: Fine-grained event filtering
- **Message Broadcasting**: Efficient message distribution
- **Connection Monitoring**: Real-time connection statistics

**Implementation**:
```rust
pub struct WebSocketService {
    connections: Arc<RwLock<HashMap<String, WebSocketConnection>>>,
    subscriptions: Arc<RwLock<HashMap<String, Vec<Subscription>>>>,
    message_queue: Arc<Mutex<VecDeque<BroadcastMessage>>>,
}

impl WebSocketService {
    pub async fn handle_connection(&self, socket: WebSocket, addr: SocketAddr) -> Result<()> {
        let connection_id = Uuid::new_v4().to_string();
        let connection = WebSocketConnection::new(connection_id.clone(), socket, addr);
        
        self.connections.write().await.insert(connection_id.clone(), connection);
        
        // Handle messages
        while let Some(message) = connection.receive().await {
            match message {
                Message::Subscribe(sub) => self.add_subscription(&connection_id, sub).await?,
                Message::Unsubscribe(sub_id) => self.remove_subscription(&connection_id, &sub_id).await?,
                Message::Ping => connection.send(Message::Pong).await?,
                _ => {}
            }
        }
        
        self.cleanup_connection(&connection_id).await?;
        Ok(())
    }
}
```

## API Design

### 1. Unified REST API

**Endpoint Structure**:
```
/api/v1/
├── transactions/
│   ├── GET    /                    # List transactions
│   ├── GET    /:hash               # Get specific transaction
│   └── POST   /multi-address       # Multi-address query
├── addresses/
│   ├── GET    /:address            # Get address info
│   └── GET    /:address/transactions # Get address transactions
├── webhooks/
│   ├── GET    /                    # List webhooks
│   ├── POST   /                    # Create webhook
│   ├── GET    /:id                 # Get webhook
│   ├── PUT    /:id                 # Update webhook
│   └── DELETE /:id                 # Delete webhook
├── websockets/
│   ├── GET    /connections         # List connections
│   └── GET    /stats               # Get statistics
├── api-keys/
│   ├── GET    /                    # List API keys
│   ├── POST   /                    # Create API key
│   └── DELETE /:id                 # Delete API key
├── config/
│   ├── GET    /                    # Get configuration
│   └── PUT    /                    # Update configuration
├── logs/
│   └── GET    /                    # Get system logs
└── dashboard/
    └── GET    /stats               # Get dashboard stats
```

**Response Format**:
```json
{
  "success": true,
  "data": { ... },
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 1000,
    "total_pages": 50
  }
}
```

### 2. WebSocket Protocol

**Connection**: `ws://localhost:8080/ws`

**Message Types**:
```json
// Subscribe to events
{
  "type": "subscribe",
  "events": ["transaction", "large_transfer"],
  "filters": {
    "addresses": ["TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"],
    "tokens": ["USDT"],
    "min_amount": "1000"
  }
}

// Unsubscribe from events
{
  "type": "unsubscribe",
  "subscription_id": "sub_123456"
}

// Transaction notification
{
  "type": "transaction",
  "data": {
    "hash": "0x...",
    "from": "TR7NHqje...",
    "to": "TLa2f6VP...",
    "amount": "1000.00",
    "token": "USDT",
    "timestamp": "2024-01-15T10:30:00Z"
  }
}
```

## Data Flow

### 1. Transaction Processing Flow

```
Tron Network → Scanner Service → Database → Notification Services
                     ↓
              [Transaction Detected]
                     ↓
              ┌─────────────────┐
              │   Database      │
              │   Storage       │
              └─────────────────┘
                     ↓
              ┌─────────────────┐
              │   Notification  │
              │   Dispatcher    │
              └─────────────────┘
                     ↓
         ┌─────────────────┬─────────────────┐
         │   Webhook       │   WebSocket     │
         │   Delivery      │   Broadcast     │
         └─────────────────┴─────────────────┘
```

### 2. API Request Flow

```
Client Request → Load Balancer → API Server → Database/Cache → Response
                                     ↓
                              ┌─────────────────┐
                              │   Middleware    │
                              │   - Auth        │
                              │   - Rate Limit  │
                              │   - Logging     │
                              └─────────────────┘
                                     ↓
                              ┌─────────────────┐
                              │   Handler       │
                              │   - Validation  │
                              │   - Business    │
                              │   - Response    │
                              └─────────────────┘
```

## Performance Optimizations

### 1. Database Optimizations

**Indexing Strategy**:
```sql
-- Transaction queries
CREATE INDEX idx_transactions_addresses ON transactions (from_address, to_address);
CREATE INDEX idx_transactions_timestamp ON transactions (timestamp DESC);
CREATE INDEX idx_transactions_token ON transactions (token);

-- Multi-address queries
CREATE INDEX idx_transactions_multi_addr ON transactions 
USING GIN ((ARRAY[from_address, to_address]));

-- Webhook queries
CREATE INDEX idx_webhooks_enabled ON webhooks (enabled) WHERE enabled = true;
```

**Query Optimization**:
- **Prepared Statements**: All queries use prepared statements
- **Connection Pooling**: Configurable connection pool size
- **Read Replicas**: Support for read-only database replicas
- **Batch Operations**: Bulk inserts and updates

### 2. Caching Strategy

**Redis Cache Layers**:
```
┌─────────────────┐
│   API Response  │  TTL: 60s
│   Cache         │
└─────────────────┘
┌─────────────────┐
│   Query Result  │  TTL: 300s
│   Cache         │
└─────────────────┘
┌─────────────────┐
│   Static Data   │  TTL: 3600s
│   Cache         │
└─────────────────┘
```

**Cache Keys**:
- `tx:hash:{hash}` - Individual transactions
- `addr:txs:{address}:{page}:{limit}` - Address transactions
- `multi:addrs:{hash}:{page}:{limit}` - Multi-address queries
- `stats:dashboard` - Dashboard statistics

### 3. Connection Management

**WebSocket Connections**:
- **Connection Pooling**: Efficient memory usage
- **Heartbeat Monitoring**: Automatic dead connection cleanup
- **Message Queuing**: Buffered message delivery
- **Subscription Indexing**: Fast subscription lookup

**HTTP Connections**:
- **Keep-Alive**: Persistent connections for webhooks
- **Connection Limits**: Configurable concurrent connections
- **Timeout Management**: Request and connection timeouts

## Security Architecture

### 1. Authentication & Authorization

**API Key Management**:
```rust
#[derive(Debug, Clone)]
pub struct ApiKey {
    pub id: String,
    pub name: String,
    pub key_hash: String,
    pub permissions: Vec<Permission>,
    pub rate_limit: u32,
    pub ip_whitelist: Option<Vec<String>>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub enum Permission {
    ReadTransactions,
    ReadAddresses,
    ManageWebhooks,
    ManageWebSockets,
    ManageApiKeys,
    ManageConfig,
    ViewLogs,
}
```

**Request Authentication**:
```
Authorization: Bearer tk_live_1234567890abcdef...
```

### 2. Webhook Security

**Signature Generation**:
```rust
fn generate_signature(secret: &str, payload: &serde_json::Value) -> String {
    let payload_str = serde_json::to_string(payload).unwrap();
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(payload_str.as_bytes());
    format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
}
```

**Verification**:
```
X-Webhook-Signature: sha256=a1b2c3d4e5f6...
```

### 3. Rate Limiting

**Implementation**:
```rust
#[derive(Debug)]
pub struct RateLimiter {
    redis: Redis,
    window_size: Duration,
    max_requests: u32,
}

impl RateLimiter {
    pub async fn check_rate_limit(&self, key: &str) -> Result<bool> {
        let current_count: u32 = self.redis.incr(key, 1).await?;
        
        if current_count == 1 {
            self.redis.expire(key, self.window_size.as_secs()).await?;
        }
        
        Ok(current_count <= self.max_requests)
    }
}
```

## Monitoring & Observability

### 1. Metrics Collection

**Prometheus Metrics**:
```rust
lazy_static! {
    static ref TRANSACTION_COUNTER: Counter = Counter::new(
        "tron_tracker_transactions_total",
        "Total number of transactions processed"
    ).unwrap();
    
    static ref API_REQUEST_DURATION: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "tron_tracker_api_request_duration_seconds",
            "API request duration in seconds"
        )
    ).unwrap();
    
    static ref WEBSOCKET_CONNECTIONS: Gauge = Gauge::new(
        "tron_tracker_websocket_connections",
        "Number of active WebSocket connections"
    ).unwrap();
}
```

**Health Checks**:
```rust
#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub version: String,
    pub timestamp: String,
    pub components: HashMap<String, ComponentHealth>,
}

#[derive(Debug, Serialize)]
pub struct ComponentHealth {
    pub status: String,
    pub last_check: String,
    pub details: Option<serde_json::Value>,
}
```

### 2. Logging Strategy

**Structured Logging**:
```rust
use tracing::{info, warn, error, debug};

#[instrument(skip(self))]
async fn process_transaction(&self, tx: &Transaction) -> Result<()> {
    info!(
        transaction_hash = %tx.hash,
        from_address = %tx.from_address,
        to_address = %tx.to_address,
        amount = %tx.amount,
        "Processing transaction"
    );
    
    // Processing logic...
    
    Ok(())
}
```

**Log Levels**:
- **ERROR**: System errors, failed operations
- **WARN**: Recoverable errors, degraded performance
- **INFO**: Normal operations, state changes
- **DEBUG**: Detailed execution information

## Deployment Architecture

### 1. Container Strategy

**Multi-Stage Dockerfile**:
```dockerfile
# Build stage
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/tron-tracker /usr/local/bin/
COPY --from=builder /app/config /etc/tron-tracker/
EXPOSE 8080 3000
CMD ["tron-tracker"]
```

**Docker Compose**:
```yaml
version: '3.8'
services:
  tron-tracker:
    build: .
    ports:
      - "8080:8080"
      - "3000:3000"
    environment:
      - DATABASE_URL=postgresql://user:pass@db:5432/trontracker
      - REDIS_URL=redis://redis:6379
    depends_on:
      - db
      - redis
      
  db:
    image: postgres:14
    environment:
      POSTGRES_DB: trontracker
      POSTGRES_USER: user
      POSTGRES_PASSWORD: pass
    volumes:
      - postgres_data:/var/lib/postgresql/data
      
  redis:
    image: redis:6-alpine
    volumes:
      - redis_data:/data
```

### 2. Kubernetes Deployment

**Deployment Manifest**:
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: tron-tracker
spec:
  replicas: 3
  selector:
    matchLabels:
      app: tron-tracker
  template:
    metadata:
      labels:
        app: tron-tracker
    spec:
      containers:
      - name: tron-tracker
        image: tron-tracker:latest
        ports:
        - containerPort: 8080
        - containerPort: 3000
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: tron-tracker-secrets
              key: database-url
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
```

## Migration Strategy

### 1. From Legacy System

**Data Migration**:
```rust
pub struct MigrationService {
    old_db: Database,
    new_db: Database,
}

impl MigrationService {
    pub async fn migrate_transactions(&self) -> Result<()> {
        let batch_size = 1000;
        let mut offset = 0;
        
        loop {
            let transactions = self.old_db
                .get_transactions_batch(offset, batch_size)
                .await?;
                
            if transactions.is_empty() {
                break;
            }
            
            let unified_transactions: Vec<Transaction> = transactions
                .into_iter()
                .map(|tx| self.convert_transaction(tx))
                .collect();
                
            self.new_db
                .insert_transactions_batch(&unified_transactions)
                .await?;
                
            offset += batch_size;
        }
        
        Ok(())
    }
}
```

**Configuration Migration**:
```rust
pub fn migrate_config(old_config: &LegacyConfig) -> Config {
    Config {
        server: ServerConfig {
            host: old_config.api.host.clone(),
            port: old_config.api.port,
            admin_port: 3000, // New default
        },
        database: DatabaseConfig {
            url: old_config.database.url.clone(),
            max_connections: old_config.database.pool_size,
        },
        // ... other mappings
    }
}
```

### 2. Zero-Downtime Deployment

**Blue-Green Strategy**:
1. Deploy new version alongside old version
2. Gradually shift traffic to new version
3. Monitor metrics and error rates
4. Complete cutover or rollback if issues detected

**Database Migrations**:
```sql
-- Add new columns with defaults
ALTER TABLE transactions ADD COLUMN unified_status VARCHAR(20) DEFAULT 'unknown';

-- Migrate data in batches
UPDATE transactions 
SET unified_status = CASE 
    WHEN old_status = 1 THEN 'success'
    WHEN old_status = 0 THEN 'failed'
    ELSE 'pending'
END
WHERE unified_status = 'unknown'
LIMIT 10000;

-- Drop old columns after migration
ALTER TABLE transactions DROP COLUMN old_status;
```

## Future Enhancements

### 1. Planned Features

**Enhanced Analytics**:
- Transaction pattern analysis
- Address clustering and labeling
- Market impact analysis
- Fraud detection algorithms

**Additional Blockchains**:
- Ethereum support
- Binance Smart Chain support
- Polygon support
- Cross-chain transaction tracking

**Advanced Notifications**:
- SMS notifications
- Email notifications
- Slack/Discord integrations
- Mobile push notifications

### 2. Scalability Improvements

**Horizontal Scaling**:
- Microservice decomposition
- Event-driven architecture
- Message queue integration
- Distributed caching

**Performance Optimizations**:
- GraphQL API support
- Streaming API endpoints
- Advanced caching strategies
- Database sharding

## Conclusion

The unified architecture of TRX Tracker represents a significant improvement over the previous fragmented approach. By consolidating functionality into a single, well-designed system, we have achieved:

- **Reduced Complexity**: Eliminated duplicate code and inconsistent interfaces
- **Improved Maintainability**: Single codebase with clear module boundaries
- **Enhanced Performance**: Optimized data flow and reduced overhead
- **Better User Experience**: Consistent API design and unified management interface
- **Simplified Deployment**: Single binary with comprehensive functionality

This architecture provides a solid foundation for future enhancements while maintaining the flexibility to adapt to changing requirements. The modular design ensures that individual components can be optimized or replaced without affecting the entire system, while the unified approach guarantees consistency and reliability across all functionality.

