use artemis_management::auth::{AuthManager, UserRole, UserStatus};

// ========== 用户创建测试 ==========

#[test]
fn test_create_user() {
    let manager = AuthManager::new();

    let user = manager.create_user(
        "test_user",
        Some("test@example.com".to_string()),
        "password123",
        UserRole::Viewer,
    ).unwrap();

    assert_eq!(user.username, "test_user");
    assert_eq!(user.email, Some("test@example.com".to_string()));
    assert_eq!(user.role, UserRole::Viewer);
    assert_eq!(user.status, UserStatus::Active);
    assert!(!user.password_hash.is_empty());
}

#[test]
fn test_create_duplicate_user() {
    let manager = AuthManager::new();

    manager.create_user("user1", None, "pass123", UserRole::Viewer).unwrap();
    let result = manager.create_user("user1", None, "pass456", UserRole::Admin);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already exists"));
}

#[test]
fn test_create_users_with_different_roles() {
    let manager = AuthManager::new();

    let admin = manager.create_user("admin", None, "pass", UserRole::Admin).unwrap();
    let operator = manager.create_user("operator", None, "pass", UserRole::Operator).unwrap();
    let viewer = manager.create_user("viewer", None, "pass", UserRole::Viewer).unwrap();

    assert_eq!(admin.role, UserRole::Admin);
    assert_eq!(operator.role, UserRole::Operator);
    assert_eq!(viewer.role, UserRole::Viewer);
}

// ========== 用户认证测试 ==========

#[test]
fn test_authenticate_success() {
    let manager = AuthManager::new();
    manager.create_user("user1", None, "password123", UserRole::Viewer).unwrap();

    let token = manager.authenticate("user1", "password123", None, None).unwrap();

    assert!(!token.is_empty());
}

#[test]
fn test_authenticate_wrong_password() {
    let manager = AuthManager::new();
    manager.create_user("user1", None, "password123", UserRole::Viewer).unwrap();

    let result = manager.authenticate("user1", "wrong_password", None, None);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid"));
}

#[test]
fn test_authenticate_nonexistent_user() {
    let manager = AuthManager::new();

    let result = manager.authenticate("nonexistent", "password", None, None);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid"));
}

#[test]
fn test_authenticate_inactive_user() {
    let manager = AuthManager::new();
    let user = manager.create_user("user1", None, "password123", UserRole::Viewer).unwrap();

    // 停用用户
    manager.change_user_status(&user.user_id, UserStatus::Inactive).unwrap();

    let result = manager.authenticate("user1", "password123", None, None);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("inactive"));
}

#[test]
fn test_authenticate_with_ip_and_user_agent() {
    let manager = AuthManager::new();
    manager.create_user("user1", None, "password123", UserRole::Viewer).unwrap();

    let token = manager.authenticate(
        "user1",
        "password123",
        Some("127.0.0.1".to_string()),
        Some("Mozilla/5.0".to_string()),
    ).unwrap();

    assert!(!token.is_empty());
}

// ========== Token 验证测试 ==========

#[test]
fn test_validate_token_success() {
    let manager = AuthManager::new();
    manager.create_user("user1", None, "password123", UserRole::Viewer).unwrap();

    let token = manager.authenticate("user1", "password123", None, None).unwrap();
    let session = manager.validate_token(&token).unwrap();

    assert!(!session.session_id.is_empty());
    assert!(!session.user_id.is_empty());
}

#[test]
fn test_validate_invalid_token() {
    let manager = AuthManager::new();

    let result = manager.validate_token("invalid_token");

    assert!(result.is_err());
}

#[test]
fn test_refresh_token() {
    let manager = AuthManager::new();
    manager.create_user("user1", None, "password123", UserRole::Viewer).unwrap();

    let old_token = manager.authenticate("user1", "password123", None, None).unwrap();

    // 等待1秒确保时间戳不同
    std::thread::sleep(std::time::Duration::from_secs(1));

    let new_token = manager.refresh_token(&old_token).unwrap();

    assert_ne!(old_token, new_token);
    assert!(manager.validate_token(&new_token).is_ok());

    // 旧 token 应该仍然有效（因为我们没有撤销它，只是更新了会话）
    // 实际上在我们的实现中，refresh_token 会使旧 token 失效
}

