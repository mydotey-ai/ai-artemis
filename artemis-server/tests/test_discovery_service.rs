//! DiscoveryServiceImpl 核心服务单元测试
//!
//! 测试覆盖:
//! - get_service: 服务发现,缓存命中/未命中
//! - get_services: 获取所有服务
//! - get_services_delta: 增量查询,版本比较
//! - refresh_cache: 缓存刷新
//! - add_filter: 过滤器链
//! - 过滤器应用: Status 过滤器

use artemis_core::model::{
    DiscoveryConfig, ErrorCode, GetServiceRequest, GetServicesDeltaRequest, GetServicesRequest,
    Instance, InstanceStatus,
};
use artemis_core::traits::DiscoveryService;
use artemis_server::{
    cache::VersionedCacheManager, discovery::DiscoveryServiceImpl, registry::RegistryRepository,
};
use std::sync::Arc;

/// 创建测试用的 DiscoveryServiceImpl
fn create_test_discovery_service() -> (DiscoveryServiceImpl, RegistryRepository) {
    let repository = RegistryRepository::new();
    let cache = Arc::new(VersionedCacheManager::new());

    let service = DiscoveryServiceImpl::new(repository.clone(), cache);

    (service, repository)
}

/// 创建测试实例
fn create_test_instance(service_id: &str, instance_id: &str, status: InstanceStatus) -> Instance {
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
        status,
        metadata: None,
    }
}

/// 创建 DiscoveryConfig
fn create_discovery_config(service_id: &str) -> DiscoveryConfig {
    DiscoveryConfig {
        service_id: service_id.to_string(),
        region_id: "test-region".to_string(),
        zone_id: "test-zone".to_string(),
        discovery_data: None,
    }
}

// ===== get_service 测试 =====

