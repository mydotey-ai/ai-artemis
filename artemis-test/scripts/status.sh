#!/bin/bash
# Artemis Hybrid Test - Status Check
# 检查所有组件的运行状态

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_DIR="$SCRIPT_DIR/.."

echo "======================================"
echo "Artemis Hybrid Test - Status"
echo "======================================"
echo ""

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 检查进程函数
check_process_by_pidfile() {
    local pid_file=$1
    local name=$2

    if [ -f "$pid_file" ]; then
        local pid=$(cat "$pid_file")
        if kill -0 "$pid" 2>/dev/null; then
            echo -e "${GREEN}✓${NC} $name (PID: $pid)"
            return 0
        else
            echo -e "${RED}✗${NC} $name (PID: $pid, DEAD)"
            return 1
        fi
    else
        echo -e "${YELLOW}○${NC} $name (NOT STARTED)"
        return 2
    fi
}

check_process_by_port() {
    local port=$1
    local name=$2
    local health_url=${3:-""}

    local pid=$(lsof -t -i:$port 2>/dev/null || echo "")
    if [ -n "$pid" ]; then
        if [ -n "$health_url" ]; then
            if curl -s --max-time 2 "$health_url" > /dev/null 2>&1; then
                echo -e "${GREEN}✓${NC} $name (port: $port, PID: $pid, HEALTHY)"
            else
                echo -e "${YELLOW}!${NC} $name (port: $port, PID: $pid, UNHEALTHY)"
            fi
        else
            echo -e "${GREEN}✓${NC} $name (port: $port, PID: $pid)"
        fi
    else
        echo -e "${RED}✗${NC} $name (port: $port, NOT RUNNING)"
    fi
}

# 集群节点状态
# Java 节点使用 /api/status/node.json (已统一上下文路径)
# Rust 节点使用 /health
echo "=== Cluster Nodes ==="
check_process_by_port 8081 "Java Node 1" "http://localhost:8081/api/status/node.json"
check_process_by_port 8082 "Java Node 2" "http://localhost:8082/api/status/node.json"
check_process_by_port 8083 "Java Node 3" "http://localhost:8083/api/status/node.json"
check_process_by_port 8084 "Rust Node 1" "http://localhost:8084/health"
check_process_by_port 8085 "Rust Node 2" "http://localhost:8085/health"
check_process_by_port 8086 "Rust Node 3" "http://localhost:8086/health"

echo ""
echo "=== Service Providers ==="
check_process_by_port 8087 "Java Provider 1"
check_process_by_port 8088 "Java Provider 2"
check_process_by_port 8089 "Rust Provider 1"
check_process_by_port 8090 "Rust Provider 2"

echo ""
echo "=== Web Console ==="
check_process_by_port 5173 "Web Console"

echo ""
echo "=== Consumers (Background Jobs) ==="
for i in 1 2; do
    check_process_by_pidfile "$TEST_DIR/logs/java-consumer${i}.pid" "Java Consumer $i"
done
for i in 1 2; do
    check_process_by_pidfile "$TEST_DIR/logs/rust-consumer${i}.pid" "Rust Consumer $i"
done

echo ""
echo "======================================"
echo "Summary"
echo "======================================"

# 统计
TOTAL_NODES=6
HEALTHY_NODES=0
for port in 8081 8082 8083 8084 8085 8086; do
    if curl -s --max-time 2 "http://localhost:$port/health" > /dev/null 2>&1; then
        HEALTHY_NODES=$((HEALTHY_NODES + 1))
    fi
done

echo "Cluster: $HEALTHY_NODES/$TOTAL_NODES nodes healthy"

# Provider 统计
HEALTHY_PROVIDERS=0
for port in 8087 8088 8089 8090; do
    if curl -s --max-time 2 "http://localhost:$port/sayHello" > /dev/null 2>&1; then
        HEALTHY_PROVIDERS=$((HEALTHY_PROVIDERS + 1))
    fi
done

echo "Providers: $HEALTHY_PROVIDERS/4 available"

# 服务发现检查
# Java 节点使用 /api/discovery/service.json?serviceId=xxx
# Rust 节点使用 /api/discovery/service?serviceId=xxx
if [ $HEALTHY_NODES -gt 0 ]; then
    echo ""
    echo "[SERVICE DISCOVERY]"
    # 先检查 Rust 节点
    for port in 8084 8085 8086; do
        if curl -s --max-time 2 "http://localhost:$port/health" > /dev/null 2>&1; then
            INSTANCES=$(curl -s --max-time 2 "http://localhost:$port/api/discovery/service?serviceId=hybrid-test-hello-service" 2>/dev/null | grep -o '"instance_id"' | wc -l)
            echo "  Rust Node $port: $INSTANCES instances"
        fi
    done
    # 再检查 Java 节点
    for port in 8081 8082 8083; do
        if curl -s --max-time 2 "http://localhost:$port/api/status/node.json" > /dev/null 2>&1; then
            INSTANCES=$(curl -s --max-time 2 "http://localhost:$port/api/discovery/service.json?serviceId=hybrid-test-hello-service" 2>/dev/null | grep -o '"instanceId"' | wc -l)
            echo "  Java Node $port: $INSTANCES instances"
        fi
    done
fi

echo ""
echo "======================================"
echo ""
echo "Useful commands:"
echo "  ./scripts/cleanup.sh    # Stop all processes"
echo "  tail -f logs/*.log      # View all logs"
echo ""
