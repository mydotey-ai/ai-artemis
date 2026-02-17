//! RegistryServiceImpl 核心服务单元测试
//!
//! 测试覆盖:
//! - register: 服务注册,单实例/多实例注册
//! - heartbeat: 心跳续约,成功/失败场景
//! - unregister: 服务注销,清理租约和缓存
//! - register_from_replication: 复制注册,不触发二次复制
//! - heartbeat_from_replication: 复制心跳
//! - unregister_from_replication: 复制注销
//! - batch_register: 批量注册
//! - batch_heartbeat: 批量心跳
//! - batch_unregister: 批量注销
//! - get_services_delta: 增量同步
//! - sync_full_data: 全量同步
//! - get_all_services: 获取所有服务
//! - get_instances_by_group: 按分组获取实例

use artemis_core::model::{
    BatchHeartbeatRequest, BatchRegisterRequest, BatchUnregisterRequest, ErrorCode,
    HeartbeatRequest, Instance, InstanceKey, InstanceStatus, RegisterRequest,
    ReplicateHeartbeatRequest, ReplicateRegisterRequest, ReplicateUnregisterRequest,
    ServicesDeltaRequest, SyncFullDataRequest, UnregisterRequest,
};
use artemis_core::traits::RegistryService;
use artemis_server::{
    RegistryServiceImpl, cache::VersionedCacheManager, change::InstanceChangeManager,
    lease::LeaseManager, registry::RegistryRepository,
};
use std::sync::Arc;
use std::time::Duration;

/// 创建测试用的 RegistryServiceImpl
fn create_test_registry_service() -> (RegistryServiceImpl, RegistryRepository) {
    let repository = RegistryRepository::new();
    let lease_manager = Arc::new(LeaseManager::new(Duration::from_secs(30)));
    let cache = Arc::new(VersionedCacheManager::new());
    let change_manager = Arc::new(InstanceChangeManager::new());

    let service = RegistryServiceImpl::new(
        repository.clone(),
        lease_manager,
        cache,
        change_manager,
        None, // No replication manager in tests
    );

    (service, repository)
}

/// 创建测试实例
fn create_test_instance(service_id: &str, instance_id: &str) -> Instance {
    Instance {
        region_id: "test-region".to_string(),
        zone_id: "test-zone".to_string(),
        group_id: None,
        service_id: service_id.to_string(),
        instance_id: instance_id.to_string(),
        machine_name: None,
        ip: "192.168.1.100".to_string(),
        port: 8080,
        protocol: None,
        url: "http://192.168.1.100:8080".to_string(),
        health_check_url: None,
        status: InstanceStatus::Up,
        metadata: None,
    }
}

/// 创建实例 key
fn create_instance_key(service_id: &str, instance_id: &str) -> InstanceKey {
    InstanceKey {
        region_id: "test-region".to_string(),
        zone_id: "test-zone".to_string(),
        group_id: String::new(),
        service_id: service_id.to_string(),
        instance_id: instance_id.to_string(),
    }
}

// ===== register 测试 =====

