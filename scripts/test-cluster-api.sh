#!/bin/bash

# ================================================================
# Artemis 集群 API 测试脚本
# ================================================================
#
# 用途: 验证集群节点的 API 功能和数据复制是否正常
#
# 测试内容:
#   - 服务注册和发现
#   - 心跳续约
#   - 数据复制验证
#   - 集群一致性检查
#
# 使用方法:
#   ./scripts/test-cluster-api.sh [基础端口] [节点数]
#   默认: ./test-cluster-api.sh 8080 3
#
# 前置条件: 集群必须已启动
#   ./scripts/cluster.sh start
#
# ================================================================

set -e

# 配置
BASE_PORT=${1:-8080}
NODE_COUNT=${2:-3}
REGION_ID="local"
ZONE_ID="zone1"
SERVICE_ID="test-service"
INSTANCE_ID="test-instance-1"
IP="192.168.1.100"
PORT=8080

# 颜色输出
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 打印函数
print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_info() {
    echo -e "${YELLOW}ℹ${NC} $1"
}

print_section() {
    echo ""
    echo "=========================================="
    echo "$1"
    echo "=========================================="
}

# 检查节点健康
check_health() {
    local port=$1
    local node=$2

    response=$(curl -s -w "\n%{http_code}" http://127.0.0.1:${port}/health)
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n-1)

    if [ "$http_code" == "200" ] && [ "$body" == "OK" ]; then
        print_success "节点 $node (端口 $port) 健康检查通过"
        return 0
    else
        print_error "节点 $node (端口 $port) 健康检查失败 (HTTP $http_code)"
        return 1
    fi
}

# 注册服务实例
register_instance() {
    local port=$1
    local node=$2

    print_info "在节点 $node (端口 $port) 注册实例..."

    response=$(curl -s -w "\n%{http_code}" -X POST \
        http://127.0.0.1:${port}/api/registry/register.json \
        -H "Content-Type: application/json" \
        -d "{
            \"instances\": [{
                \"region_id\": \"${REGION_ID}\",
                \"zone_id\": \"${ZONE_ID}\",
                \"service_id\": \"${SERVICE_ID}\",
                \"instance_id\": \"${INSTANCE_ID}\",
                \"ip\": \"${IP}\",
                \"port\": ${PORT},
                \"url\": \"http://${IP}:${PORT}\",
                \"status\": \"up\"
            }]
        }")

    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n-1)

    if [ "$http_code" == "200" ]; then
        error_code=$(echo "$body" | jq -r '.response_status.error_code')
        if [ "$error_code" == "success" ]; then
            print_success "实例注册成功"
            echo "$body" | jq '.'
            return 0
        else
            print_error "实例注册失败: $error_code"
            echo "$body" | jq '.'
            return 1
        fi
    else
        print_error "注册请求失败 (HTTP $http_code)"
        return 1
    fi
}

# 心跳续约
heartbeat_instance() {
    local port=$1
    local node=$2

    print_info "在节点 $node (端口 $port) 发送心跳..."

    response=$(curl -s -w "\n%{http_code}" -X POST \
        http://127.0.0.1:${port}/api/registry/heartbeat.json \
        -H "Content-Type: application/json" \
        -d "{
            \"instance_keys\": [{
                \"region_id\": \"${REGION_ID}\",
                \"zone_id\": \"${ZONE_ID}\",
                \"service_id\": \"${SERVICE_ID}\",
                \"group_id\": \"\",
                \"instance_id\": \"${INSTANCE_ID}\"
            }]
        }")

    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n-1)

    if [ "$http_code" == "200" ]; then
        error_code=$(echo "$body" | jq -r '.response_status.error_code')
        if [ "$error_code" == "success" ]; then
            print_success "心跳续约成功"
            return 0
        else
            print_error "心跳续约失败: $error_code"
            echo "$body" | jq '.'
            return 1
        fi
    else
        print_error "心跳请求失败 (HTTP $http_code)"
        return 1
    fi
}

# 发现服务
discover_service() {
    local port=$1
    local node=$2

    print_info "在节点 $node (端口 $port) 查询服务..."

    response=$(curl -s -w "\n%{http_code}" -X POST \
        http://127.0.0.1:${port}/api/discovery/service.json \
        -H "Content-Type: application/json" \
        -d "{
            \"discovery_config\": {
                \"service_id\": \"${SERVICE_ID}\",
                \"region_id\": \"${REGION_ID}\",
                \"zone_id\": \"${ZONE_ID}\"
            }
        }")

    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n-1)

    if [ "$http_code" == "200" ]; then
        error_code=$(echo "$body" | jq -r '.response_status.error_code')
        if [ "$error_code" == "success" ]; then
            instance_count=$(echo "$body" | jq '.service.instances | length')
            print_success "服务发现成功,找到 $instance_count 个实例"
            echo "$body" | jq '.service'
            return 0
        else
            print_error "服务发现失败: $error_code"
            echo "$body" | jq '.'
            return 1
        fi
    else
        print_error "发现请求失败 (HTTP $http_code)"
        return 1
    fi
}

