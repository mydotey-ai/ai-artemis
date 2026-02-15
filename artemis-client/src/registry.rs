use crate::http::retry_with_backoff;
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
        let url = format!("{}/api/registry/register", self.config.server_urls[0]);
        let retry_times = self.config.http_retry_times as usize;
        let retry_interval = self.config.http_retry_interval();

        retry_with_backoff(retry_times, retry_interval, || {
            let url = url.clone();
            let request_clone = request.clone();
            let client = self.client.clone();
            async move {
                let response = client.post(&url).json(&request_clone).send().await?;
                Ok(response.json().await?)
            }
        })
        .await
    }

    pub async fn heartbeat(&self, request: HeartbeatRequest) -> Result<HeartbeatResponse> {
        let url = format!("{}/api/registry/heartbeat", self.config.server_urls[0]);
        let retry_times = self.config.http_retry_times as usize;
        let retry_interval = self.config.http_retry_interval();

        retry_with_backoff(retry_times, retry_interval, || {
            let url = url.clone();
            let request_clone = request.clone();
            let client = self.client.clone();
            async move {
                let response = client.post(&url).json(&request_clone).send().await?;
                Ok(response.json().await?)
            }
        })
        .await
    }

    pub async fn unregister(&self, request: UnregisterRequest) -> Result<UnregisterResponse> {
        let url = format!("{}/api/registry/unregister", self.config.server_urls[0]);
        let retry_times = self.config.http_retry_times as usize;
        let retry_interval = self.config.http_retry_interval();

        retry_with_backoff(retry_times, retry_interval, || {
            let url = url.clone();
            let request_clone = request.clone();
            let client = self.client.clone();
            async move {
                let response = client.post(&url).json(&request_clone).send().await?;
                Ok(response.json().await?)
            }
        })
        .await
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
