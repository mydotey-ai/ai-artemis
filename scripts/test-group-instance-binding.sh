#!/bin/bash
# 分组实例绑定功能集成测试 (Phase 19)

set -e

BASE_URL="http://localhost:8080"
FAILED=0

# 颜色输出
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}=====================================${NC}"
echo -e "${YELLOW}Phase 19: 分组实例绑定功能测试${NC}"
echo -e "${YELLOW}=====================================${NC}"
echo ""

# 辅助函数: 打印步骤
step() {
    echo -e "${YELLOW}[Step $1]${NC} $2"
}

# 辅助函数: 检查响应
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

# ========== Step 1: 创建分组 ==========
step 1 "创建测试分组 (test-service:us-east:zone-1:group-binding-test)"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/routing/groups" \
  -H "Content-Type: application/json" \
  -d '{
    "service_id": "test-service",
    "region_id": "us-east",
    "zone_id": "zone-1",
    "name": "group-binding-test",
    "group_type": "logical",
    "description": "测试分组实例绑定"
  }')

check_response "$RESPONSE" '"success":true' "创建分组成功"

# 提取 group_id
GROUP_ID=$(echo "$RESPONSE" | grep -o '"group_id":[0-9]*' | grep -o '[0-9]*')
echo "  → Group ID: $GROUP_ID"

# ========== Step 2: 添加实例到分组 (手动绑定) ==========
step 2 "手动添加实例到分组"

GROUP_KEY="test-service:us-east:zone-1:group-binding-test"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/routing/groups/$GROUP_KEY/instances" \
  -H "Content-Type: application/json" \
  -d '{
    "instance_id": "inst-001",
    "region_id": "us-east",
    "zone_id": "zone-1",
    "service_id": "test-service",
    "operator_id": "test-operator"
  }')

check_response "$RESPONSE" '"success":true' "添加实例到分组成功"

# ========== Step 3: 再添加一个实例 ==========
step 3 "添加第二个实例"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/routing/groups/$GROUP_KEY/instances" \
  -H "Content-Type: application/json" \
  -d '{
    "instance_id": "inst-002",
    "region_id": "us-east",
    "zone_id": "zone-1",
    "service_id": "test-service",
    "operator_id": "test-operator"
  }')

check_response "$RESPONSE" '"success":true' "添加第二个实例成功"

# ========== Step 4: 查询分组实例 (内存查询) ==========
step 4 "查询分组实例 (通过原有 API)"

# 注意: 这里测试的是原有的内存查询API,不是新的数据库查询API
# 新 API 需要 GroupManager.get_group_instances() 返回数据库结果

# ========== Step 5: 从分组移除实例 ==========
step 5 "从分组移除实例 inst-001"

RESPONSE=$(curl -s -X DELETE "$BASE_URL/api/routing/groups/$GROUP_KEY/instances/inst-001?region_id=us-east&zone_id=zone-1")

check_response "$RESPONSE" '"success":true' "移除实例成功"

# ========== Step 6: 重复移除应该失败 ==========
step 6 "重复移除应该失败 (404)"

RESPONSE=$(curl -s -X DELETE "$BASE_URL/api/routing/groups/$GROUP_KEY/instances/inst-001?region_id=us-east&zone_id=zone-1")

check_response "$RESPONSE" '"success":false' "重复移除正确返回失败"

# ========== Step 7: 批量添加服务实例 ==========
step 7 "批量添加服务实例"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/routing/services/test-service/instances" \
  -H "Content-Type: application/json" \
  -d "{
    \"instances\": [
      {
        \"group_id\": $GROUP_ID,
        \"instance_id\": \"inst-101\",
        \"region_id\": \"us-east\",
        \"zone_id\": \"zone-1\",
        \"service_id\": \"test-service\",
        \"binding_type\": \"manual\",
        \"operator_id\": \"batch-operator\"
      },
      {
        \"group_id\": $GROUP_ID,
        \"instance_id\": \"inst-102\",
        \"region_id\": \"us-east\",
        \"zone_id\": \"zone-1\",
        \"service_id\": \"test-service\",
        \"binding_type\": \"manual\",
        \"operator_id\": \"batch-operator\"
      },
      {
        \"group_id\": $GROUP_ID,
        \"instance_id\": \"inst-103\",
        \"region_id\": \"us-east\",
        \"zone_id\": \"zone-1\",
        \"service_id\": \"test-service\",
        \"binding_type\": \"manual\",
        \"operator_id\": \"batch-operator\"
      }
    ]
  }")

check_response "$RESPONSE" '"success":true' "批量添加实例成功"

# 检查返回的数量
COUNT=$(echo "$RESPONSE" | grep -o '"data":[0-9]*' | grep -o '[0-9]*')
if [ "$COUNT" = "3" ]; then
    echo -e "${GREEN}✓ 批量添加数量正确: 3${NC}"
else
    echo -e "${RED}✗ 批量添加数量错误: expected 3, got $COUNT${NC}"
    FAILED=$((FAILED + 1))
fi

# ========== Step 8: 验证 service_id 不匹配应该失败 ==========
step 8 "验证 service_id 不匹配应该失败"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/routing/services/wrong-service/instances" \
  -H "Content-Type: application/json" \
  -d "{
    \"instances\": [
      {
        \"group_id\": $GROUP_ID,
        \"instance_id\": \"inst-999\",
        \"region_id\": \"us-east\",
        \"zone_id\": \"zone-1\",
        \"service_id\": \"test-service\",
        \"binding_type\": \"manual\"
      }
    ]
  }")

check_response "$RESPONSE" '"success":false' "service_id 不匹配正确返回失败"

# ========== Step 9: 清理 - 删除分组 ==========
step 9 "清理测试数据 - 删除分组"

RESPONSE=$(curl -s -X DELETE "$BASE_URL/api/routing/groups/$GROUP_KEY")
check_response "$RESPONSE" '"success":true' "删除分组成功"

# ========== 测试总结 ==========
echo ""
echo -e "${YELLOW}=====================================${NC}"
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ 所有测试通过! (9/9)${NC}"
    echo -e "${YELLOW}=====================================${NC}"
    exit 0
else
    echo -e "${RED}✗ $FAILED 个测试失败${NC}"
    echo -e "${YELLOW}=====================================${NC}"
    exit 1
fi
