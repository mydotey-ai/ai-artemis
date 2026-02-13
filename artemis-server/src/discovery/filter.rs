use artemis_core::model::{DiscoveryConfig, InstanceStatus, Service};
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
