import { useState, useEffect } from 'react'
import { Plus, Key, Eye, EyeOff, Copy, Trash2, Edit, MoreHorizontal, Activity } from 'lucide-react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu'
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Checkbox } from '@/components/ui/checkbox'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { apiKeyService } from '../services'
import { useApp } from '../contexts/AppContext'
import { useAsync } from '../hooks/useAsync'
import { LoadingSpinner } from '../components/LoadingSpinner'

export function ApiKeys() {
  const { apiKeys, loading, addNotification } = useApp()
  const [visibleKeys, setVisibleKeys] = useState(new Set())
  const [selectedApiKey, setSelectedApiKey] = useState(null)
  const [showCreateDialog, setShowCreateDialog] = useState(false)
  const [showEditDialog, setShowEditDialog] = useState(false)
  const [newApiKey, setNewApiKey] = useState({ name: '', permissions: [] })
  const [editApiKey, setEditApiKey] = useState({ name: '', permissions: [] })
  const [generatedKey, setGeneratedKey] = useState(null)
  const [availablePermissions, setAvailablePermissions] = useState([])
  const [stats, setStats] = useState({ total: 0, totalRequests: 0, successRate: 99.8 })

  // Load API keys and permissions
  const { execute: loadApiKeys } = useAsync(
    () => apiKeyService.getApiKeys(),
    [],
    { 
      immediate: true,
      loadingKey: 'apiKeys',
      onSuccess: (data) => {
        setStats({
          total: data.length,
          totalRequests: data.reduce((sum, key) => sum + (key.request_count || 0), 0),
          successRate: 99.8 // Would come from API
        })
      }
    }
  )

  useEffect(() => {
    setAvailablePermissions(apiKeyService.getAvailablePermissions())
  }, [])

  const toggleKeyVisibility = (keyId) => {
    const newVisible = new Set(visibleKeys)
    if (newVisible.has(keyId)) {
      newVisible.delete(keyId)
    } else {
      newVisible.add(keyId)
    }
    setVisibleKeys(newVisible)
  }

  const handleCreateApiKey = async () => {
    try {
      const result = await apiKeyService.createApiKey(newApiKey)
      setGeneratedKey(result.key)
      setNewApiKey({ name: '', permissions: [] })
      addNotification({
        type: 'success',
        title: 'API Key Created',
        message: 'Your new API key has been generated. Make sure to copy it now.'
      })
      loadApiKeys()
    } catch (error) {
      addNotification({
        type: 'error',
        title: 'Creation Failed',
        message: error.message
      })
    }
  }

  const handleEditApiKey = async () => {
    try {
      await apiKeyService.updateApiKey(selectedApiKey.id, editApiKey)
      setShowEditDialog(false)
      setSelectedApiKey(null)
      addNotification({
        type: 'success',
        message: 'API key updated successfully'
      })
      loadApiKeys()
    } catch (error) {
      addNotification({
        type: 'error',
        title: 'Update Failed',
        message: error.message
      })
    }
  }

  const handleDeleteApiKey = async (keyId) => {
    try {
      await apiKeyService.deleteApiKey(keyId)
      addNotification({
        type: 'success',
        message: 'API key deleted successfully'
      })
      loadApiKeys()
    } catch (error) {
      addNotification({
        type: 'error',
        title: 'Deletion Failed',
        message: error.message
      })
    }
  }

  const handleToggleApiKey = async (keyId, enabled) => {
    try {
      await apiKeyService.toggleApiKey(keyId, enabled)
      addNotification({
        type: 'success',
        message: `API key ${enabled ? 'enabled' : 'disabled'} successfully`
      })
      loadApiKeys()
    } catch (error) {
      addNotification({
        type: 'error',
        title: 'Toggle Failed',
        message: error.message
      })
    }
  }

  const handleRegenerateKey = async (keyId) => {
    try {
      const result = await apiKeyService.regenerateApiKey(keyId)
      setGeneratedKey(result.key)
      addNotification({
        type: 'success',
        title: 'API Key Regenerated',
        message: 'Your API key has been regenerated. Make sure to copy the new key.'
      })
      loadApiKeys()
    } catch (error) {
      addNotification({
        type: 'error',
        title: 'Regeneration Failed',
        message: error.message
      })
    }
  }

  const copyToClipboard = (text) => {
    navigator.clipboard.writeText(text)
    addNotification({
      type: 'success',
      message: 'Copied to clipboard'
    })
  }

  const handlePermissionChange = (permission, checked, isEdit = false) => {
    const target = isEdit ? editApiKey : newApiKey
    const setter = isEdit ? setEditApiKey : setNewApiKey
    
    const newPermissions = checked
      ? [...target.permissions, permission]
      : target.permissions.filter(p => p !== permission)
    
    setter({ ...target, permissions: newPermissions })
  }

  if (loading.apiKeys) {
    return <LoadingSpinner text="Loading API keys..." />
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">API Keys</h1>
          <p className="text-gray-600">Manage API keys and access permissions</p>
        </div>
        <Dialog open={showCreateDialog} onOpenChange={setShowCreateDialog}>
          <DialogTrigger asChild>
            <Button className="flex items-center space-x-2">
              <Plus className="w-4 h-4" />
              <span>Generate API Key</span>
            </Button>
          </DialogTrigger>
          <DialogContent className="max-w-md">
            <DialogHeader>
              <DialogTitle>Create New API Key</DialogTitle>
              <DialogDescription>
                Generate a new API key with specific permissions
              </DialogDescription>
            </DialogHeader>
            <div className="space-y-4">
              <div>
                <Label htmlFor="name">Name</Label>
                <Input
                  id="name"
                  placeholder="Enter API key name"
                  value={newApiKey.name}
                  onChange={(e) => setNewApiKey({ ...newApiKey, name: e.target.value })}
                />
              </div>
              <div>
                <Label>Permissions</Label>
                <div className="grid grid-cols-1 gap-2 mt-2">
                  {availablePermissions.map((permission) => (
                    <div key={permission.id} className="flex items-center space-x-2">
                      <Checkbox
                        id={permission.id}
                        checked={newApiKey.permissions.includes(permission.id)}
                        onCheckedChange={(checked) => handlePermissionChange(permission.id, checked)}
                      />
                      <Label htmlFor={permission.id} className="text-sm">
                        {permission.name}
                      </Label>
                    </div>
                  ))}
                </div>
              </div>
              <div className="flex justify-end space-x-2">
                <Button variant="outline" onClick={() => setShowCreateDialog(false)}>
                  Cancel
                </Button>
                <Button onClick={handleCreateApiKey} disabled={!newApiKey.name.trim()}>
                  Generate
                </Button>
              </div>
            </div>
          </DialogContent>
        </Dialog>
      </div>

      {generatedKey && (
        <Alert>
          <Key className="h-4 w-4" />
          <AlertDescription>
            <div className="space-y-2">
              <p className="font-medium">Your new API key has been generated:</p>
              <div className="flex items-center space-x-2 p-2 bg-gray-100 rounded font-mono text-sm">
                <span className="flex-1 break-all">{generatedKey}</span>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => copyToClipboard(generatedKey)}
                >
                  <Copy className="w-3 h-3" />
                </Button>
              </div>
              <p className="text-sm text-gray-600">
                Make sure to copy this key now. You won't be able to see it again!
              </p>
              <Button size="sm" onClick={() => setGeneratedKey(null)}>
                I've copied the key
              </Button>
            </div>
          </AlertDescription>
        </Alert>
      )}

      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total API Keys</CardTitle>
            <Key className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.total}</div>
            <p className="text-xs text-muted-foreground">Active keys</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Requests</CardTitle>
            <Activity className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.totalRequests.toLocaleString()}</div>
            <p className="text-xs text-muted-foreground">All time requests</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Success Rate</CardTitle>
            <Key className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.successRate}%</div>
            <p className="text-xs text-muted-foreground">Last 30 days</p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>API Keys</CardTitle>
          <CardDescription>Manage your API keys and their permissions</CardDescription>
        </CardHeader>
        <CardContent>
          {apiKeys.length === 0 ? (
            <div className="text-center py-8">
              <Key className="mx-auto h-12 w-12 text-gray-400" />
              <h3 className="mt-2 text-sm font-medium text-gray-900">No API keys</h3>
              <p className="mt-1 text-sm text-gray-500">Get started by creating a new API key.</p>
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Name</TableHead>
                  <TableHead>API Key</TableHead>
                  <TableHead>Permissions</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead>Requests</TableHead>
                  <TableHead>Last Used</TableHead>
                  <TableHead>Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {apiKeys.map((apiKey) => (
                  <TableRow key={apiKey.id}>
                    <TableCell className="font-medium">{apiKey.name}</TableCell>
                    <TableCell>
                      <div className="flex items-center space-x-2">
                        <span className="font-mono text-sm">
                          {visibleKeys.has(apiKey.id) 
                            ? apiKey.key || 'Hidden for security'
                            : (apiKey.key_preview || apiKey.id.substring(0, 8)) + '••••••••••••••••'
                          }
                        </span>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => toggleKeyVisibility(apiKey.id)}
                        >
                          {visibleKeys.has(apiKey.id) ? <EyeOff className="w-3 h-3" /> : <Eye className="w-3 h-3" />}
                        </Button>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => copyToClipboard(apiKey.key || apiKey.id)}
                        >
                          <Copy className="w-3 h-3" />
                        </Button>
                      </div>
                    </TableCell>
                    <TableCell>
                      <div className="flex flex-wrap gap-1">
                        {(apiKey.permissions || []).map((permission) => (
                          <Badge key={permission} variant="outline" className="text-xs">
                            {permission.replace('_', ' ')}
                          </Badge>
                        ))}
                      </div>
                    </TableCell>
                    <TableCell>
                      <Badge variant={apiKey.enabled ? 'default' : 'secondary'}>
                        {apiKey.enabled ? 'Active' : 'Disabled'}
                      </Badge>
                    </TableCell>
                    <TableCell>{(apiKey.request_count || 0).toLocaleString()}</TableCell>
                    <TableCell>
                      {apiKey.last_used ? new Date(apiKey.last_used).toLocaleString() : 'Never'}
                    </TableCell>
                    <TableCell>
                      <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                          <Button variant="ghost" size="sm">
                            <MoreHorizontal className="w-4 h-4" />
                          </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent>
                          <DropdownMenuItem
                            onClick={() => {
                              setSelectedApiKey(apiKey)
                              setEditApiKey({ 
                                name: apiKey.name, 
                                permissions: apiKey.permissions || [] 
                              })
                              setShowEditDialog(true)
                            }}
                          >
                            <Edit className="w-4 h-4 mr-2" />
                            Edit
                          </DropdownMenuItem>
                          <DropdownMenuItem
                            onClick={() => handleToggleApiKey(apiKey.id, !apiKey.enabled)}
                          >
                            {apiKey.enabled ? 'Disable' : 'Enable'}
                          </DropdownMenuItem>
                          <DropdownMenuItem onClick={() => handleRegenerateKey(apiKey.id)}>
                            <Key className="w-4 h-4 mr-2" />
                            Regenerate
                          </DropdownMenuItem>
                          <DropdownMenuItem
                            className="text-red-600"
                            onClick={() => handleDeleteApiKey(apiKey.id)}
                          >
                            <Trash2 className="w-4 h-4 mr-2" />
                            Delete
                          </DropdownMenuItem>
                        </DropdownMenuContent>
                      </DropdownMenu>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      {/* Edit Dialog */}
      <Dialog open={showEditDialog} onOpenChange={setShowEditDialog}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>Edit API Key</DialogTitle>
            <DialogDescription>
              Update the name and permissions for this API key
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div>
              <Label htmlFor="editName">Name</Label>
              <Input
                id="editName"
                value={editApiKey.name}
                onChange={(e) => setEditApiKey({ ...editApiKey, name: e.target.value })}
              />
            </div>
            <div>
              <Label>Permissions</Label>
              <div className="grid grid-cols-1 gap-2 mt-2">
                {availablePermissions.map((permission) => (
                  <div key={permission.id} className="flex items-center space-x-2">
                    <Checkbox
                      id={`edit-${permission.id}`}
                      checked={editApiKey.permissions.includes(permission.id)}
                      onCheckedChange={(checked) => handlePermissionChange(permission.id, checked, true)}
                    />
                    <Label htmlFor={`edit-${permission.id}`} className="text-sm">
                      {permission.name}
                    </Label>
                  </div>
                ))}
              </div>
            </div>
            <div className="flex justify-end space-x-2">
              <Button variant="outline" onClick={() => setShowEditDialog(false)}>
                Cancel
              </Button>
              <Button onClick={handleEditApiKey}>
                Save Changes
              </Button>
            </div>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  )
}

