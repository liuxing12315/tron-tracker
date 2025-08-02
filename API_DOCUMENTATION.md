# TRX Tracker API 文档

专业的Tron区块链数据API服务，提供批量查询、实时通知等核心功能。

## 📋 API概览

### 基础信息
- **Base URL**: `http://localhost:8080`
- **WebSocket URL**: `ws://localhost:8081`
- **认证方式**: API Key (Bearer Token)
- **响应格式**: JSON
- **版本**: v1

### 认证方式
```bash
# 在请求头中添加API密钥
curl -H "Authorization: Bearer YOUR_API_KEY" \
     "http://localhost:8080/api/v1/transactions"
```

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

## 🎯 核心API接口

### 1. 批量地址查询 (核心功能)

批量查询多个地址的交易记录，支持最多100个地址同时查询。

**端点**: `POST /api/v1/transactions/multi-address`

**请求体**:
```json
{
  "addresses": ["TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t", "TLa2f6VPqDgRE67v1736s7bJ8Ray5wYjU7"],
  "limit": 100,
  "page": 1,
  "token": "USDT",
  "status": "success",
  "min_amount": "100",
  "max_amount": "10000",
  "start_time": 1704067200,
  "end_time": 1704153600
}
```

**响应示例**:
```json
{
  "success": true,
  "data": {
    "transactions": [
      {
        "hash": "0x...",
        "block_number": 62845149,
        "from_address": "TR7NHqjeK...",
        "to_address": "TLa2f6VPq...",
        "value": "1000000000",
        "token_symbol": "USDT",
        "timestamp": "2024-01-15T10:30:15Z",
        "status": "success"
      }
    ],
    "summary": {
      "total_transactions": 1250,
      "total_addresses": 2,
      "date_range": {
        "start": "2024-01-01T00:00:00Z",
        "end": "2024-01-15T23:59:59Z"
      }
    }
  },
  "pagination": {
    "page": 1,
    "limit": 100,
    "total": 1250,
    "total_pages": 13
  }
}
```

### 2. 单地址查询

**端点**: `GET /api/v1/addresses/{address}/transactions`

**查询参数**:
- `limit`: 返回条数 (默认50)
- `page`: 页码 (默认1)
- `token`: 代币筛选
- `type`: 交易类型 (in/out/all)

**请求示例**:
```bash
curl "http://localhost:8080/api/v1/addresses/TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t/transactions?limit=50&token=USDT"
```

### 3. 交易详情查询

**端点**: `GET /api/v1/transactions/{hash}`

**响应示例**:
```json
{
  "success": true,
  "data": {
    "hash": "0x...",
    "block_number": 62845149,
    "block_hash": "0x...",
    "from_address": "TR7NHqjeK...",
    "to_address": "TLa2f6VPq...",
    "value": "1000000000",
    "token_address": "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t",
    "token_symbol": "USDT",
    "token_decimals": 6,
    "gas_used": 65000,
    "gas_price": 420,
    "status": "success",
    "timestamp": "2024-01-15T10:30:15Z",
    "confirmations": 1000
  }
}
```

## 🔔 Webhook管理

### 创建Webhook

**端点**: `POST /api/v1/webhooks`

**请求体**:
```json
{
  "name": "充值监控",
  "url": "https://your-server.com/webhook",
  "secret": "your_webhook_secret",
  "events": ["transaction"],
  "filters": {
    "addresses": ["YOUR_WALLET_ADDRESS"],
    "min_amount": "100",
    "tokens": ["USDT", "TRX"]
  },
  "enabled": true
}
```

### 获取Webhook列表

**端点**: `GET /api/v1/webhooks`

### 测试Webhook

**端点**: `POST /api/v1/webhooks/{webhook_id}/test`

### Webhook通知格式

当触发条件满足时，系统会向您的URL发送POST请求：

```json
{
  "event_type": "transaction",
  "timestamp": "2024-01-15T10:30:15Z",
  "data": {
    "transaction": {
      "hash": "0x...",
      "from_address": "TR7NHqjeK...",
      "to_address": "TLa2f6VPq...",
      "value": "1000000000",
      "token_symbol": "USDT"
    }
  },
  "webhook_id": "uuid-here"
}
```

## 🔌 WebSocket实时推送

### 连接WebSocket

