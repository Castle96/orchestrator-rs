# ARM Hypervisor Platform - Production Deployment Guide

A comprehensive guide for deploying the ARM Hypervisor Platform in production environments.

## Raspberry Pi Bare Metal Installation

**For Raspberry Pi 4/5 running Raspberry Pi OS Lite (64-bit)**

This guide walks you through installing the ARM Hypervisor on a Raspberry Pi from scratch, including preparing the microSD card, building the software, and setting up for production use.

### Quick Start Overview

1. **Prepare microSD card** with Raspberry Pi OS Lite (64-bit)
2. **Boot and configure** the Raspberry Pi
3. **Install dependencies** (Rust, LXC, system packages)
4. **Build the application** on the Pi or cross-compile
5. **Configure and deploy** the service
6. **Test and verify** functionality

---

### Step 1: Prepare the microSD Card

#### Option A: Use Raspberry Pi Imager (Recommended)

**On your computer:**

1. **Download Raspberry Pi Imager**
   - Linux: `sudo apt install rpi-imager`
   - macOS: Download from https://www.raspberrypi.com/software/
   - Windows: Download from https://www.raspberrypi.com/software/

2. **Flash the OS**
   ```bash
   # Insert microSD card (16GB minimum, 64GB+ recommended)
   # Launch Raspberry Pi Imager
   ```
   
   - **Device:** Choose your Raspberry Pi model (Pi 4/5)
   - **OS:** Choose "Raspberry Pi OS Lite (64-bit)"
     - Navigate to: Raspberry Pi OS (other) → Raspberry Pi OS Lite (64-bit)
   - **Storage:** Select your microSD card

3. **Configure Settings** (Click the gear icon or "Edit Settings")
   ```
   Hostname: arm-hypervisor
   Enable SSH: ✓ (Use password authentication or SSH key)
   Username: admin
   Password: [set a strong password]
   WiFi: [configure if needed]
   Locale: [your timezone]
   ```

4. **Write to Card**
   - Click "Write" and wait for completion
   - Safely eject the card

#### Option B: Manual Setup (Advanced)

```bash
# Download Raspberry Pi OS Lite (64-bit)
wget https://downloads.raspberrypi.org/raspios_lite_arm64/images/raspios_lite_arm64-2024-11-19/2024-11-19-raspios-bookworm-arm64-lite.img.xz

# Find your SD card device
lsblk  # Look for your SD card (e.g., /dev/sdb, /dev/mmcblk0)

# Write image to SD card (REPLACE /dev/sdX with your device!)
sudo dd if=2024-11-19-raspios-bookworm-arm64-lite.img.xz of=/dev/sdX bs=4M status=progress conv=fsync
sudo sync

# Enable SSH (mount the boot partition and create ssh file)
# The boot partition will be auto-mounted, or:
sudo mkdir -p /mnt/boot
sudo mount /dev/sdX1 /mnt/boot
sudo touch /mnt/boot/ssh
sudo umount /mnt/boot
```

---

### Step 2: First Boot and Initial Setup

**Insert the microSD card into your Raspberry Pi and power it on.**

1. **Find the Pi's IP address**
   ```bash
   # On your network router, or:
   sudo nmap -sn 192.168.1.0/24 | grep -B 2 "Raspberry Pi"
   
   # Or if you set hostname:
   ping arm-hypervisor.local
   ```

2. **SSH into the Pi**
   ```bash
   ssh admin@192.168.1.XXX  # Replace with your Pi's IP
   # Or if hostname is set:
   ssh admin@arm-hypervisor.local
   ```

3. **Update the system**
   ```bash
   sudo apt update
   sudo apt upgrade -y
   sudo apt autoremove -y
   ```

4. **Configure basic settings**
   ```bash
   # Run configuration tool
   sudo raspi-config
   
   # Recommended settings:
   # 1. System Options → Boot/Auto Login → Console (no auto-login)
   # 2. Performance Options → GPU Memory → Set to 16MB (minimal for headless)
   # 3. Localisation Options → Set timezone
   # 4. Advanced Options → Expand Filesystem (if not auto-expanded)
   
   # Reboot after changes
   sudo reboot
   ```

