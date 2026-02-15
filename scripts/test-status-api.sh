#!/bin/bash
# Phase 21: Status API 集成测试

set -e

BASE_URL="http://localhost:8080"
FAILED=0

# 颜色输出
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}=====================================${NC}"
echo -e "${YELLOW}Phase 21: Status API 测试 (12 个 API)${NC}"
echo -e "${YELLOW}=====================================${NC}"
echo ""

# 辅助函数
step() {
    echo -e "${YELLOW}[Step $1]${NC} $2"
}

check_response() {
    local response=$1
    local expected=$2
    local step_name=$3

    if echo "$response" | grep -q "$expected"; then
        echo -e "${GREEN}✓ $step_name${NC}"
    else
        echo -e "${RED}✗ $step_name${NC}"
        echo "Response: $response"
        FAILED=$((FAILED + 1))
    fi
}

# ========== Step 1: 注册测试实例 ==========
step 1 "注册测试实例用于 Leases Status 测试"

for i in 1 2 3; do
    curl -s -X POST "$BASE_URL/api/registry/register.json" \
      -H "Content-Type: application/json" \
      -d "{
        \"instances\": [{
          \"region_id\": \"us-east\",
          \"zone_id\": \"zone-1\",
          \"service_id\": \"test-status-service\",
          \"instance_id\": \"inst-$i\",
          \"ip\": \"192.168.1.$i\",
          \"port\": 8080,
          \"url\": \"http://192.168.1.$i:8080\",
          \"status\": \"up\"
        }]
      }" > /dev/null
done

sleep 1
echo -e "${GREEN}✓ 注册 3 个测试实例${NC}"

# ========== Step 2: Node Status API (POST) ==========
step 2 "测试 Node Status API (POST)"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/status/node.json" \
  -H "Content-Type: application/json" \
  -d '{}')

check_response "$RESPONSE" '"status":"SUCCESS"' "POST /api/status/node.json - 返回成功"
check_response "$RESPONSE" '"nodeStatus"' "包含 nodeStatus 字段"
check_response "$RESPONSE" '"status":"up"' "节点状态为 up"

# ========== Step 3: Node Status API (GET) ==========
step 3 "测试 Node Status API (GET)"

RESPONSE=$(curl -s "$BASE_URL/api/status/node.json")

check_response "$RESPONSE" '"status":"SUCCESS"' "GET /api/status/node.json - 返回成功"
check_response "$RESPONSE" '"canServiceDiscovery":true' "canServiceDiscovery 为 true"
check_response "$RESPONSE" '"canServiceRegistry":true' "canServiceRegistry 为 true"

# ========== Step 4: Cluster Status API (POST) ==========
step 4 "测试 Cluster Status API (POST)"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/status/cluster.json" \
  -H "Content-Type: application/json" \
  -d '{}')

check_response "$RESPONSE" '"status":"SUCCESS"' "POST /api/status/cluster.json - 返回成功"
check_response "$RESPONSE" '"nodeCount"' "包含 nodeCount 字段"
check_response "$RESPONSE" '"nodesStatus"' "包含 nodesStatus 字段"

# ========== Step 5: Cluster Status API (GET) ==========
step 5 "测试 Cluster Status API (GET)"

RESPONSE=$(curl -s "$BASE_URL/api/status/cluster.json")

check_response "$RESPONSE" '"status":"SUCCESS"' "GET /api/status/cluster.json - 返回成功"
NODE_COUNT=$(echo "$RESPONSE" | grep -o '"nodeCount":[0-9]*' | grep -o '[0-9]*')
echo "  → 集群节点数: $NODE_COUNT"

# ========== Step 6: Leases Status API (POST) ==========
step 6 "测试 Leases Status API (POST)"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/status/leases.json" \
  -H "Content-Type: application/json" \
  -d '{}')

check_response "$RESPONSE" '"status":"SUCCESS"' "POST /api/status/leases.json - 返回成功"
check_response "$RESPONSE" '"leaseCount"' "包含 leaseCount 字段"
check_response "$RESPONSE" '"leasesStatus"' "包含 leasesStatus 字段"

LEASE_COUNT=$(echo "$RESPONSE" | grep -o '"leaseCount":[0-9]*' | grep -o '[0-9]*')
echo "  → 租约总数: $LEASE_COUNT"

# ========== Step 7: Leases Status API (GET) ==========
step 7 "测试 Leases Status API (GET) - 查询所有租约"

RESPONSE=$(curl -s "$BASE_URL/api/status/leases.json")

