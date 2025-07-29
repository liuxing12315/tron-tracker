import { useState } from 'react'
import { Menu, Bell, Search, RefreshCw, AlertCircle, CheckCircle } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'

export function Header({ onMenuClick, systemStats }) {
  const [searchQuery, setSearchQuery] = useState('')
  const [isRefreshing, setIsRefreshing] = useState(false)

  const handleRefresh = async () => {
    setIsRefreshing(true)
    // Simulate refresh delay
    setTimeout(() => setIsRefreshing(false), 1000)
  }

  const formatNumber = (num) => {
    if (num >= 1000000) {
      return (num / 1000000).toFixed(1) + 'M'
    } else if (num >= 1000) {
      return (num / 1000).toFixed(1) + 'K'
    }
    return num.toLocaleString()
  }

  const formatUptime = (seconds) => {
    const days = Math.floor(seconds / 86400)
    const hours = Math.floor((seconds % 86400) / 3600)
    return `${days}d ${hours}h`
  }

  return (
    <header className="bg-white border-b border-gray-200 px-6 py-4">
      <div className="flex items-center justify-between">
        {/* Left side */}
        <div className="flex items-center space-x-4">
          <Button
            variant="ghost"
            size="sm"
            onClick={onMenuClick}
            className="lg:hidden"
          >
            <Menu className="w-5 h-5" />
          </Button>
          
          <div className="relative">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
            <Input
              type="text"
              placeholder="Search transactions, addresses..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="pl-10 w-80"
            />
          </div>
        </div>

        {/* Center - System Status */}
        <div className="hidden lg:flex items-center space-x-6">
          <div className="flex items-center space-x-2">
            <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
            <span className="text-sm text-gray-600">
              Block #{formatNumber(systemStats.currentBlock)}
            </span>
          </div>
          
          <div className="flex items-center space-x-2">
            <span className="text-sm text-gray-600">
              {systemStats.scanSpeed.toFixed(1)} blocks/min
            </span>
          </div>
          
          <div className="flex items-center space-x-2">
            {systemStats.successRate >= 99 ? (
              <CheckCircle className="w-4 h-4 text-green-500" />
            ) : (
              <AlertCircle className="w-4 h-4 text-yellow-500" />
            )}
            <span className="text-sm text-gray-600">
              {systemStats.successRate.toFixed(1)}% success
            </span>
          </div>
        </div>

        {/* Right side */}
        <div className="flex items-center space-x-4">
          {/* Quick Stats */}
          <div className="hidden md:flex items-center space-x-4">
            <div className="text-center">
              <div className="text-lg font-semibold text-gray-900">
                {formatNumber(systemStats.totalTransactions)}
              </div>
              <div className="text-xs text-gray-500">Transactions</div>
            </div>
            
            <div className="text-center">
              <div className="text-lg font-semibold text-gray-900">
                {systemStats.websocketConnections}
              </div>
              <div className="text-xs text-gray-500">WS Connections</div>
            </div>
            
            <div className="text-center">
              <div className="text-lg font-semibold text-gray-900">
                {formatUptime(systemStats.uptime)}
              </div>
              <div className="text-xs text-gray-500">Uptime</div>
            </div>
          </div>

          {/* Actions */}
          <Button
            variant="ghost"
            size="sm"
            onClick={handleRefresh}
            disabled={isRefreshing}
          >
            <RefreshCw className={`w-4 h-4 ${isRefreshing ? 'animate-spin' : ''}`} />
          </Button>
          
          <Button variant="ghost" size="sm" className="relative">
            <Bell className="w-4 h-4" />
            <Badge 
              variant="destructive" 
              className="absolute -top-1 -right-1 w-5 h-5 text-xs flex items-center justify-center p-0"
            >
              3
            </Badge>
          </Button>
        </div>
      </div>
    </header>
  )
}

