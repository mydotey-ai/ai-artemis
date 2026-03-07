#!/bin/bash
# Artemis Hybrid Test - Start Rust Only Cluster
# 启动 3 节点 Rust 集群 (纯 Rust 测试)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_DIR="$SCRIPT_DIR/.."
PROJECT_ROOT="$TEST_DIR/.."

echo "======================================"
echo "Starting Rust-Only Artemis Cluster"
echo "======================================"
echo ""

# 检查前置条件
echo "[CHECK] Checking prerequisites..."

# 检查 Rust 二进制
RUST_BIN="$PROJECT_ROOT/target/release/artemis"
if [ ! -f "$RUST_BIN" ]; then
    echo "ERROR: Rust binary not found: $RUST_BIN"
    echo "Please run: ./scripts/setup.sh"
    exit 1
fi
echo "  ✓ Rust binary: $RUST_BIN"

# 创建目录
mkdir -p "$TEST_DIR/logs" "$TEST_DIR/data"

# 清理旧进程
echo ""
echo "[CLEANUP] Checking for existing processes..."
for port in 8084 8085 8086; do
    pid=$(lsof -t -i:$port 2>/dev/null || true)
    if [ -n "$pid" ]; then
        echo "  Killing process on port $port (PID: $pid)"
        kill -9 $pid 2>/dev/null || true
    fi
done
sleep 2

# 启动 Rust 节点
echo ""
echo "======================================"
echo "Starting Rust Nodes (3 nodes)"
echo "======================================"

start_rust_node() {
    local node_num=$1
    local port=$2
    local peer_port=$3
    local log_file="$TEST_DIR/logs/rust-node${node_num}.log"
    local db_path="$TEST_DIR/data/artemis.db"

    echo "[RUST] Starting node $node_num on port $port (peer: $peer_port)..."

    # 创建临时配置文件
    local config_file="$TEST_DIR/logs/rust-node${node_num}.toml"
    cat > "$config_file" <<ENDCONFIG
[server]
node_id = "node-r${node_num}"
listen_addr = "0.0.0.0:${port}"
peer_port = ${peer_port}

[database]
db_type = "sqlite"
url = "sqlite://${db_path}"

[cluster]
enabled = true
peers = [
    "127.0.0.1:8084", "127.0.0.1:8085", "127.0.0.1:8086"
]
ENDCONFIG

    RUST_LOG=info "$RUST_BIN" server \
        --config "$config_file" \
        > "$log_file" 2>&1 &

    local pid=$!
    echo $pid > "$TEST_DIR/logs/rust-node${node_num}.pid"
    echo "  PID: $pid, Log: $log_file"
}

# 启动 3 个 Rust 节点
start_rust_node 1 8084 9094
start_rust_node 2 8085 9095
start_rust_node 3 8086 9096

# 等待节点启动
echo ""
echo "======================================"
echo "Waiting for nodes to start..."
echo "======================================"
sleep 5

# 健康检查
echo ""
echo "[HEALTH CHECK] Checking cluster health..."
ALL_HEALTHY=true
for port in 8084 8085 8086; do
    if curl -s --max-time 2 "http://127.0.0.1:$port/health" > /dev/null 2>&1; then
        echo "  ✓ Port $port: HEALTHY"
    else
        echo "  ✗ Port $port: UNHEALTHY"
        ALL_HEALTHY=false
    fi
done

echo ""
echo "======================================"
if [ "$ALL_HEALTHY" = true ]; then
    echo "Rust cluster started successfully!"
    echo "======================================"
    echo ""
    echo "Next steps:"
    echo "  ./scripts/status.sh          # 查看状态"
else
    echo "WARNING: Some nodes are unhealthy!"
    echo "Check logs in: $TEST_DIR/logs/"
    echo "======================================"
fi
