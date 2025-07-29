import { useState } from 'react'
import { Plus, Edit, Trash2, ToggleLeft, ToggleRight } from 'lucide-react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'

export function Webhooks() {
  const [webhooks] = useState([
    {
      id: '1',
      name: 'Payment Notifications',
      url: 'https://api.example.com/webhooks/payments',
      enabled: true,
      events: ['transaction'],
      successCount: 1234,
      failureCount: 5,
      lastTriggered: '2024-01-15T10:30:00Z'
    },
    {
      id: '2',
      name: 'Large Transfer Alerts',
      url: 'https://alerts.example.com/large-transfers',
      enabled: true,
      events: ['large_transfer'],
      successCount: 89,
      failureCount: 0,
      lastTriggered: '2024-01-15T09:45:00Z'
    }
  ])

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Webhooks</h1>
          <p className="text-gray-600">Manage webhook endpoints for real-time notifications</p>
        </div>
        <Button className="flex items-center space-x-2">
          <Plus className="w-4 h-4" />
          <span>Add Webhook</span>
        </Button>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Active Webhooks</CardTitle>
          <CardDescription>Configure and monitor your webhook endpoints</CardDescription>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Name</TableHead>
                <TableHead>URL</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Events</TableHead>
                <TableHead>Success Rate</TableHead>
                <TableHead>Last Triggered</TableHead>
                <TableHead>Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {webhooks.map((webhook) => (
                <TableRow key={webhook.id}>
                  <TableCell className="font-medium">{webhook.name}</TableCell>
                  <TableCell className="font-mono text-sm">{webhook.url}</TableCell>
                  <TableCell>
                    <Badge variant={webhook.enabled ? 'default' : 'secondary'}>
                      {webhook.enabled ? 'Enabled' : 'Disabled'}
                    </Badge>
                  </TableCell>
                  <TableCell>
                    <div className="flex flex-wrap gap-1">
                      {webhook.events.map((event) => (
                        <Badge key={event} variant="outline" className="text-xs">
                          {event}
                        </Badge>
                      ))}
                    </div>
                  </TableCell>
                  <TableCell>
                    {((webhook.successCount / (webhook.successCount + webhook.failureCount)) * 100).toFixed(1)}%
                  </TableCell>
                  <TableCell>
                    {new Date(webhook.lastTriggered).toLocaleString()}
                  </TableCell>
                  <TableCell>
                    <div className="flex items-center space-x-2">
                      <Button variant="ghost" size="sm">
                        <Edit className="w-4 h-4" />
                      </Button>
                      <Button variant="ghost" size="sm">
                        {webhook.enabled ? <ToggleRight className="w-4 h-4" /> : <ToggleLeft className="w-4 h-4" />}
                      </Button>
                      <Button variant="ghost" size="sm">
                        <Trash2 className="w-4 h-4" />
                      </Button>
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

