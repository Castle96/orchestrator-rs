# ARM Hypervisor Deployment Verification Checklist

## Pre-Deployment Checklist ✅

### System Requirements
- [ ] ARM64 device (Raspberry Pi 4/5 or equivalent)
- [ ] Minimum 4GB RAM (8GB+ recommended for production)
- [ ] 32GB+ storage (256GB+ SSD recommended)
- [ ] Stable network connection
- [ ] Reliable power supply

### Operating System
- [ ] ARM64-compatible OS installed (Ubuntu Server 22.04 LTS recommended)
- [ ] System is up to date: `sudo apt update && sudo apt upgrade -y`
- [ ] SSH access configured and working
- [ ] User account with sudo privileges
- [ ] Static IP address configured (recommended for production)

## Deployment Checklist ✅

### 1. Dependencies Installation
```bash
# Run this and verify each component
sudo apt install -y build-essential pkg-config libssl-dev libclang-dev curl wget git lxc lxc-utils lxc-templates bridge-utils iptables dnsmasq sqlite3
```
- [ ] Build tools installed (gcc, make, etc.)
- [ ] OpenSSL development libraries
- [ ] LXC and LXC utilities installed
- [ ] Network tools installed (bridge-utils, iptables)
- [ ] Rust installed (via rustup or package manager)

### 2. User and Directory Setup
- [ ] Dedicated user created: `arm-hypervisor`
- [ ] User added to required groups: sudo, lxd, netdev
- [ ] Application directories created:
  - [ ] `/var/lib/arm-hypervisor/`
  - [ ] `/etc/arm-hypervisor/`
  - [ ] `/var/log/arm-hypervisor/`
  - [ ] `/opt/arm-hypervisor/`
- [ ] Proper permissions set on all directories

### 3. Network Configuration
- [ ] Bridge `lxcbr0` created and configured
- [ ] Bridge IP address assigned: 192.168.100.1/24
- [ ] LXC default configuration created: `/etc/lxc/default.conf`
- [ ] Network interfaces are up and reachable
- [ ] DNS resolution working

### 4. Application Build
- [ ] Source code cloned/available
- [ ] Rust application built: `cargo build --release`
- [ ] Web UI built: `npm run build`
- [ ] Binary files copied to `/opt/arm-hypervisor/`
- [ ] Static files copied to `/opt/arm-hypervisor/static/`

### 5. Configuration Setup
- [ ] Production config file created: `/etc/arm-hypervisor/config.toml`
- [ ] JWT secret generated and configured (32+ characters)
- [ ] Database path configured
- [ ] Logging configured with appropriate level
- [ ] Storage paths configured and exist

### 6. TLS/HTTPS Setup
- [ ] SSL certificates generated or obtained
- [ ] Certificate files in place: `/etc/arm-hypervisor/ssl/`
- [ ] Certificate permissions set correctly (600 for key)
- [ ] TLS configuration updated in config.toml
- [ ] Certificate validity verified

### 7. Systemd Service Setup
- [ ] Service file created: `/etc/systemd/system/arm-hypervisor.service`
- [ ] Service enabled: `systemctl enable arm-hypervisor`
- [ ] Service started: `systemctl start arm-hypervisor`
- [ ] Service status checked: `systemctl status arm-hypervisor`

### 8. Firewall Configuration
- [ ] Required ports opened:
  - [ ] 22/tcp (SSH)
  - [ ] 8443/tcp (HTTPS API)
  - [ ] 7946/tcp (Cluster communication)
  - [ ] 67/udp (DHCP for containers)
  - [ ] 53/udp (DNS for containers)
- [ ] Firewall rules saved and persistent

## Post-Deployment Verification ✅

### 1. Service Health Check
```bash
# Check service status
sudo systemctl status arm-hypervisor

# Check service logs
sudo journalctl -u arm-hypervisor -f

# Check API health
curl -k https://localhost:8443/health
```
- [ ] Service is running and active
- [ ] No critical errors in logs
- [ ] API responds to health check
- [ ] All services reported as healthy

### 2. Web UI Access
- [ ] Web UI accessible via HTTPS: https://device-ip:8443
- [ ] TLS certificate accepted (or warnings handled)
- [ ] All pages load without errors:
  - [ ] Dashboard
  - [ ] Containers
  - [ ] Cluster
  - [ ] Storage
  - [ ] Network
- [ ] Real-time updates working (check auto-refresh)

### 3. LXC Container Testing
```bash
# Run the test script
./scripts/test-containers.sh
```
- [ ] Container creation works via API
- [ ] Container listing works
- [ ] Container start/stop works
- [ ] Container deletion works
- [ ] Container status reporting works
- [ ] Direct LXC commands work

