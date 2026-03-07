#!/bin/bash
# Simple Rust Node Test
# 用于测试 Rust 节点是否正常工作

set -e

# 路径配置
TEST_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$TEST_DIR/.."
DATA_DIR="$TEST_DIR/data"
RUST_BIN="$PROJECT_ROOT/target/release/artemis"

echo "======================================"
echo "Simple Rust Node Test"
echo "======================================"
echo ""

# 检查 Rust 二进制
if [ ! -f "$RUST_BIN" ]; then
    echo "ERROR: Rust binary not found: $RUST_BIN"
    exit 1
fi
echo "  ✓ Rust binary: $RUST_BIN"

# 清理旧进程
echo ""
echo "[CLEANUP] Checking for existing processes..."
for port in 9000 9001 9002; do
    pid=$(lsof -t -i:$port 2>/dev/null || true)
    if [ -n "$pid" ]; then
        echo "  Killing process on port $port (PID: $pid)"
        kill -9 $pid 2>/dev/null || true
    fi
done
sleep 2

# 创建目录
mkdir -p "$TEST_DIR/logs" "$DATA_DIR"

# 启动 3 个 Rust 节点
start_rust_node() {
    local node_num=$1
    local port=$2
    local peer_port=$3
    local log_file="$TEST_DIR/logs/rust-node${node_num}.log"
    local config_file="$TEST_DIR/logs/rust-node${node_num}.toml"
    local db_path="$DATA_DIR/artemis.db"

    echo "[RUST] Starting node $node_num on port ${port} (peer: ${peer_port})..."

    # 创建配置文件
    cat > "$config_file" <<ENDCONFIG
[server]
node_id = "node-r${node_num}"
listen_addr = "0.0.0.0:${port}"
peer_port = ${peer_port}

[database]
db_type = "sqlite"
url = "sqlite:${db_path}"

[cluster]
enabled = true
peers = [
    "127.0.0.1:9000", "127.0.0.1:9001", "127.0.0.1:9002"
]
ENDCONFIG

    RUST_LOG=info "$RUST_BIN" server --config "$config_file" > "$log_file" 2>&1 &

    local pid=$!
    echo $pid > "$TEST_DIR/logs/rust-node${node_num}.pid"
    echo "  PID: $pid, Log: $log_file"
}

start_rust_node 1 9000 9001
start_rust_node 2 9001 9002
start_rust_node 3 9002 9003

# 等待节点启动
echo ""
echo "======================================"
echo "Waiting for nodes to start..."
echo "======================================"
sleep 8

# 健康检查
echo ""
echo "[HEALTH CHECK] Checking cluster health..."
ALL_HEALTHY=true
for port in 9000 9001 9002; do
    if curl -s --max-time 2 "http://127.0.0.1:$port/health" > /dev/null 2>&1; then
        echo "  ✓ Port ${port}: HEALTHY"
    else
        echo "  ✗ Port ${port}: UNHEALTHY"
        ALL_HEALTHY=false
    fi
done

echo ""
echo "======================================"
if [ "$ALL_HEALTHY" = true ]; then
    echo "All Rust nodes started successfully!"
    echo "======================================"
    echo ""
    echo "Node 1: http://127.0.0.1:9000"
    echo "Node 2: http://127.0.0.1:9001"
    echo "Node 3: http://127.0.0.1:9002"
    echo ""
    echo "Testing API endpoints..."
    echo "  curl -s http://127.0.0.1:9000/health"
    echo "  curl -s. http://127.0.0.1:9000/api/cluster/status"
    echo " curl -s http://127.0.0.1:9000/api/registry/services"
    echo " curl -s http://127.0.0.1:9000/api/registry/instances"
else
    echo "WARNING: Some nodes are unhealthy!"
    echo "======================================"
fi
