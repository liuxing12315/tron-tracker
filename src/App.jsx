import { useState, useEffect } from 'react'
import { BrowserRouter as Router, Routes, Route, Link, useLocation } from 'react-router-dom'
import { Button } from '@/components/ui/button.jsx'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card.jsx'
import { Badge } from '@/components/ui/badge.jsx'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs.jsx'
import { Input } from '@/components/ui/input.jsx'
import { Label } from '@/components/ui/label.jsx'
import { Textarea } from '@/components/ui/textarea.jsx'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select.jsx'
import { Switch } from '@/components/ui/switch.jsx'
import { Progress } from '@/components/ui/progress.jsx'
import { 
  Activity, 
  Database, 
  Settings, 
  Webhook, 
  Wifi, 
  Key, 
  Search, 
  FileText,
  BarChart3,
  Users,
  Server,
  Globe,
  Shield,
  Zap,
  TrendingUp,
  AlertTriangle,
  CheckCircle,
  XCircle,
  Clock,
  Eye,
  Copy,
  Trash2,
  Edit,
  Plus,
  RefreshCw,
  Download,
  Filter
} from 'lucide-react'
import './App.css'

// Mock data
const mockStats = {
  totalTransactions: 58778,
  successRate: 96.5,
  activeWebhooks: 3,
  websocketConnections: 4,
  apiKeys: 4,
  totalRequests: 487234,
  currentBlock: 62845149,
  scanSpeed: 20,
  errorCount: 26
}

const mockWebhooks = [
  {
    id: 'webhook_1',
    name: '交易通知',
    url: 'https://api.example.com/webhook/transactions',
    enabled: true,
    successCount: 1234,
    failureCount: 45,
    successRate: 96.5,
    lastTriggered: '2024-07-29T19:45:00Z'
  },
  {
    id: 'webhook_2',
    name: '大额转账告警',
    url: 'https://alert.example.com/webhook/large-transfers',
    enabled: true,
    successCount: 567,
    failureCount: 12,
    successRate: 97.9,
    lastTriggered: '2024-07-29T18:30:00Z'
  },
  {
    id: 'webhook_3',
    name: '系统状态监控',
    url: 'https://monitor.example.com/webhook/status',
    enabled: false,
    successCount: 890,
    failureCount: 78,
    successRate: 91.9,
    lastTriggered: '2024-07-29T16:20:00Z'
  }
]

const mockConnections = [
  {
    id: 'conn_1',
    clientIp: '192.168.1.100',
    userAgent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64)',
    connectedAt: '2024-07-29T18:30:00Z',
    status: 'connected',
    messagesSent: 1234,
    messagesReceived: 567,
    latency: 15
  },
  {
    id: 'conn_2',
    clientIp: '10.0.0.50',
    userAgent: 'TronTracker Mobile App v2.1.0',
    connectedAt: '2024-07-29T17:15:00Z',
    status: 'connected',
    messagesSent: 2345,
    messagesReceived: 890,
    latency: 28
  }
]

const mockApiKeys = [
  {
    id: 'key_1',
    name: '主要 API 密钥',
    key: 'sk_test_1234567890abcdef',
    enabled: true,
    permissions: ['read_transactions', 'read_addresses', 'manage_webhooks'],
    requestCount: 125430,
    lastUsed: '2024-07-29T19:30:00Z'
  },
  {
    id: 'key_2',
    name: '移动应用密钥',
    key: 'sk_test_abcdef1234567890',
    enabled: true,
    permissions: ['read_transactions', 'read_addresses'],
    requestCount: 89234,
    lastUsed: '2024-07-29T18:45:00Z'
  }
]

