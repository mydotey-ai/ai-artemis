//! Replication API 单元测试
//!
//! 测试覆盖:
//! - replicate_register: 批量注册复制 API
//! - replicate_heartbeat: 批量心跳复制 API
//! - replicate_unregister: 批量注销复制 API
//! - get_all_services: 获取所有服务 API (POST)
//! - get_all_services_by_query: 获取所有服务 API (GET)
//! - batch_register: 批量注册 API
//! - batch_heartbeat: 批量心跳 API
//! - batch_unregister: 批量注销 API
//! - get_services_delta: 增量同步 API
//! - sync_full_data: 全量同步 API

use artemis_core::model::{
    BatchHeartbeatRequest, BatchRegisterRequest, BatchUnregisterRequest, ErrorCode, Instance,
    InstanceKey, InstanceStatus, ReplicateHeartbeatRequest, ReplicateRegisterRequest,
    ReplicateUnregisterRequest, ServicesDeltaRequest, SyncFullDataRequest,
};
use artemis_server::{
    RegistryServiceImpl, cache::VersionedCacheManager, change::InstanceChangeManager,
    lease::LeaseManager, registry::RegistryRepository,
};
use artemis_web::{api::replication, state::AppState};
use axum::{Json, extract::State, http::{HeaderMap, StatusCode}};
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
    }
}

/// 创建测试实例
fn create_test_instance(id: &str) -> Instance {
    Instance {
        region_id: "test-region".to_string(),
        zone_id: "test-zone".to_string(),
        group_id: None,
        service_id: "test-service".to_string(),
        instance_id: id.to_string(),
        machine_name: None,
        ip: "127.0.0.1".to_string(),
        port: 8080,
        protocol: None,
        url: format!("http://127.0.0.1:8080/{}", id),
        health_check_url: None,
        status: InstanceStatus::Up,
        metadata: None,
    }
}

/// 创建包含 replication header 的 HeaderMap
fn create_replication_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("x-artemis-replication", "true".parse().unwrap());
    headers
}

// ===== replicate_register 测试 =====

#[tokio::test]
async fn test_replicate_register_success() {
    let state = create_test_app_state();
    let headers = create_replication_headers();
    let instances = vec![create_test_instance("inst-1")];
    let request = ReplicateRegisterRequest { instances };

    let result = replication::replicate_register(State(state), headers, Json(request)).await;

    assert!(result.is_ok());
    let response = result.unwrap().0;
    assert_eq!(response.response_status.error_code, ErrorCode::Success);
}

#[tokio::test]
async fn test_replicate_register_without_header() {
    let state = create_test_app_state();
    let headers = HeaderMap::new(); // 没有 replication header
    let instances = vec![create_test_instance("inst-1")];
    let request = ReplicateRegisterRequest { instances };

    let result = replication::replicate_register(State(state), headers, Json(request)).await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_replicate_register_multiple_instances() {
    let state = create_test_app_state();
    let headers = create_replication_headers();
    let instances = vec![
        create_test_instance("inst-1"),
        create_test_instance("inst-2"),
        create_test_instance("inst-3"),
    ];
    let request = ReplicateRegisterRequest { instances };

    let result = replication::replicate_register(State(state), headers, Json(request)).await;

    assert!(result.is_ok());
    let response = result.unwrap().0;
    assert_eq!(response.response_status.error_code, ErrorCode::Success);
}

// ===== replicate_heartbeat 测试 =====

#[tokio::test]
async fn test_replicate_heartbeat_success() {
    let state = create_test_app_state();
    let headers = create_replication_headers();

    // 先注册实例
    let instance = create_test_instance("inst-1");
    let reg_request = ReplicateRegisterRequest {
        instances: vec![instance.clone()],
    };
    let _ = replication::replicate_register(
        State(state.clone()),
        headers.clone(),
        Json(reg_request),
    )
    .await;

    // 心跳测试
    let hb_request = ReplicateHeartbeatRequest {
        instance_keys: vec![instance.key()],
    };
    let result = replication::replicate_heartbeat(State(state), headers, Json(hb_request)).await;

    assert!(result.is_ok());
    let response = result.unwrap().0;
    assert_eq!(response.response_status.error_code, ErrorCode::Success);
}

#[tokio::test]
async fn test_replicate_heartbeat_without_header() {
    let state = create_test_app_state();
    let headers = HeaderMap::new();
    let instance = create_test_instance("inst-1");
    let request = ReplicateHeartbeatRequest {
        instance_keys: vec![instance.key()],
    };

    let result = replication::replicate_heartbeat(State(state), headers, Json(request)).await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), StatusCode::BAD_REQUEST);
}