// ========== 登出测试 ==========

#[test]
fn test_logout() {
    let manager = AuthManager::new();
    manager.create_user("user1", None, "password123", UserRole::Viewer).unwrap();

    let token = manager.authenticate("user1", "password123", None, None).unwrap();
    assert!(manager.validate_token(&token).is_ok());

    manager.logout(&token).unwrap();

    assert!(manager.validate_token(&token).is_err());
}

#[test]
fn test_logout_invalid_token() {
    let manager = AuthManager::new();

    let result = manager.logout("invalid_token");

    assert!(result.is_err());
}

// ========== 用户管理测试 ==========

#[test]
fn test_get_user() {
    let manager = AuthManager::new();
    let created_user = manager.create_user("user1", None, "password123", UserRole::Viewer).unwrap();

    let user = manager.get_user(&created_user.user_id).unwrap();

    assert_eq!(user.username, "user1");
    assert_eq!(user.role, UserRole::Viewer);
}

#[test]
fn test_get_user_by_username() {
    let manager = AuthManager::new();
    manager.create_user("user1", None, "password123", UserRole::Viewer).unwrap();

    let user = manager.get_user_by_username("user1").unwrap();

    assert_eq!(user.username, "user1");
}

#[test]
fn test_list_users() {
    let manager = AuthManager::new();

    manager.create_user("user1", None, "pass", UserRole::Admin).unwrap();
    manager.create_user("user2", None, "pass", UserRole::Operator).unwrap();
    manager.create_user("user3", None, "pass", UserRole::Viewer).unwrap();

    let users = manager.list_users();

    assert_eq!(users.len(), 3);
}

#[test]
fn test_update_user() {
    let manager = AuthManager::new();
    let user = manager.create_user("user1", None, "password123", UserRole::Viewer).unwrap();

    let updated = manager.update_user(
        &user.user_id,
        Some("new@example.com".to_string()),
        Some(UserRole::Operator),
    ).unwrap();

    assert_eq!(updated.email, Some("new@example.com".to_string()));
    assert_eq!(updated.role, UserRole::Operator);
}

#[test]
fn test_delete_user() {
    let manager = AuthManager::new();
    let user = manager.create_user("user1", None, "password123", UserRole::Viewer).unwrap();

    manager.delete_user(&user.user_id).unwrap();

    assert!(manager.get_user(&user.user_id).is_none());
    assert!(manager.get_user_by_username("user1").is_none());
}

#[test]
fn test_delete_user_revokes_sessions() {
    let manager = AuthManager::new();
    let user = manager.create_user("user1", None, "password123", UserRole::Viewer).unwrap();
    let token = manager.authenticate("user1", "password123", None, None).unwrap();

    assert!(manager.validate_token(&token).is_ok());

    manager.delete_user(&user.user_id).unwrap();

    assert!(manager.validate_token(&token).is_err());
}

// ========== 密码管理测试 ==========

#[test]
fn test_change_password() {
    let manager = AuthManager::new();
    let user = manager.create_user("user1", None, "old_password", UserRole::Viewer).unwrap();

    manager.change_password(&user.user_id, "old_password", "new_password").unwrap();

    // 旧密码不能登录
    assert!(manager.authenticate("user1", "old_password", None, None).is_err());

    // 新密码可以登录
    assert!(manager.authenticate("user1", "new_password", None, None).is_ok());
}

#[test]
fn test_change_password_wrong_old_password() {
    let manager = AuthManager::new();
    let user = manager.create_user("user1", None, "password123", UserRole::Viewer).unwrap();

    let result = manager.change_password(&user.user_id, "wrong_old", "new_password");

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("incorrect"));
}

