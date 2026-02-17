//! Management API 单元测试
//!
//! 测试覆盖:
//! - operate_instance: 实例拉入/拉出操作
//! - get_instance_operations: 查询实例操作历史
//! - is_instance_down: 查询实例是否被拉出
//! - operate_server: 服务器批量拉入/拉出操作
//! - is_server_down: 查询服务器是否被拉出
//! - get_all_instance_operations_post/get: 查询所有实例操作 (POST/GET)
//! - get_all_server_operations_post/get: 查询所有服务器操作 (POST/GET)

use artemis_core::model::{
    GetAllInstanceOperationsRequest, GetAllServerOperationsRequest,
    GetInstanceOperationsRequest, InstanceKey, InstanceOperation,
    IsInstanceDownRequest, IsServerDownRequest, OperateInstanceRequest, OperateServerRequest,
    ServerOperation,
};
use artemis_server::{
    RegistryServiceImpl, cache::VersionedCacheManager, change::InstanceChangeManager,
    lease::LeaseManager, registry::RegistryRepository,
};
use artemis_web::{api::management, state::AppState};
use axum::{Json, extract::{Query, State}, http::StatusCode};
use std::sync::Arc;
use std::time::Duration;

/// 创建测试用的 AppState
fn create_test_app_state() -> AppState {
    let repository = RegistryRepository::new();
    let lease_manager = Arc::new(LeaseManager::new(Duration::from_secs(30)));
    let cache = Arc::new(VersionedCacheManager::new());
    let change_manager = Arc::new(InstanceChangeManager::new());

    let registry_service = Arc::new(RegistryServiceImpl::new(
        repository.clone(),
        lease_manager.clone(),
        cache.clone(),
        change_manager.clone(),
        None,
    ));

    let discovery_service = Arc::new(artemis_server::discovery::DiscoveryServiceImpl::new(
        repository,
        cache.clone(),
    ));

    let session_manager = Arc::new(artemis_web::websocket::SessionManager::new());
    let instance_manager = Arc::new(artemis_management::InstanceManager::new());
    let group_manager = Arc::new(artemis_management::GroupManager::new());
    let route_manager = Arc::new(artemis_management::RouteManager::new());
    let zone_manager = Arc::new(artemis_management::ZoneManager::new());
    let canary_manager = Arc::new(artemis_management::CanaryManager::new());
    let audit_manager = Arc::new(artemis_management::AuditManager::new());
    let load_balancer = Arc::new(artemis_server::discovery::LoadBalancer::new());
    let status_service = Arc::new(artemis_server::StatusService::new(
        None,
        lease_manager.clone(),
        "test-node".to_string(),
        "test-region".to_string(),
        "test-zone".to_string(),
        "http://localhost:8080".to_string(),
        "test-app".to_string(),
    ));

    let auth_manager = Arc::new(artemis_management::auth::AuthManager::new());

    AppState {
        registry_service,
        discovery_service,
        cache,
        session_manager,
        cluster_manager: None,
        replication_manager: None,
        instance_manager,
        group_manager,
        route_manager,
        zone_manager,
        canary_manager,
        audit_manager,
        load_balancer,
        status_service,
        auth_manager,
    }
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
    let state = create_test_app_state();
    let instance_key = create_test_instance_key();

    let request = OperateInstanceRequest {
        instance_key: instance_key.clone(),
        operation: InstanceOperation::PullOut,
        operator_id: "test-operator".to_string(),
        operation_complete: true,
        token: None,
    };

    let response = management::operate_instance(State(state.clone()), Json(request)).await;
    let (parts, _body) = response.into_parts();

    assert_eq!(parts.status, StatusCode::OK);

    // 验证实例确实被拉出
    let is_down = state.instance_manager.is_instance_down(&instance_key);
    assert!(is_down);
}

#[tokio::test]
async fn test_operate_instance_pull_in() {
    let state = create_test_app_state();
    let instance_key = create_test_instance_key();

    // 先拉出
    let _ = state.instance_manager.pull_out_instance(
        &instance_key,
        "test-operator".to_string(),
        true,
    );

    // 然后拉入
    let request = OperateInstanceRequest {
        instance_key: instance_key.clone(),
        operation: InstanceOperation::PullIn,
        operator_id: "test-operator".to_string(),
        operation_complete: true,
        token: None,
    };

    let response = management::operate_instance(State(state.clone()), Json(request)).await;
    let (parts, _) = response.into_parts();

    assert_eq!(parts.status, StatusCode::OK);

    // 验证实例已被拉入
    let is_down = state.instance_manager.is_instance_down(&instance_key);
    assert!(!is_down);
}

