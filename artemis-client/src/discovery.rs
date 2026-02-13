use crate::{ClientConfig, Result};
use artemis_core::model::*;
use parking_lot::RwLock;
use reqwest::Client;
use std::sync::Arc;

pub struct DiscoveryClient {
    config: ClientConfig,
    client: Client,
    cache: Arc<RwLock<Vec<Service>>>,
}

impl DiscoveryClient {
    pub fn new(config: ClientConfig) -> Self {
        Self { config, client: Client::new(), cache: Arc::new(RwLock::new(Vec::new())) }
    }

    pub async fn get_service(&self, request: GetServiceRequest) -> Result<Option<Service>> {
        let url = format!("{}/api/discovery/service", self.config.server_url);
        let response = self.client.post(&url).json(&request).send().await?;
        let result: GetServiceResponse = response.json().await?;
        Ok(result.service)
    }

    pub async fn get_services(&self) -> Result<Vec<Service>> {
        let url = format!("{}/api/discovery/services", self.config.server_url);
        let response = self.client.get(&url).send().await?;
        let result: GetServicesResponse = response.json().await?;
        *self.cache.write() = result.services.clone();
        Ok(result.services)
    }

    pub fn get_cached_services(&self) -> Vec<Service> {
        self.cache.read().clone()
    }
}
