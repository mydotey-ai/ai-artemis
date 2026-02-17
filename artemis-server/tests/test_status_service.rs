//! StatusService 核心服务单元测试
//!
//! 测试覆盖:
//! - get_cluster_node_status: 获取当前节点状态
//! - get_cluster_status: 获取集群状态,包含/不包含集群管理器
//! - get_leases_status: 获取租约状态,带/不带过滤
//! - get_legacy_leases_status: 传统租约状态 API
//! - get_config_status: 获取配置状态
//! - get_deployment_status: 获取部署状态
//! - parse_url: URL 解析辅助函数

use artemis_core::model::{ErrorCode, InstanceKey};
use artemis_management::model::{
    GetClusterNodeStatusRequest, GetClusterStatusRequest, GetConfigStatusRequest,
    GetDeploymentStatusRequest, GetLeasesStatusRequest,
};
use artemis_server::{StatusService, cluster::ClusterManager, lease::LeaseManager};
use std::sync::Arc;
use std::time::Duration;

/// 创建测试用的 StatusService (不包含 ClusterManager)
fn create_test_status_service() -> (StatusService, Arc<LeaseManager>) {
    let lease_manager = Arc::new(LeaseManager::new(Duration::from_secs(30)));

    let service = StatusService::new(
        None, // No cluster manager
        lease_manager.clone(),
        "test-node-1".to_string(),
        "test-region".to_string(),
        "test-zone".to_string(),
        "http://localhost:8080".to_string(),
        "test-app".to_string(),
    );

    (service, lease_manager)
}

/// 创建包含 ClusterManager 的 StatusService
fn create_test_status_service_with_cluster() -> StatusService {
    let lease_manager = Arc::new(LeaseManager::new(Duration::from_secs(30)));

    // 创建有 2 个 peer 节点的 ClusterManager
    let peers = vec!["http://192.168.1.2:8080".to_string(), "http://192.168.1.3:8080".to_string()];
    let cluster_manager = Arc::new(ClusterManager::new("node-1".to_string(), peers));

    StatusService::new(
        Some(cluster_manager),
        lease_manager,
        "node-1".to_string(),
        "region-1".to_string(),
        "zone-1".to_string(),
        "http://localhost:8080".to_string(),
        "test-app".to_string(),
    )
}

/// 创建测试实例 key
fn create_test_instance_key(service_id: &str, instance_id: &str) -> InstanceKey {
    InstanceKey {
        region_id: "test-region".to_string(),
        zone_id: "test-zone".to_string(),
        group_id: String::new(),
        service_id: service_id.to_string(),
        instance_id: instance_id.to_string(),
    }
}

// ===== get_cluster_node_status 测试 =====

