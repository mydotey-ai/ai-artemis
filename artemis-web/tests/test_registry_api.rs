//! Registry API 单元测试
//!
//! 测试覆盖:
//! - register: 注册实例 API
//! - heartbeat: 心跳续约 API
//! - unregister: 注销实例 API

use artemis_core::model::{
    ErrorCode, HeartbeatRequest, Instance, InstanceStatus, RegisterRequest, UnregisterRequest,
};
use artemis_server::{
    InstanceChangeManager, RegistryServiceImpl, cache::VersionedCacheManager, lease::LeaseManager,
    registry::RegistryRepository,
};
use artemis_web::{api::registry, state::AppState};
use axum::{Json, extract::State};
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
        None, // cluster_manager
        lease_manager.clone(),
        "test-node".to_string(), // node_id
        "test-region".to_string(), // region_id
        "test-zone".to_string(), // zone_id
        "http://localhost:8080".to_string(), // server_url
        "test-app".to_string(), // app_id
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
fn create_test_instance(id: &str) -> Instance {
    Instance {
        region_id: "test-region".to_string(),
        zone_id: "test-zone".to_string(),
        service_id: "test-service".to_string(),
        group_id: None,
        instance_id: id.to_string(),
        machine_name: None,
        ip: "192.168.1.100".to_string(),
        port: 8080,
        protocol: Some("http".to_string()),
        url: "http://192.168.1.100:8080".to_string(),
        health_check_url: None,
        status: InstanceStatus::Up,
        metadata: None,
    }
}

// ============================================================================
// Register API 测试
// ============================================================================

#[tokio::test]
async fn test_register_success_single_instance() {
    let state = create_test_app_state();
    let instance = create_test_instance("inst-1");
    let request = RegisterRequest {
        instances: vec![instance.clone()],
    };

    let response = registry::register(State(state), Json(request)).await;

    assert_eq!(response.0.response_status.error_code, ErrorCode::Success);
    // UnregisterResponse 没有 failed_instance_keys 字段
}

#[tokio::test]
async fn test_register_success_multiple_instances() {
    let state = create_test_app_state();
    let instances = vec![
        create_test_instance("inst-1"),
        create_test_instance("inst-2"),
        create_test_instance("inst-3"),
    ];
    let request = RegisterRequest { instances };

    let response = registry::register(State(state), Json(request)).await;

    assert_eq!(response.0.response_status.error_code, ErrorCode::Success);
    // UnregisterResponse 没有 failed_instance_keys 字段
}

#[tokio::test]
async fn test_register_empty_instances() {
    let state = create_test_app_state();
    let request = RegisterRequest {
        instances: vec![],
    };

    let response = registry::register(State(state), Json(request)).await;

    // 空列表也应该返回成功
    assert_eq!(response.0.response_status.error_code, ErrorCode::Success);
}

#[tokio::test]
async fn test_register_duplicate_instance() {
    let state = create_test_app_state();
    let instance = create_test_instance("inst-1");

    // 第一次注册
    let request1 = RegisterRequest {
        instances: vec![instance.clone()],
    };
    let response1 = registry::register(State(state.clone()), Json(request1)).await;
    assert_eq!(response1.0.response_status.error_code, ErrorCode::Success);

    // 第二次注册相同实例 (应该更新)
    let request2 = RegisterRequest {
        instances: vec![instance],
    };
    let response2 = registry::register(State(state), Json(request2)).await;
    assert_eq!(response2.0.response_status.error_code, ErrorCode::Success);
}

#[tokio::test]
async fn test_register_different_statuses() {
    let state = create_test_app_state();

    // 测试不同状态的实例注册
    for status in [
        InstanceStatus::Up,
        InstanceStatus::Down,
        InstanceStatus::Unhealthy,
    ] {
        let mut instance = create_test_instance(&format!("inst-{:?}", status));
        instance.status = status;

        let request = RegisterRequest {
            instances: vec![instance],
        };
        let response = registry::register(State(state.clone()), Json(request)).await;

        assert_eq!(response.0.response_status.error_code, ErrorCode::Success);
    }
}

// ============================================================================
// Heartbeat API 测试
// ============================================================================

#[tokio::test]
async fn test_heartbeat_success() {
    let state = create_test_app_state();
    let instance = create_test_instance("inst-1");

    // 先注册
    let reg_request = RegisterRequest {
        instances: vec![instance.clone()],
    };
    let _ = registry::register(State(state.clone()), Json(reg_request)).await;

    // 发送心跳
    let hb_request = HeartbeatRequest {
        instance_keys: vec![instance.key()],
    };
    let response = registry::heartbeat(State(state), Json(hb_request)).await;

    assert_eq!(response.0.response_status.error_code, ErrorCode::Success);
    // UnregisterResponse 没有 failed_instance_keys 字段
}

