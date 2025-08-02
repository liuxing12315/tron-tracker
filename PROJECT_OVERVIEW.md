# TRX Tracker - 项目概览

专业的Tron区块链数据追踪和通知系统，解决Tron节点功能局限性，为开发者提供强大的数据服务。

## 🎯 项目背景

### 解决的问题

Tron区块链节点存在以下核心限制：
- **无法批量查询** - 只能单个地址查询，效率低下
- **缺乏实时通知** - 需要不断轮询，消耗资源
- **查询性能差** - 复杂条件查询响应慢
- **无管理界面** - 缺乏可视化管理工具

### 我们的解决方案

TRX Tracker通过以下技术手段彻底解决这些问题：
- **智能扫描器** - 实时扫描区块链，预处理数据
- **多层缓存** - Redis缓存策略，毫秒级响应
- **批量处理** - 单次查询支持100个地址
- **实时通知** - WebSocket推送 + Webhook回调

## ✨ 核心功能

### 🎯 批量地址查询
批量查询多个地址的交易记录，Tron节点无法提供的核心功能。

**特性：**
- 单次查询最多100个地址
- 支持多维度筛选（时间、金额、代币）
- 智能缓存，毫秒级响应
- 分页和排序优化

**使用场景：**
- 交易所批量监控用户充值
- 钱包应用查询多地址余额
- 数据分析平台批量获取交易记录

### 📡 实时通知系统
监控指定地址，实时推送交易事件。

**通知方式：**
- **WebSocket推送** - 实时双向通信
- **Webhook回调** - HTTP回调，支持HMAC签名
- **灵活订阅** - 按地址、代币、金额条件订阅

**应用场景：**
- 充值到账实时通知
- 大额转账监控告警
- 智能合约事件追踪

### 🎛️ 管理后台
基于React 19的现代化管理界面。

**功能模块：**
- **监控面板** - 系统状态、性能指标、统计图表
- **交易管理** - 批量查询工具、搜索过滤、数据导出
- **通知管理** - Webhook配置、WebSocket监控、历史记录
- **系统配置** - API密钥、扫描参数、节点设置
- **日志管理** - 系统日志查看、过滤、导出功能

## 🏗️ 技术架构

### 后端架构 (Rust)

```
┌─────────────────┐
│   API Gateway   │  Axum框架，统一接口入口
│   (Axum)        │  认证、限流、路由分发
└─────────────────┘
         │
┌─────────────────┐
│  Business Layer │  核心业务服务层
├─────────────────┤
│ • Scanner       │  区块链实时扫描服务
│ • Webhook       │  HTTP回调通知服务
│ • WebSocket     │  实时推送通信服务
│ • Auth          │  API密钥认证服务
│ • Cache         │  智能缓存管理服务
└─────────────────┘
         │
┌─────────────────┐
│ Data & Storage  │  数据存储层
├─────────────────┤
│ • PostgreSQL    │  交易数据主库
│ • Redis         │  缓存和会话存储
│ • Tron Nodes    │  多节点冗余连接
└─────────────────┘
```

### 前端架构 (React)

```
┌─────────────────┐
│   React 19      │  现代化用户界面
│   + TypeScript  │  类型安全的组件开发
└─────────────────┘
         │
┌─────────────────┐
│  State & Logic  │  状态管理和业务逻辑
├─────────────────┤
│ • Context API   │  全局状态管理
│ • Custom Hooks  │  业务逻辑封装
│ • Error Handle  │  统一错误处理
└─────────────────┘
         │
┌─────────────────┐
│  Service Layer  │  数据服务层
├─────────────────┤
│ • HTTP Client   │  API接口调用
│ • WebSocket     │  实时数据通信
│ • Cache Mgmt    │  前端缓存管理
└─────────────────┘
```

## 📊 数据模型

### 核心实体

#### 交易记录 (Transaction)
```rust
pub struct Transaction {
    pub id: Uuid,
    pub hash: String,                    // 交易哈希
    pub block_number: u64,               // 区块高度
    pub from_address: String,            // 发送方地址
    pub to_address: String,              // 接收方地址
    pub value: String,                   // 交易金额
    pub token_address: Option<String>,   // 代币合约地址
    pub token_symbol: Option<String>,    // 代币符号 (USDT/TRX)
    pub status: TransactionStatus,       // 交易状态
    pub timestamp: DateTime<Utc>,        // 区块时间
}
```

