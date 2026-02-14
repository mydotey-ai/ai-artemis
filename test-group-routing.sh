#!/bin/bash

# 分组路由功能集成测试脚本
# 测试完整的路由功能流程

set -e  # 遇到错误立即退出

BASE_URL="http://localhost:8080"
SERVICE_ID="test-routing-service"
REGION_ID="us-east"
ZONE_ID="zone-1"

echo "========================================="
echo "分组路由功能集成测试"
echo "========================================="
echo ""

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 测试结果统计
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# 测试函数
test_step() {
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    echo -e "${YELLOW}[测试 $TOTAL_TESTS]${NC} $1"
}

test_pass() {
    PASSED_TESTS=$((PASSED_TESTS + 1))
    echo -e "${GREEN}✓ 通过${NC}"
    echo ""
}

test_fail() {
    FAILED_TESTS=$((FAILED_TESTS + 1))
    echo -e "${RED}✗ 失败: $1${NC}"
    echo ""
}

# 清理函数
cleanup() {
    echo ""
    echo "========================================="
    echo "测试总结"
    echo "========================================="
    echo "总测试数: $TOTAL_TESTS"
    echo -e "${GREEN}通过: $PASSED_TESTS${NC}"
    echo -e "${RED}失败: $FAILED_TESTS${NC}"
    echo ""

    if [ $FAILED_TESTS -eq 0 ]; then
        echo -e "${GREEN}所有测试通过! 🎉${NC}"
        exit 0
    else
        echo -e "${RED}有测试失败 ❌${NC}"
        exit 1
    fi
}

trap cleanup EXIT

# 等待服务启动
wait_for_server() {
    echo "等待服务器启动..."
    for i in {1..30}; do
        if curl -s "$BASE_URL/health" > /dev/null 2>&1; then
            echo -e "${GREEN}服务器已就绪${NC}"
            echo ""
            return 0
        fi
        sleep 1
    done
    echo -e "${RED}服务器启动超时${NC}"
    exit 1
}

# 检查服务器是否运行
if ! curl -s "$BASE_URL/health" > /dev/null 2>&1; then
    echo -e "${YELLOW}服务器未运行,请先启动服务器:${NC}"
    echo "  cargo run --bin artemis -- server"
    exit 1
fi

echo "服务器已就绪"
echo ""

# ==========================================
# 步骤 1: 注册测试服务的实例
# ==========================================
test_step "注册服务实例到不同分组"

# 分组 A 的实例 (权重 70%)
for i in {1..3}; do
    curl -s -X POST "$BASE_URL/api/registry/register.json" \
      -H "Content-Type: application/json" \
      -d "{
        \"instances\": [{
          \"region_id\": \"$REGION_ID\",
          \"zone_id\": \"$ZONE_ID\",
          \"service_id\": \"$SERVICE_ID\",
          \"instance_id\": \"group-a-inst-$i\",
          \"group_id\": \"group-a\",
          \"ip\": \"192.168.1.1$i\",
          \"port\": 8080,
          \"url\": \"http://192.168.1.1$i:8080\",
          \"status\": \"up\"
        }]
      }" > /dev/null
done

# 分组 B 的实例 (权重 30%)
for i in {1..2}; do
    curl -s -X POST "$BASE_URL/api/registry/register.json" \
      -H "Content-Type: application/json" \
      -d "{
        \"instances\": [{
          \"region_id\": \"$REGION_ID\",
          \"zone_id\": \"$ZONE_ID\",
          \"service_id\": \"$SERVICE_ID\",
          \"instance_id\": \"group-b-inst-$i\",
          \"group_id\": \"group-b\",
          \"ip\": \"192.168.1.2$i\",
          \"port\": 8080,
          \"url\": \"http://192.168.1.2$i:8080\",
          \"status\": \"up\"
        }]
      }" > /dev/null
done

test_pass

# ==========================================
# 步骤 2: 验证服务发现返回所有实例 (无路由规则)
# ==========================================
test_step "验证未配置路由规则时返回所有实例"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/discovery/service.json" \
  -H "Content-Type: application/json" \
  -d "{
    \"discovery_config\": {
      \"service_id\": \"$SERVICE_ID\",
      \"region_id\": \"$REGION_ID\",
      \"zone_id\": \"$ZONE_ID\"
    }
  }")

INSTANCE_COUNT=$(echo "$RESPONSE" | jq -r '.service.instances | length')