---

### Step 3: Install System Dependencies

**SSH back into the Pi after reboot:**

```bash
ssh admin@arm-hypervisor.local
```

#### Install LXC and System Packages

```bash
# Install LXC container runtime
sudo apt install -y \
    lxc \
    lxc-templates \
    bridge-utils \
    iproute2 \
    iptables \
    debootstrap \
    dnsmasq-base \
    libvirt-clients \
    libvirt-daemon-system

# Install build tools and utilities
sudo apt install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    git \
    curl \
    wget \
    ca-certificates \
    openssl

# Verify LXC installation
lxc-checkconfig
# You should see mostly "enabled" entries
```

#### Install Rust

```bash
# Install Rust using rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Choose option 1 (default installation)

# Load Rust environment
source "$HOME/.cargo/env"

# Verify installation
rustc --version
cargo --version

# Add to shell profile for future logins
echo 'source "$HOME/.cargo/env"' >> ~/.bashrc
```

---

### Step 4: Build the ARM Hypervisor

#### Option A: Build Directly on the Raspberry Pi (Recommended)

**This takes 15-30 minutes on a Raspberry Pi 4**

```bash
# Clone the repository
cd ~
git clone https://github.com/your-org/arm-hypervisor.git
cd arm-hypervisor

# Build in release mode (optimized)
cargo build --release

# This will take a while - grab a coffee ☕
# On Raspberry Pi 4: ~20 minutes
# On Raspberry Pi 5: ~10 minutes

# Verify the build
ls -lh target/release/api-server
file target/release/api-server
# Should show: aarch64 ELF executable
```

**To monitor build progress and resource usage:**
```bash
# In another SSH session:
watch -n 1 'ps aux | grep cargo && free -h'
```

#### Option B: Cross-Compile from x86_64 Linux (Faster)

**On your x86_64 Linux development machine:**

```bash
# Install cross-compilation tools
sudo apt install -y gcc-aarch64-linux-gnu

# Add ARM64 target to Rust
rustup target add aarch64-unknown-linux-gnu

# Configure cargo for cross-compilation
mkdir -p ~/.cargo
cat >> ~/.cargo/config.toml << 'EOF'
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
EOF

# Clone and build
git clone https://github.com/your-org/arm-hypervisor.git
cd arm-hypervisor

# Cross-compile for ARM64
cargo build --release --target aarch64-unknown-linux-gnu

# Binary will be at: target/aarch64-unknown-linux-gnu/release/api-server

# Transfer to Raspberry Pi
scp target/aarch64-unknown-linux-gnu/release/api-server \
    admin@arm-hypervisor.local:~/arm-hypervisor-binary

# On the Pi, move it to the project directory
ssh admin@arm-hypervisor.local
mkdir -p ~/arm-hypervisor/target/release
mv ~/arm-hypervisor-binary ~/arm-hypervisor/target/release/api-server
chmod +x ~/arm-hypervisor/target/release/api-server
```

---

### Step 5: Install and Configure

**Back on the Raspberry Pi:**

```bash
cd ~/arm-hypervisor

# Create necessary directories
sudo mkdir -p /etc/arm-hypervisor
sudo mkdir -p /var/lib/arm-hypervisor/storage/default
sudo mkdir -p /var/log/arm-hypervisor

# Set permissions
sudo chown -R root:root /var/lib/arm-hypervisor
sudo chown -R root:root /var/log/arm-hypervisor
sudo chmod 755 /var/lib/arm-hypervisor
sudo chmod 755 /var/log/arm-hypervisor

# Install the binary
sudo cp target/release/api-server /usr/local/bin/arm-hypervisor
sudo chmod +x /usr/local/bin/arm-hypervisor

# Verify installation
/usr/local/bin/arm-hypervisor --version || echo "Binary installed successfully"
```

#### Create Configuration File