check_response "$RESPONSE" '"status":"SUCCESS"' "GET /api/status/leases.json - 返回成功"
check_response "$RESPONSE" '"isSafe"' "包含 isSafe 字段"

# ========== Step 8: Leases Status API (GET with filter) ==========
step 8 "测试 Leases Status API (GET) - 按 serviceId 过滤"

RESPONSE=$(curl -s "$BASE_URL/api/status/leases.json?appIds=test-status-service")

check_response "$RESPONSE" '"status":"SUCCESS"' "GET /api/status/leases.json?appIds=... - 返回成功"
check_response "$RESPONSE" '"leasesStatus"' "包含过滤后的 leasesStatus"

# ========== Step 9: Legacy Leases Status API (POST) ==========
step 9 "测试 Legacy Leases Status API (POST)"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/status/legacy-leases.json" \
  -H "Content-Type: application/json" \
  -d '{}')

check_response "$RESPONSE" '"status":"SUCCESS"' "POST /api/status/legacy-leases.json - 返回成功"
check_response "$RESPONSE" '"leaseCount"' "Legacy API 包含 leaseCount"

# ========== Step 10: Legacy Leases Status API (GET) ==========
step 10 "测试 Legacy Leases Status API (GET)"

RESPONSE=$(curl -s "$BASE_URL/api/status/legacy-leases.json")

check_response "$RESPONSE" '"status":"SUCCESS"' "GET /api/status/legacy-leases.json - 返回成功"

# ========== Step 11: Config Status API (POST) ==========
step 11 "测试 Config Status API (POST)"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/status/config.json" \
  -H "Content-Type: application/json" \
  -d '{}')

check_response "$RESPONSE" '"status":"SUCCESS"' "POST /api/status/config.json - 返回成功"
check_response "$RESPONSE" '"sources"' "包含 sources 字段"
check_response "$RESPONSE" '"properties"' "包含 properties 字段"

# ========== Step 12: Config Status API (GET) ==========
step 12 "测试 Config Status API (GET)"

RESPONSE=$(curl -s "$BASE_URL/api/status/config.json")

check_response "$RESPONSE" '"status":"SUCCESS"' "GET /api/status/config.json - 返回成功"
check_response "$RESPONSE" '"node_id"' "properties 包含 node_id"
check_response "$RESPONSE" '"region_id"' "properties 包含 region_id"

# ========== Step 13: Deployment Status API (POST) ==========
step 13 "测试 Deployment Status API (POST)"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/status/deployment.json" \
  -H "Content-Type: application/json" \
  -d '{}')

check_response "$RESPONSE" '"status":"SUCCESS"' "POST /api/status/deployment.json - 返回成功"
check_response "$RESPONSE" '"regionId"' "包含 regionId 字段"
check_response "$RESPONSE" '"ip"' "包含 ip 字段"
check_response "$RESPONSE" '"port"' "包含 port 字段"

# ========== Step 14: Deployment Status API (GET) ==========
step 14 "测试 Deployment Status API (GET)"

RESPONSE=$(curl -s "$BASE_URL/api/status/deployment.json")

check_response "$RESPONSE" '"status":"SUCCESS"' "GET /api/status/deployment.json - 返回成功"
check_response "$RESPONSE" '"machineName"' "包含 machineName 字段"
check_response "$RESPONSE" '"protocol"' "包含 protocol 字段"

# ========== Step 15: 清理测试数据 ==========
step 15 "清理测试数据"

for i in 1 2 3; do
    curl -s -X POST "$BASE_URL/api/registry/unregister.json" \
      -H "Content-Type: application/json" \
      -d "{
        \"instances\": [{
          \"region_id\": \"us-east\",
          \"zone_id\": \"zone-1\",
          \"service_id\": \"test-status-service\",
          \"instance_id\": \"inst-$i\",
          \"ip\": \"192.168.1.$i\",
          \"port\": 8080,
          \"url\": \"http://192.168.1.$i:8080\"
        }]
      }" > /dev/null
done

echo -e "${GREEN}✓ 清理完成${NC}"

# ========== 测试总结 ==========
echo ""
echo -e "${YELLOW}=====================================${NC}"
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ 所有测试通过! (12/12 APIs tested)${NC}"
    echo -e "${YELLOW}=====================================${NC}"
    exit 0
else
    echo -e "${RED}✗ $FAILED 个测试失败${NC}"
    echo -e "${YELLOW}=====================================${NC}"
    exit 1
fi
