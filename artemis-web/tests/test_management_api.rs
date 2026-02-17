//! Management API 单元测试
//!
//! 测试覆盖:
//! - operate_instance: 实例拉入/拉出操作
//! - get_instance_operations: 查询实例操作历史
//! - is_instance_down: 查询实例是否被拉出
//! - operate_server: 服务器批量拉入/拉出操作
//! - is_server_down: 查询服务器是否被拉出
//! - get_all_instance_operations: 查询所有实例操作
//! - get_all_server_operations: 查询所有服务器操作

use artemis_core::model::InstanceKey;
use artemis_management::{InstanceManager, model::InstanceOperation};
use std::sync::Arc;

/// 创建测试用的 InstanceManager
fn create_test_instance_manager() -> Arc<InstanceManager> {
    Arc::new(InstanceManager::new())
}

/// 创建测试实例
fn create_test_instance_key() -> InstanceKey {
    InstanceKey {
        region_id: "test-region".to_string(),
        zone_id: "test-zone".to_string(),
        group_id: String::new(),
        service_id: "test-service".to_string(),
        instance_id: "test-instance-1".to_string(),
    }
}

// ===== operate_instance 测试 =====

#[tokio::test]
async fn test_operate_instance_pull_out() {
    let instance_manager = create_test_instance_manager();
    let instance_key = create_test_instance_key();

    // 拉出实例
    let result = instance_manager.pull_out_instance(&instance_key, "test-operator".to_string(), true);

    assert!(result.is_ok());

    // 验证实例确实被拉出
    let is_down = instance_manager.is_instance_down(&instance_key);
    assert!(is_down);
}

#[tokio::test]
async fn test_operate_instance_pull_in() {
    let instance_manager = create_test_instance_manager();
    let instance_key = create_test_instance_key();

    // 先拉出
    let _ = instance_manager.pull_out_instance(&instance_key, "test-operator".to_string(), true);

    // 然后拉入
    let result = instance_manager.pull_in_instance(&instance_key, "test-operator".to_string(), true);

    assert!(result.is_ok());

    // 验证实例已被拉入
    let is_down = instance_manager.is_instance_down(&instance_key);
    assert!(!is_down);
}

// ===== get_instance_operations 测试 =====

#[tokio::test]
async fn test_get_instance_operations() {
    let instance_manager = create_test_instance_manager();
    let instance_key = create_test_instance_key();

    // 执行一些操作
    let _ = instance_manager.pull_out_instance(&instance_key, "operator1".to_string(), true);

    // 获取操作历史
    let operations = instance_manager.get_instance_operations(&instance_key);

    // 返回的是操作类型列表
    assert_eq!(operations.len(), 1);
    assert_eq!(operations[0], InstanceOperation::PullOut);
}

#[tokio::test]
async fn test_is_instance_down_true() {
    let instance_manager = create_test_instance_manager();
    let instance_key = create_test_instance_key();

    // 拉出实例
    let _ = instance_manager.pull_out_instance(&instance_key, "test-operator".to_string(), true);

    // 验证实例被拉出
    let is_down = instance_manager.is_instance_down(&instance_key);
    assert!(is_down);
}

#[tokio::test]
async fn test_is_instance_down_false() {
    let instance_manager = create_test_instance_manager();
    let instance_key = create_test_instance_key();

    // 验证实例未被拉出
    let is_down = instance_manager.is_instance_down(&instance_key);
    assert!(!is_down);
}

// ===== operate_server 测试 =====

#[tokio::test]
async fn test_operate_server_pull_out() {
    let instance_manager = create_test_instance_manager();

    // 拉出服务器
    let result = instance_manager.pull_out_server("test-server", "test-region", "test-operator".to_string(), true);

    assert!(result.is_ok());

    // 验证服务器确实被拉出
    let is_down = instance_manager.is_server_down("test-server", "test-region");
    assert!(is_down);
}

#[tokio::test]
async fn test_operate_server_pull_in() {
    let instance_manager = create_test_instance_manager();

    // 先拉出
    let _ = instance_manager.pull_out_server("test-server", "test-region", "test-operator".to_string(), true);

    // 然后拉入
    let result = instance_manager.pull_in_server("test-server", "test-region", "test-operator".to_string(), true);

    assert!(result.is_ok());

    // 验证服务器已被拉入
    let is_down = instance_manager.is_server_down("test-server", "test-region");
    assert!(!is_down);
}

// ===== is_server_down 测试 =====

#[tokio::test]
async fn test_is_server_down_true() {
    let instance_manager = create_test_instance_manager();

    // 拉出服务器
    let _ = instance_manager.pull_out_server("test-server", "test-region", "test-operator".to_string(), true);

    // 验证服务器被拉出
    let is_down = instance_manager.is_server_down("test-server", "test-region");
    assert!(is_down);
}