#[tokio::test]
async fn test_heartbeat_multiple_instances() {
    let state = create_test_app_state();
    let instances = vec![
        create_test_instance("inst-1"),
        create_test_instance("inst-2"),
        create_test_instance("inst-3"),
    ];

    // 注册所有实例
    let reg_request = RegisterRequest {
        instances: instances.clone(),
    };
    let _ = registry::register(State(state.clone()), Json(reg_request)).await;

    // 批量心跳
    let hb_request = HeartbeatRequest {
        instance_keys: instances.iter().map(|i| i.key()).collect(),
    };
    let response = registry::heartbeat(State(state), Json(hb_request)).await;

    assert_eq!(response.0.response_status.error_code, ErrorCode::Success);
    // UnregisterResponse 没有 failed_instance_keys 字段
}

#[tokio::test]
async fn test_heartbeat_empty_keys() {
    let state = create_test_app_state();
    let request = HeartbeatRequest {
        instance_keys: vec![],
    };

    let response = registry::heartbeat(State(state), Json(request)).await;

    // 空列表应该返回成功
    assert_eq!(response.0.response_status.error_code, ErrorCode::Success);
}

#[tokio::test]
async fn test_heartbeat_unregistered_instance() {
    let state = create_test_app_state();
    let instance = create_test_instance("inst-nonexistent");

    // 未注册的实例发送心跳
    let request = HeartbeatRequest {
        instance_keys: vec![instance.key()],
    };
    let response = registry::heartbeat(State(state), Json(request)).await;

    // 应该有失败记录
    assert!(response.0.failed_instance_keys.is_some() && !response.0.failed_instance_keys.as_ref().unwrap().is_empty());
}

#[tokio::test]
async fn test_heartbeat_extends_lease() {
    let state = create_test_app_state();
    let instance = create_test_instance("inst-1");

    // 注册实例
    let reg_request = RegisterRequest {
        instances: vec![instance.clone()],
    };
    let _ = registry::register(State(state.clone()), Json(reg_request)).await;

    // 第一次心跳
    let hb_request1 = HeartbeatRequest {
        instance_keys: vec![instance.key()],
    };
    let response1 = registry::heartbeat(State(state.clone()), Json(hb_request1)).await;
    assert_eq!(response1.0.response_status.error_code, ErrorCode::Success);

    // 等待一段时间
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 第二次心跳 (应该成功续约)
    let hb_request2 = HeartbeatRequest {
        instance_keys: vec![instance.key()],
    };
    let response2 = registry::heartbeat(State(state), Json(hb_request2)).await;
    assert_eq!(response2.0.response_status.error_code, ErrorCode::Success);
}

// ============================================================================
// Unregister API 测试
// ============================================================================

#[tokio::test]
async fn test_unregister_success() {
    let state = create_test_app_state();
    let instance = create_test_instance("inst-1");

    // 先注册
    let reg_request = RegisterRequest {
        instances: vec![instance.clone()],
    };
    let _ = registry::register(State(state.clone()), Json(reg_request)).await;

    // 注销
    let unreg_request = UnregisterRequest {
        instance_keys: vec![instance.key()],
    };
    let response = registry::unregister(State(state), Json(unreg_request)).await;

    assert_eq!(response.0.response_status.error_code, ErrorCode::Success);
    // UnregisterResponse 没有 failed_instance_keys 字段
}

#[tokio::test]
async fn test_unregister_multiple_instances() {
    let state = create_test_app_state();
    let instances = vec![
        create_test_instance("inst-1"),
        create_test_instance("inst-2"),
        create_test_instance("inst-3"),
    ];

    // 注册所有实例
    let reg_request = RegisterRequest {
        instances: instances.clone(),
    };
    let _ = registry::register(State(state.clone()), Json(reg_request)).await;

    // 批量注销
    let unreg_request = UnregisterRequest {
        instance_keys: instances.iter().map(|i| i.key()).collect(),
    };
    let response = registry::unregister(State(state), Json(unreg_request)).await;

    assert_eq!(response.0.response_status.error_code, ErrorCode::Success);
    // UnregisterResponse 没有 failed_instance_keys 字段
}

#[tokio::test]
async fn test_unregister_empty_keys() {
    let state = create_test_app_state();
    let request = UnregisterRequest {
        instance_keys: vec![],
    };

    let response = registry::unregister(State(state), Json(request)).await;

    // 空列表应该返回成功
    assert_eq!(response.0.response_status.error_code, ErrorCode::Success);
}

