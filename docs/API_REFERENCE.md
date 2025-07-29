# TRX Tracker - API Reference

This document provides a comprehensive reference for the TRX Tracker REST API and WebSocket protocol.

## Base URL

```
Production: https://api.trontracker.com
Development: http://localhost:8080
```

## Authentication

All API endpoints require authentication using API keys. Include your API key in the Authorization header:

```
Authorization: Bearer tk_live_1234567890abcdef...
```

## Response Format

All API responses follow a consistent format:

```json
{
  "success": true,
  "data": { ... },
  "error": null,
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 1000,
    "total_pages": 50
  }
}
```

## Error Handling

Error responses include detailed information:

```json
{
  "success": false,
  "data": null,
  "error": "Invalid transaction hash format",
  "pagination": null
}
```

## Rate Limiting

API requests are rate limited based on your API key tier:

- **Free Tier**: 100 requests/minute
- **Pro Tier**: 1,000 requests/minute  
- **Enterprise Tier**: 10,000 requests/minute

Rate limit headers are included in all responses:

```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1640995200
```

## Endpoints

### Health Check

#### GET /health

Check system health and status.

**Response:**
```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "version": "2.0.0",
    "timestamp": "2024-01-15T10:30:00Z"
  }
}
```

### Transactions

#### GET /api/v1/transactions

List transactions with optional filtering.

**Query Parameters:**
- `page` (integer, optional): Page number (default: 1)
- `limit` (integer, optional): Items per page (default: 20, max: 100)
- `status` (string, optional): Filter by status (`success`, `failed`, `pending`)
- `token` (string, optional): Filter by token (`TRX`, `USDT`)
- `from_address` (string, optional): Filter by sender address
- `to_address` (string, optional): Filter by recipient address
- `min_amount` (string, optional): Minimum transaction amount
- `max_amount` (string, optional): Maximum transaction amount
- `start_time` (string, optional): Start time (ISO 8601)
- `end_time` (string, optional): End time (ISO 8601)

**Example Request:**
```bash
curl -H "Authorization: Bearer tk_live_..." \
  "https://api.trontracker.com/api/v1/transactions?status=success&token=USDT&limit=50"
```

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "hash": "0x1a2b3c4d5e6f7890abcdef1234567890abcdef1234567890abcdef1234567890",
      "block_number": 62845149,
      "from_address": "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t",
      "to_address": "TLa2f6VPqDgRE67v1736s7bJ8Ray5wYjU7",
      "amount": "1250.500000",
      "token": "USDT",
      "status": "success",
      "timestamp": "2024-01-15T10:30:00Z",
      "gas_used": 14000,
      "gas_price": 420
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 50,
    "total": 1234567,
    "total_pages": 24692
  }
}
```

#### GET /api/v1/transactions/:hash

Get a specific transaction by hash.

**Path Parameters:**
- `hash` (string): Transaction hash (64 character hex string)

**Example Request:**
```bash
curl -H "Authorization: Bearer tk_live_..." \
  "https://api.trontracker.com/api/v1/transactions/0x1a2b3c4d5e6f7890abcdef1234567890abcdef1234567890abcdef1234567890"
```

**Response:**
```json
{
  "success": true,
  "data": {
    "hash": "0x1a2b3c4d5e6f7890abcdef1234567890abcdef1234567890abcdef1234567890",
    "block_number": 62845149,
    "from_address": "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t",
    "to_address": "TLa2f6VPqDgRE67v1736s7bJ8Ray5wYjU7",
    "amount": "1250.500000",
    "token": "USDT",
    "status": "success",
    "timestamp": "2024-01-15T10:30:00Z",
    "gas_used": 14000,
    "gas_price": 420,
    "confirmations": 1234,
    "contract_address": "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"
  }
}
```

#### POST /api/v1/transactions/multi-address

Query transactions for multiple addresses simultaneously.

**Request Body:**
```json
{
  "addresses": [
    "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t",
    "TLa2f6VPqDgRE67v1736s7bJ8Ray5wYjU7",
    "TKzxdSv2DAE9Hf8M2AyT6UqCsQMKMg2Ax"
  ],
  "page": 1,
  "limit": 50,
  "status": "success",
  "token": "USDT",
  "start_time": "2024-01-01T00:00:00Z",
  "end_time": "2024-01-31T23:59:59Z"
}
```

**Example Request:**
```bash
curl -X POST \
  -H "Authorization: Bearer tk_live_..." \
  -H "Content-Type: application/json" \
  -d '{
    "addresses": ["TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t", "TLa2f6VPqDgRE67v1736s7bJ8Ray5wYjU7"],
    "limit": 100,
    "status": "success"
  }' \
  "https://api.trontracker.com/api/v1/transactions/multi-address"