```bash
# Copy example config
sudo cp config.toml.example /etc/arm-hypervisor/config.toml

# Generate a secure JWT secret
JWT_SECRET=$(openssl rand -base64 32)

# Edit configuration
sudo nano /etc/arm-hypervisor/config.toml
```

**Update these critical settings:**

```toml
[server]
host = "0.0.0.0"  # Listen on all interfaces
port = 8080

[cluster]
node_name = "rpi-node-1"  # Unique name for this Pi
bind_address = "0.0.0.0"
bind_port = 7946

[storage]
base_path = "/var/lib/arm-hypervisor/storage"
default_pool = "default"

[logging]
level = "info"
file = "/var/log/arm-hypervisor/hypervisor.log"

[security]
auth_enabled = true
# jwt_secret will be set via environment variable
cors_origins = ["http://192.168.1.XXX:3000"]  # Replace with your network
```

**Save and exit** (Ctrl+X, Y, Enter)

#### Set JWT Secret as Environment Variable

```bash
# Create environment file
sudo mkdir -p /etc/default
echo "JWT_SECRET=$(openssl rand -base64 32)" | sudo tee /etc/default/arm-hypervisor

# Secure the file
sudo chmod 600 /etc/default/arm-hypervisor
```

---

### Step 6: Setup Systemd Service

```bash
# Copy and customize the service file
sudo cp arm-hypervisor.service.example /etc/systemd/system/arm-hypervisor.service

# Edit if needed
sudo nano /etc/systemd/system/arm-hypervisor.service
```

**The service file should look like:**

```ini
[Unit]
Description=ARM Hypervisor Platform
Documentation=https://github.com/arm-hypervisor/arm-hypervisor
After=network-online.target
Wants=network-online.target
RequiresMountsFor=/var/lib/arm-hypervisor

[Service]
Type=exec
User=root
Group=root
ExecStart=/usr/local/bin/arm-hypervisor
Restart=always
RestartSec=10

# Environment
Environment=RUST_LOG=info
EnvironmentFile=/etc/default/arm-hypervisor

# Working directory
WorkingDirectory=/var/lib/arm-hypervisor

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=arm-hypervisor

[Install]
WantedBy=multi-user.target
```

**Enable and start the service:**

```bash
# Reload systemd
sudo systemctl daemon-reload

# Enable service to start on boot
sudo systemctl enable arm-hypervisor

# Start the service
sudo systemctl start arm-hypervisor

# Check status
sudo systemctl status arm-hypervisor
```

---

### Step 7: Verify Installation

#### Check Service Status

```bash
# View service status
sudo systemctl status arm-hypervisor

# View logs
sudo journalctl -u arm-hypervisor -f

# You should see:
# "Starting ARM Hypervisor API server"
# "Server config: 0.0.0.0:8080"
# "ARM Hypervisor API server started successfully"
```

#### Test API Endpoints

```bash
# Test health check
curl http://localhost:8080/health

# Test cluster status
curl http://localhost:8080/api/v1/cluster/status

# Test container listing (may need auth)
curl http://localhost:8080/api/v1/containers
```

#### Check LXC Integration

```bash
# Verify LXC is working
sudo lxc-ls
sudo lxc-checkconfig

# Create a test container (optional)
sudo lxc-create -t download -n test-container -- \
    --dist alpine --release 3.19 --arch arm64

# List containers via API
curl http://localhost:8080/api/v1/containers
```

---

### Step 8: Network Access

#### Configure Firewall (if using ufw)

```bash
# Install firewall
sudo apt install -y ufw

# Allow SSH
sudo ufw allow 22/tcp

# Allow API server
sudo ufw allow 8080/tcp

# Allow cluster communication
sudo ufw allow 7946/tcp
sudo ufw allow 7946/udp

# Enable firewall
sudo ufw enable

# Check status
sudo ufw status
```

#### Access from Your Network

```bash
# Find your Pi's IP
hostname -I

# From another computer on your network:
curl http://192.168.1.XXX:8080/health
# Replace XXX with your Pi's IP

# Or use the hostname:
curl http://arm-hypervisor.local:8080/health
```

