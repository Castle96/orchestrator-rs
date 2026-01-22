#!/bin/bash

# ARM Hypervisor Platform Installation Script
# Enhanced version with production features

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="/etc/arm-hypervisor"
DATA_DIR="/var/lib/arm-hypervisor"
LOG_DIR="/var/log/arm-hypervisor"
SERVICE_NAME="arm-hypervisor"
BINARY_NAME="api-server"
TARGET_ARCH="${TARGET_ARCH:-aarch64-unknown-linux-gnu}"

# Functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

check_root() {
    if [ "$EUID" -ne 0 ]; then
        log_error "This script must be run as root. Please use sudo or run as root user."
        exit 1
    fi
}

check_architecture() {
    local arch=$(uname -m)
    log_info "Detected architecture: $arch"

    case $arch in
        aarch64|arm64)
            log_success "ARM64 architecture detected - compatible with ARM Hypervisor"
            ;;
        x86_64|amd64)
            log_warn "x86_64 architecture detected - ARM Hypervisor is optimized for ARM64"
            read -p "Do you want to continue anyway? (y/N): " -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                exit 1
            fi
            TARGET_ARCH="x86_64-unknown-linux-gnu"
            ;;
        *)
            log_error "Unsupported architecture: $arch"
            exit 1
            ;;
    esac
}

detect_os() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        OS=$NAME
        VER=$VERSION_ID
        log_info "Detected OS: $OS $VER"
    else
        log_error "Cannot detect operating system"
        exit 1
    fi
}

install_dependencies() {
    log_info "Installing system dependencies..."

    # Update package list
    if command -v apt-get &> /dev/null; then
        apt-get update
        apt-get install -y \
            lxc \
            lxc-templates \
            bridge-utils \
            iproute2 \
            iptables \
            curl \
            wget \
            systemctl \
            openssl \
            ca-certificates
    elif command -v yum &> /dev/null; then
        yum update -y
        yum install -y \
            lxc \
            lxc-templates \
            bridge-utils \
            iproute2 \
            iptables \
            curl \
            wget \
            systemd \
            openssl \
            ca-certificates
    elif command -v pacman &> /dev/null; then
        pacman -Syu --noconfirm
        pacman -S --noconfirm \
            lxc \
            lxc-templates \
            bridge-utils \
            iproute2 \
            iptables \
            curl \
            wget \
            systemd \
            openssl \
            ca-certificates
    else
        log_error "Unsupported package manager. Please install dependencies manually:"
        echo "  - lxc and lxc-templates"
        echo "  - bridge-utils"
        echo "  - iproute2"
        echo "  - iptables"
        echo "  - systemd"
        exit 1
    fi

    log_success "Dependencies installed successfully"
}

check_lxc() {
    log_info "Checking LXC installation..."

    if ! command -v lxc-create &> /dev/null; then
        log_error "LXC is not properly installed or not in PATH"
        exit 1
    fi

    # Check if LXC can be used
    if ! lxc-checkconfig &> /dev/null; then
        log_warn "LXC configuration check failed. Some features may not work."
        log_info "Run 'lxc-checkconfig' manually to see detailed requirements"
    else
        log_success "LXC is properly configured"
    fi

    # Ensure lxc user namespace is enabled
    if [ ! -f /etc/subuid ] || [ ! -f /etc/subgid ]; then
        log_info "Setting up LXC user namespace mappings..."
        echo "root:100000:65536" >> /etc/subuid
        echo "root:100000:65536" >> /etc/subgid
    fi
}

create_directories() {
    log_info "Creating directories..."

    # Create main directories
    mkdir -p "$CONFIG_DIR"
    mkdir -p "$DATA_DIR"
    mkdir -p "$DATA_DIR/storage/default"
    mkdir -p "$DATA_DIR/containers"
    mkdir -p "$DATA_DIR/backups"
    mkdir -p "$LOG_DIR"

    # Set proper permissions
    chmod 755 "$CONFIG_DIR"
    chmod 755 "$DATA_DIR"
    chmod 755 "$LOG_DIR"

    log_success "Directories created successfully"
}