```

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "hash": "0x...",
      "block_number": 62845149,
      "from_address": "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t",
      "to_address": "TLa2f6VPqDgRE67v1736s7bJ8Ray5wYjU7",
      "amount": "1250.500000",
      "token": "USDT",
      "status": "success",
      "timestamp": "2024-01-15T10:30:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 100,
    "total": 456,
    "total_pages": 5
  }
}
```

### Addresses

#### GET /api/v1/addresses/:address

Get address information and statistics.

**Path Parameters:**
- `address` (string): Tron address (34 character string starting with 'T')

**Example Request:**
```bash
curl -H "Authorization: Bearer tk_live_..." \
  "https://api.trontracker.com/api/v1/addresses/TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"
```

**Response:**
```json
{
  "success": true,
  "data": {
    "address": "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t",
    "balance": {
      "TRX": "1000.000000",
      "USDT": "5000.000000"
    },
    "transaction_count": 1234,
    "first_seen": "2023-01-01T00:00:00Z",
    "last_seen": "2024-01-15T10:30:00Z",
    "is_contract": true,
    "contract_type": "TRC20"
  }
}
```

#### GET /api/v1/addresses/:address/transactions

Get transactions for a specific address.

**Path Parameters:**
- `address` (string): Tron address

**Query Parameters:**
- `page` (integer, optional): Page number (default: 1)
- `limit` (integer, optional): Items per page (default: 20, max: 100)
- `direction` (string, optional): Transaction direction (`in`, `out`, `all`)
- `token` (string, optional): Filter by token
- `start_time` (string, optional): Start time (ISO 8601)
- `end_time` (string, optional): End time (ISO 8601)

**Example Request:**
```bash
curl -H "Authorization: Bearer tk_live_..." \
  "https://api.trontracker.com/api/v1/addresses/TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t/transactions?direction=in&token=USDT"
```

### Webhooks

#### GET /api/v1/webhooks

List all webhooks for your account.

**Query Parameters:**
- `page` (integer, optional): Page number (default: 1)
- `limit` (integer, optional): Items per page (default: 20, max: 100)
- `enabled` (boolean, optional): Filter by enabled status

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": "wh_1234567890",
      "name": "Payment Notifications",
      "url": "https://api.example.com/webhooks/payments",
      "events": ["transaction"],
      "filters": {
        "addresses": ["TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"],
        "tokens": ["USDT"],
        "min_amount": "100"
      },
      "enabled": true,
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-15T10:30:00Z",
      "stats": {
        "total_deliveries": 1234,
        "successful_deliveries": 1200,
        "failed_deliveries": 34,
        "last_delivery": "2024-01-15T10:25:00Z"
      }
    }
  ]
}
```

#### POST /api/v1/webhooks

Create a new webhook.

**Request Body:**
```json
{
  "name": "Payment Notifications",
  "url": "https://api.example.com/webhooks/payments",
  "secret": "your_webhook_secret",
  "events": ["transaction", "large_transfer"],
  "filters": {
    "addresses": ["TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"],
    "tokens": ["USDT"],
    "min_amount": "100"
  },
  "enabled": true,
  "timeout": 30,
  "retry_count": 3
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "wh_1234567890",
    "name": "Payment Notifications",
    "url": "https://api.example.com/webhooks/payments",
    "events": ["transaction", "large_transfer"],
    "filters": {
      "addresses": ["TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"],
      "tokens": ["USDT"],
      "min_amount": "100"
    },
    "enabled": true,
    "created_at": "2024-01-15T10:30:00Z"
  }
}
```

#### GET /api/v1/webhooks/:id

Get a specific webhook.

**Path Parameters:**
- `id` (string): Webhook ID

#### PUT /api/v1/webhooks/:id

Update a webhook.

**Request Body:** Same as POST /api/v1/webhooks

#### DELETE /api/v1/webhooks/:id

Delete a webhook.

**Response:**
```json
{
  "success": true,
  "data": {
    "message": "Webhook deleted successfully"
  }
}
```

### WebSocket Management

#### GET /api/v1/websockets/connections

List active WebSocket connections.

**Query Parameters:**
- `page` (integer, optional): Page number (default: 1)
- `limit` (integer, optional): Items per page (default: 20, max: 100)

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": "conn_1234567890",
      "ip_address": "192.168.1.100",
      "user_agent": "Mozilla/5.0...",
      "connected_at": "2024-01-15T10:00:00Z",
      "last_ping": "2024-01-15T10:29:00Z",
      "subscriptions": [
        {
          "id": "sub_1234567890",
          "events": ["transaction"],
          "filters": {
            "addresses": ["TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"]
          }
        }
      ]
    }
  ]
}
```

