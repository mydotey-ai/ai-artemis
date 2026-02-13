use crate::{ClientConfig, Result};
use artemis_core::model::*;
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

pub struct RegistryClient {
    config: ClientConfig,
    client: Client,
}

impl RegistryClient {
    pub fn new(config: ClientConfig) -> Self {
        Self { config, client: Client::new() }
    }

    pub async fn register(&self, request: RegisterRequest) -> Result<RegisterResponse> {
        let url = format!("{}/api/registry/register", self.config.server_url);
        let response = self.client.post(&url).json(&request).send().await?;
        Ok(response.json().await?)
    }

    pub async fn heartbeat(&self, request: HeartbeatRequest) -> Result<HeartbeatResponse> {
        let url = format!("{}/api/registry/heartbeat", self.config.server_url);
        let response = self.client.post(&url).json(&request).send().await?;
        Ok(response.json().await?)
    }

    pub async fn unregister(&self, request: UnregisterRequest) -> Result<UnregisterResponse> {
        let url = format!("{}/api/registry/unregister", self.config.server_url);
        let response = self.client.post(&url).json(&request).send().await?;
        Ok(response.json().await?)
    }

    pub fn start_heartbeat_task(self: Arc<Self>, keys: Vec<InstanceKey>) {
        tokio::spawn(async move {
            let mut interval =
                time::interval(Duration::from_secs(self.config.heartbeat_interval_secs));
            loop {
                interval.tick().await;
                let request = HeartbeatRequest { instance_keys: keys.clone() };
                if let Err(e) = self.heartbeat(request).await {
                    tracing::warn!("Heartbeat failed: {}", e);
                }
            }
        });
    }
}
