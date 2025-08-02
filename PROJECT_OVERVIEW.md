# TRX Tracker 项目概览

## 项目定位

TRX Tracker 是一个专注于提供 Tron 节点原生不支持功能的区块链数据服务系统。通过扫描区块链并建立本地索引，提供高效的批量查询和实时通知能力。

## 核心功能

### 1. 批量钱包交易查询 ✅
**功能描述**：支持一次查询多个钱包地址的交易记录，这是 Tron 节点无法直接提供的功能。

**实现状态**：
- ✅ 后端API已实现 (`src/api/handlers/transaction.rs::get_multi_address_transactions`)
- ✅ 支持最多100个地址批量查询
- ✅ 支持多维度过滤（时间、金额、代币类型、交易状态）
- ✅ Redis缓存优化查询性能
- ⚠️ API路由需要激活（取消注释）

**API端点**：
```
POST /api/v1/transactions/multi-address
{
  "addresses": ["address1", "address2", ...],
  "page": 1,
  "limit": 50,
  "token": "USDT",
  "status": "success"
}
```

### 2. 实时充值通知 ✅
**功能描述**：通过扫描区块链，监控特定地址的充值交易，并通过 WebSocket 或 Webhook 实时通知。

#### 2.1 扫块服务
**实现状态**：
- ✅ 完整的区块扫描引擎 (`src/services/scanner.rs`)
- ✅ 支持断点续扫
- ✅ 批量处理（默认100块/批）
- ✅ 交易事件发送机制
- ⚠️ 需要在 main.rs 中启动服务

#### 2.2 WebSocket 通知
**实现状态**：
- ✅ WebSocket 服务实现 (`src/services/websocket.rs`)
- ✅ 支持订阅特定地址/代币
- ✅ 实时推送交易事件
- ✅ 连接管理和心跳机制
- ⚠️ 需要激活 `/ws` 路由

**订阅示例**：
```javascript
ws.send(JSON.stringify({
  type: 'subscribe',
  subscription: {
    event_types: ['transaction'],
    addresses: ['TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t'],
    tokens: ['USDT']
  }
}))
```

#### 2.3 Webhook 通知
**实现状态**：
- ✅ Webhook 投递服务 (`src/services/webhook.rs`)
- ✅ HMAC-SHA256 签名验证
- ✅ 重试机制（指数退避）
- ✅ 过滤规则（地址、代币、金额）
- ⚠️ 需要连接到扫描器事件流

### 3. 管理后台前端 ✅
**功能描述**：提供 Web 界面管理和监控整个系统。

**实现状态**：
- ✅ React + Vite + TailwindCSS 技术栈
- ✅ 8个管理页面全部实现
- ✅ 响应式设计
- ✅ 实时数据展示
- ⚠️ 当前使用 mock 数据，需对接真实 API

**管理功能**：
- Dashboard：系统概览和实时监控
- Transactions：交易查询和批量地址查询
- Addresses：地址管理
- Webhooks：Webhook 配置管理
- WebSockets：WebSocket 连接监控
- API Keys：API 密钥管理
- Settings：系统配置
- Logs：日志查看

### 4. 后台管理API ✅
**功能描述**：为管理前端提供完整的 RESTful API。

**实现状态**：
- ✅ 统一的 API 响应格式
- ✅ 分页支持
- ✅ 错误处理
- ⚠️ 部分端点被注释，需要激活

**主要端点**：
```
GET  /api/v1/dashboard/stats         # 系统统计
GET  /api/v1/transactions           # 交易列表
POST /api/v1/transactions/multi-address  # 批量查询
GET  /api/v1/webhooks               # Webhook列表
GET  /api/v1/websockets/connections # WebSocket连接
GET  /api/v1/api-keys               # API密钥列表
```

## 配套功能

### 鉴权系统 ✅
- ✅ API Key 认证机制
- ✅ 权限控制
- ✅ IP 白名单

### 配置系统 ✅
- ✅ TOML 配置文件
- ✅ 环境变量覆盖
- ✅ 热重载支持

### 缓存系统 ✅
- ✅ Redis 多层缓存
- ✅ 自动过期管理
- ✅ 缓存预热

### 数据库系统 ✅
- ✅ PostgreSQL 数据存储
- ✅ 自动迁移
- ✅ 连接池管理

## 部署要求

### 系统依赖
- PostgreSQL 14+
- Redis 6+
- Rust 1.70+
- Node.js 18+ (前端构建)

### 环境配置
```bash
# 数据库
DATABASE_URL=postgresql://user:pass@localhost/trontracker

# Redis
REDIS_URL=redis://localhost:6379

# 区块链
BLOCKCHAIN_START_BLOCK=62800000

# 认证
JWT_SECRET=your-secret-key
```

## 启动步骤

### 1. 后端服务
```bash
# 安装依赖
cargo build --release

# 运行迁移
psql -U postgres -d trontracker -f migrations/001_initial.sql

# 启动服务
cargo run --release
```

### 2. 前端服务
```bash
cd admin-ui
pnpm install
pnpm build
pnpm preview
```

### 3. Docker 部署
```bash
docker-compose up -d
```

## 待完成工作

1. **激活被注释的路由**
   - 取消 `src/api/mod.rs` 中被注释的路由
   - 激活 WebSocket 路由 `/ws`

2. **连接服务组件**
   - 在 `main.rs` 中启动 Scanner 服务
   - 连接 Scanner 事件到 WebSocket/Webhook 服务

3. **前端API对接**
   - 将前端 mock 数据改为真实 API 调用
   - 添加 API 认证处理

4. **生产环境配置**
   - 配置生产数据库连接
   - 设置 Redis 集群
   - 配置 Tron 节点连接

## 项目特色

1. **专注差异化**：只做 Tron 节点不提供的功能
2. **高性能设计**：Rust 实现，异步架构
3. **实时性保证**：WebSocket + Webhook 双通道
4. **易于管理**：完整的 Web 管理界面
5. **生产就绪**：包含监控、日志、错误处理等完整功能