if [ "$INSTANCE_COUNT" -eq 5 ]; then
    test_pass
else
    test_fail "期望 5 个实例,实际 $INSTANCE_COUNT 个"
fi

# ==========================================
# 步骤 3: 创建分组 A
# ==========================================
test_step "创建分组 A (生产环境)"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/routing/groups" \
  -H "Content-Type: application/json" \
  -d "{
    \"service_id\": \"$SERVICE_ID\",
    \"region_id\": \"$REGION_ID\",
    \"zone_id\": \"$ZONE_ID\",
    \"name\": \"group-a\",
    \"group_type\": \"physical\",
    \"description\": \"生产环境分组\"
  }")

SUCCESS=$(echo "$RESPONSE" | jq -r '.success')
GROUP_A_ID=$(echo "$RESPONSE" | jq -r '.data.group_id')

if [ "$SUCCESS" = "true" ] && [ "$GROUP_A_ID" != "null" ]; then
    echo "  分组 ID: $GROUP_A_ID"
    test_pass
else
    test_fail "创建分组失败"
fi

# ==========================================
# 步骤 4: 创建分组 B
# ==========================================
test_step "创建分组 B (测试环境)"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/routing/groups" \
  -H "Content-Type: application/json" \
  -d "{
    \"service_id\": \"$SERVICE_ID\",
    \"region_id\": \"$REGION_ID\",
    \"zone_id\": \"$ZONE_ID\",
    \"name\": \"group-b\",
    \"group_type\": \"physical\",
    \"description\": \"测试环境分组\"
  }")

SUCCESS=$(echo "$RESPONSE" | jq -r '.success')
GROUP_B_ID=$(echo "$RESPONSE" | jq -r '.data.group_id')

if [ "$SUCCESS" = "true" ] && [ "$GROUP_B_ID" != "null" ]; then
    echo "  分组 ID: $GROUP_B_ID"
    test_pass
else
    test_fail "创建分组失败"
fi

# ==========================================
# 步骤 5: 创建路由规则
# ==========================================
test_step "创建加权轮询路由规则"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/routing/rules" \
  -H "Content-Type: application/json" \
  -d "{
    \"route_id\": \"rule-weighted\",
    \"service_id\": \"$SERVICE_ID\",
    \"name\": \"加权路由规则\",
    \"description\": \"70% 流量到生产,30% 到测试\",
    \"strategy\": \"weighted-round-robin\"
  }")

SUCCESS=$(echo "$RESPONSE" | jq -r '.success')
RULE_ID=$(echo "$RESPONSE" | jq -r '.data.route_id')

if [ "$SUCCESS" = "true" ] && [ "$RULE_ID" = "rule-weighted" ]; then
    test_pass
else
    test_fail "创建路由规则失败"
fi

# ==========================================
# 步骤 6: 添加分组 A 到规则 (权重 70)
# ==========================================
test_step "添加分组 A 到规则 (权重 70%)"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/routing/rules/rule-weighted/groups" \
  -H "Content-Type: application/json" \
  -d "{
    \"group_id\": \"group-a\",
    \"weight\": 70,
    \"region_id\": \"$REGION_ID\"
  }")

SUCCESS=$(echo "$RESPONSE" | jq -r '.success')

if [ "$SUCCESS" = "true" ]; then
    test_pass
else
    test_fail "添加分组失败"
fi

# ==========================================
# 步骤 7: 添加分组 B 到规则 (权重 30)
# ==========================================
test_step "添加分组 B 到规则 (权重 30%)"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/routing/rules/rule-weighted/groups" \
  -H "Content-Type: application/json" \
  -d "{
    \"group_id\": \"group-b\",
    \"weight\": 30,
    \"region_id\": \"$REGION_ID\"
  }")

SUCCESS=$(echo "$RESPONSE" | jq -r '.success')

if [ "$SUCCESS" = "true" ]; then
    test_pass
else
    test_fail "添加分组失败"
fi

# ==========================================
# 步骤 8: 验证规则的分组配置
# ==========================================
test_step "验证规则的分组配置"

RESPONSE=$(curl -s -X GET "$BASE_URL/api/routing/rules/rule-weighted/groups")

GROUP_COUNT=$(echo "$RESPONSE" | jq -r '.data | length')
WEIGHT_A=$(echo "$RESPONSE" | jq -r '.data[] | select(.group_id == "group-a") | .weight')
WEIGHT_B=$(echo "$RESPONSE" | jq -r '.data[] | select(.group_id == "group-b") | .weight')

