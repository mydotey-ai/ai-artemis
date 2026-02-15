#!/bin/bash

# ================================================================
# Artemis 管理功能集成测试
# ================================================================
#
# 用途: 测试 Artemis 所有管理功能的完整集成
#
# 测试模块 (6大类,30+步骤):
#   [实例管理] Instance Management (5步)
#   [服务器管理] Server Management (3步)
#   [Zone管理] Zone Management (4步)
#   [金丝雀发布] Canary Deployment (5步)
#   [分组路由] Group Routing (8步)
#   [审计日志] Audit Logs (3步)
#
# 使用方法:
#   ./test-management.sh [module]
#
#   module: 指定测试模块 (可选)
#     - instance    测试实例管理
#     - server      测试服务器管理
#     - zone        测试Zone管理
#     - canary      测试金丝雀发布
#     - routing     测试分组路由
#     - audit       测试审计日志
#     - all         测试所有模块 (默认)
#
# 前置条件: Artemis 集群必须已启动
#   ./cluster.sh start 3
#
# ================================================================

set -e  # 遇到错误立即退出

BASE_URL="http://localhost:8080"
SERVICE_ID="test-mgmt-service"
REGION_ID="us-east"
ZONE_ID="zone-1"
ZONE_ID_2="zone-2"

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# 测试结果统计
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
CURRENT_MODULE=""

# 全局变量
GROUP_ID_CANARY=""
GROUP_ID_STABLE=""
RULE_ID=""

# ================================================================
# 辅助函数
# ================================================================

print_header() {
    echo ""
    echo "========================================="
    echo -e "${CYAN}$1${NC}"
    echo "========================================="
    echo ""
}

print_module() {
    CURRENT_MODULE="$1"
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}  模块: $1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
}

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

# JSON 格式化 (如果有 jq)
format_json() {
    if command -v jq &> /dev/null; then
        jq '.' 2>/dev/null || cat
    else
        cat
    fi
}

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

# ================================================================
# 测试准备
# ================================================================

setup_test_data() {
    print_header "测试数据准备"

    # 先清理旧数据
    GROUP_KEY_CANARY="$SERVICE_ID:$REGION_ID:$ZONE_ID:physical"
    GROUP_KEY_STABLE="$SERVICE_ID:$REGION_ID:$ZONE_ID:logical"
    curl -s -X DELETE "$BASE_URL/api/routing/rules/route-weighted-test" > /dev/null 2>&1
    curl -s -X DELETE "$BASE_URL/api/routing/groups/$GROUP_KEY_CANARY" > /dev/null 2>&1
    curl -s -X DELETE "$BASE_URL/api/routing/groups/$GROUP_KEY_STABLE" > /dev/null 2>&1

    test_step "注册测试服务实例"

    # 注册实例到 zone-1
    for i in {1..3}; do
        curl -s -X POST "$BASE_URL/api/registry/register.json" \
            -H "Content-Type: application/json" \
            -d "{
                \"instances\": [{
                    \"region_id\": \"$REGION_ID\",
                    \"zone_id\": \"$ZONE_ID\",
                    \"service_id\": \"$SERVICE_ID\",
                    \"instance_id\": \"inst-z1-$i\",
                    \"ip\": \"192.168.1.1$i\",
                    \"port\": 808$i,
                    \"status\": \"up\",
                    \"metadata\": {
                        \"zone\": \"zone-1\",
                        \"index\": \"$i\"
                    }
                }]
            }" > /dev/null
    done

    # 注册实例到 zone-2
    for i in {1..2}; do
        curl -s -X POST "$BASE_URL/api/registry/register.json" \
            -H "Content-Type: application/json" \
            -d "{
                \"instances\": [{
                    \"region_id\": \"$REGION_ID\",
                    \"zone_id\": \"$ZONE_ID_2\",
                    \"service_id\": \"$SERVICE_ID\",
                    \"instance_id\": \"inst-z2-$i\",
                    \"ip\": \"192.168.2.1$i\",
                    \"port\": 808$i,
                    \"status\": \"up\",
                    \"metadata\": {
                        \"zone\": \"zone-2\",
                        \"index\": \"$i\"
                    }
                }]
            }" > /dev/null
    done

    test_pass

    # 等待数据同步
    sleep 1
}