const mockTransactions = [
  {
    hash: '0x1234567890abcdef1234567890abcdef12345678',
    fromAddress: 'TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t',
    toAddress: 'TLPpXqzCanWdHqaYYUFPYRrW4YvsVJvM7d',
    amount: '1000.50',
    token: 'USDT',
    status: 'success',
    timestamp: '2024-07-29T19:45:30Z',
    fee: '1.5'
  },
  {
    hash: '0xabcdef1234567890abcdef1234567890abcdef12',
    fromAddress: 'TLPpXqzCanWdHqaYYUFPYRrW4YvsVJvM7d',
    toAddress: 'TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t',
    amount: '500.25',
    token: 'TRX',
    status: 'success',
    timestamp: '2024-07-29T19:44:15Z',
    fee: '0.8'
  }
]

const mockLogs = [
  {
    id: 'log_1',
    timestamp: '2024-07-29T19:45:30Z',
    level: 'ERROR',
    module: 'Webhook',
    message: '新的日志消息 1753818279612'
  },
  {
    id: 'log_2',
    timestamp: '2024-07-29T19:44:36Z',
    level: 'ERROR',
    module: 'Scanner',
    message: '新的日志消息 1753818276612'
  },
  {
    id: 'log_3',
    timestamp: '2024-07-29T19:44:33Z',
    level: 'WARN',
    module: 'Webhook',
    message: '新的日志消息 1753818273612'
  }
]

// Navigation component
function Navigation() {
  const location = useLocation()
  
  const navItems = [
    { path: '/', label: '仪表板', icon: BarChart3 },
    { path: '/webhooks', label: 'Webhook 管理', icon: Webhook },
    { path: '/websockets', label: 'WebSocket 管理', icon: Wifi },
    { path: '/api-keys', label: 'API 签名管理', icon: Key },
    { path: '/transactions', label: '交易查询', icon: Search },
    { path: '/settings', label: '系统配置', icon: Settings },
    { path: '/logs', label: '扫描日志', icon: FileText }
  ]

  return (
    <nav className="w-64 bg-slate-900 text-white h-screen p-4">
      <div className="flex items-center gap-2 mb-8">
        <Activity className="h-8 w-8 text-blue-400" />
        <h1 className="text-xl font-bold">TRX 管理后台</h1>
      </div>
      
      <div className="space-y-2">
        {navItems.map((item) => {
          const Icon = item.icon
          const isActive = location.pathname === item.path
          
          return (
            <Link
              key={item.path}
              to={item.path}
              className={`flex items-center gap-3 px-3 py-2 rounded-lg transition-colors ${
                isActive 
                  ? 'bg-blue-600 text-white' 
                  : 'text-slate-300 hover:bg-slate-800 hover:text-white'
              }`}
            >
              <Icon className="h-5 w-5" />
              <span>{item.label}</span>
            </Link>
          )
        })}
      </div>
    </nav>
  )
}