#### GET /api/v1/websockets/stats

Get WebSocket statistics.

**Response:**
```json
{
  "success": true,
  "data": {
    "total_connections": 156,
    "active_connections": 142,
    "total_subscriptions": 234,
    "messages_sent_today": 12345,
    "average_latency_ms": 15.2,
    "uptime_seconds": 86400
  }
}
```

### API Key Management

#### GET /api/v1/api-keys

List API keys for your account.

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": "key_1234567890",
      "name": "Production API Key",
      "key_prefix": "tk_live_1234...",
      "permissions": [
        "read_transactions",
        "read_addresses",
        "manage_webhooks"
      ],
      "rate_limit": 1000,
      "created_at": "2024-01-01T00:00:00Z",
      "last_used": "2024-01-15T10:25:00Z",
      "usage_stats": {
        "requests_today": 456,
        "requests_this_month": 12345
      }
    }
  ]
}
```

#### POST /api/v1/api-keys

Create a new API key.

**Request Body:**
```json
{
  "name": "Production API Key",
  "permissions": [
    "read_transactions",
    "read_addresses",
    "manage_webhooks"
  ],
  "rate_limit": 1000,
  "ip_whitelist": ["192.168.1.0/24"],
  "expires_at": "2025-01-01T00:00:00Z"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "key_1234567890",
    "name": "Production API Key",
    "key": "tk_live_1234567890abcdef...",
    "permissions": [
      "read_transactions",
      "read_addresses",
      "manage_webhooks"
    ],
    "rate_limit": 1000,
    "created_at": "2024-01-15T10:30:00Z"
  }
}
```

#### DELETE /api/v1/api-keys/:id

Delete an API key.

### System Configuration

#### GET /api/v1/config

Get system configuration (admin only).

**Response:**
```json
{
  "success": true,
  "data": {
    "blockchain": {
      "start_block": 62800000,
      "batch_size": 100,
      "scan_interval": 3
    },
    "tron": {
      "nodes": [
        {
          "url": "https://api.trongrid.io",
          "priority": 1,
          "enabled": true
        }
      ]
    },
    "webhook": {
      "max_retries": 3,
      "timeout": 30
    }
  }
}
```

#### PUT /api/v1/config

Update system configuration (admin only).

### Logs

#### GET /api/v1/logs

Get system logs (admin only).

**Query Parameters:**
- `level` (string, optional): Log level (`error`, `warn`, `info`, `debug`)
- `module` (string, optional): Module name
- `start_time` (string, optional): Start time (ISO 8601)
- `end_time` (string, optional): End time (ISO 8601)
- `limit` (integer, optional): Number of log entries (default: 100, max: 1000)

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "timestamp": "2024-01-15T10:30:00Z",
      "level": "info",
      "module": "scanner",
      "message": "Processed block 62845149",
      "details": {
        "block_number": 62845149,
        "transaction_count": 45,
        "processing_time_ms": 150
      }
    }
  ]
}
```

### Dashboard

