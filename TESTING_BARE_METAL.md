# 🚀 ARM Hypervisor - Complete Bare Metal Testing Guide

## 📋 Overview

This guide provides everything you need to test and deploy the ARM Hypervisor on bare metal ARM hardware (Raspberry Pi, Rock Pi, etc.). It transforms the basic project into a production-ready Proxmox-like platform.

## 🎯 What You'll Have After Testing

- ✅ **Proxmox-like Web UI** with modern Material Design interface
- ✅ **Full Container Management** with LXC integration
- ✅ **Real-time Dashboard** with system metrics
- ✅ **Cluster Management** for multi-node setups  
- ✅ **Storage Management** with multiple pool support
- ✅ **Network Management** with bridge and interface control
- ✅ **TLS/HTTPS Security** with certificate management
- ✅ **Production-ready Configuration** with systemd services

## 🛠️ Quick Start Testing (15 minutes)

### Prerequisites
- ARM64 device with Ubuntu Server 22.04 LTS
- SSH access and sudo privileges
- Internet connection

### 1. Run Production Setup Script
```bash
# Download and run the automated setup
git clone <your-repo> arm-hypervisor
cd arm-hypervisor
sudo ./scripts/setup-production.sh
```

### 2. Build and Deploy
```bash
# Switch to hypervisor user
sudo -u arm-hypervisor -i

# Build the application
cd /opt/arm-hypervisor
cargo build --release

# Copy files to deployment location
sudo cp target/release/* /opt/arm-hypervisor/

# Build Web UI (from source)
cd /path/to/source/crates/web-ui
export NVM_DIR="$HOME/.config/nvm"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
npm install && npm run build
sudo cp -r dist/* /opt/arm-hypervisor/static/
```

### 3. Start Services
```bash
# Configure TLS (choose self-signed or Let's Encrypt)
sudo ./scripts/setup-tls.sh

# Start the hypervisor service
sudo systemctl start arm-hypervisor
sudo systemctl status arm-hypervisor
```

### 4. Test Everything
```bash
# Run comprehensive tests
./scripts/test-containers.sh

# Access the Web UI
# Open: https://your-device-ip:8443
```

## 📊 Testing Checklist

### 🔧 Basic Functionality Tests
- [ ] **API Server**: Responds to health checks
- [ ] **Web UI**: All pages load without errors
- [ ] **LXC Integration**: Direct container commands work
- [ ] **Network Setup**: Bridge configured and accessible
- [ ] **TLS/HTTPS**: Secure connection working

### 🏗️ Feature Tests
- [ ] **Container Creation**: Create containers via Web UI
- [ ] **Container Lifecycle**: Start/stop/delete containers
- [ ] **Resource Management**: CPU/Memory limits working
- [ ] **Storage Management**: Create and manage storage pools
- [ ] **Network Management**: Create bridges and interfaces
- [ ] **Real-time Updates**: Dashboard auto-refreshes

### 🔒 Security Tests
- [ ] **Authentication**: Login/logout functionality
- [ ] **TLS Certificate**: Valid and trusted
- [ ] **Firewall Rules**: Only required ports open
- [ ] **User Permissions**: Proper isolation and access control

### ⚡ Performance Tests
- [ ] **Multiple Containers**: Run 3-5 containers simultaneously
- [ ] **Resource Usage**: Monitor CPU/Memory under load
- [ ] **Web UI Responsiveness**: Interface remains snappy
- [ ] **API Response Times**: Queries complete quickly

## 🎯 Full Test Scenarios

### Scenario 1: Container Management Workflow
```bash
# 1. Create container via Web UI
# 2. Configure with 1 CPU, 512MB RAM, 8GB disk
# 3. Start the container
# 4. Verify it's running and accessible
# 5. Stop the container
# 6. Delete the container
# 7. Verify all resources cleaned up
```

### Scenario 2: Multi-Container Environment
```bash
# 1. Create 3 different containers (Ubuntu, Alpine, Debian)
# 2. Start all containers
# 3. Monitor resource usage
# 4. Test networking between containers
# 5. Stop and remove all containers
```

### Scenario 3: Storage and Network Testing
```bash
# 1. Create additional storage pool
# 2. Create network bridge with custom IP
# 3. Create container using custom resources
# 4. Verify isolation and connectivity
# 5. Clean up resources
```

