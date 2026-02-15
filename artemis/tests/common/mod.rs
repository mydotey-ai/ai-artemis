//! 通用测试工具和 Fixture
//!
//! 提供跨测试使用的通用功能:
//! - TestServer: 测试服务器管理
//! - TestCluster: 测试集群管理
//! - Fixture: 测试数据构造器

use artemis_core::model::{Instance, InstanceStatus, InstanceKey, ServiceGroup, RouteRule, RouteStrategy};
use artemis_server::{
    RegistryServiceImpl, VersionedCacheManager, InstanceChangeManager,
    discovery::DiscoveryServiceImpl, lease::LeaseManager, registry::RegistryRepository,
};
use artemis_web::state::AppState;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;

/// 测试服务器
pub struct TestServer {
    pub port: u16,
    pub handle: JoinHandle<()>,
}

impl TestServer {
    /// 启动测试服务器
    pub async fn start(port: u16) -> Self {
        let handle = tokio::spawn(async move {
            let repository = RegistryRepository::new();
            let lease_manager = Arc::new(LeaseManager::new(Duration::from_secs(30)));
            let cache = Arc::new(VersionedCacheManager::new());
            let change_manager = Arc::new(InstanceChangeManager::new());

            let registry_service = Arc::new(RegistryServiceImpl::new(
                repository.clone(),
                lease_manager.clone(),
                cache.clone(),
                change_manager.clone(),
                None, // No replication in test
            ));
            let discovery_service = Arc::new(DiscoveryServiceImpl::new(
                repository,
                cache.clone(),
            ));

            let session_manager = Arc::new(artemis_web::websocket::SessionManager::new());
            let instance_manager = Arc::new(artemis_management::InstanceManager::new());

            let app_state = AppState {
                registry_service,
                discovery_service,
                cache,
                session_manager,
                cluster_manager: None,
                replication_manager: None,
                instance_manager,
            };

            let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
            let _ = artemis_web::server::run_server(app_state, addr).await;
        });

        // 等待服务器启动
        tokio::time::sleep(Duration::from_millis(500)).await;

        Self { port, handle }
    }

    /// 获取服务器 URL
    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    /// 停止服务器
    pub async fn stop(self) {
        self.handle.abort();
        let _ = self.handle.await;
    }
}

/// 测试集群
pub struct TestCluster {
    pub nodes: Vec<TestServer>,
}

impl TestCluster {
    /// 启动 N 个节点的集群
    pub async fn start(node_count: usize, start_port: u16) -> Self {
        let mut nodes = Vec::new();
        for i in 0..node_count {
            let port = start_port + i as u16;
            let server = TestServer::start(port).await;
            nodes.push(server);
        }

        // 等待所有节点启动
        tokio::time::sleep(Duration::from_millis(500)).await;

        Self { nodes }
    }

    /// 获取所有节点的 URL
    pub fn urls(&self) -> Vec<String> {
        self.nodes.iter().map(|n| n.url()).collect()
    }

    /// 停止所有节点
    pub async fn stop(self) {
        for node in self.nodes {
            node.stop().await;
        }
    }
}

/// 实例 Fixture
pub struct InstanceFixture;

impl InstanceFixture {
    /// 创建默认测试实例
    pub fn default() -> Instance {
        Instance {
            region_id: "test-region".to_string(),
            zone_id: "test-zone".to_string(),
            service_id: "test-service".to_string(),
            group_id: None,
            instance_id: "test-inst-1".to_string(),
            machine_name: Some("test-machine".to_string()),
            ip: "192.168.1.100".to_string(),
            port: 8080,
            protocol: Some("http".to_string()),
            url: "http://192.168.1.100:8080".to_string(),
            health_check_url: Some("http://192.168.1.100:8080/health".to_string()),
            status: InstanceStatus::Up,
            metadata: None,
        }
    }

    /// 创建指定 ID 的实例
    pub fn with_id(id: &str) -> Instance {
        let mut instance = Self::default();
        instance.instance_id = id.to_string();
        instance
    }

    /// 创建指定服务 ID 的实例
    pub fn with_service_id(service_id: &str) -> Instance {
        let mut instance = Self::default();
        instance.service_id = service_id.to_string();
        instance
    }

    /// 创建指定状态的实例
    pub fn with_status(status: InstanceStatus) -> Instance {
        let mut instance = Self::default();
        instance.status = status;
        instance
    }

    /// 创建批量实例
    pub fn batch(count: usize) -> Vec<Instance> {
        (0..count)
            .map(|i| {
                let mut instance = Self::default();
                instance.instance_id = format!("inst-{}", i);
                instance.ip = format!("192.168.1.{}", 100 + (i % 150));
                instance.port = 8080 + (i as u16 % 100);
                instance.url = format!("http://{}:{}", instance.ip, instance.port);
                instance
            })
            .collect()
    }

