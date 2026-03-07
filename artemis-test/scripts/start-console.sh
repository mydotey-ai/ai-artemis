#!/bin/bash
# Artemis Hybrid Test - Start Web Console
# 启动 Web Console 管理集群

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_DIR="$SCRIPT_DIR/.."
PROJECT_ROOT="$TEST_DIR/.."

echo "======================================"
echo "Starting Artemis Web Console"
echo "======================================"
echo ""

# 检查 console 目录
CONSOLE_DIR="$PROJECT_ROOT/artemis-console"
if [ ! -d "$CONSOLE_DIR" ]; then
    echo "ERROR: Web Console directory not found: $CONSOLE_DIR"
    exit 1
fi

# 检查是否已安装依赖
if [ ! -d "$CONSOLE_DIR/node_modules" ]; then
    echo "[INFO] Installing dependencies..."
    cd "$CONSOLE_DIR"
    npm install
fi

# 检查是否有 console 已在运行
CONSOLE_PID=$(lsof -t -i:5173 2>/dev/null || true)
if [ -n "$CONSOLE_PID" ]; then
    echo "WARNING: Web Console already running (PID: $CONSOLE_PID)"
    echo "  URL: http://localhost:5173"
    exit 0
fi

# 启动 console
echo "[START] Starting Web Console..."
cd "$CONSOLE_DIR"

# 使用开发模式启动
npm run dev > "$TEST_DIR/logs/console.log" 2>&1 &
echo $! > "$TEST_DIR/logs/console.pid"

sleep 3

# 检查是否启动成功
if curl -s --max-time 2 "http://localhost:5173" > /dev/null 2>&1; then
    echo ""
    echo "======================================"
    echo "Web Console started successfully!"
    echo "======================================"
    echo ""
    echo "URL: http://localhost:5173"
    echo ""
    echo "Default login:"
    echo "  Username: admin"
    echo "  Password: admin123"
    echo ""
    echo "API endpoints:"
    echo "  Java Node 1: http://localhost:8081"
    echo "  Java Node 2: http://localhost:8082"
    echo "  Java Node 3: http://localhost:8083"
    echo "  Rust Node 1: http://localhost:8084"
    echo "  Rust Node 2: http://localhost:8085"
    echo "  Rust Node 3: http://localhost:8086"
else
    echo ""
    echo "ERROR: Failed to start Web Console!"
    echo "Check log: $TEST_DIR/logs/console.log"
    exit 1
fi