// Dashboard component
function Dashboard() {
  return (
    <div className="p-6">
      <h1 className="text-3xl font-bold mb-6">系统概览</h1>
      
      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">总交易数</CardTitle>
            <TrendingUp className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{mockStats.totalTransactions.toLocaleString()}</div>
            <p className="text-xs text-muted-foreground">成功率 {mockStats.successRate}%</p>
          </CardContent>
        </Card>
        
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">WebSocket 连接</CardTitle>
            <Wifi className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{mockStats.websocketConnections}</div>
            <p className="text-xs text-muted-foreground">活跃连接</p>
          </CardContent>
        </Card>
        
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">API 调用</CardTitle>
            <Zap className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{mockStats.totalRequests.toLocaleString()}</div>
            <p className="text-xs text-muted-foreground">总请求数</p>
          </CardContent>
        </Card>
        
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">当前区块</CardTitle>
            <Database className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{mockStats.currentBlock.toLocaleString()}</div>
            <p className="text-xs text-muted-foreground">扫描速度 {mockStats.scanSpeed} 块/分钟</p>
          </CardContent>
        </Card>
      </div>

      {/* Recent Activity */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <Card>
          <CardHeader>
            <CardTitle>活跃 Webhooks</CardTitle>
            <CardDescription>最近触发的 Webhook 通知</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              {mockWebhooks.slice(0, 3).map((webhook) => (
                <div key={webhook.id} className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <div className={`w-2 h-2 rounded-full ${webhook.enabled ? 'bg-green-500' : 'bg-gray-400'}`} />
                    <div>
                      <p className="font-medium">{webhook.name}</p>
                      <p className="text-sm text-muted-foreground">成功率 {webhook.successRate}%</p>
                    </div>
                  </div>
                  <Badge variant={webhook.enabled ? 'default' : 'secondary'}>
                    {webhook.enabled ? '启用' : '禁用'}
                  </Badge>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>系统状态</CardTitle>
            <CardDescription>实时系统监控指标</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div>
                <div className="flex justify-between mb-2">
                  <span className="text-sm">CPU 使用率</span>
                  <span className="text-sm">15.2%</span>
                </div>
                <Progress value={15.2} className="h-2" />
              </div>
              <div>
                <div className="flex justify-between mb-2">
                  <span className="text-sm">内存使用率</span>
                  <span className="text-sm">68.5%</span>
                </div>
                <Progress value={68.5} className="h-2" />
              </div>
              <div>
                <div className="flex justify-between mb-2">
                  <span className="text-sm">磁盘使用率</span>
                  <span className="text-sm">42.1%</span>
                </div>
                <Progress value={42.1} className="h-2" />
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}

// Webhook Management component
function WebhookManagement() {
  const [webhooks, setWebhooks] = useState(mockWebhooks)
  
  return (
    <div className="p-6">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-3xl font-bold">Webhook 管理</h1>
        <Button>
          <Plus className="h-4 w-4 mr-2" />
          新建 Webhook
        </Button>
      </div>
      
      <div className="grid gap-6">
        {webhooks.map((webhook) => (
          <Card key={webhook.id}>
            <CardHeader>
              <div className="flex justify-between items-start">
                <div>
                  <CardTitle className="flex items-center gap-2">
                    {webhook.name}
                    <Badge variant={webhook.enabled ? 'default' : 'secondary'}>
                      {webhook.enabled ? '启用' : '禁用'}
                    </Badge>
                  </CardTitle>
                  <CardDescription>{webhook.url}</CardDescription>
                </div>
                <div className="flex gap-2">
                  <Button variant="outline" size="sm">
                    <Edit className="h-4 w-4" />
                  </Button>
                  <Button variant="outline" size="sm">
                    <Trash2 className="h-4 w-4" />
                  </Button>
                </div>
              </div>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-3 gap-4">
                <div>
                  <p className="text-sm text-muted-foreground">成功次数</p>
                  <p className="text-2xl font-bold text-green-600">{webhook.successCount}</p>
                </div>
                <div>
                  <p className="text-sm text-muted-foreground">失败次数</p>
                  <p className="text-2xl font-bold text-red-600">{webhook.failureCount}</p>
                </div>
                <div>
                  <p className="text-sm text-muted-foreground">成功率</p>
                  <p className="text-2xl font-bold">{webhook.successRate}%</p>
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  )
}

// WebSocket Management component
function WebSocketManagement() {
  return (
    <div className="p-6">
      <h1 className="text-3xl font-bold mb-6">WebSocket 管理</h1>
      
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
        <Card>
          <CardHeader>
            <CardTitle>总连接数</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold">{mockConnections.length}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>活跃连接</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-green-600">
              {mockConnections.filter(c => c.status === 'connected').length}
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>平均延迟</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold">
              {Math.round(mockConnections.reduce((sum, c) => sum + c.latency, 0) / mockConnections.length)}ms
            </div>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>连接列表</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {mockConnections.map((conn) => (
              <div key={conn.id} className="flex items-center justify-between p-4 border rounded-lg">
                <div className="flex items-center gap-4">
                  <div className={`w-3 h-3 rounded-full ${conn.status === 'connected' ? 'bg-green-500' : 'bg-gray-400'}`} />
                  <div>
                    <p className="font-medium">{conn.clientIp}</p>
                    <p className="text-sm text-muted-foreground">{conn.userAgent}</p>
                  </div>
                </div>
                <div className="text-right">
                  <p className="text-sm">延迟: {conn.latency}ms</p>
                  <p className="text-sm text-muted-foreground">
                    消息: {conn.messagesSent + conn.messagesReceived}
                  </p>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

// API Key Management component
function ApiKeyManagement() {
  return (
    <div className="p-6">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-3xl font-bold">API 签名管理</h1>
        <Button>
          <Plus className="h-4 w-4 mr-2" />
          新建 API Key
        </Button>
      </div>
      
      <div className="grid gap-6">
        {mockApiKeys.map((apiKey) => (
          <Card key={apiKey.id}>
            <CardHeader>
              <div className="flex justify-between items-start">
                <div>
                  <CardTitle className="flex items-center gap-2">
                    {apiKey.name}
                    <Badge variant={apiKey.enabled ? 'default' : 'secondary'}>
                      {apiKey.enabled ? '启用' : '禁用'}
                    </Badge>
                  </CardTitle>
                  <CardDescription className="flex items-center gap-2">
                    <code className="bg-muted px-2 py-1 rounded text-sm">
                      {apiKey.key.substring(0, 20)}...
                    </code>
                    <Button variant="ghost" size="sm">
                      <Copy className="h-4 w-4" />
                    </Button>
                  </CardDescription>
                </div>
                <div className="flex gap-2">
                  <Button variant="outline" size="sm">
                    <RefreshCw className="h-4 w-4" />
                  </Button>
                  <Button variant="outline" size="sm">
                    <Edit className="h-4 w-4" />
                  </Button>
                </div>
              </div>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-2 gap-4 mb-4">
                <div>
                  <p className="text-sm text-muted-foreground">请求次数</p>
                  <p className="text-2xl font-bold">{apiKey.requestCount.toLocaleString()}</p>
                </div>
                <div>
                  <p className="text-sm text-muted-foreground">权限数量</p>
                  <p className="text-2xl font-bold">{apiKey.permissions.length}</p>
                </div>
              </div>
              <div className="flex flex-wrap gap-2">
                {apiKey.permissions.map((permission) => (
                  <Badge key={permission} variant="outline">
                    {permission}
                  </Badge>
                ))}
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  )
}

// Transaction Query component
function TransactionQuery() {
  return (
    <div className="p-6">
      <h1 className="text-3xl font-bold mb-6">交易查询</h1>
      
      <Card className="mb-6">
        <CardHeader>
          <CardTitle>搜索交易</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex gap-4">
            <Input placeholder="输入交易哈希、地址或区块号..." className="flex-1" />
            <Button>
              <Search className="h-4 w-4 mr-2" />
              搜索
            </Button>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>交易记录</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {mockTransactions.map((tx, index) => (
              <div key={index} className="flex items-center justify-between p-4 border rounded-lg">
                <div className="flex items-center gap-4">
                  <div className={`w-3 h-3 rounded-full ${tx.status === 'success' ? 'bg-green-500' : 'bg-red-500'}`} />
                  <div>
                    <p className="font-medium">
                      {tx.hash.substring(0, 20)}...
                    </p>
                    <p className="text-sm text-muted-foreground">
                      {tx.fromAddress.substring(0, 15)}... → {tx.toAddress.substring(0, 15)}...
                    </p>
                  </div>
                </div>
                <div className="text-right">
                  <p className="font-medium">{tx.amount} {tx.token}</p>
                  <p className="text-sm text-muted-foreground">手续费: {tx.fee} TRX</p>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

// System Settings component
function SystemSettings() {
  return (
    <div className="p-6">
      <h1 className="text-3xl font-bold mb-6">系统配置</h1>
      
      <Tabs defaultValue="blockchain" className="space-y-6">
        <TabsList>
          <TabsTrigger value="blockchain">区块链配置</TabsTrigger>
          <TabsTrigger value="nodes">节点管理</TabsTrigger>
          <TabsTrigger value="database">数据库配置</TabsTrigger>
        </TabsList>
        
        <TabsContent value="blockchain" className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>区块链同步配置</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div>
                <Label>同步模式</Label>
                <Select defaultValue="full">
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="full">完整同步</SelectItem>
                    <SelectItem value="fast">快速同步</SelectItem>
                    <SelectItem value="light">轻量同步</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div>
                <Label>起始区块</Label>
                <Input defaultValue="62800000" />
              </div>
              <div>
                <Label>批处理大小</Label>
                <Input defaultValue="100" />
              </div>
            </CardContent>
          </Card>
        </TabsContent>
        
        <TabsContent value="nodes" className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>Tron 节点配置</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                <div className="flex items-center justify-between p-4 border rounded-lg">
                  <div>
                    <p className="font-medium">主节点</p>
                    <p className="text-sm text-muted-foreground">https://api.trongrid.io</p>
                  </div>
                  <Badge variant="default">活跃</Badge>
                </div>
                <div className="flex items-center justify-between p-4 border rounded-lg">
                  <div>
                    <p className="font-medium">备用节点 1</p>
                    <p className="text-sm text-muted-foreground">https://api.getblock.io</p>
                  </div>
                  <Badge variant="secondary">备用</Badge>
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
        
        <TabsContent value="database" className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>数据库连接配置</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div>
                <Label>PostgreSQL 主机</Label>
                <Input defaultValue="localhost" />
              </div>
              <div>
                <Label>端口</Label>
                <Input defaultValue="5432" />
              </div>
              <div>
                <Label>数据库名</Label>
                <Input defaultValue="tron_tracker" />
              </div>
              <Button>测试连接</Button>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  )
}

// Scanner Logs component
function ScannerLogs() {
  return (
    <div className="p-6">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-3xl font-bold">扫描日志</h1>
        <div className="flex gap-2">
          <Button variant="outline">
            <Filter className="h-4 w-4 mr-2" />
            过滤
          </Button>
          <Button variant="outline">
            <Download className="h-4 w-4 mr-2" />
            导出
          </Button>
        </div>
      </div>
      
      <div className="grid grid-cols-1 md:grid-cols-4 gap-6 mb-8">
        <Card>
          <CardHeader>
            <CardTitle>当前区块</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{mockStats.currentBlock.toLocaleString()}</div>
            <p className="text-sm text-muted-foreground">扫描速度: {mockStats.scanSpeed} 块/分钟</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>处理交易数</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{mockStats.totalTransactions.toLocaleString()}</div>
            <p className="text-sm text-muted-foreground">平均处理时间: 152ms</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>日志总数</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">109</div>
            <p className="text-sm text-muted-foreground">信息: 31 | 警告: 21</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>错误数量</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-red-600">{mockStats.errorCount}</div>
            <p className="text-sm text-muted-foreground">需要关注</p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>实时日志</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-2">
            {mockLogs.map((log) => (
              <div key={log.id} className="flex items-center gap-4 p-3 border rounded-lg">
                <div className="text-sm text-muted-foreground">
                  {new Date(log.timestamp).toLocaleTimeString()}
                </div>
                <Badge variant={log.level === 'ERROR' ? 'destructive' : log.level === 'WARN' ? 'secondary' : 'default'}>
                  {log.level}
                </Badge>
                <Badge variant="outline">{log.module}</Badge>
                <div className="flex-1">{log.message}</div>
                <Button variant="ghost" size="sm">
                  <Eye className="h-4 w-4" />
                </Button>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

// Main App component
function App() {
  return (
    <Router>
      <div className="flex min-h-screen bg-gray-50">
        <Navigation />
        <main className="flex-1 overflow-auto">
          <Routes>
            <Route path="/" element={<Dashboard />} />
            <Route path="/webhooks" element={<WebhookManagement />} />
            <Route path="/websockets" element={<WebSocketManagement />} />
            <Route path="/api-keys" element={<ApiKeyManagement />} />
            <Route path="/transactions" element={<TransactionQuery />} />
            <Route path="/settings" element={<SystemSettings />} />
            <Route path="/logs" element={<ScannerLogs />} />
          </Routes>
        </main>
      </div>
    </Router>
  )
}

export default App