#### GET /api/v1/dashboard/stats

Get dashboard statistics.

**Response:**
```json
{
  "success": true,
  "data": {
    "total_transactions": 1234567,
    "total_addresses": 89234,
    "active_webhooks": 12,
    "websocket_connections": 156,
    "current_block": 62845149,
    "scan_speed": 18.5,
    "success_rate": 99.2,
    "api_requests_today": 45678,
    "uptime": 2851200
  }
}
```

## WebSocket Protocol

### Connection

Connect to the WebSocket endpoint:

```
ws://localhost:8080/ws
wss://api.trontracker.com/ws
```

### Authentication

Send authentication message after connection:

```json
{
  "type": "auth",
  "token": "tk_live_1234567890abcdef..."
}
```

### Message Types

#### Subscribe to Events

```json
{
  "type": "subscribe",
  "events": ["transaction", "large_transfer"],
  "filters": {
    "addresses": ["TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"],
    "tokens": ["USDT"],
    "min_amount": "1000"
  }
}
```

**Response:**
```json
{
  "type": "subscription_created",
  "subscription_id": "sub_1234567890",
  "events": ["transaction", "large_transfer"],
  "filters": {
    "addresses": ["TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"],
    "tokens": ["USDT"],
    "min_amount": "1000"
  }
}
```

#### Unsubscribe from Events

```json
{
  "type": "unsubscribe",
  "subscription_id": "sub_1234567890"
}
```

#### Transaction Notification

```json
{
  "type": "transaction",
  "subscription_id": "sub_1234567890",
  "data": {
    "hash": "0x1a2b3c4d5e6f7890abcdef1234567890abcdef1234567890abcdef1234567890",
    "block_number": 62845149,
    "from_address": "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t",
    "to_address": "TLa2f6VPqDgRE67v1736s7bJ8Ray5wYjU7",
    "amount": "1250.500000",
    "token": "USDT",
    "status": "success",
    "timestamp": "2024-01-15T10:30:00Z"
  }
}
```

#### Large Transfer Alert

```json
{
  "type": "large_transfer",
  "subscription_id": "sub_1234567890",
  "data": {
    "hash": "0x...",
    "amount": "1000000.000000",
    "token": "USDT",
    "from_address": "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t",
    "to_address": "TLa2f6VPqDgRE67v1736s7bJ8Ray5wYjU7",
    "timestamp": "2024-01-15T10:30:00Z"
  }
}
```

#### Heartbeat

```json
{
  "type": "ping"
}
```

**Response:**
```json
{
  "type": "pong"
}
```

#### Error Messages

```json
{
  "type": "error",
  "code": "INVALID_SUBSCRIPTION",
  "message": "Invalid subscription parameters",
  "details": {
    "field": "addresses",
    "error": "Maximum 100 addresses allowed"
  }
}
```

### Event Types

- `transaction`: New transaction matching filters
- `large_transfer`: Large value transfers (configurable threshold)
- `contract_interaction`: Smart contract interactions
- `token_transfer`: Token transfer events
- `address_activity`: Activity on watched addresses

### Filter Options

- `addresses`: Array of addresses to monitor
- `tokens`: Array of token symbols (`TRX`, `USDT`)
- `min_amount`: Minimum transaction amount
- `max_amount`: Maximum transaction amount
- `contract_addresses`: Array of contract addresses
- `event_types`: Array of event types to include

## Webhook Payload Format

### Transaction Event

```json
{
  "event": "transaction",
  "timestamp": "2024-01-15T10:30:00Z",
  "data": {
    "hash": "0x1a2b3c4d5e6f7890abcdef1234567890abcdef1234567890abcdef1234567890",
    "block_number": 62845149,
    "from_address": "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t",
    "to_address": "TLa2f6VPqDgRE67v1736s7bJ8Ray5wYjU7",
    "amount": "1250.500000",
    "token": "USDT",
    "status": "success",
    "timestamp": "2024-01-15T10:30:00Z",
    "gas_used": 14000,
    "gas_price": 420,
    "confirmations": 1
  }
}
```

### Webhook Security

All webhook requests include a signature header:

```
X-Webhook-Signature: sha256=a1b2c3d4e5f6...
```

