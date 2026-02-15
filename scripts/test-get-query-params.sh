#!/bin/bash
# Phase 22: GET 查询参数支持测试

set -e

BASE_URL="http://localhost:8080"
FAILED=0

# 颜色输出
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}=====================================${NC}"
echo -e "${YELLOW}Phase 22: GET 查询参数支持测试 (3 APIs)${NC}"
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
step 1 "注册测试实例"

for i in 1 2; do
    curl -s -X POST "$BASE_URL/api/registry/register.json" \
      -H "Content-Type: application/json" \
      -d "{
        \"instances\": [{
          \"region_id\": \"us-east\",
          \"zone_id\": \"zone-1\",
          \"service_id\": \"test-get-service\",
          \"instance_id\": \"inst-$i\",
          \"ip\": \"192.168.1.$i\",
          \"port\": 8080,
          \"url\": \"http://192.168.1.$i:8080\",
          \"status\": \"up\"
        }]
      }" > /dev/null
done

sleep 1
echo -e "${GREEN}✓ 注册 2 个测试实例${NC}"

# ========== Step 2: GET /api/discovery/service.json (完整参数) ==========
step 2 "测试 GET /api/discovery/service.json 带完整参数"

RESPONSE=$(curl -s "$BASE_URL/api/discovery/service.json?serviceId=test-get-service&regionId=us-east&zoneId=zone-1")

check_response "$RESPONSE" '"status":"SUCCESS"' "GET service.json 返回成功"
check_response "$RESPONSE" '"service_id":"test-get-service"' "包含正确的 service_id"

INST_COUNT=$(echo "$RESPONSE" | grep -o '"instance_id"' | wc -l)
echo "  → 实例数量: $INST_COUNT"

if [ "$INST_COUNT" -ge 1 ]; then
    echo -e "${GREEN}✓ 返回了实例列表${NC}"
else
    echo -e "${RED}✗ 未返回实例${NC}"
    FAILED=$((FAILED + 1))
fi

# ========== Step 3: GET /api/discovery/service.json (仅必需参数) ==========
step 3 "测试 GET /api/discovery/service.json 仅带 serviceId"

RESPONSE=$(curl -s "$BASE_URL/api/discovery/service.json?serviceId=test-get-service")

check_response "$RESPONSE" '"status":"SUCCESS"' "GET service.json (最小参数) 返回成功"
check_response "$RESPONSE" '"service_id":"test-get-service"' "包含正确的 service_id"

# ========== Step 4: POST vs GET 对比 ==========
step 4 "对比 POST 和 GET 返回结果一致性"

# POST 请求
POST_RESPONSE=$(curl -s -X POST "$BASE_URL/api/discovery/service.json" \
  -H "Content-Type: application/json" \
  -d '{
    "discovery_config": {
      "service_id": "test-get-service",
      "region_id": "us-east",
      "zone_id": "zone-1"
    }
  }')

# GET 请求
GET_RESPONSE=$(curl -s "$BASE_URL/api/discovery/service.json?serviceId=test-get-service&regionId=us-east&zoneId=zone-1")

POST_INST_COUNT=$(echo "$POST_RESPONSE" | grep -o '"instance_id"' | wc -l)
GET_INST_COUNT=$(echo "$GET_RESPONSE" | grep -o '"instance_id"' | wc -l)

if [ "$POST_INST_COUNT" -eq "$GET_INST_COUNT" ]; then
    echo -e "${GREEN}✓ POST 和 GET 返回实例数量一致 ($POST_INST_COUNT)${NC}"
else
    echo -e "${RED}✗ POST ($POST_INST_COUNT) 和 GET ($GET_INST_COUNT) 返回实例数量不一致${NC}"
    FAILED=$((FAILED + 1))
fi

# ========== Step 5: GET /api/discovery/services.json ==========
step 5 "测试 GET /api/discovery/services.json"

RESPONSE=$(curl -s "$BASE_URL/api/discovery/services.json?regionId=us-east&zoneId=zone-1")

check_response "$RESPONSE" '"status":"SUCCESS"' "GET services.json 返回成功"
check_response "$RESPONSE" '"services"' "包含 services 字段"

# ========== Step 6: GET /api/discovery/services.json (无参数) ==========
step 6 "测试 GET /api/discovery/services.json 无参数"

RESPONSE=$(curl -s "$BASE_URL/api/discovery/services.json")

check_response "$RESPONSE" '"status":"SUCCESS"' "GET services.json (无参数) 返回成功"
check_response "$RESPONSE" '"services"' "包含 services 字段"

# ========== Step 7: GET /api/replication/registry/services.json ==========
step 7 "测试 GET /api/replication/registry/services.json"

RESPONSE=$(curl -s "$BASE_URL/api/replication/registry/services.json?regionId=us-east")

check_response "$RESPONSE" '"status":"SUCCESS"' "GET replication services.json 返回成功"
check_response "$RESPONSE" '"services"' "包含 services 字段"

# ========== Step 8: 验证不存在的服务 ==========
step 8 "验证查询不存在的服务"

RESPONSE=$(curl -s "$BASE_URL/api/discovery/service.json?serviceId=non-existent-service")

check_response "$RESPONSE" '"status":"SUCCESS"' "查询不存在服务返回成功状态"
# 注意: Java 版本返回 SUCCESS 但 service 为 null

# ========== Step 9: 清理测试数据 ==========
step 9 "清理测试数据"

for i in 1 2; do
    curl -s -X POST "$BASE_URL/api/registry/unregister.json" \
      -H "Content-Type: application/json" \
      -d "{
        \"instances\": [{
          \"region_id\": \"us-east\",
          \"zone_id\": \"zone-1\",
          \"service_id\": \"test-get-service\",
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
    echo -e "${GREEN}✓ 所有测试通过! (3/3 APIs tested)${NC}"
    echo -e "${YELLOW}=====================================${NC}"
    exit 0
else
    echo -e "${RED}✗ $FAILED 个测试失败${NC}"
    echo -e "${YELLOW}=====================================${NC}"
    exit 1
fi
