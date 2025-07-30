from flask import Blueprint, jsonify, request
from flask_cors import cross_origin
import datetime
import random

admin_bp = Blueprint('admin', __name__)

# Mock data
mock_stats = {
    'total_transactions': 58778,
    'success_rate': 96.5,
    'active_webhooks': 3,
    'websocket_connections': 4,
    'api_keys': 4,
    'total_requests': 487234,
    'current_block': 62845149,
    'scan_speed': 20,
    'error_count': 26
}

mock_webhooks = [
    {
        'id': 'webhook_1',
        'name': '交易通知',
        'url': 'https://api.example.com/webhook/transactions',
        'enabled': True,
        'success_count': 1234,
        'failure_count': 45,
        'success_rate': 96.5,
        'last_triggered': '2024-07-29T19:45:00Z'
    },
    {
        'id': 'webhook_2',
        'name': '大额转账告警',
        'url': 'https://alert.example.com/webhook/large-transfers',
        'enabled': True,
        'success_count': 567,
        'failure_count': 12,
        'success_rate': 97.9,
        'last_triggered': '2024-07-29T18:30:00Z'
    },
    {
        'id': 'webhook_3',
        'name': '系统状态监控',
        'url': 'https://monitor.example.com/webhook/status',
        'enabled': False,
        'success_count': 890,
        'failure_count': 78,
        'success_rate': 91.9,
        'last_triggered': '2024-07-29T16:20:00Z'
    }
]

mock_connections = [
    {
        'id': 'conn_1',
        'client_ip': '192.168.1.100',
        'user_agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64)',
        'connected_at': '2024-07-29T18:30:00Z',
        'status': 'connected',
        'messages_sent': 1234,
        'messages_received': 567,
        'latency': 15
    },
    {
        'id': 'conn_2',
        'client_ip': '10.0.0.50',
        'user_agent': 'TronTracker Mobile App v2.1.0',
        'connected_at': '2024-07-29T17:15:00Z',
        'status': 'connected',
        'messages_sent': 2345,
        'messages_received': 890,
        'latency': 28
    }
]

mock_api_keys = [
    {
        'id': 'key_1',
        'name': '主要 API 密钥',
        'key': 'sk_test_1234567890abcdef',
        'enabled': True,
        'permissions': ['read_transactions', 'read_addresses', 'manage_webhooks'],
        'request_count': 125430,
        'last_used': '2024-07-29T19:30:00Z'
    },
    {
        'id': 'key_2',
        'name': '移动应用密钥',
        'key': 'sk_test_abcdef1234567890',
        'enabled': True,
        'permissions': ['read_transactions', 'read_addresses'],
        'request_count': 89234,
        'last_used': '2024-07-29T18:45:00Z'
    }
]

mock_transactions = [
    {
        'hash': '0x1234567890abcdef1234567890abcdef12345678',
        'from_address': 'TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t',
        'to_address': 'TLPpXqzCanWdHqaYYUFPYRrW4YvsVJvM7d',
        'amount': '1000.50',
        'token': 'USDT',
        'status': 'success',
        'timestamp': '2024-07-29T19:45:30Z',
        'fee': '1.5'
    },
    {
        'hash': '0xabcdef1234567890abcdef1234567890abcdef12',
        'from_address': 'TLPpXqzCanWdHqaYYUFPYRrW4YvsVJvM7d',
        'to_address': 'TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t',
        'amount': '500.25',
        'token': 'TRX',
        'status': 'success',
        'timestamp': '2024-07-29T19:44:15Z',
        'fee': '0.8'
    }
]

mock_logs = [
    {
        'id': 'log_1',
        'timestamp': '2024-07-29T19:45:30Z',
        'level': 'ERROR',
        'module': 'Webhook',
        'message': '新的日志消息 1753818279612'
    },
    {
        'id': 'log_2',
        'timestamp': '2024-07-29T19:44:36Z',
        'level': 'ERROR',
        'module': 'Scanner',
        'message': '新的日志消息 1753818276612'
    },
    {
        'id': 'log_3',
        'timestamp': '2024-07-29T19:44:33Z',
        'level': 'WARN',
        'module': 'Webhook',
        'message': '新的日志消息 1753818273612'
    }
]

# Dashboard APIs
@admin_bp.route('/dashboard/stats', methods=['GET'])
@cross_origin()
def dashboard_stats():
    return jsonify(mock_stats)

@admin_bp.route('/dashboard/chart-data', methods=['GET'])
@cross_origin()
def chart_data():
    # Generate some mock chart data
    chart_data = []
    for i in range(7):
        date = datetime.datetime.now() - datetime.timedelta(days=6-i)
        chart_data.append({
            'date': date.strftime('%Y-%m-%d'),
            'transactions': random.randint(5000, 8000),
            'api_calls': random.randint(15000, 25000)
        })
    return jsonify({'chart_data': chart_data})

# Webhook APIs
@admin_bp.route('/webhooks', methods=['GET'])
@cross_origin()
def list_webhooks():
    return jsonify({'webhooks': mock_webhooks, 'total': len(mock_webhooks)})

