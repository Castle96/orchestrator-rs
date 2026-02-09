# ARM Hypervisor Bare Metal Deployment Guide

## Hardware Requirements

### Minimum Requirements
- **ARM64 Device**: Raspberry Pi 4B (4GB+ RAM recommended) or equivalent
- **Storage**: 32GB+ microSD card or SSD (for production)
- **Network**: Ethernet connection (WiFi possible but less stable)
- **Power**: Reliable power supply

### Recommended Requirements
- **Device**: Raspberry Pi 5 (8GB RAM) or Rock Pi 5
- **Storage**: 256GB+ NVMe SSD via USB adapter or native SATA
- **RAM**: 8GB+ for production workloads
- **Network**: Gigabit Ethernet with reliable connectivity

## Prerequisites

### 1. OS Installation
```bash
# Flash Raspberry Pi OS Lite (64-bit) to microSD
# Enable SSH during first boot
# Boot and update system
sudo apt update && sudo apt upgrade -y

# Install required dependencies
sudo apt install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libclang-dev \
    curl \
    wget \
    git \
    lxc \
    lxc-utils \
    lxc-templates \
    bridge-utils \
    iptables \
    dnsmasq \
    nodejs \
    npm
```

### 2. User Setup
```bash
# Create hypervisor user (recommended)
sudo useradd -m -s /bin/bash arm-hypervisor
sudo usermod -aG sudo,lxd,netdev arm-hypervisor

# Switch to hypervisor user
sudo -u arm-hypervisor -i

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### 3. LXC Configuration
```bash
# Check LXC installation
lxc-checkconfig

# Create LXC directories
sudo mkdir -p /var/lib/lxc
sudo mkdir -p /var/lib/arm-hypervisor

# Set permissions
sudo chown -R arm-hypervisor:arm-hypervisor /var/lib/arm-hypervisor
sudo usermod -aG lxd arm-hypervisor

# Configure LXC networking
sudo brctl addbr lxcbr0 || echo "Bridge might already exist"
sudo ip addr add 192.168.100.1/24 dev lxcbr0
sudo ip link set lxcbr0 up

# Configure LXC default configuration
sudo mkdir -p /etc/lxc
sudo tee /etc/lxc/default.conf > /dev/null <<EOF
lxc.net.0.type = veth
lxc.net.0.link = lxcbr0
lxc.net.0.flags = up
lxc.apparmor.profile = generated
lxc.apparmor.allow_nesting = 1
EOF
```

## Deployment Steps

### 1. Build the Application
```bash
# Clone and build
git clone <repository-url> arm-hypervisor
cd arm-hypervisor

# Build for release (ARM64)
cargo build --release --target aarch64-unknown-linux-gnu

# Or build native if on ARM64
cargo build --release

# Build Web UI
cd crates/web-ui
export NVM_DIR="$HOME/.config/nvm"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
npm install
npm run build
```

### 2. Production Configuration
```bash
# Create production directories
sudo mkdir -p /etc/arm-hypervisor
sudo mkdir -p /var/lib/arm-hypervisor/storage
sudo mkdir -p /var/lib/arm-hypervisor/containers
sudo mkdir -p /var/log/arm-hypervisor

# Create production config
sudo tee /etc/arm-hypervisor/config.toml > /dev/null <<'EOF'
[server]
host = "0.0.0.0"
port = 8443
workers = 4

[database]
url = "sqlite:///var/lib/arm-hypervisor/hypervisor.db"

[cluster]
node_name = "arm-node-1"
bind_address = "0.0.0.0"
bind_port = 7946
join_addresses = []

[storage]
base_path = "/var/lib/arm-hypervisor/storage"
default_pool = "default"
pool_configs = [
    { name = "default", storage_type = "local", path = "/var/lib/arm-hypervisor/storage/default", options = {} }
]

[network]
default_bridge = "lxcbr0"
bridge_prefix = "hvbr"
ip_range = "192.168.100.0/24"
dns_servers = ["8.8.8.8", "8.8.4.4"]
firewall_enabled = true

[logging]
level = "info"
file = "/var/log/arm-hypervisor/hypervisor.log"
rotate = true
max_files = 10
max_size = "100MB"

[security]
auth_enabled = true
jwt_secret = "YOUR_SUPER_SECURE_JWT_SECRET_AT_LEAST_32_CHARACTERS_LONG"
jwt_expiry = 86400
api_keys = []
cors_origins = ["https://yourdomain.com"]
EOF

# Generate secure JWT secret
JWT_SECRET=$(openssl rand -base64 32)
sudo sed -i "s/YOUR_SUPER_SECURE_JWT_SECRET_AT_LEAST_32_CHARACTERS_LONG/$JWT_SECRET/" /etc/arm-hypervisor/config.toml
```

### 3. TLS/HTTPS Setup
```bash
# Option 1: Self-signed certificates (for testing)
sudo mkdir -p /etc/arm-hypervisor/ssl
sudo openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
    -keyout /etc/arm-hypervisor/ssl/server.key \
    -out /etc/arm-hypervisor/ssl/server.crt \
    -subj "/C=US/ST=State/L=City/O=Organization/CN=$(hostname)"

# Option 2: Let's Encrypt (for production with domain)
sudo apt install certbot
sudo certbot certonly --standalone -d yourdomain.com

# Update config for TLS
sudo tee -a /etc/arm-hypervisor/config.toml > /dev/null <<'EOF'