# ================================================================
# [模块1] 实例管理测试
# ================================================================

test_instance_management() {
    print_module "实例管理 (Instance Management)"

    # 测试1: 拉出实例
    test_step "拉出单个实例 (Pull Out)"
    RESPONSE=$(curl -s -X POST "$BASE_URL/api/management/instance/operate-instance.json" \
        -H "Content-Type: application/json" \
        -d "{
            \"instance_key\": {
                \"region_id\": \"$REGION_ID\",
                \"zone_id\": \"$ZONE_ID\",
                \"service_id\": \"$SERVICE_ID\",
                \"group_id\": \"default\",
                \"instance_id\": \"inst-z1-1\"
            },
            \"operation\": \"pullout\",
            \"operation_complete\": true,
            \"operator_id\": \"test-user\"
        }")

    if echo "$RESPONSE" | grep -q '"error_code":"success"'; then
        test_pass
    else
        test_fail "拉出实例失败: $RESPONSE"
    fi

    # 测试2: 查询实例状态
    test_step "查询实例是否被拉出"
    RESPONSE=$(curl -s -X POST "$BASE_URL/api/management/instance/is-instance-down.json" \
        -H "Content-Type: application/json" \
        -d "{
            \"instance_key\": {
                \"region_id\": \"$REGION_ID\",
                \"zone_id\": \"$ZONE_ID\",
                \"service_id\": \"$SERVICE_ID\",
                \"group_id\": \"default\",
                \"instance_id\": \"inst-z1-1\"
            }
        }")

    if echo "$RESPONSE" | grep -q '"is_down":true'; then
        test_pass
    else
        test_fail "实例状态查询失败: $RESPONSE"
    fi

    # 测试3: 查询实例操作历史
    test_step "查询实例操作历史"
    RESPONSE=$(curl -s -X POST "$BASE_URL/api/management/instance/get-instance-operations.json" \
        -H "Content-Type: application/json" \
        -d "{
            \"instance_key\": {
                \"region_id\": \"$REGION_ID\",
                \"zone_id\": \"$ZONE_ID\",
                \"service_id\": \"$SERVICE_ID\",
                \"group_id\": \"default\",
                \"instance_id\": \"inst-z1-1\"
            }
        }")

    if echo "$RESPONSE" | grep -q '"pullout"'; then
        test_pass
    else
        test_fail "操作历史查询失败: $RESPONSE"
    fi

    # 测试4: 拉入实例
    test_step "拉入实例 (Pull In)"
    RESPONSE=$(curl -s -X POST "$BASE_URL/api/management/instance/operate-instance.json" \
        -H "Content-Type: application/json" \
        -d "{
            \"instance_key\": {
                \"region_id\": \"$REGION_ID\",
                \"zone_id\": \"$ZONE_ID\",
                \"service_id\": \"$SERVICE_ID\",
                \"group_id\": \"default\",
                \"instance_id\": \"inst-z1-1\"
            },
            \"operation\": \"pullin\",
            \"operation_complete\": true,
            \"operator_id\": \"test-user\"
        }")

    if echo "$RESPONSE" | grep -q '"error_code":"success"'; then
        test_pass
    else
        test_fail "拉入实例失败: $RESPONSE"
    fi

    # 测试5: 验证实例已恢复
    test_step "验证实例已拉入"
    RESPONSE=$(curl -s -X POST "$BASE_URL/api/management/instance/is-instance-down.json" \
        -H "Content-Type: application/json" \
        -d "{
            \"instance_key\": {
                \"region_id\": \"$REGION_ID\",
                \"zone_id\": \"$ZONE_ID\",
                \"service_id\": \"$SERVICE_ID\",
                \"group_id\": \"default\",
                \"instance_id\": \"inst-z1-1\"
            }
        }")

    if echo "$RESPONSE" | grep -q '"is_down":false'; then
        test_pass
    else
        test_fail "实例状态未恢复: $RESPONSE"
    fi
}

