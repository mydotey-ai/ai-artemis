#!/bin/bash
# Artemis Hybrid Test - Start Test Applications
# 启动 4 个 Provider + 4 个 Consumer 测试应用

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_DIR="$SCRIPT_DIR/.."
PROJECT_ROOT="$TEST_DIR/.."

echo "======================================"
echo "Starting Test Applications"
echo "======================================"
echo ""
echo "This will start:"
echo "  - 2 Java Providers (ports 8087, 8088)"
echo "  - 2 Rust Providers (ports 8089, 8090)"
echo "  - 2 Java Consumers (background jobs)"
echo "  - 2 Rust Consumers (background jobs)"
echo ""

# 检查集群是否健康
echo "[CHECK] Checking cluster health..."
CLUSTER_HEALTHY=true
for port in 8081 8082 8083 8084 8085 8086; do
    if ! curl -s --max-time 2 "http://127.0.0.1:$port/health" > /dev/null 2>&1; then
        echo "  ✗ Node port $port is not healthy"
        CLUSTER_HEALTHY=false
    fi
done

if [ "$CLUSTER_HEALTHY" = false ]; then
    echo ""
    echo "WARNING: Cluster is not fully healthy!"
    echo "You can still start apps, but they may fail to register."
    read -p "Continue anyway? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# ==================== Java Providers ====================
echo ""
echo "======================================"
echo "Starting Java Providers"
echo "======================================"

# 检查 Maven 项目是否存在
JAVA_PROVIDER_DIR="$TEST_DIR/apps/java-provider"
if [ ! -f "$JAVA_PROVIDER_DIR/pom.xml" ]; then
    echo "WARNING: Java Provider project not found at $JAVA_PROVIDER_DIR"
    echo "Skipping Java Providers (you need to implement them first)"
else
    # 编译 Java Provider
    echo "[BUILD] Compiling Java Providers..."
    cd "$JAVA_PROVIDER_DIR"
    mvn package -DskipTests -q

    # 启动 Java Provider 1 (port 8087)
    echo "[START] Java Provider 1 (port 8087)..."
    java -jar target/java-provider.jar \
        --server.port=8087 \
        --artemis.servers=http://localhost:8081,http://localhost:8084 \
        > "$TEST_DIR/logs/java-provider1.log" 2>&1 &
    echo $! > "$TEST_DIR/logs/java-provider1.pid"
    echo "  PID: $(cat "$TEST_DIR/logs/java-provider1.pid")"

    # 启动 Java Provider 2 (port 8088)
    echo "[START] Java Provider 2 (port 8088)..."
    java -jar target/java-provider.jar \
        --server.port=8088 \
        --artemis.servers=http://localhost:8082,http://localhost:8085 \
        > "$TEST_DIR/logs/java-provider2.log" 2>&1 &
    echo $! > "$TEST_DIR/logs/java-provider2.pid"
    echo "  PID: $(cat "$TEST_DIR/logs/java-provider2.pid")"
fi

# ==================== Rust Providers ====================
echo ""
echo "======================================"
echo "Starting Rust Providers"
echo "======================================"

# 检查 Rust 项目是否存在
RUST_PROVIDER_DIR="$TEST_DIR/apps/rust-provider"
if [ ! -f "$RUST_PROVIDER_DIR/Cargo.toml" ]; then
    echo "WARNING: Rust Provider project not found at $RUST_PROVIDER_DIR"
    echo "Skipping Rust Providers (you need to implement them first)"