---

### Step 9: Optional - Install Web UI

**If you want the web interface:**

```bash
# Install Node.js
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs

# Build web UI
cd ~/arm-hypervisor/crates/web-ui
npm install
npm run build

# Serve with nginx or use the built-in server
# Option 1: Use a simple HTTP server for testing
python3 -m http.server 3000 --directory dist &

# Option 2: Install nginx for production
sudo apt install -y nginx
sudo cp dist/* /var/www/html/
sudo systemctl restart nginx
```

**Access web UI:**
- http://arm-hypervisor.local:3000 (if using python server)
- http://arm-hypervisor.local (if using nginx)

---

### Performance Optimization for Raspberry Pi

#### Recommended Settings for Raspberry Pi 4

```bash
# Edit /boot/config.txt
sudo nano /boot/firmware/config.txt

# Add these lines:
# Increase GPU memory for headless operation
gpu_mem=16

# Enable hardware random number generator
dtparam=random=on

# Overclock (optional, use with good cooling)
# over_voltage=2
# arm_freq=1800

# Save and reboot
sudo reboot
```

#### Optimize Storage Performance

```bash
# If using USB SSD instead of microSD (recommended for production)
# 1. Clone the SD card to SSD:
sudo dd if=/dev/mmcblk0 of=/dev/sda bs=4M status=progress

# 2. Modify /boot/cmdline.txt to boot from USB
# Change root=/dev/mmcblk0p2 to root=/dev/sda2

# Enable fstrim for SSD
sudo systemctl enable fstrim.timer
```

---

### Troubleshooting Common Issues

#### Service Won't Start

```bash
# Check detailed logs
sudo journalctl -u arm-hypervisor -n 100 --no-pager

# Check configuration
sudo /usr/local/bin/arm-hypervisor --help

# Test configuration manually
cd /var/lib/arm-hypervisor
JWT_SECRET="test-secret-at-least-32-chars-long" sudo /usr/local/bin/arm-hypervisor
```

#### LXC Containers Won't Start

```bash
# Check LXC configuration
sudo lxc-checkconfig

# Check cgroup support
grep cgroup /proc/mounts

# Enable legacy cgroups if needed (older Raspberry Pi OS)
sudo nano /boot/cmdline.txt
# Add: systemd.unified_cgroup_hierarchy=0
```

#### Out of Memory

```bash
# Add swap space
sudo dphys-swapfile swapoff
sudo nano /etc/dphys-swapfile
# Set CONF_SWAPSIZE=2048 (2GB)
sudo dphys-swapfile setup
sudo dphys-swapfile swapon

# Monitor memory
free -h
watch -n 1 free -h
```

#### Network Issues

```bash
# Check network interfaces
ip addr show

# Test cluster binding
sudo netstat -tulpn | grep 7946

# Check API server
sudo netstat -tulpn | grep 8080
```

---

### Monitoring and Maintenance

#### Setup Log Rotation

```bash
# Create logrotate config
sudo nano /etc/logrotate.d/arm-hypervisor

# Add:
/var/log/arm-hypervisor/*.log {
    daily
    rotate 7
    compress
    delaycompress
    notifempty
    create 0640 root root
    sharedscripts
    postrotate
        systemctl reload arm-hypervisor > /dev/null 2>&1 || true
    endscript
}
```

#### Automatic Updates

```bash
# Setup automatic security updates
sudo apt install -y unattended-upgrades
sudo dpkg-reconfigure -plow unattended-upgrades
```

#### Monitoring Script

```bash
# Create simple monitoring script
cat > ~/monitor.sh << 'EOF'
#!/bin/bash
echo "=== ARM Hypervisor Status ==="
echo "Service: $(systemctl is-active arm-hypervisor)"
echo "Memory: $(free -h | grep Mem | awk '{print $3 "/" $2}')"
echo "CPU Temp: $(vcgencmd measure_temp)"
echo "Uptime: $(uptime -p)"
echo "Containers: $(sudo lxc-ls | wc -l)"
echo "API Health: $(curl -s http://localhost:8080/health || echo "FAIL")"
EOF

chmod +x ~/monitor.sh

# Run it
./monitor.sh

# Add to crontab for regular checks
(crontab -l 2>/dev/null; echo "*/5 * * * * /home/admin/monitor.sh >> /var/log/arm-hypervisor/monitor.log 2>&1") | crontab -
```