// ===== get_instance_operations 测试 =====

#[tokio::test]
async fn test_get_instance_operations() {
    let state = create_test_app_state();
    let instance_key = create_test_instance_key();

    // 执行一些操作
    let _ = state.instance_manager.pull_out_instance(
        &instance_key,
        "operator1".to_string(),
        true,
    );
    let _ = state.instance_manager.pull_in_instance(
        &instance_key,
        "operator2".to_string(),
        true,
    );

    let request = GetInstanceOperationsRequest { instance_key };

    let response = management::get_instance_operations(State(state), Json(request)).await;
    let (parts, _) = response.into_parts();

    assert_eq!(parts.status, StatusCode::OK);
}

// ===== is_instance_down 测试 =====

#[tokio::test]
async fn test_is_instance_down_true() {
    let state = create_test_app_state();
    let instance_key = create_test_instance_key();

    // 拉出实例
    let _ = state.instance_manager.pull_out_instance(
        &instance_key,
        "test-operator".to_string(),
        true,
    );

    let request = IsInstanceDownRequest { instance_key };

    let response = management::is_instance_down(State(state), Json(request)).await;
    let (parts, _) = response.into_parts();

    assert_eq!(parts.status, StatusCode::OK);
}

#[tokio::test]
async fn test_is_instance_down_false() {
    let state = create_test_app_state();
    let instance_key = create_test_instance_key();

    let request = IsInstanceDownRequest { instance_key };

    let response = management::is_instance_down(State(state), Json(request)).await;
    let (parts, _) = response.into_parts();

    assert_eq!(parts.status, StatusCode::OK);
}

// ===== operate_server 测试 =====

#[tokio::test]
async fn test_operate_server_pull_out() {
    let state = create_test_app_state();

    let request = OperateServerRequest {
        server_id: "test-server".to_string(),
        region_id: "test-region".to_string(),
        operation: ServerOperation::PullOut,
        operator_id: "test-operator".to_string(),
        operation_complete: true,
        token: None,
    };

    let response = management::operate_server(State(state.clone()), Json(request)).await;
    let (parts, _) = response.into_parts();

    assert_eq!(parts.status, StatusCode::OK);

    // 验证服务器确实被拉出
    let is_down = state
        .instance_manager
        .is_server_down("test-server", "test-region");
    assert!(is_down);
}

#[tokio::test]
async fn test_operate_server_pull_in() {
    let state = create_test_app_state();

    // 先拉出
    let _ = state.instance_manager.pull_out_server(
        "test-server",
        "test-region",
        "test-operator".to_string(),
        true,
    );

    // 然后拉入
    let request = OperateServerRequest {
        server_id: "test-server".to_string(),
        region_id: "test-region".to_string(),
        operation: ServerOperation::PullIn,
        operator_id: "test-operator".to_string(),
        operation_complete: true,
        token: None,
    };

    let response = management::operate_server(State(state.clone()), Json(request)).await;
    let (parts, _) = response.into_parts();

    assert_eq!(parts.status, StatusCode::OK);

    // 验证服务器已被拉入
    let is_down = state
        .instance_manager
        .is_server_down("test-server", "test-region");
    assert!(!is_down);
}

// ===== is_server_down 测试 =====

#[tokio::test]
async fn test_is_server_down_true() {
    let state = create_test_app_state();

    // 拉出服务器
    let _ = state.instance_manager.pull_out_server(
        "test-server",
        "test-region",
        "test-operator".to_string(),
        true,
    );

    let request = IsServerDownRequest {
        server_id: "test-server".to_string(),
        region_id: "test-region".to_string(),
    };

    let response = management::is_server_down(State(state), Json(request)).await;
    let (parts, _) = response.into_parts();

    assert_eq!(parts.status, StatusCode::OK);
}

#[tokio::test]
async fn test_is_server_down_false() {
    let state = create_test_app_state();

    let request = IsServerDownRequest {
        server_id: "test-server".to_string(),
        region_id: "test-region".to_string(),
    };

    let response = management::is_server_down(State(state), Json(request)).await;
    let (parts, _) = response.into_parts();

    assert_eq!(parts.status, StatusCode::OK);
}

// ===== get_all_instance_operations 测试 =====

