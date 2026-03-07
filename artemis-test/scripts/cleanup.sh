#!/bin/bash
# Artemis Hybrid Test - Cleanup
# 停止所有进程并清理资源

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_DIR="$SCRIPT_DIR/.."

echo "======================================"
echo "Artemis Hybrid Test - Cleanup"
echo "======================================"
echo ""

# 函数：停止由 PID 文件管理的进程
stop_by_pidfile() {
    local pid_file=$1
    local name=$2

    if [ -f "$pid_file" ]; then
        local pid=$(cat "$pid_file")
        if kill -0 "$pid" 2>/dev/null; then
            echo "  Stopping $name (PID: $pid)..."
            kill -TERM "$pid" 2>/dev/null || true
            sleep 1
            if kill -0 "$pid" 2>/dev/null; then
                echo "  Force killing $name..."
                kill -9 "$pid" 2>/dev/null || true
            fi
        fi
        rm -f "$pid_file"
    fi
}

# 函数：停止占用特定端口的进程
stop_by_port() {
    local port=$1
    local name=$2

    local pid=$(lsof -t -i:$port 2>/dev/null || echo "")
    if [ -n "$pid" ]; then
        echo "  Stopping $name on port $port (PID: $pid)..."
        kill -TERM $pid 2>/dev/null || true
        sleep 1
        if kill -0 $pid 2>/dev/null; then
            kill -9 $pid 2>/dev/null || true
        fi
    fi
}

echo "[1/4] Stopping managed processes..."

# 停止集群节点（通过 PID 文件）
for i in 1 2 3; do
    stop_by_pidfile "$TEST_DIR/logs/java-node${i}.pid" "Java Node $i"
    stop_by_pidfile "$TEST_DIR/logs/rust-node${i}.pid" "Rust Node $i"
done

# 停止测试应用
stop_by_pidfile "$TEST_DIR/logs/java-provider1.pid" "Java Provider 1"
stop_by_pidfile "$TEST_DIR/logs/java-provider2.pid" "Java Provider 2"
stop_by_pidfile "$TEST_DIR/logs/rust-provider1.pid" "Rust Provider 1"
stop_by_pidfile "$TEST_DIR/logs/rust-provider2.pid" "Rust Provider 2"

for i in 1 2; do
    stop_by_pidfile "$TEST_DIR/logs/java-consumer$i.pid" "Java Consumer $i"
    stop_by_pidfile "$TEST_DIR/logs/rust-consumer$i.pid" "Rust Consumer $i"
done

# 停止 Console
stop_by_pidfile "$TEST_DIR/logs/console.pid" "Web Console"

echo ""
echo "[2/4] Stopping any remaining processes by port..."

# 集群节点端口
for port in 8081 8082 8083 8084 8085 8086; do
    stop_by_port $port "Node"
done

# Provider 端口
for port in 8087 8088 8089 8090; do
    stop_by_port $port "Provider"
done

# Console 端口
stop_by_port 5173 "Web Console"

echo ""
echo "[3/4] Cleaning up temporary files..."

# 删除 PID 文件
rm -f "$TEST_DIR"/logs/*.pid

# 保留日志文件，但清理空的日志文件
find "$TEST_DIR/logs" -name "*.log" -size 0 -delete 2>/dev/null || true

echo "  ✓ Temporary files cleaned"

echo ""
echo "[4/4] Final status check..."

# 检查是否还有进程在运行
RUNNING_PROCESSES=0
for port in 8081 8082 8083 8084 8085 8086 8087 8088 8089 8090 5173; do
    if lsof -Pi :$port -sTCP:LISTEN -t > /dev/null 2>&1; then
        RUNNING_PROCESSES=$((RUNNING_PROCESSES + 1))
    fi
done

if [ $RUNNING_PROCESSES -eq 0 ]; then
    echo "  ✓ All processes stopped successfully"
else
    echo "  ⚠ $RUNNING_PROCESSES processes may still be running"
fi

echo ""
echo "======================================"
echo "Cleanup Complete!"
echo "======================================"
echo ""
echo "Log files preserved in: $TEST_DIR/logs/"
echo ""
echo "To restart the test environment:"
echo "  ./scripts/start-cluster.sh    # 启动集群"
echo "  ./scripts/start-apps.sh       # 启动测试应用"