# ================================================================
# [模块2] 服务器管理测试
# ================================================================

test_server_management() {
    print_module "服务器管理 (Server Management)"

    # 测试1: 拉出服务器
    test_step "拉出整个服务器 (所有实例)"
    RESPONSE=$(curl -s -X POST "$BASE_URL/api/management/server/operate-server.json" \
        -H "Content-Type: application/json" \
        -d "{
            \"server_id\": \"192.168.1.11\",
            \"region_id\": \"$REGION_ID\",
            \"operation\": \"pullout\",
            \"operation_complete\": true,
            \"operator_id\": \"test-user\"
        }")

    if echo "$RESPONSE" | grep -q '"error_code":"success"'; then
        test_pass
    else
        test_fail "拉出服务器失败: $RESPONSE"
    fi

    # 测试2: 查询服务器状态
    test_step "查询服务器是否被拉出"
    RESPONSE=$(curl -s -X POST "$BASE_URL/api/management/server/is-server-down.json" \
        -H "Content-Type: application/json" \
        -d "{
            \"server_id\": \"192.168.1.11\",
            \"region_id\": \"$REGION_ID\"
        }")

    if echo "$RESPONSE" | grep -q '"is_down":true'; then
        test_pass
    else
        test_fail "服务器状态查询失败: $RESPONSE"
    fi

    # 测试3: 拉入服务器
    test_step "拉入服务器 (恢复所有实例)"
    RESPONSE=$(curl -s -X POST "$BASE_URL/api/management/server/operate-server.json" \
        -H "Content-Type: application/json" \
        -d "{
            \"server_id\": \"192.168.1.11\",
            \"region_id\": \"$REGION_ID\",
            \"operation\": \"pullin\",
            \"operation_complete\": true,
            \"operator_id\": \"test-user\"
        }")

    if echo "$RESPONSE" | grep -q '"error_code":"success"'; then
        test_pass
    else
        test_fail "拉入服务器失败: $RESPONSE"
    fi
}

# ================================================================
# [模块3] Zone管理测试
# ================================================================

test_zone_management() {
    print_module "Zone管理 (Zone Management)"

    # 测试1: 拉出整个Zone
    test_step "拉出整个Zone (所有实例)"
    RESPONSE=$(curl -s -X POST "$BASE_URL/api/management/zone/pull-out" \
        -H "Content-Type: application/json" \
        -d "{
            \"zone_id\": \"$ZONE_ID_2\",
            \"region_id\": \"$REGION_ID\",
            \"operation\": \"pullout\",
            \"operator_id\": \"test-user\"
        }")

    if echo "$RESPONSE" | grep -q '"success":true'; then
        test_pass
    else
        test_fail "拉出Zone失败: $RESPONSE"
    fi

    # 测试2: 查询Zone状态
    test_step "查询Zone状态"
    RESPONSE=$(curl -s "$BASE_URL/api/management/zone/status/$ZONE_ID_2/$REGION_ID")

    if echo "$RESPONSE" | grep -q '"is_down":true'; then
        test_pass
    else
        test_fail "Zone状态查询失败: $RESPONSE"
    fi

    # 测试3: 查询所有Zone操作
    test_step "查询所有Zone操作记录"
    RESPONSE=$(curl -s "$BASE_URL/api/management/zone/operations")

    if echo "$RESPONSE" | grep -q "\"zone_id\":\"$ZONE_ID_2\""; then
        test_pass
    else
        test_fail "Zone操作记录查询失败: $RESPONSE"
    fi

    # 测试4: 拉入Zone
    test_step "拉入Zone (恢复所有实例)"
    RESPONSE=$(curl -s -X POST "$BASE_URL/api/management/zone/pull-in" \
        -H "Content-Type: application/json" \
        -d "{
            \"zone_id\": \"$ZONE_ID_2\",
            \"region_id\": \"$REGION_ID\",
            \"operation\": \"pullin\",
            \"operator_id\": \"test-user\"
        }")

    if echo "$RESPONSE" | grep -q '"success":true'; then
        test_pass
    else
        test_fail "拉入Zone失败: $RESPONSE"
    fi
}