@admin_bp.route('/webhooks', methods=['POST'])
@cross_origin()
def create_webhook():
    data = request.get_json()
    new_webhook = {
        'id': f'webhook_{len(mock_webhooks) + 1}',
        'name': data.get('name', '新 Webhook'),
        'url': data.get('url', ''),
        'enabled': data.get('enabled', True),
        'success_count': 0,
        'failure_count': 0,
        'success_rate': 100.0,
        'last_triggered': None
    }
    mock_webhooks.append(new_webhook)
    return jsonify(new_webhook), 201

@admin_bp.route('/webhooks/<webhook_id>', methods=['PUT'])
@cross_origin()
def update_webhook(webhook_id):
    webhook = next((w for w in mock_webhooks if w['id'] == webhook_id), None)
    if not webhook:
        return jsonify({'error': 'Webhook not found'}), 404
    
    data = request.get_json()
    webhook.update(data)
    return jsonify(webhook)

@admin_bp.route('/webhooks/<webhook_id>', methods=['DELETE'])
@cross_origin()
def delete_webhook(webhook_id):
    global mock_webhooks
    mock_webhooks = [w for w in mock_webhooks if w['id'] != webhook_id]
    return '', 204

@admin_bp.route('/webhooks/stats', methods=['GET'])
@cross_origin()
def webhook_stats():
    total_success = sum(w['success_count'] for w in mock_webhooks)
    total_failure = sum(w['failure_count'] for w in mock_webhooks)
    overall_success_rate = (total_success / (total_success + total_failure)) * 100 if (total_success + total_failure) > 0 else 0
    
    return jsonify({
        'total_webhooks': len(mock_webhooks),
        'active_webhooks': len([w for w in mock_webhooks if w['enabled']]),
        'total_calls': total_success + total_failure,
        'success_rate': round(overall_success_rate, 2)
    })

# WebSocket APIs
@admin_bp.route('/websockets/connections', methods=['GET'])
@cross_origin()
def list_connections():
    return jsonify({'connections': mock_connections, 'total': len(mock_connections)})

@admin_bp.route('/websockets/stats', methods=['GET'])
@cross_origin()
def websocket_stats():
    active_connections = len([c for c in mock_connections if c['status'] == 'connected'])
    total_messages = sum(c['messages_sent'] + c['messages_received'] for c in mock_connections)
    avg_latency = sum(c['latency'] for c in mock_connections) / len(mock_connections) if mock_connections else 0
    
    return jsonify({
        'total_connections': len(mock_connections),
        'active_connections': active_connections,
        'total_messages': total_messages,
        'avg_latency': round(avg_latency, 2)
    })

@admin_bp.route('/websockets/connections/<connection_id>/disconnect', methods=['POST'])
@cross_origin()
def disconnect_connection(connection_id):
    connection = next((c for c in mock_connections if c['id'] == connection_id), None)
    if connection:
        connection['status'] = 'disconnected'
        return jsonify({'message': 'Connection disconnected'})
    return jsonify({'error': 'Connection not found'}), 404

# API Key APIs
@admin_bp.route('/api-keys', methods=['GET'])
@cross_origin()
def list_api_keys():
    return jsonify({'api_keys': mock_api_keys, 'total': len(mock_api_keys)})

@admin_bp.route('/api-keys', methods=['POST'])
@cross_origin()
def create_api_key():
    data = request.get_json()
    new_key = {
        'id': f'key_{len(mock_api_keys) + 1}',
        'name': data.get('name', '新 API Key'),
        'key': f'sk_test_{random.randint(1000000000000000, 9999999999999999):016x}',
        'enabled': data.get('enabled', True),
        'permissions': data.get('permissions', []),
        'request_count': 0,
        'last_used': None
    }
    mock_api_keys.append(new_key)
    return jsonify(new_key), 201

@admin_bp.route('/api-keys/<key_id>', methods=['PUT'])
@cross_origin()
def update_api_key(key_id):
    api_key = next((k for k in mock_api_keys if k['id'] == key_id), None)
    if not api_key:
        return jsonify({'error': 'API Key not found'}), 404
    
    data = request.get_json()
    api_key.update(data)
    return jsonify(api_key)

@admin_bp.route('/api-keys/<key_id>', methods=['DELETE'])
@cross_origin()
def delete_api_key(key_id):
    global mock_api_keys
    mock_api_keys = [k for k in mock_api_keys if k['id'] != key_id]
    return '', 204

@admin_bp.route('/api-keys/stats', methods=['GET'])
@cross_origin()
def api_key_stats():
    total_requests = sum(k['request_count'] for k in mock_api_keys)
    active_keys = len([k for k in mock_api_keys if k['enabled']])
    
    return jsonify({
        'total_keys': len(mock_api_keys),
        'active_keys': active_keys,
        'total_requests': total_requests,
        'avg_requests_per_key': round(total_requests / len(mock_api_keys), 2) if mock_api_keys else 0
    })

# Transaction APIs
@admin_bp.route('/transactions', methods=['GET'])
@cross_origin()
def list_transactions():
    return jsonify({'transactions': mock_transactions, 'total': len(mock_transactions)})

