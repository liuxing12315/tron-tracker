# TRX Tracker 快速启动指南

本指南帮助您快速启动 TRX Tracker 系统的核心功能。

## 前置准备

### 1. 安装依赖
```bash
# PostgreSQL
brew install postgresql@14  # macOS
sudo apt install postgresql-14  # Ubuntu

# Redis
brew install redis  # macOS
sudo apt install redis-server  # Ubuntu

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. 创建数据库
```bash
# 启动 PostgreSQL
pg_ctl -D /usr/local/var/postgres start

# 创建数据库
createdb trontracker

# 运行迁移
psql -U postgres -d trontracker -f migrations/001_initial.sql
```

### 3. 启动 Redis
```bash
redis-server
```

## 快速启动

### 步骤1: 激活核心功能

编辑 `src/api/mod.rs`，取消注释第30行：
```rust
// 修改前：
// .route("/api/v1/transactions/multi-address", post(transaction::multi_address_query))

// 修改后：
.route("/api/v1/transactions/multi-address", post(transaction::multi_address_query))
```

### 步骤2: 连接扫描器和通知服务

编辑 `src/main.rs`，在第100行后添加：
```rust
// 创建事件通道
let (tx, rx) = mpsc::unbounded_channel::<TransactionEvent>();

// 配置扫描器
let mut scanner = scanner_service.clone();
scanner.set_notification_sender(tx.clone());

// 启动扫描器
let scanner_handle = tokio::spawn(async move {
    if let Err(e) = scanner.start().await {
        error!("Scanner error: {}", e);
    }
});

// 启动 Webhook 服务
let webhook_service_clone = webhook_service.clone();
let webhook_handle = tokio::spawn(async move {
    webhook_service_clone.start().await
});

// 连接事件处理
let event_handle = tokio::spawn(async move {
    while let Some(event) = rx.recv().await {
        // 发送到 Webhook
        if let Err(e) = webhook_service.handle_transaction_event(event.clone()).await {
            warn!("Webhook handling error: {}", e);
        }
        
        // 发送到 WebSocket
        // WebSocket 广播逻辑
    }
});
```

### 步骤3: 配置系统

创建 `config/local.toml`：
```toml
[database]
url = "postgresql://postgres:password@localhost:5432/trontracker"

[redis]
url = "redis://localhost:6379"

[blockchain]
start_block = 62800000  # 设置起始区块
batch_size = 100
scan_interval = 3

[tron]
nodes = [
    { name = "TronGrid", url = "https://api.trongrid.io", priority = 1 }
]
```

### 步骤4: 启动后端服务
```bash
# 开发模式
cargo run

# 生产模式
cargo build --release
./target/release/tron-tracker
```

### 步骤5: 启动前端服务
```bash
cd admin-ui

# 安装依赖
pnpm install

# 开发模式
pnpm dev

# 生产构建
pnpm build
pnpm preview
```

## 测试核心功能

### 1. 测试批量查询
```bash
curl -X POST http://localhost:8080/api/v1/transactions/multi-address \
  -H "Content-Type: application/json" \
  -d '{
    "addresses": "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t,TLa2f6VPqDgRE67v1736s7bJ8Ray5wYjU7",
    "limit": 10
  }'
```

### 2. 测试 WebSocket 连接
```javascript
// 在浏览器控制台运行
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onopen = () => {
  console.log('Connected to WebSocket');
  
  // 订阅地址
  ws.send(JSON.stringify({
    type: 'subscribe',
    subscription: {
      event_types: ['transaction'],
      addresses: ['TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t']
    }
  }));
};

ws.onmessage = (event) => {
  console.log('Received:', JSON.parse(event.data));
};
```

### 3. 配置 Webhook
通过管理界面 (http://localhost:3000/webhooks) 添加 Webhook：
- Name: "Payment Notifications"
- URL: "https://your-server.com/webhook"
- Events: ["transaction"]
- Secret: "your-webhook-secret"

### 4. 查看管理界面
访问 http://localhost:3000 查看：
- Dashboard: 系统运行状态
- Transactions: 查询交易和测试批量查询
- WebSockets: 查看实时连接
- Webhooks: 管理 Webhook 配置

## 生产部署

### 使用 Docker
```bash
# 构建镜像
docker-compose build

# 启动所有服务
docker-compose up -d

# 查看日志
docker-compose logs -f
```

### 环境变量配置
```bash
export DATABASE_URL="postgresql://user:pass@db-host/trontracker"
export REDIS_URL="redis://redis-host:6379"
export BLOCKCHAIN_START_BLOCK="62800000"
export JWT_SECRET="your-production-secret"
```

## 常见问题

### Q: 扫描器没有开始工作？
A: 检查日志确认数据库连接和 Tron 节点连接是否正常。

### Q: WebSocket 连接失败？
A: 确认 `/ws` 路由已激活，防火墙允许 WebSocket 连接。

### Q: 批量查询返回空结果？
A: 确认扫描器已经扫描了包含这些地址交易的区块。

### Q: Webhook 没有收到通知？
A: 检查 Webhook URL 是否可访问，查看 Webhook 投递日志。

## 下一步

1. 配置 API 密钥认证保护接口
2. 设置监控和告警
3. 优化数据库索引
4. 配置 Redis 持久化
5. 部署到生产环境