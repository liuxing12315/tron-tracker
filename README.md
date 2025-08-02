# TRX Tracker - 高性能Tron区块链数据服务

专业的Tron区块链数据追踪和通知系统，提供批量地址查询、实时通知、完整管理界面等核心功能。

## ✨ 核心功能

### 🎯 批量地址查询
- **多地址交易查询** - 一次查询最多100个地址
- **智能筛选** - 按时间、金额、代币类型过滤
- **高性能缓存** - Redis多层缓存，毫秒级响应

### 📡 实时通知系统
- **WebSocket推送** - 实时交易事件推送
- **Webhook回调** - HTTP回调通知，支持HMAC签名
- **灵活过滤** - 自定义触发条件和过滤器

### 🎛️ 管理后台
- **系统监控** - 实时状态、性能指标、统计数据
- **交易管理** - 查询、搜索、导出功能
- **配置管理** - API密钥、Webhook、系统配置
- **日志管理** - 查看、过滤、导出系统日志

### ⚡ 技术特点
- **Rust驱动** - 高性能异步处理
- **模块化架构** - 易于扩展和维护
- **生产就绪** - 完整的监控、日志、错误处理
- **RESTful API** - 标准化接口设计

## 🚀 快速启动

### 环境要求
- **Rust** 1.70+
- **PostgreSQL** 13+
- **Redis** 6+
- **Node.js** 18+ (管理界面)

### 一键启动

```bash
# 1. 克隆项目
git clone https://github.com/your-repo/tron-tracker.git
cd tron-tracker

# 2. 安装依赖
brew install postgresql redis  # macOS
# 或者: apt install postgresql redis-server  # Ubuntu

# 3. 初始化数据库
createdb trontracker
psql -d trontracker -f migrations/001_initial.sql

# 4. 启动后端服务
cargo run

# 5. 启动管理界面（新终端）
cd admin-ui && npm install && npm run dev
```

### 验证安装

```bash
# 健康检查
curl http://localhost:8080/health

# 测试批量查询
curl -X POST http://localhost:8080/api/v1/transactions/multi-address \
  -H "Content-Type: application/json" \
  -d '{"addresses": ["TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"]}'

# 访问管理界面
open http://localhost:5173
```

## 🏗️ 系统架构

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   管理后台      │    │   REST API      │    │   WebSocket     │
│   React 19      │    │   批量查询      │    │   实时推送      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────────┐
                    │   TRX Tracker   │
                    │   Rust Core     │
                    └─────────────────┘
                             │
         ┌───────────────────┼───────────────────┐
         │                   │                   │
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│   区块扫描器    │ │   数据存储      │ │   通知系统      │
│   Transaction   │ │   PostgreSQL    │ │   Webhook &     │
│   Scanner       │ │   + Redis       │ │   WebSocket     │
└─────────────────┘ └─────────────────┘ └─────────────────┘
```

## ⚙️ 配置说明

### 默认配置文件 `config/default.toml`

```toml
[server]
host = "0.0.0.0"
port = 8080

[database]
host = "localhost"
port = 5432
database = "trontracker"
username = "postgres"
max_connections = 20

[cache]
enabled = true
redis_url = "redis://localhost:6379"
default_ttl_seconds = 3600

[scanner]
enabled = true
scan_interval_ms = 5000
batch_size = 10
start_block = 62800000

[tron]
nodes = [
    { name = "TronGrid", url = "https://api.trongrid.io", priority = 1 },
    { name = "GetBlock", url = "https://go.getblock.io", priority = 2 }
]
```

### 环境变量覆盖

```bash
export DATABASE_URL="postgresql://user:pass@localhost/trontracker"
export REDIS_URL="redis://localhost:6379"
export RUST_LOG="info"
```

## 📡 API接口示例

### 核心功能 - 批量地址查询

```bash
# 查询多个地址的所有交易
curl -X POST http://localhost:8080/api/v1/transactions/multi-address \
  -H "Content-Type: application/json" \
  -d '{
    "addresses": ["TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t", "TLa2f6VPqDgRE67v1736s7bJ8Ray5wYjU7"],
    "limit": 100,
    "token": "USDT"
  }'
```

### 单地址查询

```bash
# 查询单个地址交易记录
curl "http://localhost:8080/api/v1/addresses/TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t/transactions?limit=50&token=USDT"
```

### WebSocket 实时监控

```javascript
const ws = new WebSocket('ws://localhost:8081');

