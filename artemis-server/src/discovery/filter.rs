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
