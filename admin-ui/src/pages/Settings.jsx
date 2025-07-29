import { useState } from 'react'
import { Save, RefreshCw, Database, Globe, Shield, Bell } from 'lucide-react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'

export function Settings() {
  const [settings, setSettings] = useState({
    blockchain: {
      startBlock: '62800000',
      batchSize: '100',
      scanInterval: '3'
    },
    database: {
      maxConnections: '20',
      connectionTimeout: '30'
    },
    api: {
      rateLimit: '1000',
      corsEnabled: true
    },
    notifications: {
      webhookTimeout: '30',
      retryAttempts: '3'
    }
  })

  const handleSave = () => {
    console.log('Saving settings:', settings)
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Settings</h1>
          <p className="text-gray-600">Configure system parameters and preferences</p>
        </div>
        <Button onClick={handleSave} className="flex items-center space-x-2">
          <Save className="w-4 h-4" />
          <span>Save Changes</span>
        </Button>
      </div>

      <Tabs defaultValue="blockchain" className="space-y-6">
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="blockchain">Blockchain</TabsTrigger>
          <TabsTrigger value="database">Database</TabsTrigger>
          <TabsTrigger value="api">API</TabsTrigger>
          <TabsTrigger value="notifications">Notifications</TabsTrigger>
        </TabsList>

        <TabsContent value="blockchain">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center space-x-2">
                <Globe className="w-5 h-5" />
                <span>Blockchain Configuration</span>
              </CardTitle>
              <CardDescription>
                Configure blockchain scanning parameters and node settings
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div className="space-y-2">
                  <Label htmlFor="startBlock">Start Block Number</Label>
                  <Input
                    id="startBlock"
                    value={settings.blockchain.startBlock}
                    onChange={(e) => setSettings({
                      ...settings,
                      blockchain: { ...settings.blockchain, startBlock: e.target.value }
                    })}
                  />
                  <p className="text-xs text-gray-500">Block number to start scanning from</p>
                </div>

                <div className="space-y-2">
                  <Label htmlFor="batchSize">Batch Size</Label>
                  <Input
                    id="batchSize"
                    value={settings.blockchain.batchSize}
                    onChange={(e) => setSettings({
                      ...settings,
                      blockchain: { ...settings.blockchain, batchSize: e.target.value }
                    })}
                  />
                  <p className="text-xs text-gray-500">Number of blocks to process in each batch</p>
                </div>

                <div className="space-y-2">
                  <Label htmlFor="scanInterval">Scan Interval (seconds)</Label>
                  <Input
                    id="scanInterval"
                    value={settings.blockchain.scanInterval}
                    onChange={(e) => setSettings({
                      ...settings,
                      blockchain: { ...settings.blockchain, scanInterval: e.target.value }
                    })}
                  />
                  <p className="text-xs text-gray-500">Time between scan cycles</p>
                </div>
              </div>

              <div className="space-y-4">
                <h3 className="text-lg font-medium">Node Configuration</h3>
                <div className="space-y-4">
                  <div className="p-4 border rounded-lg">
                    <div className="flex items-center justify-between mb-2">
                      <h4 className="font-medium">TronGrid (Primary)</h4>
                      <Switch defaultChecked />
                    </div>
                    <Input defaultValue="https://api.trongrid.io" className="mb-2" />
                    <p className="text-xs text-gray-500">Primary Tron network node</p>
                  </div>

                  <div className="p-4 border rounded-lg">
                    <div className="flex items-center justify-between mb-2">
                      <h4 className="font-medium">GetBlock (Backup)</h4>
                      <Switch defaultChecked />
                    </div>
                    <Input defaultValue="https://go.getblock.io" className="mb-2" />
                    <p className="text-xs text-gray-500">Backup node for redundancy</p>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="database">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center space-x-2">
                <Database className="w-5 h-5" />
                <span>Database Configuration</span>
              </CardTitle>
              <CardDescription>
                Configure database connection and performance settings
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div className="space-y-2">
                  <Label htmlFor="maxConnections">Max Connections</Label>
                  <Input
                    id="maxConnections"
                    value={settings.database.maxConnections}
                    onChange={(e) => setSettings({
                      ...settings,
                      database: { ...settings.database, maxConnections: e.target.value }
                    })}
                  />
                  <p className="text-xs text-gray-500">Maximum database connections</p>
                </div>

                <div className="space-y-2">
                  <Label htmlFor="connectionTimeout">Connection Timeout (seconds)</Label>
                  <Input
                    id="connectionTimeout"
                    value={settings.database.connectionTimeout}
                    onChange={(e) => setSettings({
                      ...settings,
                      database: { ...settings.database, connectionTimeout: e.target.value }
                    })}
                  />
                  <p className="text-xs text-gray-500">Database connection timeout</p>
                </div>
              </div>

              <div className="space-y-4">
                <h3 className="text-lg font-medium">Connection Status</h3>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div className="p-4 border rounded-lg">
                    <div className="flex items-center justify-between">
                      <span>PostgreSQL</span>
                      <span className="text-green-600 text-sm">Connected</span>
                    </div>
                    <p className="text-xs text-gray-500 mt-1">Primary database</p>
                  </div>
                  <div className="p-4 border rounded-lg">
                    <div className="flex items-center justify-between">
                      <span>Redis Cache</span>
                      <span className="text-green-600 text-sm">Connected</span>
                    </div>
                    <p className="text-xs text-gray-500 mt-1">Cache layer</p>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="api">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center space-x-2">
                <Shield className="w-5 h-5" />
                <span>API Configuration</span>
              </CardTitle>
              <CardDescription>
                Configure API security and performance settings
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div className="space-y-2">
                  <Label htmlFor="rateLimit">Rate Limit (requests/minute)</Label>
                  <Input
                    id="rateLimit"
                    value={settings.api.rateLimit}
                    onChange={(e) => setSettings({
                      ...settings,
                      api: { ...settings.api, rateLimit: e.target.value }
                    })}
                  />
                  <p className="text-xs text-gray-500">Maximum requests per minute per API key</p>
                </div>

                <div className="space-y-2">
                  <Label htmlFor="corsEnabled">CORS Enabled</Label>
                  <div className="flex items-center space-x-2">
                    <Switch
                      checked={settings.api.corsEnabled}
                      onCheckedChange={(checked) => setSettings({
                        ...settings,
                        api: { ...settings.api, corsEnabled: checked }
                      })}
                    />
                    <span className="text-sm text-gray-600">
                      {settings.api.corsEnabled ? 'Enabled' : 'Disabled'}
                    </span>
                  </div>
                  <p className="text-xs text-gray-500">Allow cross-origin requests</p>
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="notifications">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center space-x-2">
                <Bell className="w-5 h-5" />
                <span>Notification Configuration</span>
              </CardTitle>
              <CardDescription>
                Configure webhook and WebSocket notification settings
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div className="space-y-2">
                  <Label htmlFor="webhookTimeout">Webhook Timeout (seconds)</Label>
                  <Input
                    id="webhookTimeout"
                    value={settings.notifications.webhookTimeout}
                    onChange={(e) => setSettings({
                      ...settings,
                      notifications: { ...settings.notifications, webhookTimeout: e.target.value }
                    })}
                  />
                  <p className="text-xs text-gray-500">Timeout for webhook requests</p>
                </div>

                <div className="space-y-2">
                  <Label htmlFor="retryAttempts">Retry Attempts</Label>
                  <Input
                    id="retryAttempts"
                    value={settings.notifications.retryAttempts}
                    onChange={(e) => setSettings({
                      ...settings,
                      notifications: { ...settings.notifications, retryAttempts: e.target.value }
                    })}
                  />
                  <p className="text-xs text-gray-500">Number of retry attempts for failed webhooks</p>
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  )
}

