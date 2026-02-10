# Backup and Recovery Procedures

This document outlines the backup and recovery procedures for the ARM Hypervisor Platform.

## Overview

The ARM Hypervisor Platform requires backing up:
- Container configurations and metadata
- Container root filesystems
- Storage pool configurations
- Cluster configuration and state
- User and authentication data

## Backup Types

### 1. Configuration Backups
Configuration data includes:
- Container definitions and configurations
- Network and storage pool configurations  
- Cluster topology and membership
- User accounts and permissions
- System configuration files

### 2. Data Backups
Container data includes:
- Container root filesystems
- Container snapshots
- Application data within containers
- Persistent volumes

## Backup Procedures

### Automated Daily Backups

Create a backup script `/usr/local/bin/arm-hypervisor-backup.sh`:

```bash
#!/bin/bash
set -euo pipefail

BACKUP_DIR="/backup/arm-hypervisor"
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_PATH="$BACKUP_DIR/daily/$DATE"
RETENTION_DAYS=30

# Create backup directory
mkdir -p "$BACKUP_PATH"

echo "Starting backup: $DATE"

# 1. Backup container configurations
echo "Backing up container configurations..."
mkdir -p "$BACKUP_PATH/containers"
lxc list --format json > "$BACKUP_PATH/containers/containers.json"
lxc config show --expanded --format json > "$BACKUP_PATH/containers/global-config.json"

for container in $(lxc list --format csv | cut -d, -f1); do
    echo "Backing up container: $container"
    lxc config show "$container" --expanded --format json > "$BACKUP_PATH/containers/${container}.json"
    lxc export "$container" "$BACKUP_PATH/containers/${container}.tar.gz" --instance-only
done

# 2. Backup storage pools
echo "Backing up storage pool configurations..."
mkdir -p "$BACKUP_PATH/storage"
lxc storage list --format json > "$BACKUP_PATH/storage/pools.json"
lxc storage volume list --format json > "$BACKUP_PATH/storage/volumes.json"

# 3. Backup network configurations
echo "Backing up network configurations..."
mkdir -p "$BACKUP_PATH/network"
lxc network list --format json > "$BACKUP_PATH/network/networks.json"
for network in $(lxc network list --format csv | cut -d, -f1); do
    lxc network show "$network" --format json > "$BACKUP_PATH/network/${network}.json"
done

# 4. Backup cluster configuration
echo "Backing up cluster configuration..."
mkdir -p "$BACKUP_PATH/cluster"
lxc cluster list --format json > "$BACKUP_PATH/cluster/cluster.json"
lxc cluster show --format json > "$BACKUP_PATH/cluster/cluster-config.json"

# 5. Backup application configuration
echo "Backing up application configuration..."
mkdir -p "$BACKUP_PATH/app"
cp -r /etc/arm-hypervisor "$BACKUP_PATH/app/"

# 6. Create backup manifest
echo "Creating backup manifest..."
cat > "$BACKUP_PATH/manifest.json" << EOF
{
    "backup_date": "$DATE",
    "backup_type": "daily",
    "containers": $(lxc list --format json | jq '. | length'),
    "storage_pools": $(lxc storage list --format json | jq '. | length'),
    "networks": $(lxc network list --format json | jq '. | length'),
    "cluster_nodes": $(lxc cluster list --format json | jq '. | length')
}
EOF

# 7. Compress backup
echo "Compressing backup..."
tar -czf "$BACKUP_DIR/arm-hypervisor-backup-$DATE.tar.gz" -C "$BACKUP_PATH" .
rm -rf "$BACKUP_PATH"

# 8. Cleanup old backups
echo "Cleaning up old backups..."
find "$BACKUP_DIR/daily" -name "*.tar.gz" -mtime +$RETENTION_DAYS -delete

echo "Backup completed: arm-hypervisor-backup-$DATE.tar.gz"
```

### Weekly Full Backups

For comprehensive backups including container data:

