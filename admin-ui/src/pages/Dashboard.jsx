import { useState, useEffect } from 'react'
import { 
  ArrowRightLeft, 
  MapPin, 
  Webhook, 
  Wifi, 
  TrendingUp, 
  TrendingDown,
  Activity,
  Clock,
  AlertTriangle,
  CheckCircle
} from 'lucide-react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Progress } from '@/components/ui/progress'
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, AreaChart, Area } from 'recharts'

const mockChartData = [
  { time: '00:00', transactions: 1200, blocks: 20 },
  { time: '04:00', transactions: 800, blocks: 18 },
  { time: '08:00', transactions: 2100, blocks: 22 },
  { time: '12:00', transactions: 2800, blocks: 25 },
  { time: '16:00', transactions: 3200, blocks: 28 },
  { time: '20:00', transactions: 2400, blocks: 24 },
  { time: '24:00', transactions: 1800, blocks: 21 },
]

const recentTransactions = [
  {
    hash: '0x1a2b3c4d5e6f7890abcdef1234567890abcdef12',
    from: 'TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t',
    to: 'TLa2f6VPqDgRE67v1736s7bJ8Ray5wYjU7',
    amount: '1,250.50',
    token: 'USDT',
    status: 'success',
    time: '2 min ago'
  },
  {
    hash: '0x2b3c4d5e6f7890abcdef1234567890abcdef123a',
    from: 'TLa2f6VPqDgRE67v1736s7bJ8Ray5wYjU7',
    to: 'TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t',
    amount: '500.00',
    token: 'USDT',
    status: 'success',
    time: '5 min ago'
  },
  {
    hash: '0x3c4d5e6f7890abcdef1234567890abcdef123a2b',
    from: 'TKzxdSv2FZKQrEqkKVgp5DcwEXBEKMg2Ax',
    to: 'TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t',
    amount: '10,000.00',
    token: 'USDT',
    status: 'pending',
    time: '8 min ago'
  }
]

