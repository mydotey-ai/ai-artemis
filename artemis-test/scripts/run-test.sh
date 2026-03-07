#!/bin/bash
# Artemis Hybrid Test - Run Integration Test
# 运行集成测试，监控指标和日志

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_DIR="$SCRIPT_DIR/.."

echo "======================================"
echo "Artemis Hybrid Integration Test"
echo "======================================"
echo ""

# 参数
DURATION=${1:-600}  # 默认 10 分钟
REPORT_DIR="$TEST_DIR/reports"
LOGS_DIR="$TEST_DIR/logs"

mkdir -p "$REPORT_DIR"

# 检查集群健康
echo "[CHECK] Checking cluster health..."
CLUSTER_NODES=0
for port in 8081 8082 8083 8084 8085 8086; do
    if curl -s --max-time 2 "http://127.0.0.1:$port/health" > /dev/null 2>&1; then
        echo "  ✓ Node port $port: HEALTHY"
        CLUSTER_NODES=$((CLUSTER_NODES + 1))
    else
        echo "  ✗ Node port $port: UNHEALTHY"
    fi
done

echo ""
echo "Cluster status: $CLUSTER_NODES/6 nodes healthy"

if [ $CLUSTER_NODES -eq 0 ]; then
    echo ""
    echo "ERROR: Cluster is not running!"
    echo "Please run: ./scripts/start-cluster.sh"
    exit 1
fi

# 检查 Providers
echo ""
echo "[CHECK] Checking service providers..."
PROVIDER_COUNT=0
for port in 8087 8088 8089 8090; do
    response=$(curl -s --max-time 2 "http://127.0.0.1:$port/sayHello" 2>/dev/null || echo "ERROR")
    if [ "$response" != "ERROR" ] && [ -n "$response" ]; then
        echo "  ✓ Provider port $port: $response"
        PROVIDER_COUNT=$((PROVIDER_COUNT + 1))
    else
        echo "  ✗ Provider port $port: NOT RESPONDING"
    fi
done

echo ""
echo "Providers: $PROVIDER_COUNT/4 available"

# 开始测试
echo ""
echo "======================================"
echo "Starting Load Test"
echo "======================================"
echo "Duration: $DURATION seconds"
echo "Start time: $(date)"
echo ""

# 创建统计文件
STATS_FILE="$REPORT_DIR/test-stats.csv"
echo "timestamp,node,status,latency_ms" > "$STATS_FILE"

START_TIME=$(date +%s)
END_TIME=$((START_TIME + DURATION))
ITERATION=0

# 主测试循环
while [ $(date +%s) -lt $END_TIME ]; do
    ITERATION=$((ITERATION + 1))
    CURRENT_TIME=$(date +%s)
    ELAPSED=$((CURRENT_TIME - START_TIME))
    REMAINING=$((DURATION - ELAPSED))

    # 每 10 秒显示一次进度
    if [ $((ITERATION % 10)) -eq 0 ]; then
        echo "[PROGRESS] Elapsed: ${ELAPSED}s / ${DURATION}s | Remaining: ${REMAINING}s | Iterations: $ITERATION"
    fi

    # 检查集群节点
    for port in 8081 8082 8083 8084 8085 8086; do
        START_CHECK=$(date +%s%N)
        if curl -s --max-time 2 "http://127.0.0.1:$port/health" > /dev/null 2>&1; then
            STATUS="UP"
        else
            STATUS="DOWN"
        fi
        END_CHECK=$(date +%s%N)
        LATENCY=$(( (END_CHECK - START_CHECK) / 1000000 ))  # ns to ms
        echo "$(date +%Y-%m-%d\ %H:%M:%S),node_$port,$STATUS,$LATENCY" >> "$STATS_FILE"
    done

    # 检查 providers
    for port in 8087 8088 8089 8090; do
        START_CHECK=$(date +%s%N)
        if curl -s --max-time 2 "http://127.0.0.1:$port/sayHello" > /dev/null 2>&1; then
            STATUS="UP"
        else
            STATUS="DOWN"
        fi
        END_CHECK=$(date +%s%N)
        LATENCY=$(( (END_CHECK - START_CHECK) / 1000000 ))
        echo "$(date +%Y-%m-%d\ %H:%M:%S),provider_$port,$STATUS,$LATENCY" >> "$STATS_FILE"
    done

    sleep 1
done

# 测试完成
echo ""
echo "======================================"
echo "Load Test Completed"
echo "======================================"
echo "End time: $(date)"
echo "Total iterations: $ITERATION"
echo ""

# 生成报告
echo "[REPORT] Generating test report..."
REPORT_FILE="$REPORT_DIR/test-report-$(date +%Y%m%d-%H%M%S).txt"

{
    echo "======================================"
    echo "Artemis Hybrid Test Report"
    echo "======================================"
    echo "Generated: $(date)"
    echo "Duration: $DURATION seconds"
    echo ""
    echo "=== Cluster Status ==="
    for port in 8081 8082 8083 8084 8085 8086; do
        if curl -s --max-time 2 "http://127.0.0.1:$port/health" > /dev/null 2>&1; then
            echo "Node $port: HEALTHY"
        else
            echo "Node $port: UNHEALTHY"
        fi
    done
    echo ""
    echo "=== Provider Status ==="
    for port in 8087 8088 8089 8090; do
        response=$(curl -s --max-time 2 "http://127.0.0.1:$port/sayHello" 2>/dev/null || echo "UNAVAILABLE")
        echo "Provider $port: $response"
    done
    echo ""
    echo "=== Statistics Summary ==="
    if [ -f "$STATS_FILE" ]; then
        echo "Total records: $(wc -l < "$STATS_FILE")"
        echo ""
        echo "Node availability:"
        grep "node_" "$STATS_FILE" | cut -d',' -f3 | sort | uniq -c | sort -rn
        echo ""
        echo "Provider availability:"
        grep "provider_" "$STATS_FILE" | cut -d',' -f3 | sort | uniq -c | sort -rn
    fi
    echo ""
    echo "=== Log Files ==="
    ls -la "$TEST_DIR/logs/"
} > "$REPORT_FILE"

echo ""
echo "Report saved to: $REPORT_FILE"
echo "Statistics saved to: $STATS_FILE"
echo ""
echo "To view the report:"
echo "  cat $REPORT_FILE"