@admin_bp.route('/transactions/search', methods=['GET'])
@cross_origin()
def search_transactions():
    query = request.args.get('q', '')
    status = request.args.get('status', '')
    
    filtered_transactions = mock_transactions
    if query:
        filtered_transactions = [t for t in filtered_transactions if query.lower() in t['hash'].lower() or query.lower() in t['from_address'].lower() or query.lower() in t['to_address'].lower()]
    if status:
        filtered_transactions = [t for t in filtered_transactions if t['status'] == status]
    
    return jsonify({'transactions': filtered_transactions, 'total': len(filtered_transactions)})

@admin_bp.route('/transactions/stats', methods=['GET'])
@cross_origin()
def transaction_stats():
    success_count = len([t for t in mock_transactions if t['status'] == 'success'])
    success_rate = (success_count / len(mock_transactions)) * 100 if mock_transactions else 0
    total_fees = sum(float(t['fee']) for t in mock_transactions)
    
    return jsonify({
        'total_transactions': len(mock_transactions),
        'success_count': success_count,
        'success_rate': round(success_rate, 2),
        'total_fees': round(total_fees, 2),
        'avg_fee': round(total_fees / len(mock_transactions), 2) if mock_transactions else 0
    })

# System Configuration APIs
@admin_bp.route('/config/blockchain', methods=['GET'])
@cross_origin()
def get_blockchain_config():
    return jsonify({
        'sync_mode': 'full',
        'start_block': 62800000,
        'batch_size': 100,
        'start_time': '2024-01-01T00:00:00Z',
        'end_time': None
    })

@admin_bp.route('/config/blockchain', methods=['PUT'])
@cross_origin()
def update_blockchain_config():
    data = request.get_json()
    return jsonify({'message': 'Blockchain configuration updated', 'config': data})

@admin_bp.route('/config/nodes', methods=['GET'])
@cross_origin()
def get_nodes():
    nodes = [
        {
            'id': 'node_1',
            'name': '主节点',
            'url': 'https://api.trongrid.io',
            'priority': 1,
            'status': 'active',
            'latency': 45,
            'last_check': '2024-07-29T19:45:00Z'
        },
        {
            'id': 'node_2',
            'name': '备用节点 1',
            'url': 'https://api.getblock.io',
            'priority': 2,
            'status': 'standby',
            'latency': 67,
            'last_check': '2024-07-29T19:44:00Z'
        }
    ]
    return jsonify({'nodes': nodes})

@admin_bp.route('/config/nodes', methods=['POST'])
@cross_origin()
def create_node():
    data = request.get_json()
    new_node = {
        'id': f'node_{random.randint(100, 999)}',
        'name': data.get('name', '新节点'),
        'url': data.get('url', ''),
        'priority': data.get('priority', 3),
        'status': 'inactive',
        'latency': None,
        'last_check': None
    }
    return jsonify(new_node), 201

@admin_bp.route('/config/database', methods=['GET'])
@cross_origin()
def get_database_config():
    return jsonify({
        'postgresql': {
            'host': 'localhost',
            'port': 5432,
            'database': 'tron_tracker',
            'username': 'postgres',
            'max_connections': 20
        },
        'redis': {
            'host': 'localhost',
            'port': 6379,
            'database': 0,
            'max_connections': 10
        }
    })

@admin_bp.route('/config/database/test', methods=['POST'])
@cross_origin()
def test_database():
    return jsonify({'status': 'connected', 'latency': 12})

# Logs APIs
@admin_bp.route('/logs', methods=['GET'])
@cross_origin()
def list_logs():
    level = request.args.get('level', '')
    module = request.args.get('module', '')
    
    filtered_logs = mock_logs
    if level:
        filtered_logs = [l for l in filtered_logs if l['level'] == level]
    if module:
        filtered_logs = [l for l in filtered_logs if l['module'] == module]
    
    return jsonify({'logs': filtered_logs, 'total': len(filtered_logs)})

@admin_bp.route('/logs/stats', methods=['GET'])
@cross_origin()
def log_stats():
    error_count = len([l for l in mock_logs if l['level'] == 'ERROR'])
    warn_count = len([l for l in mock_logs if l['level'] == 'WARN'])
    info_count = len([l for l in mock_logs if l['level'] == 'INFO'])
    
    return jsonify({
        'total_logs': len(mock_logs),
        'error_count': error_count,
        'warn_count': warn_count,
        'info_count': info_count,
        'current_block': mock_stats['current_block'],
        'scan_speed': mock_stats['scan_speed']
    })

@admin_bp.route('/logs/export', methods=['GET'])
@cross_origin()
def export_logs():
    return jsonify({'download_url': '/api/logs/download/logs_export.csv'})

@admin_bp.route('/logs/clear', methods=['POST'])
@cross_origin()
def clear_logs():
    global mock_logs
    mock_logs = []
    return jsonify({'message': 'Logs cleared successfully'})

# Health check
@admin_bp.route('/health', methods=['GET'])
@cross_origin()
def health_check():
    return jsonify({
        'status': 'healthy',
        'timestamp': datetime.datetime.utcnow().isoformat(),
        'version': '2.0.0'
    })

