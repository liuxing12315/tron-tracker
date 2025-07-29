import { useState, useEffect } from 'react'
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom'
import { Sidebar } from './components/Sidebar'
import { Header } from './components/Header'
import { Dashboard } from './pages/Dashboard'
import { Transactions } from './pages/Transactions'
import { Addresses } from './pages/Addresses'
import { Webhooks } from './pages/Webhooks'
import { WebSockets } from './pages/WebSockets'
import { ApiKeys } from './pages/ApiKeys'
import { Settings } from './pages/Settings'
import { Logs } from './pages/Logs'
import './App.css'

function App() {
  const [sidebarOpen, setSidebarOpen] = useState(true)
  const [systemStats, setSystemStats] = useState({
    totalTransactions: 0,
    totalAddresses: 0,
    currentBlock: 0,
    scanSpeed: 0,
    activeWebhooks: 0,
    websocketConnections: 0,
    apiRequestsToday: 0,
    successRate: 0,
    uptime: 0
  })

  useEffect(() => {
    // Fetch system stats
    fetchSystemStats()
    
    // Set up periodic refresh
    const interval = setInterval(fetchSystemStats, 30000) // 30 seconds
    
    return () => clearInterval(interval)
  }, [])

  const fetchSystemStats = async () => {
    try {
      const response = await fetch('/api/system/stats')
      if (response.ok) {
        const stats = await response.json()
        setSystemStats(stats)
      }
    } catch (error) {
      console.error('Failed to fetch system stats:', error)
      // Use mock data for demo
      setSystemStats({
        totalTransactions: 1247856,
        totalAddresses: 89234,
        currentBlock: 62845149,
        scanSpeed: 18.5,
        activeWebhooks: 12,
        websocketConnections: 156,
        apiRequestsToday: 45678,
        successRate: 99.2,
        uptime: 2847600 // 33 days
      })
    }
  }

  return (
    <Router>
      <div className="min-h-screen bg-gray-50 flex">
        <Sidebar open={sidebarOpen} onToggle={() => setSidebarOpen(!sidebarOpen)} />
        
        <div className={`flex-1 flex flex-col transition-all duration-300 ${sidebarOpen ? 'ml-64' : 'ml-16'}`}>
          <Header 
            onMenuClick={() => setSidebarOpen(!sidebarOpen)}
            systemStats={systemStats}
          />
          
          <main className="flex-1 p-6 overflow-auto">
            <Routes>
              <Route path="/" element={<Navigate to="/dashboard" replace />} />
              <Route path="/dashboard" element={<Dashboard systemStats={systemStats} />} />
              <Route path="/transactions" element={<Transactions />} />
              <Route path="/addresses" element={<Addresses />} />
              <Route path="/webhooks" element={<Webhooks />} />
              <Route path="/websockets" element={<WebSockets />} />
              <Route path="/api-keys" element={<ApiKeys />} />
              <Route path="/settings" element={<Settings />} />
              <Route path="/logs" element={<Logs />} />
            </Routes>
          </main>
        </div>
      </div>
    </Router>
  )
}

export default App

