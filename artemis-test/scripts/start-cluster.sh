#!/bin/bash
# Artemis Hybrid Test - Start Cluster
# 启动 6 节点混合集群 (3 Java + 3 Rust)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PROJECT_ROOT="$(cd "$TEST_DIR/.." && pwd)"
SHARED_DATA_DIR="/tmp/artemis-test-shared"

echo "======================================"
echo "Starting Artemis Hybrid Cluster"
echo "======================================"
echo ""

echo "[CHECK] Checking prerequisites..."

JAVA_JAR="$PROJECT_ROOT/artemis-java/artemis-package/target/artemis-2.0.2.jar"
if [ ! -f "$JAVA_JAR" ]; then
    echo "ERROR: Java JAR not found: $JAVA_JAR"
    echo "Please run: ./scripts/setup.sh"
    exit 1
fi
echo "  Java JAR: $JAVA_JAR"

RUST_BIN="$PROJECT_ROOT/target/release/artemis"
if [ ! -f "$RUST_BIN" ]; then
    echo "ERROR: Rust binary not found: $RUST_BIN"
    echo "Please run: ./scripts/setup.sh"
    exit 1
fi
echo "  Rust binary: $RUST_BIN"

mkdir -p "$TEST_DIR/logs" "$SHARED_DATA_DIR"

# 初始化共享数据库 (使用 Java 表结构)
if [ ! -f "$SHARED_DATA_DIR/artemis.db" ]; then
    echo "[INIT] Initializing shared database..."
    sqlite3 "$SHARED_DATA_DIR/artemis.db" < "$TEST_DIR/data/init-sqlite.sql" 2>/dev/null || true
fi

echo ""
echo "[CLEANUP] Checking for existing processes..."
for port in 8081 8082 8083 8084 8085 8086; do
    pid=$(lsof -t -i:$port 2>/dev/null || true)
    if [ -n "$pid" ]; then
        echo "  Killing process on port $port (PID: $pid)"
        kill -9 $pid 2>/dev/null || true
    fi
done
sleep 2

echo ""
echo "======================================"
echo "Starting Java Nodes (3 nodes)"
echo "======================================"

start_java_node() {
    local node_num=$1
    local port=$2
    local peer_port=$3
    local log_file="$TEST_DIR/logs/java-node${node_num}.log"

    echo "[JAVA] Starting node $node_num on port $port (peer: $peer_port)..."

    # 格式化: http://127.0.0.1:8081,http://127.0.0.1:8082,...
    local peers=""
    for p in 8081 8082 8083 8084 8085 8086; do
        if [ $p -ne $port ]; then
            peers="$peers,http://127.0.0.1:$p"
        fi
    done
    peers="${peers#,}"  # 移除前导逗号

    local db_url="jdbc:sqlite:$SHARED_DATA_DIR/artemis.db"

    ARTEMIS_DB_URL="$db_url" ARTEMIS_DB_DRIVER="org.sqlite.JDBC" java \
        -Dserver.port=$port \
        -Dapp.port=$port \
        -Dartemis.service.cluster.nodes="$peers" \
        -Dlogging.file.name="$log_file" \
        -jar "$JAVA_JAR" \
        > "$log_file" 2>&1 &

    local pid=$!
    echo $pid > "$TEST_DIR/logs/java-node${node_num}.pid"
    echo "  PID: $pid, Log: $log_file"
}

start_java_node 1 8081 9091
start_java_node 2 8082 9092
start_java_node 3 8083 9093

echo ""
echo "======================================"
echo "Starting Rust Nodes (3 nodes)"
echo "======================================"

start_rust_node() {
    local node_num=$1
    local port=$2
    local peer_port=$3
    local log_file="$TEST_DIR/logs/rust-node${node_num}.log"
    local config_file="$TEST_DIR/logs/rust-node${node_num}.toml"
    local db_path="$SHARED_DATA_DIR/artemis.db"

    echo "[RUST] Starting node $node_num on port $port (peer: $peer_port)..."

    # 生成排除当前节点的 peers 列表
    local peers=""
    for p in 8081 8082 8083 8084 8085 8086; do
        if [ $p -ne $port ]; then
            peers="$peers\"127.0.0.1:$p\", "
        fi
    done
    peers="${peers%, }"  # 移除最后的逗号和空格

    # 使用 printf 生成配置文件
    printf '[server]\nnode_id = "node-r%d"\nlisten_addr = "0.0.0.0:%d"\npeer_port = %d\n\n[database]\ndb_type = "sqlite"\nurl = "sqlite://%s?mode=rwc"\n\n[cluster]\nenabled = true\npeers = [%s]\n' \
        "$node_num" "$port" "$peer_port" "$db_path" "$peers" > "$config_file"

    RUST_LOG=info "$RUST_BIN" server --config "$config_file" > "$log_file" 2>&1 &

    local pid=$!
    echo $pid > "$TEST_DIR/logs/rust-node${node_num}.pid"
    echo "  PID: $pid, Log: $log_file"
}

start_rust_node 1 8084 9094
start_rust_node 2 8085 9095
start_rust_node 3 8086 9096

echo ""
echo "======================================"
echo "Waiting for nodes to start..."
echo "======================================"
sleep 5

echo ""
echo "[HEALTH CHECK] Checking cluster health..."
# Java 节点: 8081-8083, 使用 /api/status/node.json (统一后)
# Rust 节点: 8084-8086, 使用 /health
ALL_HEALTHY=true
for port in 8081 8082 8083; do
    if curl -s --max-time 2 "http://127.0.0.1:$port/api/status/node.json" > /dev/null 2>&1; then
        echo "  Port $port (Java): HEALTHY"
    else
        echo "  Port $port (Java): UNHEALTHY"
        ALL_HEALTHY=false
    fi
done
for port in 8084 8085 8086; do
    if curl -s --max-time 2 "http://127.0.0.1:$port/health" > /dev/null 2>&1; then
        echo "  Port $port (Rust): HEALTHY"
    else
        echo "  Port $port (Rust): UNHEALTHY"
        ALL_HEALTHY=false
    fi
done

echo ""
echo "======================================"
if [ "$ALL_HEALTHY" = true ]; then
    echo "Cluster started successfully!"
    echo "======================================"
    echo ""
    echo "Next steps:"
    echo "  ./scripts/start-console.sh    # 启动 Web Console"
    echo "  ./scripts/start-apps.sh       # 启动测试应用"
else
    echo "WARNING: Some nodes are unhealthy!"
    echo "Check logs in: $TEST_DIR/logs/"
    echo "======================================"
fi
