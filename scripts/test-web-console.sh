#!/bin/bash

# Web Console 测试数据准备脚本

set -e

BASE_URL="http://localhost:8080"

echo "正在准备测试数据..."

# 注册测试服务实例
echo "1. 注册测试服务实例..."

# 服务 1: user-service (2 个实例)
curl -s -X POST "${BASE_URL}/api/registry/register.json" \
  -H "Content-Type: application/json" \
  -d '{
    "instances": [
      {
        "region_id": "local",
        "zone_id": "zone1",
        "service_id": "user-service",
        "instance_id": "user-1",
        "ip": "192.168.1.100",
        "port": 8080,
        "url": "http://192.168.1.100:8080",
        "status": "up",
        "metadata": {"version": "1.0.0", "env": "production"}
      },
      {
        "region_id": "local",
        "zone_id": "zone1",
        "service_id": "user-service",
        "instance_id": "user-2",
        "ip": "192.168.1.101",
        "port": 8080,
        "url": "http://192.168.1.101:8080",
        "status": "up",
        "metadata": {"version": "1.0.0", "env": "production"}
      }
    ]
  }' | jq '.'

echo ""

# 服务 2: order-service (3 个实例)
curl -s -X POST "${BASE_URL}/api/registry/register.json" \
  -H "Content-Type: application/json" \
  -d '{
    "instances": [
      {
        "region_id": "local",
        "zone_id": "zone1",
        "service_id": "order-service",
        "instance_id": "order-1",
        "ip": "192.168.1.200",
        "port": 8081,
        "url": "http://192.168.1.200:8081",
        "status": "up",
        "metadata": {"version": "2.0.0", "env": "production"}
      },
      {
        "region_id": "local",
        "zone_id": "zone1",
        "service_id": "order-service",
        "instance_id": "order-2",
        "ip": "192.168.1.201",
        "port": 8081,
        "url": "http://192.168.1.201:8081",
        "status": "up",
        "metadata": {"version": "2.0.0", "env": "production"}
      },
      {
        "region_id": "local",
        "zone_id": "zone1",
        "service_id": "order-service",
        "instance_id": "order-3",
        "ip": "192.168.1.202",
        "port": 8081,
        "url": "http://192.168.1.202:8081",
        "status": "starting",
        "metadata": {"version": "2.1.0", "env": "staging"}
      }
    ]
  }' | jq '.'

echo ""

# 服务 3: payment-service (1 个实例)
curl -s -X POST "${BASE_URL}/api/registry/register.json" \
  -H "Content-Type: application/json" \
  -d '{
    "instances": [
      {
        "region_id": "local",
        "zone_id": "zone1",
        "service_id": "payment-service",
        "instance_id": "payment-1",
        "ip": "192.168.1.300",
        "port": 8082,
        "url": "http://192.168.1.300:8082",
        "status": "up",
        "metadata": {"version": "1.5.0", "env": "production"}
      }
    ]
  }' | jq '.'

echo ""
echo "✅ 测试数据准备完成!"
echo ""

# 验证数据
echo "2. 验证服务列表..."
curl -s -X POST "${BASE_URL}/api/discovery/services.json" \
  -H "Content-Type: application/json" \
  -d '{"region_id":"local","zone_id":"zone1"}' | jq '.'

echo ""
echo "✅ 测试数据验证完成!"
echo ""
echo "现在可以访问 Web Console: http://localhost:3002/"
