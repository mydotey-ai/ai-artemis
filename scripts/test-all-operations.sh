#!/bin/bash
# Phase 25: 批量操作查询 API 集成测试

set -e

BASE_URL="http://localhost:8080"
FAILED=0

# 颜色输出
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}=====================================${NC}"
echo -e "${YELLOW}Phase 25: 批量操作查询 API 测试 (4 APIs)${NC}"
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

# ========== Step 1: 生成测试数据 - 注册多个实例并执行操作 ==========
step 1 "生成测试数据 - 注册实例并执行拉出操作"

# 注册 us-east 区域的 3 个实例
curl -s -X POST "$BASE_URL/api/registry/register.json" \
  -H "Content-Type: application/json" \
  -d '{
    "instances": [
      {
        "region_id": "us-east",
        "zone_id": "zone-1",
        "service_id": "test-service",
        "instance_id": "inst-1",
        "ip": "192.168.1.1",
        "port": 8080,
        "url": "http://192.168.1.1:8080",
        "status": "up"
      },
      {
        "region_id": "us-east",
        "zone_id": "zone-1",
        "service_id": "test-service",
        "instance_id": "inst-2",
        "ip": "192.168.1.2",
        "port": 8080,
        "url": "http://192.168.1.2:8080",
        "status": "up"
      }
    ]
  }' > /dev/null

# 注册 us-west 区域的 2 个实例
curl -s -X POST "$BASE_URL/api/registry/register.json" \
  -H "Content-Type: application/json" \
  -d '{
    "instances": [
      {
        "region_id": "us-west",
        "zone_id": "zone-1",
        "service_id": "test-service",
        "instance_id": "inst-3",
        "ip": "192.168.2.1",
        "port": 8080,
        "url": "http://192.168.2.1:8080",
        "status": "up"
      }
    ]
  }' > /dev/null

# 拉出 us-east 的两个实例
curl -s -X POST "$BASE_URL/api/management/instance/operate-instance.json" \
  -H "Content-Type: application/json" \
  -d '{
    "instance_key": {
      "region_id": "us-east",
      "zone_id": "zone-1",
      "group_id": "",
      "service_id": "test-service",
      "instance_id": "inst-1"
    },
    "operation": "pullout",
    "operation_complete": true,
    "operator_id": "admin"
  }' > /dev/null

curl -s -X POST "$BASE_URL/api/management/instance/operate-instance.json" \
  -H "Content-Type: application/json" \
  -d '{
    "instance_key": {
      "region_id": "us-east",
      "zone_id": "zone-1",
      "group_id": "",
      "service_id": "test-service",
      "instance_id": "inst-2"
    },
    "operation": "pullout",
    "operation_complete": true,
    "operator_id": "admin"
  }' > /dev/null

# 拉出 us-west 的一个实例
curl -s -X POST "$BASE_URL/api/management/instance/operate-instance.json" \
  -H "Content-Type: application/json" \
  -d '{
    "instance_key": {
      "region_id": "us-west",
      "zone_id": "zone-1",
      "group_id": "",
      "service_id": "test-service",
      "instance_id": "inst-3"
    },
    "operation": "pullout",
    "operation_complete": true,
    "operator_id": "admin"
  }' > /dev/null

# 拉出两台服务器 (us-east 和 us-west 各一台)
curl -s -X POST "$BASE_URL/api/management/server/operate-server.json" \
  -H "Content-Type: application/json" \
  -d '{
    "server_id": "192.168.1.1",
    "region_id": "us-east",
    "operation": "pullout",
    "operation_complete": true,
    "operator_id": "admin"
  }' > /dev/null

curl -s -X POST "$BASE_URL/api/management/server/operate-server.json" \
  -H "Content-Type: application/json" \
  -d '{
    "server_id": "192.168.2.1",
    "region_id": "us-west",
    "operation": "pullout",
    "operation_complete": true,
    "operator_id": "admin"
  }' > /dev/null

sleep 1
echo -e "${GREEN}✓ 测试数据生成完成 (3 个实例操作 + 2 个服务器操作)${NC}"

# ========== Step 2: 测试查询所有实例操作 (POST) ==========
step 2 "测试查询所有实例操作 API (POST)"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/management/all-instance-operations.json" \
  -H "Content-Type: application/json" \
  -d '{}')

check_response "$RESPONSE" '"error_code":"success"' "所有实例操作 API (POST) 返回成功"
check_response "$RESPONSE" '"instance_operation_records"' "包含 instance_operation_records 字段"

# 验证返回了 3 条记录
RECORD_COUNT=$(echo "$RESPONSE" | grep -o '"instance_id"' | wc -l)
if [ "$RECORD_COUNT" -eq 3 ]; then
    echo -e "${GREEN}✓ 返回了 3 条实例操作记录${NC}"
else
    echo -e "${RED}✗ 记录数量不正确: $RECORD_COUNT (期望 3)${NC}"
    FAILED=$((FAILED + 1))
fi

# ========== Step 3: 测试查询所有实例操作 (POST, 按 region_id 过滤) ==========
step 3 "测试查询所有实例操作 API (POST, 按 region_id 过滤)"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/management/all-instance-operations.json" \
  -H "Content-Type: application/json" \
  -d '{
    "region_id": "us-east"
  }')

