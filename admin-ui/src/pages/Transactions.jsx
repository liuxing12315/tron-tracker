import { useState, useEffect } from 'react'
import { Search, Filter, Download, ExternalLink, Copy, CheckCircle, Clock, AlertTriangle } from 'lucide-react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'

const mockTransactions = [
  {
    id: '1',
    hash: '0x1a2b3c4d5e6f7890abcdef1234567890abcdef12345678901234567890abcdef',
    blockNumber: 62845149,
    from: 'TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t',
    to: 'TLa2f6VPqDgRE67v1736s7bJ8Ray5wYjU7',
    value: '1250.50',
    token: 'USDT',
    status: 'success',
    timestamp: '2024-01-15T10:30:00Z',
    gasUsed: '14500',
    gasPrice: '420'
  },
  {
    id: '2',
    hash: '0x2b3c4d5e6f7890abcdef1234567890abcdef12345678901234567890abcdef12',
    blockNumber: 62845148,
    from: 'TLa2f6VPqDgRE67v1736s7bJ8Ray5wYjU7',
    to: 'TKzxdSv2FZKQrEqkKVgp5DcwEXBEKMg2Ax',
    value: '500.00',
    token: 'USDT',
    status: 'success',
    timestamp: '2024-01-15T10:25:00Z',
    gasUsed: '14200',
    gasPrice: '420'
  },
  {
    id: '3',
    hash: '0x3c4d5e6f7890abcdef1234567890abcdef12345678901234567890abcdef123a',
    blockNumber: 62845147,
    from: 'TKzxdSv2FZKQrEqkKVgp5DcwEXBEKMg2Ax',
    to: 'TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t',
    value: '10000.00',
    token: 'USDT',
    status: 'pending',
    timestamp: '2024-01-15T10:20:00Z',
    gasUsed: '15000',
    gasPrice: '420'
  },
  {
    id: '4',
    hash: '0x4d5e6f7890abcdef1234567890abcdef12345678901234567890abcdef123a2b',
    blockNumber: 62845146,
    from: 'TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t',
    to: 'TLa2f6VPqDgRE67v1736s7bJ8Ray5wYjU7',
    value: '75.25',
    token: 'USDT',
    status: 'failed',
    timestamp: '2024-01-15T10:15:00Z',
    gasUsed: '0',
    gasPrice: '420'
  }
]

