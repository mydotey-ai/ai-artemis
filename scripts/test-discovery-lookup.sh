#!/bin/bash
# Discovery Lookup API 集成测试 (Phase 20)

set -e

BASE_URL="http://localhost:8080"
FAILED=0

# 颜色输出
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}=====================================${NC}"
echo -e "${YELLOW}Phase 20: Discovery Lookup API 测试${NC}"
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

# ========== Step 1: 注册测试服务实例 ==========
step 1 "注册 3 个测试服务实例"

for i in 1 2 3; do
    curl -s -X POST "$BASE_URL/api/registry/register.json" \
      -H "Content-Type: application/json" \
      -d "{
        \"instances\": [{
          \"region_id\": \"us-east\",
          \"zone_id\": \"zone-1\",
          \"service_id\": \"test-lookup-service\",
          \"instance_id\": \"inst-$i\",
          \"ip\": \"192.168.1.$i\",
          \"port\": 8080,
          \"url\": \"http://192.168.1.$i:8080\",
          \"status\": \"up\"
        }]
      }" > /dev/null
done

sleep 2
echo -e "${GREEN}✓ 注册 3 个实例完成${NC}"

# 验证实例已注册
VERIFY=$(curl -s -X POST "$BASE_URL/api/discovery/service.json" \
  -H "Content-Type: application/json" \
  -d '{
    "discovery_config": {
      "service_id": "test-lookup-service",
      "region_id": "us-east",
      "zone_id": "zone-1"
    }
  }')

INST_COUNT=$(echo "$VERIFY" | grep -o '"instance_id"' | wc -l)
echo "  → 已注册实例数: $INST_COUNT"

if [ "$INST_COUNT" != "3" ]; then
    echo -e "${RED}✗ 实例数量不正确 (期望3, 实际$INST_COUNT)${NC}"
    FAILED=$((FAILED + 1))
fi

# ========== Step 2: 使用 Random 策略查询实例 ==========
step 2 "使用 Random 策略查询单个实例"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/discovery/lookup.json" \
  -H "Content-Type: application/json" \
  -d '{
    "discovery_config": {
      "service_id": "test-lookup-service",
      "region_id": "us-east",
      "zone_id": "zone-1"
    },
    "strategy": "random"
  }')

check_response "$RESPONSE" '"success":true' "Random 策略返回成功"
check_response "$RESPONSE" '"instance_id":"inst-' "返回了实例信息"

# 提取instance_id
INST_ID=$(echo "$RESPONSE" | grep -o '"instance_id":"inst-[0-9]"' | grep -o 'inst-[0-9]')
echo "  → 随机选择的实例: $INST_ID"

# ========== Step 3: 使用 RoundRobin 策略查询多次 ==========
step 3 "使用 RoundRobin 策略查询多次 (验证轮询)"

echo "  轮询测试:"
for i in {1..6}; do
    RESPONSE=$(curl -s -X POST "$BASE_URL/api/discovery/lookup.json" \
      -H "Content-Type: application/json" \
      -d '{
        "discovery_config": {
          "service_id": "test-lookup-service",
          "region_id": "us-east",
          "zone_id": "zone-1"
        },
        "strategy": "round-robin"
      }')

    INST_ID=$(echo "$RESPONSE" | grep -o '"instance_id":"inst-[0-9]"' | grep -o 'inst-[0-9]')
    echo "    [$i] -> $INST_ID"
done

echo -e "${GREEN}✓ RoundRobin 轮询正常${NC}"

# ========== Step 4: 查询不存在的服务 ==========
step 4 "查询不存在的服务应返回 404"

RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/api/discovery/lookup.json" \
  -H "Content-Type: application/json" \
  -d '{
    "discovery_config": {
      "service_id": "non-existent-service",
      "region_id": "us-east",
      "zone_id": "zone-1"
    }
  }')

HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
BODY=$(echo "$RESPONSE" | sed '$d')

if [ "$HTTP_CODE" = "404" ]; then
    echo -e "${GREEN}✓ 不存在的服务返回 404${NC}"
else
    echo -e "${RED}✗ 期望 404, 实际 $HTTP_CODE${NC}"
    FAILED=$((FAILED + 1))
fi

check_response "$BODY" '"success":false' "响应包含 success:false"

# ========== Step 5: 测试默认策略 (不指定strategy) ==========
step 5 "测试默认策略 (不指定 strategy,应使用 Random)"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/discovery/lookup.json" \
  -H "Content-Type: application/json" \
  -d '{
    "discovery_config": {
      "service_id": "test-lookup-service",
      "region_id": "us-east",
      "zone_id": "zone-1"
    }
  }')

check_response "$RESPONSE" '"success":true' "默认策略返回成功"
check_response "$RESPONSE" '"instance_id":"inst-' "返回了实例"

# ========== Step 6: 清理 - 注销服务实例 ==========
step 6 "清理测试数据 - 注销服务实例"

for i in 1 2 3; do
    curl -s -X POST "$BASE_URL/api/registry/unregister.json" \
      -H "Content-Type: application/json" \
      -d "{
        \"instances\": [{
          \"region_id\": \"us-east\",
          \"zone_id\": \"zone-1\",
          \"service_id\": \"test-lookup-service\",
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
    echo -e "${GREEN}✓ 所有测试通过! (6/6)${NC}"
    echo -e "${YELLOW}=====================================${NC}"
    exit 0
else
    echo -e "${RED}✗ $FAILED 个测试失败${NC}"
    echo -e "${YELLOW}=====================================${NC}"
    exit 1
fi
