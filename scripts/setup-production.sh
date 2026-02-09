#!/bin/bash

# ARM Hypervisor Production Setup Script
# This script prepares a bare metal ARM device for production deployment

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
HYP_USER="arm-hypervisor"
HYP_HOME="/var/lib/arm-hypervisor"
HYP_CONFIG="/etc/arm-hypervisor"
HYP_LOG="/var/log/arm-hypervisor"

# Functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_root() {
    if [[ $EUID -ne 0 ]]; then
        log_error "This script must be run as root"
        exit 1
    fi
}

check_architecture() {
    ARCH=$(uname -m)
    if [[ "$ARCH" != "aarch64" ]]; then
        log_warning "Detected architecture: $ARCH. This is optimized for ARM64 (aarch64)"
        read -p "Continue anyway? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
}

install_dependencies() {
    log_info "Installing system dependencies..."
    
    apt update
    apt install -y \
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
        sqlite3 \
        certbot \
        nginx \
        htop \
        iotop \
        sysbench
    
    log_success "Dependencies installed"
}

setup_user() {
    log_info "Setting up hypervisor user..."
    
    if ! id "$HYP_USER" &>/dev/null; then
        useradd -m -s /bin/bash "$HYP_USER"
        log_success "Created user: $HYP_USER"
    else
        log_info "User $HYP_USER already exists"
    fi
    
    # Add to necessary groups
    usermod -aG sudo,lxd,netdev,docker "$HYP_USER" 2>/dev/null || true
    log_success "Added user to required groups"
}

setup_directories() {
    log_info "Creating application directories..."
    
    mkdir -p "$HYP_HOME"/{storage,containers,ssl}
    mkdir -p "$HYP_CONFIG"
    mkdir -p "$HYP_LOG"
    mkdir -p /opt/arm-hypervisor
    
    # Set permissions
    chown -R "$HYP_USER:$HYP_USER" "$HYP_HOME"
    chown -R "$HYP_USER:$HYP_USER" "$HYP_LOG"
    chown -R "$HYP_USER:$HYP_USER" /opt/arm-hypervisor
    
    # Config directory stays root-owned
    chown root:root "$HYP_CONFIG"
    chmod 755 "$HYP_CONFIG"
    
    log_success "Directories created and permissions set"
}

setup_networking() {
    log_info "Configuring LXC networking..."
    
    # Create bridge
    if ! ip link show lxcbr0 &>/dev/null; then
        brctl addbr lxcbr0
        ip addr add 192.168.100.1/24 dev lxcbr0
        ip link set lxcbr0 up
        log_success "Created lxcbr0 bridge"
    else
        log_info "lxcbr0 bridge already exists"
    fi
    
    # Configure LXC default
    mkdir -p /etc/lxc
    cat > /etc/lxc/default.conf <<'EOF'
lxc.net.0.type = veth
lxc.net.0.link = lxcbr0
lxc.net.0.flags = up
lxc.apparmor.profile = generated
lxc.apparmor.allow_nesting = 1
EOF
    
    log_success "LXC networking configured"
}

generate_secrets() {
    log_info "Generating secure secrets..."
    
    # Generate JWT secret
    JWT_SECRET=$(openssl rand -base64 32)
    echo "$JWT_SECRET" > "$HYP_CONFIG/jwt_secret"
    chmod 600 "$HYP_CONFIG/jwt_secret"
    chown root:root "$HYP_CONFIG/jwt_secret"
    
    log_success "Generated secure JWT secret"
}