# ================================================================
# [模块4] 金丝雀发布测试
# ================================================================

test_canary_deployment() {
    print_module "金丝雀发布 (Canary Deployment)"

    # 测试1: 创建金丝雀配置
    test_step "创建金丝雀配置 (IP白名单)"
    RESPONSE=$(curl -s -X POST "$BASE_URL/api/management/canary/config" \
        -H "Content-Type: application/json" \
        -d "{
            \"service_id\": \"$SERVICE_ID\",
            \"ip_whitelist\": [\"10.0.0.1\", \"10.0.0.2\"]
        }")

    if echo "$RESPONSE" | grep -q '"success":true'; then
        test_pass
    else
        test_fail "创建金丝雀配置失败: $RESPONSE"
    fi

    # 测试2: 查询金丝雀配置
    test_step "查询金丝雀配置"
    RESPONSE=$(curl -s "$BASE_URL/api/management/canary/config/$SERVICE_ID")

    if echo "$RESPONSE" | grep -q "\"service_id\":\"$SERVICE_ID\""; then
        test_pass
    else
        test_fail "查询金丝雀配置失败: $RESPONSE"
    fi

    # 测试3: 启用金丝雀发布
    test_step "启用金丝雀发布"
    RESPONSE=$(curl -s -X POST "$BASE_URL/api/management/canary/enable" \
        -H "Content-Type: application/json" \
        -d "{
            \"service_id\": \"$SERVICE_ID\",
            \"enabled\": true
        }")

    if echo "$RESPONSE" | grep -q '"success":true'; then
        test_pass
    else
        test_fail "启用金丝雀失败: $RESPONSE"
    fi

    # 测试4: 查询所有金丝雀配置
    test_step "查询所有金丝雀配置"
    RESPONSE=$(curl -s "$BASE_URL/api/management/canary/configs")

    if echo "$RESPONSE" | grep -q "\"service_id\":\"$SERVICE_ID\""; then
        test_pass
    else
        test_fail "查询所有配置失败: $RESPONSE"
    fi

    # 测试5: 删除金丝雀配置
    test_step "删除金丝雀配置"
    RESPONSE=$(curl -s -X DELETE "$BASE_URL/api/management/canary/config/$SERVICE_ID")

    if echo "$RESPONSE" | grep -q '"success":true'; then
        test_pass
    else
        test_fail "删除金丝雀配置失败: $RESPONSE"
    fi
}

# ================================================================
# [模块5] 分组路由测试
# ================================================================

