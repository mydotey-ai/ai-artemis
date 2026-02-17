#!/bin/bash

# Artemis 认证系统集成测试脚本
# 测试所有 18 个认证 API 端点

set -e

API_BASE="http://127.0.0.1:8080"
ADMIN_USER="admin"
ADMIN_PASS="admin123"
TOKEN=""
TEST_USER_ID=""
SESSION_ID=""

# 颜色输出
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 打印分隔线
print_separator() {
    echo "=============================================="
}

# 打印测试标题
print_test() {
    echo -e "\n${YELLOW}=== $1 ===${NC}"
}

# 打印成功
print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

# 打印失败
print_error() {
    echo -e "${RED}✗ $1${NC}"
    exit 1
}

# 检查服务器是否运行
check_server() {
    print_test "检查服务器状态"
    if curl -s -f "$API_BASE/health" > /dev/null; then
        print_success "服务器运行中"
    else
        print_error "服务器未运行，请先启动: cargo run --bin artemis -- server --addr 127.0.0.1:8080"
    fi
}

# Test 1: 登录
test_login() {
    print_test "Test 1: 用户登录 (POST /api/auth/login)"

    RESPONSE=$(curl -s -X POST "$API_BASE/api/auth/login" \
        -H "Content-Type: application/json" \
        -d "{\"username\":\"$ADMIN_USER\",\"password\":\"$ADMIN_PASS\"}")

    echo "$RESPONSE" | jq .

    # 提取 token
    TOKEN=$(echo "$RESPONSE" | jq -r '.data.access_token')

    if [ "$TOKEN" != "null" ] && [ -n "$TOKEN" ]; then
        print_success "登录成功，获取到 token"
    else
        print_error "登录失败"
    fi
}

# Test 2: 登录失败测试
test_login_fail() {
    print_test "Test 2: 错误密码登录 (应该失败)"

    RESPONSE=$(curl -s -X POST "$API_BASE/api/auth/login" \
        -H "Content-Type: application/json" \
        -d '{"username":"admin","password":"wrong_password"}')

    echo "$RESPONSE" | jq .

    SUCCESS=$(echo "$RESPONSE" | jq -r '.success')

    if [ "$SUCCESS" == "false" ]; then
        print_success "正确拒绝了错误密码"
    else
        print_error "应该拒绝错误密码"
    fi
}

# Test 3: 获取当前用户信息
test_get_current_user() {
    print_test "Test 3: 获取当前用户信息 (GET /api/auth/user)"

    RESPONSE=$(curl -s -X GET "$API_BASE/api/auth/user" \
        -H "Authorization: Bearer $TOKEN")

    echo "$RESPONSE" | jq .

    USERNAME=$(echo "$RESPONSE" | jq -r '.data.username')

    if [ "$USERNAME" == "$ADMIN_USER" ]; then
        print_success "获取用户信息成功"
    else
        print_error "获取用户信息失败"
    fi
}

# Test 4: 获取用户权限
test_get_permissions() {
    print_test "Test 4: 获取用户权限 (GET /api/auth/permissions)"

    RESPONSE=$(curl -s -X GET "$API_BASE/api/auth/permissions" \
        -H "Authorization: Bearer $TOKEN")

    echo "$RESPONSE" | jq .

    PERMS=$(echo "$RESPONSE" | jq -r '.data[0]')

    if [ "$PERMS" == "*:*" ]; then
        print_success "Admin 拥有全部权限"
    else
        print_error "权限获取失败"
    fi
}

# Test 5: 列出所有角色
test_list_roles() {
    print_test "Test 5: 列出所有角色 (GET /api/auth/roles)"

    RESPONSE=$(curl -s -X GET "$API_BASE/api/auth/roles")

    echo "$RESPONSE" | jq .

    ROLES_COUNT=$(echo "$RESPONSE" | jq -r '.data | length')

    if [ "$ROLES_COUNT" == "3" ]; then
        print_success "获取到 3 个角色"
    else
        print_error "角色列表不正确"
    fi
}

# Test 6: 检查权限
test_check_permission() {
    print_test "Test 6: 检查权限 (POST /api/auth/check-permission)"

    RESPONSE=$(curl -s -X POST "$API_BASE/api/auth/check-permission" \
        -H "Authorization: Bearer $TOKEN" \
        -H "Content-Type: application/json" \
        -d '{"resource":"users","action":"write"}')

    echo "$RESPONSE" | jq .

    ALLOWED=$(echo "$RESPONSE" | jq -r '.data.allowed')

    if [ "$ALLOWED" == "true" ]; then
        print_success "Admin 有 users:write 权限"
    else
        print_error "权限检查失败"
    fi
}

