#!/bin/bash
# Phase 23: 批量复制 API 集成测试

set -e

BASE_URL="http://localhost:8080"
FAILED=0

# 颜色输出
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}=====================================${NC}"
echo -e "${YELLOW}Phase 23: 批量复制 API 测试 (5 APIs)${NC}"
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

# ========== Step 1: 批量注册 ==========
step 1 "测试批量注册 API (Batch Register)"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/replication/registry/batch-register.json" \
  -H "Content-Type: application/json" \
  -H "X-Artemis-Replication: true" \
  -d '{
    "instances": [
      {
        "region_id": "us-east",
        "zone_id": "zone-1",
        "group_id": "",
        "service_id": "batch-test-service",
        "instance_id": "inst-1",
        "ip": "192.168.1.1",
        "port": 8080,
        "url": "http://192.168.1.1:8080",
        "status": "up"
      },
      {
        "region_id": "us-east",
        "zone_id": "zone-1",
        "group_id": "",
        "service_id": "batch-test-service",
        "instance_id": "inst-2",
        "ip": "192.168.1.2",
        "port": 8080,
        "url": "http://192.168.1.2:8080",
        "status": "up"
      },
      {
        "region_id": "us-east",
        "zone_id": "zone-1",
        "group_id": "",
        "service_id": "batch-test-service",
        "instance_id": "inst-3",
        "ip": "192.168.1.3",
        "port": 8080,
        "url": "http://192.168.1.3:8080",
        "status": "up"
      }
    ]
  }')

check_response "$RESPONSE" '"error_code":"success"' "批量注册返回成功"

# 验证实例已注册
DISCOVER_RESPONSE=$(curl -s -X POST "$BASE_URL/api/discovery/service.json" \
  -H "Content-Type: application/json" \
  -d '{
    "discovery_config": {
      "group_id": "",
        "service_id": "batch-test-service",
      "region_id": "us-east",
      "zone_id": "zone-1"
    }
  }')

INST_COUNT=$(echo "$DISCOVER_RESPONSE" | grep -o '"instance_id"' | wc -l)
if [ "$INST_COUNT" -eq 3 ]; then
    echo -e "${GREEN}✓ 批量注册的 3 个实例全部生效${NC}"
else
    echo -e "${RED}✗ 注册的实例数量不正确: $INST_COUNT (期望 3)${NC}"
    FAILED=$((FAILED + 1))
fi

# ========== Step 2: 批量心跳 ==========
step 2 "测试批量心跳 API (Batch Heartbeat)"

sleep 1

RESPONSE=$(curl -s -X POST "$BASE_URL/api/replication/registry/batch-heartbeat.json" \
  -H "Content-Type: application/json" \
  -H "X-Artemis-Replication: true" \
  -d '{
    "instance_keys": [
      {
        "region_id": "us-east",
        "zone_id": "zone-1",
        "group_id": "",
        "service_id": "batch-test-service",
        "instance_id": "inst-1"
      },
      {
        "region_id": "us-east",
        "zone_id": "zone-1",
        "group_id": "",
        "service_id": "batch-test-service",
        "instance_id": "inst-2"
      },
      {
        "region_id": "us-east",
        "zone_id": "zone-1",
        "group_id": "",
        "service_id": "batch-test-service",
        "instance_id": "inst-3"
      }
    ]
  }')

check_response "$RESPONSE" '"error_code":"success"' "批量心跳返回成功"

# ========== Step 3: 测试批量心跳失败场景 ==========
step 3 "测试批量心跳 - 部分实例不存在"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/replication/registry/batch-heartbeat.json" \
  -H "Content-Type: application/json" \
  -H "X-Artemis-Replication: true" \
  -d '{
    "instance_keys": [
      {
        "region_id": "us-east",
        "zone_id": "zone-1",
        "group_id": "",
        "service_id": "batch-test-service",
        "instance_id": "inst-1"
      },
      {
        "region_id": "us-east",
        "zone_id": "zone-1",
        "group_id": "",
        "service_id": "batch-test-service",
        "instance_id": "non-existent"
      }
    ]
  }')

check_response "$RESPONSE" '"failed_instance_keys"' "批量心跳返回失败实例列表"

# ========== Step 4: 增量同步 API (Services Delta) ==========
step 4 "测试增量同步 API (Services Delta)"

TIMESTAMP=$(($(date +%s) * 1000 - 60000))  # 1分钟前的时间戳

RESPONSE=$(curl -s -X POST "$BASE_URL/api/replication/registry/services-delta.json" \
  -H "Content-Type: application/json" \
  -d "{
    \"region_id\": \"us-east\",
    \"zone_id\": \"zone-1\",
    \"since_timestamp\": $TIMESTAMP
  }")

