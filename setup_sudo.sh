#!/bin/bash

echo "=== Setting Up Passwordless Sudo for LXC ==="
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

CURRENT_USER=$(whoami)

echo -e "${YELLOW}Creating passwordless sudo configuration...${NC}"

# Create the sudoers file
echo "${CURRENT_USER} ALL=(ALL) NOPASSWD: /usr/bin/lxc-*" | sudo tee /etc/sudoers.d/lxc-${CURRENT_USER} > /dev/null

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Passwordless sudo configured successfully!${NC}"
    echo "User: ${CURRENT_USER}"
    echo "Commands: /usr/bin/lxc-*"
    echo "Config file: /etc/sudoers.d/lxc-${CURRENT_USER}"
else
    echo -e "${RED}❌ Failed to configure sudo${NC}"
    exit 1
fi

echo ""

echo -e "${YELLOW}Testing sudo access...${NC}"
if sudo -n /usr/bin/lxc-ls > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Sudo access confirmed!${NC}"
else
    echo -e "${RED}❌ Sudo test failed${NC}"
    echo "You may need to:"
    echo "1. Log out and log back in"
    echo "2. Or restart your shell session"
fi

echo ""

echo -e "${YELLOW}To remove this configuration later:${NC}"
echo "sudo rm /etc/sudoers.d/lxc-${CURRENT_USER}"

echo ""

echo -e "${GREEN}🎉 Setup complete! You can now run the orchestrator without root privileges.${NC}"