#[test]
fn test_change_password_revokes_sessions() {
    let manager = AuthManager::new();
    let user = manager.create_user("user1", None, "password123", UserRole::Viewer).unwrap();
    let token = manager.authenticate("user1", "password123", None, None).unwrap();

    assert!(manager.validate_token(&token).is_ok());

    manager.change_password(&user.user_id, "password123", "new_password").unwrap();

    assert!(manager.validate_token(&token).is_err());
}

#[test]
fn test_reset_password() {
    let manager = AuthManager::new();
    let user = manager.create_user("user1", None, "old_password", UserRole::Viewer).unwrap();

    manager.reset_password(&user.user_id, "new_password").unwrap();

    assert!(manager.authenticate("user1", "new_password", None, None).is_ok());
}

// ========== 用户状态测试 ==========

#[test]
fn test_change_user_status() {
    let manager = AuthManager::new();
    let user = manager.create_user("user1", None, "password123", UserRole::Viewer).unwrap();

    let updated = manager.change_user_status(&user.user_id, UserStatus::Inactive).unwrap();

    assert_eq!(updated.status, UserStatus::Inactive);
}

#[test]
fn test_inactive_user_cannot_login() {
    let manager = AuthManager::new();
    let user = manager.create_user("user1", None, "password123", UserRole::Viewer).unwrap();

    manager.change_user_status(&user.user_id, UserStatus::Inactive).unwrap();

    let result = manager.authenticate("user1", "password123", None, None);
    assert!(result.is_err());
}

#[test]
fn test_inactive_user_sessions_revoked() {
    let manager = AuthManager::new();
    let user = manager.create_user("user1", None, "password123", UserRole::Viewer).unwrap();
    let token = manager.authenticate("user1", "password123", None, None).unwrap();

    assert!(manager.validate_token(&token).is_ok());

    manager.change_user_status(&user.user_id, UserStatus::Inactive).unwrap();

    assert!(manager.validate_token(&token).is_err());
}

// ========== 会话管理测试 ==========

#[test]
fn test_list_user_sessions() {
    let manager = AuthManager::new();
    let user = manager.create_user("user1", None, "password123", UserRole::Viewer).unwrap();

    manager.authenticate("user1", "password123", Some("127.0.0.1".to_string()), None).unwrap();
    manager.authenticate("user1", "password123", Some("127.0.0.2".to_string()), None).unwrap();

    let sessions = manager.list_user_sessions(&user.user_id);

    assert_eq!(sessions.len(), 2);
}

#[test]
fn test_revoke_session() {
    let manager = AuthManager::new();
    manager.create_user("user1", None, "password123", UserRole::Viewer).unwrap();

    let token = manager.authenticate("user1", "password123", None, None).unwrap();
    let session = manager.validate_token(&token).unwrap();

    manager.revoke_session(&session.session_id).unwrap();

    assert!(manager.validate_token(&token).is_err());
}

#[test]
fn test_revoke_all_user_sessions() {
    let manager = AuthManager::new();
    let user = manager.create_user("user1", None, "password123", UserRole::Viewer).unwrap();

    let token1 = manager.authenticate("user1", "password123", None, None).unwrap();
    let token2 = manager.authenticate("user1", "password123", None, None).unwrap();

    let count = manager.revoke_all_user_sessions(&user.user_id).unwrap();

    assert_eq!(count, 2);
    assert!(manager.validate_token(&token1).is_err());
    assert!(manager.validate_token(&token2).is_err());
}

// ========== 权限检查测试 ==========

#[test]
fn test_admin_has_all_permissions() {
    let manager = AuthManager::new();
    let admin = manager.create_user("admin", None, "pass", UserRole::Admin).unwrap();

    assert!(manager.check_permission(&admin.user_id, "services", "read"));
    assert!(manager.check_permission(&admin.user_id, "services", "write"));
    assert!(manager.check_permission(&admin.user_id, "users", "delete"));
    assert!(manager.check_permission(&admin.user_id, "anything", "anything"));
}