---

### Backup Strategy

```bash
# Backup configuration
sudo tar -czf ~/arm-hypervisor-backup-$(date +%Y%m%d).tar.gz \
    /etc/arm-hypervisor \
    /etc/default/arm-hypervisor \
    /etc/systemd/system/arm-hypervisor.service \
    /var/lib/arm-hypervisor

# Copy to another machine
scp ~/arm-hypervisor-backup-*.tar.gz user@backup-server:/backups/

# Or backup the entire SD card periodically from your computer
sudo dd if=/dev/sdX of=~/rpi-backup-$(date +%Y%m%d).img bs=4M status=progress
```

---

### Production Checklist

Before going to production on your Raspberry Pi:

- [ ] Raspberry Pi OS updated to latest version
- [ ] Strong password set or SSH key authentication configured
- [ ] SSH port changed from default 22 (optional security)
- [ ] Firewall configured and enabled
- [ ] JWT secret generated and secured
- [ ] Configuration file reviewed and customized
- [ ] Service starts automatically on boot
- [ ] Logs are being written correctly
- [ ] Log rotation configured
- [ ] Monitoring script running
- [ ] Backup strategy implemented
- [ ] UPS or power backup recommended
- [ ] Good cooling solution (heatsink + fan)
- [ ] Using quality SD card (A2 rated) or USB SSD
- [ ] Network connectivity stable
- [ ] API endpoints responding correctly
- [ ] LXC containers can be created and managed

---

### Next Steps

Once your Raspberry Pi is running:

1. **Test container operations** - Create, start, stop containers
2. **Setup clustering** - Add more Raspberry Pis to create a cluster
3. **Deploy web UI** - Install and configure the management interface
4. **Setup monitoring** - Add Prometheus/Grafana for metrics
5. **Configure backups** - Automate regular backups
6. **Plan scaling** - Add more nodes as needed

**Congratulations! Your ARM Hypervisor is now running on bare metal Raspberry Pi!** 🎉

---

## Table of Contents