generate_certificates() {
    log_info "Generating TLS certificates..."
    
    mkdir -p "$HYP_HOME/ssl"
    
    # Generate self-signed certificate
    openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
        -keyout "$HYP_HOME/ssl/server.key" \
        -out "$HYP_HOME/ssl/server.crt" \
        -subj "/C=US/ST=State/L=City/O=ARM Hypervisor/CN=$(hostname)" \
        2>/dev/null
    
    chmod 600 "$HYP_HOME/ssl/server.key"
    chown root:root "$HYP_HOME/ssl"/*
    
    log_success "TLS certificates generated"
    log_warning "For production, use Let's Encrypt certificates instead"
}

setup_config() {
    log_info "Setting up production configuration..."
    
    # Copy production config template
    cp "$(dirname "$0")/../config/production.toml" "$HYP_CONFIG/config.toml"
    
    # Update configuration with generated secrets
    if [[ -f "$HYP_CONFIG/jwt_secret" ]]; then
        JWT_SECRET=$(cat "$HYP_CONFIG/jwt_secret")
        sed -i "s/REPLACE_WITH_SECURE_RANDOM_32_CHAR_SECRET_MINIMUM/$JWT_SECRET/" "$HYP_CONFIG/config.toml"
    fi
    
    # Update paths
    sed -i "s|/etc/arm-hypervisor/ssl|$HYP_HOME/ssl|g" "$HYP_CONFIG/config.toml"
    
    # Set permissions
    chmod 640 "$HYP_CONFIG/config.toml"
    chown root:root "$HYP_CONFIG/config.toml"
    
    log_success "Production configuration setup complete"
}

setup_systemd() {
    log_info "Setting up systemd service..."
    
    cat > /etc/systemd/system/arm-hypervisor.service <<'EOF'
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

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/arm-hypervisor /var/log/arm-hypervisor

[Install]
WantedBy=multi-user.target
EOF
    
    systemctl daemon-reload
    systemctl enable arm-hypervisor
    
    log_success "Systemd service configured and enabled"
}

setup_firewall() {
    log_info "Configuring firewall..."
    
    if command -v ufw &> /dev/null; then
        ufw allow 22/tcp     # SSH
        ufw allow 8443/tcp   # HTTPS API
        ufw allow 7946/tcp   # Cluster communication
        ufw allow 67/udp     # DHCP for containers
        ufw allow 53/udp     # DNS for containers
        
        log_warning "Firewall rules added. Run 'ufw enable' to activate."
    else
        log_warning "ufw not found. Configure firewall manually."
    fi
}

run_tests() {
    log_info "Running basic functionality tests..."
    
    # Test LXC
    if ! lxc-checkconfig | grep -q "enabled"; then
        log_error "LXC not properly configured"
        return 1
    fi
    log_success "LXC configuration check passed"
    
    # Test bridge
    if ! ip link show lxcbr0 &>/dev/null; then
        log_error "lxcbr0 bridge not found"
        return 1
    fi
    log_success "Network bridge check passed"
    
    # Test certificate
    if [[ ! -f "$HYP_HOME/ssl/server.crt" ]]; then
        log_error "SSL certificate not found"
        return 1
    fi
    log_success "SSL certificate check passed"
    
    log_success "All basic tests passed"
}

show_summary() {
    log_success "Setup completed successfully!"
    echo
    echo "=== ARM Hypervisor Setup Summary ==="
    echo "User: $HYP_USER"
    echo "Home Directory: $HYP_HOME"
    echo "Config Directory: $HYP_CONFIG"
    echo "Log Directory: $HYP_LOG"
    echo "Bridge: lxcbr0 (192.168.100.1/24)"
    echo "Service: arm-hypervisor (enabled)"
    echo
    echo "=== Next Steps ==="
    echo "1. Build and deploy the application:"
    echo "   sudo -u $HYP_USER -i"
    echo "   cd /opt/arm-hypervisor"
    echo "   cargo build --release"
    echo "   cp target/release/* ."
    echo
    echo "2. Start the service:"
    echo "   sudo systemctl start arm-hypervisor"
    echo
    echo "3. Check status:"
    echo "   sudo systemctl status arm-hypervisor"
    echo "   curl -k https://localhost:8443/health"
    echo
    echo "4. Access Web UI:"
    echo "   https://$(hostname -I | awk '{print $1}'):8443"
    echo
    log_warning "Remember to replace the self-signed certificate with Let's Encrypt for production!"
}

# Main execution
main() {
    log_info "Starting ARM Hypervisor production setup..."
    
    check_root
    check_architecture
    
    # Ask user for confirmation
    echo
    echo "This script will:"
    echo "  - Install required dependencies"
    echo "  - Create hypervisor user and directories"
    echo "  - Configure networking and LXC"
    echo "  - Generate TLS certificates"
    echo "  - Setup systemd service"
    echo "  - Configure firewall"
    echo
    read -p "Continue with setup? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "Setup cancelled"
        exit 0
    fi
    
    # Run setup steps
    install_dependencies
    setup_user
    setup_directories
    setup_networking
    generate_secrets
    generate_certificates
    setup_config
    setup_systemd
    setup_firewall
    run_tests
    show_summary
}

# Run main function
main "$@"