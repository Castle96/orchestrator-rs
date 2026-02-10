import axios from 'axios'

interface ImportMetaEnv {
  env?: {
    VITE_API_URL?: string
    [key: string]: string | undefined
  }
}

interface ImportMeta {
  env: ImportMetaEnv['env']
}

interface NetworkInterface {
  name: string
  type: 'ethernet' | 'wifi' | 'bridge'
  status: 'up' | 'down'
  ip_address?: string
  mac_address?: string
}

const API_BASE_URL = (import.meta as ImportMeta).env?.VITE_API_URL || '/api/v1'

const api = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
})

export interface Container {
  id: string
  name: string
  status: 'stopped' | 'starting' | 'running' | 'stopping' | 'frozen' | 'error'
  template: string
  node_id?: string
  created_at: string
  updated_at: string
  config: {
    cpu_limit?: number
    memory_limit?: number
    disk_limit?: number
    network_interfaces: NetworkInterface[]
    rootfs_path: string
    environment: [string, string][]
  }
}

export interface CreateContainerRequest {
  name: string
  template: string
  config: {
    cpu_limit?: number
    memory_limit?: number
    disk_limit?: number
    network_interfaces: NetworkInterface[]
    rootfs_path: string
    environment: [string, string][]
  }
}

export interface Node {
  id: string
  name: string
  address: string
  port: number
  status: 'online' | 'offline' | 'joining' | 'leaving'
  cluster_id?: string
  resources: {
    cpu_cores: number
    memory_total: number
    memory_used: number
    disk_total: number
    disk_used: number
  }
  joined_at: string
  last_seen: string
}

export interface StoragePool {
  id: string
  name: string
  storage_type: 'local' | 'nfs' | 'cifs'
  path: string
  total_size: number
  used_size: number
  available_size: number
  created_at: string
}

export interface Bridge {
  name: string
  interfaces: string[]
  ip_address?: string
  stp_enabled: boolean
}

export const containerApi = {
  list: () => api.get<{ containers: Container[] }>('/containers'),
  get: (id: string) => api.get<{ container: Container }>(`/containers/${id}`),
  create: (data: CreateContainerRequest) => api.post<{ container: Container }>('/containers', data),
  start: (id: string) => api.post(`/containers/${id}/start`),
  stop: (id: string) => api.post(`/containers/${id}/stop`),
  delete: (id: string) => api.delete(`/containers/${id}`),
}

export const clusterApi = {
  listNodes: () => api.get<{ nodes: Node[] }>('/cluster/nodes'),
  join: (data: { cluster_name: string; node_address: string; node_port: number }) =>
    api.post('/cluster/join', data),
  status: () => api.get('/cluster/status'),
}

export const storageApi = {
  listPools: () => api.get<{ pools: StoragePool[] }>('/storage'),
  createPool: (data: { name: string; storage_type: 'local' | 'nfs' | 'cifs'; path: string }) =>
    api.post<StoragePool>('/storage', data),
}

export const networkApi = {
  listInterfaces: () => api.get<{ interfaces: NetworkInterface[] }>('/network'),
  listBridges: () => api.get<{ bridges: string[] }>('/network/bridges'),
  createBridge: (data: { name: string; ip_address?: string; stp_enabled: boolean }) =>
    api.post<Bridge>('/network/bridges', data),
}

export default api