test_group_routing() {
    print_module "分组路由 (Group Routing)"

    # 测试1: 创建服务分组
    test_step "创建金丝雀分组"
    RESPONSE=$(curl -s -X POST "$BASE_URL/api/routing/groups" \
        -H "Content-Type: application/json" \
        -d "{
            \"service_id\": \"$SERVICE_ID\",
            \"region_id\": \"$REGION_ID\",
            \"zone_id\": \"$ZONE_ID\",
            \"name\": \"金丝雀分组\",
            \"group_type\": \"physical\",
            \"description\": \"测试金丝雀实例\"
        }")

    if echo "$RESPONSE" | grep -q '"success":true'; then
        GROUP_ID_CANARY=$(echo "$RESPONSE" | grep -o '"group_id":[0-9]*' | head -1 | cut -d: -f2)
        echo "分组ID: $GROUP_ID_CANARY"
        test_pass
    else
        test_fail "创建金丝雀分组失败: $RESPONSE"
    fi

    test_step "创建稳定分组"
    RESPONSE=$(curl -s -X POST "$BASE_URL/api/routing/groups" \
        -H "Content-Type: application/json" \
        -d "{
            \"service_id\": \"$SERVICE_ID\",
            \"region_id\": \"$REGION_ID\",
            \"zone_id\": \"$ZONE_ID\",
            \"name\": \"稳定分组\",
            \"group_type\": \"logical\",
            \"description\": \"稳定版本实例\"
        }")

    if echo "$RESPONSE" | grep -q '"success":true'; then
        GROUP_ID_STABLE=$(echo "$RESPONSE" | grep -o '"group_id":[0-9]*' | head -1 | cut -d: -f2)
        echo "分组ID: $GROUP_ID_STABLE"
        test_pass
    else
        test_fail "创建稳定分组失败: $RESPONSE"
    fi

    # 测试2: 创建路由规则
    test_step "创建加权路由规则 (canary:10%, stable:90%)"
    RESPONSE=$(curl -s -X POST "$BASE_URL/api/routing/rules" \
        -H "Content-Type: application/json" \
        -d "{
            \"route_id\": \"route-weighted-test\",
            \"service_id\": \"$SERVICE_ID\",
            \"name\": \"加权路由\",
            \"description\": \"测试加权轮询\",
            \"strategy\": \"weighted-round-robin\"
        }")

    if echo "$RESPONSE" | grep -q '"success":true'; then
        RULE_ID=$(echo "$RESPONSE" | grep -o '"route_id":"[^"]*"' | head -1 | cut -d'"' -f4)
        echo "规则ID: $RULE_ID"
        test_pass
    else
        test_fail "创建路由规则失败: $RESPONSE"
    fi

    # 测试3: 添加分组到规则
    test_step "添加金丝雀分组到规则 (权重10)"
    if [ -n "$GROUP_ID_CANARY" ] && [ -n "$RULE_ID" ]; then
        RESPONSE=$(curl -s -X POST "$BASE_URL/api/routing/rules/$RULE_ID/groups" \
            -H "Content-Type: application/json" \
            -d "{
                \"group_id\": \"$GROUP_ID_CANARY\",
                \"weight\": 10
            }")
    else
        RESPONSE='{"success":false,"message":"Missing group_id or rule_id"}'
    fi

    if echo "$RESPONSE" | grep -q '"success":true'; then
        test_pass
    else
        test_fail "添加金丝雀分组失败: $RESPONSE"
    fi

    test_step "添加稳定分组到规则 (权重90)"
    if [ -n "$GROUP_ID_STABLE" ] && [ -n "$RULE_ID" ]; then
        RESPONSE=$(curl -s -X POST "$BASE_URL/api/routing/rules/$RULE_ID/groups" \
            -H "Content-Type: application/json" \
            -d "{
                \"group_id\": \"$GROUP_ID_STABLE\",
                \"weight\": 90
            }")
    else
        RESPONSE='{"success":false,"message":"Missing group_id or rule_id"}'
    fi

    if echo "$RESPONSE" | grep -q '"success":true'; then
        test_pass
    else
        test_fail "添加稳定分组失败: $RESPONSE"
    fi

    # 测试4: 发布路由规则
    test_step "发布路由规则"
    RESPONSE=$(curl -s -X POST "$BASE_URL/api/routing/rules/$RULE_ID/publish")

    if echo "$RESPONSE" | grep -q '"success":true'; then
        test_pass
    else
        test_fail "发布规则失败: $RESPONSE"
    fi

    # 测试5: 查询规则详情
    test_step "查询规则详情"
    RESPONSE=$(curl -s "$BASE_URL/api/routing/rules/$RULE_ID")

    if echo "$RESPONSE" | grep -q '"status":"active"'; then
        test_pass
    else
        test_fail "查询规则失败: $RESPONSE"
    fi

    # 测试6: 停用规则
    test_step "停用路由规则"
    RESPONSE=$(curl -s -X POST "$BASE_URL/api/routing/rules/$RULE_ID/unpublish")

    if echo "$RESPONSE" | grep -q '"success":true'; then
        test_pass
    else
        test_fail "停用规则失败: $RESPONSE"
    fi
}

# ================================================================
# [模块6] 审计日志测试
# ================================================================

