# TRX Tracker API 文档

## 概述

TRX Tracker 提供 RESTful API 和 WebSocket 接口，支持批量查询交易、实时通知等 Tron 节点不提供的功能。

### 基础信息
- **Base URL**: `http://localhost:8080`
- **WebSocket URL**: `ws://localhost:8080/ws`
- **认证方式**: API Key (Bearer Token)
- **响应格式**: JSON

### 通用响应格式
```json
{
  "success": true,
  "data": {},
  "error": null,
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 100,
    "total_pages": 5
  }
}
```

## 核心功能 API

### 1. 批量地址交易查询

**端点**: `POST /api/v1/transactions/multi-address`

**描述**: 批量查询多个地址的交易记录，最多支持100个地址同时查询。

**请求体**:
```json
{
  "addresses": "address1,address2,address3",  // 逗号分隔的地址列表
  "page": 1,
  "limit": 50,
  "token": "USDT",                            // 可选：筛选特定代币
  "status": "success",                        // 可选：success/failed/pending
  "min_amount": "100",                        // 可选：最小金额
  "max_amount": "10000",                      // 可选：最大金额
  "start_time": 1704067200,                   // 可选：开始时间戳
  "end_time": 1704153600,                     // 可选：结束时间戳
  "group_by_address": false                   // 可选：是否按地址分组
}
```

**响应示例**:
```json
{
  "success": true,
  "data": {
    "transactions": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "hash": "0x1a2b3c...",
        "block_number": 62845149,
        "from_address": "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t",
        "to_address": "TLa2f6VPqDgRE67v1736s7bJ8Ray5wYjU7",
        "value": "1250.50",
        "token_symbol": "USDT",
        "status": "success",
        "timestamp": "2024-01-15T10:30:00Z"
      }
    ],
    "address_stats": {
      "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t": {
        "total_transactions": 156,
        "total_sent": "50000.00",
        "total_received": "45000.00"
      }
    },
    "query_time_ms": 125,
    "cache_hit": false
  },
  "pagination": {
    "page": 1,
    "limit": 50,
    "total": 234,
    "total_pages": 5
  }
}
```

### 2. 单地址交易查询

**端点**: `GET /api/v1/addresses/:address/transactions`

**查询参数**:
- `page`: 页码（默认：1）
- `limit`: 每页数量（默认：20，最大：100）
- `token`: 代币类型筛选
- `status`: 交易状态筛选

### 3. 获取交易详情

**端点**: `GET /api/v1/transactions/:hash`

**响应示例**:
```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "hash": "0x1a2b3c...",
    "block_number": 62845149,
    "block_hash": "0xdef456...",
    "from_address": "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t",
    "to_address": "TLa2f6VPqDgRE67v1736s7bJ8Ray5wYjU7",
    "value": "1250.50",
    "token_address": "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t",
    "token_symbol": "USDT",
    "token_decimals": 6,
    "gas_used": 14500,
    "gas_price": "420",
    "status": "success",
    "timestamp": "2024-01-15T10:30:00Z"
  }
}
```

## WebSocket API

### 连接建立

**URL**: `ws://localhost:8080/ws`

**连接成功响应**:
```json
{
  "type": "Connected",
  "connection_id": "conn_123456",
  "server_time": 1704153600
}
```

### 订阅交易事件

**订阅请求**:
```json
{
  "type": "subscribe",
  "subscription": {
    "event_types": ["transaction", "large_transfer"],
    "addresses": ["TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"],
    "tokens": ["USDT", "TRX"],
    "min_amount": "1000"
  }
}
```

**交易通知**:
```json
{
  "type": "TransactionNotification",
  "transaction": {
    "hash": "0x1a2b3c...",
    "from_address": "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t",
    "to_address": "TLa2f6VPqDgRE67v1736s7bJ8Ray5wYjU7",
    "value": "5000.00",
    "token_symbol": "USDT",
    "timestamp": "2024-01-15T10:30:00Z"
  },
  "event_type": "transaction",
  "subscription_id": "sub_789"
}
```

### 心跳保持

**Ping**:
```json
{
  "type": "ping",
  "timestamp": 1704153600
}
```

**Pong**:
```json
{
  "type": "pong",
  "timestamp": 1704153600
}
```

## Webhook 管理 API

### 1. 创建 Webhook

**端点**: `POST /api/v1/webhooks`

**请求体**:
```json
{
  "name": "Payment Notifications",
  "url": "https://your-server.com/webhook",
  "secret": "webhook_secret_key",
  "events": ["transaction", "large_transfer"],
  "filters": {
    "addresses": ["TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"],
    "tokens": ["USDT"],
    "min_amount": "1000"
  }
}
```

### 2. Webhook 通知格式

**Headers**:
```
X-Webhook-Signature: sha256=abcdef123456...
X-Webhook-Timestamp: 1704153600
X-Webhook-Event: transaction
```

**Body**:
```json
{
  "event": "transaction",
  "timestamp": 1704153600,
  "data": {
    "transaction": {
      "hash": "0x1a2b3c...",
      "from_address": "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t",
      "to_address": "TLa2f6VPqDgRE67v1736s7bJ8Ray5wYjU7",
      "value": "5000.00",
      "token_symbol": "USDT"
    }
  }
}
```

**签名验证** (Node.js):
```javascript
const crypto = require('crypto');

function verifyWebhookSignature(payload, signature, secret) {
  const hash = crypto
    .createHmac('sha256', secret)
    .update(payload)
    .digest('hex');
  
  return `sha256=${hash}` === signature;
}
```

## 管理 API

### 1. 系统统计

**端点**: `GET /api/v1/dashboard/stats`

**响应**:
```json
{
  "success": true,
  "data": {
    "total_transactions": 1247856,
    "total_addresses": 89234,
    "current_block": 62845149,
    "scan_speed": 18.5,
    "active_webhooks": 12,
    "websocket_connections": 156,
    "api_requests_today": 45678,
    "success_rate": 99.2,
    "uptime": 2847600
  }
}
```

### 2. WebSocket 连接管理

**端点**: `GET /api/v1/websockets/connections`

**响应**:
```json
{
  "success": true,
  "data": {
    "connections": [
      {
        "id": "conn_123",
        "client_ip": "192.168.1.100",
        "connected_at": "2024-01-15T10:30:00Z",
        "subscriptions": ["sub_789"],
        "message_count": 156
      }
    ]
  }
}
```

## 错误处理

### 错误响应格式
```json
{
  "success": false,
  "data": null,
  "error": "Error message description"
}
```

### 常见错误码
- `400 Bad Request`: 请求参数错误
- `401 Unauthorized`: 未授权访问
- `404 Not Found`: 资源不存在
- `500 Internal Server Error`: 服务器内部错误

## 认证

### API Key 认证

**Header**:
```
Authorization: Bearer your_api_key_here
```

### 获取 API Key
通过管理界面创建和管理 API Key：
1. 访问 http://localhost:3000/api-keys
2. 点击 "Create API Key"
3. 设置权限
4. 保存生成的 Key（只显示一次）
