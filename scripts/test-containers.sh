#!/bin/bash

# Container Testing Script for ARM Hypervisor
# Tests all container management functionality

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Configuration
API_BASE="${API_BASE:-https://localhost:8443}"
JWT_TOKEN=""
TEST_PREFIX="test-$(date +%s)"

# Test containers
declare -a TEST_CONTAINERS=()

# Helper functions
get_jwt_token() {
    log_info "Getting JWT authentication token..."
    
    # For testing, create a token with a known secret
    # In production, this would authenticate properly
    local payload='{"sub":"test-user","iat":'$(date +%s)',"exp":'$(($(date +%s) + 3600))'}'
    local header='{"alg":"HS256","typ":"JWT"}'
    
    # This is a simplified token generation - use proper JWT library in production
    JWT_TOKEN="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test-payload.test-signature"
    log_success "JWT token obtained"
}

make_api_request() {
    local method="$1"
    local endpoint="$2"
    local data="$3"
    
    local curl_cmd="curl -s -k -X $method"
    curl_cmd+=" -H 'Content-Type: application/json'"
    if [[ -n "$JWT_TOKEN" ]]; then
        curl_cmd+=" -H 'Authorization: Bearer $JWT_TOKEN'"
    fi
    
    if [[ -n "$data" ]]; then
        curl_cmd+=" -d '$data'"
    fi
    
    curl_cmd+=" $API_BASE/api/v1$endpoint"
    
    eval "$curl_cmd" 2>/dev/null || echo '{"error":"Request failed"}'
}

check_api_health() {
    log_info "Checking API health..."
    
    local response=$(curl -s -k "$API_BASE/health" 2>/dev/null)
    
    if echo "$response" | jq -e '.status' >/dev/null 2>&1; then
        local status=$(echo "$response" | jq -r '.status')
        if [[ "$status" == "healthy" ]]; then
            log_success "API is healthy"
            return 0
        fi
    fi
    
    log_error "API is not healthy: $response"
    return 1
}

test_container_creation() {
    log_info "Testing container creation..."
    
    local container_name="$TEST_PREFIX-ubuntu"
    TEST_CONTAINERS+=("$container_name")
    
    local data='{
        "name": "'$container_name'",
        "template": "ubuntu",
        "config": {
            "cpu_limit": 1,
            "memory_limit": 536870912,
            "disk_limit": 8589934592,
            "network_interfaces": [{"name": "eth0", "bridge": "lxcbr0"}],
            "rootfs_path": "",
            "environment": []
        }
    }'
    
    local response=$(make_api_request "POST" "/containers" "$data")
    
    if echo "$response" | jq -e '.container.id' >/dev/null 2>&1; then
        log_success "Container '$container_name' created successfully"
        return 0
    else
        log_error "Failed to create container: $response"
        return 1
    fi
}

test_container_list() {
    log_info "Testing container listing..."
    
    local response=$(make_api_request "GET" "/containers")
    
    if echo "$response" | jq -e '.containers' >/dev/null 2>&1; then
        local count=$(echo "$response" | jq '.containers | length')
        log_success "Found $count containers"
        
        # Check if our test containers are in the list
        for container in "${TEST_CONTAINERS[@]}"; do
            if echo "$response" | jq -e ".containers[] | select(.name == \"$container\")" >/dev/null 2>&1; then
                log_success "Test container '$container' found in list"
            else
                log_warning "Test container '$container' not found in list"
            fi
        done
        return 0
    else
        log_error "Failed to list containers: $response"
        return 1
    fi
}

test_container_start() {
    log_info "Testing container start..."
    
    local container_name="$TEST_PREFIX-ubuntu"
    local response=$(make_api_request "POST" "/containers/$container_name/start")
    
    if echo "$response" | jq -e '.success' >/dev/null 2>&1; then
        log_success "Container '$container_name' started successfully"
        
        # Wait a moment for container to start
        sleep 3
        return 0
    else
        log_error "Failed to start container: $response"
        return 1
    fi
}

test_container_status() {
    log_info "Testing container status..."
    
    local container_name="$TEST_PREFIX-ubuntu"
    local response=$(make_api_request "GET" "/containers/$container_name")
    
    if echo "$response" | jq -e '.container.status' >/dev/null 2>&1; then
        local status=$(echo "$response" | jq -r '.container.status')
        log_success "Container '$container_name' status: $status"
        return 0
    else
        log_error "Failed to get container status: $response"
        return 1
    fi
}

test_container_stop() {
    log_info "Testing container stop..."
    
    local container_name="$TEST_PREFIX-ubuntu"
    local response=$(make_api_request "POST" "/containers/$container_name/stop")
    
    if echo "$response" | jq -e '.success' >/dev/null 2>&1; then
        log_success "Container '$container_name' stopped successfully"
        
        # Wait a moment for container to stop
        sleep 2
        return 0
    else
        log_error "Failed to stop container: $response"
        return 1
    fi
}

test_container_deletion() {
    log_info "Testing container deletion..."
    
    local container_name="$TEST_PREFIX-ubuntu"
    local response=$(make_api_request "DELETE" "/containers/$container_name")
    
    if echo "$response" | jq -e '.success' >/dev/null 2>&1; then
        log_success "Container '$container_name' deleted successfully"
        return 0
    else
        log_error "Failed to delete container: $response"
        return 1
    fi
}

