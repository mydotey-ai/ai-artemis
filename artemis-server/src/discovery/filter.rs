use artemis_core::model::{DiscoveryConfig, InstanceStatus, Service};
use artemis_management::{InstanceManager, RouteManager};
use async_trait::async_trait;
use std::sync::Arc;

use crate::routing::{RouteContext, RouteEngine};

pub type Result<T> = std::result::Result<T, anyhow::Error>;

#[async_trait]
pub trait DiscoveryFilter: Send + Sync {
    async fn filter(&self, service: &mut Service, config: &DiscoveryConfig) -> Result<()>;
}

#[derive(Clone)]
pub struct DiscoveryFilterChain {
    filters: Vec<Arc<dyn DiscoveryFilter>>,
}

impl DiscoveryFilterChain {
    pub fn new() -> Self {
        Self { filters: Vec::new() }
    }

    pub fn add_filter(&mut self, filter: Arc<dyn DiscoveryFilter>) {
        self.filters.push(filter);
    }

    pub async fn apply(&self, service: &mut Service, config: &DiscoveryConfig) -> Result<()> {
        for filter in &self.filters {
            filter.filter(service, config).await?;
        }
        Ok(())
    }
}

impl Default for DiscoveryFilterChain {
    fn default() -> Self {
        Self::new()
    }
}

pub struct StatusFilter;

#[async_trait]
impl DiscoveryFilter for StatusFilter {
    async fn filter(&self, service: &mut Service, _config: &DiscoveryConfig) -> Result<()> {
        service.instances.retain(|inst| matches!(inst.status, InstanceStatus::Up));
        Ok(())
    }
}

/// 管理过滤器 - 移除被拉出的实例和服务器
pub struct ManagementDiscoveryFilter {
    instance_manager: Arc<InstanceManager>,
}

impl ManagementDiscoveryFilter {
    pub fn new(instance_manager: Arc<InstanceManager>) -> Self {
        Self { instance_manager }
    }
}

#[async_trait]
impl DiscoveryFilter for ManagementDiscoveryFilter {
    async fn filter(&self, service: &mut Service, _config: &DiscoveryConfig) -> Result<()> {
        let original_count = service.instances.len();

        // 移除被拉出的实例和服务器
        service.instances.retain(|inst| {
            let key = inst.key();

            // 检查实例是否被拉出
            if self.instance_manager.is_instance_down(&key) {
                tracing::debug!("Filtering out instance (pull-out): {:?}", key);
                return false;
            }

            // 检查服务器是否被拉出
            if self.instance_manager.is_server_down(&inst.ip, &inst.region_id) {
                tracing::debug!(
                    "Filtering out instance (server pull-out): {} on {}",
                    key.instance_id,
                    inst.ip
                );
                return false;
            }

            true
        });

        let filtered_count = original_count - service.instances.len();
        if filtered_count > 0 {
            tracing::info!(
                "ManagementDiscoveryFilter filtered {} instances from service {}",
                filtered_count,
                service.service_id
            );
        }

        Ok(())
    }
}

/// 分组路由过滤器 - 根据路由规则过滤实例
pub struct GroupRoutingFilter {
    route_manager: Arc<RouteManager>,
    route_engine: Arc<RouteEngine>,
}

impl GroupRoutingFilter {
    pub fn new(route_manager: Arc<RouteManager>, route_engine: Arc<RouteEngine>) -> Self {
        Self { route_manager, route_engine }
    }
}