# Test 7: 列出所有用户
test_list_users() {
    print_test "Test 7: 列出所有用户 (GET /api/auth/users)"

    RESPONSE=$(curl -s -X GET "$API_BASE/api/auth/users" \
        -H "Authorization: Bearer $TOKEN")

    echo "$RESPONSE" | jq .

    USERS_COUNT=$(echo "$RESPONSE" | jq -r '.data | length')

    if [ "$USERS_COUNT" -ge "1" ]; then
        print_success "获取到 $USERS_COUNT 个用户"
    else
        print_error "用户列表为空"
    fi
}

# Test 8: 创建新用户
test_create_user() {
    print_test "Test 8: 创建新用户 (POST /api/auth/users)"

    RESPONSE=$(curl -s -X POST "$API_BASE/api/auth/users" \
        -H "Authorization: Bearer $TOKEN" \
        -H "Content-Type: application/json" \
        -d '{"username":"testuser","email":"test@example.com","password":"test123","role":"operator"}')

    echo "$RESPONSE" | jq .

    TEST_USER_ID=$(echo "$RESPONSE" | jq -r '.data.user_id')

    if [ "$TEST_USER_ID" != "null" ] && [ -n "$TEST_USER_ID" ]; then
        print_success "创建用户成功，user_id: $TEST_USER_ID"
    else
        print_error "创建用户失败"
    fi
}

# Test 9: 获取用户详情
test_get_user() {
    print_test "Test 9: 获取用户详情 (GET /api/auth/users/:user_id)"

    RESPONSE=$(curl -s -X GET "$API_BASE/api/auth/users/$TEST_USER_ID" \
        -H "Authorization: Bearer $TOKEN")

    echo "$RESPONSE" | jq .

    USERNAME=$(echo "$RESPONSE" | jq -r '.data.username')

    if [ "$USERNAME" == "testuser" ]; then
        print_success "获取用户详情成功"
    else
        print_error "获取用户详情失败"
    fi
}

# Test 10: 更新用户
test_update_user() {
    print_test "Test 10: 更新用户 (PUT /api/auth/users/:user_id)"

    RESPONSE=$(curl -s -X PUT "$API_BASE/api/auth/users/$TEST_USER_ID" \
        -H "Authorization: Bearer $TOKEN" \
        -H "Content-Type: application/json" \
        -d '{"email":"updated@example.com","role":"viewer"}')

    echo "$RESPONSE" | jq .

    EMAIL=$(echo "$RESPONSE" | jq -r '.data.email')
    ROLE=$(echo "$RESPONSE" | jq -r '.data.role')

    if [ "$EMAIL" == "updated@example.com" ] && [ "$ROLE" == "viewer" ]; then
        print_success "更新用户成功"
    else
        print_error "更新用户失败"
    fi
}

# Test 11: 修改用户状态
test_update_user_status() {
    print_test "Test 11: 修改用户状态 (PATCH /api/auth/users/:user_id/status)"

    RESPONSE=$(curl -s -X PATCH "$API_BASE/api/auth/users/$TEST_USER_ID/status" \
        -H "Authorization: Bearer $TOKEN" \
        -H "Content-Type: application/json" \
        -d '{"status":"inactive"}')

    echo "$RESPONSE" | jq .

    STATUS=$(echo "$RESPONSE" | jq -r '.data.status')

    if [ "$STATUS" == "inactive" ]; then
        print_success "修改用户状态成功"
    else
        print_error "修改用户状态失败"
    fi

    # 恢复为 active
    curl -s -X PATCH "$API_BASE/api/auth/users/$TEST_USER_ID/status" \
        -H "Authorization: Bearer $TOKEN" \
        -H "Content-Type: application/json" \
        -d '{"status":"active"}' > /dev/null
}

# Test 12: 重置密码（管理员操作）
test_reset_password() {
    print_test "Test 12: 重置用户密码 (POST /api/auth/password/reset/:user_id)"

    RESPONSE=$(curl -s -X POST "$API_BASE/api/auth/password/reset/$TEST_USER_ID" \
        -H "Authorization: Bearer $TOKEN" \
        -H "Content-Type: application/json" \
        -d '{"new_password":"newpass123"}')

    echo "$RESPONSE" | jq .

    SUCCESS=$(echo "$RESPONSE" | jq -r '.success')

    if [ "$SUCCESS" == "true" ]; then
        print_success "重置密码成功"
    else
        print_error "重置密码失败"
    fi
}

# Test 13: 列出会话
test_list_sessions() {
    print_test "Test 13: 列出用户会话 (GET /api/auth/sessions)"

    RESPONSE=$(curl -s -X GET "$API_BASE/api/auth/sessions" \
        -H "Authorization: Bearer $TOKEN")

    echo "$RESPONSE" | jq .

    SESSIONS_COUNT=$(echo "$RESPONSE" | jq -r '.data | length')
    SESSION_ID=$(echo "$RESPONSE" | jq -r '.data[0].session_id')

    if [ "$SESSIONS_COUNT" -ge "1" ]; then
        print_success "获取到 $SESSIONS_COUNT 个会话"
    else
        print_error "会话列表为空"
    fi
}