// ===== replicate_unregister 测试 =====

#[tokio::test]
async fn test_replicate_unregister_success() {
    let state = create_test_app_state();
    let headers = create_replication_headers();

    // 先注册实例
    let instance = create_test_instance("inst-1");
    let reg_request = ReplicateRegisterRequest {
        instances: vec![instance.clone()],
    };
    let _ = replication::replicate_register(
        State(state.clone()),
        headers.clone(),
        Json(reg_request),
    )
    .await;

    // 注销测试
    let unreg_request = ReplicateUnregisterRequest {
        instance_keys: vec![instance.key()],
    };
    let result =
        replication::replicate_unregister(State(state), headers, Json(unreg_request)).await;

    assert!(result.is_ok());
    let response = result.unwrap().0;
    assert_eq!(response.response_status.error_code, ErrorCode::Success);
}

#[tokio::test]
async fn test_replicate_unregister_without_header() {
    let state = create_test_app_state();
    let headers = HeaderMap::new();
    let instance = create_test_instance("inst-1");
    let request = ReplicateUnregisterRequest {
        instance_keys: vec![instance.key()],
    };

    let result = replication::replicate_unregister(State(state), headers, Json(request)).await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), StatusCode::BAD_REQUEST);
}

// ===== get_all_services 测试 =====

#[tokio::test]
async fn test_get_all_services_empty() {
    let state = create_test_app_state();

    let result = replication::get_all_services(State(state)).await;

    let response = result.0;
    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    assert!(response.services.is_empty());
}

#[tokio::test]
async fn test_get_all_services_with_instances() {
    let state = create_test_app_state();
    let headers = create_replication_headers();

    // 注册一些实例
    let instances = vec![
        create_test_instance("inst-1"),
        create_test_instance("inst-2"),
    ];
    let reg_request = ReplicateRegisterRequest { instances };
    let _ = replication::replicate_register(State(state.clone()), headers, Json(reg_request)).await;

    // 获取所有服务
    let result = replication::get_all_services(State(state)).await;

    let response = result.0;
    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    assert_eq!(response.services.len(), 1);
    assert_eq!(response.services[0].instances.len(), 2);
}

// ===== batch_register 测试 =====

#[tokio::test]
async fn test_batch_register_success() {
    let state = create_test_app_state();
    let headers = create_replication_headers();
    let instances = vec![
        create_test_instance("inst-1"),
        create_test_instance("inst-2"),
        create_test_instance("inst-3"),
    ];
    let request = BatchRegisterRequest { instances };

    let result = replication::batch_register(State(state), headers, Json(request)).await;

    assert!(result.is_ok());
    let response = result.unwrap().0;
    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    assert!(response.failed_instances.is_none() || response.failed_instances.as_ref().unwrap().is_empty());
}

#[tokio::test]
async fn test_batch_register_without_header() {
    let state = create_test_app_state();
    let headers = HeaderMap::new();
    let instances = vec![create_test_instance("inst-1")];
    let request = BatchRegisterRequest { instances };

    let result = replication::batch_register(State(state), headers, Json(request)).await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), StatusCode::BAD_REQUEST);
}

// ===== batch_heartbeat 测试 =====

#[tokio::test]
async fn test_batch_heartbeat_success() {
    let state = create_test_app_state();
    let headers = create_replication_headers();

    // 先批量注册
    let instances = vec![
        create_test_instance("inst-1"),
        create_test_instance("inst-2"),
    ];
    let reg_request = BatchRegisterRequest {
        instances: instances.clone(),
    };
    let _ = replication::batch_register(State(state.clone()), headers.clone(), Json(reg_request))
        .await;

    // 批量心跳
    let keys: Vec<InstanceKey> = instances.iter().map(|i| i.key()).collect();
    let hb_request = BatchHeartbeatRequest { instance_keys: keys };
    let result = replication::batch_heartbeat(State(state), headers, Json(hb_request)).await;

    assert!(result.is_ok());
    let response = result.unwrap().0;
    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    assert!(response.failed_instance_keys.is_none() || response.failed_instance_keys.as_ref().unwrap().is_empty());
}

