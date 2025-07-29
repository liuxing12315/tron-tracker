import { useState } from 'react'
import { Search, Filter, Download, RefreshCw, AlertTriangle, Info, AlertCircle } from 'lucide-react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'

const mockLogs = [
  {
    id: '1',
    timestamp: '2024-01-15T10:30:15Z',
    level: 'INFO',
    module: 'Scanner',
    message: 'Successfully processed block 62845149 with 25 transactions',
    details: 'Block processing completed in 152ms'
  },
  {
    id: '2',
    timestamp: '2024-01-15T10:30:12Z',
    level: 'WARN',
    module: 'WebSocket',
    message: 'Client connection timeout detected',
    details: 'Client 192.168.1.100 disconnected after 30s timeout'
  },
  {
    id: '3',
    timestamp: '2024-01-15T10:30:08Z',
    level: 'ERROR',
    module: 'Webhook',
    message: 'Failed to deliver webhook notification',
    details: 'HTTP 500 error from https://api.example.com/webhooks/payments'
  },
  {
    id: '4',
    timestamp: '2024-01-15T10:30:05Z',
    level: 'INFO',
    module: 'API',
    message: 'Multi-address query processed successfully',
    details: 'Queried 5 addresses, returned 127 transactions in 45ms'
  },
  {
    id: '5',
    timestamp: '2024-01-15T10:30:02Z',
    level: 'DEBUG',
    module: 'Database',
    message: 'Connection pool status check',
    details: 'Active: 8/20 connections, Queue: 0 pending'
  }
]

export function Logs() {
  const [logs] = useState(mockLogs)
  const [searchQuery, setSearchQuery] = useState('')
  const [levelFilter, setLevelFilter] = useState('all')
  const [moduleFilter, setModuleFilter] = useState('all')
  const [autoRefresh, setAutoRefresh] = useState(true)

  const getLevelIcon = (level) => {
    switch (level) {
      case 'ERROR':
        return <AlertTriangle className="w-4 h-4 text-red-500" />
      case 'WARN':
        return <AlertCircle className="w-4 h-4 text-yellow-500" />
      case 'INFO':
        return <Info className="w-4 h-4 text-blue-500" />
      case 'DEBUG':
        return <Info className="w-4 h-4 text-gray-500" />
      default:
        return null
    }
  }

  const getLevelBadge = (level) => {
    const variants = {
      ERROR: 'destructive',
      WARN: 'secondary',
      INFO: 'default',
      DEBUG: 'outline'
    }
    return (
      <Badge variant={variants[level]} className="text-xs">
        {level}
      </Badge>
    )
  }

  const filteredLogs = logs.filter(log => {
    const matchesSearch = !searchQuery || 
      log.message.toLowerCase().includes(searchQuery.toLowerCase()) ||
      log.module.toLowerCase().includes(searchQuery.toLowerCase())
    
    const matchesLevel = levelFilter === 'all' || log.level === levelFilter
    const matchesModule = moduleFilter === 'all' || log.module === moduleFilter
    
    return matchesSearch && matchesLevel && matchesModule
  })

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">System Logs</h1>
          <p className="text-gray-600">Monitor system activity and troubleshoot issues</p>
        </div>
        
        <div className="flex items-center space-x-2">
          <Button
            variant={autoRefresh ? "default" : "outline"}
            size="sm"
            onClick={() => setAutoRefresh(!autoRefresh)}
            className="flex items-center space-x-2"
          >
            <RefreshCw className={`w-4 h-4 ${autoRefresh ? 'animate-spin' : ''}`} />
            <span>{autoRefresh ? 'Auto Refresh' : 'Manual'}</span>
          </Button>
          
          <Button variant="outline" size="sm" className="flex items-center space-x-2">
            <Download className="w-4 h-4" />
            <span>Export</span>
          </Button>
        </div>
      </div>

      {/* Log Statistics */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Logs</CardTitle>
            <Info className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{logs.length}</div>
            <p className="text-xs text-muted-foreground">Last 24 hours</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Errors</CardTitle>
            <AlertTriangle className="h-4 w-4 text-red-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-red-600">
              {logs.filter(log => log.level === 'ERROR').length}
            </div>
            <p className="text-xs text-muted-foreground">Requires attention</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Warnings</CardTitle>
            <AlertCircle className="h-4 w-4 text-yellow-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-yellow-600">
              {logs.filter(log => log.level === 'WARN').length}
            </div>
            <p className="text-xs text-muted-foreground">Monitor closely</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Success Rate</CardTitle>
            <Info className="h-4 w-4 text-green-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-green-600">
              {(((logs.length - logs.filter(log => log.level === 'ERROR').length) / logs.length) * 100).toFixed(1)}%
            </div>
            <p className="text-xs text-muted-foreground">System health</p>
          </CardContent>
        </Card>
      </div>

      {/* Search and Filters */}
      <Card>
        <CardHeader>
          <CardTitle>Filter Logs</CardTitle>
          <CardDescription>Search and filter system logs by level, module, or content</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex flex-col lg:flex-row gap-4">
            <div className="flex-1">
              <div className="relative">
                <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
                <Input
                  type="text"
                  placeholder="Search logs by message or module..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="pl-10"
                />
              </div>
            </div>
            
            <div className="flex gap-2">
              <Select value={levelFilter} onValueChange={setLevelFilter}>
                <SelectTrigger className="w-32">
                  <SelectValue placeholder="Level" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Levels</SelectItem>
                  <SelectItem value="ERROR">Error</SelectItem>
                  <SelectItem value="WARN">Warning</SelectItem>
                  <SelectItem value="INFO">Info</SelectItem>
                  <SelectItem value="DEBUG">Debug</SelectItem>
                </SelectContent>
              </Select>
              
              <Select value={moduleFilter} onValueChange={setModuleFilter}>
                <SelectTrigger className="w-32">
                  <SelectValue placeholder="Module" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Modules</SelectItem>
                  <SelectItem value="Scanner">Scanner</SelectItem>
                  <SelectItem value="API">API</SelectItem>
                  <SelectItem value="WebSocket">WebSocket</SelectItem>
                  <SelectItem value="Webhook">Webhook</SelectItem>
                  <SelectItem value="Database">Database</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Log Results */}
      <Card>
        <CardHeader>
          <CardTitle>Log Entries</CardTitle>
          <CardDescription>
            Showing {filteredLogs.length} of {logs.length} log entries
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="overflow-x-auto">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Time</TableHead>
                  <TableHead>Level</TableHead>
                  <TableHead>Module</TableHead>
                  <TableHead>Message</TableHead>
                  <TableHead>Details</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {filteredLogs.map((log) => (
                  <TableRow key={log.id}>
                    <TableCell>
                      <span className="font-mono text-sm">
                        {new Date(log.timestamp).toLocaleTimeString()}
                      </span>
                    </TableCell>
                    <TableCell>
                      <div className="flex items-center space-x-2">
                        {getLevelIcon(log.level)}
                        {getLevelBadge(log.level)}
                      </div>
                    </TableCell>
                    <TableCell>
                      <Badge variant="outline" className="text-xs">
                        {log.module}
                      </Badge>
                    </TableCell>
                    <TableCell>
                      <span className="text-sm">{log.message}</span>
                    </TableCell>
                    <TableCell>
                      <span className="text-xs text-gray-500">{log.details}</span>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