```javascript
const ws = new WebSocket('ws://localhost:8081');

ws.onopen = () => {
  console.log('WebSocket连接已建立');
};
```

### 订阅交易通知

```javascript
ws.send(JSON.stringify({
  type: 'subscribe',
  subscription: {
    id: 'my-subscription-1',
    event_types: ['transaction'],
    addresses: ['YOUR_WALLET_ADDRESS'],
    tokens: ['USDT', 'TRX'],
    min_amount: '100'
  }
}));
```

### 接收通知

```javascript
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  
  if (data.type === 'transaction_notification') {
    console.log('收到新交易:', data.transaction);
  }
};
```

### 取消订阅

```javascript
ws.send(JSON.stringify({
  type: 'unsubscribe',
  subscription_id: 'my-subscription-1'
}));
```

## 🔑 API密钥管理

### 创建API密钥

**端点**: `POST /api/v1/api-keys`

**请求体**:
```json
{
  "name": "我的应用密钥",
  "permissions": ["read_transactions", "manage_webhooks"],
  "rate_limit": 1000,
  "expires_in_days": 365
}
```

### 获取密钥列表

**端点**: `GET /api/v1/api-keys`

### 密钥使用统计

**端点**: `GET /api/v1/api-keys/{key_id}/usage`

## 🎛️ 管理接口

### 系统健康检查

**端点**: `GET /health`

**响应示例**:
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T10:30:15Z",
  "version": "2.0.0",
  "components": {
    "database": "healthy",
    "cache": "healthy",
    "scanner": "healthy"
  }
}
```

### 获取系统统计

**端点**: `GET /admin/dashboard/stats`

### 日志管理

- `GET /admin/logs` - 获取系统日志
- `DELETE /admin/logs` - 清空日志
- `GET /admin/logs/export` - 导出日志

### 扫描器控制

- `POST /admin/scanner/restart` - 重启扫描器
- `POST /admin/scanner/stop` - 停止扫描器
- `POST /admin/scanner/scan/{block_number}` - 手动扫描指定区块

## 📊 响应状态码

| 状态码 | 说明 |
|--------|------|
| 200 | 请求成功 |
| 201 | 创建成功 |
| 400 | 请求参数错误 |
| 401 | 未授权 |
| 403 | 权限不足 |
| 404 | 资源不存在 |
| 429 | 请求过于频繁 |
| 500 | 服务器内部错误 |

## 🚨 错误处理

### 错误响应格式

```json
{
  "success": false,
  "error": {
    "code": "INVALID_ADDRESS",
    "message": "Invalid address format",
    "details": "Address must be a valid Tron address starting with T"
  },
  "data": null
}
```

### 常见错误码

| 错误码 | 说明 |
|--------|------|
| `INVALID_ADDRESS` | 地址格式错误 |
| `ADDRESS_LIMIT_EXCEEDED` | 地址数量超限 |
| `INVALID_TOKEN` | 无效的代币类型 |
| `RATE_LIMIT_EXCEEDED` | 请求频率超限 |
| `INSUFFICIENT_PERMISSIONS` | 权限不足 |

## 📝 使用限制

### 速率限制
- **默认限制**: 1000 请求/小时
- **批量查询**: 最多100个地址
- **WebSocket连接**: 每个IP最多10个连接

### 数据限制
- **历史数据**: 支持查询最近6个月的数据
- **实时数据**: 延迟通常在1-3秒
- **分页限制**: 单次查询最多返回1000条记录

## 🔧 SDK和示例

### JavaScript/Node.js示例

```javascript
const TronTracker = require('tron-tracker-sdk');

const client = new TronTracker({
  apiKey: 'YOUR_API_KEY',
  baseUrl: 'http://localhost:8080'
});

// 批量查询
const transactions = await client.transactions.multiAddress({
  addresses: ['TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t'],
  token: 'USDT',
  limit: 100
});

console.log(transactions);
```

### Python示例

```python
import requests

def query_transactions(addresses, token='USDT'):
    response = requests.post(
        'http://localhost:8080/api/v1/transactions/multi-address',
        headers={'Authorization': 'Bearer YOUR_API_KEY'},
        json={
            'addresses': addresses,
            'token': token,
            'limit': 100
        }
    )
    return response.json()

# 使用示例
result = query_transactions(['TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t'])
print(result)
```

---

**TRX Tracker API** - 专业的Tron区块链数据接口服务