if [ "$GROUP_COUNT" -eq 2 ] && [ "$WEIGHT_A" -eq 70 ] && [ "$WEIGHT_B" -eq 30 ]; then
    echo "  分组数: $GROUP_COUNT"
    echo "  分组 A 权重: $WEIGHT_A"
    echo "  分组 B 权重: $WEIGHT_B"
    test_pass
else
    test_fail "分组配置不正确"
fi

# ==========================================
# 步骤 9: 发布路由规则
# ==========================================
test_step "发布路由规则"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/routing/rules/rule-weighted/publish")

SUCCESS=$(echo "$RESPONSE" | jq -r '.success')

if [ "$SUCCESS" = "true" ]; then
    test_pass
else
    test_fail "发布规则失败"
fi

# ==========================================
# 步骤 10: 测试加权路由 - 统计分组分布
# ==========================================
test_step "测试加权路由 - 统计 100 次请求的分组分布"

COUNT_A=0
COUNT_B=0

for i in {1..100}; do
    RESPONSE=$(curl -s -X POST "$BASE_URL/api/discovery/service.json" \
      -H "Content-Type: application/json" \
      -d "{
        \"discovery_config\": {
          \"service_id\": \"$SERVICE_ID\",
          \"region_id\": \"$REGION_ID\"
        }
      }")

    # 获取第一个实例的 group_id
    GROUP=$(echo "$RESPONSE" | jq -r '.service.instances[0].group_id // empty')

    if [ "$GROUP" = "group-a" ]; then
        COUNT_A=$((COUNT_A + 1))
    elif [ "$GROUP" = "group-b" ]; then
        COUNT_B=$((COUNT_B + 1))
    fi
done

PERCENT_A=$((COUNT_A))
PERCENT_B=$((COUNT_B))

echo "  分组 A: $COUNT_A 次 ($PERCENT_A%)"
echo "  分组 B: $COUNT_B 次 ($PERCENT_B%)"

# 允许 ±10% 误差 (期望 70/30,实际应在 60-80 / 20-40 之间)
if [ $COUNT_A -ge 60 ] && [ $COUNT_A -le 80 ] && \
   [ $COUNT_B -ge 20 ] && [ $COUNT_B -le 40 ]; then
    test_pass
else
    test_fail "权重分布不符合预期 (期望约 70/30)"
fi

# ==========================================
# 步骤 11: 停用路由规则
# ==========================================
test_step "停用路由规则"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/routing/rules/rule-weighted/unpublish")

SUCCESS=$(echo "$RESPONSE" | jq -r '.success')

if [ "$SUCCESS" = "true" ]; then
    test_pass
else
    test_fail "停用规则失败"
fi

# ==========================================
# 步骤 12: 验证停用后返回所有实例
# ==========================================
test_step "验证停用路由规则后返回所有实例"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/discovery/service.json" \
  -H "Content-Type: application/json" \
  -d "{
    \"discovery_config\": {
      \"service_id\": \"$SERVICE_ID\",
      \"region_id\": \"$REGION_ID\",
      \"zone_id\": \"$ZONE_ID\"
    }
  }")

INSTANCE_COUNT=$(echo "$RESPONSE" | jq -r '.service.instances | length')

if [ "$INSTANCE_COUNT" -eq 5 ]; then
    echo "  实例数: $INSTANCE_COUNT (符合预期)"
    test_pass
else
    test_fail "期望 5 个实例,实际 $INSTANCE_COUNT 个"
fi

# ==========================================
# 步骤 13: 清理测试数据
# ==========================================
test_step "清理测试数据"

# 删除规则
curl -s -X DELETE "$BASE_URL/api/routing/rules/rule-weighted" > /dev/null

# 删除分组 (使用 group_key)
GROUP_A_KEY="$SERVICE_ID:$REGION_ID:$ZONE_ID:group-a"
GROUP_B_KEY="$SERVICE_ID:$REGION_ID:$ZONE_ID:group-b"
curl -s -X DELETE "$BASE_URL/api/routing/groups/$GROUP_A_KEY" > /dev/null
curl -s -X DELETE "$BASE_URL/api/routing/groups/$GROUP_B_KEY" > /dev/null

test_pass

echo -e "${GREEN}集成测试完成!${NC}"
