import { useState, useEffect } from 'react'
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom'
import { AuthProvider } from './contexts/AuthContext'
import { AppProvider } from './contexts/AppContext'
import { WebSocketProvider } from './contexts/WebSocketContext'
import { ProtectedRoute } from './components/ProtectedRoute'
import { NotificationSystem } from './components/NotificationSystem'
import ErrorBoundary from './components/ErrorBoundary'
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
import { systemService } from './services'
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
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState(null)

  useEffect(() => {
    // Fetch system stats on mount
    fetchSystemStats()
    
    // Set up periodic refresh every 30 seconds
    const interval = setInterval(fetchSystemStats, 30000)
    
    return () => clearInterval(interval)
  }, [])

  const fetchSystemStats = async () => {
    try {
      setError(null)
      const stats = await systemService.getSystemStats()
      setSystemStats(stats)
    } catch (error) {
      console.error('Failed to fetch system stats:', error)
      setError(error.message)
      // systemService already provides fallback data
    } finally {
      setLoading(false)
    }
  }

  return (
    <ErrorBoundary>
      <AuthProvider>
        <AppProvider>
          <WebSocketProvider>
            <Router>
              <ProtectedRoute>
                <div className="min-h-screen bg-gray-50 flex">
                  <Sidebar open={sidebarOpen} onToggle={() => setSidebarOpen(!sidebarOpen)} />
                  
                  <div className={`flex-1 flex flex-col transition-all duration-300 ${sidebarOpen ? 'ml-64' : 'ml-16'}`}>
                    <Header 
                      onMenuClick={() => setSidebarOpen(!sidebarOpen)}
                      systemStats={systemStats}
                    />
                    
                    <main className="flex-1 p-6 overflow-auto">
                      <ErrorBoundary>
                        <Routes>
                          <Route path="/" element={<Navigate to="/dashboard" replace />} />
                          <Route path="/dashboard" element={<Dashboard systemStats={systemStats} />} />
                          <Route path="/transactions" element={
                            <ProtectedRoute permission="read_transactions">
                              <Transactions />
                            </ProtectedRoute>
                          } />
                          <Route path="/addresses" element={
                            <ProtectedRoute permission="read_addresses">
                              <Addresses />
                            </ProtectedRoute>
                          } />
                          <Route path="/webhooks" element={
                            <ProtectedRoute permission="manage_webhooks">
                              <Webhooks />
                            </ProtectedRoute>
                          } />
                          <Route path="/websockets" element={
                            <ProtectedRoute permission="manage_system">
                              <WebSockets />
                            </ProtectedRoute>
                          } />
                          <Route path="/api-keys" element={
                            <ProtectedRoute permission="manage_api_keys">
                              <ApiKeys />
                            </ProtectedRoute>
                          } />
                          <Route path="/settings" element={
                            <ProtectedRoute permission="manage_system">
                              <Settings />
                            </ProtectedRoute>
                          } />
                          <Route path="/logs" element={
                            <ProtectedRoute permission="manage_system">
                              <Logs />
                            </ProtectedRoute>
                          } />
                        </Routes>
                      </ErrorBoundary>
                    </main>
                  </div>
                  <NotificationSystem />
                </div>
              </ProtectedRoute>
            </Router>
          </WebSocketProvider>
        </AppProvider>
      </AuthProvider>
    </ErrorBoundary>
  )
}

export default App

