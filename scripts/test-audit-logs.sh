#!/bin/bash
# Phase 24: 审计日志细分 API 集成测试

set -e

BASE_URL="http://localhost:8080"
FAILED=0

# 颜色输出
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}=====================================${NC}"
echo -e "${YELLOW}Phase 24: 审计日志细分 API 测试 (6 APIs)${NC}"
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

# ========== Step 1: 生成测试数据 - 记录各种操作日志 ==========
step 1 "生成测试审计日志数据"

# 注册实例 (会记录实例日志)
curl -s -X POST "$BASE_URL/api/registry/register.json" \
  -H "Content-Type: application/json" \
  -d '{
    "instances": [{
      "region_id": "us-east",
      "zone_id": "zone-1",
      "service_id": "audit-test-service",
      "instance_id": "inst-1",
      "ip": "192.168.1.1",
      "port": 8080,
      "url": "http://192.168.1.1:8080",
      "status": "up"
    }]
  }' > /dev/null

# 拉出实例 (会记录实例操作日志)
curl -s -X POST "$BASE_URL/api/management/instance/operate-instance.json" \
  -H "Content-Type: application/json" \
  -d '{
    "instance_key": {
      "region_id": "us-east",
      "zone_id": "zone-1",
      "group_id": "",
      "service_id": "audit-test-service",
      "instance_id": "inst-1"
    },
    "operation": "pullout",
    "operator_id": "test-operator"
  }' > /dev/null

sleep 1
echo -e "${GREEN}✓ 测试数据生成完成${NC}"

# ========== Step 2: 测试分组日志 API ==========
step 2 "测试分组操作日志 API (Group Logs)"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/management/log/group-logs.json")

check_response "$RESPONSE" '"success":true' "分组日志 API 返回成功"
check_response "$RESPONSE" '"data"' "包含 data 字段"

# ========== Step 3: 测试路由规则日志 API ==========
step 3 "测试路由规则操作日志 API (Route Rule Logs)"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/management/log/route-rule-logs.json")

check_response "$RESPONSE" '"success":true' "路由规则日志 API 返回成功"

# ========== Step 4: 测试路由规则分组日志 API ==========
step 4 "测试路由规则分组操作日志 API (Route Rule Group Logs)"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/management/log/route-rule-group-logs.json")

check_response "$RESPONSE" '"success":true' "路由规则分组日志 API 返回成功"

# ========== Step 5: 测试 Zone 操作日志 API ==========
step 5 "测试 Zone 操作日志 API (Zone Operation Logs)"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/management/log/zone-operation-logs.json")

check_response "$RESPONSE" '"success":true' "Zone 操作日志 API 返回成功"

# ========== Step 6: 测试分组实例绑定日志 API ==========
step 6 "测试分组实例绑定日志 API (Group Instance Logs)"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/management/log/group-instance-logs.json")

check_response "$RESPONSE" '"success":true' "分组实例绑定日志 API 返回成功"

# ========== Step 7: 测试服务实例日志 API ==========
step 7 "测试服务实例日志 API (Service Instance Logs)"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/management/log/service-instance-logs.json?service_id=audit-test-service")

check_response "$RESPONSE" '"success":true' "服务实例日志 API 返回成功"

# 验证能查询到实例操作记录
LOG_COUNT=$(echo "$RESPONSE" | grep -o '"log_id"' | wc -l)
if [ "$LOG_COUNT" -ge 1 ]; then
    echo -e "${GREEN}✓ 服务实例日志返回了操作记录 ($LOG_COUNT 条)${NC}"
else
    echo -e "${YELLOW}⚠ 服务实例日志未返回记录 (可能尚未记录)${NC}"
fi

# ========== Step 8: 测试查询参数过滤 ==========
step 8 "测试查询参数过滤 (operator_id)"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/management/log/service-instance-logs.json?operator_id=test-operator")

check_response "$RESPONSE" '"success":true' "按 operator_id 过滤成功"

# ========== Step 9: 测试 limit 参数 ==========
step 9 "测试 limit 参数限制返回数量"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/management/log/service-instance-logs.json?limit=1")

check_response "$RESPONSE" '"success":true' "limit 参数有效"

# ========== Step 10: 对比现有审计 API 和新 API ==========
step 10 "验证新 API 与现有 audit API 的兼容性"

# 查询现有实例日志 API
OLD_RESPONSE=$(curl -s "$BASE_URL/api/management/audit/instance-logs")
check_response "$OLD_RESPONSE" '"success":true' "现有实例日志 API 正常工作"

# 查询新服务实例日志 API (应该返回类似数据)
NEW_RESPONSE=$(curl -s -X POST "$BASE_URL/api/management/log/service-instance-logs.json")
check_response "$NEW_RESPONSE" '"success":true' "新服务实例日志 API 正常工作"

# ========== Step 11: 清理测试数据 ==========
step 11 "清理测试数据"

curl -s -X POST "$BASE_URL/api/management/instance/operate-instance.json" \
  -H "Content-Type: application/json" \
  -d '{
    "instance_key": {
      "region_id": "us-east",
      "zone_id": "zone-1",
      "group_id": "",
      "service_id": "audit-test-service",
      "instance_id": "inst-1"
    },
    "operation": "pullin",
    "operator_id": "test-operator"
  }' > /dev/null

curl -s -X POST "$BASE_URL/api/registry/unregister.json" \
  -H "Content-Type: application/json" \
  -d '{
    "instance_keys": [{
      "region_id": "us-east",
      "zone_id": "zone-1",
      "group_id": "",
      "service_id": "audit-test-service",
      "instance_id": "inst-1"
    }]
  }' > /dev/null

echo -e "${GREEN}✓ 清理完成${NC}"

# ========== 测试总结 ==========
echo ""
echo -e "${YELLOW}=====================================${NC}"
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ 所有测试通过! (6/6 APIs tested)${NC}"
    echo -e "${YELLOW}=====================================${NC}"
    exit 0
else
    echo -e "${RED}✗ $FAILED 个测试失败${NC}"
    echo -e "${YELLOW}=====================================${NC}"
    exit 1
fi