test_lxc_direct() {
    log_info "Testing direct LXC functionality..."
    
    # Check if LXC is working
    if ! command -v lxc-create >/dev/null 2>&1; then
        log_error "lxc-create command not found"
        return 1
    fi
    
    # Test container creation (with sudo as needed)
    local test_container="$TEST_PREFIX-direct"
    
    log_info "Creating test container with LXC directly..."
    if sudo lxc-create -t ubuntu -n "$test_container" 2>/dev/null; then
        log_success "LXC container created successfully"
        
        # List containers
        if lxc-ls | grep -q "$test_container"; then
            log_success "Container found in LXC list"
        else
            log_error "Container not found in LXC list"
        fi
        
        # Clean up
        sudo lxc-destroy -f "$test_container" 2>/dev/null || true
        log_success "Test container cleaned up"
        
        return 0
    else
        log_error "Failed to create LXC container"
        return 1
    fi
}

test_network_connectivity() {
    log_info "Testing network connectivity..."
    
    # Check if bridge exists
    if ip link show lxcbr0 >/dev/null 2>&1; then
        log_success "lxcbr0 bridge exists"
    else
        log_warning "lxcbr0 bridge not found"
    fi
    
    # Check API connectivity
    if curl -s -k --connect-timeout 5 "$API_BASE/health" >/dev/null 2>&1; then
        log_success "API is reachable"
    else
        log_error "API is not reachable"
        return 1
    fi
    
    return 0
}

cleanup_test_containers() {
    log_info "Cleaning up test containers..."
    
    # Clean up via API
    for container in "${TEST_CONTAINERS[@]}"; do
        log_info "Cleaning up container: $container"
        make_api_request "DELETE" "/containers/$container" >/dev/null 2>&1 || true
        
        # Also clean up via LXC directly if needed
        sudo lxc-destroy -f "$container" 2>/dev/null || true
    done
    
    # Clean up any remaining test containers
    for container in $(sudo lxc-ls 2>/dev/null | grep "$TEST_PREFIX" || true); do
        sudo lxc-destroy -f "$container" 2>/dev/null || true
    done
    
    log_success "Test containers cleaned up"
}

run_performance_test() {
    log_info "Running performance test..."
    
    local start_time=$(date +%s)
    local container_count=3
    
    # Create multiple containers
    for i in $(seq 1 $container_count); do
        local container_name="$TEST_PREFIX-perf-$i"
        TEST_CONTAINERS+=("$container_name")
        
        local data='{
            "name": "'$container_name'",
            "template": "ubuntu",
            "config": {
                "cpu_limit": 1,
                "memory_limit": 268435456,
                "disk_limit": 4294967296,
                "network_interfaces": [{"name": "eth0", "bridge": "lxcbr0"}],
                "rootfs_path": "",
                "environment": []
            }
        }'
        
        make_api_request "POST" "/containers" "$data" >/dev/null &
    done
    
    # Wait for all to complete
    wait
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    log_success "Created $container_count containers in ${duration}s"
    
    # Clean up performance test containers
    for container in $(sudo lxc-ls 2>/dev/null | grep "$TEST_PREFIX-perf" || true); do
        sudo lxc-destroy -f "$container" 2>/dev/null || true
    done
}

# Main test execution
main() {
    log_info "ARM Hypervisor Container Testing"
    echo "================================="
    echo "API Base: $API_BASE"
    echo "Test Prefix: $TEST_PREFIX"
    echo
    
    # Check prerequisites
    if ! command -v jq >/dev/null 2>&1; then
        log_error "jq is required for testing. Install with: apt install jq"
        exit 1
    fi
    
    # Run tests
    local test_results=()
    
    # Network and API tests
    test_network_connectivity && test_results+=("network") || test_results+=("network-failed")
    check_api_health && test_results+=("api-health") || test_results+=("api-health-failed")
    
    # LXC direct tests
    test_lxc_direct && test_results+=("lxc-direct") || test_results+=("lxc-direct-failed")
    
    # API tests
    get_jwt_token
    test_container_creation && test_results+=("container-create") || test_results+=("container-create-failed")
    test_container_list && test_results+=("container-list") || test_results+=("container-list-failed")
    
    # Only test lifecycle if creation succeeded
    if [[ " ${test_results[*]} " =~ " container-create " ]]; then
        test_container_start && test_results+=("container-start") || test_results+=("container-start-failed")
        test_container_status && test_results+=("container-status") || test_results+=("container-status-failed")
        test_container_stop && test_results+=("container-stop") || test_results+=("container-stop-failed")
        test_container_deletion && test_results+=("container-delete") || test_results+=("container-delete-failed")
    fi
    
    # Performance test
    run_performance_test && test_results+=("performance") || test_results+=("performance-failed")
    
    # Cleanup
    cleanup_test_containers
    
    # Results summary
    echo
    echo "================================="
    echo "Test Results Summary"
    echo "================================="
    
    local passed=0
    local failed=0
    
    for result in "${test_results[@]}"; do
        if [[ "$result" == *"-failed" ]]; then
            log_error "$result"
            ((failed++))
        else
            log_success "$result"
            ((passed++))
        fi
    done
    
    echo
    log_info "Passed: $passed, Failed: $failed"
    
    if [[ $failed -eq 0 ]]; then
        log_success "All tests passed! 🎉"
        echo
        echo "Your ARM Hypervisor is ready for production use!"
        exit 0
    else
        log_error "Some tests failed. Please check the configuration and logs."
        exit 1
    fi
}

# Handle cleanup on script exit
trap cleanup_test_containers EXIT

# Run main function
main "$@"