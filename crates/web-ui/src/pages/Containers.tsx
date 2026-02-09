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
  Chip,
  IconButton,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  CircularProgress,
  Grid,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  Tabs,
  Tab,
  Accordion,
  AccordionSummary,
  AccordionDetails,
  Divider,
  Alert,
  Snackbar,
  LinearProgress,
} from '@mui/material'
import PlayArrowIcon from '@mui/icons-material/PlayArrow'
import StopIcon from '@mui/icons-material/Stop'
import DeleteIcon from '@mui/icons-material/Delete'
import AddIcon from '@mui/icons-material/Add'
import SettingsIcon from '@mui/icons-material/Settings'
import ComputerIcon from '@mui/icons-material/Computer'
import MemoryIcon from '@mui/icons-material/Memory'
import StorageIcon from '@mui/icons-material/Storage'
import NetworkIcon from '@mui/icons-material/Network'
import ExpandMoreIcon from '@mui/icons-material/ExpandMore'
import { containerApi, Container, CreateContainerRequest } from '../services/api'

interface TabPanelProps {
  children?: React.ReactNode
  index: number
  value: number
}

function TabPanel(props: TabPanelProps) {
  const { children, value, index, ...other } = props

  return (
    <div
      role="tabpanel"
      hidden={value !== index}
      id={`container-tabpanel-${index}`}
      aria-labelledby={`container-tab-${index}`}
      {...other}
    >
      {value === index && <Box sx={{ p: 3 }}>{children}</Box>}
    </div>
  )
}