else
    # 编译 Rust Provider
    echo "[BUILD] Compiling Rust Providers..."
    cd "$RUST_PROVIDER_DIR"
    cargo build --release 2>&1 | tail -5

    # 启动 Rust Provider 1 (port 8089)
    echo "[START] Rust Provider 1 (port 8089)..."
    PORT=8089 \
    ARTEMIS_SERVERS="http://localhost:8083,http://localhost:8086" \
    ./target/release/rust-provider \
        > "$TEST_DIR/logs/rust-provider1.log" 2>&1 &
    echo $! > "$TEST_DIR/logs/rust-provider1.pid"
    echo "  PID: $(cat "$TEST_DIR/logs/rust-provider1.pid")"

    # 启动 Rust Provider 2 (port 8090)
    echo "[START] Rust Provider 2 (port 8090)..."
    PORT=8090 \
    ARTEMIS_SERVERS="http://localhost:8081,http://localhost:8084" \
    ./target/release/rust-provider \
        > "$TEST_DIR/logs/rust-provider2.log" 2>&1 &
    echo $! > "$TEST_DIR/logs/rust-provider2.pid"
    echo "  PID: $(cat "$TEST_DIR/logs/rust-provider2.pid")"
fi

# ==================== Java Consumers ====================
echo ""
echo "======================================"
echo "Starting Java Consumers"
echo "======================================"

JAVA_CONSUMER_DIR="$TEST_DIR/apps/java-consumer"
if [ ! -f "$JAVA_CONSUMER_DIR/pom.xml" ]; then
    echo "WARNING: Java Consumer project not found"
    echo "Skipping Java Consumers"
else
    echo "[BUILD] Compiling Java Consumers..."
    cd "$JAVA_CONSUMER_DIR"
    mvn package -DskipTests -q

    for i in 1 2; do
        echo "[START] Java Consumer $i..."
        java -jar target/java-consumer.jar \
            --artemis.servers=http://localhost:8081,http://localhost:8082,http://localhost:8083,http://localhost:8084,http://localhost:8085,http://localhost:8086 \
            --consumer.id="java-consumer-$i" \
            > "$TEST_DIR/logs/java-consumer$i.log" 2>&1 &
        echo $! > "$TEST_DIR/logs/java-consumer$i.pid"
        echo "  PID: $(cat "$TEST_DIR/logs/java-consumer$i.pid")"
    done
fi

# ==================== Rust Consumers ====================
echo ""
echo "======================================"
echo "Starting Rust Consumers"
echo "======================================"

RUST_CONSUMER_DIR="$TEST_DIR/apps/rust-consumer"
if [ ! -f "$RUST_CONSUMER_DIR/Cargo.toml" ]; then
    echo "WARNING: Rust Consumer project not found"
    echo "Skipping Rust Consumers"
else
    echo "[BUILD] Compiling Rust Consumers..."
    cd "$RUST_CONSUMER_DIR"
    cargo build --release 2>&1 | tail -5

    for i in 1 2; do
        echo "[START] Rust Consumer $i..."
        CONSUMER_ID="rust-consumer-$i" \
        ./target/release/rust-consumer \
            > "$TEST_DIR/logs/rust-consumer$i.log" 2>&1 &
        echo $! > "$TEST_DIR/logs/rust-consumer$i.pid"
        echo "  PID: $(cat "$TEST_DIR/logs/rust-consumer$i.pid")"
    done
fi

# ==================== Summary ====================
echo ""
echo "======================================"
echo "Test Applications Started!"
echo "======================================"
echo ""

# 等待几秒让应用启动
sleep 3

# 检查 Provider 健康
echo "[HEALTH CHECK] Providers:"
for port in 8087 8088 8089 8090; do
    if curl -s --max-time 2 "http://127.0.0.1:$port/sayHello" > /dev/null 2>&1; then
        echo "  ✓ Port $port: HEALTHY"
    else
        echo "  ✗ Port $port: NOT RESPONDING"
    fi
done

echo ""
echo "Logs available in: $TEST_DIR/logs/"
echo ""
echo "Next steps:"
echo "  ./scripts/start-console.sh  # 启动 Web Console"
echo "  ./scripts/run-test.sh 600   # 运行 10 分钟测试"
echo "  ./scripts/cleanup.sh          # 清理所有进程"