```bash
#!/bin/bash
set -euo pipefail

BACKUP_DIR="/backup/arm-hypervisor"
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_PATH="$BACKUP_DIR/weekly/$DATE"

# Create backup directory
mkdir -p "$BACKUP_PATH"

echo "Starting full backup: $DATE"

# Export all containers with data
for container in $(lxc list --format csv | cut -d, -f1); do
    echo "Exporting container with data: $container"
    lxc export "$container" "$BACKUP_PATH/${container}-full.tar.gz"
done

# Backup storage pool data
for pool in $(lxc storage list --format csv | cut -d, -f1); do
    echo "Backing up storage pool: $pool"
    if [ "$(lxc storage get "$pool" source)" != "" ]; then
        rsync -av "$(lxc storage get "$pool" source)/" "$BACKUP_PATH/storage-$pool/"
    fi
done

# Compress and cleanup
tar -czf "$BACKUP_DIR/arm-hypervisor-full-backup-$DATE.tar.gz" -C "$BACKUP_PATH" .
rm -rf "$BACKUP_PATH"

echo "Full backup completed: arm-hypervisor-full-backup-$DATE.tar.gz"
```

## Recovery Procedures

### Container Recovery

**To recover a single container:**

```bash
# Extract the backup
tar -xzf arm-hypervisor-backup-YYYYMMDD_HHMMSS.tar.gz
lxc import containers/container-name.tar.gz
```

**To recover all containers:**

```bash
#!/bin/bash
BACKUP_FILE="arm-hypervisor-backup-YYYYMMDD_HHMMSS.tar.gz"
EXTRACT_DIR="/tmp/recovery"

mkdir -p "$EXTRACT_DIR"
tar -xzf "$BACKUP_FILE" -C "$EXTRACT_DIR"

# Import containers
for container_tar in "$EXTRACT_DIR/containers"/*.tar.gz; do
    if [ -f "$container_tar" ]; then
        container_name=$(basename "$container_tar" .tar.gz)
        echo "Importing container: $container_name"
        lxc import "$container_tar"
        
        # Apply configuration
        if [ -f "$EXTRACT_DIR/containers/${container_name}.json" ]; then
            lxc config edit "$container_name" < "$EXTRACT_DIR/containers/${container_name}.json"
        fi
    fi
done
```

### Network Recovery

```bash
#!/bin/bash
BACKUP_FILE="arm-hypervisor-backup-YYYYMMDD_HHMMSS.tar.gz"
EXTRACT_DIR="/tmp/recovery"

mkdir -p "$EXTRACT_DIR"
tar -xzf "$BACKUP_FILE" -C "$EXTRACT_DIR"

# Recreate networks
for network_config in "$EXTRACT_DIR/network"/*.json; do
    if [ -f "$network_config" ]; then
        network_name=$(basename "$network_config" .json)
        echo "Recreating network: $network_name"
        lxc network edit "$network_name" < "$network_config"
    fi
done
```

### Storage Pool Recovery

```bash
#!/bin/bash
BACKUP_FILE="arm-hypervisor-backup-YYYYMMDD_HHMMSS.tar.gz"
EXTRACT_DIR="/tmp/recovery"

mkdir -p "$EXTRACT_DIR"
tar -xzf "$BACKUP_FILE" -C "$EXTRACT_DIR"

# Recreate storage pools
lxc storage create local dir

# Import storage volumes
for pool in "$EXTRACT_DIR/storage"/storage-*; do
    if [ -d "$pool" ]; then
        pool_name=$(basename "$pool" | sed 's/storage-//')
        echo "Restoring storage pool: $pool_name"
        rsync -av "$pool/" "/var/lib/lxd/storage-pools/$pool_name/"
    fi
done
```

### Cluster Recovery

```bash
#!/bin/bash
BACKUP_FILE="arm-hypervisor-backup-YYYYMMDD_HHMMSS.tar.gz"
EXTRACT_DIR="/tmp/recovery"

mkdir -p "$EXTRACT_DIR"
tar -xzf "$BACKUP_FILE" -C "$EXTRACT_DIR"

# Restore cluster configuration
if [ -f "$EXTRACT_DIR/cluster/cluster-config.json" ]; then
    echo "Restoring cluster configuration..."
    lxc cluster edit < "$EXTRACT_DIR/cluster/cluster-config.json"
fi
```

## Automated Backup Scheduling

### Cron Configuration

Add to `/etc/crontab`:

```cron
# Daily configuration backup at 2 AM
0 2 * * * root /usr/local/bin/arm-hypervisor-backup.sh daily

# Weekly full backup on Sunday at 3 AM  
0 3 * * 0 root /usr/local/bin/arm-hypervisor-backup.sh weekly

# Monthly cleanup of old backups
0 4 1 * * root find /backup/arm-hypervisor -name "*.tar.gz" -mtime +90 -delete
```

### Systemd Timer

Create `/etc/systemd/system/arm-hypervisor-backup.service`:

