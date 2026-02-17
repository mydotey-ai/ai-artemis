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
use artemis_management::model::{
    GetAllInstanceOperationsRequest, GetAllServerOperationsRequest, GetInstanceOperationsRequest,
    InstanceOperation, IsInstanceDownRequest, IsServerDownRequest, OperateInstanceRequest,
    OperateServerRequest,
};
use artemis_management::web::api::instance as instance_api;
use artemis_management::{InstanceManager, ManagementState};
use axum::{Json, extract::State};
use std::sync::Arc;

/// 创建测试用的 ManagementState
fn create_test_management_state() -> ManagementState {
    let instance_manager = Arc::new(InstanceManager::new());

    ManagementState::new(
        Default::default(), // auth_manager
        instance_manager,
        Default::default(), // group_manager
        Default::default(), // route_manager
        Default::default(), // zone_manager
        Default::default(), // canary_manager
        Default::default(), // audit_manager
    )
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
    let state = create_test_management_state();
    let instance_key = create_test_instance_key();

    let request = OperateInstanceRequest {
        instance_key: instance_key.clone(),
        operation: InstanceOperation::PullOut,
        operator_id: "test-operator".to_string(),
        operation_complete: true,
        token: None,
    };

    let response = instance_api::operate_instance(State(state.clone()), Json(request)).await;
    let (parts, _body) = response.into_parts();

    assert_eq!(parts.status, 200); // StatusCode::OK

    // 验证实例确实被拉出
    let is_down = state.instance_manager.is_instance_down(&instance_key);
    assert!(is_down);
}

#[tokio::test]
async fn test_operate_instance_pull_in() {
    let state = create_test_management_state();
    let instance_key = create_test_instance_key();

    // 先拉出
    let _ = state.instance_manager.pull_out_instance(&instance_key, "test-operator".to_string(), true);

    // 然后拉入
    let request = OperateInstanceRequest {
        instance_key: instance_key.clone(),
        operation: InstanceOperation::PullIn,
        operator_id: "test-operator".to_string(),
        operation_complete: true,
        token: None,
    };

    let response = instance_api::operate_instance(State(state.clone()), Json(request)).await;
    let (parts, _) = response.into_parts();

    assert_eq!(parts.status, 200);

    // 验证实例已被拉入
    let is_down = state.instance_manager.is_instance_down(&instance_key);
    assert!(!is_down);
}

// ===== get_instance_operations 测试 =====

#[tokio::test]
async fn test_get_instance_operations() {
    let state = create_test_management_state();
    let instance_key = create_test_instance_key();

    // 执行一些操作
    let _ = state.instance_manager.pull_out_instance(&instance_key, "operator1".to_string(), true);

    let request = GetInstanceOperationsRequest { instance_key };

    let response = instance_api::get_instance_operations(State(state), Json(request)).await;
    let (parts, _) = response.into_parts();

    assert_eq!(parts.status, 200);
}

// ===== is_instance_down 测试 =====

#[tokio::test]
async fn test_is_instance_down_true() {
    let state = create_test_management_state();
    let instance_key = create_test_instance_key();

    // 拉出实例
    let _ = state.instance_manager.pull_out_instance(&instance_key, "test-operator".to_string(), true);

    let request = IsInstanceDownRequest { instance_key };

    let response = instance_api::is_instance_down(State(state), Json(request)).await;
    let (parts, _) = response.into_parts();

    assert_eq!(parts.status, 200);
}

#[tokio::test]
async fn test_is_instance_down_false() {
    let state = create_test_management_state();
    let instance_key = create_test_instance_key();

    let request = IsInstanceDownRequest { instance_key };

    let response = instance_api::is_instance_down(State(state), Json(request)).await;
    let (parts, _) = response.into_parts();

    assert_eq!(parts.status, 200);
}

// ===== operate_server 测试 =====

#[tokio::test]
async fn test_operate_server_pull_out() {
    let state = create_test_management_state();

    let request = OperateServerRequest {
        server_id: "test-server".to_string(),
        region_id: "test-region".to_string(),
        operation: artemis_management::model::ServerOperation::PullOut,
        operator_id: "test-operator".to_string(),
        operation_complete: true,
        token: None,
    };

    let response = instance_api::operate_server(State(state.clone()), Json(request)).await;
    let (parts, _) = response.into_parts();

    assert_eq!(parts.status, 200);

    // 验证服务器确实被拉出
    let is_down = state.instance_manager.is_server_down("test-server", "test-region");
    assert!(is_down);
}

#[tokio::test]
async fn test_operate_server_pull_in() {
    let state = create_test_management_state();

    // 先拉出
    let _ = state.instance_manager.pull_out_server("test-server", "test-region", "test-operator".to_string(), true);

    // 然后拉入
    let request = OperateServerRequest {
        server_id: "test-server".to_string(),
        region_id: "test-region".to_string(),
        operation: artemis_management::model::ServerOperation::PullIn,
        operator_id: "test-operator".to_string(),
        operation_complete: true,
        token: None,
    };

    let response = instance_api::operate_server(State(state.clone()), Json(request)).await;
    let (parts, _) = response.into_parts();

    assert_eq!(parts.status, 200);

    // 验证服务器已被拉入
    let is_down = state.instance_manager.is_server_down("test-server", "test-region");
    assert!(!is_down);
}

