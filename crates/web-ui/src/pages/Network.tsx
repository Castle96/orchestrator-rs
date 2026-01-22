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
  Checkbox,
  FormControlLabel,
  CircularProgress,
} from '@mui/material'
import AddIcon from '@mui/icons-material/Add'
import { networkApi, Bridge } from '../services/api'

export default function Network() {
  const queryClient = useQueryClient()
  const [createDialogOpen, setCreateDialogOpen] = useState(false)
  const [newBridge, setNewBridge] = useState({
    name: '',
    ip_address: '',
    stp_enabled: false,
  })

  const { data: bridges, isLoading } = useQuery({
    queryKey: ['bridges'],
    queryFn: () => networkApi.listBridges().then((res) => res.data.bridges || []),
  })

  const createMutation = useMutation({
    mutationFn: (data: { name: string; ip_address?: string; stp_enabled: boolean }) =>
      networkApi.createBridge(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['bridges'] })
      setCreateDialogOpen(false)
      setNewBridge({ name: '', ip_address: '', stp_enabled: false })
    },
  })

  return (
    <Box>
      <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 3 }}>
        <Typography variant="h4">Network</Typography>
        <Button
          variant="contained"
          startIcon={<AddIcon />}
          onClick={() => setCreateDialogOpen(true)}
        >
          Create Bridge
        </Button>
      </Box>

      <Typography variant="h6" gutterBottom sx={{ mt: 3 }}>
        Bridges
      </Typography>
      <TableContainer component={Paper}>
        <Table>
          <TableHead>
            <TableRow>
              <TableCell>Name</TableCell>
              <TableCell>IP Address</TableCell>
              <TableCell>STP Enabled</TableCell>
              <TableCell>Interfaces</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={4} align="center">
                  <CircularProgress />
                </TableCell>
              </TableRow>
            ) : bridges?.length === 0 ? (
              <TableRow>
                <TableCell colSpan={4} align="center">
                  No bridges found
                </TableCell>
              </TableRow>
            ) : (
              bridges?.map((bridge: string) => (
                <TableRow key={bridge}>
                  <TableCell>{bridge}</TableCell>
                  <TableCell>-</TableCell>
                  <TableCell>-</TableCell>
                  <TableCell>-</TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </TableContainer>

      <Dialog open={createDialogOpen} onClose={() => setCreateDialogOpen(false)}>
        <DialogTitle>Create Bridge</DialogTitle>
        <DialogContent>
          <TextField
            autoFocus
            margin="dense"
            label="Bridge Name"
            fullWidth
            variant="outlined"
            value={newBridge.name}
            onChange={(e) => setNewBridge({ ...newBridge, name: e.target.value })}
            sx={{ mb: 2 }}
          />
          <TextField
            margin="dense"
            label="IP Address (optional)"
            fullWidth
            variant="outlined"
            value={newBridge.ip_address}
            onChange={(e) => setNewBridge({ ...newBridge, ip_address: e.target.value })}
            placeholder="192.168.1.1/24"
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
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setCreateDialogOpen(false)}>Cancel</Button>
          <Button
            onClick={() => createMutation.mutate(newBridge)}
            variant="contained"
            disabled={!newBridge.name || createMutation.isPending}
          >
            Create
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  )
}