check_response "$RESPONSE" '"error_code":"success"' "按 region_id 过滤返回成功"

# 验证只返回 us-east 的记录 (2 条)
RECORD_COUNT=$(echo "$RESPONSE" | grep -o '"instance_id"' | wc -l)
if [ "$RECORD_COUNT" -eq 2 ]; then
    echo -e "${GREEN}✓ 按 region_id 过滤返回了 2 条记录${NC}"
else
    echo -e "${RED}✗ 过滤后记录数量不正确: $RECORD_COUNT (期望 2)${NC}"
    FAILED=$((FAILED + 1))
fi

# ========== Step 4: 测试查询所有实例操作 (GET) ==========
step 4 "测试查询所有实例操作 API (GET)"

RESPONSE=$(curl -s -X GET "$BASE_URL/api/management/all-instance-operations.json")

check_response "$RESPONSE" '"error_code":"success"' "所有实例操作 API (GET) 返回成功"
check_response "$RESPONSE" '"instance_operation_records"' "包含 instance_operation_records 字段"

# ========== Step 5: 测试查询所有实例操作 (GET, 带 regionId 参数) ==========
step 5 "测试查询所有实例操作 API (GET, 带 regionId 参数)"

RESPONSE=$(curl -s -X GET "$BASE_URL/api/management/all-instance-operations.json?regionId=us-west")

check_response "$RESPONSE" '"error_code":"success"' "带 regionId 参数的 GET 请求返回成功"

# 验证只返回 us-west 的记录 (1 条)
RECORD_COUNT=$(echo "$RESPONSE" | grep -o '"instance_id"' | wc -l)
if [ "$RECORD_COUNT" -eq 1 ]; then
    echo -e "${GREEN}✓ 按 regionId 过滤返回了 1 条记录${NC}"
else
    echo -e "${RED}✗ 过滤后记录数量不正确: $RECORD_COUNT (期望 1)${NC}"
    FAILED=$((FAILED + 1))
fi

# ========== Step 6: 测试查询所有服务器操作 (POST) ==========
step 6 "测试查询所有服务器操作 API (POST)"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/management/all-server-operations.json" \
  -H "Content-Type: application/json" \
  -d '{}')

check_response "$RESPONSE" '"error_code":"success"' "所有服务器操作 API (POST) 返回成功"
check_response "$RESPONSE" '"server_operation_records"' "包含 server_operation_records 字段"

# 验证返回了 2 条记录
RECORD_COUNT=$(echo "$RESPONSE" | grep -o '"server_id"' | wc -l)
if [ "$RECORD_COUNT" -eq 2 ]; then
    echo -e "${GREEN}✓ 返回了 2 条服务器操作记录${NC}"
else
    echo -e "${RED}✗ 记录数量不正确: $RECORD_COUNT (期望 2)${NC}"
    FAILED=$((FAILED + 1))
fi

# ========== Step 7: 测试查询所有服务器操作 (POST, 按 region_id 过滤) ==========
step 7 "测试查询所有服务器操作 API (POST, 按 region_id 过滤)"

RESPONSE=$(curl -s -X POST "$BASE_URL/api/management/all-server-operations.json" \
  -H "Content-Type: application/json" \
  -d '{
    "region_id": "us-east"
  }')

check_response "$RESPONSE" '"error_code":"success"' "按 region_id 过滤返回成功"

# 验证只返回 us-east 的记录 (1 条)
RECORD_COUNT=$(echo "$RESPONSE" | grep -o '"server_id"' | wc -l)
if [ "$RECORD_COUNT" -eq 1 ]; then
    echo -e "${GREEN}✓ 按 region_id 过滤返回了 1 条记录${NC}"
else
    echo -e "${RED}✗ 过滤后记录数量不正确: $RECORD_COUNT (期望 1)${NC}"
    FAILED=$((FAILED + 1))
fi

# ========== Step 8: 测试查询所有服务器操作 (GET) ==========
step 8 "测试查询所有服务器操作 API (GET)"

RESPONSE=$(curl -s -X GET "$BASE_URL/api/management/all-server-operations.json")

check_response "$RESPONSE" '"error_code":"success"' "所有服务器操作 API (GET) 返回成功"
check_response "$RESPONSE" '"server_operation_records"' "包含 server_operation_records 字段"

# ========== Step 9: 测试查询所有服务器操作 (GET, 带 regionId 参数) ==========
step 9 "测试查询所有服务器操作 API (GET, 带 regionId 参数)"

RESPONSE=$(curl -s -X GET "$BASE_URL/api/management/all-server-operations.json?regionId=us-west")

check_response "$RESPONSE" '"error_code":"success"' "带 regionId 参数的 GET 请求返回成功"

# 验证只返回 us-west 的记录 (1 条)
RECORD_COUNT=$(echo "$RESPONSE" | grep -o '"server_id"' | wc -l)
if [ "$RECORD_COUNT" -eq 1 ]; then
    echo -e "${GREEN}✓ 按 regionId 过滤返回了 1 条记录${NC}"