#[tokio::test]
async fn test_register_single_instance() {
    let (service, repo) = create_test_registry_service();

    let instance = create_test_instance("my-service", "inst-1");
    let request = RegisterRequest { instances: vec![instance] };

    let response = service.register(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    assert!(response.failed_instances.is_none());
    assert_eq!(repo.count(), 1);
}

#[tokio::test]
async fn test_register_multiple_instances() {
    let (service, repo) = create_test_registry_service();

    let instances = vec![
        create_test_instance("service-1", "inst-1"),
        create_test_instance("service-1", "inst-2"),
        create_test_instance("service-2", "inst-1"),
    ];

    let request = RegisterRequest { instances };
    let response = service.register(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    assert_eq!(repo.count(), 3);
}

#[tokio::test]
async fn test_register_updates_cache() {
    let (service, _repo) = create_test_registry_service();

    let instance = create_test_instance("my-service", "inst-1");
    let request = RegisterRequest { instances: vec![instance] };

    service.register(request).await;

    // 验证缓存已更新 (通过 get_all_services)
    let all_services = service.get_all_services().await;
    assert_eq!(all_services.services.len(), 1);
    assert_eq!(all_services.services[0].service_id, "my-service");
}

// ===== heartbeat 测试 =====

#[tokio::test]
async fn test_heartbeat_success() {
    let (service, _repo) = create_test_registry_service();

    // 先注册
    let instance = create_test_instance("my-service", "inst-1");
    let key = instance.key();
    service.register(RegisterRequest { instances: vec![instance] }).await;

    // 心跳
    let request = HeartbeatRequest { instance_keys: vec![key] };
    let response = service.heartbeat(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    assert!(response.failed_instance_keys.is_none());
}

#[tokio::test]
async fn test_heartbeat_non_existent_instance() {
    let (service, _repo) = create_test_registry_service();

    let key = create_instance_key("non-existent", "inst-1");
    let request = HeartbeatRequest { instance_keys: vec![key.clone()] };

    let response = service.heartbeat(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::BadRequest);
    assert!(response.failed_instance_keys.is_some());
    assert_eq!(response.failed_instance_keys.unwrap().len(), 1);
}

#[tokio::test]
async fn test_heartbeat_multiple_instances() {
    let (service, _repo) = create_test_registry_service();

    // 注册 2 个实例
    let inst1 = create_test_instance("my-service", "inst-1");
    let inst2 = create_test_instance("my-service", "inst-2");
    let key1 = inst1.key();
    let key2 = inst2.key();

    service.register(RegisterRequest { instances: vec![inst1, inst2] }).await;

    // 心跳
    let request = HeartbeatRequest { instance_keys: vec![key1, key2] };
    let response = service.heartbeat(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    assert!(response.failed_instance_keys.is_none());
}

#[tokio::test]
async fn test_heartbeat_partial_failure() {
    let (service, _repo) = create_test_registry_service();

    // 只注册 1 个实例
    let inst1 = create_test_instance("my-service", "inst-1");
    let key1 = inst1.key();
    service.register(RegisterRequest { instances: vec![inst1] }).await;

    // 心跳包含 1 个存在的和 1 个不存在的
    let key2 = create_instance_key("my-service", "inst-2");
    let request = HeartbeatRequest { instance_keys: vec![key1, key2] };

    let response = service.heartbeat(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::BadRequest);
    assert!(response.failed_instance_keys.is_some());
    assert_eq!(response.failed_instance_keys.unwrap().len(), 1);
}

// ===== unregister 测试 =====

#[tokio::test]
async fn test_unregister_single_instance() {
    let (service, repo) = create_test_registry_service();

    // 注册
    let instance = create_test_instance("my-service", "inst-1");
    let key = instance.key();
    service.register(RegisterRequest { instances: vec![instance] }).await;

    assert_eq!(repo.count(), 1);

    // 注销
    let request = UnregisterRequest { instance_keys: vec![key] };
    let response = service.unregister(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    assert_eq!(repo.count(), 0);
}

#[tokio::test]
async fn test_unregister_multiple_instances() {
    let (service, repo) = create_test_registry_service();

    // 注册 3 个实例
    let instances = vec![
        create_test_instance("my-service", "inst-1"),
        create_test_instance("my-service", "inst-2"),
        create_test_instance("my-service", "inst-3"),
    ];
    let keys: Vec<_> = instances.iter().map(|i| i.key()).collect();

    service.register(RegisterRequest { instances }).await;
    assert_eq!(repo.count(), 3);

    // 注销全部
    let request = UnregisterRequest { instance_keys: keys };
    service.unregister(request).await;

    assert_eq!(repo.count(), 0);
}

#[tokio::test]
async fn test_unregister_non_existent_instance() {
    let (service, repo) = create_test_registry_service();

    let key = create_instance_key("non-existent", "inst-1");
    let request = UnregisterRequest { instance_keys: vec![key] };

    let response = service.unregister(request).await;

    // 注销不存在的实例应该成功(幂等操作)
    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    assert_eq!(repo.count(), 0);
}

// ===== 复制方法测试 =====

#[tokio::test]
async fn test_register_from_replication() {
    let (service, repo) = create_test_registry_service();

    let instance = create_test_instance("my-service", "inst-1");
    let request = ReplicateRegisterRequest { instances: vec![instance] };

    let response = service.register_from_replication(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    assert!(response.failed_instances.is_none());
    assert_eq!(repo.count(), 1);
}

#[tokio::test]
async fn test_heartbeat_from_replication() {
    let (service, _repo) = create_test_registry_service();

    // 先注册
    let instance = create_test_instance("my-service", "inst-1");
    let key = instance.key();
    service.register(RegisterRequest { instances: vec![instance] }).await;

    // 复制心跳
    let request = ReplicateHeartbeatRequest { instance_keys: vec![key] };
    let response = service.heartbeat_from_replication(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    assert!(response.failed_instance_keys.is_none());
}

#[tokio::test]
async fn test_unregister_from_replication() {
    let (service, repo) = create_test_registry_service();

    // 先注册
    let instance = create_test_instance("my-service", "inst-1");
    let key = instance.key();
    service.register(RegisterRequest { instances: vec![instance] }).await;

    // 复制注销
    let request = ReplicateUnregisterRequest { instance_keys: vec![key] };
    let response = service.unregister_from_replication(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    assert_eq!(repo.count(), 0);
}

// ===== 批量操作测试 =====

#[tokio::test]
async fn test_batch_register() {
    let (service, repo) = create_test_registry_service();

    let instances = vec![
        create_test_instance("service-1", "inst-1"),
        create_test_instance("service-1", "inst-2"),
        create_test_instance("service-2", "inst-1"),
    ];

    let request = BatchRegisterRequest { instances };
    let response = service.batch_register(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    assert!(response.failed_instances.is_none());
    assert_eq!(repo.count(), 3);
}

#[tokio::test]
async fn test_batch_heartbeat() {
    let (service, _repo) = create_test_registry_service();

    // 先注册
    let instances = vec![
        create_test_instance("my-service", "inst-1"),
        create_test_instance("my-service", "inst-2"),
    ];
    let keys: Vec<_> = instances.iter().map(|i| i.key()).collect();

    service.register(RegisterRequest { instances }).await;

    // 批量心跳
    let request = BatchHeartbeatRequest { instance_keys: keys };
    let response = service.batch_heartbeat(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    assert!(response.failed_instance_keys.is_none());
}

#[tokio::test]
async fn test_batch_heartbeat_partial_failure() {
    let (service, _repo) = create_test_registry_service();

    // 只注册 1 个
    let inst1 = create_test_instance("my-service", "inst-1");
    let key1 = inst1.key();
    service.register(RegisterRequest { instances: vec![inst1] }).await;

    // 批量心跳包含不存在的实例
    let key2 = create_instance_key("my-service", "inst-2");
    let request = BatchHeartbeatRequest { instance_keys: vec![key1, key2] };

    let response = service.batch_heartbeat(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::BadRequest);
    assert!(response.failed_instance_keys.is_some());
    assert_eq!(response.failed_instance_keys.unwrap().len(), 1);
}

#[tokio::test]
async fn test_batch_unregister() {
    let (service, repo) = create_test_registry_service();

    // 先注册
    let instances = vec![
        create_test_instance("my-service", "inst-1"),
        create_test_instance("my-service", "inst-2"),
        create_test_instance("my-service", "inst-3"),
    ];
    let keys: Vec<_> = instances.iter().map(|i| i.key()).collect();

    service.register(RegisterRequest { instances }).await;
    assert_eq!(repo.count(), 3);

    // 批量注销
    let request = BatchUnregisterRequest { instance_keys: keys };
    let response = service.batch_unregister(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    assert!(response.failed_instance_keys.is_none());
    assert_eq!(repo.count(), 0);
}

// ===== 同步方法测试 =====

#[tokio::test]
async fn test_get_services_delta() {
    let (service, _repo) = create_test_registry_service();

    // 注册一些实例
    let instances = vec![
        create_test_instance("service-1", "inst-1"),
        create_test_instance("service-2", "inst-1"),
    ];
    service.register(RegisterRequest { instances }).await;

    // 增量同步
    let request = ServicesDeltaRequest {
        region_id: "test-region".to_string(),
        zone_id: "test-zone".to_string(),
        since_timestamp: 0,
    };

    let response = service.get_services_delta(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    assert_eq!(response.services.len(), 2);
    assert!(response.current_timestamp > 0);
}

#[tokio::test]
async fn test_sync_full_data() {
    let (service, _repo) = create_test_registry_service();

    // 注册一些实例
    let instances = vec![
        create_test_instance("service-1", "inst-1"),
        create_test_instance("service-2", "inst-1"),
        create_test_instance("service-2", "inst-2"),
    ];
    service.register(RegisterRequest { instances }).await;

    // 全量同步
    let request = SyncFullDataRequest {
        region_id: "test-region".to_string(),
        zone_id: Some("test-zone".to_string()),
    };

    let response = service.sync_full_data(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    assert_eq!(response.services.len(), 2);
    assert!(response.sync_timestamp > 0);
}

#[tokio::test]
async fn test_get_all_services() {
    let (service, _repo) = create_test_registry_service();

    // 注册不同服务的实例
    let instances = vec![
        create_test_instance("service-1", "inst-1"),
        create_test_instance("service-1", "inst-2"),
        create_test_instance("service-2", "inst-1"),
    ];
    service.register(RegisterRequest { instances }).await;

    // 获取所有服务
    let response = service.get_all_services().await;

    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    assert_eq!(response.services.len(), 2);

    // 验证服务实例数量
    let service_1 = response.services.iter().find(|s| s.service_id == "service-1");
    let service_2 = response.services.iter().find(|s| s.service_id == "service-2");

    assert!(service_1.is_some());
    assert!(service_2.is_some());
    assert_eq!(service_1.unwrap().instances.len(), 2);
    assert_eq!(service_2.unwrap().instances.len(), 1);
}

// ===== 分组查询测试 =====

#[tokio::test]
async fn test_get_instances_by_group() {
    let (service, _repo) = create_test_registry_service();

    // 注册带分组的实例
    let mut inst1 = create_test_instance("my-service", "inst-1");
    inst1.group_id = Some("group-1".to_string());

    let mut inst2 = create_test_instance("my-service", "inst-2");
    inst2.group_id = Some("group-2".to_string());

    service.register(RegisterRequest { instances: vec![inst1, inst2] }).await;

    // 查询 group-1 的实例
    let instances = service.get_instances_by_group("my-service", "group-1", None);

    assert_eq!(instances.len(), 1);
    assert_eq!(instances[0].instance_id, "inst-1");
}

#[tokio::test]
async fn test_get_instances_by_group_empty() {
    let (service, _repo) = create_test_registry_service();

    // 注册实例但不指定分组
    let instance = create_test_instance("my-service", "inst-1");
    service.register(RegisterRequest { instances: vec![instance] }).await;

    // 查询不存在的分组
    let instances = service.get_instances_by_group("my-service", "non-existent-group", None);

    assert_eq!(instances.len(), 0);
}

// ===== 缓存一致性测试 =====

#[tokio::test]
async fn test_cache_consistency_after_register() {
    let (service, _repo) = create_test_registry_service();

    let instance = create_test_instance("my-service", "inst-1");
    service.register(RegisterRequest { instances: vec![instance] }).await;

    // 验证通过 get_all_services 能获取到注册的实例
    let all_services = service.get_all_services().await;
    assert_eq!(all_services.services.len(), 1);
    assert_eq!(all_services.services[0].instances.len(), 1);
}

#[tokio::test]
async fn test_cache_consistency_after_unregister() {
    let (service, _repo) = create_test_registry_service();

    // 注册
    let instance = create_test_instance("my-service", "inst-1");
    let key = instance.key();
    service.register(RegisterRequest { instances: vec![instance] }).await;

    // 注销
    service.unregister(UnregisterRequest { instance_keys: vec![key] }).await;

    // 验证缓存已清空
    let all_services = service.get_all_services().await;
    assert_eq!(all_services.services.len(), 0);
}

#[tokio::test]
async fn test_cache_consistency_with_multiple_services() {
    let (service, _repo) = create_test_registry_service();

    // 注册多个服务的实例
    let instances = vec![
        create_test_instance("service-1", "inst-1"),
        create_test_instance("service-1", "inst-2"),
        create_test_instance("service-2", "inst-1"),
    ];
    service.register(RegisterRequest { instances }).await;

    // 注销 service-1 的所有实例
    let key1 = create_instance_key("service-1", "inst-1");
    let key2 = create_instance_key("service-1", "inst-2");
    service.unregister(UnregisterRequest { instance_keys: vec![key1, key2] }).await;

    // 验证 service-1 已被删除,但 service-2 仍存在
    let all_services = service.get_all_services().await;
    assert_eq!(all_services.services.len(), 1);
    assert_eq!(all_services.services[0].service_id, "service-2");
}
