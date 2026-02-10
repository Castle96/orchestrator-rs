import React, { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import {
  Box,
  Typography,
  Button,
  Paper,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  Checkbox,
  FormControlLabel,
  CircularProgress,
  Grid,
  Card,
  CardContent,
  Avatar,
  Chip,
  IconButton,
  Alert,
  Snackbar,
  List,
  ListItem,
  ListItemText,
  ListItemIcon,
  Divider,
  LinearProgress,
} from '@mui/material'
import {
  Network as NetworkIcon,
  Add as AddIcon,
  Delete as DeleteIcon,
  Edit as EditIcon,
  Refresh as RefreshIcon,
  Settings as SettingsIcon,
  Router as RouterIcon,
  Lan as LanIcon,
  Wifi as WifiIcon,
  Ethernet as EthernetIcon,
} from '@mui/icons-material'
import { networkApi, Bridge } from '../services/api'

interface NetworkInterface {
  name: string
  type: 'ethernet' | 'wifi' | 'bridge'
  status: 'up' | 'down'
  ip_address?: string
  mac_address?: string
  speed?: number
  mtu?: number
}

export default function Network() {
  const queryClient = useQueryClient()
  const [createDialogOpen, setCreateDialogOpen] = useState(false)
  
  const [notification, setNotification] = useState({ open: false, message: '', severity: 'success' as 'success' | 'error' })
  
  const [newBridge, setNewBridge] = useState({
    name: '',
    ip_address: '',
    stp_enabled: false,
  })

  const { data: bridges, isLoading: bridgesLoading } = useQuery({
    queryKey: ['bridges'],
    queryFn: () => networkApi.listBridges().then((res) => res.data.bridges || []),
    refetchInterval: 10000,
  })

  const { data: interfaces, isLoading: interfacesLoading } = useQuery({
    queryKey: ['interfaces'],
    queryFn: () => networkApi.listInterfaces().then((res) => res.data.interfaces || []),
    refetchInterval: 10000,
  })

  const createMutation = useMutation({
    mutationFn: (data: { name: string; ip_address?: string; stp_enabled: boolean }) =>
      networkApi.createBridge(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['bridges'] })
      setCreateDialogOpen(false)
      setNewBridge({ name: '', ip_address: '', stp_enabled: false })
      setNotification({ open: true, message: 'Bridge created successfully', severity: 'success' })
    },
    onError: (error: Error) => {
      setNotification({ open: true, message: `Failed to create bridge: ${error.message}`, severity: 'error' })
    },
  })

  const getStatusColor = (status: string): 'success' | 'default' | 'primary' | 'secondary' | 'error' | 'info' | 'warning' => {
    return status === 'up' ? 'success' : 'default'
  }

  const getInterfaceIcon = (type: string) => {
    switch (type) {
      case 'ethernet':
        return <EthernetIcon />
      case 'wifi':
        return <WifiIcon />
      case 'bridge':
        return <RouterIcon />
      default:
        return <NetworkIcon />
    }
  }

  const getNetworkStats = () => {
    const totalBridges = bridges?.length || 0
    const activeBridges = bridges?.filter(() => true).length || 0 // Mock - would check actual status
    const totalInterfaces = interfaces?.length || 0
    const activeInterfaces = interfaces?.filter((iface: NetworkInterface) => iface.status === 'up').length || 0

    return { totalBridges, activeBridges, totalInterfaces, activeInterfaces }
  }

  const stats = getNetworkStats()

  return (
    <Box>
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 3 }}>
        <Typography variant="h4">Network Management</Typography>
        <Box>
          <Button
            variant="outlined"
            startIcon={<RefreshIcon />}
            onClick={() => {
              queryClient.invalidateQueries({ queryKey: ['bridges'] })
              queryClient.invalidateQueries({ queryKey: ['interfaces'] })
            }}
            sx={{ mr: 2 }}
          >
            Refresh
          </Button>
          <Button
            variant="contained"
            startIcon={<AddIcon />}
            onClick={() => setCreateDialogOpen(true)}
          >
            Create Bridge
          </Button>
        </Box>
      </Box>

      {/* Network Overview */}
      <Grid container spacing={3} sx={{ mb: 3 }}>
        <Grid item xs={12} md={3}>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', alignItems: 'center' }}>
                <Avatar sx={{ bgcolor: 'primary.main', mr: 2 }}>
                  <RouterIcon />
                </Avatar>
                <Box>
                  <Typography color="textSecondary" variant="body2">
                    Network Bridges
                  </Typography>
                  <Typography variant="h5">
                    {stats.totalBridges}
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
                  <LanIcon />
                </Avatar>
                <Box>
                  <Typography color="textSecondary" variant="body2">
                    Active Bridges
                  </Typography>
                  <Typography variant="h5">
                    {stats.activeBridges}
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
                  <NetworkIcon />
                </Avatar>
                <Box>
                  <Typography color="textSecondary" variant="body2">
                    Interfaces
                  </Typography>
                  <Typography variant="h5">
                    {stats.totalInterfaces}
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
                  <EthernetIcon />
                </Avatar>
                <Box>
                  <Typography color="textSecondary" variant="body2">
                    Active Interfaces
                  </Typography>
                  <Typography variant="h5">
                    {stats.activeInterfaces}
                  </Typography>
                </Box>
              </Box>
            </CardContent>
          </Card>
        </Grid>
      </Grid>

      <Grid container spacing={3}>
        {/* Network Bridges */}
        <Grid item xs={12} md={6}>
          <Paper sx={{ p: 3, height: '100%' }}>
            <Typography variant="h6" gutterBottom>
              Network Bridges
            </Typography>
            <List>
              {bridgesLoading ? (
                <LinearProgress />
              ) : bridges?.length === 0 ? (
                <Typography color="textSecondary" sx={{ py: 2 }}>
                  No network bridges configured
                </Typography>
              ) : (
                bridges?.map((bridge: string, index: number) => (
                  <React.Fragment key={bridge}>
                    <ListItem>
                      <ListItemIcon>
                        <RouterIcon color="primary" />
                      </ListItemIcon>
                      <ListItemText
                        primary={bridge}
                        secondary={`Bridge ${index + 1}`}
                      />
                      <Chip
                        label="Active"
                        color="success"
                        size="small"
                        sx={{ mr: 1 }}
                      />
                      <IconButton size="small">
                        <EditIcon fontSize="small" />
                      </IconButton>
                      <IconButton size="small" color="error">
                        <DeleteIcon fontSize="small" />
                      </IconButton>
                    </ListItem>
                    {index < bridges.length - 1 && <Divider />}
                  </React.Fragment>
                ))
              )}
            </List>
          </Paper>
        </Grid>

        {/* Network Interfaces */}
        <Grid item xs={12} md={6}>
          <Paper sx={{ p: 3, height: '100%' }}>
            <Typography variant="h6" gutterBottom>
              Network Interfaces
            </Typography>
            <List>
              {interfacesLoading ? (
                <LinearProgress />
              ) : interfaces?.length === 0 ? (
                <Typography color="textSecondary" sx={{ py: 2 }}>
                  No network interfaces found
                </Typography>
              ) : (
                interfaces?.map((iface: NetworkInterface, index: number) => (
                  <React.Fragment key={iface.name}>
                    <ListItem>
                      <ListItemIcon>
                        {getInterfaceIcon(iface.type)}
                      </ListItemIcon>
                      <ListItemText
                        primary={iface.name}
                        secondary={
                          <Box>
                            <Typography variant="caption" display="block">
                              Type: {iface.type} | MTU: {iface.mtu || 'N/A'}
                            </Typography>
                            <Typography variant="caption" display="block">
                              MAC: {iface.mac_address || 'N/A'}
                            </Typography>
                            {iface.ip_address && (
                              <Typography variant="caption" display="block">
                                IP: {iface.ip_address}
                              </Typography>
                            )}
                          </Box>
                        }
                      />
                      <Chip
                        label={iface.status}
                        color={getStatusColor(iface.status)}
                        size="small"
                        sx={{ mr: 1 }}
                      />
                      <IconButton size="small">
                        <SettingsIcon fontSize="small" />
                      </IconButton>
                    </ListItem>
                    {index < interfaces.length - 1 && <Divider />}
                  </React.Fragment>
                ))
              )}
            </List>
          </Paper>
        </Grid>
      </Grid>

      {/* Create Bridge Dialog */}
      <Dialog open={createDialogOpen} onClose={() => setCreateDialogOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Create Network Bridge</DialogTitle>
        <DialogContent>
          <Alert severity="info" sx={{ mb: 2 }}>
            Network bridges allow containers to communicate with each other and the external network.
          </Alert>
          <TextField
            autoFocus
            margin="dense"
            label="Bridge Name"
            fullWidth
            variant="outlined"
            value={newBridge.name}
            onChange={(e) => setNewBridge({ ...newBridge, name: e.target.value })}
            sx={{ mb: 2 }}
            placeholder="e.g., lxcbr0, vmbr0"
          />
          <TextField
            margin="dense"
            label="IP Address (optional)"
            fullWidth
            variant="outlined"
            value={newBridge.ip_address}
            onChange={(e) => setNewBridge({ ...newBridge, ip_address: e.target.value })}
            placeholder="192.168.1.1/24"
            helperText="Optional IP address and subnet mask for the bridge"
            sx={{ mb: 2 }}
          />
          <FormControlLabel
            control={
              <Checkbox
                checked={newBridge.stp_enabled}
                onChange={(e) =>
                  setNewBridge({ ...newBridge, stp_enabled: e.target.checked })
                }
              />
            }
            label="Enable STP (Spanning Tree Protocol)"
          />
          <Alert severity="warning" sx={{ mt: 2 }}>
            <Typography variant="body2">
              STP helps prevent network loops but may add latency to small networks.
            </Typography>
          </Alert>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setCreateDialogOpen(false)}>Cancel</Button>
          <Button
            onClick={() => createMutation.mutate(newBridge)}
            variant="contained"
            disabled={!newBridge.name || createMutation.isPending}
          >
            Create Bridge
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
