
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom'
import { ThemeProvider, createTheme } from '@mui/material/styles'
import CssBaseline from '@mui/material/CssBaseline'
import Layout from './components/Layout'
import Dashboard from './pages/Dashboard'
import Containers from './pages/Containers'
import Cluster from './pages/Cluster'
import Storage from './pages/Storage'
import Network from './pages/Network'

const theme = createTheme({
  palette: {
    mode: 'light',
    primary: {
      main: '#1976d2',
    },
    secondary: {
      main: '#dc004e',
    },
  },
})

function App() {
  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <Router>
        <Layout>
          <Routes>
            <Route path="/" element={<Dashboard />} />
            <Route path="/containers" element={<Containers />} />
            <Route path="/cluster" element={<Cluster />} />
            <Route path="/storage" element={<Storage />} />
            <Route path="/network" element={<Network />} />
          </Routes>
        </Layout>
      </Router>
    </ThemeProvider>
  )
}

export default App