ws.onopen = () => {
  // 订阅交易通知
  ws.send(JSON.stringify({
    type: 'subscribe',
    subscription: {
      event_types: ['transaction'],
      addresses: ['YOUR_WALLET_ADDRESS'],
      tokens: ['USDT', 'TRX']
    }
  }));
};

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('实时交易:', data);
};
```

### Webhook 通知配置

```bash
# 创建Webhook
curl -X POST http://localhost:8080/api/v1/webhooks \
  -H "Content-Type: application/json" \
  -d '{
    "name": "充值监控",
    "url": "https://your-server.com/webhook",
    "secret": "your_webhook_secret",
    "events": ["transaction"],
    "filters": {
      "addresses": ["YOUR_WALLET_ADDRESS"],
      "min_amount": "100"
    },
    "enabled": true
  }'
```

## 🎛️ 管理后台

现代化的React管理界面，提供完整的系统管理功能：

### 核心模块

#### 📊 监控面板
- 实时系统状态和性能指标
- 交易统计和趋势图表
- 错误监控和告警信息

#### 💰 交易管理
- 批量地址查询工具
- 交易记录搜索和过滤
- 数据导出功能

#### 🔔 通知管理
- Webhook配置和测试
- WebSocket连接监控
- 通知历史记录

#### ⚙️ 系统配置
- API密钥管理
- 扫描器参数设置
- 节点配置和健康检查

#### 📋 日志管理
- 系统日志查看和过滤
- 日志级别设置
- 日志导出和清理

### 访问地址
- **开发环境**: http://localhost:5173
- **生产环境**: http://localhost:3000

## 🔐 安全特性

### API认证
- **Bearer Token**: 使用API密钥进行身份验证
- **权限控制**: 基于角色的访问控制
- **速率限制**: 防止API滥用

### Webhook安全
- **HMAC签名**: SHA-256签名验证
- **重试机制**: 指数退避重试策略
- **HTTPS强制**: 生产环境仅支持HTTPS

## ⚡ 性能优化

- **数据库连接池**: 高效的连接复用
- **多层缓存**: Redis + 内存缓存
- **批量处理**: 优化的区块扫描
- **异步架构**: 全异步非阻塞I/O

## 🚀 部署指南

### Docker部署（推荐）

```bash
# 一键启动所有服务
docker-compose up -d

# 查看运行状态
docker-compose ps

# 查看实时日志
docker-compose logs -f

# 停止服务
docker-compose down
```

### 手动部署

```bash
# 编译生产版本
cargo build --release

# 准备配置文件
cp config/default.toml config/production.toml

# 启动服务
./target/release/tron-tracker --config config/production.toml
```

### 生产环境配置示例

```toml
[server]
host = "0.0.0.0"
port = 8080

[database]
host = "db.example.com"
port = 5432
database = "trontracker"
username = "prod_user"
max_connections = 50

[cache]
redis_url = "redis://cache.example.com:6379"

[logging]
level = "info"
```

## 🛠️ 开发指南

### 从源码构建

```bash
# 安装Rust工具链
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 克隆项目
git clone https://github.com/your-repo/tron-tracker.git
cd tron-tracker

# 构建项目
cargo build

# 运行测试
cargo test

# 启动开发服务器
cargo run
```

### 代码规范

```bash
# 格式化代码
cargo fmt

# 代码检查
cargo clippy

# 运行所有测试
cargo test --all
```

## 📚 技术栈

### 后端技术
- **Rust** - 核心语言，高性能系统编程
- **Axum** - 现代化Web框架
- **SQLx** - 异步数据库驱动
- **Tokio** - 异步运行时
- **Redis** - 缓存和会话存储

### 前端技术
- **React 19** - 用户界面框架
- **Vite** - 构建工具
- **TailwindCSS** - 样式框架
- **shadcn/ui** - 组件库

### 基础设施
- **PostgreSQL** - 主数据库
- **Docker** - 容器化部署
- **WebSocket** - 实时通信

## 📄 许可证

本项目采用 MIT 许可证 - 详情请查看 [LICENSE](LICENSE) 文件

---

**TRX Tracker** - 专业的Tron区块链数据追踪服务