## 🐛 Common Issues and Solutions

### Permission Issues
**Problem**: Container creation fails with permission denied
```bash
# Solution: Fix LXC permissions
sudo usermod -aG lxd arm-hypervisor
sudo chown -R arm-hypervisor:arm-hypervisor /var/lib/arm-hypervisor
```

### Network Issues
**Problem**: Containers can't access internet
```bash
# Solution: Configure iptables and bridge
sudo iptables -t nat -A POSTROUTING -o eth0 -j MASQUERADE
sudo sysctl -w net.ipv4.ip_forward=1
```

### Certificate Issues
**Problem**: TLS certificate errors
```bash
# Solution: Regenerate certificates
sudo ./scripts/setup-tls.sh
sudo systemctl restart arm-hypervisor
```

## 📈 Performance Benchmarks

### Expected Performance (Raspberry Pi 4B 8GB)
- **Container Creation**: 30-60 seconds
- **Container Start**: 5-10 seconds  
- **API Response**: <200ms
- **Web UI Load**: <2 seconds
- **Memory Usage**: 200-400MB base + containers
- **CPU Usage**: 5-15% idle, 30-80% under load

### Stress Testing
```bash
# Run performance test
sysbench cpu --cpu-max-prime=20000 run
sysbench memory --memory-block-size=1K --memory-total-size=10G run

# Monitor during test
htop
iotop
sudo lxc-ls --fancy
```

## 🔍 Verification Commands

### Service Health
```bash
# Check service status
sudo systemctl status arm-hypervisor

# Check logs
sudo journalctl -u arm-hypervisor -f

# Check API health
curl -k https://localhost:8443/health
```

### LXC Functionality
```bash
# Check LXC configuration
lxc-checkconfig

# List containers
sudo lxc-ls --fancy

# Test container creation
sudo lxc-create -t ubuntu -n test-container
sudo lxc-start -n test-container
sudo lxc-stop -n test-container
sudo lxc-destroy -n test-container
```

### Network Verification
```bash
# Check bridge
ip addr show lxcbr0

# Check routing
ip route show

# Test DNS
nslookup google.com 8.8.8.8
```

## 📚 Documentation and Resources

### Complete Documentation
- **Deployment Guide**: `DEPLOYMENT_BARE_METAL.md`
- **Configuration**: `config/production.toml`
- **Checklist**: `DEPLOYMENT_CHECKLIST.md`

### Scripts
- **Production Setup**: `scripts/setup-production.sh`
- **TLS Setup**: `scripts/setup-tls.sh`
- **Container Testing**: `scripts/test-containers.sh`

### Troubleshooting
- Check service logs: `sudo journalctl -u arm-hypervisor`
- Verify permissions: `ls -la /var/lib/arm-hypervisor/`
- Test API directly: `curl -k https://localhost:8443/health`

## 🎉 Success Criteria

Your deployment is **production-ready** when:

✅ **All Services Running**: API server and Web UI accessible  
✅ **Container Operations**: Create, start, stop, delete work  
✅ **Secure Access**: TLS/HTTPS with valid certificates  
✅ **Resource Management**: CPU, memory, storage limits enforced  
✅ **Network Functionality**: Containers can communicate externally  
✅ **Monitoring**: Logs and metrics available  
✅ **Performance**: Acceptable response times under load  
✅ **Security**: Proper authentication and firewall rules  

## 🚀 Next Steps After Testing

1. **Backup Configuration**: Save `/etc/arm-hypervisor/` to backup location
2. **Set Up Monitoring**: Configure alerts for system health
3. **Plan Updates**: Document update procedures
4. **User Training**: Document custom procedures for your environment
5. **Scale Out**: Add additional nodes for clustering

---

## 🏆 Final Result

You now have a **fully functional ARM Hypervisor platform** that rivals commercial solutions like Proxmox VE, specifically optimized for ARM architectures! 

**Key Achievements:**
- 🎯 **Proxmox-like interface** with modern design
- 🐳 **Complete container lifecycle management**
- 📊 **Real-time monitoring and metrics**
- 🔒 **Production security and TLS**
- 🏗️ **Scalable clustering architecture**
- 💾 **Flexible storage management**
- 🌐 **Advanced networking capabilities**

**Your ARM Hypervisor is ready for production use! 🎉**