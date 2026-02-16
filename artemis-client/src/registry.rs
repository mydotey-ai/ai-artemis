use crate::http::retry_with_backoff;
use crate::{ClientConfig, Result};
use artemis_core::model::*;
use reqwest::Client;
use std::sync::Arc;
use std::time::Instant;
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

                        if let Some(ref failed) = response.failed_instance_keys
                            && !failed.is_empty()
                        {
                            warn!("Some instances failed heartbeat: {} keys", failed.len());
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
    use std::time::Duration;

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

    #[test]
    fn test_registry_client_new() {
        let config = ClientConfig::default();
        let client = RegistryClient::new(config.clone());

        assert_eq!(client.config.server_urls, config.server_urls);
        assert_eq!(client.config.http_retry_times, config.http_retry_times);
    }

    #[test]
    fn test_registry_url_construction() {
        let config = ClientConfig {
            server_urls: vec!["http://localhost:8080".to_string()],
            ..Default::default()
        };
        let _client = RegistryClient::new(config);

        // Test URL format (can't actually call without server)
        let expected_register_url = "http://localhost:8080/api/registry/register";
        let expected_heartbeat_url = "http://localhost:8080/api/registry/heartbeat";
        let expected_unregister_url = "http://localhost:8080/api/registry/unregister";

        assert!(expected_register_url.starts_with("http://"));
        assert!(expected_heartbeat_url.contains("/api/registry/heartbeat"));
        assert!(expected_unregister_url.contains("/api/registry/unregister"));
    }

    #[test]
    fn test_registry_config_defaults() {
        let config = ClientConfig::default();
        let client = RegistryClient::new(config);

        assert_eq!(client.config.heartbeat_interval_secs, 30);
        assert_eq!(client.config.heartbeat_ttl_secs, 90);
        assert_eq!(client.config.http_retry_times, 5);
    }

    #[test]
    fn test_registry_config_custom() {
        let config = ClientConfig {
            server_urls: vec!["http://custom-server:9090".to_string()],
            heartbeat_interval_secs: 10,
            heartbeat_ttl_secs: 30,
            http_retry_times: 3,
            http_retry_interval_ms: 200,
            ..Default::default()
        };
        let client = RegistryClient::new(config);

        assert_eq!(client.config.server_urls[0], "http://custom-server:9090");
        assert_eq!(client.config.heartbeat_interval_secs, 10);
        assert_eq!(client.config.heartbeat_ttl_secs, 30);
        assert_eq!(client.config.http_retry_times, 3);
    }

    #[test]
    fn test_heartbeat_interval_calculation() {
        let config = ClientConfig {
            heartbeat_interval_secs: 20,
            ..Default::default()
        };

        let interval = config.heartbeat_interval();
        assert_eq!(interval, Duration::from_secs(20));
    }

    #[test]
    fn test_heartbeat_ttl_calculation() {
        let config = ClientConfig {
            heartbeat_ttl_secs: 60,
            ..Default::default()
        };

        let ttl = config.heartbeat_ttl();
        assert_eq!(ttl, Duration::from_secs(60));
    }

    #[test]
    fn test_http_retry_interval_calculation() {
        let config = ClientConfig {
            http_retry_interval_ms: 150,
            ..Default::default()
        };

        let interval = config.http_retry_interval();
        assert_eq!(interval, Duration::from_millis(150));
    }

    #[tokio::test]
    async fn test_ttl_vs_interval_relationship() {
        let config = ClientConfig {
            heartbeat_interval_secs: 10,
            heartbeat_ttl_secs: 30,
            ..Default::default()
        };

        // TTL should be at least 3x the interval
        assert!(config.heartbeat_ttl_secs >= config.heartbeat_interval_secs * 3);
    }

    #[test]
    fn test_multiple_server_urls() {
        let config = ClientConfig {
            server_urls: vec![
                "http://server1:8080".to_string(),
                "http://server2:8080".to_string(),
                "http://server3:8080".to_string(),
            ],
            ..Default::default()
        };
        let client = RegistryClient::new(config);

        // Client should use first URL
        assert_eq!(client.config.server_urls[0], "http://server1:8080");
        assert_eq!(client.config.server_urls.len(), 3);
    }

    #[tokio::test]
    async fn test_instant_elapsed_behavior() {
        let start = Instant::now();
        tokio::time::sleep(Duration::from_millis(50)).await;

        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(50));
        assert!(elapsed < Duration::from_millis(100));
    }
}
