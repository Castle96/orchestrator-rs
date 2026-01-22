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
  CircularProgress,
} from '@mui/material'
import AddIcon from '@mui/icons-material/Add'
import { storageApi, StoragePool } from '../services/api'

export default function Storage() {
  const queryClient = useQueryClient()
  const [createDialogOpen, setCreateDialogOpen] = useState(false)
  const [newPool, setNewPool] = useState({
    name: '',
    storage_type: 'local' as 'local' | 'nfs' | 'cifs',
    path: '',
  })

  const { data: pools, isLoading } = useQuery({
    queryKey: ['storage-pools'],
    queryFn: () => storageApi.listPools().then((res) => res.data.pools),
  })

  const createMutation = useMutation({
    mutationFn: (data: typeof newPool) => storageApi.createPool(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['storage-pools'] })
      setCreateDialogOpen(false)
      setNewPool({ name: '', storage_type: 'local', path: '' })
    },
  })

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i]
  }

  return (
    <Box>
      <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 3 }}>
        <Typography variant="h4">Storage</Typography>
        <Button
          variant="contained"
          startIcon={<AddIcon />}
          onClick={() => setCreateDialogOpen(true)}
        >
          Create Storage Pool
        </Button>
      </Box>

      <TableContainer component={Paper}>
        <Table>
          <TableHead>
            <TableRow>
              <TableCell>Name</TableCell>
              <TableCell>Type</TableCell>
              <TableCell>Path</TableCell>
              <TableCell>Total Size</TableCell>
              <TableCell>Used</TableCell>
              <TableCell>Available</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={6} align="center">
                  <CircularProgress />
                </TableCell>
              </TableRow>
            ) : pools?.length === 0 ? (
              <TableRow>
                <TableCell colSpan={6} align="center">
                  No storage pools found
                </TableCell>
              </TableRow>
            ) : (
              pools?.map((pool) => (
                <TableRow key={pool.id}>
                  <TableCell>{pool.name}</TableCell>
                  <TableCell>{pool.storage_type.toUpperCase()}</TableCell>
                  <TableCell>{pool.path}</TableCell>
                  <TableCell>{formatBytes(pool.total_size)}</TableCell>
                  <TableCell>{formatBytes(pool.used_size)}</TableCell>
                  <TableCell>{formatBytes(pool.available_size)}</TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </TableContainer>

      <Dialog open={createDialogOpen} onClose={() => setCreateDialogOpen(false)}>
        <DialogTitle>Create Storage Pool</DialogTitle>
        <DialogContent>
          <TextField
            autoFocus
            margin="dense"
            label="Pool Name"
            fullWidth
            variant="outlined"
            value={newPool.name}
            onChange={(e) => setNewPool({ ...newPool, name: e.target.value })}
            sx={{ mb: 2 }}
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
              <MenuItem value="local">Local</MenuItem>
              <MenuItem value="nfs">NFS</MenuItem>
              <MenuItem value="cifs">CIFS</MenuItem>
            </Select>
          </FormControl>
          <TextField
            margin="dense"
            label={newPool.storage_type === 'local' ? 'Path' : newPool.storage_type === 'nfs' ? 'Server:Path' : '//Server/Share'}
            fullWidth
            variant="outlined"
            value={newPool.path}
            onChange={(e) => setNewPool({ ...newPool, path: e.target.value })}
            placeholder={
              newPool.storage_type === 'local'
                ? '/var/lib/arm-hypervisor/storage'
                : newPool.storage_type === 'nfs'
                ? '192.168.1.100:/exports/pool'
                : '//server/share'
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
            Create
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  )
}
