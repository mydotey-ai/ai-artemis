#!/bin/bash

# Artemis Web Console API 自动化测试脚本
# 用途: 验证 Web Console 后端 API 功能

set -e

BASE_URL="${BASE_URL:-http://localhost:8080}"

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;36m'
NC='\033[0m' # No Color

# 测试计数器
TESTS_PASSED=0
TESTS_FAILED=0

# 测试函数
test_assert() {
    local test_name="$1"
    local expected="$2"
    local actual="$3"

    if [ "$expected" = "$actual" ]; then
        echo -e "   ${GREEN}✅ $test_name${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "   ${RED}❌ $test_name${NC}"
        echo -e "      期望: $expected, 实际: $actual"
        ((TESTS_FAILED++))
    fi
}

echo -e "${BLUE}=========================================${NC}"
echo -e "${BLUE}  Artemis Web Console API 测试${NC}"
echo -e "${BLUE}=========================================${NC}"
echo ""

# 1. 测试仪表盘 API
echo -e "${BLUE}━━━ 1. 仪表盘 API ━━━${NC}"
SERVICES=$(curl -s -X POST "${BASE_URL}/api/discovery/services.json" \
  -H "Content-Type: application/json" \
  -d '{"region_id":"local","zone_id":"zone1"}')

SERVICE_COUNT=$(echo "$SERVICES" | jq '.services | length')
INSTANCE_COUNT=$(echo "$SERVICES" | jq '[.services[].instances | length] | add')
HEALTHY_COUNT=$(echo "$SERVICES" | jq '[.services[].instances[] | select(.status == "up")] | length')

test_assert "服务数量正确" "3" "$SERVICE_COUNT"
test_assert "实例总数正确" "6" "$INSTANCE_COUNT"
test_assert "健康实例数正确" "5" "$HEALTHY_COUNT"
echo ""

# 2. 测试服务列表 API
echo -e "${BLUE}━━━ 2. 服务列表 API ━━━${NC}"

# user-service
USER_SERVICE=$(curl -s -X POST "${BASE_URL}/api/discovery/service.json" \
  -H "Content-Type: application/json" \
  -d '{
    "discovery_config": {
      "service_id": "user-service",
      "region_id": "local",
      "zone_id": "zone1"
    }
  }')
USER_INSTANCE_COUNT=$(echo "$USER_SERVICE" | jq '.service.instances | length')
test_assert "user-service 实例数" "2" "$USER_INSTANCE_COUNT"

# order-service (列表查询)
ORDER_COUNT=$(echo "$SERVICES" | jq '.services[] | select(.service_id == "order-service") | .instances | length')
test_assert "order-service 实例数" "3" "$ORDER_COUNT"

# payment-service
PAYMENT_COUNT=$(echo "$SERVICES" | jq '.services[] | select(.service_id == "payment-service") | .instances | length')
test_assert "payment-service 实例数" "1" "$PAYMENT_COUNT"

echo ""

# 3. 测试实例管理 API
echo -e "${BLUE}━━━ 3. 实例管理 API ━━━${NC}"

UP_COUNT=$(echo "$SERVICES" | jq '[.services[].instances[] | select(.status == "up")] | length')
STARTING_COUNT=$(echo "$SERVICES" | jq '[.services[].instances[] | select(.status == "starting")] | length')

test_assert "UP 状态实例数" "5" "$UP_COUNT"
test_assert "STARTING 状态实例数" "1" "$STARTING_COUNT"

# 检查 metadata 存在
HAS_METADATA=$(echo "$SERVICES" | jq '[.services[].instances[] | select(.metadata != null)] | length > 0')
test_assert "实例包含 Metadata" "true" "$HAS_METADATA"

echo ""

# 4. 测试集群管理 API
echo -e "${BLUE}━━━ 4. 集群管理 API ━━━${NC}"

CLUSTER=$(curl -s "${BASE_URL}/api/status/cluster.json")
NODE_COUNT=$(echo "$CLUSTER" | jq '.nodeCount')
NODE_STATUS=$(echo "$CLUSTER" | jq -r '.nodesStatus[0].status')

test_assert "集群节点数" "1" "$NODE_COUNT"
test_assert "节点状态" "up" "$NODE_STATUS"

echo ""

# 5. 测试错误处理
echo -e "${BLUE}━━━ 5. 错误处理 ━━━${NC}"

# 不存在的服务
ERROR_RESPONSE=$(curl -s -X POST "${BASE_URL}/api/discovery/service.json" \
  -H "Content-Type: application/json" \
  -d '{
    "discovery_config": {
      "service_id": "non-existent-service",
      "region_id": "local",
      "zone_id": "zone1"
    }
  }')
ERROR_CODE=$(echo "$ERROR_RESPONSE" | jq -r '.response_status.error_code')
test_assert "不存在的服务返回错误" "bad-request" "$ERROR_CODE"

# 不存在的端点
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "${BASE_URL}/api/non-existent-endpoint")
test_assert "不存在的端点返回 404" "404" "$HTTP_CODE"

echo ""

# 6. 高级功能 API (可访问性测试)
echo -e "${BLUE}━━━ 6. 高级功能 API 可访问性 ━━━${NC}"

# 路由分组
GROUPS_CODE=$(curl -s -o /dev/null -w "%{http_code}" "${BASE_URL}/api/routing/groups")
test_assert "路由分组 API 可访问" "200" "$GROUPS_CODE"

# 路由规则
RULES_CODE=$(curl -s -o /dev/null -w "%{http_code}" "${BASE_URL}/api/routing/rules")
test_assert "路由规则 API 可访问" "200" "$RULES_CODE"

# Zone 操作
ZONES_CODE=$(curl -s -o /dev/null -w "%{http_code}" "${BASE_URL}/api/management/zone/operations")
test_assert "Zone 管理 API 可访问" "200" "$ZONES_CODE"

# 金丝雀配置
CANARY_CODE=$(curl -s -o /dev/null -w "%{http_code}" "${BASE_URL}/api/management/canary/configs")
test_assert "金丝雀 API 可访问" "200" "$CANARY_CODE"

echo ""

# 测试总结
echo -e "${BLUE}=========================================${NC}"
echo -e "${BLUE}  测试总结${NC}"
echo -e "${BLUE}=========================================${NC}"
echo ""
echo -e "   ${GREEN}通过: $TESTS_PASSED${NC}"
echo -e "   ${RED}失败: $TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ 所有测试通过! Web Console API 工作正常${NC}"
    exit 0
else
    echo -e "${RED}❌ 有 $TESTS_FAILED 个测试失败${NC}"
    exit 1
fi
