#!/bin/bash
# Instance Management Integration Test
# Tests instance pull-in/pull-out functionality

set -e

BASE_URL="http://localhost:8080"
INSTANCE_KEY='{
  "service_id": "test-service",
  "instance_id": "inst-1",
  "region_id": "us-east",
  "zone_id": "zone-1",
  "group_id": "default"
}'

echo "========================================="
echo "Instance Management Integration Test"
echo "========================================="
echo ""

# Step 1: Register a test instance
echo "[1] Registering test instance..."
curl -s -X POST "$BASE_URL/api/registry/register.json" \
  -H "Content-Type: application/json" \
  -d '{
    "instances": [{
      "region_id": "us-east",
      "zone_id": "zone-1",
      "service_id": "test-service",
      "instance_id": "inst-1",
      "ip": "192.168.1.100",
      "port": 8080,
      "url": "http://192.168.1.100:8080",
      "status": "up"
    }]
  }' | jq '.'
echo ""

# Step 2: Discover service (should return 1 instance)
echo "[2] Discovering service (before pull-out)..."
INSTANCES_BEFORE=$(curl -s -X POST "$BASE_URL/api/discovery/service.json" \
  -H "Content-Type: application/json" \
  -d '{
    "discovery_config": {
      "service_id": "test-service",
      "region_id": "us-east",
      "zone_id": "zone-1"
    }
  }' | jq '.service.instances | length')
echo "Instances found: $INSTANCES_BEFORE"
echo ""

# Step 3: Pull-out instance
echo "[3] Pulling out instance (complete=true)..."
curl -s -X POST "$BASE_URL/api/management/instance/operate-instance.json" \
  -H "Content-Type: application/json" \
  -d "{
    \"instance_key\": $INSTANCE_KEY,
    \"operation\": \"pullout\",
    \"operation_complete\": true,
    \"operator_id\": \"test-admin\"
  }" | jq '.'
echo ""

# Step 4: Check if instance is down
echo "[4] Checking if instance is down..."
IS_DOWN=$(curl -s -X POST "$BASE_URL/api/management/instance/is-instance-down.json" \
  -H "Content-Type: application/json" \
  -d "{
    \"instance_key\": $INSTANCE_KEY
  }" | jq '.is_down')
echo "Instance is down: $IS_DOWN"
if [ "$IS_DOWN" = "true" ]; then
  echo "✅ PASS: Instance correctly marked as down"
else
  echo "❌ FAIL: Instance should be down"
  exit 1
fi
echo ""

# Step 5: Discover service (should return 0 instances due to filter)
echo "[5] Discovering service (after pull-out)..."
INSTANCES_AFTER=$(curl -s -X POST "$BASE_URL/api/discovery/service.json" \
  -H "Content-Type: application/json" \
  -d '{
    "discovery_config": {
      "service_id": "test-service",
      "region_id": "us-east",
      "zone_id": "zone-1"
    }
  }' | jq '.service.instances | length')
echo "Instances found: $INSTANCES_AFTER"
if [ "$INSTANCES_AFTER" = "0" ]; then
  echo "✅ PASS: Pulled-out instance filtered from discovery"
else
  echo "❌ FAIL: Instance should be filtered (expected 0, got $INSTANCES_AFTER)"
  exit 1
fi
echo ""

# Step 6: Pull-in instance
echo "[6] Pulling in instance (complete=true)..."
curl -s -X POST "$BASE_URL/api/management/instance/operate-instance.json" \
  -H "Content-Type: application/json" \
  -d "{
    \"instance_key\": $INSTANCE_KEY,
    \"operation\": \"pullin\",
    \"operation_complete\": true,
    \"operator_id\": \"test-admin\"
  }" | jq '.'
echo ""

# Step 7: Check if instance is up again
echo "[7] Checking if instance is up again..."
IS_DOWN_AFTER=$(curl -s -X POST "$BASE_URL/api/management/instance/is-instance-down.json" \
  -H "Content-Type: application/json" \
  -d "{
    \"instance_key\": $INSTANCE_KEY
  }" | jq '.is_down')