[server.tls]
cert_file = "/etc/arm-hypervisor/ssl/server.crt"
key_file = "/etc/arm-hypervisor/ssl/server.key"
EOF
```

### 4. Systemd Service Setup
```bash
# Create systemd service
sudo tee /etc/systemd/system/arm-hypervisor.service > /dev/null <<'EOF'
[Unit]
Description=ARM Hypervisor Platform
After=network.target lxc.service
Wants=network.target

[Service]
Type=simple
User=arm-hypervisor
Group=arm-hypervisor
WorkingDirectory=/opt/arm-hypervisor
ExecStart=/opt/arm-hypervisor/target/release/api-server
Restart=always
RestartSec=5
Environment="RUST_LOG=info"
Environment="JWT_SECRET_FILE=/etc/arm-hypervisor/jwt_secret"

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/arm-hypervisor /var/log/arm-hypervisor

[Install]
WantedBy=multi-user.target
EOF

# Install the application
sudo mkdir -p /opt/arm-hypervisor
sudo cp -r target/release/* /opt/arm-hypervisor/
sudo cp -r crates/web-ui/dist /opt/arm-hypervisor/static
sudo chown -R arm-hypervisor:arm-hypervisor /opt/arm-hypervisor

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable arm-hypervisor
sudo systemctl start arm-hypervisor
```

### 5. Firewall Configuration
```bash
# Configure firewall (if using ufw)
sudo ufw allow 22/tcp     # SSH
sudo ufw allow 8443/tcp   # HTTPS API
sudo ufw allow 7946/tcp   # Cluster communication
sudo ufw allow 67/udp     # DHCP for containers
sudo ufw allow 53/udp     # DNS for containers
sudo ufw enable
```

## Verification Checklist

### 1. Service Status
```bash
# Check service status
sudo systemctl status arm-hypervisor

# Check logs
sudo journalctl -u arm-hypervisor -f

# Check if API is responding
curl -k https://localhost:8443/health
```

### 2. Network Configuration
```bash
# Verify bridge exists
ip addr show lxcbr0

# Check LXC configuration
lxc-checkconfig

# Test container creation (should work now)
sudo lxc-create -t ubuntu -n test-container
sudo lxc-ls
```

### 3. Web UI Access
- Open browser to: https://your-device-ip:8443
- Accept self-signed certificate warning
- Login with authentication (if enabled)
- Verify all pages load correctly

### 4. Container Testing
```bash
# Test container creation via API
curl -k -X POST https://localhost:8443/api/v1/containers \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <your-jwt-token>" \
  -d '{
    "name": "test-ubuntu",
    "template": "ubuntu",
    "config": {
      "cpu_limit": 1,
      "memory_limit": 536870912,
      "disk_limit": 8589934592,
      "network_interfaces": [{"name": "eth0", "bridge": "lxcbr0"}],
      "rootfs_path": "",
      "environment": []
    }
  }'

# List containers
curl -k https://localhost:8443/api/v1/containers \
  -H "Authorization: Bearer <your-jwt-token>"
```

## Performance Testing

### 1. System Benchmarks
```bash
# CPU benchmark
sysbench cpu --cpu-max-prime=20000 run

# Memory benchmark
sysbench memory --memory-block-size=1K --memory-total-size=10G run

# Storage benchmark
sysbench fileio --file-total-size=1G --file-test-mode=rndrw prepare
sysbench fileio --file-total-size=1G --file-test-mode=rndrw run
```

### 2. Container Performance
```bash
# Create multiple containers
for i in {1..5}; do
  sudo lxc-create -t ubuntu -n test-$i
  sudo lxc-start -n test-$i
done

# Monitor resource usage
htop
iotop
sudo lxc-ls --fancy
```

## Troubleshooting

### Common Issues
1. **Permission Denied**: Ensure user is in lxd group and LXC paths are accessible
2. **Bridge Not Working**: Check bridge configuration and iptables rules
3. **TLS Certificate Issues**: Verify certificate paths and permissions
4. **Service Not Starting**: Check journalctl logs for detailed errors

### Log Locations
- **Service Logs**: `sudo journalctl -u arm-hypervisor`
- **Application Logs**: `/var/log/arm-hypervisor/hypervisor.log`
- **LXC Logs**: `/var/log/lxc/`

### Recovery Commands
```bash
# Restart service
sudo systemctl restart arm-hypervisor

# Reset LXC networking
sudo ip link set lxcbr0 down
sudo ip link set lxcbr0 up
sudo ip addr add 192.168.100.1/24 dev lxcbr0

# Emergency container cleanup
sudo lxc-stop -n test-container
sudo lxc-destroy -n test-container
```

## Production Considerations

### Security
- Use proper domain names with Let's Encrypt certificates
- Enable firewall rules
- Regular security updates
- Monitor logs for suspicious activity

### Backup Strategy
```bash
# Backup configuration
sudo tar -czf /backup/arm-hypervisor-config-$(date +%Y%m%d).tar.gz /etc/arm-hypervisor

# Backup containers
sudo lxc-copy -n container-name -N backup-container-name
sudo lxc-stop -n backup-container-name

# Backup database
sudo cp /var/lib/arm-hypervisor/hypervisor.db /backup/hypervisor-$(date +%Y%m%d).db
```

### Monitoring
- Set up monitoring for system resources
- Monitor API response times
- Alert on container failures
- Regular health checks