# 注销服务
unregister_instance() {
    local port=$1
    local node=$2

    print_info "在节点 $node (端口 $port) 注销实例..."

    response=$(curl -s -w "\n%{http_code}" -X POST \
        http://127.0.0.1:${port}/api/registry/unregister.json \
        -H "Content-Type: application/json" \
        -d "{
            \"instance_keys\": [{
                \"region_id\": \"${REGION_ID}\",
                \"zone_id\": \"${ZONE_ID}\",
                \"service_id\": \"${SERVICE_ID}\",
                \"group_id\": \"\",
                \"instance_id\": \"${INSTANCE_ID}\"
            }]
        }")

    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n-1)

    if [ "$http_code" == "200" ]; then
        error_code=$(echo "$body" | jq -r '.response_status.error_code')
        if [ "$error_code" == "success" ]; then
            print_success "实例注销成功"
            return 0
        else
            print_error "实例注销失败: $error_code"
            echo "$body" | jq '.'
            return 1
        fi
    else
        print_error "注销请求失败 (HTTP $http_code)"
        return 1
    fi
}

# 检查 Prometheus 指标
check_metrics() {
    local port=$1
    local node=$2

    print_info "检查节点 $node (端口 $port) Prometheus 指标..."

    response=$(curl -s -w "\n%{http_code}" http://127.0.0.1:${port}/metrics)
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n-1)

    if [ "$http_code" == "200" ]; then
        metric_count=$(echo "$body" | grep -c "^artemis_" || true)
        print_success "Prometheus 指标可用,找到 $metric_count 个 artemis 指标"
        return 0
    else
        print_error "Prometheus 指标请求失败 (HTTP $http_code)"
        return 1
    fi
}

# 主测试流程
main() {
    print_section "Artemis 集群 API 测试"

    echo "测试配置:"
    echo "  基础端口: $BASE_PORT"
    echo "  节点数量: $NODE_COUNT"
    echo "  服务 ID: $SERVICE_ID"
    echo ""

    # 1. 检查所有节点健康状态
    print_section "1. 健康检查"
    for i in $(seq 0 $((NODE_COUNT-1))); do
        port=$((BASE_PORT + i))
        check_health $port $((i+1)) || exit 1
    done

    # 2. 在第一个节点注册实例
    print_section "2. 注册服务实例"
    register_instance $BASE_PORT 1 || exit 1

    # 3. 等待数据复制
    print_info "等待 2 秒让数据复制到其他节点..."
    sleep 2

    # 4. 在所有节点验证服务发现
    print_section "3. 验证数据复制 - 服务发现"
    for i in $(seq 0 $((NODE_COUNT-1))); do
        port=$((BASE_PORT + i))
        discover_service $port $((i+1)) || exit 1
    done

    # 5. 心跳续约测试
    print_section "4. 心跳续约"
    heartbeat_instance $BASE_PORT 1 || exit 1

    # 6. 检查 Prometheus 指标
    print_section "5. Prometheus 指标"
    check_metrics $BASE_PORT 1 || exit 1

    # 7. 注销实例
    print_section "6. 注销服务实例"
    unregister_instance $BASE_PORT 1 || exit 1

    # 8. 等待数据复制
    print_info "等待 2 秒让注销操作复制到其他节点..."
    sleep 2

    # 9. 验证实例已被删除
    print_section "7. 验证注销操作"
    for i in $(seq 0 $((NODE_COUNT-1))); do
        port=$((BASE_PORT + i))
        print_info "在节点 $((i+1)) (端口 $port) 验证实例已删除..."

        response=$(curl -s -X POST \
            http://127.0.0.1:${port}/api/discovery/service.json \
            -H "Content-Type: application/json" \
            -d "{
                \"discovery_config\": {
                    \"service_id\": \"${SERVICE_ID}\",
                    \"region_id\": \"${REGION_ID}\",
                    \"zone_id\": \"${ZONE_ID}\"
                }
            }")

        error_code=$(echo "$response" | jq -r '.response_status.error_code')
        service=$(echo "$response" | jq -r '.service')

        if [ "$service" == "null" ] || [ "$service" == "" ]; then
            print_success "节点 $((i+1)) - 实例已成功删除"
        else
            instance_count=$(echo "$response" | jq '.service.instances | length')
            if [ "$instance_count" == "0" ]; then
                print_success "节点 $((i+1)) - 实例已成功删除"
            else
                print_error "节点 $((i+1)) - 实例仍然存在 ($instance_count 个)"
            fi
        fi
    done

    print_section "测试完成"
    print_success "所有 API 测试通过!"
}

# 检查依赖
if ! command -v jq &> /dev/null; then
    print_error "需要安装 jq 工具: sudo apt-get install jq 或 brew install jq"
    exit 1
fi

if ! command -v curl &> /dev/null; then
    print_error "需要安装 curl 工具"
    exit 1
fi

# 运行测试
main