    /// 创建指定服务的批量实例
    pub fn batch_for_service(service_id: &str, count: usize) -> Vec<Instance> {
        Self::batch(count)
            .into_iter()
            .map(|mut inst| {
                inst.service_id = service_id.to_string();
                inst
            })
            .collect()
    }
}

/// 服务分组 Fixture
pub struct GroupFixture;

impl GroupFixture {
    /// 创建默认分组
    pub fn default() -> ServiceGroup {
        ServiceGroup {
            group_id: "test-group".to_string(),
            region_id: "test-region".to_string(),
            description: Some("Test group".to_string()),
            tags: None,
        }
    }

    /// 创建指定 ID 的分组
    pub fn with_id(group_id: &str) -> ServiceGroup {
        let mut group = Self::default();
        group.group_id = group_id.to_string();
        group
    }
}

/// 路由规则 Fixture
pub struct RouteRuleFixture;

impl RouteRuleFixture {
    /// 创建默认路由规则 (加权轮询)
    pub fn default() -> RouteRule {
        RouteRule {
            rule_id: "test-rule".to_string(),
            strategy: RouteStrategy::WeightedRoundRobin,
            unfiltered_group_ids: Vec::new(),
            route_groups: Vec::new(),
        }
    }

    /// 创建就近访问规则
    pub fn close_by_visit() -> RouteRule {
        RouteRule {
            rule_id: "close-by-rule".to_string(),
            strategy: RouteStrategy::CloseByVisit,
            unfiltered_group_ids: Vec::new(),
            route_groups: Vec::new(),
        }
    }

    /// 创建指定 ID 的规则
    pub fn with_id(rule_id: &str) -> RouteRule {
        let mut rule = Self::default();
        rule.rule_id = rule_id.to_string();
        rule
    }
}

/// 实例 Key Fixture
pub struct InstanceKeyFixture;

impl InstanceKeyFixture {
    /// 创建默认实例 Key
    pub fn default() -> InstanceKey {
        InstanceKey::new(
            "test-region",
            "test-zone",
            "test-service",
            "test-inst-1",
        )
    }

    /// 从实例创建 Key
    pub fn from_instance(instance: &Instance) -> InstanceKey {
        instance.key()
    }

    /// 创建批量实例 Key
    pub fn batch(count: usize) -> Vec<InstanceKey> {
        (0..count)
            .map(|i| {
                InstanceKey::new(
                    "test-region",
                    "test-zone",
                    "test-service",
                    &format!("inst-{}", i),
                )
            })
            .collect()
    }
}

/// 等待条件满足的辅助函数
pub async fn wait_for_condition<F, Fut>(
    condition: F,
    timeout: Duration,
    interval: Duration,
) -> bool
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if condition().await {
            return true;
        }
        tokio::time::sleep(interval).await;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_fixture_default() {
        let instance = InstanceFixture::default();
        assert_eq!(instance.region_id, "test-region");
        assert_eq!(instance.zone_id, "test-zone");
        assert_eq!(instance.service_id, "test-service");
        assert_eq!(instance.instance_id, "test-inst-1");
        assert_eq!(instance.status, InstanceStatus::Up);
    }

    #[test]
    fn test_instance_fixture_with_id() {
        let instance = InstanceFixture::with_id("custom-id");
        assert_eq!(instance.instance_id, "custom-id");
    }

    #[test]
    fn test_instance_fixture_batch() {
        let instances = InstanceFixture::batch(5);
        assert_eq!(instances.len(), 5);
        assert_eq!(instances[0].instance_id, "inst-0");
        assert_eq!(instances[4].instance_id, "inst-4");
    }

    #[test]
    fn test_group_fixture_default() {
        let group = GroupFixture::default();
        assert_eq!(group.group_id, "test-group");
        assert_eq!(group.region_id, "test-region");
    }

    #[test]
    fn test_route_rule_fixture_default() {
        let rule = RouteRuleFixture::default();
        assert_eq!(rule.rule_id, "test-rule");
        assert_eq!(rule.strategy, RouteStrategy::WeightedRoundRobin);
    }

    #[test]
    fn test_instance_key_fixture_default() {
        let key = InstanceKeyFixture::default();
        assert_eq!(key.region_id, "test-region");
        assert_eq!(key.zone_id, "test-zone");
        assert_eq!(key.service_id, "test-service");
        assert_eq!(key.instance_id, "test-inst-1");
    }

    #[tokio::test]
    async fn test_wait_for_condition_success() {
        let result = wait_for_condition(
            || async { true },
            Duration::from_secs(1),
            Duration::from_millis(100),
        )
        .await;
        assert!(result);
    }

    #[tokio::test]
    async fn test_wait_for_condition_timeout() {
        let result = wait_for_condition(
            || async { false },
            Duration::from_millis(200),
            Duration::from_millis(50),
        )
        .await;
        assert!(!result);
    }
}