#[async_trait]
impl DiscoveryFilter for GroupRoutingFilter {
    async fn filter(&self, service: &mut Service, config: &DiscoveryConfig) -> Result<()> {
        // 获取服务的激活路由规则
        let active_rules = self.route_manager.get_active_rules(&service.service_id);

        if active_rules.is_empty() {
            tracing::debug!("No active routing rules for service {}", service.service_id);
            return Ok(());
        }

        // 构建路由上下文 (从 config 中提取客户端信息)
        let context = RouteContext::new()
            .with_region(config.region_id.clone())
            .with_zone(config.zone_id.clone());

        // 应用第一个激活的路由规则 (如果有多个规则,按优先级应用第一个)
        if let Some(rule) = active_rules.first() {
            tracing::info!(
                "Applying routing rule {} to service {} with strategy {:?}",
                rule.route_id,
                service.service_id,
                rule.strategy
            );

            // 提取实例列表
            let instances = std::mem::take(&mut service.instances);

            // 应用路由引擎
            let filtered_instances = self.route_engine.apply_route_rule(instances, rule, &context);

            // 更新服务实例
            service.instances = filtered_instances;

            tracing::info!(
                "Routing filter result: {} instances for service {}",
                service.instances.len(),
                service.service_id
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use artemis_core::model::Instance;

    fn create_test_instance(instance_id: &str, status: InstanceStatus) -> Instance {
        Instance {
            region_id: "test-region".to_string(),
            zone_id: "test-zone".to_string(),
            group_id: None,
            service_id: "test-service".to_string(),
            instance_id: instance_id.to_string(),
            machine_name: None,
            ip: "127.0.0.1".to_string(),
            port: 8080,
            protocol: None,
            url: "http://127.0.0.1:8080".to_string(),
            status,
            metadata: None,
            health_check_url: None,
        }
    }

    fn create_test_service(instances: Vec<Instance>) -> Service {
        Service {
            service_id: "test-service".to_string(),
            instances,
            metadata: None,
            logic_instances: None,
            route_rules: None,
        }
    }

    // ========== DiscoveryFilterChain 测试 ==========

    #[tokio::test]
    async fn test_filter_chain_empty() {
        let chain = DiscoveryFilterChain::new();
        let mut service = create_test_service(vec![]);
        let config = DiscoveryConfig {
            service_id: "test-service".to_string(),
            region_id: "test-region".to_string(),
            zone_id: "test-zone".to_string(),
            discovery_data: None,
        };

        let result = chain.apply(&mut service, &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_filter_chain_default() {
        let chain = DiscoveryFilterChain::default();
        let mut service = create_test_service(vec![]);
        let config = DiscoveryConfig {
            service_id: "test-service".to_string(),
            region_id: "test-region".to_string(),
            zone_id: "test-zone".to_string(),
            discovery_data: None,
        };

        let result = chain.apply(&mut service, &config).await;
        assert!(result.is_ok());
    }

    // ========== StatusFilter 测试 ==========

    #[tokio::test]
    async fn test_status_filter_removes_non_up_instances() {
        let filter = StatusFilter;
        let mut service = create_test_service(vec![
            create_test_instance("inst-1", InstanceStatus::Up),
            create_test_instance("inst-2", InstanceStatus::Down),
            create_test_instance("inst-3", InstanceStatus::Starting),
            create_test_instance("inst-4", InstanceStatus::Up),
        ]);
        let config = DiscoveryConfig {
            service_id: "test-service".to_string(),
            region_id: "test-region".to_string(),
            zone_id: "test-zone".to_string(),
            discovery_data: None,
        };

        let result = filter.filter(&mut service, &config).await;
        assert!(result.is_ok());
        assert_eq!(service.instances.len(), 2);
        assert!(service.instances.iter().all(|inst| inst.status == InstanceStatus::Up));
    }

    #[tokio::test]
    async fn test_status_filter_all_up() {
        let filter = StatusFilter;
        let mut service = create_test_service(vec![
            create_test_instance("inst-1", InstanceStatus::Up),
            create_test_instance("inst-2", InstanceStatus::Up),
        ]);
        let config = DiscoveryConfig {
            service_id: "test-service".to_string(),
            region_id: "test-region".to_string(),
            zone_id: "test-zone".to_string(),
            discovery_data: None,
        };

        let result = filter.filter(&mut service, &config).await;
        assert!(result.is_ok());
        assert_eq!(service.instances.len(), 2);
    }

    #[tokio::test]
    async fn test_status_filter_all_down() {
        let filter = StatusFilter;
        let mut service = create_test_service(vec![
            create_test_instance("inst-1", InstanceStatus::Down),
            create_test_instance("inst-2", InstanceStatus::Down),
        ]);
        let config = DiscoveryConfig {
            service_id: "test-service".to_string(),
            region_id: "test-region".to_string(),
            zone_id: "test-zone".to_string(),
            discovery_data: None,
        };

        let result = filter.filter(&mut service, &config).await;
        assert!(result.is_ok());
        assert_eq!(service.instances.len(), 0);
    }
}