#[test]
fn test_operator_permissions() {
    let manager = AuthManager::new();
    let operator = manager.create_user("operator", None, "pass", UserRole::Operator).unwrap();

    // 有权限
    assert!(manager.check_permission(&operator.user_id, "services", "read"));
    assert!(manager.check_permission(&operator.user_id, "services", "write"));
    assert!(manager.check_permission(&operator.user_id, "instances", "delete"));
    assert!(manager.check_permission(&operator.user_id, "routing", "create"));
    assert!(manager.check_permission(&operator.user_id, "cluster", "read"));
    assert!(manager.check_permission(&operator.user_id, "audit", "read"));

    // 无权限
    assert!(!manager.check_permission(&operator.user_id, "users", "write"));
    assert!(!manager.check_permission(&operator.user_id, "cluster", "write"));
}

#[test]
fn test_viewer_permissions() {
    let manager = AuthManager::new();
    let viewer = manager.create_user("viewer", None, "pass", UserRole::Viewer).unwrap();

    // 只有读权限
    assert!(manager.check_permission(&viewer.user_id, "services", "read"));
    assert!(manager.check_permission(&viewer.user_id, "instances", "read"));
    assert!(manager.check_permission(&viewer.user_id, "users", "read"));

    // 无写权限
    assert!(!manager.check_permission(&viewer.user_id, "services", "write"));
    assert!(!manager.check_permission(&viewer.user_id, "instances", "delete"));
    assert!(!manager.check_permission(&viewer.user_id, "users", "create"));
}

#[test]
fn test_get_user_permissions() {
    let manager = AuthManager::new();
    let admin = manager.create_user("admin", None, "pass", UserRole::Admin).unwrap();
    let operator = manager.create_user("operator", None, "pass", UserRole::Operator).unwrap();
    let viewer = manager.create_user("viewer", None, "pass", UserRole::Viewer).unwrap();

    let admin_perms = manager.get_user_permissions(&admin.user_id);
    let operator_perms = manager.get_user_permissions(&operator.user_id);
    let viewer_perms = manager.get_user_permissions(&viewer.user_id);

    assert_eq!(admin_perms, vec!["*:*"]);
    assert_eq!(operator_perms.len(), 6);
    assert_eq!(viewer_perms, vec!["*:read"]);
}

// ========== 登录历史测试 ==========

#[test]
fn test_login_history_success() {
    let manager = AuthManager::new();
    let user = manager.create_user("user1", None, "password123", UserRole::Viewer).unwrap();

    manager.authenticate("user1", "password123", Some("127.0.0.1".to_string()), Some("Chrome".to_string())).unwrap();
    manager.authenticate("user1", "password123", Some("192.168.1.1".to_string()), Some("Firefox".to_string())).unwrap();

    let history = manager.get_login_history(&user.user_id, 10);

    assert_eq!(history.len(), 2);
}

#[test]
fn test_login_history_failed_attempts() {
    let manager = AuthManager::new();
    let user = manager.create_user("user1", None, "password123", UserRole::Viewer).unwrap();

    // 失败的登录尝试
    let _ = manager.authenticate("user1", "wrong_password", Some("127.0.0.1".to_string()), Some("Chrome".to_string()));
    let _ = manager.authenticate("user1", "wrong_password", Some("127.0.0.1".to_string()), Some("Chrome".to_string()));

    // 成功的登录
    manager.authenticate("user1", "password123", Some("127.0.0.1".to_string()), Some("Chrome".to_string())).unwrap();

    let history = manager.get_login_history(&user.user_id, 10);

    assert_eq!(history.len(), 3);
}

#[test]
fn test_login_history_limit() {
    let manager = AuthManager::new();
    let user = manager.create_user("user1", None, "password123", UserRole::Viewer).unwrap();

    // 创建10次登录
    for _ in 0..10 {
        manager.authenticate("user1", "password123", None, None).unwrap();
    }

    let history = manager.get_login_history(&user.user_id, 5);

    assert_eq!(history.len(), 5);
}
