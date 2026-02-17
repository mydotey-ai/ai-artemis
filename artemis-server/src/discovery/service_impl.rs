use super::filter::{DiscoveryFilterChain, StatusFilter};
use crate::cache::VersionedCacheManager;
use crate::registry::RegistryRepository;
use crate::traits::DiscoveryService;
use artemis_core::model::{
    ErrorCode, GetServiceRequest, GetServiceResponse, GetServicesDeltaRequest,
    GetServicesDeltaResponse, GetServicesRequest, GetServicesResponse, ResponseStatus, Service,
};
use async_trait::async_trait;
use std::sync::Arc;

#[derive(Clone)]
pub struct DiscoveryServiceImpl {
    repository: RegistryRepository,
    cache: Arc<VersionedCacheManager>,
    filter_chain: DiscoveryFilterChain,
}

impl DiscoveryServiceImpl {
    pub fn new(repository: RegistryRepository, cache: Arc<VersionedCacheManager>) -> Self {
        let mut filter_chain = DiscoveryFilterChain::new();
        filter_chain.add_filter(Arc::new(StatusFilter));

        Self { repository, cache, filter_chain }
    }

    /// 添加过滤器到过滤链
    pub fn add_filter(&mut self, filter: Arc<dyn super::filter::DiscoveryFilter>) {
        self.filter_chain.add_filter(filter);
    }

    fn build_service(&self, service_id: &str) -> Option<Service> {
        let instances = self.repository.get_instances_by_service(service_id);
        if instances.is_empty() {
            None
        } else {
            Some(Service {
                service_id: service_id.to_string(),
                metadata: None,
                instances,
                logic_instances: None,
            })
        }
    }

    pub fn refresh_cache(&self) {
        let all_instances = self.repository.get_all_instances();

        // 使用HashSet去重，避免排序和dedup操作
        let service_ids: std::collections::HashSet<String> =
            all_instances.iter().map(|inst| inst.service_id.to_lowercase()).collect();

        for service_id in service_ids {
            if let Some(service) = self.build_service(&service_id) {
                self.cache.update_service(service);
            }
        }
    }
}

#[async_trait]
impl DiscoveryService for DiscoveryServiceImpl {
    async fn get_service(&self, request: GetServiceRequest) -> GetServiceResponse {
        let service_id = request.discovery_config.service_id.to_lowercase();

        if let Some(mut service) = self.cache.get_service(&service_id) {
            if let Err(e) = self.filter_chain.apply(&mut service, &request.discovery_config).await {
                tracing::warn!("Filter failed: {}", e);
            }
            return GetServiceResponse {
                response_status: ResponseStatus::success(),
                service: Some(service),
            };
        }

        match self.build_service(&service_id) {
            Some(mut service) => {
                if let Err(e) =
                    self.filter_chain.apply(&mut service, &request.discovery_config).await
                {
                    tracing::warn!("Filter failed: {}", e);
                }
                // 先更新缓存，避免在响应中克隆
                let service_for_cache = service.clone();
                self.cache.update_service(service_for_cache);
                GetServiceResponse {
                    response_status: ResponseStatus::success(),
                    service: Some(service),
                }
            }
            None => GetServiceResponse {
                response_status: ResponseStatus::error(
                    ErrorCode::BadRequest,
                    format!("Service not found: {}", service_id),
                ),
                service: None,
            },
        }
    }

    async fn get_services(&self, _request: GetServicesRequest) -> GetServicesResponse {
        let services = self.cache.get_all_services();
        GetServicesResponse { response_status: ResponseStatus::success(), services }
    }

    async fn get_services_delta(
        &self,
        request: GetServicesDeltaRequest,
    ) -> GetServicesDeltaResponse {
        let current_version = self.cache.get_version();

        if request.since_timestamp >= current_version {
            return GetServicesDeltaResponse {
                response_status: ResponseStatus::success(),
                services: vec![],
                current_timestamp: current_version,
            };
        }

        let services = self.cache.get_all_services();
        GetServicesDeltaResponse {
            response_status: ResponseStatus::success(),
            services,
            current_timestamp: current_version,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use artemis_core::model::{DiscoveryConfig, Instance, InstanceStatus};

    fn create_test_instance(service_id: &str) -> Instance {
        Instance {
            region_id: "test".to_string(),
            zone_id: "zone".to_string(),
            group_id: None,
            service_id: service_id.to_string(),
            instance_id: "inst-1".to_string(),
            machine_name: None,
            ip: "127.0.0.1".to_string(),
            port: 8080,
            protocol: None,
            url: "http://127.0.0.1:8080".to_string(),
            health_check_url: None,
            status: InstanceStatus::Up,
            metadata: None,
        }
    }

    #[tokio::test]
    async fn test_get_service() {
        let repo = RegistryRepository::new();
        let cache = Arc::new(VersionedCacheManager::new());
        let service = DiscoveryServiceImpl::new(repo.clone(), cache);

        repo.register(create_test_instance("my-service"));

        let request = GetServiceRequest {
            discovery_config: DiscoveryConfig {
                service_id: "my-service".to_string(),
                region_id: "test".to_string(),
                zone_id: "zone".to_string(),
                discovery_data: None,
            },
        };

        let response = service.get_service(request).await;
        assert_eq!(response.response_status.error_code, ErrorCode::Success);
        assert!(response.service.is_some());
    }
}