install_binary() {
    log_info "Installing ARM Hypervisor binary..."

    local binary_path="target/$TARGET_ARCH/release/$BINARY_NAME"

    if [ -f "$binary_path" ]; then
        log_info "Installing from local build..."
        cp "$binary_path" "$INSTALL_DIR/arm-hypervisor"
        chmod +x "$INSTALL_DIR/arm-hypervisor"
        log_success "Binary installed from local build"
    else
        log_warn "Local binary not found at $binary_path"
        log_info "Attempting to download pre-built binary..."

        # Download pre-built binary (this would be from releases)
        local download_url="https://github.com/arm-hypervisor/arm-hypervisor/releases/latest/download/arm-hypervisor-$TARGET_ARCH"

        if curl -L -o "$INSTALL_DIR/arm-hypervisor" "$download_url"; then
            chmod +x "$INSTALL_DIR/arm-hypervisor"
            log_success "Binary downloaded and installed"
        else
            log_error "Failed to download binary. Please build manually using:"
            echo "  cargo build --target $TARGET_ARCH --release"
            exit 1
        fi
    fi
}

create_config() {
    log_info "Creating configuration files..."

    if [ ! -f "$CONFIG_DIR/config.toml" ]; then
        if [ -f "config.toml.example" ]; then
            cp config.toml.example "$CONFIG_DIR/config.toml"
            log_info "Configuration template copied to $CONFIG_DIR/config.toml"
        else
            log_info "Creating basic configuration file..."
            cat > "$CONFIG_DIR/config.toml" << EOF
# ARM Hypervisor Platform Configuration
# Auto-generated during installation

[server]
host = "0.0.0.0"
port = 8080
workers = 4

[database]
url = "sqlite://$DATA_DIR/database.db"

[cluster]
node_name = "$(hostname)"
bind_address = "0.0.0.0"
bind_port = 7946

[storage]
base_path = "$DATA_DIR/storage"
default_pool = "default"

[[storage.pool_configs]]
name = "default"
storage_type = "local"
path = "$DATA_DIR/storage/default"

[network]
default_bridge = "lxcbr0"
bridge_prefix = "hvbr"
ip_range = "192.168.100.0/24"
dns_servers = ["8.8.8.8", "8.8.4.4"]
firewall_enabled = true

[logging]
level = "info"
file = "$LOG_DIR/hypervisor.log"

[security]
auth_enabled = false  # Disabled by default for initial setup
cors_origins = ["*"]
EOF
        fi

        chmod 600 "$CONFIG_DIR/config.toml"
        log_success "Configuration file created"
    else
        log_info "Configuration file already exists, skipping"
    fi
}

setup_systemd() {
    log_info "Setting up systemd service..."

    local service_file="/etc/systemd/system/$SERVICE_NAME.service"

    if [ -f "arm-hypervisor.service.example" ]; then
        cp arm-hypervisor.service.example "$service_file"
    else
        cat > "$service_file" << EOF
[Unit]
Description=ARM Hypervisor Platform
Documentation=https://github.com/arm-hypervisor/arm-hypervisor
After=network-online.target
Wants=network-online.target
RequiresMountsFor=$DATA_DIR

[Service]
Type=exec
User=root
Group=root
ExecStart=$INSTALL_DIR/arm-hypervisor
ExecReload=/bin/kill -HUP \$MAINPID
Restart=always
RestartSec=10
StartLimitInterval=0

# Security settings
NoNewPrivileges=false
PrivateTmp=true
ProtectSystem=strict
ReadWritePaths=$DATA_DIR $LOG_DIR /run/lxc
ProtectHome=true

# Resource limits
LimitNOFILE=1048576
LimitNPROC=infinity
TasksMax=infinity

# Environment
Environment=RUST_LOG=info
Environment=RUST_BACKTRACE=1

# Working directory
WorkingDirectory=$DATA_DIR

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=arm-hypervisor

# Capabilities needed for container and network management
AmbientCapabilities=CAP_NET_ADMIN CAP_SYS_ADMIN CAP_SETUID CAP_SETGID CAP_DAC_OVERRIDE CAP_FOWNER CAP_CHOWN

[Install]
WantedBy=multi-user.target
EOF
    fi

    # Reload systemd and enable service
    systemctl daemon-reload
    systemctl enable "$SERVICE_NAME"

    log_success "Systemd service configured and enabled"
}

