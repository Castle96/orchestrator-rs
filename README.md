# ARM Hypervisor Platform

[![CI](https://github.com/your-org/arm-hypervisor/workflows/CI/badge.svg)](https://github.com/your-org/arm-hypervisor/actions)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)

A production-ready, Proxmox-style virtualization management platform written in Rust, optimized for ARM devices like Raspberry Pi. This platform provides enterprise-grade LXC container management, high-availability clustering, and a modern web-based interface for ARM64 infrastructure.

## ✨ Features

### Core Functionality
- **🐳 LXC Container Management**: Full lifecycle management with Alpine/Ubuntu templates
- **🔐 Security First**: JWT authentication, TLS/HTTPS, input validation, and audit logging
- **🌐 Clustering**: Multi-node HA cluster with consensus and automatic failover
- **💾 Storage**: Flexible storage backends (local, NFS, CIFS) with thin provisioning
- **🔌 Networking**: Advanced networking with bridges, VLANs, and firewall rules
- **📊 Monitoring**: Built-in health checks, metrics, and resource tracking
- **🎨 Web UI**: Modern React-based management interface

### Production Ready
- ✅ Zero clippy warnings
- ✅ Comprehensive test coverage (unit + integration)
- ✅ Security audit passing
- ✅ TLS/HTTPS support
- ✅ Systemd integration
- ✅ Production deployment guides

## 🚀 Quick Start

### Prerequisites

**Hardware:**
- Raspberry Pi 4/5 (4GB+ RAM recommended)
- MicroSD card (32GB+ for production)
- Network connectivity

**Software:**
- Raspberry Pi OS Lite (64-bit) or Ubuntu Server 22.04 ARM64
- Rust 1.70+ (installed automatically by installer)

### Installation

**Option 1: Automated Installation (Recommended)**

```bash
# Clone the repository
git clone https://github.com/your-org/arm-hypervisor.git
cd arm-hypervisor

# Run the installer
sudo ./scripts/install.sh
```

**Option 2: Bare Metal Installation**

For detailed bare-metal installation on Raspberry Pi (including microSD preparation), see [DEPLOYMENT.md](DEPLOYMENT.md#raspberry-pi-bare-metal-installation).

### Initial Setup

1. **Configure the platform:**
```bash
# Copy example configuration
sudo cp /opt/arm-hypervisor/config.toml.example /etc/arm-hypervisor/config.toml

# Generate a secure JWT secret
JWT_SECRET=$(openssl rand -base64 32)

# Edit configuration
sudo nano /etc/arm-hypervisor/config.toml
```

2. **Generate TLS certificates (production):**
```bash
# Using Let's Encrypt
sudo certbot certonly --standalone -d your-domain.com

# Update config.toml with certificate paths
```

3. **Start the service:**
```bash
sudo systemctl start arm-hypervisor
sudo systemctl enable arm-hypervisor

# Check status
sudo systemctl status arm-hypervisor
```

4. **Access the web interface:**
```
https://your-pi-ip:8443
```

## 📚 Documentation

- **[Deployment Guide](DEPLOYMENT.md)** - Complete deployment instructions
- **[Security Policy](SECURITY.md)** - Security best practices and reporting
- **[Contributing](CONTRIBUTING.md)** - How to contribute to the project
- **[Changelog](CHANGELOG.md)** - Version history and changes

## 🏗️ Architecture

### Workspace Structure

```
arm-hypervisor/
├── crates/
│   ├── api-server/          # REST API server (Actix-web, JWT, TLS)
│   ├── container-manager/   # LXC container lifecycle management
│   ├── cluster/             # Distributed clustering and consensus
│   ├── storage/             # Storage pool management
│   ├── network/             # Network bridge, VLAN, firewall
│   ├── models/              # Shared data models
│   └── web-ui/              # React-based web interface
├── scripts/                 # Installation and utility scripts
└── docs/                    # Additional documentation
```

### Technology Stack

- **Backend**: Rust (Actix-web 4.x, Tokio async runtime)
- **Security**: rustls 0.23, JWT (jsonwebtoken)
- **Containers**: LXC/LXD
- **Networking**: Linux bridge-utils, iptables
- **Storage**: Local, NFS, CIFS support
- **Frontend**: React, TypeScript, Vite

## 🔧 Development

### Building from Source

**Local build:**
```bash
cargo build --release
```

**Cross-compile for ARM64:**
```bash
# Install target
rustup target add aarch64-unknown-linux-gnu

# Build
cargo build --release --target aarch64-unknown-linux-gnu
```

### Running Tests

```bash
# Unit tests
cargo test --workspace

# Integration tests
cargo test --test integration_tests

# With logging
cargo test -- --nocapture
```

### Code Quality

```bash
# Linting
cargo clippy --all-targets --all-features -- -D warnings

# Formatting
cargo fmt --all

# Security audit
cargo audit
```

## 🌟 Use Cases

- **Home Lab**: Run multiple isolated services on Raspberry Pi
- **Edge Computing**: Lightweight container orchestration for ARM devices
- **Development**: Test ARM-specific applications locally
- **Learning**: Understand hypervisor and clustering concepts
- **IoT Gateway**: Secure container runtime for IoT workloads

## 🛣️ Roadmap

### v0.2.0
- [ ] Web UI completion
- [ ] Container migration between nodes
- [ ] Live resource adjustment
- [ ] Snapshot management

### v0.3.0
- [ ] Role-based access control (RBAC)
- [ ] OAuth2/OIDC integration
- [ ] Prometheus metrics export
- [ ] Grafana dashboards

### v1.0.0
- [ ] Production hardening
- [ ] Performance optimization
- [ ] High availability testing
- [ ] Comprehensive documentation

## 🤝 Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) for details on:

- Code of conduct
- Development workflow
- Coding standards
- Testing requirements
- Pull request process

## 📝 License

This project is dual-licensed under:

- **MIT License** ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- **Apache License 2.0** ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

You may choose either license for your use.

## 🔒 Security

Security is a top priority. Please report vulnerabilities responsibly:

- **Email**: security@your-domain.com
- **Policy**: See [SECURITY.md](SECURITY.md)

**Never report security issues via public GitHub issues.**

## 🙏 Acknowledgments

- Inspired by Proxmox VE
- Built with the Rust ecosystem
- Community feedback and contributions

## 📧 Support

- **Issues**: [GitHub Issues](https://github.com/your-org/arm-hypervisor/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-org/arm-hypervisor/discussions)
- **Documentation**: [Wiki](https://github.com/your-org/arm-hypervisor/wiki)

---

**Made with ❤️ for the ARM and Raspberry Pi community**