# Test 14: 获取登录历史
test_login_history() {
    print_test "Test 14: 获取登录历史 (GET /api/auth/users/:user_id/login-history)"

    # 先使用 admin 的 user_id
    ADMIN_USER_ID=$(curl -s -X GET "$API_BASE/api/auth/user" \
        -H "Authorization: Bearer $TOKEN" | jq -r '.data.user_id')

    RESPONSE=$(curl -s -X GET "$API_BASE/api/auth/users/$ADMIN_USER_ID/login-history" \
        -H "Authorization: Bearer $TOKEN")

    echo "$RESPONSE" | jq .

    HISTORY_COUNT=$(echo "$RESPONSE" | jq -r '.data | length')

    if [ "$HISTORY_COUNT" -ge "1" ]; then
        print_success "获取到 $HISTORY_COUNT 条登录历史"
    else
        print_error "登录历史为空"
    fi
}

# Test 15: 刷新 Token
test_refresh_token() {
    print_test "Test 15: 刷新 Token (POST /api/auth/refresh)"

    RESPONSE=$(curl -s -X POST "$API_BASE/api/auth/refresh" \
        -H "Content-Type: application/json" \
        -d "{\"token\":\"$TOKEN\"}")

    echo "$RESPONSE" | jq .

    NEW_TOKEN=$(echo "$RESPONSE" | jq -r '.data.access_token')

    if [ "$NEW_TOKEN" != "null" ] && [ -n "$NEW_TOKEN" ] && [ "$NEW_TOKEN" != "$TOKEN" ]; then
        print_success "Token 刷新成功"
        TOKEN="$NEW_TOKEN"  # 更新 token
    else
        print_error "Token 刷新失败"
    fi
}

# Test 16: 无 Token 访问受保护端点（应该失败）
test_unauthorized_access() {
    print_test "Test 16: 无 Token 访问受保护端点 (应该失败)"

    RESPONSE=$(curl -s -X GET "$API_BASE/api/auth/users" \
        -w "\nHTTP_CODE:%{http_code}")

    HTTP_CODE=$(echo "$RESPONSE" | grep "HTTP_CODE" | cut -d: -f2)

    if [ "$HTTP_CODE" == "401" ]; then
        print_success "正确拒绝了未授权访问"
    else
        print_error "应该返回 401 状态码"
    fi
}

# Test 17: 删除用户
test_delete_user() {
    print_test "Test 17: 删除用户 (DELETE /api/auth/users/:user_id)"

    RESPONSE=$(curl -s -X DELETE "$API_BASE/api/auth/users/$TEST_USER_ID" \
        -H "Authorization: Bearer $TOKEN")

    echo "$RESPONSE" | jq .

    SUCCESS=$(echo "$RESPONSE" | jq -r '.success')

    if [ "$SUCCESS" == "true" ]; then
        print_success "删除用户成功"
    else
        print_error "删除用户失败"
    fi
}

# Test 18: 登出
test_logout() {
    print_test "Test 18: 用户登出 (POST /api/auth/logout)"

    RESPONSE=$(curl -s -X POST "$API_BASE/api/auth/logout" \
        -H "Authorization: Bearer $TOKEN")

    echo "$RESPONSE" | jq .

    SUCCESS=$(echo "$RESPONSE" | jq -r '.success')

    if [ "$SUCCESS" == "true" ]; then
        print_success "登出成功"
    else
        print_error "登出失败"
    fi
}

# Test 19: 登出后访问（应该失败）
test_after_logout() {
    print_test "Test 19: 登出后访问受保护端点 (应该失败)"

    RESPONSE=$(curl -s -X GET "$API_BASE/api/auth/user" \
        -H "Authorization: Bearer $TOKEN" \
        -w "\nHTTP_CODE:%{http_code}")

    HTTP_CODE=$(echo "$RESPONSE" | grep "HTTP_CODE" | cut -d: -f2)

    if [ "$HTTP_CODE" == "401" ]; then
        print_success "登出后 Token 正确失效"
    else
        print_error "登出后 Token 应该失效"
    fi
}

# 主测试流程
main() {
    print_separator
    echo "Artemis 认证系统集成测试"
    print_separator

    check_server

    # 执行所有测试
    test_login
    test_login_fail
    test_get_current_user
    test_get_permissions
    test_list_roles
    test_check_permission
    test_list_users
    test_create_user
    test_get_user
    test_update_user
    test_update_user_status
    test_reset_password
    test_list_sessions
    test_login_history
    test_refresh_token
    test_unauthorized_access
    test_delete_user
    test_logout
    test_after_logout

    print_separator
    echo -e "${GREEN}所有测试通过! ✓${NC}"
    print_separator
}

# 运行测试
main
