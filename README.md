# TRX Tracker - Tron 区块链增强数据服务

专注于提供 Tron 节点原生不支持功能的高性能区块链数据服务系统，包括批量地址查询、实时充值通知等核心功能。

## 🎯 核心价值

TRX Tracker 填补了 Tron 节点功能空白，为开发者提供：

### 核心功能

1. **批量地址交易查询** - Tron 节点无法直接提供
   - 一次查询最多100个地址的交易记录
   - 支持多维度筛选（时间、金额、代币类型）
   - Redis 缓存优化，毫秒级响应

2. **实时充值通知** - 监控特定地址的充值事件
   - WebSocket 实时推送
   - Webhook HTTP 回调
   - 支持 HMAC 签名验证

3. **Web 管理界面** - 完整的系统管理
   - 实时监控面板
   - 交易查询和分析
   - Webhook/WebSocket 管理
   - API 密钥管理

### 技术特点

- **高性能**: Rust 实现，异步处理
- **低延迟**: 缓存优化，快速响应
- **可扩展**: 模块化架构，易于扩展
- **生产就绪**: 包含监控、日志、错误处理

## 🚀 快速开始

详细启动指南请查看 [QUICK_START.md](QUICK_START.md)

### 最简启动

```bash
# 1. 安装依赖
brew install postgresql redis  # macOS
cargo build

# 2. 初始化数据库
createdb trontracker
psql -d trontracker -f migrations/001_initial.sql

# 3. 启动服务
cargo run

# 4. 访问管理界面
cd admin-ui && pnpm install && pnpm dev
# 访问 http://localhost:5173
```

### 测试核心功能

```bash
# 批量查询地址交易
curl -X POST http://localhost:8080/api/v1/transactions/multi-address \
  -H "Content-Type: application/json" \
  -d '{"addresses": "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t,TLa2f6VPqDgRE67v1736s7bJ8Ray5wYjU7"}'
```

## 📊 系统架构

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   管理界面      │    │   REST API      │    │   WebSocket     │
│   (React)       │    │   批量查询      │    │   实时推送      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────────┐
                    │   核心引擎      │
                    │   (Rust)        │
                    └─────────────────┘
                             │
         ┌───────────────────┼───────────────────┐
         │                   │                   │
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│   区块扫描      │ │   数据存储      │ │   Webhook       │
│   监控充值      │ │   PostgreSQL    │ │   HTTP 回调     │
└─────────────────┘ └─────────────────┘ └─────────────────┘
```

## 🔧 Configuration

The system uses a single configuration file with sensible defaults. All settings can be overridden via environment variables.

### Basic Configuration

```toml
[server]
host = "0.0.0.0"
port = 8080
admin_port = 3000

[blockchain]
start_block = 62800000
batch_size = 100
scan_interval = 3

[database]
url = "postgresql://user:pass@localhost/trontracker"
max_connections = 20

[redis]
url = "redis://localhost:6379"

[tron]
nodes = [
    { url = "https://api.trongrid.io", priority = 1 },
    { url = "https://go.getblock.io", priority = 2 }
]
```

## 📡 API 使用示例

完整 API 文档请查看 [API_DOCUMENTATION.md](API_DOCUMENTATION.md)

### 批量地址查询（核心功能）

```bash
# 查询多个地址的 USDT 交易
curl -X POST http://localhost:8080/api/v1/transactions/multi-address \
  -H "Content-Type: application/json" \
  -d '{
    "addresses": "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t,TLa2f6VPqDgRE67v1736s7bJ8Ray5wYjU7",
    "token": "USDT",
    "limit": 50
  }'
```

### WebSocket 实时监控

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onopen = () => {
  // 订阅地址的充值通知
  ws.send(JSON.stringify({
    type: 'subscribe',
    subscription: {
      event_types: ['transaction'],
      addresses: ['YOUR_WALLET_ADDRESS'],
      tokens: ['USDT']
    }
  }));
};

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  if (data.type === 'TransactionNotification') {
    console.log('收到充值:', data.transaction);
  }
};
```

### Webhook 配置

```bash
# 创建 Webhook 接收充值通知
curl -X POST http://localhost:8080/api/v1/webhooks \
  -H "Content-Type: application/json" \
  -d '{
    "name": "充值通知",
    "url": "https://your-server.com/webhook",
    "secret": "your_secret",
    "events": ["transaction"],
    "filters": {
      "addresses": ["YOUR_WALLET_ADDRESS"],
      "min_amount": "100"
    }
  }'
```

## 🎛️ 管理界面

基于 React + TailwindCSS 的现代化管理界面：

### 功能模块
- **监控面板**: 实时系统状态、交易统计、性能指标
- **交易管理**: 批量查询、交易搜索、导出功能
- **通知配置**: Webhook 管理、WebSocket 连接监控
- **系统设置**: API 密钥、扫描参数、节点配置

### 访问地址
```
开发环境: http://localhost:5173
生产环境: http://localhost:3000
```

## 🔐 Security

### API Authentication
All API endpoints require authentication using API keys:

- **API Keys**: Bearer token authentication

### Webhook Security
- **Signature Verification**: HMAC-SHA256 request signing
- **SSL/TLS**: HTTPS-only webhook delivery
- **Retry Logic**: Exponential backoff for failed deliveries

## 🔧 优化特性

- **连接池**: 数据库连接复用
- **Redis 缓存**: 多层缓存架构
- **批量处理**: 100块/批扫描
- **异步架构**: 全异步非阻塞

## 🚀 Deployment

### Docker Deployment

```bash
# Build and start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Scale WebSocket service
docker-compose up -d --scale websocket=3
```

### Production Configuration

```toml
[server]
host = "0.0.0.0"
port = 8080

[database]
url = "postgresql://user:pass@db.example.com/trontracker"
max_connections = 50

[redis]
url = "redis://cache.example.com:6379"

[logging]
level = "info"
format = "json"
```

## 🔧 Development

### Building from Source

```bash
# Install dependencies
cargo build --release

# Run tests
cargo test

# Run with development config
cargo run -- --config config/development.toml
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## 📚 项目文档

- [PROJECT_OVERVIEW.md](PROJECT_OVERVIEW.md) - 项目概览和功能说明
- [QUICK_START.md](QUICK_START.md) - 快速启动指南
- [API_DOCUMENTATION.md](API_DOCUMENTATION.md) - 完整 API 文档
- [UNIFIED_ARCHITECTURE.md](docs/UNIFIED_ARCHITECTURE.md) - 架构设计文档

## 🆘 Support

### Getting Help
- **Documentation**: Check the docs/ directory for detailed guides
- **Issues**: Report bugs and request features on GitHub
- **Discussions**: Join community discussions for questions and ideas

### System Requirements
- **Minimum**: 2 CPU cores, 4GB RAM, 100GB storage
- **Recommended**: 4+ CPU cores, 8GB+ RAM, SSD storage
- **Network**: Stable internet connection for blockchain access

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Tron Foundation for blockchain infrastructure
- Rust community for excellent tooling and libraries
- Contributors and users for feedback and improvements

---

**TRX Tracker** - 专注于 Tron 节点功能增强的区块链数据服务