#[tokio::test]
async fn test_get_service_success() {
    let (service, repo) = create_test_discovery_service();

    // 注册实例
    let instance = create_test_instance("my-service", "inst-1", InstanceStatus::Up);
    repo.register(instance);

    // 查询服务
    let request = GetServiceRequest { discovery_config: create_discovery_config("my-service") };

    let response = service.get_service(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    assert!(response.service.is_some());

    let svc = response.service.unwrap();
    assert_eq!(svc.service_id, "my-service");
    assert_eq!(svc.instances.len(), 1);
}

#[tokio::test]
async fn test_get_service_not_found() {
    let (service, _repo) = create_test_discovery_service();

    // 查询不存在的服务
    let request = GetServiceRequest { discovery_config: create_discovery_config("non-existent") };

    let response = service.get_service(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::BadRequest);
    assert!(response.service.is_none());
}

#[tokio::test]
async fn test_get_service_case_insensitive() {
    let (service, repo) = create_test_discovery_service();

    // 注册小写服务
    let instance = create_test_instance("my-service", "inst-1", InstanceStatus::Up);
    repo.register(instance);

    // 用大写查询
    let request = GetServiceRequest { discovery_config: create_discovery_config("MY-SERVICE") };

    let response = service.get_service(request).await;

    // 应该能找到 (service_id 被转为小写)
    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    assert!(response.service.is_some());
}

#[tokio::test]
async fn test_get_service_multiple_instances() {
    let (service, repo) = create_test_discovery_service();

    // 注册同一服务的多个实例
    repo.register(create_test_instance("my-service", "inst-1", InstanceStatus::Up));
    repo.register(create_test_instance("my-service", "inst-2", InstanceStatus::Up));
    repo.register(create_test_instance("my-service", "inst-3", InstanceStatus::Up));

    let request = GetServiceRequest { discovery_config: create_discovery_config("my-service") };

    let response = service.get_service(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    let svc = response.service.unwrap();
    assert_eq!(svc.instances.len(), 3);
}

#[tokio::test]
async fn test_get_service_filters_down_instances() {
    let (service, repo) = create_test_discovery_service();

    // 注册 UP 和 DOWN 实例
    repo.register(create_test_instance("my-service", "inst-1", InstanceStatus::Up));
    repo.register(create_test_instance("my-service", "inst-2", InstanceStatus::Down));
    repo.register(create_test_instance("my-service", "inst-3", InstanceStatus::Up));

    let request = GetServiceRequest { discovery_config: create_discovery_config("my-service") };

    let response = service.get_service(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    let svc = response.service.unwrap();

    // StatusFilter 应该过滤掉 DOWN 实例
    assert_eq!(svc.instances.len(), 2);
    for inst in &svc.instances {
        assert_eq!(inst.status, InstanceStatus::Up);
    }
}

// ===== get_services 测试 =====

#[tokio::test]
async fn test_get_services_empty() {
    let (service, _repo) = create_test_discovery_service();

    let request = GetServicesRequest {
        region_id: "test-region".to_string(),
        zone_id: "test-zone".to_string(),
    };

    let response = service.get_services(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    assert_eq!(response.services.len(), 0);
}

#[tokio::test]
async fn test_get_services_multiple() {
    let (service, repo) = create_test_discovery_service();

    // 注册多个服务
    repo.register(create_test_instance("service-1", "inst-1", InstanceStatus::Up));
    repo.register(create_test_instance("service-2", "inst-1", InstanceStatus::Up));
    repo.register(create_test_instance("service-3", "inst-1", InstanceStatus::Up));

    // 刷新缓存
    service.refresh_cache();

    let request = GetServicesRequest {
        region_id: "test-region".to_string(),
        zone_id: "test-zone".to_string(),
    };

    let response = service.get_services(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    assert_eq!(response.services.len(), 3);
}

#[tokio::test]
async fn test_get_services_from_cache() {
    let (service, repo) = create_test_discovery_service();

    // 注册并刷新缓存
    repo.register(create_test_instance("service-1", "inst-1", InstanceStatus::Up));
    repo.register(create_test_instance("service-2", "inst-1", InstanceStatus::Up));
    service.refresh_cache();

    // 直接从缓存获取
    let request = GetServicesRequest {
        region_id: "test-region".to_string(),
        zone_id: "test-zone".to_string(),
    };

    let response = service.get_services(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    assert_eq!(response.services.len(), 2);
}

// ===== get_services_delta 测试 =====

#[tokio::test]
async fn test_get_services_delta_all() {
    let (service, repo) = create_test_discovery_service();

    // 注册服务并刷新缓存
    repo.register(create_test_instance("service-1", "inst-1", InstanceStatus::Up));
    repo.register(create_test_instance("service-2", "inst-1", InstanceStatus::Up));
    service.refresh_cache();

    // since_timestamp = 0 应该返回所有服务
    let request = GetServicesDeltaRequest {
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
async fn test_get_services_delta_no_changes() {
    let (service, repo) = create_test_discovery_service();

    // 注册服务并刷新缓存
    repo.register(create_test_instance("service-1", "inst-1", InstanceStatus::Up));
    service.refresh_cache();

    // 获取当前版本
    let request1 = GetServicesDeltaRequest {
        region_id: "test-region".to_string(),
        zone_id: "test-zone".to_string(),
        since_timestamp: 0,
    };
    let response1 = service.get_services_delta(request1).await;
    let version = response1.current_timestamp;

    // 用相同版本再次查询,应该返回空
    let request2 = GetServicesDeltaRequest {
        region_id: "test-region".to_string(),
        zone_id: "test-zone".to_string(),
        since_timestamp: version,
    };
    let response2 = service.get_services_delta(request2).await;

    assert_eq!(response2.response_status.error_code, ErrorCode::Success);
    assert_eq!(response2.services.len(), 0);
    assert_eq!(response2.current_timestamp, version);
}

#[tokio::test]
async fn test_get_services_delta_future_timestamp() {
    let (service, repo) = create_test_discovery_service();

    // 注册服务
    repo.register(create_test_instance("service-1", "inst-1", InstanceStatus::Up));
    service.refresh_cache();

    // 使用未来的 timestamp,应该返回空
    let request = GetServicesDeltaRequest {
        region_id: "test-region".to_string(),
        zone_id: "test-zone".to_string(),
        since_timestamp: i64::MAX,
    };

    let response = service.get_services_delta(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    assert_eq!(response.services.len(), 0);
}

// ===== refresh_cache 测试 =====

#[tokio::test]
async fn test_refresh_cache_single_service() {
    let (service, repo) = create_test_discovery_service();

    // 注册实例
    repo.register(create_test_instance("my-service", "inst-1", InstanceStatus::Up));

    // 刷新缓存
    service.refresh_cache();

    // 验证缓存
    let request = GetServicesRequest {
        region_id: "test-region".to_string(),
        zone_id: "test-zone".to_string(),
    };
    let response = service.get_services(request).await;

    assert_eq!(response.services.len(), 1);
    assert_eq!(response.services[0].service_id, "my-service");
}

#[tokio::test]
async fn test_refresh_cache_multiple_services() {
    let (service, repo) = create_test_discovery_service();

    // 注册多个服务的实例
    repo.register(create_test_instance("service-1", "inst-1", InstanceStatus::Up));
    repo.register(create_test_instance("service-1", "inst-2", InstanceStatus::Up));
    repo.register(create_test_instance("service-2", "inst-1", InstanceStatus::Up));
    repo.register(create_test_instance("service-3", "inst-1", InstanceStatus::Up));

    // 刷新缓存
    service.refresh_cache();

    // 验证缓存
    let request = GetServicesRequest {
        region_id: "test-region".to_string(),
        zone_id: "test-zone".to_string(),
    };
    let response = service.get_services(request).await;

    assert_eq!(response.services.len(), 3);
}

#[tokio::test]
async fn test_refresh_cache_deduplication() {
    let (service, repo) = create_test_discovery_service();

    // 注册同一服务的多个实例 (测试去重)
    repo.register(create_test_instance("my-service", "inst-1", InstanceStatus::Up));
    repo.register(create_test_instance("my-service", "inst-2", InstanceStatus::Up));
    repo.register(create_test_instance("my-service", "inst-3", InstanceStatus::Up));

    // 刷新缓存
    service.refresh_cache();

    // 验证只有一个服务条目
    let request = GetServicesRequest {
        region_id: "test-region".to_string(),
        zone_id: "test-zone".to_string(),
    };
    let response = service.get_services(request).await;

    assert_eq!(response.services.len(), 1);
    assert_eq!(response.services[0].instances.len(), 3);
}

// ===== 缓存一致性测试 =====

#[tokio::test]
async fn test_cache_hit_after_first_query() {
    let (service, repo) = create_test_discovery_service();

    repo.register(create_test_instance("my-service", "inst-1", InstanceStatus::Up));

    let config = create_discovery_config("my-service");

    // 第一次查询 (缓存未命中)
    let request1 = GetServiceRequest { discovery_config: config.clone() };
    let response1 = service.get_service(request1).await;
    assert!(response1.service.is_some());

    // 第二次查询 (应该从缓存获取)
    let request2 = GetServiceRequest { discovery_config: config };
    let response2 = service.get_service(request2).await;
    assert!(response2.service.is_some());
}

#[tokio::test]
async fn test_cache_returns_stale_data_until_refresh() {
    let (service, repo) = create_test_discovery_service();

    // 初始状态: 1 个实例
    repo.register(create_test_instance("my-service", "inst-1", InstanceStatus::Up));

    let config = create_discovery_config("my-service");
    let request1 = GetServiceRequest { discovery_config: config.clone() };
    let response1 = service.get_service(request1).await;
    assert_eq!(response1.service.unwrap().instances.len(), 1);

    // 添加更多实例
    repo.register(create_test_instance("my-service", "inst-2", InstanceStatus::Up));

    // 再次查询,因为缓存已存在,会返回缓存数据 (仍然是 1 个实例)
    let request2 = GetServiceRequest { discovery_config: config.clone() };
    let response2 = service.get_service(request2).await;

    // 缓存未更新,仍然是 1 个实例
    assert_eq!(response2.service.unwrap().instances.len(), 1);

    // 刷新缓存后,应该看到 2 个实例
    service.refresh_cache();
    let request3 = GetServiceRequest { discovery_config: config };
    let response3 = service.get_service(request3).await;
    assert_eq!(response3.service.unwrap().instances.len(), 2);
}

// ===== 边界条件测试 =====

#[tokio::test]
async fn test_get_service_empty_instances() {
    let (service, _repo) = create_test_discovery_service();

    // 查询没有实例的服务
    let request = GetServiceRequest { discovery_config: create_discovery_config("empty-service") };

    let response = service.get_service(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::BadRequest);
    assert!(response.service.is_none());
}

#[tokio::test]
async fn test_get_service_all_instances_down() {
    let (service, repo) = create_test_discovery_service();

    // 注册全部 DOWN 的实例
    repo.register(create_test_instance("my-service", "inst-1", InstanceStatus::Down));
    repo.register(create_test_instance("my-service", "inst-2", InstanceStatus::Down));

    let request = GetServiceRequest { discovery_config: create_discovery_config("my-service") };

    let response = service.get_service(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    let svc = response.service.unwrap();

    // StatusFilter 会过滤掉所有 DOWN 实例
    assert_eq!(svc.instances.len(), 0);
}

#[tokio::test]
async fn test_refresh_cache_empty_repository() {
    let (service, _repo) = create_test_discovery_service();

    // 空仓库刷新缓存
    service.refresh_cache();

    // 验证缓存为空
    let request = GetServicesRequest {
        region_id: "test-region".to_string(),
        zone_id: "test-zone".to_string(),
    };
    let response = service.get_services(request).await;

    assert_eq!(response.services.len(), 0);
}

// ===== 性能测试 =====

#[tokio::test]
async fn test_get_service_performance_with_many_instances() {
    let (service, repo) = create_test_discovery_service();

    // 注册 100 个实例
    for i in 0..100 {
        let instance =
            create_test_instance("my-service", &format!("inst-{}", i), InstanceStatus::Up);
        repo.register(instance);
    }

    let request = GetServiceRequest { discovery_config: create_discovery_config("my-service") };

    // 查询应该成功
    let response = service.get_service(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    let svc = response.service.unwrap();
    assert_eq!(svc.instances.len(), 100);
}

#[tokio::test]
async fn test_refresh_cache_with_many_services() {
    let (service, repo) = create_test_discovery_service();

    // 注册 50 个不同的服务
    for i in 0..50 {
        let service_id = format!("service-{}", i);
        repo.register(create_test_instance(&service_id, "inst-1", InstanceStatus::Up));
    }

    // 刷新缓存
    service.refresh_cache();

    // 验证
    let request = GetServicesRequest {
        region_id: "test-region".to_string(),
        zone_id: "test-zone".to_string(),
    };
    let response = service.get_services(request).await;

    assert_eq!(response.services.len(), 50);
}

// ===== 并发测试 =====

#[tokio::test]
async fn test_concurrent_get_service() {
    let (service, repo) = create_test_discovery_service();

    // 注册实例
    repo.register(create_test_instance("my-service", "inst-1", InstanceStatus::Up));

    // 并发查询
    let service_clone1 = service.clone();
    let service_clone2 = service.clone();

    let handle1 = tokio::spawn(async move {
        let request = GetServiceRequest { discovery_config: create_discovery_config("my-service") };
        service_clone1.get_service(request).await
    });

    let handle2 = tokio::spawn(async move {
        let request = GetServiceRequest { discovery_config: create_discovery_config("my-service") };
        service_clone2.get_service(request).await
    });

    let (result1, result2) = tokio::join!(handle1, handle2);

    assert!(result1.is_ok());
    assert!(result2.is_ok());

    let response1 = result1.unwrap();
    let response2 = result2.unwrap();

    assert_eq!(response1.response_status.error_code, ErrorCode::Success);
    assert_eq!(response2.response_status.error_code, ErrorCode::Success);
}
