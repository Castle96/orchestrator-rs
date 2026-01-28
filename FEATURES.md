# New Features Documentation

This document describes the new features implemented for the ARM Hypervisor platform.

## 1. Enhanced Observability & Monitoring

### Prometheus Metrics

The platform now exports metrics in Prometheus format for monitoring and alerting.

**Endpoint:** `GET /metrics`

**Metrics exported:**
- `arm_hypervisor_http_requests_total` - Total HTTP requests received
- `arm_hypervisor_http_errors_total` - Total HTTP errors
- `arm_hypervisor_uptime_seconds` - Server uptime
- `arm_hypervisor_system_load_*` - System load average (1min, 5min, 15min)
- `arm_hypervisor_memory_*` - Memory usage metrics
- `arm_hypervisor_disk_*` - Disk usage metrics  
- `arm_hypervisor_cpu_count` - Number of CPU cores
- `arm_hypervisor_containers_*` - Container status metrics
- `arm_hypervisor_bridges_total` - Network bridge count

**JSON Metrics Endpoint:** `GET /metrics/json`

Returns the same metrics in JSON format for programmatic access.

### Health Checks

**Health Check:** `GET /health`
Returns overall system health status with service-level details.

**Readiness Check:** `GET /ready`
Kubernetes-compatible readiness probe that checks if the service can accept traffic.

### Request Tracing

All requests are automatically traced with correlation IDs. The correlation ID can be:
- Provided in the `X-Correlation-ID` header
- Auto-generated if not provided

Correlation IDs appear in all logs for request tracking across the system.

## 2. Container Snapshot Management

### Create Snapshot

Create a snapshot of a container for backup or cloning.

```bash
POST /api/v1/containers/{container_name}/snapshots
Content-Type: application/json

{
  "name": "backup-20240128",  // Optional
  "comment": "Before upgrade"  // Optional
}
```

### List Snapshots

List all snapshots for a container.

```bash
GET /api/v1/containers/{container_name}/snapshots
```

### Restore Snapshot

Restore a container to a previous snapshot state.

```bash
POST /api/v1/containers/{container_name}/snapshots/restore
Content-Type: application/json

{
  "snapshot_name": "backup-20240128"
}
```

### Delete Snapshot

Delete a snapshot.

```bash
DELETE /api/v1/containers/{container_name}/snapshots/{snapshot_name}
```

### Clone from Snapshot

Create a new container from a snapshot.

```bash
POST /api/v1/containers/{container_name}/snapshots/clone
Content-Type: application/json

{
  "snapshot_name": "backup-20240128",
  "new_container_name": "test-container-clone"
}
```

## 3. Role-Based Access Control (RBAC)

### Built-in Roles

**Admin:**
- Full system access
- All permissions

**Operator:**
- Can manage containers (start, stop, snapshot)
- Read access to cluster, storage, network
- Cannot create or delete resources

**Viewer:**
- Read-only access to all resources
- Cannot modify anything

**Custom:**
- Define custom permissions per user

### Permissions

- **Container:** Create, Read, Update, Delete, Start, Stop, Snapshot
- **Cluster:** Read, Write, Join, Leave
- **Storage:** Read, Write, Delete
- **Network:** Read, Write, Delete
- **System:** Read, Write, Admin

### User Management API

**List Users:**
```bash
GET /api/v1/users
```

**Get User:**
```bash
GET /api/v1/users/{username}
```

**Create User:**
```bash
POST /api/v1/users
Content-Type: application/json

{
  "username": "john",
  "email": "john@example.com",
  "role": "Operator"
}
```

**Update User:**
```bash
PUT /api/v1/users/{username}
Content-Type: application/json

{
  "email": "newemail@example.com",
  "role": "Viewer",
  "enabled": true
}
```

**Delete User:**
```bash
DELETE /api/v1/users/{username}
```

**Note:** The default admin user cannot be deleted.

## 4. Audit Logging

All system operations are automatically logged for security auditing.

### Query Audit Logs

```bash
GET /api/v1/audit/logs?user=admin&resource_type=container&limit=100
```

**Query Parameters:**
- `user` - Filter by username
- `resource_type` - Filter by resource type (container, user, cluster, etc.)
- `limit` - Maximum number of logs to return

**Response:**
```json
{
  "total": 1523,
  "logs": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "timestamp": "2024-01-28T12:34:56Z",
      "user": "admin",
      "action": "ContainerCreated",
      "resource_type": "container",
      "resource_id": "web-server",
      "result": "Success",
      "ip_address": "192.168.1.100",
      "correlation_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
      "details": "Created alpine container"
    }
  ]
}
```

### Audit Actions Tracked

- Container: Created, Deleted, Started, Stopped, Updated, Snapshot operations
- User: Created, Updated, Deleted, Login, Logout
- Cluster: Joined, Left, Node operations
- Storage: Pool/Volume created/deleted
- Network: Bridge/Interface created/deleted
- System: Configuration changes, start/stop

### Log Retention

By default, the system keeps the most recent 10,000 audit log entries in memory. For production use, configure a persistent audit log backend.

## Configuration Examples

### Prometheus Integration

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'arm-hypervisor'
    static_configs:
      - targets: ['hypervisor-host:8443']
    metrics_path: '/metrics'
    scheme: 'https'
    tls_config:
      insecure_skip_verify: true  # Use proper certs in production
```

### Grafana Dashboard

Import metrics using the Prometheus data source. Key metrics to monitor:
- Request rate: `rate(arm_hypervisor_http_requests_total[5m])`
- Error rate: `rate(arm_hypervisor_http_errors_total[5m])`
- Container count: `arm_hypervisor_containers_running`
- System load: `arm_hypervisor_system_load_1min`

### Kubernetes Integration

```yaml
# deployment.yaml
apiVersion: v1
kind: Pod
metadata:
  name: arm-hypervisor
spec:
  containers:
  - name: api-server
    image: arm-hypervisor:latest
    livenessProbe:
      httpGet:
        path: /health
        port: 8443
        scheme: HTTPS
      initialDelaySeconds: 30
      periodSeconds: 10
    readinessProbe:
      httpGet:
        path: /ready
        port: 8443
        scheme: HTTPS
      initialDelaySeconds: 5
      periodSeconds: 5
```

## Security Considerations

1. **RBAC:** Always use least-privilege principle when assigning roles
2. **Audit Logs:** Regularly review audit logs for suspicious activity
3. **Snapshots:** Snapshots may contain sensitive data; secure appropriately
4. **Metrics:** Metrics endpoint exposes system information; protect with authentication
5. **Default Admin:** Change the default admin credentials immediately after installation

## Future Enhancements

Planned features for future releases:
- Permission enforcement middleware (currently infrastructure only)
- OAuth2/OIDC integration for authentication
- Persistent audit log storage (database/file)
- Snapshot encryption
- Automated snapshot scheduling
- Snapshot retention policies
- Live resource adjustment for containers
- Container template management