export function Transactions() {
  const [transactions, setTransactions] = useState(mockTransactions)
  const [searchQuery, setSearchQuery] = useState('')
  const [statusFilter, setStatusFilter] = useState('all')
  const [tokenFilter, setTokenFilter] = useState('all')
  const [loading, setLoading] = useState(false)
  const [currentPage, setCurrentPage] = useState(1)
  const [totalPages, setTotalPages] = useState(10)

  const handleSearch = async () => {
    setLoading(true)
    // Simulate API call
    setTimeout(() => {
      setLoading(false)
    }, 1000)
  }

  const handleMultiAddressQuery = async () => {
    const addresses = searchQuery.split(',').map(addr => addr.trim()).filter(addr => addr)
    if (addresses.length === 0) return

    setLoading(true)
    try {
      // Simulate multi-address API call
      setTimeout(() => {
        setLoading(false)
      }, 1500)
    } catch (error) {
      console.error('Multi-address query failed:', error)
      setLoading(false)
    }
  }

  const copyToClipboard = (text) => {
    navigator.clipboard.writeText(text)
  }

  const formatAddress = (address) => {
    return `${address.slice(0, 8)}...${address.slice(-6)}`
  }

  const formatHash = (hash) => {
    return `${hash.slice(0, 10)}...${hash.slice(-8)}`
  }

  const formatTime = (timestamp) => {
    return new Date(timestamp).toLocaleString()
  }

  const getStatusIcon = (status) => {
    switch (status) {
      case 'success':
        return <CheckCircle className="w-4 h-4 text-green-500" />
      case 'pending':
        return <Clock className="w-4 h-4 text-yellow-500" />
      case 'failed':
        return <AlertTriangle className="w-4 h-4 text-red-500" />
      default:
        return null
    }
  }

  const getStatusBadge = (status) => {
    const variants = {
      success: 'default',
      pending: 'secondary',
      failed: 'destructive'
    }
    return (
      <Badge variant={variants[status]} className="capitalize">
        {status}
      </Badge>
    )
  }

  const filteredTransactions = transactions.filter(tx => {
    const matchesSearch = !searchQuery || 
      tx.hash.toLowerCase().includes(searchQuery.toLowerCase()) ||
      tx.from.toLowerCase().includes(searchQuery.toLowerCase()) ||
      tx.to.toLowerCase().includes(searchQuery.toLowerCase())
    
    const matchesStatus = statusFilter === 'all' || tx.status === statusFilter
    const matchesToken = tokenFilter === 'all' || tx.token === tokenFilter
    
    return matchesSearch && matchesStatus && matchesToken
  })

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Transactions</h1>
          <p className="text-gray-600">Search and analyze TRX/USDT transactions</p>
        </div>
        
        <Button onClick={() => {}} className="flex items-center space-x-2">
          <Download className="w-4 h-4" />
          <span>Export</span>
        </Button>
      </div>

      {/* Search and Filters */}
      <Card>
        <CardHeader>
          <CardTitle>Search Transactions</CardTitle>
          <CardDescription>
            Search by transaction hash, address, or use multi-address query (comma-separated)
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex flex-col lg:flex-row gap-4">
            <div className="flex-1">
              <div className="relative">
                <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
                <Input
                  type="text"
                  placeholder="Transaction hash, address, or multiple addresses (comma-separated)"
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="pl-10"
                />
              </div>
            </div>
            
            <div className="flex gap-2">
              <Select value={statusFilter} onValueChange={setStatusFilter}>
                <SelectTrigger className="w-32">
                  <SelectValue placeholder="Status" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Status</SelectItem>
                  <SelectItem value="success">Success</SelectItem>
                  <SelectItem value="pending">Pending</SelectItem>
                  <SelectItem value="failed">Failed</SelectItem>
                </SelectContent>
              </Select>
              
              <Select value={tokenFilter} onValueChange={setTokenFilter}>
                <SelectTrigger className="w-32">
                  <SelectValue placeholder="Token" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Tokens</SelectItem>
                  <SelectItem value="USDT">USDT</SelectItem>
                  <SelectItem value="TRX">TRX</SelectItem>
                </SelectContent>
              </Select>
              
              <Button onClick={handleSearch} disabled={loading}>
                {loading ? 'Searching...' : 'Search'}
              </Button>
              
              {searchQuery.includes(',') && (
                <Button variant="outline" onClick={handleMultiAddressQuery} disabled={loading}>
                  Multi-Address Query
                </Button>
              )}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Results */}
      <Card>
        <CardHeader>
          <CardTitle>Transaction Results</CardTitle>
          <CardDescription>
            Found {filteredTransactions.length} transactions
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="overflow-x-auto">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Status</TableHead>
                  <TableHead>Hash</TableHead>
                  <TableHead>Block</TableHead>
                  <TableHead>From</TableHead>
                  <TableHead>To</TableHead>
                  <TableHead>Amount</TableHead>
                  <TableHead>Time</TableHead>
                  <TableHead>Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {filteredTransactions.map((tx) => (
                  <TableRow key={tx.id}>
                    <TableCell>
                      <div className="flex items-center space-x-2">
                        {getStatusIcon(tx.status)}
                        {getStatusBadge(tx.status)}
                      </div>
                    </TableCell>
                    <TableCell>
                      <div className="flex items-center space-x-2">
                        <span className="font-mono text-sm">{formatHash(tx.hash)}</span>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => copyToClipboard(tx.hash)}
                        >
                          <Copy className="w-3 h-3" />
                        </Button>
                      </div>
                    </TableCell>
                    <TableCell>
                      <span className="font-mono text-sm">{tx.blockNumber.toLocaleString()}</span>
                    </TableCell>
                    <TableCell>
                      <div className="flex items-center space-x-2">
                        <span className="font-mono text-sm">{formatAddress(tx.from)}</span>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => copyToClipboard(tx.from)}
                        >
                          <Copy className="w-3 h-3" />
                        </Button>
                      </div>
                    </TableCell>
                    <TableCell>
                      <div className="flex items-center space-x-2">
                        <span className="font-mono text-sm">{formatAddress(tx.to)}</span>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => copyToClipboard(tx.to)}
                        >
                          <Copy className="w-3 h-3" />
                        </Button>
                      </div>
                    </TableCell>
                    <TableCell>
                      <div className="text-right">
                        <div className="font-semibold">{parseFloat(tx.value).toLocaleString()}</div>
                        <div className="text-xs text-gray-500">{tx.token}</div>
                      </div>
                    </TableCell>
                    <TableCell>
                      <span className="text-sm text-gray-600">{formatTime(tx.timestamp)}</span>
                    </TableCell>
                    <TableCell>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => window.open(`https://tronscan.org/#/transaction/${tx.hash}`, '_blank')}
                      >
                        <ExternalLink className="w-4 h-4" />
                      </Button>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </div>
          
          {/* Pagination */}
          <div className="flex items-center justify-between mt-4">
            <div className="text-sm text-gray-600">
              Showing {filteredTransactions.length} of {mockTransactions.length} transactions
            </div>
            <div className="flex items-center space-x-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => setCurrentPage(Math.max(1, currentPage - 1))}
                disabled={currentPage === 1}
              >
                Previous
              </Button>
              <span className="text-sm text-gray-600">
                Page {currentPage} of {totalPages}
              </span>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setCurrentPage(Math.min(totalPages, currentPage + 1))}
                disabled={currentPage === totalPages}
              >
                Next
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