#[tokio::test]
async fn test_is_server_down_false() {
    let instance_manager = create_test_instance_manager();

    // 验证服务器未被拉出
    let is_down = instance_manager.is_server_down("test-server", "test-region");
    assert!(!is_down);
}

// ===== get_all_instance_operations 测试 =====

#[tokio::test]
async fn test_get_all_instance_operations() {
    let instance_manager = create_test_instance_manager();

    // 执行一些操作
    let instance_key1 = create_test_instance_key();
    let mut instance_key2 = create_test_instance_key();
    instance_key2.instance_id = "test-instance-2".to_string();

    let _ = instance_manager.pull_out_instance(&instance_key1, "operator1".to_string(), true);
    let _ = instance_manager.pull_out_instance(&instance_key2, "operator2".to_string(), true);

    // 获取所有操作
    let all_operations = instance_manager.get_all_instance_operations(None);

    assert!(all_operations.len() >= 2);
}

#[tokio::test]
async fn test_get_all_instance_operations_with_region_filter() {
    let instance_manager = create_test_instance_manager();

    // 不同 region 的操作
    let mut instance_key1 = create_test_instance_key();
    instance_key1.region_id = "region1".to_string();

    let mut instance_key2 = create_test_instance_key();
    instance_key2.region_id = "region2".to_string();

    let _ = instance_manager.pull_out_instance(&instance_key1, "operator1".to_string(), true);
    let _ = instance_manager.pull_out_instance(&instance_key2, "operator2".to_string(), true);

    // 查询特定 region 的操作
    let region1_operations = instance_manager.get_all_instance_operations(Some("region1"));

    assert_eq!(region1_operations.len(), 1);
    assert_eq!(region1_operations[0].instance_key.region_id, "region1");
}

// ===== get_all_server_operations 测试 =====

#[tokio::test]
async fn test_get_all_server_operations() {
    let instance_manager = create_test_instance_manager();

    // 执行一些操作
    let _ = instance_manager.pull_out_server("server1", "region1", "operator1".to_string(), true);
    let _ = instance_manager.pull_out_server("server2", "region1", "operator2".to_string(), true);

    // 获取所有操作
    let all_operations = instance_manager.get_all_server_operations(None);

    assert!(all_operations.len() >= 2);
}

#[tokio::test]
async fn test_get_all_server_operations_with_region_filter() {
    let instance_manager = create_test_instance_manager();

    // 不同 region 的操作
    let _ = instance_manager.pull_out_server("server1", "region1", "operator1".to_string(), true);
    let _ = instance_manager.pull_out_server("server2", "region2", "operator2".to_string(), true);

    // 查询特定 region 的操作
    let region1_operations = instance_manager.get_all_server_operations(Some("region1"));

    assert_eq!(region1_operations.len(), 1);
    assert_eq!(region1_operations[0].0, "server1"); // (server_id, region_id, operation)
}

// ===== 边界条件测试 =====

#[tokio::test]
async fn test_operate_instance_idempotent() {
    let instance_manager = create_test_instance_manager();
    let instance_key = create_test_instance_key();

    // 多次拉出同一个实例 - 应该幂等
    let result1 = instance_manager.pull_out_instance(&instance_key, "test-operator".to_string(), true);
    let result2 = instance_manager.pull_out_instance(&instance_key, "test-operator".to_string(), true);

    assert!(result1.is_ok());
    assert!(result2.is_ok());

    // 验证状态一致
    let is_down = instance_manager.is_instance_down(&instance_key);
    assert!(is_down);
}

#[tokio::test]
async fn test_operate_server_idempotent() {
    let instance_manager = create_test_instance_manager();

    // 多次拉出同一个服务器 - 应该幂等
    let result1 = instance_manager.pull_out_server("test-server", "test-region", "test-operator".to_string(), true);
    let result2 = instance_manager.pull_out_server("test-server", "test-region", "test-operator".to_string(), true);

    assert!(result1.is_ok());
    assert!(result2.is_ok());

    // 验证状态一致
    let is_down = instance_manager.is_server_down("test-server", "test-region");
    assert!(is_down);
}

#[tokio::test]
async fn test_empty_operations() {
    let instance_manager = create_test_instance_manager();
    let instance_key = create_test_instance_key();

    // 查询不存在的实例操作
    let operations = instance_manager.get_instance_operations(&instance_key);

    assert_eq!(operations.len(), 0);

    // 查询不存在的服务器的所有操作
    let server_ops = instance_manager.get_all_server_operations(None);

    // 应该为空（因为我们没有执行任何服务器操作）
    assert_eq!(server_ops.len(), 0);
}

