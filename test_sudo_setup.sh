#!/bin/bash

echo "=== Ready to Test After Sudo Setup ==="
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${YELLOW}Waiting for you to run sudo setup...${NC}"
echo ""
echo "Once you've run either:"
echo "  ./setup_sudo.sh"
echo "  OR"
echo "  echo '$(whoami) ALL=(ALL) NOPASSWD: ALL' | sudo tee /etc/sudoers.d/opencode-full"
echo ""
echo -e "${GREEN}I will:${NC}"
echo "✅ Restart the orchestrator"
echo "✅ Test container creation"
echo "✅ Run the full deployment demo"
echo "✅ Show nginx working in LXC"
echo ""
echo -e "${YELLOW}Press any key when you're ready, or run 'test_sudo_setup.sh'${NC}"
echo ""

# Test if sudo is working
if sudo -n true 2>/dev/null; then
    echo -e "${GREEN}🎉 Sudo access detected! Starting tests...${NC}"
    exec ./test_full_deployment.sh
else
    echo -e "${RED}❌ Sudo not yet configured${NC}"
    echo "Please run the sudo setup commands first."
fi