test_audit_logs() {
    print_module "审计日志 (Audit Logs)"

    # 测试1: 查询所有审计日志
    test_step "查询所有审计日志"
    RESPONSE=$(curl -s "$BASE_URL/api/management/audit/logs?limit=20")

    if echo "$RESPONSE" | grep -q '"success":true'; then
        LOG_COUNT=$(echo "$RESPONSE" | grep -o '"operation":"[^"]*"' | wc -l)
        echo "找到 $LOG_COUNT 条审计日志 (可能为0)"
        test_pass
    else
        test_fail "查询审计日志失败: $RESPONSE"
    fi

    # 测试2: 查询实例操作日志
    test_step "查询实例操作日志"
    RESPONSE=$(curl -s "$BASE_URL/api/management/audit/instance-logs?region_id=$REGION_ID&zone_id=$ZONE_ID&service_id=$SERVICE_ID&instance_id=inst-z1-1")

    if echo "$RESPONSE" | grep -q '"success":true'; then
        test_pass
    else
        test_fail "查询实例日志失败: $RESPONSE"
    fi

    # 测试3: 查询服务器操作日志
    test_step "查询服务器操作日志"
    RESPONSE=$(curl -s "$BASE_URL/api/management/audit/server-logs?region_id=$REGION_ID&zone_id=$ZONE_ID&ip=192.168.1.11")

    if echo "$RESPONSE" | grep -q '"success":true'; then
        test_pass
    else
        test_fail "查询服务器日志失败: $RESPONSE"
    fi
}

# ================================================================
# 测试清理
# ================================================================

cleanup_test_data() {
    print_header "清理测试数据"

    # 删除路由规则
    if [ -n "$RULE_ID" ]; then
        curl -s -X DELETE "$BASE_URL/api/routing/rules/$RULE_ID" > /dev/null 2>&1
    fi

    # 删除分组 (使用 group_key)
    GROUP_KEY_CANARY="$SERVICE_ID:$REGION_ID:$ZONE_ID:Canary"
    GROUP_KEY_STABLE="$SERVICE_ID:$REGION_ID:$ZONE_ID:Stable"

    curl -s -X DELETE "$BASE_URL/api/routing/groups/$GROUP_KEY_CANARY" > /dev/null 2>&1
    curl -s -X DELETE "$BASE_URL/api/routing/groups/$GROUP_KEY_STABLE" > /dev/null 2>&1

    echo "测试数据已清理"
}

# ================================================================
# 测试总结
# ================================================================

print_summary() {
    echo ""
    echo "========================================="
    echo "测试总结"
    echo "========================================="
    echo "总测试数: $TOTAL_TESTS"
    echo -e "${GREEN}通过: $PASSED_TESTS${NC}"
    echo -e "${RED}失败: $FAILED_TESTS${NC}"
    echo ""

    if [ $FAILED_TESTS -eq 0 ]; then
        echo -e "${GREEN}✓ 所有测试通过! 🎉${NC}"
        exit 0
    else
        echo -e "${RED}✗ 有测试失败 ❌${NC}"
        exit 1
    fi
}

# ================================================================
# 主函数
# ================================================================

main() {
    local module="${1:-all}"

    print_header "Artemis 管理功能集成测试"

    wait_for_server
    setup_test_data

    case "$module" in
        instance)
            test_instance_management
            ;;
        server)
            test_server_management
            ;;
        zone)
            test_zone_management
            ;;
        canary)
            test_canary_deployment
            ;;
        routing)
            test_group_routing
            ;;
        audit)
            test_audit_logs
            ;;
        all)
            test_instance_management
            test_server_management
            test_zone_management
            test_canary_deployment
            test_group_routing
            test_audit_logs
            ;;
        *)
            echo -e "${RED}错误: 未知的测试模块 '$module'${NC}"
            echo ""
            echo "可用模块:"
            echo "  instance    测试实例管理"
            echo "  server      测试服务器管理"
            echo "  zone        测试Zone管理"
            echo "  canary      测试金丝雀发布"
            echo "  routing     测试分组路由"
            echo "  audit       测试审计日志"
            echo "  all         测试所有模块 (默认)"
            exit 1
            ;;
    esac

    cleanup_test_data
    print_summary
}

# 执行主函数
main "$@"