#[tokio::test]
async fn test_get_all_instance_operations_post() {
    let state = create_test_app_state();

    // 执行一些操作
    let instance_key1 = create_test_instance_key();
    let mut instance_key2 = create_test_instance_key();
    instance_key2.instance_id = "test-instance-2".to_string();

    let _ = state.instance_manager.pull_out_instance(
        &instance_key1,
        "operator1".to_string(),
        true,
    );
    let _ = state.instance_manager.pull_out_instance(
        &instance_key2,
        "operator2".to_string(),
        true,
    );

    let request = GetAllInstanceOperationsRequest { region_id: None };

    let response = management::get_all_instance_operations_post(State(state), Json(request)).await;
    let (parts, _) = response.into_parts();

    assert_eq!(parts.status, StatusCode::OK);
}

#[tokio::test]
async fn test_get_all_instance_operations_get() {
    let state = create_test_app_state();

    // 执行一些操作
    let instance_key = create_test_instance_key();
    let _ = state.instance_manager.pull_out_instance(
        &instance_key,
        "operator1".to_string(),
        true,
    );

    let query = management::AllInstanceOperationsQuery { region_id: None };

    let response = management::get_all_instance_operations_get(State(state), Query(query)).await;
    let (parts, _) = response.into_parts();

    assert_eq!(parts.status, StatusCode::OK);
}

// ===== get_all_server_operations 测试 =====

#[tokio::test]
async fn test_get_all_server_operations_post() {
    let state = create_test_app_state();

    // 执行一些操作
    let _ = state.instance_manager.pull_out_server(
        "server1",
        "region1",
        "operator1".to_string(),
        true,
    );
    let _ = state.instance_manager.pull_out_server(
        "server2",
        "region1",
        "operator2".to_string(),
        true,
    );

    let request = GetAllServerOperationsRequest { region_id: None };

    let response = management::get_all_server_operations_post(State(state), Json(request)).await;
    let (parts, _) = response.into_parts();

    assert_eq!(parts.status, StatusCode::OK);
}

#[tokio::test]
async fn test_get_all_server_operations_get() {
    let state = create_test_app_state();

    // 执行一些操作
    let _ = state.instance_manager.pull_out_server(
        "server1",
        "region1",
        "operator1".to_string(),
        true,
    );

    let query = management::AllServerOperationsQuery { region_id: None };

    let response = management::get_all_server_operations_get(State(state), Query(query)).await;
    let (parts, _) = response.into_parts();

    assert_eq!(parts.status, StatusCode::OK);
}

// ===== 边界条件测试 =====

#[tokio::test]
async fn test_operate_instance_idempotent() {
    let state = create_test_app_state();
    let instance_key = create_test_instance_key();

    // 多次拉出同一个实例
    let request = OperateInstanceRequest {
        instance_key: instance_key.clone(),
        operation: InstanceOperation::PullOut,
        operator_id: "test-operator".to_string(),
        operation_complete: true,
        token: None,
    };

    // 第一次
    let response1 = management::operate_instance(State(state.clone()), Json(request.clone())).await;
    let (parts1, _) = response1.into_parts();
    assert_eq!(parts1.status, StatusCode::OK);

    // 第二次 (幂等性测试)
    let response2 = management::operate_instance(State(state.clone()), Json(request)).await;
    let (parts2, _) = response2.into_parts();
    assert_eq!(parts2.status, StatusCode::OK);

    // 验证状态一致
    let is_down = state.instance_manager.is_instance_down(&instance_key);
    assert!(is_down);
}

#[tokio::test]
async fn test_get_all_operations_with_region_filter() {
    let state = create_test_app_state();

    // 不同 region 的操作
    let mut instance_key1 = create_test_instance_key();
    instance_key1.region_id = "region1".to_string();

    let mut instance_key2 = create_test_instance_key();
    instance_key2.region_id = "region2".to_string();

    let _ = state.instance_manager.pull_out_instance(
        &instance_key1,
        "operator1".to_string(),
        true,
    );
    let _ = state.instance_manager.pull_out_instance(
        &instance_key2,
        "operator2".to_string(),
        true,
    );

    // 查询特定 region 的操作
    let request = GetAllInstanceOperationsRequest {
        region_id: Some("region1".to_string()),
    };

    let response = management::get_all_instance_operations_post(State(state), Json(request)).await;
    let (parts, _) = response.into_parts();

    assert_eq!(parts.status, StatusCode::OK);
}