1. [Raspberry Pi Bare Metal Installation](#raspberry-pi-bare-metal-installation) **← START HERE FOR RASPBERRY PI**
2. [Prerequisites](#prerequisites)
3. [System Requirements](#system-requirements)
4. [Installation](#installation)
5. [Configuration](#configuration)
6. [Security Setup](#security-setup)
7. [Monitoring & Logging](#monitoring--logging)
8. [Backup & Recovery](#backup--recovery)
9. [Troubleshooting](#troubleshooting)
10. [Performance Tuning](#performance-tuning)
11. [Maintenance](#maintenance)

## Prerequisites

### Hardware Requirements

**Minimum Requirements:**
- ARM64 processor (Raspberry Pi 4 or equivalent)
- 4GB RAM
- 32GB storage
- Network interface

**Recommended for Production:**
- ARM64 multi-core processor (8+ cores)
- 16GB+ RAM
- 500GB+ SSD storage
- Multiple network interfaces
- UPS for power redundancy

### Software Requirements

**Operating System:**
- Ubuntu 22.04 LTS (ARM64) or newer
- Debian 12 (ARM64) or newer
- Raspberry Pi OS (64-bit)

**System Packages:**
- LXC/LXD runtime
- Docker (optional, for containerized deployment)
- systemd
- Network utilities (bridge-utils, iproute2, iptables)

## Installation

### Method 1: Using Installation Script (Recommended)

```bash
# Download the project
git clone https://github.com/your-org/arm-hypervisor.git
cd arm-hypervisor

# Run installation script
sudo ./scripts/install.sh
```

### Method 2: Manual Installation

```bash
# Install system dependencies
sudo apt update
sudo apt install -y lxc lxc-templates bridge-utils iproute2 iptables \
    curl wget systemd openssl ca-certificates

# Build from source
cargo build --target aarch64-unknown-linux-gnu --release

# Copy binary
sudo cp target/aarch64-unknown-linux-gnu/release/api-server /usr/local/bin/arm-hypervisor

# Create directories
sudo mkdir -p /etc/arm-hypervisor /var/lib/arm-hypervisor /var/log/arm-hypervisor

# Setup systemd service
sudo cp arm-hypervisor.service.example /etc/systemd/system/arm-hypervisor.service
sudo systemctl daemon-reload
sudo systemctl enable arm-hypervisor
```

### Method 3: Docker Deployment

```bash
# Build Docker image
docker build -t arm-hypervisor:latest .

# Run container
docker run -d \
  --name arm-hypervisor \
  --privileged \
  --network host \
  -v /var/lib/arm-hypervisor:/var/lib/arm-hypervisor \
  -v /etc/arm-hypervisor:/etc/arm-hypervisor \
  arm-hypervisor:latest
```

## Configuration

### Basic Configuration

Create `/etc/arm-hypervisor/config.toml`:

```toml
[server]
host = "0.0.0.0"
port = 8080
workers = 4

[database]
url = "sqlite:///var/lib/arm-hypervisor/database.db"

[cluster]
node_name = "node-1"
bind_address = "0.0.0.0"
bind_port = 7946

[storage]
base_path = "/var/lib/arm-hypervisor/storage"
default_pool = "default"

[[storage.pool_configs]]
name = "default"
storage_type = "local"
path = "/var/lib/arm-hypervisor/storage/default"

[network]
default_bridge = "lxcbr0"
firewall_enabled = true

[logging]
level = "info"
file = "/var/log/arm-hypervisor/hypervisor.log"

[security]
auth_enabled = true
jwt_secret = "CHANGE-THIS-SECRET-KEY"
```

### High Availability Configuration

For multi-node clusters:

```toml
[cluster]
node_name = "node-1"  # Unique per node
bind_address = "0.0.0.0"
bind_port = 7946
advertise_address = "192.168.1.100"  # This node's IP
join_addresses = ["192.168.1.101:7946", "192.168.1.102:7946"]
```

### Storage Configuration

#### Local Storage
```toml
[[storage.pool_configs]]
name = "fast-ssd"
storage_type = "local"
path = "/mnt/ssd/storage"
```

#### NFS Storage
```toml
[[storage.pool_configs]]
name = "shared-nfs"
storage_type = "nfs"
path = "nfs-server.example.com:/exports/storage"
```

#### CIFS/SMB Storage
```toml
[[storage.pool_configs]]
name = "shared-cifs"
storage_type = "cifs"
path = "//fileserver.example.com/storage"

[storage.pool_configs.options]
username = "storage_user"
domain = "example.com"
```

## Security Setup

### TLS Configuration

Generate certificates:

```bash
# Generate private key
openssl genrsa -out /etc/ssl/private/arm-hypervisor.key 4096

# Generate certificate signing request
openssl req -new -key /etc/ssl/private/arm-hypervisor.key \
    -out /etc/ssl/certs/arm-hypervisor.csr

# Generate self-signed certificate (for testing)
openssl req -x509 -key /etc/ssl/private/arm-hypervisor.key \
    -in /etc/ssl/certs/arm-hypervisor.csr \
    -out /etc/ssl/certs/arm-hypervisor.crt \
    -days 365
```

Update configuration:

```toml
[server.tls]
cert_file = "/etc/ssl/certs/arm-hypervisor.crt"
key_file = "/etc/ssl/private/arm-hypervisor.key"
```

### Authentication Setup

#### JWT Authentication
```toml
[security]
auth_enabled = true
jwt_secret = "your-256-bit-secret-key-here"
jwt_expiry = 86400  # 24 hours
```

#### API Key Authentication
```toml
[security]
api_keys = [
    "api-key-for-monitoring",
    "api-key-for-automation"
]
```

### Firewall Configuration

```bash
# Allow API access
sudo ufw allow 8080/tcp

# Allow cluster communication
sudo ufw allow 7946/tcp
sudo ufw allow 7946/udp

# Allow SSH
sudo ufw allow 22/tcp

# Enable firewall
sudo ufw enable
```

### Network Security

```bash
# Create isolated bridge for containers
sudo ip link add name hvbr0 type bridge
sudo ip addr add 172.16.100.1/24 dev hvbr0
sudo ip link set dev hvbr0 up

# Setup iptables rules
sudo iptables -A FORWARD -i hvbr0 -o hvbr0 -j ACCEPT
sudo iptables -A FORWARD -i hvbr0 ! -o hvbr0 -j ACCEPT
sudo iptables -A FORWARD -o hvbr0 -m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT
```

## Monitoring & Logging

### Health Monitoring

The platform exposes health endpoints:

```bash
# Check overall health
curl http://localhost:8080/health

# Check metrics
curl http://localhost:8080/metrics
```

### Log Configuration

#### Structured Logging
```toml
[logging]
level = "info"
format = "json"
file = "/var/log/arm-hypervisor/hypervisor.log"
rotate = true
max_files = 10
max_size = "100MB"
```

#### Log Aggregation with rsyslog
```bash
# Add to /etc/rsyslog.d/arm-hypervisor.conf
if $programname == 'arm-hypervisor' then {
    /var/log/arm-hypervisor/hypervisor.log
    stop
}
```

### Prometheus Integration

Metrics endpoint compatible with Prometheus:

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'arm-hypervisor'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/metrics'
    scrape_interval: 30s
```

### Alerting

Example alerts for critical conditions:

```yaml
# alerts.yml
groups:
  - name: arm-hypervisor
    rules:
      - alert: ServiceDown
        expr: up{job="arm-hypervisor"} == 0
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "ARM Hypervisor service is down"

      - alert: HighMemoryUsage
        expr: (memory_used_kb / memory_total_kb) > 0.9
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage detected"
```

## Backup & Recovery

### Configuration Backup

```bash
#!/bin/bash
# backup-config.sh
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/var/backups/arm-hypervisor"

mkdir -p "$BACKUP_DIR"

# Backup configuration
tar -czf "$BACKUP_DIR/config-$DATE.tar.gz" \
    /etc/arm-hypervisor/ \
    /etc/systemd/system/arm-hypervisor.service

# Backup database
cp /var/lib/arm-hypervisor/database.db "$BACKUP_DIR/database-$DATE.db"

# Cleanup old backups (keep 30 days)
find "$BACKUP_DIR" -name "*.tar.gz" -mtime +30 -delete
find "$BACKUP_DIR" -name "*.db" -mtime +30 -delete
```

### Container Data Backup

```bash
#!/bin/bash
# backup-containers.sh
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/var/backups/arm-hypervisor/containers"

mkdir -p "$BACKUP_DIR"

# Backup all container data
for container in $(lxc-ls); do
    echo "Backing up container: $container"
    tar -czf "$BACKUP_DIR/$container-$DATE.tar.gz" \
        "/var/lib/lxc/$container"
done
```

### Recovery Procedures

1. **Service Recovery:**
   ```bash
   sudo systemctl stop arm-hypervisor
   sudo tar -xzf config-backup.tar.gz -C /
   sudo systemctl start arm-hypervisor
   ```

2. **Database Recovery:**
   ```bash
   sudo systemctl stop arm-hypervisor
   sudo cp database-backup.db /var/lib/arm-hypervisor/database.db
   sudo chown arm-hypervisor:arm-hypervisor /var/lib/arm-hypervisor/database.db
   sudo systemctl start arm-hypervisor
   ```

## Troubleshooting

### Common Issues

#### Service Won't Start
```bash
# Check service status
sudo systemctl status arm-hypervisor

# Check logs
sudo journalctl -u arm-hypervisor -f

# Check configuration
sudo arm-hypervisor --config-check
```

#### Container Creation Fails
```bash
# Check LXC configuration
sudo lxc-checkconfig

# Check user namespaces
cat /etc/subuid
cat /etc/subgid

# Check bridge configuration
ip link show
brctl show
```

#### Network Issues
```bash
# Check bridge status
ip link show lxcbr0

# Check iptables rules
sudo iptables -L -n -v

# Check routing
ip route show
```

### Debug Mode

Enable debug logging:

```toml
[logging]
level = "debug"
```

### Performance Issues

Check system resources:

```bash
# CPU and memory usage
htop

# Disk usage
df -h
iostat -x 1

# Network traffic
iftop

# Container resource usage
lxc-top
```

## Performance Tuning

### System Optimizations

#### Kernel Parameters
```bash
# Add to /etc/sysctl.d/99-arm-hypervisor.conf
net.bridge.bridge-nf-call-iptables = 1
net.bridge.bridge-nf-call-ip6tables = 1
net.ipv4.ip_forward = 1
vm.max_map_count = 262144
fs.inotify.max_user_watches = 1048576
```

#### Resource Limits
```bash
# Add to /etc/security/limits.d/arm-hypervisor.conf
root soft nofile 65536
root hard nofile 65536
```

### Application Tuning

#### Worker Configuration
```toml
[server]
workers = 8  # Number of CPU cores
max_connections = 2000
keepalive = 60
```

#### Database Optimization
```toml
[database]
max_connections = 50
min_connections = 5
acquire_timeout = 30
idle_timeout = 600
```

## Maintenance

### Regular Tasks

#### Daily
- Check service health
- Monitor disk usage
- Review error logs

#### Weekly
- Update system packages
- Backup configurations
- Check container resource usage

#### Monthly
- Security updates
- Log rotation cleanup
- Performance review

### Update Procedures

1. **Backup current installation**
2. **Download new version**
3. **Stop service**
4. **Replace binary**
5. **Update configuration if needed**
6. **Start service**
7. **Verify functionality**

```bash
#!/bin/bash
# update-arm-hypervisor.sh
set -e

echo "Starting ARM Hypervisor update..."

# Backup
sudo systemctl stop arm-hypervisor
sudo cp /usr/local/bin/arm-hypervisor /usr/local/bin/arm-hypervisor.backup

# Update
sudo cp target/aarch64-unknown-linux-gnu/release/api-server /usr/local/bin/arm-hypervisor
sudo chmod +x /usr/local/bin/arm-hypervisor

# Start and verify
sudo systemctl start arm-hypervisor
sleep 5

if curl -f http://localhost:8080/health; then
    echo "Update successful!"
    sudo rm /usr/local/bin/arm-hypervisor.backup
else
    echo "Update failed, rolling back..."
    sudo systemctl stop arm-hypervisor
    sudo mv /usr/local/bin/arm-hypervisor.backup /usr/local/bin/arm-hypervisor
    sudo systemctl start arm-hypervisor
    exit 1
fi
```

### Scaling Considerations

#### Horizontal Scaling
- Add nodes to cluster
- Configure load balancing
- Implement shared storage

#### Vertical Scaling
- Increase memory allocation
- Add CPU cores
- Upgrade storage

## Support & Documentation

### Log Files
- Application logs: `/var/log/arm-hypervisor/`
- System logs: `journalctl -u arm-hypervisor`
- Container logs: `/var/log/lxc/`

### Configuration Files
- Main config: `/etc/arm-hypervisor/config.toml`
- Service file: `/etc/systemd/system/arm-hypervisor.service`
- LXC config: `/etc/lxc/`

### API Documentation
- Health endpoint: `GET /health`
- Metrics endpoint: `GET /metrics`
- API endpoints: `GET /api/v1/*`

For additional support, please refer to:
- GitHub repository: https://github.com/your-org/arm-hypervisor
- Documentation: https://docs.your-org.com/arm-hypervisor
- Support forum: https://forum.your-org.com