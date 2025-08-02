import { useState, useEffect, useCallback } from 'react'
import { Search, Filter, Download, RefreshCw, AlertTriangle, Info, AlertCircle, Clock, Trash2 } from 'lucide-react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { LoadingSpinner } from '@/components/LoadingSpinner'
import { useAsync } from '@/hooks/useAsync'
import { useApp } from '@/contexts/AppContext'
import { logService } from '@/services/logService'

export function Logs() {
  const [searchQuery, setSearchQuery] = useState('')
  const [levelFilter, setLevelFilter] = useState('all')
  const [moduleFilter, setModuleFilter] = useState('all')
  const [autoRefresh, setAutoRefresh] = useState(true)
  const [page, setPage] = useState(1)
  const [limit] = useState(50)
  const [logStats, setLogStats] = useState(null)
  
  const { addNotification } = useApp()
  
  // 获取日志数据
  const {
    data: logsData,
    loading: logsLoading,
    error: logsError,
    execute: fetchLogs
  } = useAsync()

  // 获取日志统计
  const {
    data: statsData,
    loading: statsLoading,
    execute: fetchStats
  } = useAsync()

  // 清空日志
  const {
    loading: clearLoading,
    execute: clearLogs
  } = useAsync()

  // 导出日志
  const {
    loading: exportLoading,
    execute: exportLogs
  } = useAsync()

  // 加载日志数据
  const loadLogs = useCallback(async () => {
    const filters = {}
    if (levelFilter !== 'all') filters.level = levelFilter.toLowerCase()
    if (moduleFilter !== 'all') filters.module = moduleFilter
    if (searchQuery) filters.search = searchQuery

    await fetchLogs(() => logService.getLogs(filters, { page, limit }))
  }, [levelFilter, moduleFilter, searchQuery, page, limit, fetchLogs])

  // 加载统计数据
  const loadStats = useCallback(async () => {
    await fetchStats(() => logService.getLogStats())
  }, [fetchStats])

  // 处理清空日志
  const handleClearLogs = async () => {
    if (!window.confirm('Are you sure you want to clear all logs? This action cannot be undone.')) {
      return
    }

    const result = await clearLogs(() => logService.clearLogs())
    if (result) {
      addNotification('success', `Cleared ${result.deleted_count} log entries`)
      loadLogs()
      loadStats()
    }
  }

  // 处理导出日志
  const handleExportLogs = async () => {
    const filters = {}
    if (levelFilter !== 'all') filters.level = levelFilter.toLowerCase()
    if (moduleFilter !== 'all') filters.module = moduleFilter
    if (searchQuery) filters.search = searchQuery

    const result = await exportLogs(() => logService.exportLogs(filters))
    if (result) {
      // 创建下载链接
      const blob = new Blob([result], { type: 'text/csv' })
      const url = window.URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `tron-tracker-logs-${new Date().toISOString().split('T')[0]}.csv`
      document.body.appendChild(a)
      a.click()
      document.body.removeChild(a)
      window.URL.revokeObjectURL(url)
      
      addNotification('success', 'Logs exported successfully')
    }
  }

  // 初始加载
  useEffect(() => {
    loadLogs()
    loadStats()
  }, [loadLogs, loadStats])

  // 自动刷新
  useEffect(() => {
    if (!autoRefresh) return

    const interval = setInterval(() => {
      loadLogs()
      loadStats()
    }, 30000) // 每30秒刷新

    return () => clearInterval(interval)
  }, [autoRefresh, loadLogs, loadStats])

  // 处理搜索和过滤变化
  useEffect(() => {
    setPage(1) // 重置到第一页
    loadLogs()
  }, [searchQuery, levelFilter, moduleFilter])

  const getLevelIcon = (level) => {
    const normalizedLevel = level?.toUpperCase()
    switch (normalizedLevel) {
      case 'ERROR':
        return <AlertTriangle className="w-4 h-4 text-red-500" />
      case 'WARN':
        return <AlertCircle className="w-4 h-4 text-yellow-500" />
      case 'INFO':
        return <Info className="w-4 h-4 text-blue-500" />
      case 'DEBUG':
      case 'TRACE':
        return <Info className="w-4 h-4 text-gray-500" />
      default:
        return <Clock className="w-4 h-4 text-gray-400" />
    }
  }

  const getLevelBadge = (level) => {
    const normalizedLevel = level?.toUpperCase()
    const variants = {
      ERROR: 'destructive',
      WARN: 'secondary',
      INFO: 'default',
      DEBUG: 'outline',
      TRACE: 'outline'
    }
    return (
      <Badge variant={variants[normalizedLevel] || 'outline'} className="text-xs">
        {normalizedLevel}
      </Badge>
    )
  }

  // 获取实际日志数据
  const logs = logsData?.logs || []
  const totalLogs = logsData?.total_count || 0
  const totalPages = logsData?.total_pages || 1

  // 获取统计数据
  const errorCount = statsData?.error_count || logs.filter(log => log.level?.toLowerCase() === 'error').length
  const warnCount = statsData?.warn_count || logs.filter(log => log.level?.toLowerCase() === 'warn').length
  const totalCount = statsData?.total_count || totalLogs

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
            disabled={logsLoading}
          >
            <RefreshCw className={`w-4 h-4 ${autoRefresh && logsLoading ? 'animate-spin' : ''}`} />
            <span>{autoRefresh ? 'Auto Refresh' : 'Manual'}</span>
          </Button>
          
          <Button 
            variant="outline" 
            size="sm" 
            className="flex items-center space-x-2"
            onClick={handleExportLogs}
            disabled={exportLoading}
          >
            <Download className="w-4 h-4" />
            <span>{exportLoading ? 'Exporting...' : 'Export'}</span>
          </Button>

          <Button 
            variant="outline" 
            size="sm" 
            className="flex items-center space-x-2 text-red-600 hover:text-red-700"
            onClick={handleClearLogs}
            disabled={clearLoading}
          >
            <Trash2 className="w-4 h-4" />
            <span>{clearLoading ? 'Clearing...' : 'Clear All'}</span>
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
            {statsLoading ? (
              <div className="flex items-center space-x-2">
                <LoadingSpinner size="sm" />
                <span className="text-sm text-gray-500">Loading...</span>
              </div>
            ) : (
              <>
                <div className="text-2xl font-bold">{totalCount.toLocaleString()}</div>
                <p className="text-xs text-muted-foreground">Last 24 hours</p>
              </>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Errors</CardTitle>
            <AlertTriangle className="h-4 w-4 text-red-500" />
          </CardHeader>
          <CardContent>
            {statsLoading ? (
              <div className="flex items-center space-x-2">
                <LoadingSpinner size="sm" />
                <span className="text-sm text-gray-500">Loading...</span>
              </div>
            ) : (
              <>
                <div className="text-2xl font-bold text-red-600">
                  {errorCount.toLocaleString()}
                </div>
                <p className="text-xs text-muted-foreground">Requires attention</p>
              </>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Warnings</CardTitle>
            <AlertCircle className="h-4 w-4 text-yellow-500" />
          </CardHeader>
          <CardContent>
            {statsLoading ? (
              <div className="flex items-center space-x-2">
                <LoadingSpinner size="sm" />
                <span className="text-sm text-gray-500">Loading...</span>
              </div>
            ) : (
              <>
                <div className="text-2xl font-bold text-yellow-600">
                  {warnCount.toLocaleString()}
                </div>
                <p className="text-xs text-muted-foreground">Monitor closely</p>
              </>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Success Rate</CardTitle>
            <Info className="h-4 w-4 text-green-500" />
          </CardHeader>
          <CardContent>
            {statsLoading ? (
              <div className="flex items-center space-x-2">
                <LoadingSpinner size="sm" />
                <span className="text-sm text-gray-500">Loading...</span>
              </div>
            ) : (
              <>
                <div className="text-2xl font-bold text-green-600">
                  {totalCount > 0 ? (((totalCount - errorCount) / totalCount) * 100).toFixed(1) : '100.0'}%
                </div>
                <p className="text-xs text-muted-foreground">System health</p>
              </>
            )}
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
                  {logService.getLogLevels().map((level) => (
                    <SelectItem key={level.value} value={level.value.toUpperCase()}>
                      {level.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              
              <Select value={moduleFilter} onValueChange={setModuleFilter}>
                <SelectTrigger className="w-40">
                  <SelectValue placeholder="Module" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Modules</SelectItem>
                  {logService.getLogModules().map((module) => (
                    <SelectItem key={module.value} value={module.value}>
                      {module.label}
                    </SelectItem>
                  ))}
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
            Showing {logs.length} of {totalLogs} log entries
            {totalPages > 1 && ` (Page ${page} of ${totalPages})`}
          </CardDescription>
        </CardHeader>
        <CardContent>
          {logsLoading ? (
            <div className="flex items-center justify-center py-8">
              <LoadingSpinner />
              <span className="ml-2 text-gray-500">Loading logs...</span>
            </div>
          ) : logsError ? (
            <div className="flex items-center justify-center py-8 text-red-500">
              <AlertTriangle className="w-5 h-5 mr-2" />
              <span>Failed to load logs: {logsError.message}</span>
            </div>
          ) : logs.length === 0 ? (
            <div className="flex items-center justify-center py-8 text-gray-500">
              <Info className="w-5 h-5 mr-2" />
              <span>No log entries found</span>
            </div>
          ) : (
            <div className="overflow-x-auto">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Time</TableHead>
                    <TableHead>Level</TableHead>
                    <TableHead>Module</TableHead>
                    <TableHead>Message</TableHead>
                    <TableHead>Details</TableHead>
                    <TableHead>Trace ID</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {logs.map((log) => (
                    <TableRow key={log.id}>
                      <TableCell>
                        <span className="font-mono text-sm">
                          {new Date(log.timestamp).toLocaleString()}
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
                      <TableCell className="max-w-xs">
                        {log.details && (
                          <span className="text-xs text-gray-500 truncate block">
                            {typeof log.details === 'string' 
                              ? log.details 
                              : JSON.stringify(log.details)
                            }
                          </span>
                        )}
                      </TableCell>
                      <TableCell>
                        {log.trace_id && (
                          <Badge variant="secondary" className="text-xs font-mono">
                            {log.trace_id.substring(0, 8)}...
                          </Badge>
                        )}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>
          )}

          {/* Pagination */}
          {totalPages > 1 && (
            <div className="flex items-center justify-between mt-4">
              <div className="text-sm text-gray-500">
                Showing {((page - 1) * limit) + 1} to {Math.min(page * limit, totalLogs)} of {totalLogs} entries
              </div>
              <div className="flex space-x-2">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => setPage(p => Math.max(1, p - 1))}
                  disabled={page <= 1 || logsLoading}
                >
                  Previous
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => setPage(p => Math.min(totalPages, p + 1))}
                  disabled={page >= totalPages || logsLoading}
                >
                  Next
                </Button>
              </div>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  )
}

