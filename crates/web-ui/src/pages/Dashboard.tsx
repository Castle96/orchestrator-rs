import React from 'react'
import { useQuery } from '@tanstack/react-query'
import {
  Grid,
  Paper,
  Typography,
  Box,
  Card,
  CardContent,
  LinearProgress,
  Chip,
  List,
  ListItem,
  ListItemText,
  ListItemIcon,
  Divider,
  Avatar,
  Button,
} from '@mui/material'
import {
  Computer as ComputerIcon,
  Memory as MemoryIcon,
  Storage as StorageIcon,
  Router as NetworkIcon,
  CheckCircle as CheckCircleIcon,
  Error as ErrorIcon,
  Warning as WarningIcon,
  PlayArrow as PlayArrowIcon,
  Stop as StopIcon,
  Refresh as RefreshIcon,
} from '@mui/icons-material'
import { containerApi, clusterApi } from '../services/api'

interface SystemStats {
  cpu_usage: number
  memory_usage: number
  memory_total: number
  memory_used: number
  disk_usage: number
  disk_total: number
  disk_used: number
  network_rx: number
  network_tx: number
}

export default function Dashboard() {
  const { data: containers, isLoading: containersLoading } = useQuery({
    queryKey: ['containers'],
    queryFn: () => containerApi.list().then((res) => res.data.containers),
    refetchInterval: 5000, // Refresh every 5 seconds
  })

  const { data: clusterStatus } = useQuery({
    queryKey: ['cluster-status'],
    queryFn: () => clusterApi.status().then((res) => res.data),
    refetchInterval: 10000, // Refresh every 10 seconds
  })

  // Mock system stats for now - would come from a system metrics API
  const { data: systemStats } = useQuery({
    queryKey: ['system-stats'],
    queryFn: (): SystemStats => ({
      cpu_usage: Math.random() * 100,
      memory_usage: Math.random() * 100,
      memory_total: 8 * 1024 * 1024 * 1024, // 8GB
      memory_used: Math.random() * 8 * 1024 * 1024 * 1024,
      disk_usage: Math.random() * 100,
      disk_total: 500 * 1024 * 1024 * 1024, // 500GB
      disk_used: Math.random() * 500 * 1024 * 1024 * 1024,
      network_rx: Math.random() * 1024 * 1024, // Random MB
      network_tx: Math.random() * 1024 * 1024,
    }),
    refetchInterval: 3000,
  })

  const runningContainers = containers?.filter((c) => c.status === 'running').length || 0
  const stoppedContainers = containers?.filter((c) => c.status === 'stopped').length || 0
  

  const formatBytes = (bytes: number) => {
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
    if (bytes === 0) return '0 B'
    const i = Math.floor(Math.log(bytes) / Math.log(1024))
    return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + ' ' + sizes[i]
  }

  const formatNetworkSpeed = (bytes: number) => {
    const mbps = bytes / (1024 * 1024)
    return mbps.toFixed(1) + ' MB/s'
  }

  const getClusterHealthColor = (status: string): 'success' | 'error' | 'warning' | 'info' | 'primary' | 'secondary' | 'default' => {
    if (status === 'healthy') return 'success'
    if (status === 'unhealthy') return 'error'
    return 'warning'
  }

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'healthy':
        return <CheckCircleIcon color="success" />
      case 'unhealthy':
        return <ErrorIcon color="error" />
      default:
        return <WarningIcon color="warning" />
    }
  }

  return (
    <Box>
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 3 }}>
        <Typography variant="h4">Dashboard</Typography>
        <Button variant="outlined" startIcon={<RefreshIcon />} onClick={() => window.location.reload()}>
          Refresh
        </Button>
      </Box>

      {/* System Overview Cards */}
      <Grid container spacing={3} sx={{ mb: 3 }}>
        <Grid item xs={12} sm={6} md={3}>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                <Avatar sx={{ bgcolor: 'primary.main', mr: 2 }}>
                  <ComputerIcon />
                </Avatar>
                <Box>
                  <Typography color="textSecondary" gutterBottom variant="body2">
                    Total Containers
                  </Typography>
                  <Typography variant="h4">
                    {containers?.length || 0}
                  </Typography>
                </Box>
              </Box>
            </CardContent>
          </Card>
        </Grid>
        <Grid item xs={12} sm={6} md={3}>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                <Avatar sx={{ bgcolor: 'success.main', mr: 2 }}>
                  <PlayArrowIcon />
                </Avatar>
                <Box>
                  <Typography color="textSecondary" gutterBottom variant="body2">
                    Running
                  </Typography>
                  <Typography variant="h4" color="success.main">
                    {runningContainers}
                  </Typography>
                </Box>
              </Box>
            </CardContent>
          </Card>
        </Grid>
        <Grid item xs={12} sm={6} md={3}>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                <Avatar sx={{ bgcolor: 'text.secondary', mr: 2 }}>
                  <StopIcon />
                </Avatar>
                <Box>
                  <Typography color="textSecondary" gutterBottom variant="body2">
                    Stopped
                  </Typography>
                  <Typography variant="h4" color="textSecondary">
                    {stoppedContainers}
                  </Typography>
                </Box>
              </Box>
            </CardContent>
          </Card>
        </Grid>
        <Grid item xs={12} sm={6} md={3}>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                <Avatar sx={{ bgcolor: 'info.main', mr: 2 }}>
                  <NetworkIcon />
                </Avatar>
                <Box>
                  <Typography color="textSecondary" gutterBottom variant="body2">
                    Cluster Nodes
                  </Typography>
                  <Typography variant="h4">
                    {clusterStatus?.cluster?.node_count || 0}
                  </Typography>
                </Box>
              </Box>
            </CardContent>
          </Card>
        </Grid>
      </Grid>

      {/* System Metrics and Cluster Status */}
      <Grid container spacing={3}>
        {/* System Resources */}
        <Grid item xs={12} md={8}>
          <Paper sx={{ p: 3, height: '100%' }}>
            <Typography variant="h6" gutterBottom>
              System Resources
            </Typography>
            <Grid container spacing={3}>
              <Grid item xs={12}>
                <Box sx={{ mb: 2 }}>
                  <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 1 }}>
                    <Box sx={{ display: 'flex', alignItems: 'center' }}>
                      <ComputerIcon sx={{ mr: 1, fontSize: 20, color: 'text.secondary' }} />
                      <Typography variant="body2">CPU Usage</Typography>
                    </Box>
                    <Typography variant="body2" fontWeight="medium">
                      {systemStats?.cpu_usage.toFixed(1)}%
                    </Typography>
                  </Box>
                  <LinearProgress 
                    variant="determinate" 
                    value={systemStats?.cpu_usage || 0} 
                    color={systemStats?.cpu_usage && systemStats.cpu_usage > 80 ? 'error' : 'primary'}
                  />
                </Box>
              </Grid>
              
              <Grid item xs={12}>
                <Box sx={{ mb: 2 }}>
                  <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 1 }}>
                    <Box sx={{ display: 'flex', alignItems: 'center' }}>
                      <MemoryIcon sx={{ mr: 1, fontSize: 20, color: 'text.secondary' }} />
                      <Typography variant="body2">Memory Usage</Typography>
                    </Box>
                    <Typography variant="body2" fontWeight="medium">
                      {formatBytes(systemStats?.memory_used || 0)} / {formatBytes(systemStats?.memory_total || 0)}
                    </Typography>
                  </Box>
                  <LinearProgress 
                    variant="determinate" 
                    value={systemStats?.memory_usage || 0} 
                    color={systemStats?.memory_usage && systemStats.memory_usage > 80 ? 'error' : 'primary'}
                  />
                </Box>
              </Grid>
              
              <Grid item xs={12}>
                <Box sx={{ mb: 2 }}>
                  <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 1 }}>
                    <Box sx={{ display: 'flex', alignItems: 'center' }}>
                      <StorageIcon sx={{ mr: 1, fontSize: 20, color: 'text.secondary' }} />
                      <Typography variant="body2">Disk Usage</Typography>
                    </Box>
                    <Typography variant="body2" fontWeight="medium">
                      {formatBytes(systemStats?.disk_used || 0)} / {formatBytes(systemStats?.disk_total || 0)}
                    </Typography>
                  </Box>
                  <LinearProgress 
                    variant="determinate" 
                    value={systemStats?.disk_usage || 0} 
                    color={systemStats?.disk_usage && systemStats.disk_usage > 80 ? 'error' : 'primary'}
                  />
                </Box>
              </Grid>
            </Grid>
          </Paper>
        </Grid>

        {/* Cluster Status */}
        <Grid item xs={12} md={4}>
          <Paper sx={{ p: 3, height: '100%' }}>
            <Typography variant="h6" gutterBottom>
              Cluster Status
            </Typography>
            <Box sx={{ mb: 2 }}>
              <Box sx={{ display: 'flex', alignItems: 'center', mb: 1 }}>
                {getStatusIcon(clusterStatus?.status || 'unknown')}
                <Typography variant="body2" sx={{ ml: 1 }}>
                  Status: {clusterStatus?.status || 'Unknown'}
                </Typography>
              </Box>
              <Chip 
                label={clusterStatus?.status || 'Unknown'} 
                color={getClusterHealthColor(clusterStatus?.status || 'unknown')}
                size="small"
                sx={{ ml: 3 }}
              />
            </Box>
            <Divider sx={{ my: 2 }} />
            <List dense>
              <ListItem>
                <ListItemIcon>
                  <ComputerIcon fontSize="small" />
                </ListItemIcon>
                <ListItemText 
                  primary="Nodes" 
                  secondary={clusterStatus?.cluster?.node_count || 0} 
                />
              </ListItem>
              <ListItem>
                <ListItemIcon>
                  <NetworkIcon fontSize="small" />
                </ListItemIcon>
                <ListItemText 
                  primary="Network RX" 
                  secondary={formatNetworkSpeed(systemStats?.network_rx || 0)} 
                />
              </ListItem>
              <ListItem>
                <ListItemIcon>
                  <NetworkIcon fontSize="small" />
                </ListItemIcon>
                <ListItemText 
                  primary="Network TX" 
                  secondary={formatNetworkSpeed(systemStats?.network_tx || 0)} 
                />
              </ListItem>
            </List>
          </Paper>
        </Grid>

        {/* Recent Containers */}
        <Grid item xs={12} md={6}>
          <Paper sx={{ p: 3, height: '100%' }}>
            <Typography variant="h6" gutterBottom>
              Recent Containers
            </Typography>
            <List>
              {containersLoading ? (
                <LinearProgress />
              ) : containers?.length === 0 ? (
                <Typography color="textSecondary">No containers found</Typography>
              ) : (
                containers?.slice(0, 5).map((container) => (
                  <React.Fragment key={container.id}>
                    <ListItem>
                      <ListItemIcon>
                        <ComputerIcon 
                          fontSize="small" 
                          color={container.status === 'running' ? 'success' : 'disabled'}
                        />
                      </ListItemIcon>
                      <ListItemText 
                        primary={container.name}
                        secondary={
                          <Box sx={{ display: 'flex', alignItems: 'center' }}>
                            <Chip
                              label={container.status}
                              color={container.status === 'running' ? 'success' : 'default'}
                              size="small"
                              sx={{ mr: 1 }}
                            />
                            <Typography variant="caption" color="textSecondary">
                              {new Date(container.created_at).toLocaleDateString()}
                            </Typography>
                          </Box>
                        }
                      />
                    </ListItem>
                    {container !== containers[containers.length - 1] && <Divider />}
                  </React.Fragment>
                ))
              )}
            </List>
          </Paper>
        </Grid>

        {/* Quick Actions */}
        <Grid item xs={12} md={6}>
          <Paper sx={{ p: 3, height: '100%' }}>
            <Typography variant="h6" gutterBottom>
              Quick Actions
            </Typography>
            <Grid container spacing={2}>
              <Grid item xs={6}>
                <Button 
                  variant="contained" 
                  fullWidth 
                  href="/containers"
                  startIcon={<ComputerIcon />}
                >
                  Manage Containers
                </Button>
              </Grid>
              <Grid item xs={6}>
                <Button 
                  variant="outlined" 
                  fullWidth 
                  href="/cluster"
                  startIcon={<NetworkIcon />}
                >
                  Cluster Status
                </Button>
              </Grid>
              <Grid item xs={6}>
                <Button 
                  variant="outlined" 
                  fullWidth 
                  href="/storage"
                  startIcon={<StorageIcon />}
                >
                  Storage Pools
                </Button>
              </Grid>
              <Grid item xs={6}>
                <Button 
                  variant="outlined" 
                  fullWidth 
                  href="/network"
                  startIcon={<NetworkIcon />}
                >
                  Network Config
                </Button>
              </Grid>
            </Grid>
          </Paper>
        </Grid>
      </Grid>
    </Box>
  )
}
