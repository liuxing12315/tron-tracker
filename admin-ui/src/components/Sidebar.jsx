import { Link, useLocation } from 'react-router-dom'
import { 
  LayoutDashboard, 
  ArrowRightLeft, 
  MapPin, 
  Webhook, 
  Wifi, 
  Key, 
  Settings, 
  FileText,
  ChevronLeft,
  ChevronRight,
  Activity
} from 'lucide-react'
import { cn } from '@/lib/utils'

const navigation = [
  { name: 'Dashboard', href: '/dashboard', icon: LayoutDashboard },
  { name: 'Transactions', href: '/transactions', icon: ArrowRightLeft },
  { name: 'Addresses', href: '/addresses', icon: MapPin },
  { name: 'Webhooks', href: '/webhooks', icon: Webhook },
  { name: 'WebSockets', href: '/websockets', icon: Wifi },
  { name: 'API Keys', href: '/api-keys', icon: Key },
  { name: 'Logs', href: '/logs', icon: FileText },
  { name: 'Settings', href: '/settings', icon: Settings },
]

export function Sidebar({ open, onToggle }) {
  const location = useLocation()

  return (
    <div className={cn(
      "fixed inset-y-0 left-0 z-50 bg-white border-r border-gray-200 transition-all duration-300",
      open ? "w-64" : "w-16"
    )}>
      {/* Header */}
      <div className="flex items-center justify-between h-16 px-4 border-b border-gray-200">
        <div className={cn("flex items-center space-x-3", !open && "justify-center")}>
          <div className="flex items-center justify-center w-8 h-8 bg-blue-600 rounded-lg">
            <Activity className="w-5 h-5 text-white" />
          </div>
          {open && (
            <div>
              <h1 className="text-lg font-semibold text-gray-900">TRX Tracker</h1>
              <p className="text-xs text-gray-500">Admin Dashboard</p>
            </div>
          )}
        </div>
        
        <button
          onClick={onToggle}
          className="p-1.5 rounded-lg hover:bg-gray-100 transition-colors"
        >
          {open ? (
            <ChevronLeft className="w-4 h-4 text-gray-600" />
          ) : (
            <ChevronRight className="w-4 h-4 text-gray-600" />
          )}
        </button>
      </div>

      {/* Navigation */}
      <nav className="flex-1 px-3 py-4 space-y-1">
        {navigation.map((item) => {
          const isActive = location.pathname === item.href
          const Icon = item.icon
          
          return (
            <Link
              key={item.name}
              to={item.href}
              className={cn(
                "flex items-center px-3 py-2.5 text-sm font-medium rounded-lg transition-all duration-200",
                isActive
                  ? "bg-blue-50 text-blue-700 border-r-2 border-blue-600"
                  : "text-gray-700 hover:bg-gray-50 hover:text-gray-900",
                !open && "justify-center"
              )}
              title={!open ? item.name : undefined}
            >
              <Icon className={cn("w-5 h-5", open && "mr-3")} />
              {open && <span>{item.name}</span>}
            </Link>
          )
        })}
      </nav>

      {/* Footer */}
      {open && (
        <div className="p-4 border-t border-gray-200">
          <div className="flex items-center space-x-3">
            <div className="w-8 h-8 bg-gray-300 rounded-full"></div>
            <div className="flex-1 min-w-0">
              <p className="text-sm font-medium text-gray-900 truncate">Admin User</p>
              <p className="text-xs text-gray-500 truncate">admin@trontracker.com</p>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

