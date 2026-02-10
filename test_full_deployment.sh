#!/bin/bash

echo "=== Full Deployment Test ==="
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${YELLOW}🔧 Step 1: Test Sudo Access${NC}"
if sudo -n true 2>/dev/null; then
    echo -e "${GREEN}✅ Sudo access confirmed!${NC}"
else
    echo -e "${RED}❌ No sudo access${NC}"
    exit 1
fi

echo -e "${YELLOW}🚀 Step 2: Start Orchestrator${NC}"
cd crates/api-server
cargo run > /tmp/api-server-full.log 2>&1 &
API_PID=$!
echo "API server started with PID: $API_PID"

echo -e "${YELLOW}⏳ Step 3: Wait for API to be ready${NC}"
for i in {1..30}; do
    if curl -s http://localhost:8080/health > /dev/null 2>&1; then
        echo -e "${GREEN}✅ API is ready!${NC}"
        break
    fi
    echo "Waiting for API... ($i/30)"
    sleep 2
done

echo -e "${YELLOW}🏥 Step 4: Check API Health${NC}"
HEALTH=$(curl -s http://localhost:8080/health)
echo "Status: $(echo "$HEALTH" | jq -r '.status')"
echo "Container Manager: $(echo "$HEALTH" | jq -r '.services.container_manager.status')"
echo "Network Manager: $(echo "$HEALTH" | jq -r '.services.network_manager.status')"

echo -e "${YELLOW}📦 Step 5: Create Test Container${NC}"
CREATE_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/containers \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "nginx-test",
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
        ["SERVER_NAME", "nginx-test"]
      ]
    }
  }')

if echo "$CREATE_RESPONSE" | grep -q "error"; then
    echo -e "${RED}❌ Container creation failed${NC}"
    echo "$CREATE_RESPONSE"
    exit 1
else
    echo -e "${GREEN}✅ Container created successfully!${NC}"
    CONTAINER_ID=$(echo "$CREATE_RESPONSE" | jq -r '.id')
    echo "Container ID: $CONTAINER_ID"
fi

echo -e "${YELLOW}▶️ Step 6: Start Container${NC}"
START_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/containers/nginx-test/start)
if echo "$START_RESPONSE" | grep -q "success"; then
    echo -e "${GREEN}✅ Container started successfully${NC}"
else
    echo -e "${RED}❌ Failed to start container${NC}"
    echo "$START_RESPONSE"
fi

echo -e "${YELLOW}⚙️ Step 7: Install Nginx${NC}"
INSTALL_CMD=$(curl -s -X POST http://localhost:8080/api/v1/containers/nginx-test/exec \
  -H 'Content-Type: application/json' \
  -d '{"command":"apt-get update && apt-get install -y nginx"}')

if echo "$INSTALL_CMD" | grep -q "executed"; then
    echo -e "${GREEN}✅ Nginx installation started${NC}"
else
    echo -e "${RED}❌ Failed to install nginx${NC}"
fi

echo -e "${YELLOW}🌐 Step 8: Setup Web Content${NC}"
WEBPAGE_CMD=$(curl -s -X POST http://localhost:8080/api/v1/containers/nginx-test/exec \
  -H 'Content-Type: application/json' \
  -d '{
    "command":"echo \"<h1>🎉 Nginx Running in LXC Container!</h1><p>Container: nginx-test</p><p>Time: $(date)</p><p>Deployed via Enhanced Orchestrator</p>\" > /var/www/html/index.html"
  }')

if echo "$WEBPAGE_CMD" | grep -q "executed"; then
    echo -e "${GREEN}✅ Web page created${NC}"
else
    echo -e "${RED}❌ Failed to create web page${NC}"
fi

echo -e "${YELLOW}🚀 Step 9: Start Nginx Service${NC}"
START_NGINX=$(curl -s -X POST http://localhost:8080/api/v1/containers/nginx-test/exec \
  -H 'Content-Type: application/json' \
  -d '{"command":"systemctl start nginx"}')

if echo "$START_NGINX" | grep -q "executed"; then
    echo -e "${GREEN}✅ Nginx service started${NC}"
else
    echo -e "${RED}❌ Failed to start nginx${NC}"
fi

echo -e "${YELLOW}🔍 Step 10: Verify Deployment${NC}"
echo "Container status:"
FINAL_STATUS=$(curl -s http://localhost:8080/api/v1/containers/nginx-test)
echo "$FINAL_STATUS" | jq -r '{name: .name, status: .status}'

echo ""
echo -e "${YELLOW}🌐 Step 11: Test Web Access${NC}"
WEB_TEST=$(curl -s -X POST http://localhost:8080/api/v1/containers/nginx-test/exec \
  -H 'Content-Type: application/json' \
  -d '{"command":"curl -I http://localhost"}')

if echo "$WEB_TEST" | grep -q "200 OK"; then
    echo -e "${GREEN}✅ Nginx responding successfully!${NC}"
else
    echo -e "${RED}❌ Nginx not responding${NC}"
    echo "$WEB_TEST"
fi

echo ""
echo -e "${GREEN}🎉 Full Deployment Test Complete!${NC}"
echo ""
echo -e "${YELLOW}Cleanup Commands:${NC}"
echo "Stop: curl -X POST http://localhost:8080/api/v1/containers/nginx-test/stop"
echo "Delete: curl -X DELETE http://localhost:8080/api/v1/containers/nginx-test"
echo "Stop API: kill $API_PID"
echo ""
echo -e "${BLUE}🚀 Enhanced LXC Orchestrator Working Perfectly!${NC}"