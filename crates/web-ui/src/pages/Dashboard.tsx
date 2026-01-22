import React from 'react'
import { useQuery } from '@tanstack/react-query'
import {
  Grid,
  Paper,
  Typography,
  Box,
  Card,
  CardContent,
} from '@mui/material'
import { containerApi, clusterApi } from '../services/api'

export default function Dashboard() {
  const { data: containers } = useQuery({
    queryKey: ['containers'],
    queryFn: () => containerApi.list().then((res) => res.data.containers),
  })

  const { data: clusterStatus } = useQuery({
    queryKey: ['cluster-status'],
    queryFn: () => clusterApi.status().then((res) => res.data),
  })

  const runningContainers = containers?.filter((c) => c.status === 'running').length || 0
  const stoppedContainers = containers?.filter((c) => c.status === 'stopped').length || 0

  return (
    <Box>
      <Typography variant="h4" gutterBottom>
        Dashboard
      </Typography>
      <Grid container spacing={3} sx={{ mt: 2 }}>
        <Grid item xs={12} sm={6} md={3}>
          <Card>
            <CardContent>
              <Typography color="textSecondary" gutterBottom>
                Total Containers
              </Typography>
              <Typography variant="h4">
                {containers?.length || 0}
              </Typography>
            </CardContent>
          </Card>
        </Grid>
        <Grid item xs={12} sm={6} md={3}>
          <Card>
            <CardContent>
              <Typography color="textSecondary" gutterBottom>
                Running
              </Typography>
              <Typography variant="h4" color="success.main">
                {runningContainers}
              </Typography>
            </CardContent>
          </Card>
        </Grid>
        <Grid item xs={12} sm={6} md={3}>
          <Card>
            <CardContent>
              <Typography color="textSecondary" gutterBottom>
                Stopped
              </Typography>
              <Typography variant="h4" color="textSecondary">
                {stoppedContainers}
              </Typography>
            </CardContent>
          </Card>
        </Grid>
        <Grid item xs={12} sm={6} md={3}>
          <Card>
            <CardContent>
              <Typography color="textSecondary" gutterBottom>
                Cluster Nodes
              </Typography>
              <Typography variant="h4">
                {clusterStatus?.cluster?.node_count || 0}
              </Typography>
            </CardContent>
          </Card>
        </Grid>
      </Grid>
    </Box>
  )
}