### 4. Storage and Network Testing
- [ ] Storage pools can be created
- [ ] Network bridges can be created
- [ ] Container networking works
- [ ] DNS resolution in containers works

### 5. Performance and Resource Testing
```bash
# System benchmarks
sysbench cpu --cpu-max-prime=20000 run
sysbench memory --memory-block-size=1K --memory-total-size=10G run
```
- [ ] System resources are adequate
- [ ] Memory usage is reasonable
- [ ] CPU usage is acceptable
- [ ] Disk performance is sufficient

## Production Readiness Checklist ✅

### Security
- [ ] TLS/HTTPS properly configured with valid certificates
- [ ] JWT secret is secure (random, 32+ characters)
- [ ] Authentication enabled in production
- [ ] Firewall rules configured and active
- [ ] Regular security updates planned
- [ ] Access logs being monitored
- [ ] Container isolation working properly

### Backup and Recovery
- [ ] Configuration backup plan in place
- [ ] Container backup strategy defined
- [ ] Database backup procedures documented
- [ ] Recovery procedures tested
- [ ] Backup storage secured

### Monitoring and Maintenance
- [ ] Log rotation configured
- [ ] Monitoring setup for system resources
- [ ] Alert thresholds configured
- [ ] Regular maintenance schedule defined
- [ ] Update procedures documented

### Documentation
- [ ] Deployment guide completed
- [ ] User documentation available
- [ ] API documentation accessible
- [ ] Troubleshooting guide created
- [ ] Contact information for support

## Optional Advanced Features ✅

### High Availability
- [ ] Multiple nodes configured
- [ ] Cluster communication working
- [ ] Node failover tested
- [ ] Shared storage configured
- [ ] Load balancing setup

### Advanced Networking
- [ ] VLANs configured
- [ ] Advanced firewall rules
- [ ] SDN features working
- [ ] Network templates created
- [ ] QoS policies configured

### Integration Features
- [ ] LDAP/AD authentication configured
- [ ] External monitoring integrated
- [ ] Backup solutions integrated
- [ ] API tokens configured
- [ ] Webhook notifications setup

## Troubleshooting Quick Reference

### Common Issues and Solutions

#### Service Won't Start
```bash
# Check logs
sudo journalctl -u arm-hypervisor -n 50

# Check configuration
sudo /opt/arm-hypervisor/api-server --config /etc/arm-hypervisor/config.toml

# Check permissions
ls -la /opt/arm-hypervisor/
ls -la /var/lib/arm-hypervisor/
```

#### LXC Permission Issues
```bash
# Check user groups
groups arm-hypervisor

# Check LXC config
lxc-checkconfig

# Fix permissions
sudo usermod -aG lxd arm-hypervisor
sudo chown -R arm-hypervisor:arm-hypervisor /var/lib/arm-hypervisor
```

#### Network Issues
```bash
# Check bridge
ip addr show lxcbr0

# Check routing
ip route show

# Restart networking
sudo systemctl restart networking
sudo ip link set lxcbr0 down && sudo ip link set lxcbr0 up
```

#### Certificate Issues
```bash
# Verify certificate
openssl x509 -in /etc/arm-hypervisor/ssl/server.crt -noout -dates

# Check key match
openssl x509 -noout -modulus -in /etc/arm-hypervisor/ssl/server.crt | openssl md5
openssl rsa -noout -modulus -in /etc/arm-hypervisor/ssl/server.key | openssl md5

# Regenerate certificates
./scripts/setup-tls.sh
```

## Final Verification ✅

### Full End-to-End Test
1. [ ] Create a test container via Web UI
2. [ ] Start the container
3. [ ] Verify container is accessible
4. [ ] Stop the container
5. [ ] Delete the container
6. [ ] Check that all resources are cleaned up

### Documentation Verification
- [ ] All documentation files present and accurate
- [ ] API examples work correctly
- [ ] Configuration examples tested
- [ ] Troubleshooting guide helpful

### Performance Verification
- [ ] System responsive under normal load
- [ ] Multiple containers can run simultaneously
- [ ] Web UI remains responsive
- [ ] API response times acceptable

---

## ✅ Deployment Complete!

If all items in this checklist are marked as complete, your ARM Hypervisor deployment is ready for production use. The system should now be providing a robust, Proxmox-like virtualization management platform optimized for ARM architectures.

### Next Steps:
1. Monitor the system for the first 24-48 hours
2. Set up regular backup schedules
3. Configure monitoring and alerting
4. Plan for regular maintenance windows
5. Document any customizations for your environment

**Congratulations! 🎉 You now have a fully functional ARM Hypervisor platform!**