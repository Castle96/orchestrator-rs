import React, { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import {
  Box,
  Typography,
  Paper,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Chip,
  Button,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  Grid,
  Card,
  CardContent,
  LinearProgress,
  Avatar,
  IconButton,
  Alert,
  Snackbar,
  Tooltip,
} from '@mui/material'
import {
  Computer as ComputerIcon,
  Add as AddIcon,
  Refresh as RefreshIcon,
  Settings as SettingsIcon,
  CheckCircle as CheckCircleIcon,
  Error as ErrorIcon,
  Warning as WarningIcon,
} from '@mui/icons-material'
import { clusterApi, Node } from '../services/api'

export default function Cluster() {
  const queryClient = useQueryClient()
  const [joinDialogOpen, setJoinDialogOpen] = useState(false)
  const [joinForm, setJoinForm] = useState({
    cluster_name: '',
    node_address: '',
    node_port: 7946,
  })
  const [notification, setNotification] = useState({ open: false, message: '', severity: 'success' as 'success' | 'error' })

  const { data: nodes, isLoading: nodesLoading } = useQuery({
    queryKey: ['cluster-nodes'],
    queryFn: () => clusterApi.listNodes().then((res) => res.data.nodes),
    refetchInterval: 10000, // Refresh every 10 seconds
  })

  const { data: clusterStatus } = useQuery({
    queryKey: ['cluster-status'],
    queryFn: () => clusterApi.status().then((res) => res.data),
    refetchInterval: 10000,
  })

  const joinMutation = useMutation({
    mutationFn: (data: typeof joinForm) => clusterApi.join(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['cluster-nodes'] })
      queryClient.invalidateQueries({ queryKey: ['cluster-status'] })
      setJoinDialogOpen(false)
      setJoinForm({
        cluster_name: '',
        node_address: '',
        node_port: 7946,
      })
      setNotification({ open: true, message: 'Node joined cluster successfully', severity: 'success' })
    },
    onError: (error: Error) => {
      setNotification({ open: true, message: `Failed to join cluster: ${error.message}`, severity: 'error' })
    },
  })

  const getStatusColor = (status: string): 'success' | 'error' | 'warning' | 'info' | 'primary' | 'secondary' | 'default' => {
    switch (status) {
      case 'online':
        return 'success'
      case 'offline':
        return 'error'
      case 'joining':
      case 'leaving':
        return 'warning'
      default:
        return 'default'
    }
  }

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'online':
        return <CheckCircleIcon color="success" />
      case 'offline':
        return <ErrorIcon color="error" />
      case 'joining':
      case 'leaving':
        return <WarningIcon color="warning" />
      default:
        return <WarningIcon color="disabled" />
    }
  }

  const formatBytes = (bytes: number) => {
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
    if (bytes === 0) return '0 B'
    const i = Math.floor(Math.log(bytes) / Math.log(1024))
    return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + ' ' + sizes[i]
  }

  const calculateUsagePercentage = (used: number, total: number) => {
    if (total === 0) return 0
    return Math.round((used / total) * 100)
  }

  return (
    <Box>
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 3 }}>
        <Typography variant="h4">Cluster Management</Typography>
        <Button
          variant="contained"
          startIcon={<AddIcon />}
          onClick={() => setJoinDialogOpen(true)}
        >
          Join Cluster
        </Button>
      </Box>

      {/* Cluster Overview */}
      <Grid container spacing={3} sx={{ mb: 3 }}>
        <Grid item xs={12} md={3}>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', alignItems: 'center' }}>
                <Avatar sx={{ bgcolor: 'info.main', mr: 2 }}>
                  <ComputerIcon />
                </Avatar>
                <Box>
                  <Typography color="textSecondary" variant="body2">
                    Cluster Nodes
                  </Typography>
                  <Typography variant="h5">
                    {clusterStatus?.cluster?.node_count || 0}
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
                <Avatar sx={{ bgcolor: getStatusColor(clusterStatus?.status || 'unknown'), mr: 2 }}>
                  {getStatusIcon(clusterStatus?.status || 'unknown')}
                </Avatar>
                <Box>
                  <Typography color="textSecondary" variant="body2">
                    Cluster Status
                  </Typography>
                  <Typography variant="h5">
                    {clusterStatus?.status || 'Unknown'}
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
                  <CheckCircleIcon />
                </Avatar>
                <Box>
                  <Typography color="textSecondary" variant="body2">
                    Online Nodes
                  </Typography>
                  <Typography variant="h5">
                    {nodes?.filter(n => n.status === 'online').length || 0}
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
                <Avatar sx={{ bgcolor: 'error.main', mr: 2 }}>
                  <ErrorIcon />
                </Avatar>
                <Box>
                  <Typography color="textSecondary" variant="body2">
                    Offline Nodes
                  </Typography>
                  <Typography variant="h5">
                    {nodes?.filter(n => n.status === 'offline').length || 0}
                  </Typography>
                </Box>
              </Box>
            </CardContent>
          </Card>
        </Grid>
      </Grid>

      {/* Cluster Information */}
      <Paper sx={{ p: 3, mb: 3 }}>
        <Typography variant="h6" gutterBottom>
          Cluster Information
        </Typography>
        <Grid container spacing={2}>
          <Grid item xs={12} md={4}>
            <Typography variant="body2" color="textSecondary">
              Cluster Name
            </Typography>
            <Typography variant="body1" fontWeight="medium">
              {clusterStatus?.cluster?.name || 'N/A'}
            </Typography>
          </Grid>
          <Grid item xs={12} md={4}>
            <Typography variant="body2" color="textSecondary">
              Cluster ID
            </Typography>
            <Typography variant="body1" fontWeight="medium" sx={{ fontFamily: 'monospace' }}>
              {clusterStatus?.cluster?.id?.slice(0, 8) || 'N/A'}...
            </Typography>
          </Grid>
          <Grid item xs={12} md={4}>
            <Typography variant="body2" color="textSecondary">
              Last Updated
            </Typography>
            <Typography variant="body1" fontWeight="medium">
              {new Date().toLocaleString()}
            </Typography>
          </Grid>
        </Grid>
      </Paper>

      {/* Nodes Table */}
      <Paper sx={{ p: 3 }}>
        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
          <Typography variant="h6">Cluster Nodes</Typography>
          <Button
            variant="outlined"
            startIcon={<RefreshIcon />}
            onClick={() => {
              queryClient.invalidateQueries({ queryKey: ['cluster-nodes'] })
              queryClient.invalidateQueries({ queryKey: ['cluster-status'] })
            }}
          >
            Refresh
          </Button>
        </Box>
        
        <TableContainer>
          <Table>
            <TableHead>
              <TableRow>
                <TableCell>Node</TableCell>
                <TableCell>Address</TableCell>
                <TableCell>Status</TableCell>
                <TableCell>CPU</TableCell>
                <TableCell>Memory</TableCell>
                <TableCell>Disk</TableCell>
                <TableCell>Joined</TableCell>
                <TableCell>Actions</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {nodesLoading ? (
                <TableRow>
                  <TableCell colSpan={8} align="center">
                    <LinearProgress />
                  </TableCell>
                </TableRow>
              ) : nodes?.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={8} align="center" sx={{ py: 4 }}>
                    <Typography color="textSecondary">
                      No nodes in cluster. Join a node to get started.
                    </Typography>
                  </TableCell>
                </TableRow>
              ) : (
                nodes?.map((node) => (
                  <TableRow key={node.id} hover>
                    <TableCell>
                      <Box sx={{ display: 'flex', alignItems: 'center' }}>
                        {getStatusIcon(node.status)}
                        <Box sx={{ ml: 2 }}>
                          <Typography variant="body2" fontWeight="medium">
                            {node.name}
                          </Typography>
                          <Typography variant="caption" color="textSecondary">
                            ID: {node.id.slice(0, 8)}...
                          </Typography>
                        </Box>
                      </Box>
                    </TableCell>
                    <TableCell>
                      <Typography variant="body2">
                        {node.address}:{node.port}
                      </Typography>
                    </TableCell>
                    <TableCell>
                      <Chip
                        label={node.status}
                        color={getStatusColor(node.status)}
                        size="small"
                        sx={{ fontWeight: 'medium' }}
                      />
                    </TableCell>
                    <TableCell>
                      <Box>
                        <Typography variant="body2" fontWeight="medium">
                          {node.resources.cpu_cores} cores
                        </Typography>
                        <LinearProgress
                          variant="determinate"
                          value={calculateUsagePercentage(node.resources.memory_used, node.resources.memory_total)}
                          sx={{ mt: 0.5, height: 4 }}
                        />
                      </Box>
                    </TableCell>
                    <TableCell>
                      <Box>
                        <Typography variant="body2" fontWeight="medium">
                          {formatBytes(node.resources.memory_used)} / {formatBytes(node.resources.memory_total)}
                        </Typography>
                        <Typography variant="caption" color="textSecondary">
                          {calculateUsagePercentage(node.resources.memory_used, node.resources.memory_total)}% used
                        </Typography>
                      </Box>
                    </TableCell>
                    <TableCell>
                      <Box>
                        <Typography variant="body2" fontWeight="medium">
                          {formatBytes(node.resources.disk_used)} / {formatBytes(node.resources.disk_total)}
                        </Typography>
                        <Typography variant="caption" color="textSecondary">
                          {calculateUsagePercentage(node.resources.disk_used, node.resources.disk_total)}% used
                        </Typography>
                      </Box>
                    </TableCell>
                    <TableCell>
                      <Typography variant="body2">
                        {new Date(node.joined_at).toLocaleDateString()}
                      </Typography>
                    </TableCell>
                    <TableCell>
                      <Tooltip title="Node Settings">
                        <IconButton size="small">
                          <SettingsIcon />
                        </IconButton>
                      </Tooltip>
                    </TableCell>
                  </TableRow>
                ))
              )}
            </TableBody>
          </Table>
        </TableContainer>
      </Paper>

      {/* Join Cluster Dialog */}
      <Dialog open={joinDialogOpen} onClose={() => setJoinDialogOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Join Cluster</DialogTitle>
        <DialogContent>
          <Alert severity="info" sx={{ mb: 2 }}>
            Enter the details of an existing cluster to join this node to it.
          </Alert>
          <TextField
            autoFocus
            margin="dense"
            label="Cluster Name"
            fullWidth
            variant="outlined"
            value={joinForm.cluster_name}
            onChange={(e) => setJoinForm({ ...joinForm, cluster_name: e.target.value })}
            sx={{ mb: 2 }}
          />
          <TextField
            margin="dense"
            label="Node Address"
            placeholder="192.168.1.100"
            fullWidth
            variant="outlined"
            value={joinForm.node_address}
            onChange={(e) => setJoinForm({ ...joinForm, node_address: e.target.value })}
            sx={{ mb: 2 }}
          />
          <TextField
            margin="dense"
            label="Node Port"
            type="number"
            fullWidth
            variant="outlined"
            value={joinForm.node_port}
            onChange={(e) => setJoinForm({ ...joinForm, node_port: parseInt(e.target.value) })}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setJoinDialogOpen(false)}>Cancel</Button>
          <Button
            onClick={() => joinMutation.mutate(joinForm)}
            variant="contained"
            disabled={!joinForm.cluster_name || !joinForm.node_address || joinMutation.isPending}
          >
            Join Cluster
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
