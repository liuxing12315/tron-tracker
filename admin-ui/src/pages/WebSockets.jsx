import { useState } from 'react'
import { Wifi, Users, MessageSquare, Activity } from 'lucide-react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'

export function WebSockets() {
  const [connections] = useState([
    {
      id: '1',
      clientIp: '192.168.1.100',
      userAgent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64)',
      connectedAt: '2024-01-15T10:30:00Z',
      messagesSent: 156,
      messagesReceived: 23,
      subscriptions: ['transactions', 'large_transfers']
    },
    {
      id: '2',
      clientIp: '10.0.0.50',
      userAgent: 'TronTracker Mobile App v1.2.0',
      connectedAt: '2024-01-15T09:45:00Z',
      messagesSent: 89,
      messagesReceived: 12,
      subscriptions: ['transactions']
    }
  ])

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-gray-900">WebSocket Connections</h1>
        <p className="text-gray-600">Monitor real-time WebSocket connections and activity</p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Active Connections</CardTitle>
            <Users className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{connections.length}</div>
            <p className="text-xs text-muted-foreground">Currently connected</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Messages Sent</CardTitle>
            <MessageSquare className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {connections.reduce((sum, conn) => sum + conn.messagesSent, 0)}
            </div>
            <p className="text-xs text-muted-foreground">Total messages sent</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Messages Received</CardTitle>
            <Activity className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {connections.reduce((sum, conn) => sum + conn.messagesReceived, 0)}
            </div>
            <p className="text-xs text-muted-foreground">Total messages received</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Avg Latency</CardTitle>
            <Wifi className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">24ms</div>
            <p className="text-xs text-muted-foreground">Average response time</p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Active Connections</CardTitle>
          <CardDescription>Real-time WebSocket connection details</CardDescription>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Client IP</TableHead>
                <TableHead>User Agent</TableHead>
                <TableHead>Connected</TableHead>
                <TableHead>Messages</TableHead>
                <TableHead>Subscriptions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {connections.map((connection) => (
                <TableRow key={connection.id}>
                  <TableCell className="font-mono">{connection.clientIp}</TableCell>
                  <TableCell className="max-w-xs truncate">{connection.userAgent}</TableCell>
                  <TableCell>{new Date(connection.connectedAt).toLocaleString()}</TableCell>
                  <TableCell>
                    <div className="text-sm">
                      <div>Sent: {connection.messagesSent}</div>
                      <div>Received: {connection.messagesReceived}</div>
                    </div>
                  </TableCell>
                  <TableCell>
                    <div className="flex flex-wrap gap-1">
                      {connection.subscriptions.map((sub) => (
                        <Badge key={sub} variant="outline" className="text-xs">
                          {sub}
                        </Badge>
                      ))}
                    </div>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  )
}