setup_network() {
    log_info "Setting up network configuration..."

    # Check if lxcbr0 exists, create if not
    if ! ip link show lxcbr0 &> /dev/null; then
        log_info "Creating lxcbr0 bridge..."

        # Create LXC default configuration
        mkdir -p /etc/lxc
        if [ ! -f /etc/lxc/default.conf ]; then
            echo "lxc.network.type = veth" > /etc/lxc/default.conf
            echo "lxc.network.link = lxcbr0" >> /etc/lxc/default.conf
            echo "lxc.network.flags = up" >> /etc/lxc/default.conf
            echo "lxc.network.hwaddr = 00:16:3e:xx:xx:xx" >> /etc/lxc/default.conf
        fi

        # Start LXC networking
        if command -v lxc-net &> /dev/null; then
            systemctl enable lxc-net
            systemctl start lxc-net
        fi
    fi

    log_success "Network configuration completed"
}

post_install_checks() {
    log_info "Running post-installation checks..."

    # Verify binary is executable
    if [ -x "$INSTALL_DIR/arm-hypervisor" ]; then
        log_success "Binary is installed and executable"
    else
        log_error "Binary is not executable"
        exit 1
    fi

    # Test configuration loading
    if "$INSTALL_DIR/arm-hypervisor" --help &> /dev/null; then
        log_success "Binary can be executed"
    else
        log_warn "Binary execution test failed - may need additional dependencies"
    fi

    # Check service status
    if systemctl is-enabled "$SERVICE_NAME" &> /dev/null; then
        log_success "Service is enabled"
    else
        log_error "Service is not enabled"
    fi
}

show_completion_message() {
    echo
    log_success "ARM Hypervisor Platform installation completed!"
    echo
    echo -e "${BLUE}Installation Summary:${NC}"
    echo "  - Binary installed: $INSTALL_DIR/arm-hypervisor"
    echo "  - Configuration: $CONFIG_DIR/config.toml"
    echo "  - Data directory: $DATA_DIR"
    echo "  - Log directory: $LOG_DIR"
    echo "  - Service: $SERVICE_NAME"
    echo
    echo -e "${BLUE}Next Steps:${NC}"
    echo "  1. Review and customize the configuration:"
    echo "     sudo nano $CONFIG_DIR/config.toml"
    echo
    echo "  2. Start the service:"
    echo "     sudo systemctl start $SERVICE_NAME"
    echo
    echo "  3. Check service status:"
    echo "     sudo systemctl status $SERVICE_NAME"
    echo
    echo "  4. View logs:"
    echo "     sudo journalctl -u $SERVICE_NAME -f"
    echo
    echo "  5. Access the API:"
    echo "     curl http://localhost:8080/health"
    echo
    echo -e "${YELLOW}Security Note:${NC}"
    echo "  Authentication is disabled by default for initial setup."
    echo "  Enable it in the configuration file and set strong credentials."
    echo
}

# Main installation flow
main() {
    log_info "Starting ARM Hypervisor Platform installation..."

    check_root
    detect_os
    check_architecture
    install_dependencies
    check_lxc
    create_directories
    install_binary
    create_config
    setup_systemd
    setup_network
    post_install_checks
    show_completion_message

    log_success "Installation completed successfully!"
}

# Handle command line arguments
case "${1:-install}" in
    install)
        main
        ;;
    uninstall)
        log_info "Uninstalling ARM Hypervisor Platform..."
        systemctl stop "$SERVICE_NAME" 2>/dev/null || true
        systemctl disable "$SERVICE_NAME" 2>/dev/null || true
        rm -f "/etc/systemd/system/$SERVICE_NAME.service"
        rm -f "$INSTALL_DIR/arm-hypervisor"
        systemctl daemon-reload
        log_success "Uninstallation completed"
        echo "Note: Configuration and data directories preserved:"
        echo "  - $CONFIG_DIR"
        echo "  - $DATA_DIR"
        echo "  - $LOG_DIR"
        ;;
    --help|-h)
        echo "ARM Hypervisor Platform Installation Script"
        echo
        echo "Usage: $0 [command]"
        echo
        echo "Commands:"
        echo "  install     Install ARM Hypervisor Platform (default)"
        echo "  uninstall   Remove ARM Hypervisor Platform"
        echo "  --help      Show this help message"
        echo
        echo "Environment Variables:"
        echo "  TARGET_ARCH    Target architecture (default: aarch64-unknown-linux-gnu)"
        ;;
    *)
        log_error "Unknown command: $1"
        echo "Use '$0 --help' for usage information"
        exit 1
        ;;
esac
