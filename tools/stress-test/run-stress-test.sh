#!/bin/bash
#
# Artemis 压力测试运行脚本
#
# 用法:
#   ./run-stress-test.sh [profile]
#
# Profiles:
#   quick   - 快速测试 (1000 QPS, 10秒)
#   normal  - 常规测试 (10000 QPS, 60秒) [默认]
#   heavy   - 重度测试 (20000 QPS, 300秒)
#   extreme - 极限测试 (50000 QPS, 600秒)

set -e

PROFILE="${1:-normal}"
SERVER_URL="${ARTEMIS_URL:-http://localhost:8080}"

echo "=== Artemis 压力测试 ==="
echo "Profile: $PROFILE"
echo "Server: $SERVER_URL"
echo

# 检查服务器是否运行
if ! curl -s "$SERVER_URL/health" > /dev/null; then
    echo "错误: 无法连接到 Artemis 服务器 ($SERVER_URL)"
    echo "请先启动服务器: ./target/release/artemis server"
    exit 1
fi

# 编译压力测试工具
echo "编译压力测试工具..."
cd "$(dirname "$0")"
cargo build --release --quiet

# 根据 profile 设置参数
case "$PROFILE" in
    quick)
        CONCURRENCY=10
        QPS=1000
        DURATION=10
        MODE="mixed"
        ;;
    normal)
        CONCURRENCY=100
        QPS=10000
        DURATION=60
        MODE="mixed"
        ;;
    heavy)
        CONCURRENCY=200
        QPS=20000
        DURATION=300
        MODE="mixed"
        ;;
    extreme)
        CONCURRENCY=500
        QPS=50000
        DURATION=600
        MODE="heartbeat"  # 极限测试只测试心跳
        ;;
    *)
        echo "未知的 profile: $PROFILE"
        echo "可用 profiles: quick, normal, heavy, extreme"
        exit 1
        ;;
esac

echo "配置:"
echo "  并发数: $CONCURRENCY"
echo "  目标 QPS: $QPS"
echo "  持续时间: $DURATION 秒"
echo "  测试模式: $MODE"
echo

# 运行压力测试
./target/release/artemis-stress-test \
    --url "$SERVER_URL" \
    --concurrency "$CONCURRENCY" \
    --qps "$QPS" \
    --duration "$DURATION" \
    --mode "$MODE"

echo
echo "压力测试完成!"
