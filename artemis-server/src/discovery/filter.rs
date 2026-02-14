use artemis_core::model::{DiscoveryConfig, InstanceStatus, Service};
use artemis_management::InstanceManager;
use async_trait::async_trait;
use std::sync::Arc;

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
            if self
                .instance_manager
                .is_server_down(&inst.ip, &inst.region_id)
            {
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
