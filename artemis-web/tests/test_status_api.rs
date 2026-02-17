//! Status API 单元测试
//!
//! 测试覆盖:
//! - get_cluster_node_status: 获取集群节点状态 (POST/GET)
//! - get_cluster_status: 获取集群状态 (POST/GET)
//! - get_leases_status: 获取租约状态 (POST/GET)
//! - get_legacy_leases_status: 获取传统租约状态 (POST/GET)
//! - get_config_status: 获取配置状态 (POST/GET)
//! - get_deployment_status: 获取部署状态 (POST/GET)

use artemis_core::model::{
    GetClusterNodeStatusRequest, GetClusterStatusRequest, GetConfigStatusRequest,
    GetDeploymentStatusRequest, GetLeasesStatusRequest,
};
use artemis_server::{
    RegistryServiceImpl, cache::VersionedCacheManager, change::InstanceChangeManager,
    lease::LeaseManager, registry::RegistryRepository,
};
use artemis_web::{api::status, state::AppState};
use axum::{Json, extract::{Query, State}, response::IntoResponse};
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

// ===== get_cluster_node_status 测试 =====

#[tokio::test]
async fn test_get_cluster_node_status_post() {
    let state = create_test_app_state();
    let request = GetClusterNodeStatusRequest {};

    let response = status::get_cluster_node_status_post(State(state), Json(request)).await;
    let json_response = response.into_response();

    // 验证响应成功生成
    assert_eq!(json_response.status(), 200);
}

#[tokio::test]
async fn test_get_cluster_node_status_get() {
    let state = create_test_app_state();

    let response = status::get_cluster_node_status_get(State(state)).await;
    let json_response = response.into_response();

    assert_eq!(json_response.status(), 200);
}

// ===== get_cluster_status 测试 =====

#[tokio::test]
async fn test_get_cluster_status_post() {
    let state = create_test_app_state();
    let request = GetClusterStatusRequest {};

    let response = status::get_cluster_status_post(State(state), Json(request)).await;
    let json_response = response.into_response();

    assert_eq!(json_response.status(), 200);
}

#[tokio::test]
async fn test_get_cluster_status_get() {
    let state = create_test_app_state();

    let response = status::get_cluster_status_get(State(state)).await;
    let json_response = response.into_response();

    assert_eq!(json_response.status(), 200);
}

// ===== get_leases_status 测试 =====

#[tokio::test]
async fn test_get_leases_status_post_no_filter() {
    let state = create_test_app_state();
    let request = GetLeasesStatusRequest {
        service_ids: None,
    };

    let response = status::get_leases_status_post(State(state), Json(request)).await;
    let json_response = response.into_response();

    assert_eq!(json_response.status(), 200);
}

#[tokio::test]
async fn test_get_leases_status_post_with_filter() {
    let state = create_test_app_state();
    let request = GetLeasesStatusRequest {
        service_ids: Some(vec!["test-service".to_string()]),
    };

    let response = status::get_leases_status_post(State(state), Json(request)).await;
    let json_response = response.into_response();

    assert_eq!(json_response.status(), 200);
}

#[tokio::test]
async fn test_get_leases_status_get_no_filter() {
    let state = create_test_app_state();
    let query = status::GetLeasesQuery { app_ids: None };

    let response = status::get_leases_status_get(State(state), Query(query)).await;
    let json_response = response.into_response();

    assert_eq!(json_response.status(), 200);
}

#[tokio::test]
async fn test_get_leases_status_get_with_filter() {
    let state = create_test_app_state();
    let query = status::GetLeasesQuery {
        app_ids: Some(vec!["test-service".to_string()]),
    };

    let response = status::get_leases_status_get(State(state), Query(query)).await;
    let json_response = response.into_response();

    assert_eq!(json_response.status(), 200);
}

// ===== get_legacy_leases_status 测试 =====

#[tokio::test]
async fn test_get_legacy_leases_status_post() {
    let state = create_test_app_state();
    let request = GetLeasesStatusRequest {
        service_ids: None,
    };

    let response = status::get_legacy_leases_status_post(State(state), Json(request)).await;
    let json_response = response.into_response();

    assert_eq!(json_response.status(), 200);
}

