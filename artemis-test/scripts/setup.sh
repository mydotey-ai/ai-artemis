#!/bin/bash
# Artemis Hybrid Test - Setup Script
# 环境准备：编译所有依赖

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR/../.."
TEST_DIR="$SCRIPT_DIR/.."

echo "======================================"
echo "Artemis Hybrid Test - Setup"
echo "======================================"
echo ""

# 1. 检查必要工具
echo "[1/7] Checking prerequisites..."
command -v java >/dev/null 2>&1 || { echo "ERROR: Java not found"; exit 1; }
command -v mvn >/dev/null 2>&1 || { echo "ERROR: Maven not found"; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo "ERROR: Rust/Cargo not found"; exit 1; }
echo "  ✓ Java: $(java -version 2>&1 | head -1)"
echo "  ✓ Maven: $(mvn -version 2>&1 | head -1)"
echo "  ✓ Cargo: $(cargo --version)"

# 2. 创建目录结构
echo ""
echo "[2/7] Creating directory structure..."
mkdir -p "$TEST_DIR"/{data,logs,reports}
mkdir -p "$TEST_DIR/apps"/{java-provider,java-consumer,rust-provider,rust-consumer}
touch "$TEST_DIR/data/.gitkeep"
echo "  ✓ Directories created"

# 3. 编译 Rust Artemis 服务端
echo ""
echo "[3/7] Building Rust Artemis Server..."
cd "$PROJECT_ROOT"
if [ ! -f target/release/artemis ]; then
    echo "  Building release binary..."
    cargo build --release --bin artemis 2>&1 | tail -5
else
    echo "  ✓ Binary already exists: target/release/artemis"
fi

# 4. 编译 Java Artemis 服务端
echo ""
echo "[4/7] Building Java Artemis Server..."
cd "$PROJECT_ROOT/artemis-java"
if [ ! -f artemis-server/target/artemis-server.jar ]; then
    echo "  Building with Maven..."
    mvn package -DskipTests -q 2>&1 | tail -10
else
    echo "  ✓ JAR already exists: artemis-server/target/artemis-server.jar"
fi

# 5. 生成配置文件
echo ""
echo "[5/7] Generating configuration files..."
cd "$TEST_DIR"

# 创建 Rust 节点配置
for i in 1 2 3; do
    port=$((8083 + i))
    peer_port=$((9093 + i))

    # 生成排除当前节点的 peers 列表
    peers=""
    for p in 8081 8082 8083 8084 8085 8086; do
        if [ $p -ne $port ]; then
            peers="$peers\"127.0.0.1:$p\", "
        fi
    done
    peers="${peers%, }"

    cat > "config/rust-node${i}.toml" <<EOF
[server]
node_id = "node-r${i}"
listen_addr = "0.0.0.0:${port}"
peer_port = ${peer_port}

[database]
db_type = "sqlite"
url = "sqlite://$(pwd)/data/artemis.db"

[cluster]
enabled = true
peers = [${peers}]
EOF
done

echo "  ✓ Configuration files generated"

# 6. 初始化共享数据库
echo ""
echo "[6/7] Initializing shared SQLite database..."
DB_FILE="$TEST_DIR/data/artemis.db"
if [ ! -f "$DB_FILE" ]; then
    echo "  Creating database..."
    # 使用 Rust 节点首次启动时自动创建表
    # 这里仅创建一个空文件，表结构由应用自动创建
    touch "$DB_FILE"
    echo "  ✓ Database initialized (tables will be created on first start)"
else
    echo "  ✓ Database already exists"
fi

# 7. 创建启动脚本（如果不存在）
echo ""
echo "[7/7] Checking start scripts..."
for script in start-cluster.sh start-apps.sh start-console.sh run-test.sh cleanup.sh; do
    if [ ! -f "$TEST_DIR/scripts/$script" ]; then
        echo "  ⚠ $script not found (will be created in next step)"
    else
        echo "  ✓ $script exists"
    fi
done

echo ""
echo "======================================"
echo "Setup Complete!"
echo "======================================"
echo ""
echo "Next steps:"
echo "  cd $TEST_DIR"
echo "  ./scripts/start-cluster.sh    # 启动 6 节点集群"
echo "  ./scripts/start-apps.sh       # 启动测试应用"
echo "  ./scripts/start-console.sh    # 启动 Web Console"
echo "  ./scripts/run-test.sh 600     # 运行 10 分钟测试"
echo ""
