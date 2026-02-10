#!/bin/bash

echo "=== Complete LXC Nginx Deployment Demo ==="
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${YELLOW}🌐 Step 1: Check API Health${NC}"
HEALTH=$(curl -s http://localhost:8080/health)
echo "$HEALTH" | jq -r '.status'
echo ""

echo -e "${YELLOW}📋 Step 2: List Existing Containers${NC}"
CONTAINERS=$(curl -s http://localhost:8080/api/v1/containers)
echo "$CONTAINERS" | jq -r '.containers | length'
echo "Existing containers:"
echo "$CONTAINERS" | jq -r '.containers[] | {name: .name, status: .status}'
echo ""

echo -e "${YELLOW}🏗️  Step 3: Create Nginx Container${NC}"
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
    echo -e "${RED}❌ Container creation failed${NC}"
    echo "$CREATE_RESPONSE"
else
    echo -e "${GREEN}✅ Container created successfully${NC}"
    CONTAINER_ID=$(echo "$CREATE_RESPONSE" | jq -r '.id')
    echo "Container ID: $CONTAINER_ID"
fi
echo ""

echo -e "${YELLOW}▶️  Step 4: Start Container${NC}"
if [ -n "$CONTAINER_ID" ]; then
    START_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/containers/nginx-demo/start)
    if echo "$START_RESPONSE" | grep -q "success"; then
        echo -e "${GREEN}✅ Container started successfully${NC}"
    else
        echo -e "${RED}❌ Failed to start container${NC}"
        echo "$START_RESPONSE"
    fi
fi
echo ""

echo -e "${YELLOW}⚙️  Step 5: Setup Nginx and Web Content${NC}"
# Install nginx
INSTALL_CMD=$(curl -s -X POST http://localhost:8080/api/v1/containers/nginx-demo/exec \
  -H 'Content-Type: application/json' \
  -d '{"command":"apt-get update && apt-get install -y nginx"}')

if echo "$INSTALL_CMD" | grep -q "executed"; then
    echo -e "${GREEN}✅ Nginx installation started${NC}"
else
    echo -e "${RED}❌ Failed to install nginx${NC}"
fi

# Create web page
WEBPAGE_CMD=$(curl -s -X POST http://localhost:8080/api/v1/containers/nginx-demo/exec \
  -H 'Content-Type: application/json' \
  -d '{
    "command":"echo \"<h1>Hello from Nginx in LXC Container!</h1><p>Server: nginx-demo</p><p>Time: $(date)</p><p>Accessed via: http://localhost:8080/api/v1/containers/nginx-demo</p>\" > /var/www/html/index.html"
  }')

if echo "$WEBPAGE_CMD" | grep -q "executed"; then
    echo -e "${GREEN}✅ Web page created${NC}"
else
    echo -e "${RED}❌ Failed to create web page${NC}"
fi

# Configure nginx
NGINX_CONFIG=$(curl -s -X POST http://localhost:8080/api/v1/containers/nginx-demo/exec \
  -H 'Content-Type: application/json' \
  -d '{
    "command":"echo \"server { listen 80; server_name _; location / { root /var/www/html; index index.html; } }\" > /etc/nginx/sites-available/default"
  }')

if echo "$NGINX_CONFIG" | grep -q "executed"; then
    echo -e "${GREEN}✅ Nginx configured${NC}"
else
    echo -e "${RED}❌ Failed to configure nginx${NC}"
fi

# Start nginx service
START_NGINX=$(curl -s -X POST http://localhost:8080/api/v1/containers/nginx-demo/exec \
  -H 'Content-Type: application/json' \
  -d '{"command":"systemctl start nginx"}')

if echo "$START_NGINX" | grep -q "executed"; then
    echo -e "${GREEN}✅ Nginx service started${NC}"
else
    echo -e "${RED}❌ Failed to start nginx${NC}"
fi
echo ""

echo -e "${YELLOW}🔍 Step 6: Verify Deployment${NC}"
echo "Container status after deployment:"
FINAL_STATUS=$(curl -s http://localhost:8080/api/v1/containers/nginx-demo)
echo "$FINAL_STATUS" | jq -r '{name: .name, status: .status, template: .template, config: {cpu: .config.cpu_limit, memory: .config.memory_limit}}'
echo ""

echo -e "${GREEN}🎉 Nginx Container Deployment Demo Complete!${NC}"
echo ""
echo -e "${YELLOW}Testing Options:${NC}"
echo "1. Test web access via container exec:"
echo "   curl -s -X POST http://localhost:8080/api/v1/containers/nginx-demo/exec \\"
echo "     -H 'Content-Type: application/json' \\"
echo "     -d '{\"command\":\"curl -I http://localhost\"}'"
echo ""
echo "2. Check all containers:"
echo "   curl -s http://localhost:8080/api/v1/containers | jq '.containers[] | {name, status}'"
echo ""

echo -e "${YELLOW}Cleanup Commands:${NC}"
echo "Stop container: curl -X POST http://localhost:8080/api/v1/containers/nginx-demo/stop"
echo "Delete container: curl -X DELETE http://localhost:8080/api/v1/containers/nginx-demo"