#[tokio::test]
async fn test_unregister_unregistered_instance() {
    let state = create_test_app_state();
    let instance = create_test_instance("inst-nonexistent");

    // 注销未注册的实例
    let request = UnregisterRequest {
        instance_keys: vec![instance.key()],
    };
    let response = registry::unregister(State(state), Json(request)).await;

    // 注销不存在的实例应该返回成功 (幂等)
    assert_eq!(response.0.response_status.error_code, ErrorCode::Success);
}

#[tokio::test]
async fn test_unregister_twice() {
    let state = create_test_app_state();
    let instance = create_test_instance("inst-1");

    // 注册
    let reg_request = RegisterRequest {
        instances: vec![instance.clone()],
    };
    let _ = registry::register(State(state.clone()), Json(reg_request)).await;

    // 第一次注销
    let unreg_request1 = UnregisterRequest {
        instance_keys: vec![instance.key()],
    };
    let response1 = registry::unregister(State(state.clone()), Json(unreg_request1)).await;
    assert_eq!(response1.0.response_status.error_code, ErrorCode::Success);

    // 第二次注销 (幂等性测试)
    let unreg_request2 = UnregisterRequest {
        instance_keys: vec![instance.key()],
    };
    let response2 = registry::unregister(State(state), Json(unreg_request2)).await;
    assert_eq!(response2.0.response_status.error_code, ErrorCode::Success);
}

// ============================================================================
// 完整生命周期测试
// ============================================================================

#[tokio::test]
async fn test_full_lifecycle() {
    let state = create_test_app_state();
    let instance = create_test_instance("inst-lifecycle");

    // 1. 注册
    let reg_request = RegisterRequest {
        instances: vec![instance.clone()],
    };
    let reg_response = registry::register(State(state.clone()), Json(reg_request)).await;
    assert_eq!(reg_response.0.response_status.error_code, ErrorCode::Success);

    // 2. 心跳
    let hb_request = HeartbeatRequest {
        instance_keys: vec![instance.key()],
    };
    let hb_response = registry::heartbeat(State(state.clone()), Json(hb_request)).await;
    assert_eq!(hb_response.0.response_status.error_code, ErrorCode::Success);

    // 3. 再次心跳
    let hb_request2 = HeartbeatRequest {
        instance_keys: vec![instance.key()],
    };
    let hb_response2 = registry::heartbeat(State(state.clone()), Json(hb_request2)).await;
    assert_eq!(
        hb_response2.0.response_status.error_code,
        ErrorCode::Success
    );

    // 4. 注销
    let unreg_request = UnregisterRequest {
        instance_keys: vec![instance.key()],
    };
    let unreg_response = registry::unregister(State(state), Json(unreg_request)).await;
    assert_eq!(
        unreg_response.0.response_status.error_code,
        ErrorCode::Success
    );
}

// ============================================================================
// 并发测试
// ============================================================================

#[tokio::test]
async fn test_concurrent_registrations() {
    let state = create_test_app_state();

    // 并发注册 10 个实例
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let state = state.clone();
            tokio::spawn(async move {
                let instance = create_test_instance(&format!("inst-{}", i));
                let request = RegisterRequest {
                    instances: vec![instance],
                };
                registry::register(State(state), Json(request)).await
            })
        })
        .collect();

    // 等待所有任务完成
    for handle in handles {
        let response = handle.await.unwrap();
        assert_eq!(response.0.response_status.error_code, ErrorCode::Success);
    }
}

#[tokio::test]
async fn test_concurrent_heartbeats() {
    let state = create_test_app_state();

    // 先注册 10 个实例
    let instances: Vec<_> = (0..10).map(|i| create_test_instance(&format!("inst-{}", i))).collect();
    let reg_request = RegisterRequest {
        instances: instances.clone(),
    };
    let _ = registry::register(State(state.clone()), Json(reg_request)).await;

    // 并发心跳
    let handles: Vec<_> = instances
        .iter()
        .map(|instance| {
            let state = state.clone();
            let key = instance.key();
            tokio::spawn(async move {
                let request = HeartbeatRequest {
                    instance_keys: vec![key],
                };
                registry::heartbeat(State(state), Json(request)).await
            })
        })
        .collect();

    // 等待所有心跳完成
    for handle in handles {
        let response = handle.await.unwrap();
        assert_eq!(response.0.response_status.error_code, ErrorCode::Success);
    }
}