// ===== is_server_down 测试 =====

#[tokio::test]
async fn test_is_server_down_true() {
    let state = create_test_management_state();

    // 拉出服务器
    let _ = state.instance_manager.pull_out_server("test-server", "test-region", "test-operator".to_string(), true);

    let request = IsServerDownRequest {
        server_id: "test-server".to_string(),
        region_id: "test-region".to_string(),
    };

    let response = instance_api::is_server_down(State(state), Json(request)).await;
    let (parts, _) = response.into_parts();

    assert_eq!(parts.status, 200);
}

#[tokio::test]
async fn test_is_server_down_false() {
    let state = create_test_management_state();

    let request = IsServerDownRequest {
        server_id: "test-server".to_string(),
        region_id: "test-region".to_string(),
    };

    let response = instance_api::is_server_down(State(state), Json(request)).await;
    let (parts, _) = response.into_parts();

    assert_eq!(parts.status, 200);
}

// ===== get_all_instance_operations 测试 =====

#[tokio::test]
async fn test_get_all_instance_operations_post() {
    let state = create_test_management_state();

    // 执行一些操作
    let instance_key1 = create_test_instance_key();
    let mut instance_key2 = create_test_instance_key();
    instance_key2.instance_id = "test-instance-2".to_string();

    let _ = state.instance_manager.pull_out_instance(&instance_key1, "operator1".to_string(), true);
    let _ = state.instance_manager.pull_out_instance(&instance_key2, "operator2".to_string(), true);

    let request = GetAllInstanceOperationsRequest { region_id: None };

    let response = instance_api::get_all_instance_operations_post(State(state), Json(request)).await;
    let (parts, _) = response.into_parts();

    assert_eq!(parts.status, 200);
}

#[tokio::test]
async fn test_get_all_instance_operations_get() {
    let state = create_test_management_state();

    // 执行一些操作
    let instance_key = create_test_instance_key();
    let _ = state.instance_manager.pull_out_instance(&instance_key, "operator1".to_string(), true);

    use artemis_management::web::api::instance::AllInstanceOperationsQuery;
    let query = AllInstanceOperationsQuery { region_id: None };

    let response = instance_api::get_all_instance_operations_get(State(state), axum::extract::Query(query)).await;
    let (parts, _) = response.into_parts();

    assert_eq!(parts.status, 200);
}

// ===== get_all_server_operations 测试 =====

#[tokio::test]
async fn test_get_all_server_operations_post() {
    let state = create_test_management_state();

    // 执行一些操作
    let _ = state.instance_manager.pull_out_server("server1", "region1", "operator1".to_string(), true);
    let _ = state.instance_manager.pull_out_server("server2", "region1", "operator2".to_string(), true);

    let request = GetAllServerOperationsRequest { region_id: None };

    let response = instance_api::get_all_server_operations_post(State(state), Json(request)).await;
    let (parts, _) = response.into_parts();

    assert_eq!(parts.status, 200);
}

#[tokio::test]
async fn test_get_all_server_operations_get() {
    let state = create_test_management_state();

    // 执行一些操作
    let _ = state.instance_manager.pull_out_server("server1", "region1", "operator1".to_string(), true);

    use artemis_management::web::api::instance::AllServerOperationsQuery;
    let query = AllServerOperationsQuery { region_id: None };

    let response = instance_api::get_all_server_operations_get(State(state), axum::extract::Query(query)).await;
    let (parts, _) = response.into_parts();

    assert_eq!(parts.status, 200);
}

// ===== 边界条件测试 =====

#[tokio::test]
async fn test_operate_instance_idempotent() {
    let state = create_test_management_state();
    let instance_key = create_test_instance_key();

    let request = OperateInstanceRequest {
        instance_key: instance_key.clone(),
        operation: InstanceOperation::PullOut,
        operator_id: "test-operator".to_string(),
        operation_complete: true,
        token: None,
    };

    // 第一次
    let response1 = instance_api::operate_instance(State(state.clone()), Json(request.clone())).await;
    let (parts1, _) = response1.into_parts();
    assert_eq!(parts1.status, 200);

    // 第二次 (幂等性测试)
    let response2 = instance_api::operate_instance(State(state.clone()), Json(request)).await;
    let (parts2, _) = response2.into_parts();
    assert_eq!(parts2.status, 200);

    // 验证状态一致
    let is_down = state.instance_manager.is_instance_down(&instance_key);
    assert!(is_down);
}

#[tokio::test]
async fn test_get_all_operations_with_region_filter() {
    let state = create_test_management_state();

    // 不同 region 的操作
    let mut instance_key1 = create_test_instance_key();
    instance_key1.region_id = "region1".to_string();

    let mut instance_key2 = create_test_instance_key();
    instance_key2.region_id = "region2".to_string();

    let _ = state.instance_manager.pull_out_instance(&instance_key1, "operator1".to_string(), true);
    let _ = state.instance_manager.pull_out_instance(&instance_key2, "operator2".to_string(), true);

    // 查询特定 region 的操作
    let request = GetAllInstanceOperationsRequest { region_id: Some("region1".to_string()) };

    let response = instance_api::get_all_instance_operations_post(State(state), Json(request)).await;
    let (parts, _) = response.into_parts();

    assert_eq!(parts.status, 200);
}