#[tokio::test]
async fn test_get_legacy_leases_status_get() {
    let state = create_test_app_state();
    let query = status::GetLeasesQuery { app_ids: None };

    let response = status::get_legacy_leases_status_get(State(state), Query(query)).await;
    let json_response = response.into_response();

    assert_eq!(json_response.status(), 200);
}

// ===== get_config_status 测试 =====

#[tokio::test]
async fn test_get_config_status_post() {
    let state = create_test_app_state();
    let request = GetConfigStatusRequest {};

    let response = status::get_config_status_post(State(state), Json(request)).await;
    let json_response = response.into_response();

    assert_eq!(json_response.status(), 200);
}

#[tokio::test]
async fn test_get_config_status_get() {
    let state = create_test_app_state();

    let response = status::get_config_status_get(State(state)).await;
    let json_response = response.into_response();

    assert_eq!(json_response.status(), 200);
}

// ===== get_deployment_status 测试 =====

#[tokio::test]
async fn test_get_deployment_status_post() {
    let state = create_test_app_state();
    let request = GetDeploymentStatusRequest {};

    let response = status::get_deployment_status_post(State(state), Json(request)).await;
    let json_response = response.into_response();

    assert_eq!(json_response.status(), 200);
}

#[tokio::test]
async fn test_get_deployment_status_get() {
    let state = create_test_app_state();

    let response = status::get_deployment_status_get(State(state)).await;
    let json_response = response.into_response();

    assert_eq!(json_response.status(), 200);
}

// ===== 边界条件测试 =====

#[tokio::test]
async fn test_get_leases_status_empty_service_ids() {
    let state = create_test_app_state();
    let request = GetLeasesStatusRequest {
        service_ids: Some(vec![]),
    };

    let response = status::get_leases_status_post(State(state), Json(request)).await;
    let json_response = response.into_response();

    assert_eq!(json_response.status(), 200);
}

#[tokio::test]
async fn test_get_leases_status_multiple_service_ids() {
    let state = create_test_app_state();
    let request = GetLeasesStatusRequest {
        service_ids: Some(vec![
            "service-1".to_string(),
            "service-2".to_string(),
            "service-3".to_string(),
        ]),
    };

    let response = status::get_leases_status_post(State(state), Json(request)).await;
    let json_response = response.into_response();

    assert_eq!(json_response.status(), 200);
}

// ===== POST vs GET 一致性测试 =====

#[tokio::test]
async fn test_cluster_node_status_post_get_consistency() {
    let state = create_test_app_state();

    // POST
    let request = GetClusterNodeStatusRequest {};
    let post_response = status::get_cluster_node_status_post(State(state.clone()), Json(request))
        .await
        .into_response();

    // GET
    let get_response = status::get_cluster_node_status_get(State(state))
        .await
        .into_response();

    // 两者都应该成功
    assert_eq!(post_response.status(), 200);
    assert_eq!(get_response.status(), 200);
}

#[tokio::test]
async fn test_cluster_status_post_get_consistency() {
    let state = create_test_app_state();

    // POST
    let request = GetClusterStatusRequest {};
    let post_response = status::get_cluster_status_post(State(state.clone()), Json(request))
        .await
        .into_response();

    // GET
    let get_response = status::get_cluster_status_get(State(state))
        .await
        .into_response();

    assert_eq!(post_response.status(), 200);
    assert_eq!(get_response.status(), 200);
}

#[tokio::test]
async fn test_config_status_post_get_consistency() {
    let state = create_test_app_state();

    // POST
    let request = GetConfigStatusRequest {};
    let post_response = status::get_config_status_post(State(state.clone()), Json(request))
        .await
        .into_response();

    // GET
    let get_response = status::get_config_status_get(State(state))
        .await
        .into_response();

    assert_eq!(post_response.status(), 200);
    assert_eq!(get_response.status(), 200);
}

#[tokio::test]
async fn test_deployment_status_post_get_consistency() {
    let state = create_test_app_state();

    // POST
    let request = GetDeploymentStatusRequest {};
    let post_response = status::get_deployment_status_post(State(state.clone()), Json(request))
        .await
        .into_response();

    // GET
    let get_response = status::get_deployment_status_get(State(state))
        .await
        .into_response();

    assert_eq!(post_response.status(), 200);
    assert_eq!(get_response.status(), 200);
}