#[tokio::test]
async fn test_get_cluster_node_status_success() {
    let (service, _) = create_test_status_service();

    let request = GetClusterNodeStatusRequest {};
    let response = service.get_cluster_node_status(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    assert!(response.node_status.is_some());

    let node_status = response.node_status.unwrap();
    assert_eq!(node_status.node.node_id, "test-node-1");
    assert_eq!(node_status.node.region_id, "test-region");
    assert_eq!(node_status.node.zone_id, "test-zone");
    assert_eq!(node_status.node.url, "http://localhost:8080");
    assert_eq!(node_status.status, "up");
    assert!(node_status.can_service_discovery);
    assert!(node_status.can_service_registry);
}

#[tokio::test]
async fn test_get_cluster_node_status_returns_correct_capabilities() {
    let (service, _) = create_test_status_service();

    let request = GetClusterNodeStatusRequest {};
    let response = service.get_cluster_node_status(request).await;

    let node_status = response.node_status.unwrap();

    // 验证节点能力
    assert!(node_status.allow_registry_from_other_zone);
    assert!(node_status.allow_discovery_from_other_zone);
}

// ===== get_cluster_status 测试 =====

#[tokio::test]
async fn test_get_cluster_status_without_cluster_manager() {
    let (service, _) = create_test_status_service();

    let request = GetClusterStatusRequest {};
    let response = service.get_cluster_status(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::Success);

    // 只有当前节点
    assert_eq!(response.node_count, 1);
    assert_eq!(response.nodes_status.len(), 1);

    let node = &response.nodes_status[0];
    assert_eq!(node.node.node_id, "test-node-1");
    assert_eq!(node.status, "up");
}

#[tokio::test]
async fn test_get_cluster_status_with_cluster_manager() {
    let service = create_test_status_service_with_cluster();

    let request = GetClusterStatusRequest {};
    let response = service.get_cluster_status(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::Success);

    // 至少有当前节点 (集群节点可能因为健康检查未通过而不在列表中)
    assert!(response.node_count >= 1);
    assert!(!response.nodes_status.is_empty());

    // 验证当前节点存在
    let has_current_node = response.nodes_status.iter().any(|n| n.node.node_id == "node-1");
    assert!(has_current_node);
}

#[tokio::test]
async fn test_get_cluster_status_all_nodes_healthy() {
    let service = create_test_status_service_with_cluster();

    let request = GetClusterStatusRequest {};
    let response = service.get_cluster_status(request).await;

    // 所有返回的节点应该是 UP 状态 (get_healthy_nodes 已过滤)
    for node_status in &response.nodes_status {
        assert_eq!(node_status.status, "up");
        assert!(node_status.can_service_discovery);
        assert!(node_status.can_service_registry);
    }
}

// ===== get_leases_status 测试 =====

#[tokio::test]
async fn test_get_leases_status_empty() {
    let (service, _) = create_test_status_service();

    let request = GetLeasesStatusRequest { service_ids: None };

    let response = service.get_leases_status(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    assert_eq!(response.lease_count, 0);
    assert!(response.leases_status.is_empty());
    assert!(response.is_safe);
}

#[tokio::test]
async fn test_get_leases_status_with_leases() {
    let (service, lease_mgr) = create_test_status_service();

    // 创建租约
    let key1 = create_test_instance_key("service-1", "inst-1");
    let key2 = create_test_instance_key("service-1", "inst-2");
    let key3 = create_test_instance_key("service-2", "inst-1");

    lease_mgr.create_lease(key1);
    lease_mgr.create_lease(key2);
    lease_mgr.create_lease(key3);

    let request = GetLeasesStatusRequest { service_ids: None };

    let response = service.get_leases_status(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    assert_eq!(response.lease_count, 3);
    assert_eq!(response.leases_status.len(), 2); // 2 个服务

    // 验证 service-1 有 2 个租约
    assert!(response.leases_status.contains_key("service-1"));
    assert_eq!(response.leases_status["service-1"].len(), 2);

    // 验证 service-2 有 1 个租约
    assert!(response.leases_status.contains_key("service-2"));
    assert_eq!(response.leases_status["service-2"].len(), 1);
}

#[tokio::test]
async fn test_get_leases_status_with_filter() {
    let (service, lease_mgr) = create_test_status_service();

    // 创建租约
    let key1 = create_test_instance_key("service-1", "inst-1");
    let key2 = create_test_instance_key("service-2", "inst-1");

    lease_mgr.create_lease(key1);
    lease_mgr.create_lease(key2);

    // 只查询 service-1
    let request = GetLeasesStatusRequest { service_ids: Some(vec!["service-1".to_string()]) };

    let response = service.get_leases_status(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    assert_eq!(response.lease_count, 1);
    assert_eq!(response.leases_status.len(), 1);
    assert!(response.leases_status.contains_key("service-1"));
    assert!(!response.leases_status.contains_key("service-2"));
}

#[tokio::test]
async fn test_get_leases_status_filter_multiple_services() {
    let (service, lease_mgr) = create_test_status_service();

    // 创建租约
    let key1 = create_test_instance_key("service-1", "inst-1");
    let key2 = create_test_instance_key("service-2", "inst-1");
    let key3 = create_test_instance_key("service-3", "inst-1");

    lease_mgr.create_lease(key1);
    lease_mgr.create_lease(key2);
    lease_mgr.create_lease(key3);

    // 查询 service-1 和 service-3
    let request = GetLeasesStatusRequest {
        service_ids: Some(vec!["service-1".to_string(), "service-3".to_string()]),
    };

    let response = service.get_leases_status(request).await;

    assert_eq!(response.lease_count, 2);
    assert_eq!(response.leases_status.len(), 2);
    assert!(response.leases_status.contains_key("service-1"));
    assert!(!response.leases_status.contains_key("service-2"));
    assert!(response.leases_status.contains_key("service-3"));
}

#[tokio::test]
async fn test_get_leases_status_lease_fields() {
    let (service, lease_mgr) = create_test_status_service();

    let key = create_test_instance_key("my-service", "inst-1");
    lease_mgr.create_lease(key);

    let request = GetLeasesStatusRequest { service_ids: None };

    let response = service.get_leases_status(request).await;

    let lease_status = &response.leases_status["my-service"][0];

    // 验证租约字段
    assert_eq!(lease_status.instance, "inst-1");
    assert!(lease_status.creation_time.contains("seconds ago"));
    assert!(lease_status.renewal_time.contains("seconds ago"));
    assert!(lease_status.ttl > 0);

    // eviction_time 应该是 "in X seconds" 或 "expired"
    assert!(lease_status.evition_time.contains("in") || lease_status.evition_time == "expired");
}

// ===== get_legacy_leases_status 测试 =====

#[tokio::test]
async fn test_get_legacy_leases_status() {
    let (service, lease_mgr) = create_test_status_service();

    // 创建租约
    let key = create_test_instance_key("my-service", "inst-1");
    lease_mgr.create_lease(key);

    let request = GetLeasesStatusRequest { service_ids: None };

    // Legacy API 应该返回与 get_leases_status 相同的结果
    let legacy_response = service.get_legacy_leases_status(request.clone()).await;
    let normal_response = service.get_leases_status(request).await;

    assert_eq!(legacy_response.lease_count, normal_response.lease_count);
    assert_eq!(legacy_response.leases_status.len(), normal_response.leases_status.len());
}

// ===== get_config_status 测试 =====

#[tokio::test]
async fn test_get_config_status() {
    let (service, _) = create_test_status_service();

    let request = GetConfigStatusRequest {};
    let response = service.get_config_status(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::Success);

    // 验证 sources
    assert!(response.sources.contains_key("default"));
    assert_eq!(response.sources["default"], 1);

    // 验证 properties
    assert_eq!(response.properties["node_id"], "test-node-1");
    assert_eq!(response.properties["region_id"], "test-region");
    assert_eq!(response.properties["zone_id"], "test-zone");
    assert_eq!(response.properties["app_id"], "test-app");
}

#[tokio::test]
async fn test_get_config_status_contains_required_fields() {
    let (service, _) = create_test_status_service();

    let request = GetConfigStatusRequest {};
    let response = service.get_config_status(request).await;

    // 必须包含的字段
    assert!(response.properties.contains_key("node_id"));
    assert!(response.properties.contains_key("region_id"));
    assert!(response.properties.contains_key("zone_id"));
    assert!(response.properties.contains_key("app_id"));
}

// ===== get_deployment_status 测试 =====

#[tokio::test]
async fn test_get_deployment_status() {
    let (service, _) = create_test_status_service();

    let request = GetDeploymentStatusRequest {};
    let response = service.get_deployment_status(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::Success);

    // 验证基本字段
    assert_eq!(response.region_id, "test-region");
    assert_eq!(response.zone_id, "test-zone");
    assert_eq!(response.app_id, "test-app");

    // 验证 URL 解析
    assert_eq!(response.ip, "localhost");
    assert_eq!(response.port, 8080);
    assert_eq!(response.protocol, "http");
    assert_eq!(response.path, "/");

    // machine_name 应该不为空
    assert!(!response.machine_name.is_empty());

    // 验证 sources 和 properties
    assert!(response.sources.contains_key("default"));
    assert!(response.properties.contains_key("node_id"));
}

#[tokio::test]
async fn test_get_deployment_status_with_complex_url() {
    let lease_manager = Arc::new(LeaseManager::new(Duration::from_secs(30)));

    let service = StatusService::new(
        None,
        lease_manager,
        "test-node".to_string(),
        "region-1".to_string(),
        "zone-1".to_string(),
        "https://192.168.1.100:9090/api/v1".to_string(),
        "my-app".to_string(),
    );

    let request = GetDeploymentStatusRequest {};
    let response = service.get_deployment_status(request).await;

    // 验证复杂 URL 解析
    assert_eq!(response.ip, "192.168.1.100");
    assert_eq!(response.port, 9090);
    assert_eq!(response.protocol, "https");
    assert_eq!(response.path, "/api/v1");
}

#[tokio::test]
async fn test_get_deployment_status_url_without_port() {
    let lease_manager = Arc::new(LeaseManager::new(Duration::from_secs(30)));

    let service = StatusService::new(
        None,
        lease_manager,
        "test-node".to_string(),
        "region-1".to_string(),
        "zone-1".to_string(),
        "http://example.com".to_string(),
        "my-app".to_string(),
    );

    let request = GetDeploymentStatusRequest {};
    let response = service.get_deployment_status(request).await;

    assert_eq!(response.ip, "example.com");
    assert_eq!(response.port, 8080); // 默认端口
    assert_eq!(response.path, "/");
}

// ===== 多服务租约测试 =====

#[tokio::test]
async fn test_get_leases_status_many_services() {
    let (service, lease_mgr) = create_test_status_service();

    // 创建 10 个服务,每个服务 3 个实例
    for i in 1..=10 {
        let service_id = format!("service-{}", i);
        for j in 1..=3 {
            let instance_id = format!("inst-{}", j);
            let key = create_test_instance_key(&service_id, &instance_id);
            lease_mgr.create_lease(key);
        }
    }

    let request = GetLeasesStatusRequest { service_ids: None };

    let response = service.get_leases_status(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    assert_eq!(response.lease_count, 30); // 10 服务 * 3 实例
    assert_eq!(response.leases_status.len(), 10); // 10 个服务
}

// ===== 边界条件测试 =====

#[tokio::test]
async fn test_get_leases_status_filter_non_existent_service() {
    let (service, lease_mgr) = create_test_status_service();

    // 创建租约
    let key = create_test_instance_key("existing-service", "inst-1");
    lease_mgr.create_lease(key);

    // 查询不存在的服务
    let request =
        GetLeasesStatusRequest { service_ids: Some(vec!["non-existent-service".to_string()]) };

    let response = service.get_leases_status(request).await;

    assert_eq!(response.response_status.error_code, ErrorCode::Success);
    assert_eq!(response.lease_count, 0);
    assert!(response.leases_status.is_empty());
}

#[tokio::test]
async fn test_get_leases_status_empty_filter() {
    let (service, lease_mgr) = create_test_status_service();

    // 创建租约
    let key = create_test_instance_key("my-service", "inst-1");
    lease_mgr.create_lease(key);

    // 空过滤器
    let request = GetLeasesStatusRequest { service_ids: Some(vec![]) };

    let response = service.get_leases_status(request).await;

    // 空过滤器应该返回 0 个结果
    assert_eq!(response.lease_count, 0);
    assert!(response.leases_status.is_empty());
}

#[tokio::test]
async fn test_get_deployment_status_properties() {
    let (service, _) = create_test_status_service();

    let request = GetDeploymentStatusRequest {};
    let response = service.get_deployment_status(request).await;

    // 验证 properties 包含必需字段
    assert!(response.properties.contains_key("node_id"));
    assert!(response.properties.contains_key("region_id"));
    assert!(response.properties.contains_key("zone_id"));
    assert!(response.properties.contains_key("server_url"));

    // 验证字段值
    assert_eq!(response.properties["node_id"], "test-node-1");
    assert_eq!(response.properties["region_id"], "test-region");
    assert_eq!(response.properties["zone_id"], "test-zone");
    assert_eq!(response.properties["server_url"], "http://localhost:8080");
}