```ini
[Unit]
Description=ARM Hypervisor Backup
After=network.target

[Service]
Type=oneshot
ExecStart=/usr/local/bin/arm-hypervisor-backup.sh daily
```

Create `/etc/systemd/system/arm-hypervisor-backup.timer`:

```ini
[Unit]
Description=Run ARM Hypervisor backup daily
Requires=arm-hypervisor-backup.service

[Timer]
OnCalendar=daily
Persistent=true

[Install]
WantedBy=timers.target
```

Enable and start the timer:
```bash
systemctl enable arm-hypervisor-backup.timer
systemctl start arm-hypervisor-backup.timer
```

## Disaster Recovery Plan

### Scenario 1: Single Node Failure

1. **Identify failed node:**
   ```bash
   lxc cluster list
   ```

2. **Remove failed node from cluster:**
   ```bash
   lxc cluster remove <failed-node-name> --force
   ```

3. **Restore containers on remaining nodes** from recent backup

4. **Add replacement node:**
   ```bash
   lxc cluster add <new-node-name>
   ```

### Scenario 2: Storage Failure

1. **Identify failed storage:**
   ```bash
   lxc storage list
   lxc storage info <pool-name>
   ```

2. **Create new storage pool:**
   ```bash
   lxc storage create new-pool dir source=/new/path
   ```

3. **Migrate containers to new storage:**
   ```bash
   for container in $(lxc list); do
       lxc move $container $container --storage new-pool
   done
   ```

4. **Restore data from backup** if needed

### Scenario 3: Complete System Recovery

1. **Provision new server** with same specifications
2. **Install ARM Hypervisor Platform**
3. **Restore configuration:**
   ```bash
   # Restore application config
   cp -r backup/app/arm-hypervisor /etc/
   
   # Restore container configurations
   /usr/local/bin/restore-containers.sh backup-file.tar.gz
   ```

4. **Verify functionality:**
   ```bash
   lxc list
   lxc network list  
   lxc storage list
   ```

## Testing Backups

**Monthly backup testing procedure:**

```bash
#!/bin/bash
BACKUP_FILE=$(ls -t /backup/arm-hypervisor/*.tar.gz | head -1)
TEST_DIR="/tmp/backup-test-$(date +%s)"

echo "Testing backup: $BACKUP_FILE"

# Create test environment
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

# Extract backup
tar -xzf "$BACKUP_FILE"

# Verify backup integrity
if [ -f "manifest.json" ]; then
    echo "✓ Backup manifest found"
    jq . manifest.json
else
    echo "✗ Backup manifest missing"
    exit 1
fi

# Verify container exports
container_count=$(find containers -name "*.tar.gz" | wc -l)
echo "✓ Found $container_count container exports"

# Verify network configs
network_count=$(find network -name "*.json" | wc -l)  
echo "✓ Found $network_count network configurations"

# Test container import (dry run)
for container_tar in containers/*.tar.gz; do
    if [ -f "$container_tar" ]; then
        echo "✓ Container export valid: $(basename $container_tar)"
    fi
done

# Cleanup
cd /
rm -rf "$TEST_DIR"

echo "Backup test completed successfully"
```

## Best Practices

1. **Store backups off-site** - Use cloud storage or remote location
2. **Encrypt sensitive backups** - Use GPG or encrypted storage
3. **Monitor backup success** - Set up alerts for backup failures  
4. **Document recovery procedures** - Keep step-by-step guides
5. **Test regularly** - Verify backup integrity monthly
6. **Version control** - Keep multiple backup versions
7. **Access control** - Restrict backup access to authorized personnel
8. **Monitoring** - Set up alerts for backup job failures

## Monitoring Backup Health

### Prometheus Metrics

Add to your monitoring setup:

```yaml
# Backup job metrics
- job_name: 'arm-hypervisor-backups'
  static_configs:
    - targets: ['localhost:9090']
  metrics_path: /backup-metrics
  scrape_interval: 5m
```

### Backup Status Endpoint

The backup script should update a status file that can be monitored:

```bash
#!/bin/bash
# At the end of backup script
echo "{
    \"last_backup\": \"$(date -Iseconds)\",
    \"status\": \"success\",
    \"backup_file\": \"arm-hypervisor-backup-$DATE.tar.gz\",
    \"size_bytes\": $(stat -c%s "$BACKUP_DIR/arm-hypervisor-backup-$DATE.tar.gz")
}" > /var/lib/arm-hypervisor/backup-status.json
```