export default function Containers() {
  const queryClient = useQueryClient()
  const [createDialogOpen, setCreateDialogOpen] = useState(false)
  const [selectedContainer, setSelectedContainer] = useState<Container | null>(null)
  const [configDialogOpen, setConfigDialogOpen] = useState(false)
  const [tabValue, setTabValue] = useState(0)
  const [notification, setNotification] = useState({ open: false, message: '', severity: 'success' as 'success' | 'error' })
  
  const [newContainer, setNewContainer] = useState<CreateContainerRequest>({
    name: '',
    template: 'ubuntu',
    config: {
      cpu_limit: 1,
      memory_limit: 512 * 1024 * 1024, // 512MB in bytes
      disk_limit: 8 * 1024 * 1024 * 1024, // 8GB in bytes
      network_interfaces: [{ name: 'eth0', bridge: 'lxcbr0' }],
      rootfs_path: '',
      environment: [],
    },
  })

  const { data: containers, isLoading } = useQuery({
    queryKey: ['containers'],
    queryFn: () => containerApi.list().then((res) => res.data.containers),
    refetchInterval: 5000, // Refresh every 5 seconds
  })

  const startMutation = useMutation({
    mutationFn: (id: string) => containerApi.start(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['containers'] })
      setNotification({ open: true, message: 'Container started successfully', severity: 'success' })
    },
    onError: (error: any) => {
      setNotification({ open: true, message: `Failed to start container: ${error.message}`, severity: 'error' })
    },
  })

  const stopMutation = useMutation({
    mutationFn: (id: string) => containerApi.stop(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['containers'] })
      setNotification({ open: true, message: 'Container stopped successfully', severity: 'success' })
    },
    onError: (error: any) => {
      setNotification({ open: true, message: `Failed to stop container: ${error.message}`, severity: 'error' })
    },
  })

  const deleteMutation = useMutation({
    mutationFn: (id: string) => containerApi.delete(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['containers'] })
      setNotification({ open: true, message: 'Container deleted successfully', severity: 'success' })
    },
    onError: (error: any) => {
      setNotification({ open: true, message: `Failed to delete container: ${error.message}`, severity: 'error' })
    },
  })

  const createMutation = useMutation({
    mutationFn: (data: CreateContainerRequest) => containerApi.create(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['containers'] })
      setCreateDialogOpen(false)
      setNewContainer({
        name: '',
        template: 'ubuntu',
        config: {
          cpu_limit: 1,
          memory_limit: 512 * 1024 * 1024,
          disk_limit: 8 * 1024 * 1024 * 1024,
          network_interfaces: [{ name: 'eth0', bridge: 'lxcbr0' }],
          rootfs_path: '',
          environment: [],
        },
      })
      setNotification({ open: true, message: 'Container created successfully', severity: 'success' })
    },
    onError: (error: any) => {
      setNotification({ open: true, message: `Failed to create container: ${error.message}`, severity: 'error' })
    },
  })

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'running':
        return 'success'
      case 'stopped':
        return 'default'
      case 'starting':
      case 'stopping':
        return 'warning'
      case 'error':
        return 'error'
      default:
        return 'default'
    }
  }

  const formatBytes = (bytes: number) => {
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
    if (bytes === 0) return '0 B'
    const i = Math.floor(Math.log(bytes) / Math.log(1024))
    return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + ' ' + sizes[i]
  }

  return (
    <Box>
      <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 3 }}>
        <Typography variant="h4">Containers</Typography>
        <Button
          variant="contained"
          startIcon={<AddIcon />}
          onClick={() => setCreateDialogOpen(true)}
        >
          Create Container
        </Button>
      </Box>

      <TableContainer component={Paper}>
        <Table>
          <TableHead>
            <TableRow>
              <TableCell>Name</TableCell>
              <TableCell>Status</TableCell>
              <TableCell>Template</TableCell>
              <TableCell>CPU</TableCell>
              <TableCell>Memory</TableCell>
              <TableCell>Disk</TableCell>
              <TableCell>Created</TableCell>
              <TableCell align="right">Actions</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={8} align="center">
                  <LinearProgress />
                </TableCell>
              </TableRow>
            ) : containers?.length === 0 ? (
              <TableRow>
                <TableCell colSpan={8} align="center" sx={{ py: 4 }}>
                  <Typography color="textSecondary">
                    No containers found. Create your first container to get started.
                  </Typography>
                </TableCell>
              </TableRow>
            ) : (
              containers?.map((container) => (
                <TableRow key={container.id} hover>
                  <TableCell>
                    <Box sx={{ display: 'flex', alignItems: 'center' }}>
                      <ComputerIcon sx={{ mr: 1, fontSize: 20 }} />
                      <Typography variant="body2" fontWeight="medium">
                        {container.name}
                      </Typography>
                    </Box>
                  </TableCell>
                  <TableCell>
                    <Chip
                      label={container.status}
                      color={getStatusColor(container.status) as any}
                      size="small"
                      sx={{ fontWeight: 'medium' }}
                    />
                  </TableCell>
                  <TableCell>
                    <Typography variant="body2">{container.template}</Typography>
                  </TableCell>
                  <TableCell>
                    <Box sx={{ display: 'flex', alignItems: 'center' }}>
                      <SettingsIcon sx={{ mr: 1, fontSize: 16, color: 'text.secondary' }} />
                      <Typography variant="body2">
                        {container.config.cpu_limit || 'Unlimited'}
                      </Typography>
                    </Box>
                  </TableCell>
                  <TableCell>
                    <Box sx={{ display: 'flex', alignItems: 'center' }}>
                      <MemoryIcon sx={{ mr: 1, fontSize: 16, color: 'text.secondary' }} />
                      <Typography variant="body2">
                        {container.config.memory_limit ? formatBytes(container.config.memory_limit) : 'Unlimited'}
                      </Typography>
                    </Box>
                  </TableCell>
                  <TableCell>
                    <Box sx={{ display: 'flex', alignItems: 'center' }}>
                      <StorageIcon sx={{ mr: 1, fontSize: 16, color: 'text.secondary' }} />
                      <Typography variant="body2">
                        {container.config.disk_limit ? formatBytes(container.config.disk_limit) : 'Unlimited'}
                      </Typography>
                    </Box>
                  </TableCell>
                  <TableCell>
                    <Typography variant="body2">
                      {new Date(container.created_at).toLocaleDateString()}
                    </Typography>
                  </TableCell>
                  <TableCell align="right">
                    <IconButton
                      size="small"
                      onClick={() => {
                        setSelectedContainer(container)
                        setConfigDialogOpen(true)
                      }}
                      title="Configuration"
                    >
                      <SettingsIcon />
                    </IconButton>
                    {container.status === 'running' ? (
                      <IconButton
                        size="small"
                        onClick={() => stopMutation.mutate(container.name)}
                        disabled={stopMutation.isPending}
                        title="Stop"
                      >
                        <StopIcon />
                      </IconButton>
                    ) : (
                      <IconButton
                        size="small"
                        onClick={() => startMutation.mutate(container.name)}
                        disabled={startMutation.isPending}
                        title="Start"
                      >
                        <PlayArrowIcon />
                      </IconButton>
                    )}
                    <IconButton
                      size="small"
                      onClick={() => deleteMutation.mutate(container.name)}
                      disabled={deleteMutation.isPending}
                      title="Delete"
                      color="error"
                    >
                      <DeleteIcon />
                    </IconButton>
                  </TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </TableContainer>

      {/* Create Container Dialog */}
      <Dialog open={createDialogOpen} onClose={() => setCreateDialogOpen(false)} maxWidth="md" fullWidth>
        <DialogTitle>Create Container</DialogTitle>
        <DialogContent>
          <Tabs value={tabValue} onChange={(_, newValue) => setTabValue(newValue)} sx={{ mb: 2 }}>
            <Tab label="Basic" />
            <Tab label="Resources" />
            <Tab label="Network" />
            <Tab label="Advanced" />
          </Tabs>

          <TabPanel value={tabValue} index={0}>
            <Grid container spacing={2}>
              <Grid item xs={12}>
                <TextField
                  autoFocus
                  margin="dense"
                  label="Container Name"
                  fullWidth
                  variant="outlined"
                  value={newContainer.name}
                  onChange={(e) =>
                    setNewContainer({ ...newContainer, name: e.target.value })
                  }
                />
              </Grid>
              <Grid item xs={12}>
                <FormControl fullWidth>
                  <InputLabel>Template</InputLabel>
                  <Select
                    value={newContainer.template}
                    label="Template"
                    onChange={(e) =>
                      setNewContainer({ ...newContainer, template: e.target.value })
                    }
                  >
                    <MenuItem value="ubuntu">Ubuntu</MenuItem>
                    <MenuItem value="debian">Debian</MenuItem>
                    <MenuItem value="alpine">Alpine</MenuItem>
                    <MenuItem value="centos">CentOS</MenuItem>
                    <MenuItem value="busybox">BusyBox</MenuItem>
                  </Select>
                </FormControl>
              </Grid>
            </Grid>
          </TabPanel>

          <TabPanel value={tabValue} index={1}>
            <Grid container spacing={2}>
              <Grid item xs={12} md={4}>
                <TextField
                  margin="dense"
                  label="CPU Cores"
                  type="number"
                  fullWidth
                  variant="outlined"
                  value={newContainer.config.cpu_limit || ''}
                  onChange={(e) =>
                    setNewContainer({
                      ...newContainer,
                      config: { ...newContainer.config, cpu_limit: parseInt(e.target.value) || undefined }
                    })
                  }
                  inputProps={{ min: 1, max: 32 }}
                />
              </Grid>
              <Grid item xs={12} md={4}>
                <TextField
                  margin="dense"
                  label="Memory (MB)"
                  type="number"
                  fullWidth
                  variant="outlined"
                  value={(newContainer.config.memory_limit || 0) / (1024 * 1024)}
                  onChange={(e) =>
                    setNewContainer({
                      ...newContainer,
                      config: { ...newContainer.config, memory_limit: parseInt(e.target.value) * 1024 * 1024 }
                    })
                  }
                  inputProps={{ min: 64, step: 64 }}
                />
              </Grid>
              <Grid item xs={12} md={4}>
                <TextField
                  margin="dense"
                  label="Disk Size (GB)"
                  type="number"
                  fullWidth
                  variant="outlined"
                  value={(newContainer.config.disk_limit || 0) / (1024 * 1024 * 1024)}
                  onChange={(e) =>
                    setNewContainer({
                      ...newContainer,
                      config: { ...newContainer.config, disk_limit: parseInt(e.target.value) * 1024 * 1024 * 1024 }
                    })
                  }
                  inputProps={{ min: 1, step: 1 }}
                />
              </Grid>
            </Grid>
          </TabPanel>

          <TabPanel value={tabValue} index={2}>
            <Typography variant="h6" gutterBottom>
              Network Interfaces
            </Typography>
            {newContainer.config.network_interfaces.map((iface, index) => (
              <Accordion key={index}>
                <AccordionSummary expandIcon={<ExpandMoreIcon />}>
                  <Typography>{iface.name}</Typography>
                </AccordionSummary>
                <AccordionDetails>
                  <Grid container spacing={2}>
                    <Grid item xs={6}>
                      <TextField
                        margin="dense"
                        label="Interface Name"
                        fullWidth
                        variant="outlined"
                        value={iface.name}
                        onChange={(e) => {
                          const updated = [...newContainer.config.network_interfaces]
                          updated[index] = { ...updated[index], name: e.target.value }
                          setNewContainer({
                            ...newContainer,
                            config: { ...newContainer.config, network_interfaces: updated }
                          })
                        }}
                      />
                    </Grid>
                    <Grid item xs={6}>
                      <TextField
                        margin="dense"
                        label="Bridge"
                        fullWidth
                        variant="outlined"
                        value={iface.bridge}
                        onChange={(e) => {
                          const updated = [...newContainer.config.network_interfaces]
                          updated[index] = { ...updated[index], bridge: e.target.value }
                          setNewContainer({
                            ...newContainer,
                            config: { ...newContainer.config, network_interfaces: updated }
                          })
                        }}
                      />
                    </Grid>
                  </Grid>
                </AccordionDetails>
              </Accordion>
            ))}
          </TabPanel>

          <TabPanel value={tabValue} index={3}>
            <Typography variant="h6" gutterBottom>
              Environment Variables
            </Typography>
            <Alert severity="info" sx={{ mb: 2 }}>
              Environment variables will be available inside the container.
            </Alert>
            {/* TODO: Add environment variable management */}
          </TabPanel>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setCreateDialogOpen(false)}>Cancel</Button>
          <Button
            onClick={() => createMutation.mutate(newContainer)}
            variant="contained"
            disabled={!newContainer.name || createMutation.isPending}
          >
            Create
          </Button>
        </DialogActions>
      </Dialog>

      {/* Configuration Dialog */}
      <Dialog open={configDialogOpen} onClose={() => setConfigDialogOpen(false)} maxWidth="md" fullWidth>
        <DialogTitle>Container Configuration - {selectedContainer?.name}</DialogTitle>
        <DialogContent>
          {selectedContainer && (
            <Grid container spacing={2}>
              <Grid item xs={12}>
                <Typography variant="h6" gutterBottom>
                  Basic Information
                </Typography>
                <Divider sx={{ mb: 2 }} />
              </Grid>
              <Grid item xs={6}>
                <Typography variant="body2" color="textSecondary">
                  Status
                </Typography>
                <Chip
                  label={selectedContainer.status}
                  color={getStatusColor(selectedContainer.status) as any}
                  size="small"
                />
              </Grid>
              <Grid item xs={6}>
                <Typography variant="body2" color="textSecondary">
                  Template
                </Typography>
                <Typography variant="body1">{selectedContainer.template}</Typography>
              </Grid>
              <Grid item xs={6}>
                <Typography variant="body2" color="textSecondary">
                  Created
                </Typography>
                <Typography variant="body1">
                  {new Date(selectedContainer.created_at).toLocaleString()}
                </Typography>
              </Grid>
              <Grid item xs={6}>
                <Typography variant="body2" color="textSecondary">
                  Last Updated
                </Typography>
                <Typography variant="body1">
                  {new Date(selectedContainer.updated_at).toLocaleString()}
                </Typography>
              </Grid>
              
              <Grid item xs={12} sx={{ mt: 2 }}>
                <Typography variant="h6" gutterBottom>
                  Resource Limits
                </Typography>
                <Divider sx={{ mb: 2 }} />
              </Grid>
              <Grid item xs={4}>
                <Box sx={{ display: 'flex', alignItems: 'center' }}>
                  <SettingsIcon sx={{ mr: 1, fontSize: 20, color: 'text.secondary' }} />
                  <Box>
                    <Typography variant="body2" color="textSecondary">
                      CPU Cores
                    </Typography>
                    <Typography variant="body1">
                      {selectedContainer.config.cpu_limit || 'Unlimited'}
                    </Typography>
                  </Box>
                </Box>
              </Grid>
              <Grid item xs={4}>
                <Box sx={{ display: 'flex', alignItems: 'center' }}>
                  <MemoryIcon sx={{ mr: 1, fontSize: 20, color: 'text.secondary' }} />
                  <Box>
                    <Typography variant="body2" color="textSecondary">
                      Memory
                    </Typography>
                    <Typography variant="body1">
                      {selectedContainer.config.memory_limit ? formatBytes(selectedContainer.config.memory_limit) : 'Unlimited'}
                    </Typography>
                  </Box>
                </Box>
              </Grid>
              <Grid item xs={4}>
                <Box sx={{ display: 'flex', alignItems: 'center' }}>
                  <StorageIcon sx={{ mr: 1, fontSize: 20, color: 'text.secondary' }} />
                  <Box>
                    <Typography variant="body2" color="textSecondary">
                      Disk
                    </Typography>
                    <Typography variant="body1">
                      {selectedContainer.config.disk_limit ? formatBytes(selectedContainer.config.disk_limit) : 'Unlimited'}
                    </Typography>
                  </Box>
                </Box>
              </Grid>
            </Grid>
          )}
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setConfigDialogOpen(false)}>Close</Button>
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