echo "Instance is down: $IS_DOWN_AFTER"
if [ "$IS_DOWN_AFTER" = "false" ]; then
  echo "✅ PASS: Instance correctly marked as up"
else
  echo "❌ FAIL: Instance should be up after pull-in"
  exit 1
fi
echo ""

# Step 8: Discover service again (should return 1 instance)
echo "[8] Discovering service (after pull-in)..."
INSTANCES_FINAL=$(curl -s -X POST "$BASE_URL/api/discovery/service.json" \
  -H "Content-Type: application/json" \
  -d '{
    "discovery_config": {
      "service_id": "test-service",
      "region_id": "us-east",
      "zone_id": "zone-1"
    }
  }' | jq '.service.instances | length')
echo "Instances found: $INSTANCES_FINAL"
if [ "$INSTANCES_FINAL" = "1" ]; then
  echo "✅ PASS: Instance visible in discovery after pull-in"
else
  echo "❌ FAIL: Instance should be visible (expected 1, got $INSTANCES_FINAL)"
  exit 1
fi
echo ""

# Step 9: Test server-level pull-out
echo "[9] Testing server-level pull-out..."
curl -s -X POST "$BASE_URL/api/management/server/operate-server.json" \
  -H "Content-Type: application/json" \
  -d '{
    "server_id": "192.168.1.100",
    "region_id": "us-east",
    "operation": "pullout",
    "operation_complete": true,
    "operator_id": "test-admin"
  }' | jq '.'
echo ""

# Step 10: Verify server is down
echo "[10] Verifying server is down..."
IS_SERVER_DOWN=$(curl -s -X POST "$BASE_URL/api/management/server/is-server-down.json" \
  -H "Content-Type: application/json" \
  -d '{
    "server_id": "192.168.1.100",
    "region_id": "us-east"
  }' | jq '.is_down')
echo "Server is down: $IS_SERVER_DOWN"
if [ "$IS_SERVER_DOWN" = "true" ]; then
  echo "✅ PASS: Server correctly marked as down"
else
  echo "❌ FAIL: Server should be down"
  exit 1
fi
echo ""

# Step 11: Verify instances on server are filtered
echo "[11] Discovering service (server pulled out)..."
INSTANCES_SERVER_DOWN=$(curl -s -X POST "$BASE_URL/api/discovery/service.json" \
  -H "Content-Type: application/json" \
  -d '{
    "discovery_config": {
      "service_id": "test-service",
      "region_id": "us-east",
      "zone_id": "zone-1"
    }
  }' | jq '.service.instances | length')
echo "Instances found: $INSTANCES_SERVER_DOWN"
if [ "$INSTANCES_SERVER_DOWN" = "0" ]; then
  echo "✅ PASS: Instances on pulled-out server filtered from discovery"
else
  echo "❌ FAIL: Instances should be filtered when server is down"
  exit 1
fi
echo ""

# Step 12: Pull-in server
echo "[12] Pulling in server..."
curl -s -X POST "$BASE_URL/api/management/server/operate-server.json" \
  -H "Content-Type: application/json" \
  -d '{
    "server_id": "192.168.1.100",
    "region_id": "us-east",
    "operation": "pullin",
    "operation_complete": true,
    "operator_id": "test-admin"
  }' | jq '.'
echo ""

# Step 13: Final verification
echo "[13] Final discovery (server pulled in)..."
INSTANCES_END=$(curl -s -X POST "$BASE_URL/api/discovery/service.json" \
  -H "Content-Type: application/json" \
  -d '{
    "discovery_config": {
      "service_id": "test-service",
      "region_id": "us-east",
      "zone_id": "zone-1"
    }
  }' | jq '.service.instances | length')
echo "Instances found: $INSTANCES_END"
if [ "$INSTANCES_END" = "1" ]; then
  echo "✅ PASS: Instances visible after server pull-in"
else
  echo "❌ FAIL: Instance should be visible after server pull-in"
  exit 1
fi
echo ""

echo "========================================="
echo "✅ ALL TESTS PASSED!"
echo "========================================="
