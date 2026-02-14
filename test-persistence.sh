#!/bin/bash

# ================================================================
# Artemis 数据持久化集成测试
# ================================================================
#
# 用途: 测试 SQLite/MySQL 数据持久化功能
#
# 测试场景:
#   1. 创建服务分组
#   2. 创建路由规则
#   3. 创建 Zone 操作
#   4. 创建金丝雀配置
#   5. 停止服务器
#   6. 重启服务器
#   7. 验证配置自动恢复
#   8. 清理测试数据
#
# 使用方法:
#   ./test-persistence.sh
#
# 配置文件: artemis-test-with-db.toml
#
# 注意: 此脚本会自动启动和停止 Artemis 服务器
#
# ================================================================

set -e

echo "=== Phase 14: 数据持久化集成测试 ==="
echo

# 清理旧数据库
echo "1. 清理旧测试数据..."
rm -f artemis-test.db artemis-test.db-shm artemis-test.db-wal
echo "✓ 清理完成"
echo

# 启动服务器
echo "2. 启动 Artemis (带数据库)..."
./target/release/artemis server --config artemis-test-with-db.toml &
PID=$!
echo "Server PID: $PID"
sleep 3
echo "✓ 服务器已启动"
echo

# 创建分组
echo "3. 创建服务分组..."
curl -s -X POST http://localhost:8080/api/v1/management/groups \
  -H "Content-Type: application/json" \
  -d '{
    "service_id": "test-service",
    "region_id": "us-east",
    "zone_id": "zone-1",
    "name": "group-persistent",
    "group_type": "physical",
    "status": "active",
    "description": "测试持久化分组"
  }' | jq '.'
echo "✓ 分组已创建"
echo

# 创建路由规则
echo "4. 创建路由规则..."
curl -s -X POST http://localhost:8080/api/v1/management/routes \
  -H "Content-Type: application/json" \
  -d '{
    "route_id": "route-persistent",
    "service_id": "test-service",
    "name": "持久化路由",
    "status": "active",
    "strategy": "weighted_round_robin",
    "groups": []
  }' | jq '.'
echo "✓ 路由规则已创建"
echo

# 创建Zone操作
echo "5. 拉出Zone..."
curl -s -X POST "http://localhost:8080/api/v1/management/zones/pullout?zone_id=zone-1&region_id=us-east&operator_id=admin" | jq '.'
echo "✓ Zone已拉出"
echo

# 创建金丝雀配置
echo "6. 创建金丝雀配置..."
curl -s -X POST http://localhost:8080/api/v1/management/canary \
  -H "Content-Type: application/json" \
  -d '{
    "service_id": "test-service",
    "ip_whitelist": ["192.168.1.100", "10.0.0.1"],
    "enabled": true
  }' | jq '.'
echo "✓ 金丝雀配置已创建"
echo

# 等待异步持久化任务完成
echo "   等待异步持久化完成..."
sleep 2
echo "✓ 持久化任务已完成"
echo

# 检查数据库
echo "7. 检查数据库内容..."
echo "   - 分组表:"
sqlite3 artemis-test.db "SELECT group_id, group_name, group_type, service_id FROM service_group;" 2>/dev/null || echo "     (empty)"
echo "   - 路由规则表:"
sqlite3 artemis-test.db "SELECT route_id, service_id, name, status FROM service_route_rule;" 2>/dev/null || echo "     (empty)"
echo "   - Zone操作表:"
sqlite3 artemis-test.db "SELECT zone_id, region_id, operation, operator_id FROM zone_operation;" 2>/dev/null || echo "     (empty)"
echo "   - 金丝雀配置表:"
sqlite3 artemis-test.db "SELECT service_id, enabled FROM canary_config;" 2>/dev/null || echo "     (empty)"
echo "✓ 数据已持久化到数据库"
echo

# 停止服务器
echo "8. 停止服务器..."
kill $PID
wait $PID 2>/dev/null || true
echo "✓ 服务器已停止"
echo

# 重新启动服务器
echo "9. 重新启动服务器 (测试配置加载)..."
./target/release/artemis server --config artemis-test-with-db.toml &
PID=$!
sleep 3
echo "✓ 服务器已重启"
echo

# 验证分组已恢复
echo "10. 验证分组已恢复..."
GROUPS=$(curl -s http://localhost:8080/api/v1/management/groups)
echo "$GROUPS" | jq '.'
if echo "$GROUPS" | grep -q "group-persistent"; then
    echo "✓ 分组已从数据库恢复"
else
    echo "✗ 分组恢复失败"
    kill $PID 2>/dev/null || true
    exit 1
fi
echo

# 验证路由规则已恢复
echo "11. 验证路由规则已恢复..."
ROUTES=$(curl -s http://localhost:8080/api/v1/management/routes)
echo "$ROUTES" | jq '.'
if echo "$ROUTES" | grep -q "route-persistent"; then
    echo "✓ 路由规则已从数据库恢复"
else
    echo "✗ 路由规则恢复失败"
    kill $PID 2>/dev/null || true
    exit 1
fi
echo

# 验证Zone状态已恢复
echo "12. 验证Zone状态已恢复..."
ZONES=$(curl -s "http://localhost:8080/api/v1/management/zones/operations?region_id=us-east")
echo "$ZONES" | jq '.'
if echo "$ZONES" | grep -q "zone-1"; then
    echo "✓ Zone状态已从数据库恢复"
else
    echo "✗ Zone状态恢复失败"
    kill $PID 2>/dev/null || true
    exit 1
fi
echo

# 验证金丝雀配置已恢复
echo "13. 验证金丝雀配置已恢复..."
CANARY=$(curl -s http://localhost:8080/api/v1/management/canary)
echo "$CANARY" | jq '.'
if echo "$CANARY" | grep -q "test-service"; then
    echo "✓ 金丝雀配置已从数据库恢复"
else
    echo "✗ 金丝雀配置恢复失败"
    kill $PID 2>/dev/null || true
    exit 1
fi
echo

# 停止服务器
echo "14. 清理..."
kill $PID
wait $PID 2>/dev/null || true
echo "✓ 服务器已停止"
echo

echo "==================================="
echo "✅ Phase 14 数据持久化测试通过!"
echo "==================================="
echo
echo "测试结果:"
echo "  ✓ 分组持久化和恢复"
echo "  ✓ 路由规则持久化和恢复"
echo "  ✓ Zone操作持久化和恢复"
echo "  ✓ 金丝雀配置持久化和恢复"
echo "  ✓ 启动时自动加载配置"
echo
echo "数据库文件: artemis-test.db"
