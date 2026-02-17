//! Discovery Filter 单元测试
//!
//! 测试覆盖:
//! - StatusFilter: 过滤 DOWN 状态的实例
//! - ManagementDiscoveryFilter: 过滤被拉出的实例和服务器
//! - GroupRoutingFilter: 根据路由规则过滤实例
//! - DiscoveryFilterChain: 过滤器链组合测试

use artemis_core::model::{DiscoveryConfig, Instance, InstanceStatus, Service};
use artemis_management::{InstanceManager, RouteManager};
use artemis_server::discovery::filter::{
    DiscoveryFilter, DiscoveryFilterChain, GroupRoutingFilter, ManagementDiscoveryFilter,
    StatusFilter,
};
use artemis_server::routing::RouteEngine;
use std::sync::Arc;

/// 创建测试实例
fn create_test_instance(service_id: &str, instance_id: &str, status: InstanceStatus) -> Instance {
    Instance {
        region_id: "test-region".to_string(),
        zone_id: "test-zone".to_string(),
        group_id: Some("default".to_string()),
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

/// 创建测试服务
fn create_test_service(service_id: &str, instances: Vec<Instance>) -> Service {
    Service {
        service_id: service_id.to_string(),
        metadata: None,
        instances,
        logic_instances: None,
        route_rules: None,
    }
}

/// 创建测试 DiscoveryConfig
fn create_discovery_config(service_id: &str) -> DiscoveryConfig {
    DiscoveryConfig {
        service_id: service_id.to_string(),
        region_id: "test-region".to_string(),
        zone_id: "test-zone".to_string(),
        discovery_data: None,
    }
}

// ===== StatusFilter 测试 =====

#[tokio::test]
async fn test_status_filter_removes_down_instances() {
    let filter = StatusFilter;
    let config = create_discovery_config("my-service");

    let instances = vec![
        create_test_instance("my-service", "inst-1", InstanceStatus::Up),
        create_test_instance("my-service", "inst-2", InstanceStatus::Down),
        create_test_instance("my-service", "inst-3", InstanceStatus::Up),
    ];

    let mut service = create_test_service("my-service", instances);
    assert_eq!(service.instances.len(), 3);

    filter.filter(&mut service, &config).await.unwrap();

    // 只保留 UP 状态的实例
    assert_eq!(service.instances.len(), 2);
    assert!(service.instances.iter().all(|i| i.status == InstanceStatus::Up));
}

#[tokio::test]
async fn test_status_filter_all_up() {
    let filter = StatusFilter;
    let config = create_discovery_config("my-service");

    let instances = vec![
        create_test_instance("my-service", "inst-1", InstanceStatus::Up),
        create_test_instance("my-service", "inst-2", InstanceStatus::Up),
    ];

    let mut service = create_test_service("my-service", instances);
    filter.filter(&mut service, &config).await.unwrap();

    // 全部保留
    assert_eq!(service.instances.len(), 2);
}

#[tokio::test]
async fn test_status_filter_all_down() {
    let filter = StatusFilter;
    let config = create_discovery_config("my-service");

    let instances = vec![
        create_test_instance("my-service", "inst-1", InstanceStatus::Down),
        create_test_instance("my-service", "inst-2", InstanceStatus::Down),
    ];

    let mut service = create_test_service("my-service", instances);
    filter.filter(&mut service, &config).await.unwrap();

    // 全部过滤
    assert_eq!(service.instances.len(), 0);
}

#[tokio::test]
async fn test_status_filter_empty_service() {
    let filter = StatusFilter;
    let config = create_discovery_config("my-service");

    let mut service = create_test_service("my-service", vec![]);
    filter.filter(&mut service, &config).await.unwrap();

    // 保持为空
    assert_eq!(service.instances.len(), 0);
}

// ===== ManagementDiscoveryFilter 测试 =====

#[tokio::test]
async fn test_management_filter_removes_pulled_out_instances() {
    let instance_manager = Arc::new(InstanceManager::new());

    // 拉出一个实例
    let inst1 = create_test_instance("my-service", "inst-1", InstanceStatus::Up);
    let key1 = inst1.key();
    let _ = instance_manager.pull_out_instance(&key1, "test-operator".to_string(), true);

    let filter = ManagementDiscoveryFilter::new(instance_manager);
    let config = create_discovery_config("my-service");

    let instances = vec![
        create_test_instance("my-service", "inst-1", InstanceStatus::Up),
        create_test_instance("my-service", "inst-2", InstanceStatus::Up),
    ];

    let mut service = create_test_service("my-service", instances);
    filter.filter(&mut service, &config).await.unwrap();

    // inst-1 被过滤,只保留 inst-2
    assert_eq!(service.instances.len(), 1);
    assert_eq!(service.instances[0].instance_id, "inst-2");
}

#[tokio::test]
async fn test_management_filter_removes_server_pulled_out() {
    let instance_manager = Arc::new(InstanceManager::new());

    // 拉出服务器
    let _ = instance_manager.pull_out_server(
        "192.168.1.100",
        "test-region",
        "test-operator".to_string(),
        true,
    );

    let filter = ManagementDiscoveryFilter::new(instance_manager);
    let config = create_discovery_config("my-service");

    let instances = vec![
        create_test_instance("my-service", "inst-1", InstanceStatus::Up),
        create_test_instance("my-service", "inst-2", InstanceStatus::Up),
    ];

    let mut service = create_test_service("my-service", instances);
    filter.filter(&mut service, &config).await.unwrap();

    // 所有实例在同一个 IP 上,都被过滤
    assert_eq!(service.instances.len(), 0);
}

#[tokio::test]
async fn test_management_filter_no_pulled_out() {
    let instance_manager = Arc::new(InstanceManager::new());
    let filter = ManagementDiscoveryFilter::new(instance_manager);
    let config = create_discovery_config("my-service");

    let instances = vec![
        create_test_instance("my-service", "inst-1", InstanceStatus::Up),
        create_test_instance("my-service", "inst-2", InstanceStatus::Up),
    ];

    let mut service = create_test_service("my-service", instances);
    filter.filter(&mut service, &config).await.unwrap();

    // 全部保留
    assert_eq!(service.instances.len(), 2);
}

#[tokio::test]
async fn test_management_filter_pull_in_restores() {
    let instance_manager = Arc::new(InstanceManager::new());

    // 先拉出
    let inst1 = create_test_instance("my-service", "inst-1", InstanceStatus::Up);
    let key1 = inst1.key();
    let _ = instance_manager.pull_out_instance(&key1, "test-operator".to_string(), true);

    // 再拉入
    let _ = instance_manager.pull_in_instance(&key1, "test-operator".to_string(), true);

    let filter = ManagementDiscoveryFilter::new(instance_manager);
    let config = create_discovery_config("my-service");

    let instances = vec![
        create_test_instance("my-service", "inst-1", InstanceStatus::Up),
        create_test_instance("my-service", "inst-2", InstanceStatus::Up),
    ];

    let mut service = create_test_service("my-service", instances);
    filter.filter(&mut service, &config).await.unwrap();

    // 全部保留 (inst-1 已被拉入)
    assert_eq!(service.instances.len(), 2);
}

// ===== GroupRoutingFilter 测试 =====

#[tokio::test]
async fn test_group_routing_filter_no_rules() {
    let route_manager = Arc::new(RouteManager::new());
    let route_engine = Arc::new(RouteEngine::new());
    let filter = GroupRoutingFilter::new(route_manager, route_engine);
    let config = create_discovery_config("my-service");

    let instances = vec![
        create_test_instance("my-service", "inst-1", InstanceStatus::Up),
        create_test_instance("my-service", "inst-2", InstanceStatus::Up),
    ];

    let mut service = create_test_service("my-service", instances);
    filter.filter(&mut service, &config).await.unwrap();

    // 没有路由规则,全部保留
    assert_eq!(service.instances.len(), 2);
}

// GroupRoutingFilter 的详细规则测试需要复杂的 ServiceGroup/RouteRule 设置
// 这些在集成测试中已经有完整覆盖 (test-group-routing.sh)
// 这里主要测试过滤器的基本集成

// ===== DiscoveryFilterChain 测试 =====

#[tokio::test]
async fn test_filter_chain_empty() {
    let chain = DiscoveryFilterChain::new();
    let config = create_discovery_config("my-service");

    let instances = vec![
        create_test_instance("my-service", "inst-1", InstanceStatus::Up),
        create_test_instance("my-service", "inst-2", InstanceStatus::Down),
    ];

    let mut service = create_test_service("my-service", instances);
    chain.apply(&mut service, &config).await.unwrap();

    // 空链不改变实例列表
    assert_eq!(service.instances.len(), 2);
}

#[tokio::test]
async fn test_filter_chain_single_filter() {
    let mut chain = DiscoveryFilterChain::new();
    chain.add_filter(Arc::new(StatusFilter));

    let config = create_discovery_config("my-service");

    let instances = vec![
        create_test_instance("my-service", "inst-1", InstanceStatus::Up),
        create_test_instance("my-service", "inst-2", InstanceStatus::Down),
        create_test_instance("my-service", "inst-3", InstanceStatus::Up),
    ];

    let mut service = create_test_service("my-service", instances);
    chain.apply(&mut service, &config).await.unwrap();

    // StatusFilter 过滤 DOWN 实例
    assert_eq!(service.instances.len(), 2);
}

#[tokio::test]
async fn test_filter_chain_multiple_filters() {
    let mut chain = DiscoveryFilterChain::new();

    // 添加 StatusFilter
    chain.add_filter(Arc::new(StatusFilter));

    // 添加 ManagementDiscoveryFilter
    let instance_manager = Arc::new(InstanceManager::new());
    let inst2 = create_test_instance("my-service", "inst-2", InstanceStatus::Up);
    let key2 = inst2.key();
    let _ = instance_manager.pull_out_instance(&key2, "test-operator".to_string(), true);
    chain.add_filter(Arc::new(ManagementDiscoveryFilter::new(instance_manager)));

    let config = create_discovery_config("my-service");

    let instances = vec![
        create_test_instance("my-service", "inst-1", InstanceStatus::Up),
        create_test_instance("my-service", "inst-2", InstanceStatus::Up),
        create_test_instance("my-service", "inst-3", InstanceStatus::Down),
    ];

    let mut service = create_test_service("my-service", instances);
    chain.apply(&mut service, &config).await.unwrap();

    // StatusFilter 过滤 inst-3 (DOWN)
    // ManagementDiscoveryFilter 过滤 inst-2 (pulled out)
    // 只保留 inst-1
    assert_eq!(service.instances.len(), 1);
    assert_eq!(service.instances[0].instance_id, "inst-1");
}

#[tokio::test]
async fn test_filter_chain_order_matters() {
    // 创建两个链,顺序不同
    let mut chain1 = DiscoveryFilterChain::new();
    let mut chain2 = DiscoveryFilterChain::new();

    let instance_manager = Arc::new(InstanceManager::new());
    let inst1 = create_test_instance("my-service", "inst-1", InstanceStatus::Up);
    let key1 = inst1.key();
    let _ = instance_manager.pull_out_instance(&key1, "test-operator".to_string(), true);

    // chain1: StatusFilter -> ManagementDiscoveryFilter
    chain1.add_filter(Arc::new(StatusFilter));
    chain1.add_filter(Arc::new(ManagementDiscoveryFilter::new(instance_manager.clone())));

    // chain2: ManagementDiscoveryFilter -> StatusFilter
    chain2.add_filter(Arc::new(ManagementDiscoveryFilter::new(instance_manager)));
    chain2.add_filter(Arc::new(StatusFilter));

    let config = create_discovery_config("my-service");

    let instances1 = vec![
        create_test_instance("my-service", "inst-1", InstanceStatus::Up),
        create_test_instance("my-service", "inst-2", InstanceStatus::Down),
    ];

    let instances2 = instances1.clone();

    let mut service1 = create_test_service("my-service", instances1);
    let mut service2 = create_test_service("my-service", instances2);

    chain1.apply(&mut service1, &config).await.unwrap();
    chain2.apply(&mut service2, &config).await.unwrap();

    // 两个链的结果应该相同 (因为过滤是独立的)
    assert_eq!(service1.instances.len(), service2.instances.len());
    assert_eq!(service1.instances.len(), 0);
}

// ===== 边界条件测试 =====

#[tokio::test]
async fn test_filter_chain_all_instances_filtered() {
    let mut chain = DiscoveryFilterChain::new();
    chain.add_filter(Arc::new(StatusFilter));

    let config = create_discovery_config("my-service");

    let instances = vec![
        create_test_instance("my-service", "inst-1", InstanceStatus::Down),
        create_test_instance("my-service", "inst-2", InstanceStatus::Down),
    ];

    let mut service = create_test_service("my-service", instances);
    chain.apply(&mut service, &config).await.unwrap();

    // 所有实例都被过滤
    assert_eq!(service.instances.len(), 0);
}

#[tokio::test]
async fn test_management_filter_multiple_pulled_out() {
    let instance_manager = Arc::new(InstanceManager::new());

    // 拉出多个实例
    let inst1 = create_test_instance("my-service", "inst-1", InstanceStatus::Up);
    let inst2 = create_test_instance("my-service", "inst-2", InstanceStatus::Up);

    let _ = instance_manager.pull_out_instance(&inst1.key(), "test-operator".to_string(), true);
    let _ = instance_manager.pull_out_instance(&inst2.key(), "test-operator".to_string(), true);

    let filter = ManagementDiscoveryFilter::new(instance_manager);
    let config = create_discovery_config("my-service");

    let instances = vec![
        create_test_instance("my-service", "inst-1", InstanceStatus::Up),
        create_test_instance("my-service", "inst-2", InstanceStatus::Up),
        create_test_instance("my-service", "inst-3", InstanceStatus::Up),
    ];

    let mut service = create_test_service("my-service", instances);
    filter.filter(&mut service, &config).await.unwrap();

    // 只保留 inst-3
    assert_eq!(service.instances.len(), 1);
    assert_eq!(service.instances[0].instance_id, "inst-3");
}

#[tokio::test]
async fn test_filter_chain_default_constructor() {
    let chain = DiscoveryFilterChain::default();
    let config = create_discovery_config("my-service");

    let instances = vec![create_test_instance("my-service", "inst-1", InstanceStatus::Up)];

    let mut service = create_test_service("my-service", instances);
    chain.apply(&mut service, &config).await.unwrap();

    // Default 构造的链应该为空
    assert_eq!(service.instances.len(), 1);
}

#[tokio::test]
async fn test_filter_chain_clone() {
    let mut chain = DiscoveryFilterChain::new();
    chain.add_filter(Arc::new(StatusFilter));

    // Clone 过滤器链
    let cloned_chain = chain.clone();

    let config = create_discovery_config("my-service");

    let instances = vec![
        create_test_instance("my-service", "inst-1", InstanceStatus::Up),
        create_test_instance("my-service", "inst-2", InstanceStatus::Down),
    ];

    let mut service = create_test_service("my-service", instances);
    cloned_chain.apply(&mut service, &config).await.unwrap();

    // Clone 的链应该有相同的过滤器
    assert_eq!(service.instances.len(), 1);
}