Verify the signature using HMAC-SHA256:

```python
import hmac
import hashlib
import json

def verify_webhook(payload, signature, secret):
    expected_signature = hmac.new(
        secret.encode('utf-8'),
        json.dumps(payload, separators=(',', ':')).encode('utf-8'),
        hashlib.sha256
    ).hexdigest()
    
    return hmac.compare_digest(
        f"sha256={expected_signature}",
        signature
    )
```

## SDKs and Libraries

### JavaScript/TypeScript

```bash
npm install @trontracker/sdk
```

```javascript
import { TronTracker } from '@trontracker/sdk';

const client = new TronTracker({
  apiKey: 'tk_live_1234567890abcdef...',
  baseUrl: 'https://api.trontracker.com'
});

// Get transactions
const transactions = await client.transactions.list({
  status: 'success',
  token: 'USDT',
  limit: 50
});

// Multi-address query
const multiAddressResults = await client.transactions.multiAddress({
  addresses: [
    'TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t',
    'TLa2f6VPqDgRE67v1736s7bJ8Ray5wYjU7'
  ],
  limit: 100
});

// WebSocket connection
const ws = client.websocket.connect();
ws.subscribe({
  events: ['transaction'],
  filters: {
    addresses: ['TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t'],
    tokens: ['USDT']
  }
});

ws.on('transaction', (transaction) => {
  console.log('New transaction:', transaction);
});
```

### Python

```bash
pip install trontracker-python
```

```python
from trontracker import TronTracker

client = TronTracker(
    api_key='tk_live_1234567890abcdef...',
    base_url='https://api.trontracker.com'
)

# Get transactions
transactions = client.transactions.list(
    status='success',
    token='USDT',
    limit=50
)

# Multi-address query
results = client.transactions.multi_address(
    addresses=[
        'TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t',
        'TLa2f6VPqDgRE67v1736s7bJ8Ray5wYjU7'
    ],
    limit=100
)

# WebSocket connection
import asyncio

async def handle_transaction(transaction):
    print(f"New transaction: {transaction}")

async def main():
    ws = await client.websocket.connect()
    await ws.subscribe(
        events=['transaction'],
        filters={
            'addresses': ['TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t'],
            'tokens': ['USDT']
        }
    )
    
    ws.on('transaction', handle_transaction)
    await ws.listen()

asyncio.run(main())
```

### Go

```bash
go get github.com/trontracker/go-sdk
```

```go
package main

import (
    "context"
    "fmt"
    "github.com/trontracker/go-sdk"
)

func main() {
    client := trontracker.NewClient(&trontracker.Config{
        APIKey:  "tk_live_1234567890abcdef...",
        BaseURL: "https://api.trontracker.com",
    })

    // Get transactions
    transactions, err := client.Transactions.List(context.Background(), &trontracker.TransactionQuery{
        Status: "success",
        Token:  "USDT",
        Limit:  50,
    })
    if err != nil {
        panic(err)
    }

    fmt.Printf("Found %d transactions\n", len(transactions.Data))

    // Multi-address query
    results, err := client.Transactions.MultiAddress(context.Background(), &trontracker.MultiAddressQuery{
        Addresses: []string{
            "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t",
            "TLa2f6VPqDgRE67v1736s7bJ8Ray5wYjU7",
        },
        Limit: 100,
    })
    if err != nil {
        panic(err)
    }

    fmt.Printf("Found %d transactions across addresses\n", len(results.Data))
}
```

## Error Codes

| Code | Description |
|------|-------------|
| 400 | Bad Request - Invalid parameters |
| 401 | Unauthorized - Invalid or missing API key |
| 403 | Forbidden - Insufficient permissions |
| 404 | Not Found - Resource not found |
| 429 | Too Many Requests - Rate limit exceeded |
| 500 | Internal Server Error - Server error |
| 503 | Service Unavailable - Temporary service issue |

## Support

For API support and questions:

- **Documentation**: https://docs.trontracker.com
- **Support Email**: support@trontracker.com
- **Status Page**: https://status.trontracker.com
- **GitHub Issues**: https://github.com/trontracker/api-issues

