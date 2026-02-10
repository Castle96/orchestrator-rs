#!/bin/bash

echo "=== LXC Operations Modification Guide ==="
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${YELLOW}Option 1: Passwordless Sudo Configuration${NC}"
echo "Add this to /etc/sudoers.d/lxc-user:"
echo "$(whoami) ALL=(ALL) NOPASSWD: /usr/bin/lxc-*"
echo ""
echo "Commands:"
echo "  echo '$(whoami) ALL=(ALL) NOPASSWD: /usr/bin/lxc-*' | sudo tee /etc/sudoers.d/lxc-user"
echo ""

echo -e "${YELLOW}Option 2: Run as Root User${NC}"
echo "Start the entire orchestrator as root:"
echo "  sudo cargo run --bin api-server"
echo ""

echo -e "${YELLOW}Option 3: Use Sudo with -S flag (pipe password)${NC}"
echo "Requires modifying the LXC command implementation to:"
echo "1. Read password from environment variable or file"
echo "2. Use sudo -S to read password from stdin"
echo ""

echo -e "${YELLOW}Option 4: User Namespaces (Rootless Containers)${NC}"
echo "Configure LXC to use user namespaces:"
echo "  lxc-create -t download -n container1 -- --dist ubuntu --release focal --arch amd64"
echo "  This allows unprivileged containers"
echo ""

echo -e "${YELLOW}Option 5: Docker/Podman Integration${NC}"
echo "Replace LXC with Docker/Podman for better permission handling"
echo "Docker has better user namespace support and socket-based access"
echo ""

echo -e "${YELLOW}Option 6: Setuid Wrapper${NC}"
echo "Create a setuid wrapper for LXC commands"
echo "More complex but provides granular control"
echo ""

echo -e "${BLUE}=== Code Modifications ===${NC}"
echo ""

echo -e "${YELLOW}1. Modify LXC Command Execution${NC}"
echo "Current implementation in crates/container-manager/src/lxc.rs"
echo ""

echo -e "${YELLOW}2. Add Environment Variable Support${NC}"
echo "Use LXC_SUDO_PASSWORD environment variable when needed"
echo ""

echo -e "${YELLOW}3. Fallback to Rootless Mode${NC}"
echo "Try unprivileged operations first, fall back to privileged"
echo ""

echo -e "${YELLOW}4. Add Permission Detection${NC}"
echo "Detect if running as root or has sudo access"
echo ""

echo "Choose the option that best fits your security requirements!"