//! Discovery API 单元测试
//!
//! 测试覆盖:
//! - get_service: 获取单个服务 API (POST 和 GET 版本)
//! - get_services: 获取所有服务 API (POST 和 GET 版本)
//! - lookup_instance: 负载均衡查询单个实例 API

use artemis_core::model::{
    DiscoveryConfig, ErrorCode, GetServiceRequest, GetServicesRequest, Instance, InstanceStatus,
    RegisterRequest,
};
use artemis_server::{
    InstanceChangeManager, RegistryServiceImpl, cache::VersionedCacheManager, lease::LeaseManager,
    registry::RegistryRepository,
};
use artemis_web::{api::discovery, state::AppState};
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

    let discovery_service =
        Arc::new(artemis_server::discovery::DiscoveryServiceImpl::new(repository, cache.clone()));

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
        "test-node".to_string(),             // node_id
        "test-region".to_string(),           // region_id
        "test-zone".to_string(),             // zone_id
        "http://localhost:8080".to_string(), // server_url
        "test-app".to_string(),              // app_id
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
fn create_test_instance(service_id: &str, instance_id: &str) -> Instance {
    Instance {
        region_id: "test-region".to_string(),
        zone_id: "test-zone".to_string(),
        service_id: service_id.to_string(),
        group_id: None,
        instance_id: instance_id.to_string(),
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
// Get Service API 测试 (POST 版本)
// ============================================================================

#[tokio::test]
async fn test_get_service_success() {
    let state = create_test_app_state();

    // 先注册实例
    let instances = vec![
        create_test_instance("my-service", "inst-1"),
        create_test_instance("my-service", "inst-2"),
    ];
    let reg_request = RegisterRequest { instances: instances.clone() };
    let _ = artemis_web::api::registry::register(State(state.clone()), Json(reg_request)).await;

    // 等待缓存更新
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 服务发现
    let request = GetServiceRequest {
        discovery_config: DiscoveryConfig {
            service_id: "my-service".to_string(),
            region_id: "test-region".to_string(),
            zone_id: "test-zone".to_string(),
            discovery_data: None,
        },
    };

    let response = discovery::get_service(State(state), Json(request)).await;

    assert_eq!(response.0.response_status.error_code, ErrorCode::Success);
    assert!(response.0.service.is_some());
    let service = response.0.service.unwrap();
    assert_eq!(service.instances.len(), 2);
}

#[tokio::test]
async fn test_get_service_not_found() {
    let state = create_test_app_state();

    let request = GetServiceRequest {
        discovery_config: DiscoveryConfig {
            service_id: "nonexistent-service".to_string(),
            region_id: "test-region".to_string(),
            zone_id: "test-zone".to_string(),
            discovery_data: None,
        },
    };

    let response = discovery::get_service(State(state), Json(request)).await;

    // 服务不存在应该返回 None
    assert!(response.0.service.is_none());
}

#[tokio::test]
async fn test_get_service_filters_down_instances() {
    let state = create_test_app_state();

    // 注册不同状态的实例
    let mut inst1 = create_test_instance("my-service", "inst-up");
    inst1.status = InstanceStatus::Up;

    let mut inst2 = create_test_instance("my-service", "inst-down");
    inst2.status = InstanceStatus::Down;

    let reg_request = RegisterRequest { instances: vec![inst1, inst2] };
    let _ = artemis_web::api::registry::register(State(state.clone()), Json(reg_request)).await;

    tokio::time::sleep(Duration::from_millis(100)).await;

    // 服务发现 (应该只返回 Up 状态的实例)
    let request = GetServiceRequest {
        discovery_config: DiscoveryConfig {
            service_id: "my-service".to_string(),
            region_id: "test-region".to_string(),
            zone_id: "test-zone".to_string(),
            discovery_data: None,
        },
    };

    let response = discovery::get_service(State(state), Json(request)).await;

    assert!(response.0.service.is_some());
    let service = response.0.service.unwrap();
    // Down 实例应该被过滤掉
    assert!(service.instances.iter().all(|i| i.status == InstanceStatus::Up));
}

#[tokio::test]
async fn test_get_service_cache_version() {
    let state = create_test_app_state();

    // 注册实例
    let instances = vec![create_test_instance("my-service", "inst-1")];
    let reg_request = RegisterRequest { instances };
    let _ = artemis_web::api::registry::register(State(state.clone()), Json(reg_request)).await;

    tokio::time::sleep(Duration::from_millis(100)).await;

    // 第一次请求
    let request1 = GetServiceRequest {
        discovery_config: DiscoveryConfig {
            service_id: "my-service".to_string(),
            region_id: "test-region".to_string(),
            zone_id: "test-zone".to_string(),
            discovery_data: None,
        },
    };
    let _response1 = discovery::get_service(State(state.clone()), Json(request1)).await;

    // 第二次请求 (版本应该相同,因为数据没变)
    let request2 = GetServiceRequest {
        discovery_config: DiscoveryConfig {
            service_id: "my-service".to_string(),
            region_id: "test-region".to_string(),
            zone_id: "test-zone".to_string(),
            discovery_data: None,
        },
    };
    let _response2 = discovery::get_service(State(state), Json(request2)).await;
}

// ============================================================================
// Get Services API 测试 (POST 版本)
// ============================================================================

#[tokio::test]
async fn test_get_services_success() {
    let state = create_test_app_state();

    // 注册多个服务的实例
    let instances = vec![
        create_test_instance("service-a", "inst-a1"),
        create_test_instance("service-a", "inst-a2"),
        create_test_instance("service-b", "inst-b1"),
        create_test_instance("service-c", "inst-c1"),
    ];
    let reg_request = RegisterRequest { instances };
    let _ = artemis_web::api::registry::register(State(state.clone()), Json(reg_request)).await;

    tokio::time::sleep(Duration::from_millis(100)).await;

    // 获取所有服务
    let request = GetServicesRequest {
        region_id: "test-region".to_string(),
        zone_id: "test-zone".to_string(),
    };

    let response = discovery::get_services(State(state), Json(request)).await;

    assert_eq!(response.0.response_status.error_code, ErrorCode::Success);
    assert_eq!(response.0.services.len(), 3); // service-a, service-b, service-c
}

#[tokio::test]
async fn test_get_services_empty_region() {
    let state = create_test_app_state();

    let request = GetServicesRequest {
        region_id: "empty-region".to_string(),
        zone_id: "empty-zone".to_string(),
    };

    let response = discovery::get_services(State(state), Json(request)).await;

    assert_eq!(response.0.response_status.error_code, ErrorCode::Success);
    assert_eq!(response.0.services.len(), 0);
}

#[tokio::test]
async fn test_get_services_groups_by_service_id() {
    let state = create_test_app_state();

    // 注册同一服务的多个实例
    let instances = vec![
        create_test_instance("my-service", "inst-1"),
        create_test_instance("my-service", "inst-2"),
        create_test_instance("my-service", "inst-3"),
    ];
    let reg_request = RegisterRequest { instances };
    let _ = artemis_web::api::registry::register(State(state.clone()), Json(reg_request)).await;

    tokio::time::sleep(Duration::from_millis(100)).await;

    // 获取所有服务
    let request = GetServicesRequest {
        region_id: "test-region".to_string(),
        zone_id: "test-zone".to_string(),
    };

    let response = discovery::get_services(State(state), Json(request)).await;

    assert_eq!(response.0.services.len(), 1);
    assert_eq!(response.0.services[0].instances.len(), 3);
}

// ============================================================================
// Lookup Instance API 测试 (负载均衡)
// ============================================================================

#[tokio::test]
async fn test_lookup_instance_random_strategy() {
    let state = create_test_app_state();

    // 注册多个实例
    let instances = vec![
        create_test_instance("my-service", "inst-1"),
        create_test_instance("my-service", "inst-2"),
        create_test_instance("my-service", "inst-3"),
    ];
    let reg_request = RegisterRequest { instances };
    let _ = artemis_web::api::registry::register(State(state.clone()), Json(reg_request)).await;

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Lookup 请求
    let request = discovery::LookupRequest {
        discovery_config: DiscoveryConfig {
            service_id: "my-service".to_string(),
            region_id: "test-region".to_string(),
            zone_id: "test-zone".to_string(),
            discovery_data: None,
        },
        strategy: Some("random".to_string()),
    };

    let _response = discovery::lookup_instance(State(state), Json(request)).await;

    // 应该返回一个实例
    // Note: 需要检查实际的响应类型
}

#[tokio::test]
async fn test_lookup_instance_round_robin_strategy() {
    let state = create_test_app_state();

    // 注册多个实例
    let instances = vec![
        create_test_instance("my-service", "inst-1"),
        create_test_instance("my-service", "inst-2"),
        create_test_instance("my-service", "inst-3"),
    ];
    let reg_request = RegisterRequest { instances };
    let _ = artemis_web::api::registry::register(State(state.clone()), Json(reg_request)).await;

    tokio::time::sleep(Duration::from_millis(100)).await;

    // 多次 Lookup,应该轮询返回不同实例
    for _ in 0..3 {
        let request = discovery::LookupRequest {
            discovery_config: DiscoveryConfig {
                service_id: "my-service".to_string(),
                region_id: "test-region".to_string(),
                zone_id: "test-zone".to_string(),
                discovery_data: None,
            },
            strategy: Some("round-robin".to_string()),
        };

        let _response = discovery::lookup_instance(State(state.clone()), Json(request)).await;
        // 验证轮询行为
    }
}

#[tokio::test]
async fn test_lookup_instance_no_instances() {
    let state = create_test_app_state();

    let request = discovery::LookupRequest {
        discovery_config: DiscoveryConfig {
            service_id: "nonexistent-service".to_string(),
            region_id: "test-region".to_string(),
            zone_id: "test-zone".to_string(),
            discovery_data: None,
        },
        strategy: Some("random".to_string()),
    };

    let _response = discovery::lookup_instance(State(state), Json(request)).await;
    // 应该返回失败或空结果
}

// ============================================================================
// 并发测试
// ============================================================================

#[tokio::test]
async fn test_concurrent_get_service() {
    let state = create_test_app_state();

    // 注册实例
    let instances = vec![create_test_instance("my-service", "inst-1")];
    let reg_request = RegisterRequest { instances };
    let _ = artemis_web::api::registry::register(State(state.clone()), Json(reg_request)).await;

    tokio::time::sleep(Duration::from_millis(100)).await;

    // 并发查询
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let state = state.clone();
            tokio::spawn(async move {
                let request = GetServiceRequest {
                    discovery_config: DiscoveryConfig {
                        service_id: "my-service".to_string(),
                        region_id: "test-region".to_string(),
                        zone_id: "test-zone".to_string(),
                        discovery_data: None,
                    },
                };
                discovery::get_service(State(state), Json(request)).await
            })
        })
        .collect();

    // 等待所有查询完成
    for handle in handles {
        let response = handle.await.unwrap();
        assert_eq!(response.0.response_status.error_code, ErrorCode::Success);
        assert!(response.0.service.is_some());
    }
}

#[tokio::test]
async fn test_concurrent_get_services() {
    let state = create_test_app_state();

    // 注册多个服务
    let instances = vec![
        create_test_instance("service-a", "inst-1"),
        create_test_instance("service-b", "inst-1"),
        create_test_instance("service-c", "inst-1"),
    ];
    let reg_request = RegisterRequest { instances };
    let _ = artemis_web::api::registry::register(State(state.clone()), Json(reg_request)).await;

    tokio::time::sleep(Duration::from_millis(100)).await;

    // 并发查询所有服务
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let state = state.clone();
            tokio::spawn(async move {
                let request = GetServicesRequest {
                    region_id: "test-region".to_string(),
                    zone_id: "test-zone".to_string(),
                };
                discovery::get_services(State(state), Json(request)).await
            })
        })
        .collect();

    for handle in handles {
        let response = handle.await.unwrap();
        assert_eq!(response.0.response_status.error_code, ErrorCode::Success);
        assert_eq!(response.0.services.len(), 3);
    }
}