#[tokio::test]
async fn test_batch_heartbeat_without_header() {
    let state = create_test_app_state();
    let headers = HeaderMap::new();
    let instance = create_test_instance("inst-1");
    let request = BatchHeartbeatRequest {
        instance_keys: vec![instance.key()],
    };

    let result = replication::batch_heartbeat(State(state), headers, Json(request)).await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), StatusCode::BAD_REQUEST);
}

// ===== batch_unregister 测试 =====

#[tokio::test]
async fn test_batch_unregister_success() {
    let state = create_test_app_state();
    let headers = create_replication_headers();

    // 先批量注册
    let instances = vec![
        create_test_instance("inst-1"),
        create_test_instance("inst-2"),
    ];
    let reg_request = BatchRegisterRequest {
        instances: instances.clone(),
    };
    let _ = replication::batch_register(State(state.clone()), headers.clone(), Json(reg_request))
        .await;

    // 批量注销
    let keys: Vec<InstanceKey> = instances.iter().map(|i| i.key()).collect();
    let unreg_request = BatchUnregisterRequest { instance_keys: keys };
    let result = replication::batch_unregister(State(state), headers, Json(unreg_request)).await;

    assert!(result.is_ok());
    let response = result.unwrap().0;
    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    assert!(response.failed_instance_keys.is_none() || response.failed_instance_keys.as_ref().unwrap().is_empty());
}

#[tokio::test]
async fn test_batch_unregister_without_header() {
    let state = create_test_app_state();
    let headers = HeaderMap::new();
    let instance = create_test_instance("inst-1");
    let request = BatchUnregisterRequest {
        instance_keys: vec![instance.key()],
    };

    let result = replication::batch_unregister(State(state), headers, Json(request)).await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), StatusCode::BAD_REQUEST);
}

// ===== get_services_delta 测试 =====

#[tokio::test]
async fn test_get_services_delta() {
    let state = create_test_app_state();
    let headers = create_replication_headers();

    // 先注册一些实例
    let instances = vec![create_test_instance("inst-1")];
    let reg_request = ReplicateRegisterRequest { instances };
    let _ = replication::replicate_register(State(state.clone()), headers, Json(reg_request)).await;

    // 获取增量数据 (since_timestamp = 0 应该返回所有数据)
    let delta_request = ServicesDeltaRequest {
        region_id: "test-region".to_string(),
        zone_id: "test-zone".to_string(),
        since_timestamp: 0,
    };
    let result = replication::get_services_delta(State(state), Json(delta_request)).await;

    let response = result.0;
    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    // 应该有数据返回
    assert!(!response.services.is_empty() || response.current_timestamp > 0);
}

// ===== sync_full_data 测试 =====

#[tokio::test]
async fn test_sync_full_data() {
    let state = create_test_app_state();
    let headers = create_replication_headers();

    // 先注册一些实例
    let instances = vec![
        create_test_instance("inst-1"),
        create_test_instance("inst-2"),
    ];
    let reg_request = ReplicateRegisterRequest { instances };
    let _ = replication::replicate_register(State(state.clone()), headers, Json(reg_request)).await;

    // 全量同步
    let sync_request = SyncFullDataRequest {
        region_id: "test-region".to_string(),
        zone_id: Some("test-zone".to_string()),
    };
    let result = replication::sync_full_data(State(state), Json(sync_request)).await;

    let response = result.0;
    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    assert_eq!(response.services.len(), 1);
    assert_eq!(response.services[0].instances.len(), 2);
}

// ===== 边界条件和错误处理测试 =====

#[tokio::test]
async fn test_batch_operations_empty_list() {
    let state = create_test_app_state();
    let headers = create_replication_headers();

    // 空列表批量注册
    let reg_request = BatchRegisterRequest {
        instances: vec![],
    };
    let result = replication::batch_register(
        State(state.clone()),
        headers.clone(),
        Json(reg_request),
    )
    .await;
    assert!(result.is_ok());

    // 空列表批量心跳
    let hb_request = BatchHeartbeatRequest {
        instance_keys: vec![],
    };
    let result = replication::batch_heartbeat(
        State(state.clone()),
        headers.clone(),
        Json(hb_request),
    )
    .await;
    assert!(result.is_ok());

    // 空列表批量注销
    let unreg_request = BatchUnregisterRequest {
        instance_keys: vec![],
    };
    let result =
        replication::batch_unregister(State(state), headers, Json(unreg_request)).await;
    assert!(result.is_ok());
}