export function Dashboard({ systemStats }) {
  const [timeRange, setTimeRange] = useState('24h')

  const formatNumber = (num) => {
    if (num >= 1000000) {
      return (num / 1000000).toFixed(1) + 'M'
    } else if (num >= 1000) {
      return (num / 1000).toFixed(1) + 'K'
    }
    return num.toLocaleString()
  }

  const StatCard = ({ title, value, change, icon: Icon, trend, description }) => (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle className="text-sm font-medium text-gray-600">{title}</CardTitle>
        <Icon className="h-4 w-4 text-gray-400" />
      </CardHeader>
      <CardContent>
        <div className="text-2xl font-bold text-gray-900">{value}</div>
        {change && (
          <div className="flex items-center space-x-1 mt-1">
            {trend === 'up' ? (
              <TrendingUp className="h-3 w-3 text-green-500" />
            ) : (
              <TrendingDown className="h-3 w-3 text-red-500" />
            )}
            <span className={`text-xs ${trend === 'up' ? 'text-green-600' : 'text-red-600'}`}>
              {change}
            </span>
            <span className="text-xs text-gray-500">vs last 24h</span>
          </div>
        )}
        {description && (
          <p className="text-xs text-gray-500 mt-1">{description}</p>
        )}
      </CardContent>
    </Card>
  )

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Dashboard</h1>
          <p className="text-gray-600">Monitor your TRX/USDT tracking system</p>
        </div>
        
        <div className="flex items-center space-x-2">
          <Badge variant="outline" className="text-green-600 border-green-200">
            <div className="w-2 h-2 bg-green-500 rounded-full mr-2"></div>
            System Healthy
          </Badge>
        </div>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <StatCard
          title="Total Transactions"
          value={formatNumber(systemStats.totalTransactions)}
          change="+12.5%"
          trend="up"
          icon={ArrowRightLeft}
          description="All time transaction count"
        />
        
        <StatCard
          title="Tracked Addresses"
          value={formatNumber(systemStats.totalAddresses)}
          change="+8.2%"
          trend="up"
          icon={MapPin}
          description="Unique addresses monitored"
        />
        
        <StatCard
          title="Active Webhooks"
          value={systemStats.activeWebhooks}
          change="+2"
          trend="up"
          icon={Webhook}
          description="Currently enabled webhooks"
        />
        
        <StatCard
          title="WS Connections"
          value={systemStats.websocketConnections}
          change="-5.1%"
          trend="down"
          icon={Wifi}
          description="Real-time connections"
        />
      </div>

      {/* Charts Row */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Transaction Volume Chart */}
        <Card>
          <CardHeader>
            <CardTitle>Transaction Volume</CardTitle>
            <CardDescription>Transactions processed over time</CardDescription>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={300}>
              <AreaChart data={mockChartData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="time" />
                <YAxis />
                <Tooltip />
                <Area 
                  type="monotone" 
                  dataKey="transactions" 
                  stroke="#3b82f6" 
                  fill="#3b82f6" 
                  fillOpacity={0.1}
                />
              </AreaChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>

        {/* Block Processing Chart */}
        <Card>
          <CardHeader>
            <CardTitle>Block Processing</CardTitle>
            <CardDescription>Blocks processed per hour</CardDescription>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={300}>
              <LineChart data={mockChartData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="time" />
                <YAxis />
                <Tooltip />
                <Line 
                  type="monotone" 
                  dataKey="blocks" 
                  stroke="#10b981" 
                  strokeWidth={2}
                  dot={{ fill: '#10b981' }}
                />
              </LineChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>
      </div>

      {/* System Status and Recent Activity */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* System Status */}
        <Card>
          <CardHeader>
            <CardTitle>System Status</CardTitle>
            <CardDescription>Current system health metrics</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <div className="flex justify-between text-sm">
                <span>Scanner Status</span>
                <Badge variant="outline" className="text-green-600">
                  <CheckCircle className="w-3 h-3 mr-1" />
                  Running
                </Badge>
              </div>
              <div className="flex justify-between text-sm">
                <span>Current Block</span>
                <span className="font-mono">#{formatNumber(systemStats.currentBlock)}</span>
              </div>
              <div className="flex justify-between text-sm">
                <span>Scan Speed</span>
                <span>{systemStats.scanSpeed.toFixed(1)} blocks/min</span>
              </div>
            </div>
            
            <div className="space-y-2">
              <div className="flex justify-between text-sm">
                <span>Success Rate</span>
                <span>{systemStats.successRate.toFixed(1)}%</span>
              </div>
              <Progress value={systemStats.successRate} className="h-2" />
            </div>
            
            <div className="space-y-2">
              <div className="flex justify-between text-sm">
                <span>API Requests Today</span>
                <span>{formatNumber(systemStats.apiRequestsToday)}</span>
              </div>
              <div className="flex justify-between text-sm">
                <span>System Uptime</span>
                <span>{Math.floor(systemStats.uptime / 86400)}d {Math.floor((systemStats.uptime % 86400) / 3600)}h</span>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Recent Transactions */}
        <Card className="lg:col-span-2">
          <CardHeader>
            <CardTitle>Recent Transactions</CardTitle>
            <CardDescription>Latest transactions processed by the system</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              {recentTransactions.map((tx, index) => (
                <div key={index} className="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                  <div className="flex items-center space-x-3">
                    <div className="flex-shrink-0">
                      {tx.status === 'success' ? (
                        <CheckCircle className="w-5 h-5 text-green-500" />
                      ) : tx.status === 'pending' ? (
                        <Clock className="w-5 h-5 text-yellow-500" />
                      ) : (
                        <AlertTriangle className="w-5 h-5 text-red-500" />
                      )}
                    </div>
                    <div>
                      <div className="font-mono text-sm text-gray-900">
                        {tx.hash.slice(0, 10)}...{tx.hash.slice(-8)}
                      </div>
                      <div className="text-xs text-gray-500">
                        {tx.from.slice(0, 8)}...{tx.from.slice(-6)} → {tx.to.slice(0, 8)}...{tx.to.slice(-6)}
                      </div>
                    </div>
                  </div>
                  
                  <div className="text-right">
                    <div className="font-semibold text-gray-900">
                      {tx.amount} {tx.token}
                    </div>
                    <div className="text-xs text-gray-500">{tx.time}</div>
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}

