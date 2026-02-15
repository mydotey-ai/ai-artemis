use crate::http::retry_with_backoff;
use crate::{ClientConfig, Result};
use artemis_core::model::*;
use reqwest::Client;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time;
use tracing::{error, info, warn};

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

    /// Start a background heartbeat task with TTL checking.
    ///
    /// Tracks the last successful heartbeat time. If the elapsed time since
    /// the last success exceeds the configured TTL, an error is logged but
    /// the task continues retrying to allow recovery.
    pub fn start_heartbeat_task(self: Arc<Self>, keys: Vec<InstanceKey>) {
        let heartbeat_interval = self.config.heartbeat_interval();
        let heartbeat_ttl = self.config.heartbeat_ttl();

        tokio::spawn(async move {
            let mut interval = time::interval(heartbeat_interval);
            let mut last_success = Instant::now();

            loop {
                interval.tick().await;

                // TTL check: log error if exceeded, but keep trying
                if last_success.elapsed() > heartbeat_ttl {
                    error!(
                        "Heartbeat TTL exceeded ({:?} since last success), connection may be broken",
                        last_success.elapsed()
                    );
                }

                let request = HeartbeatRequest { instance_keys: keys.clone() };

                match self.heartbeat(request).await {
                    Ok(response) => {
                        last_success = Instant::now();
                        info!("Heartbeat successful");

                        if let Some(ref failed) = response.failed_instance_keys {
                            if !failed.is_empty() {
                                warn!("Some instances failed heartbeat: {} keys", failed.len());
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Heartbeat request failed: {}", e);

                        if last_success.elapsed() > heartbeat_ttl {
                            error!(
                                "Heartbeat has been failing for {:?}, instances may expire",
                                last_success.elapsed()
                            );
                        }
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_heartbeat_ttl_check() {
        let config = ClientConfig {
            heartbeat_interval_secs: 1,
            heartbeat_ttl_secs: 3,
            ..Default::default()
        };

        // Test TTL timing logic
        let start = Instant::now();
        let ttl = config.heartbeat_ttl();

        tokio::time::sleep(Duration::from_millis(100)).await;

        // Should not have exceeded TTL yet
        assert!(start.elapsed() < ttl);

        // Simulate TTL exceeded scenario
        let old_start = Instant::now() - Duration::from_secs(4);
        assert!(old_start.elapsed() > ttl);
    }
}
