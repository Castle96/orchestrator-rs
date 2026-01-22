import React from 'react'
import { useQuery } from '@tanstack/react-query'
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
  CircularProgress,
} from '@mui/material'
import { clusterApi } from '../services/api'

export default function Cluster() {
  const { data: nodes, isLoading } = useQuery({
    queryKey: ['cluster-nodes'],
    queryFn: () => clusterApi.listNodes().then((res) => res.data.nodes),
  })

  const { data: clusterStatus } = useQuery({
    queryKey: ['cluster-status'],
    queryFn: () => clusterApi.status().then((res) => res.data),
  })

  const getStatusColor = (status: string) => {
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

  return (
    <Box>
      <Typography variant="h4" gutterBottom>
        Cluster
      </Typography>

      <Paper sx={{ p: 2, mb: 3 }}>
        <Typography variant="h6" gutterBottom>
          Cluster Information
        </Typography>
        <Typography>
          <strong>Name:</strong> {clusterStatus?.cluster?.name || 'N/A'}
        </Typography>
        <Typography>
          <strong>Node Count:</strong> {clusterStatus?.cluster?.node_count || 0}
        </Typography>
      </Paper>

      <Typography variant="h6" gutterBottom sx={{ mt: 3 }}>
        Nodes
      </Typography>
      <TableContainer component={Paper}>
        <Table>
          <TableHead>
            <TableRow>
              <TableCell>Name</TableCell>
              <TableCell>Address</TableCell>
              <TableCell>Status</TableCell>
              <TableCell>CPU Cores</TableCell>
              <TableCell>Memory</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={5} align="center">
                  <CircularProgress />
                </TableCell>
              </TableRow>
            ) : nodes?.length === 0 ? (
              <TableRow>
                <TableCell colSpan={5} align="center">
                  No nodes in cluster
                </TableCell>
              </TableRow>
            ) : (
              nodes?.map((node) => (
                <TableRow key={node.id}>
                  <TableCell>{node.name}</TableCell>
                  <TableCell>
                    {node.address}:{node.port}
                  </TableCell>
                  <TableCell>
                    <Chip
                      label={node.status}
                      color={getStatusColor(node.status) as any}
                      size="small"
                    />
                  </TableCell>
                  <TableCell>{node.resources.cpu_cores}</TableCell>
                  <TableCell>
                    {(node.resources.memory_total / 1024 / 1024 / 1024).toFixed(2)} GB
                  </TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </TableContainer>
    </Box>
  )
}