check_response "$RESPONSE" '"error_code":"success"' "增量同步返回成功"
check_response "$RESPONSE" '"services"' "增量同步包含 services 字段"
check_response "$RESPONSE" '"current_timestamp"' "增量同步包含 current_timestamp"

# ========== Step 5: 全量同步 API (Sync Full Data) ==========
step 5 "测试全量同步 API (Sync Full Data)"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/replication/registry/sync-full.json" \
  -H "Content-Type: application/json" \
  -d '{
    "region_id": "us-east",
    "zone_id": "zone-1"
  }')

check_response "$RESPONSE" '"error_code":"success"' "全量同步返回成功"
check_response "$RESPONSE" '"services"' "全量同步包含 services 字段"
check_response "$RESPONSE" '"sync_timestamp"' "全量同步包含 sync_timestamp"

# 验证返回所有服务
SERVICES_COUNT=$(echo "$RESPONSE" | grep -o '"service_id"' | wc -l)
if [ "$SERVICES_COUNT" -ge 1 ]; then
    echo -e "${GREEN}✓ 全量同步返回了服务列表 ($SERVICES_COUNT 个服务)${NC}"
else
    echo -e "${RED}✗ 全量同步未返回服务${NC}"
    FAILED=$((FAILED + 1))
fi

# ========== Step 6: 批量注销 ==========
step 6 "测试批量注销 API (Batch Unregister)"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/replication/registry/batch-unregister.json" \
  -H "Content-Type: application/json" \
  -H "X-Artemis-Replication: true" \
  -d '{
    "instance_keys": [
      {
        "region_id": "us-east",
        "zone_id": "zone-1",
        "group_id": "",
        "service_id": "batch-test-service",
        "instance_id": "inst-1"
      },
      {
        "region_id": "us-east",
        "zone_id": "zone-1",
        "group_id": "",
        "service_id": "batch-test-service",
        "instance_id": "inst-2"
      }
    ]
  }')

check_response "$RESPONSE" '"error_code":"success"' "批量注销返回成功"

# 验证实例已注销
DISCOVER_RESPONSE=$(curl -s -X POST "$BASE_URL/api/discovery/service.json" \
  -H "Content-Type: application/json" \
  -d '{
    "discovery_config": {
      "group_id": "",
        "service_id": "batch-test-service",
      "region_id": "us-east",
      "zone_id": "zone-1"
    }
  }')

REMAINING_COUNT=$(echo "$DISCOVER_RESPONSE" | grep -o '"instance_id"' | wc -l)
if [ "$REMAINING_COUNT" -eq 1 ]; then
    echo -e "${GREEN}✓ 批量注销成功,剩余 1 个实例${NC}"
else
    echo -e "${RED}✗ 批量注销后剩余实例数量不正确: $REMAINING_COUNT (期望 1)${NC}"
    FAILED=$((FAILED + 1))
fi

# ========== Step 7: 验证 X-Artemis-Replication header 检查 ==========
step 7 "验证复制请求必须包含 X-Artemis-Replication header"

STATUS_CODE=$(curl -s -o /dev/null -w "%{http_code}" -X POST "$BASE_URL/api/replication/registry/batch-register.json" \
  -H "Content-Type: application/json" \
  -d '{"instances": []}')

if [ "$STATUS_CODE" -eq 400 ]; then
    echo -e "${GREEN}✓ 缺少 header 时返回 400 Bad Request${NC}"
else
    echo -e "${RED}✗ 缺少 header 时应返回 400,实际返回: $STATUS_CODE${NC}"
    FAILED=$((FAILED + 1))
fi

# ========== Step 8: 清理测试数据 ==========
step 8 "清理测试数据"

curl -s -X POST "$BASE_URL/api/replication/registry/batch-unregister.json" \
  -H "Content-Type: application/json" \
  -H "X-Artemis-Replication: true" \
  -d '{
    "instance_keys": [
      {
        "region_id": "us-east",
        "zone_id": "zone-1",
        "group_id": "",
        "service_id": "batch-test-service",
        "instance_id": "inst-3"
      }
    ]
  }' > /dev/null

echo -e "${GREEN}✓ 清理完成${NC}"

# ========== 测试总结 ==========
echo ""
echo -e "${YELLOW}=====================================${NC}"
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ 所有测试通过! (5/5 APIs tested)${NC}"
    echo -e "${YELLOW}=====================================${NC}"
    exit 0
else
    echo -e "${RED}✗ $FAILED 个测试失败${NC}"
    echo -e "${YELLOW}=====================================${NC}"
    exit 1
fi
