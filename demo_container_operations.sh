#!/bin/bash

echo "=== Orchestrator API Demo (Container Operations Simulation) ==="
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${YELLOW}🌐 Step 1: Check API Health${NC}"
HEALTH=$(curl -s http://localhost:8080/health)
echo "API Status: $(echo "$HEALTH" | jq -r '.status')"
echo "Container Manager: $(echo "$HEALTH" | jq -r '.services.container_manager.status')"
echo "Network Manager: $(echo "$HEALTH" | jq -r '.services.network_manager.status')"
echo ""

echo -e "${YELLOW}📋 Step 2: List Existing Containers${NC}"
CONTAINERS=$(curl -s http://localhost:8080/api/v1/containers)
echo "Total containers: $(echo "$CONTAINERS" | jq -r '.containers | length')"
if [ "$(echo "$CONTAINERS" | jq -r '.containers | length')" -gt 0 ]; then
    echo "Existing containers:"
    echo "$CONTAINERS" | jq -r '.containers[] | {name: .name, status: .status, template: .template}'
else
    echo "No containers found - this is expected in demo mode"
fi
echo ""

echo -e "${YELLOW}🏗️  Step 3: Simulate Container Creation Request${NC}"
echo "This shows the API request format (will fail due to permissions):"
CREATE_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/containers \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "nginx-demo",
    "template": "ubuntu",
    "config": {
      "cpu_limit": 1,
      "memory_limit": 536870912,
      "disk_limit": 10737418240,
      "network_interfaces": [
        {
          "name": "eth0",
          "bridge": "lxcbr0",
          "ipv4": "auto"
        }
      ],
      "rootfs_path": "/var/lib/lxc",
      "environment": [
        ["NGINX_PORT", "80"],
        ["SERVER_NAME", "nginx-demo"]
      ]
    }
  }')

if echo "$CREATE_RESPONSE" | grep -q "error"; then
    echo -e "${RED}❌ Container creation failed (expected due to permissions)${NC}"
    echo "Error: $(echo "$CREATE_RESPONSE" | jq -r '.error')"
    echo -e "${BLUE}ℹ️  Note: In production, the orchestrator would need root privileges for LXC operations${NC}"
else
    echo -e "${GREEN}✅ Container created successfully${NC}"
    CONTAINER_ID=$(echo "$CREATE_RESPONSE" | jq -r '.id')
    echo "Container ID: $CONTAINER_ID"
fi
echo ""

echo -e "${YELLOW}🔍 Step 4: Demonstrate Container Management API${NC}"
echo "Available container operations:"

echo -e "${BLUE}1. Create container:${NC}"
echo "POST /api/v1/containers"
echo "Body: {\"name\": \"container-name\", \"template\": \"ubuntu\", \"config\": {...}}"

echo -e "${BLUE}2. Start container:${NC}"
echo "POST /api/v1/containers/{name}/start"

echo -e "${BLUE}3. Stop container:${NC}"
echo "POST /api/v1/containers/{name}/stop"

echo -e "${BLUE}4. Execute command in container:${NC}"
echo "POST /api/v1/containers/{name}/exec"
echo "Body: {\"command\": \"apt-get update\"}"

echo -e "${BLUE}5. Get container info:${NC}"
echo "GET /api/v1/containers/{name}"

echo -e "${BLUE}6. List all containers:${NC}"
echo "GET /api/v1/containers"

echo -e "${BLUE}7. Delete container:${NC}"
echo "DELETE /api/v1/containers/{name}"
echo ""

echo -e "${YELLOW}📊 Step 5: Show Current System State${NC}"
echo "Container service status:"
curl -s http://localhost:8080/health | jq '.services'

echo ""
echo "Active containers:"
curl -s http://localhost:8080/api/v1/containers | jq '.containers | length'

echo ""

echo -e "${GREEN}🎉 Orchestrator API Demo Complete!${NC}"
echo ""
echo -e "${YELLOW}Production Setup Requirements:${NC}"
echo "1. Configure passwordless sudo for LXC commands:"
echo "   echo \"\$(whoami) ALL=(ALL) NOPASSWD: /usr/bin/lxc-*\" | sudo tee /etc/sudoers.d/lxc-user"
echo ""
echo -e "${YELLOW}Testing Options:${NC}"
echo "1. Test with proper sudo configuration (as shown above)"
echo "2. Use root user to run the orchestrator:"
echo "   sudo cargo run --bin api-server"
echo ""
echo -e "${YELLOW}Architecture Overview:${NC}"
echo "- API Server: Handles HTTP requests on port 8080"
echo "- Container Manager: Manages LXC container lifecycle"
echo "- Network Manager: Handles container networking"
echo "- Web UI: Frontend interface (development mode on port 5173)"