else
    echo -e "${RED}✗ 过滤后记录数量不正确: $RECORD_COUNT (期望 1)${NC}"
    FAILED=$((FAILED + 1))
fi

# ========== Step 10: 验证所有 4 个 API 都能正常工作 ==========
step 10 "验证所有 4 个 API 都能正常工作"

# POST 查询所有实例操作
RESPONSE1=$(curl -s -X POST "$BASE_URL/api/management/all-instance-operations.json" \
  -H "Content-Type: application/json" \
  -d '{}')

# GET 查询所有实例操作
RESPONSE2=$(curl -s -X GET "$BASE_URL/api/management/all-instance-operations.json")

# POST 查询所有服务器操作
RESPONSE3=$(curl -s -X POST "$BASE_URL/api/management/all-server-operations.json" \
  -H "Content-Type: application/json" \
  -d '{}')

# GET 查询所有服务器操作
RESPONSE4=$(curl -s -X GET "$BASE_URL/api/management/all-server-operations.json")

ALL_SUCCESS=true
if ! echo "$RESPONSE1" | grep -q '"error_code":"success"'; then ALL_SUCCESS=false; fi
if ! echo "$RESPONSE2" | grep -q '"error_code":"success"'; then ALL_SUCCESS=false; fi
if ! echo "$RESPONSE3" | grep -q '"error_code":"success"'; then ALL_SUCCESS=false; fi
if ! echo "$RESPONSE4" | grep -q '"error_code":"success"'; then ALL_SUCCESS=false; fi

if [ "$ALL_SUCCESS" = true ]; then
    echo -e "${GREEN}✓ 所有 4 个 API 都返回成功${NC}"
else
    echo -e "${RED}✗ 部分 API 返回失败${NC}"
    FAILED=$((FAILED + 1))
fi

# ========== Step 11: 清理测试数据 ==========
step 11 "清理测试数据"

# 拉入所有实例
curl -s -X POST "$BASE_URL/api/management/instance/operate-instance.json" \
  -H "Content-Type: application/json" \
  -d '{
    "instance_key": {
      "region_id": "us-east",
      "zone_id": "zone-1",
      "group_id": "",
      "service_id": "test-service",
      "instance_id": "inst-1"
    },
    "operation": "pullin",
    "operation_complete": true,
    "operator_id": "admin"
  }' > /dev/null

curl -s -X POST "$BASE_URL/api/management/instance/operate-instance.json" \
  -H "Content-Type: application/json" \
  -d '{
    "instance_key": {
      "region_id": "us-east",
      "zone_id": "zone-1",
      "group_id": "",
      "service_id": "test-service",
      "instance_id": "inst-2"
    },
    "operation": "pullin",
    "operation_complete": true,
    "operator_id": "admin"
  }' > /dev/null

curl -s -X POST "$BASE_URL/api/management/instance/operate-instance.json" \
  -H "Content-Type: application/json" \
  -d '{
    "instance_key": {
      "region_id": "us-west",
      "zone_id": "zone-1",
      "group_id": "",
      "service_id": "test-service",
      "instance_id": "inst-3"
    },
    "operation": "pullin",
    "operation_complete": true,
    "operator_id": "admin"
  }' > /dev/null

# 拉入所有服务器
curl -s -X POST "$BASE_URL/api/management/server/operate-server.json" \
  -H "Content-Type: application/json" \
  -d '{
    "server_id": "192.168.1.1",
    "region_id": "us-east",
    "operation": "pullin",
    "operation_complete": true,
    "operator_id": "admin"
  }' > /dev/null

curl -s -X POST "$BASE_URL/api/management/server/operate-server.json" \
  -H "Content-Type: application/json" \
  -d '{
    "server_id": "192.168.2.1",
    "region_id": "us-west",
    "operation": "pullin",
    "operation_complete": true,
    "operator_id": "admin"
  }' > /dev/null

# 注销所有实例
curl -s -X POST "$BASE_URL/api/registry/unregister.json" \
  -H "Content-Type: application/json" \
  -d '{
    "instance_keys": [
      {
        "region_id": "us-east",
        "zone_id": "zone-1",
        "group_id": "",
        "service_id": "test-service",
        "instance_id": "inst-1"
      },
      {
        "region_id": "us-east",
        "zone_id": "zone-1",
        "group_id": "",
        "service_id": "test-service",
        "instance_id": "inst-2"
      },
      {
        "region_id": "us-west",
        "zone_id": "zone-1",
        "group_id": "",
        "service_id": "test-service",
        "instance_id": "inst-3"
      }
    ]
  }' > /dev/null

echo -e "${GREEN}✓ 清理完成${NC}"

# ========== 测试总结 ==========
echo ""
echo -e "${YELLOW}=====================================${NC}"
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ 所有测试通过! (4/4 APIs tested)${NC}"
    echo -e "${YELLOW}=====================================${NC}"
    exit 0
else
    echo -e "${RED}✗ $FAILED 个测试失败${NC}"
    echo -e "${YELLOW}=====================================${NC}"
    exit 1
fi
