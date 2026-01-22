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
} from '@mui/material'
import PlayArrowIcon from '@mui/icons-material/PlayArrow'
import StopIcon from '@mui/icons-material/Stop'
import DeleteIcon from '@mui/icons-material/Delete'
import AddIcon from '@mui/icons-material/Add'
import { containerApi, Container, CreateContainerRequest } from '../services/api'

export default function Containers() {
  const queryClient = useQueryClient()
  const [createDialogOpen, setCreateDialogOpen] = useState(false)
  const [newContainer, setNewContainer] = useState<CreateContainerRequest>({
    name: '',
    template: 'ubuntu',
    config: {
      cpu_limit: undefined,
      memory_limit: undefined,
      disk_limit: undefined,
      network_interfaces: [],
      rootfs_path: '',
      environment: [],
    },
  })

  const { data: containers, isLoading } = useQuery({
    queryKey: ['containers'],
    queryFn: () => containerApi.list().then((res) => res.data.containers),
  })

  const startMutation = useMutation({
    mutationFn: (id: string) => containerApi.start(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['containers'] })
    },
  })

  const stopMutation = useMutation({
    mutationFn: (id: string) => containerApi.stop(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['containers'] })
    },
  })

  const deleteMutation = useMutation({
    mutationFn: (id: string) => containerApi.delete(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['containers'] })
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
          cpu_limit: undefined,
          memory_limit: undefined,
          disk_limit: undefined,
          network_interfaces: [],
          rootfs_path: '',
          environment: [],
        },
      })
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
              <TableCell>Created</TableCell>
              <TableCell align="right">Actions</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={5} align="center">
                  <CircularProgress />
                </TableCell>
              </TableRow>
            ) : containers?.length === 0 ? (
              <TableRow>
                <TableCell colSpan={5} align="center">
                  No containers found
                </TableCell>
              </TableRow>
            ) : (
              containers?.map((container) => (
                <TableRow key={container.id}>
                  <TableCell>{container.name}</TableCell>
                  <TableCell>
                    <Chip
                      label={container.status}
                      color={getStatusColor(container.status) as any}
                      size="small"
                    />
                  </TableCell>
                  <TableCell>{container.template}</TableCell>
                  <TableCell>
                    {new Date(container.created_at).toLocaleDateString()}
                  </TableCell>
                  <TableCell align="right">
                    {container.status === 'running' ? (
                      <IconButton
                        size="small"
                        onClick={() => stopMutation.mutate(container.name)}
                        disabled={stopMutation.isPending}
                      >
                        <StopIcon />
                      </IconButton>
                    ) : (
                      <IconButton
                        size="small"
                        onClick={() => startMutation.mutate(container.name)}
                        disabled={startMutation.isPending}
                      >
                        <PlayArrowIcon />
                      </IconButton>
                    )}
                    <IconButton
                      size="small"
                      onClick={() => deleteMutation.mutate(container.name)}
                      disabled={deleteMutation.isPending}
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

      <Dialog open={createDialogOpen} onClose={() => setCreateDialogOpen(false)}>
        <DialogTitle>Create Container</DialogTitle>
        <DialogContent>
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
            sx={{ mb: 2 }}
          />
          <TextField
            margin="dense"
            label="Template"
            fullWidth
            variant="outlined"
            value={newContainer.template}
            onChange={(e) =>
              setNewContainer({ ...newContainer, template: e.target.value })
            }
          />
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
    </Box>
  )
}
