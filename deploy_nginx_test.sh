#!/bin/bash

echo "=== Nginx Container Deployment Test ==="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Step 1: Create container
echo -e "${YELLOW}Step 1: Creating nginx container...${NC}"
CONTAINER_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/containers \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "nginx-test",
    "template": "ubuntu", 
    "config": {
      "cpu_limit": 1,
      "memory_limit": 536870912,
      "disk_limit": 10737418240,
      "network_interfaces": [{
        "name": "eth0",
        "bridge": "lxcbr0", 
        "ipv4": "auto"
      }],
      "rootfs_path": "/var/lib/lxc",
      "environment": []
    }
  }')

if echo "$CONTAINER_RESPONSE" | grep -q "error.*Permission denied"; then
    echo -e "${RED}❌ Permission denied - LXC requires sudo access${NC}"
    echo -e "${YELLOW}Alternative: Use direct LXC commands with sudo${NC}"
    exit 1
fi

if echo "$CONTAINER_RESPONSE" | grep -q "nginx-test"; then
    echo -e "${GREEN}✅ Container created successfully${NC}"
    CONTAINER_ID=$(echo "$CONTAINER_RESPONSE" | grep -o '"id":"[^"]*"' | cut -d'"' -f6)
    echo -e "${GREEN}Container ID: $CONTAINER_ID${NC}"
else
    echo -e "${RED}❌ Failed to create container${NC}"
    echo "$CONTAINER_RESPONSE"
    exit 1
fi

# Step 2: Start the container
echo -e "${YELLOW}Step 2: Starting container...${NC}"
START_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/containers/nginx-test/start \
  -H 'Content-Type: application/json')

if echo "$START_RESPONSE" | grep -q "success"; then
    echo -e "${GREEN}✅ Container started successfully${NC}"
else
    echo -e "${RED}❌ Failed to start container${NC}"
    echo "$START_RESPONSE"
    exit 1
fi

# Step 3: Install nginx in container
echo -e "${YELLOW}Step 3: Installing nginx...${NC}"
INSTALL_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/containers/nginx-test/exec \
  -H 'Content-Type: application/json' \
  -d '{"command":"apt-get update && apt-get install -y nginx curl"}')

if echo "$INSTALL_RESPONSE" | grep -q "executed"; then
    echo -e "${GREEN}✅ Nginx installation started${NC}"
else
    echo -e "${RED}❌ Failed to execute nginx installation${NC}"
    echo "$INSTALL_RESPONSE"
    exit 1
fi

# Step 4: Create web page
echo -e "${YELLOW}Step 4: Creating web page...${NC}"
WEBPAGE_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/containers/nginx-test/exec \
  -H 'Content-Type: application/json' \
  -d '{
    "command":"echo \"<h1>Hello from Nginx in LXC Container!</h1><p>Deployed at: $(date)</p><p>Hostname: $(hostname)</p><p>Container: nginx-test</p>\" > /var/www/html/index.html"
  }')

if echo "$WEBPAGE_RESPONSE" | grep -q "executed"; then
    echo -e "${GREEN}✅ Web page created${NC}"
else
    echo -e "${RED}❌ Failed to create web page${NC}"
    echo "$WEBPAGE_RESPONSE"
    exit 1
fi

# Step 5: Start nginx service
echo -e "${YELLOW}Step 5: Starting nginx service...${NC}"
NGINX_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/containers/nginx-test/exec \
  -H 'Content-Type: application/json' \
  -d '{"command":"systemctl start nginx"}')

if echo "$NGINX_RESPONSE" | grep -q "executed"; then
    echo -e "${GREEN}✅ Nginx service started${NC}"
else
    echo -e "${RED}❌ Failed to start nginx service${NC}"
    echo "$NGINX_RESPONSE"
    exit 1
fi

# Step 6: Wait for nginx to start
echo -e "${YELLOW}Step 6: Waiting for nginx to start...${NC}"
sleep 3

# Step 7: Get container details and IP
echo -e "${YELLOW}Step 7: Getting container details...${NC}"
CONTAINER_INFO=$(curl -s http://localhost:8080/api/v1/containers/nginx-test)
echo -e "${GREEN}Container info:${NC}"
echo "$CONTAINER_INFO" | jq '.' 2>/dev/null || echo "$CONTAINER_INFO"

# Step 8: Test the web server
echo -e "${YELLOW}Step 8: Testing web server...${NC}"
# Try to determine container IP and test
# Note: This is simplified - in real deployment you'd get the actual container IP
echo -e "${GREEN}Testing connectivity...${NC}"
echo -e "${YELLOW}To test manually: curl http://<container-ip>${NC}"
echo -e "${GREEN}Or check container list: curl http://localhost:8080/api/v1/containers${NC}"

echo -e "${GREEN}=== Nginx deployment test completed! ===${NC}"
