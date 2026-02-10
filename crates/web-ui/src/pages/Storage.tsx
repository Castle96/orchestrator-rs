import React, { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import {
  Box,
  Typography,
  Button,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  Select,
  MenuItem,
  FormControl,
  InputLabel,
  Grid,
  Card,
  CardContent,
  LinearProgress,
  Avatar,
  Chip,
  IconButton,
  Tooltip,
  Alert,
  Snackbar,
} from '@mui/material'
import {
  Storage as StorageIcon,
  Add as AddIcon,
  Delete as DeleteIcon,
  Edit as EditIcon,
  Refresh as RefreshIcon,
  Computer as ComputerIcon,
  CloudQueue as CloudIcon,
  Folder as FolderIcon,
  SdStorage as SdStorageIcon,
} from '@mui/icons-material'
import { storageApi, StoragePool } from '../services/api'

export default function Storage() {
  const queryClient = useQueryClient()
  const [createDialogOpen, setCreateDialogOpen] = useState(false)
  
  const [notification, setNotification] = useState({ open: false, message: '', severity: 'success' as 'success' | 'error' })
  
  const [newPool, setNewPool] = useState({
    name: '',
    storage_type: 'local' as 'local' | 'nfs' | 'cifs',
    path: '',
  })

  const { data: pools, isLoading } = useQuery({
    queryKey: ['storage-pools'],
    queryFn: () => storageApi.listPools().then((res) => res.data.pools),
    refetchInterval: 10000, // Refresh every 10 seconds
  })

  const createMutation = useMutation({
    mutationFn: (data: typeof newPool) => storageApi.createPool(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['storage-pools'] })
      setCreateDialogOpen(false)
      setNewPool({ name: '', storage_type: 'local', path: '' })
      setNotification({ open: true, message: 'Storage pool created successfully', severity: 'success' })
    },
    onError: (error: Error) => {
      setNotification({ open: true, message: `Failed to create storage pool: ${error.message}`, severity: 'error' })
    },
  })

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i]
  }

  const getStorageIcon = (type: string) => {
    switch (type) {
      case 'local':
        return <SdStorageIcon />
      case 'nfs':
        return <CloudIcon />
      case 'cifs':
        return <FolderIcon />
      default:
        return <StorageIcon />
    }
  }

  const getStorageColor = (type: string): 'primary' | 'success' | 'error' | 'warning' | 'info' | 'secondary' | 'default' => {
    switch (type) {
      case 'local':
        return 'primary'
      case 'nfs':
        return 'success'
      case 'cifs':
        return 'warning'
      default:
        return 'default'
    }
  }

  const getUsagePercentage = (used: number, total: number) => {
    if (total === 0) return 0
    return Math.round((used / total) * 100)
  }

  const getUsageColor = (percentage: number): 'error' | 'warning' | 'success' | 'info' | 'primary' | 'secondary' | 'default' => {
    if (percentage >= 90) return 'error'
    if (percentage >= 75) return 'warning'
    return 'success'
  }

  const totalSize = pools?.reduce((sum, pool) => sum + pool.total_size, 0) || 0
  const totalUsed = pools?.reduce((sum, pool) => sum + pool.used_size, 0) || 0
  const totalAvailable = pools?.reduce((sum, pool) => sum + pool.available_size, 0) || 0

  return (
    <Box>
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 3 }}>
        <Typography variant="h4">Storage Management</Typography>
        <Box>
          <Button
            variant="outlined"
            startIcon={<RefreshIcon />}
            onClick={() => queryClient.invalidateQueries({ queryKey: ['storage-pools'] })}
            sx={{ mr: 2 }}
          >
            Refresh
          </Button>
          <Button
            variant="contained"
            startIcon={<AddIcon />}
            onClick={() => setCreateDialogOpen(true)}
          >
            Create Storage Pool
          </Button>
        </Box>
      </Box>

      {/* Storage Overview */}
      <Grid container spacing={3} sx={{ mb: 3 }}>
        <Grid item xs={12} md={3}>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', alignItems: 'center' }}>
                <Avatar sx={{ bgcolor: 'primary.main', mr: 2 }}>
                  <StorageIcon />
                </Avatar>
                <Box>
                  <Typography color="textSecondary" variant="body2">
                    Total Storage
                  </Typography>
                  <Typography variant="h5">
                    {formatBytes(totalSize)}
                  </Typography>
                </Box>
              </Box>
            </CardContent>
          </Card>
        </Grid>
        <Grid item xs={12} md={3}>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', alignItems: 'center' }}>
                <Avatar sx={{ bgcolor: 'success.main', mr: 2 }}>
                  <SdStorageIcon />
                </Avatar>
                <Box>
                  <Typography color="textSecondary" variant="body2">
                    Used Space
                  </Typography>
                  <Typography variant="h5">
                    {formatBytes(totalUsed)}
                  </Typography>
                </Box>
              </Box>
            </CardContent>
          </Card>
        </Grid>
        <Grid item xs={12} md={3}>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', alignItems: 'center' }}>
                <Avatar sx={{ bgcolor: 'info.main', mr: 2 }}>
                  <FolderIcon />
                </Avatar>
                <Box>
                  <Typography color="textSecondary" variant="body2">
                    Available Space
                  </Typography>
                  <Typography variant="h5">
                    {formatBytes(totalAvailable)}
                  </Typography>
                </Box>
              </Box>
            </CardContent>
          </Card>
        </Grid>
        <Grid item xs={12} md={3}>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', alignItems: 'center' }}>
                <Avatar sx={{ bgcolor: 'warning.main', mr: 2 }}>
                  <ComputerIcon />
                </Avatar>
                <Box>
                  <Typography color="textSecondary" variant="body2">
                    Storage Pools
                  </Typography>
                  <Typography variant="h5">
                    {pools?.length || 0}
                  </Typography>
                </Box>
              </Box>
            </CardContent>
          </Card>
        </Grid>
      </Grid>

      {/* Storage Pools Table */}
      <Paper sx={{ p: 3 }}>
        <Typography variant="h6" gutterBottom>
          Storage Pools
        </Typography>
        <TableContainer>
          <Table>
            <TableHead>
              <TableRow>
                <TableCell>Pool Name</TableCell>
                <TableCell>Type</TableCell>
                <TableCell>Path</TableCell>
                <TableCell>Total Size</TableCell>
                <TableCell>Used Space</TableCell>
                <TableCell>Available</TableCell>
                <TableCell>Usage</TableCell>
                <TableCell>Created</TableCell>
                <TableCell>Actions</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {isLoading ? (
                <TableRow>
                  <TableCell colSpan={9} align="center">
                    <LinearProgress />
                  </TableCell>
                </TableRow>
              ) : pools?.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={9} align="center" sx={{ py: 4 }}>
                    <Typography color="textSecondary">
                      No storage pools found. Create your first storage pool to get started.
                    </Typography>
                  </TableCell>
                </TableRow>
              ) : (
                pools?.map((pool) => {
                  const usagePercentage = getUsagePercentage(pool.used_size, pool.total_size)
                  return (
                    <TableRow key={pool.id} hover>
                      <TableCell>
                        <Box sx={{ display: 'flex', alignItems: 'center' }}>
                          <Avatar sx={{ bgcolor: getStorageColor(pool.storage_type), mr: 2, width: 32, height: 32 }}>
                            {getStorageIcon(pool.storage_type)}
                          </Avatar>
                          <Box>
                            <Typography variant="body2" fontWeight="medium">
                              {pool.name}
                            </Typography>
                            <Typography variant="caption" color="textSecondary">
                              ID: {pool.id.slice(0, 8)}...
                            </Typography>
                          </Box>
                        </Box>
                      </TableCell>
                      <TableCell>
                        <Chip
                          label={pool.storage_type.toUpperCase()}
                          color={getStorageColor(pool.storage_type)}
                          size="small"
                          sx={{ fontWeight: 'medium' }}
                        />
                      </TableCell>
                      <TableCell>
                        <Typography variant="body2" sx={{ fontFamily: 'monospace', fontSize: '0.75rem' }}>
                          {pool.path}
                        </Typography>
                      </TableCell>
                      <TableCell>
                        <Typography variant="body2" fontWeight="medium">
                          {formatBytes(pool.total_size)}
                        </Typography>
                      </TableCell>
                      <TableCell>
                        <Box>
                          <Typography variant="body2" fontWeight="medium">
                            {formatBytes(pool.used_size)}
                          </Typography>
                          <LinearProgress
                            variant="determinate"
                            value={usagePercentage}
                            color={getUsageColor(usagePercentage)}
                            sx={{ mt: 0.5, height: 4 }}
                          />
                        </Box>
                      </TableCell>
                      <TableCell>
                        <Typography variant="body2" fontWeight="medium">
                          {formatBytes(pool.available_size)}
                        </Typography>
                      </TableCell>
                      <TableCell>
                        <Box>
                          <Typography variant="body2" fontWeight="medium">
                            {usagePercentage}%
                          </Typography>
                          <LinearProgress
                            variant="determinate"
                            value={usagePercentage}
                            color={getUsageColor(usagePercentage)}
                            sx={{ mt: 0.5, height: 4 }}
                          />
                        </Box>
                      </TableCell>
                      <TableCell>
                        <Typography variant="body2">
                          {new Date(pool.created_at).toLocaleDateString()}
                        </Typography>
                      </TableCell>
                      <TableCell>
                        <Tooltip title="Edit Pool">
                          <IconButton
                            size="small"
                            onClick={() => setSelectedPool(pool)}
                          >
                            <EditIcon />
                          </IconButton>
                        </Tooltip>
                        <Tooltip title="Delete Pool">
                          <IconButton size="small" color="error">
                            <DeleteIcon />
                          </IconButton>
                        </Tooltip>
                      </TableCell>
                    </TableRow>
                  )
                })
              )}
            </TableBody>
          </Table>
        </TableContainer>
      </Paper>

      {/* Create Storage Pool Dialog */}
      <Dialog open={createDialogOpen} onClose={() => setCreateDialogOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Create Storage Pool</DialogTitle>
        <DialogContent>
          <Alert severity="info" sx={{ mb: 2 }}>
            Storage pools provide the underlying storage for container images and volumes.
          </Alert>
          <TextField
            autoFocus
            margin="dense"
            label="Pool Name"
            fullWidth
            variant="outlined"
            value={newPool.name}
            onChange={(e) => setNewPool({ ...newPool, name: e.target.value })}
            sx={{ mb: 2 }}
            placeholder="e.g., local-ssd, nfs-storage"
          />
          <FormControl fullWidth sx={{ mb: 2 }}>
            <InputLabel>Storage Type</InputLabel>
            <Select
              value={newPool.storage_type}
              label="Storage Type"
              onChange={(e) =>
                setNewPool({
                  ...newPool,
                  storage_type: e.target.value as 'local' | 'nfs' | 'cifs',
                })
              }
            >
              <MenuItem value="local">
                <Box sx={{ display: 'flex', alignItems: 'center' }}>
                  <SdStorageIcon sx={{ mr: 1 }} />
                  Local Storage
                </Box>
              </MenuItem>
              <MenuItem value="nfs">
                <Box sx={{ display: 'flex', alignItems: 'center' }}>
                  <CloudIcon sx={{ mr: 1 }} />
                  NFS Network Storage
                </Box>
              </MenuItem>
              <MenuItem value="cifs">
                <Box sx={{ display: 'flex', alignItems: 'center' }}>
                  <FolderIcon sx={{ mr: 1 }} />
                  CIFS/Samba Share
                </Box>
              </MenuItem>
            </Select>
          </FormControl>
          <TextField
            margin="dense"
            label={
              newPool.storage_type === 'local'
                ? 'Local Path'
                : newPool.storage_type === 'nfs'
                ? 'NFS Server:Path'
                : 'CIFS Share Path'
            }
            fullWidth
            variant="outlined"
            value={newPool.path}
            onChange={(e) => setNewPool({ ...newPool, path: e.target.value })}
            placeholder={
              newPool.storage_type === 'local'
                ? '/var/lib/arm-hypervisor/storage'
                : newPool.storage_type === 'nfs'
                ? '192.168.1.100:/exports/storage'
                : '//server/share'
            }
            helperText={
              newPool.storage_type === 'local'
                ? 'Local directory path for storage'
                : newPool.storage_type === 'nfs'
                ? 'NFS export in format: server:/path'
                : 'SMB/CIFS share path'
            }
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setCreateDialogOpen(false)}>Cancel</Button>
          <Button
            onClick={() => createMutation.mutate(newPool)}
            variant="contained"
            disabled={!newPool.name || !newPool.path || createMutation.isPending}
          >
            Create Pool
          </Button>
        </DialogActions>
      </Dialog>

      {/* Notification Snackbar */}
      <Snackbar
        open={notification.open}
        autoHideDuration={6000}
        onClose={() => setNotification({ ...notification, open: false })}
      >
        <Alert
          onClose={() => setNotification({ ...notification, open: false })}
          severity={notification.severity}
          sx={{ width: '100%' }}
        >
          {notification.message}
        </Alert>
      </Snackbar>
    </Box>
  )
}