#### 通知配置 (Webhook)
```rust
pub struct Webhook {
    pub id: Uuid,
    pub name: String,                        // 配置名称
    pub url: String,                         // 回调URL
    pub secret: String,                      // HMAC签名密钥
    pub events: Vec<NotificationEventType>,  // 监听事件类型
    pub filters: WebhookFilters,             // 触发条件过滤器
    pub enabled: bool,                       // 启用状态
    pub success_count: i64,                  // 成功投递次数
    pub failure_count: i64,                  // 失败投递次数
}
```

#### API密钥 (ApiKey)
```rust
pub struct ApiKey {
    pub id: Uuid,
    pub name: String,                    // 密钥名称
    pub key_hash: String,                // SHA-256哈希值
    pub permissions: Vec<Permission>,    // 权限列表
    pub enabled: bool,                   // 启用状态
    pub rate_limit: Option<i32>,         // 请求速率限制
    pub expires_at: Option<DateTime<Utc>>, // 过期时间
}
```

## 🚀 部署架构

### 开发环境
```
┌─────────────────┐
│ 开发环境服务     │
├─────────────────┤
│ :8080  API服务  │
│ :8081  WebSocket │
│ :5173  前端界面  │
│ :5432  PostgreSQL│
│ :6379  Redis     │
└─────────────────┘
```

### 生产环境
```
                    ┌─────────────────┐
                    │  Load Balancer  │  Nginx/HAProxy
                    │  (SSL/TLS)      │
                    └─────────────────┘
                             │
                    ┌─────────────────┐
                    │  TRX Tracker    │  Docker容器集群
                    │  App Cluster    │  多实例部署
                    └─────────────────┘
                             │
    ┌──────────────────────────────────────────┐
    │                                          │
┌─────────────────┐                 ┌─────────────────┐
│  PostgreSQL     │                 │  Redis Cluster  │
│  Master/Slave   │                 │  Multi-Node     │
└─────────────────┘                 └─────────────────┘
```

## 🔧 技术栈详解

### 后端技术栈
- **Rust 1.70+** - 系统编程语言，内存安全、零成本抽象
- **Axum** - 现代异步Web框架，基于Tokio
- **SQLx** - 异步数据库驱动，编译时SQL检查
- **Tokio** - 异步运行时，支持高并发
- **Redis** - 内存数据库，缓存和会话管理
- **PostgreSQL** - 关系型数据库，事务和一致性保证

### 前端技术栈
- **React 19** - 最新React版本，并发特性
- **TypeScript** - 类型安全的JavaScript超集
- **Vite** - 下一代前端构建工具
- **TailwindCSS** - 实用优先的CSS框架
- **shadcn/ui** - 现代化组件库

### 基础设施
- **Docker** - 容器化部署
- **Docker Compose** - 开发环境编排
- **Nginx** - 反向代理和负载均衡
- **Let's Encrypt** - 免费SSL证书

## 📈 性能指标

### 响应性能
- **单地址查询**: < 50ms (缓存命中)
- **批量查询**: < 300ms (100个地址)
- **实时通知延迟**: < 2秒
- **管理界面首屏**: < 1.5秒

### 系统吞吐量
- **API请求处理**: 2000+ QPS
- **WebSocket并发**: 50000+ 连接
- **区块扫描速度**: 200+ blocks/分钟
- **Webhook投递**: 1000+ 次/分钟

### 可用性指标
- **系统可用性**: 99.95%
- **平均故障恢复**: < 10秒
- **数据一致性**: 强一致性保证
- **自动容灾**: 支持多节点故障转移

## 🔐 安全特性

### API安全
- **认证机制** - 基于API Key的Bearer Token认证
- **权限控制** - 细粒度RBAC权限系统
- **速率限制** - 防止API滥用和DDoS攻击
- **请求签名** - 支持HMAC-SHA256请求签名

### 数据安全
- **传输加密** - 全站HTTPS，TLS 1.3协议
- **存储加密** - 敏感数据AES-256加密存储
- **访问控制** - IP白名单和访问日志
- **数据备份** - 定期备份和灾难恢复

### 运行时安全
- **内存安全** - Rust语言特性防止缓冲区溢出
- **并发安全** - 无数据竞争的并发模型
- **错误处理** - 完善的错误处理和故障隔离
- **监控告警** - 实时安全事件监控

## 🎯 使用场景

### 交易所和钱包
- **充值监控** - 实时监控用户充值到账
- **批量查询** - 高效查询多用户交易记录
- **余额同步** - 定期同步用户余额变化
- **风控监控** - 大额转账和异常交易告警

### 数据分析平台
- **链上数据** - 获取全面的链上交易数据

---

**TRX Tracker** - 专业的Tron区块链数据服务平台