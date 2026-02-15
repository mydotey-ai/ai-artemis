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
        Self {
            service,
            cached_at: Instant::now(),
            ttl,
        }
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
        Self {
            config,
            client: Client::new(),
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_service(&self, request: GetServiceRequest) -> Result<Option<Service>> {
        let service_id = request.discovery_config.service_id.clone();

        // Check cache first
        {
            let cache = self.cache.read();
            if let Some(cached) = cache.get(&service_id) {
                if !cached.is_expired() {
                    return Ok(Some(cached.get().clone()));
                }
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
}
