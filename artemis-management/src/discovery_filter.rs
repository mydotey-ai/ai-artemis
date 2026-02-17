//! Discovery filters for management features

use artemis_core::model::{DiscoveryConfig, Service};
use artemis_server::discovery::filter::{DiscoveryFilter, Result};
use async_trait::async_trait;
use std::sync::Arc;

use crate::routing::{RouteContext, RouteEngine};
use crate::{InstanceManager, RouteManager};

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
