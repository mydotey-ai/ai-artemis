use crate::{ClientConfig, Result};
use artemis_core::model::*;
use parking_lot::RwLock;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Cached service entry with TTL tracking
#[derive(Debug, Clone)]
struct CachedService {
    service: Service,
    cached_at: Instant,
    ttl: Duration,
}

impl CachedService {
    fn new(service: Service, ttl: Duration) -> Self {
        Self { service, cached_at: Instant::now(), ttl }
    }

    fn is_expired(&self) -> bool {
        self.cached_at.elapsed() > self.ttl
    }

    #[allow(dead_code)]
    fn refresh(&mut self, service: Service) {
        self.service = service;
        self.cached_at = Instant::now();
    }

    fn get(&self) -> &Service {
        &self.service
    }
}

pub struct DiscoveryClient {
    config: ClientConfig,
    client: Client,
    cache: Arc<RwLock<HashMap<String, CachedService>>>,
}

impl DiscoveryClient {
    pub fn new(config: ClientConfig) -> Self {
        Self { config, client: Client::new(), cache: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub async fn get_service(&self, request: GetServiceRequest) -> Result<Option<Service>> {
        let service_id = request.discovery_config.service_id.clone();

        // Check cache first
        {
            let cache = self.cache.read();
            if let Some(cached) = cache.get(&service_id)
                && !cached.is_expired()
            {
                return Ok(Some(cached.get().clone()));
            }
        }

        // Cache expired or missing, fetch from server
        let url = format!("{}/api/discovery/service", self.config.server_urls[0]);
        let response = self.client.post(&url).json(&request).send().await?;
        let result: GetServiceResponse = response.json().await?;

        if let Some(ref service) = result.service {
            // Update cache
            let cached = CachedService::new(service.clone(), self.config.cache_ttl());
            self.cache.write().insert(service_id, cached);
        }

        Ok(result.service)
    }

    pub async fn get_services(&self) -> Result<Vec<Service>> {
        let url = format!("{}/api/discovery/services", self.config.server_urls[0]);
        let response = self.client.get(&url).send().await?;
        let result: GetServicesResponse = response.json().await?;

        // Update cache with all services
        let ttl = self.config.cache_ttl();
        let mut cache = self.cache.write();
        for service in &result.services {
            let cached = CachedService::new(service.clone(), ttl);
            cache.insert(service.service_id.clone(), cached);
        }

        Ok(result.services)
    }

    /// Batch query multiple services at once.
    ///
    /// Sends a single request with multiple discovery configs and returns
    /// all matching services. Results are cached with the configured TTL.
    pub async fn get_services_batch(&self, configs: Vec<DiscoveryConfig>) -> Result<Vec<Service>> {
        if configs.is_empty() {
            return Ok(Vec::new());
        }

        let url = format!("{}/api/discovery/lookup", self.config.server_urls[0]);
        let request = LookupServicesRequest { discovery_configs: configs };

        let resp = self.client.post(&url).json(&request).send().await?;
        let response: LookupServicesResponse = resp.json().await?;

        // Update cache with results
        let ttl = self.config.cache_ttl();
        let mut cache = self.cache.write();
        for service in &response.services {
            let cached = CachedService::new(service.clone(), ttl);
            cache.insert(service.service_id.clone(), cached);
        }

        Ok(response.services)
    }

    /// Get all cached services that have not expired
    pub fn get_cached_services(&self) -> Vec<Service> {
        self.cache
            .read()
            .values()
            .filter(|cached| !cached.is_expired())
            .map(|cached| cached.get().clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_service(service_id: &str) -> Service {
        Service {
            service_id: service_id.into(),
            instances: vec![],
            metadata: None,
            logic_instances: None,
            route_rules: None,
        }
    }

    #[test]
    fn test_batch_request_construction() {
        let configs = vec![
            DiscoveryConfig {
                service_id: "service1".into(),
                region_id: "region1".into(),
                zone_id: "zone1".into(),
                discovery_data: None,
            },
            DiscoveryConfig {
                service_id: "service2".into(),
                region_id: "region1".into(),
                zone_id: "zone1".into(),
                discovery_data: None,
            },
        ];

        let request = LookupServicesRequest { discovery_configs: configs };

        assert_eq!(request.discovery_configs.len(), 2);
        assert_eq!(request.discovery_configs[0].service_id, "service1");
        assert_eq!(request.discovery_configs[1].service_id, "service2");
    }

    #[test]
    fn test_cached_service_expiry() {
        let service = make_test_service("test");

        let cached = CachedService::new(service, Duration::from_secs(1));
        assert!(!cached.is_expired());

        std::thread::sleep(Duration::from_millis(1100));
        assert!(cached.is_expired());
    }

    #[test]
    fn test_cached_service_refresh() {
        let service = make_test_service("test");

        let mut cached = CachedService::new(service.clone(), Duration::from_secs(60));
        assert!(!cached.is_expired());

        // Refresh cache
        let new_service = make_test_service("test-updated");
        cached.refresh(new_service);
        assert!(!cached.is_expired());
        assert_eq!(cached.get().service_id, "test-updated");
    }

    #[test]
    fn test_cached_service_get() {
        let service = make_test_service("my-service");
        let cached = CachedService::new(service.clone(), Duration::from_secs(60));

        let retrieved = cached.get();
        assert_eq!(retrieved.service_id, "my-service");
    }

    #[test]
    fn test_cached_service_clone() {
        let service = make_test_service("test");
        let cached = CachedService::new(service, Duration::from_secs(60));

        let cloned = cached.clone();
        assert_eq!(cached.get().service_id, cloned.get().service_id);
    }

    #[test]
    fn test_cached_service_debug() {
        let service = make_test_service("debug-test");
        let cached = CachedService::new(service, Duration::from_secs(60));

        let debug_str = format!("{:?}", cached);
        assert!(debug_str.contains("CachedService"));
    }

    #[test]
    fn test_cached_service_zero_ttl() {
        let service = make_test_service("test");
        let cached = CachedService::new(service, Duration::from_secs(0));

        // With 0 TTL, should expire immediately
        std::thread::sleep(Duration::from_millis(1));
        assert!(cached.is_expired());
    }

    #[test]
    fn test_cached_service_long_ttl() {
        let service = make_test_service("test");
        let cached = CachedService::new(service, Duration::from_secs(3600)); // 1 hour

        assert!(!cached.is_expired());
    }

    #[test]
    fn test_discovery_client_new() {
        let config = ClientConfig::default();

        let client = DiscoveryClient::new(config.clone());
        assert_eq!(client.config.server_urls, config.server_urls);
    }

    #[test]
    fn test_get_cached_services_empty() {
        let config = ClientConfig::default();

        let client = DiscoveryClient::new(config);
        let cached = client.get_cached_services();
        assert_eq!(cached.len(), 0);
    }

    #[test]
    fn test_get_cached_services_with_valid_cache() {
        let config = ClientConfig::default();

        let client = DiscoveryClient::new(config);

        // 手动添加一个未过期的缓存项
        {
            let service = make_test_service("cached-service");
            let cached = CachedService::new(service, Duration::from_secs(60));
            client.cache.write().insert("cached-service".to_string(), cached);
        }

        let cached_services = client.get_cached_services();
        assert_eq!(cached_services.len(), 1);
        assert_eq!(cached_services[0].service_id, "cached-service");
    }

    #[test]
    fn test_get_cached_services_filters_expired() {
        let config = ClientConfig::default();

        let client = DiscoveryClient::new(config);

        // 添加一个已过期的缓存项
        {
            let service = make_test_service("expired-service");
            let cached = CachedService::new(service, Duration::from_secs(0));
            client.cache.write().insert("expired-service".to_string(), cached);
        }

        std::thread::sleep(Duration::from_millis(10));

        let cached_services = client.get_cached_services();
        assert_eq!(cached_services.len(), 0); // 过期的应该被过滤掉
    }

    #[test]
    fn test_get_cached_services_mixed() {
        let config = ClientConfig::default();

        let client = DiscoveryClient::new(config);

        // 添加一个有效的和一个过期的缓存项
        {
            let valid_service = make_test_service("valid-service");
            let valid_cached = CachedService::new(valid_service, Duration::from_secs(60));
            client.cache.write().insert("valid-service".to_string(), valid_cached);

            let expired_service = make_test_service("expired-service");
            let expired_cached = CachedService::new(expired_service, Duration::from_secs(0));
            client.cache.write().insert("expired-service".to_string(), expired_cached);
        }

        std::thread::sleep(Duration::from_millis(10));

        let cached_services = client.get_cached_services();
        assert_eq!(cached_services.len(), 1); // 只有有效的被返回
        assert_eq!(cached_services[0].service_id, "valid-service");
    }

    #[tokio::test]
    async fn test_get_services_batch_empty() {
        let config = ClientConfig::default();

        let client = DiscoveryClient::new(config);
        let result = client.get_services_batch(vec![]).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }
}
