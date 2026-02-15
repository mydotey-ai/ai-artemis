use std::time::Duration;

/// Client configuration for Artemis service discovery
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// List of Artemis server URLs (for failover and load balancing)
    pub server_urls: Vec<String>,

    /// Heartbeat interval in seconds (default: 30s)
    pub heartbeat_interval_secs: u64,

    /// Heartbeat TTL in seconds (must be >= 3x heartbeat_interval, default: 90s)
    pub heartbeat_ttl_secs: u64,

    /// HTTP request retry times (default: 5)
    pub http_retry_times: u32,

    /// HTTP request retry interval in milliseconds (default: 100ms)
    pub http_retry_interval_ms: u64,

    /// WebSocket ping interval in seconds (default: 30s)
    pub websocket_ping_interval_secs: u64,

    /// Local cache TTL in seconds (default: 900s = 15min)
    pub cache_ttl_secs: u64,

    /// Server address refresh interval in seconds (default: 60s)
    pub address_refresh_interval_secs: u64,

    /// Enable Prometheus metrics (default: false)
    pub enable_metrics: bool,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            server_urls: vec!["http://localhost:8080".to_string()],
            heartbeat_interval_secs: 30,
            heartbeat_ttl_secs: 90,
            http_retry_times: 5,
            http_retry_interval_ms: 100,
            websocket_ping_interval_secs: 30,
            cache_ttl_secs: 900,
            address_refresh_interval_secs: 60,
            enable_metrics: false,
        }
    }
}

impl ClientConfig {
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate server URLs
        if self.server_urls.is_empty() {
            return Err("At least one server URL must be provided".to_string());
        }

        // Validate heartbeat TTL (must be at least 3x heartbeat interval)
        if self.heartbeat_ttl_secs < self.heartbeat_interval_secs * 3 {
            return Err(format!(
                "TTL must be at least 3x heartbeat interval (got TTL={}, interval={})",
                self.heartbeat_ttl_secs, self.heartbeat_interval_secs
            ));
        }

        // Validate retry configuration
        if self.http_retry_times == 0 {
            return Err("HTTP retry times must be greater than 0".to_string());
        }

        Ok(())
    }

    /// Get heartbeat interval as Duration
    pub fn heartbeat_interval(&self) -> Duration {
        Duration::from_secs(self.heartbeat_interval_secs)
    }

    /// Get heartbeat TTL as Duration
    pub fn heartbeat_ttl(&self) -> Duration {
        Duration::from_secs(self.heartbeat_ttl_secs)
    }

    /// Get HTTP retry interval as Duration
    pub fn http_retry_interval(&self) -> Duration {
        Duration::from_millis(self.http_retry_interval_ms)
    }

    /// Get WebSocket ping interval as Duration
    pub fn websocket_ping_interval(&self) -> Duration {
        Duration::from_secs(self.websocket_ping_interval_secs)
    }

    /// Get cache TTL as Duration
    pub fn cache_ttl(&self) -> Duration {
        Duration::from_secs(self.cache_ttl_secs)
    }

    /// Get address refresh interval as Duration
    pub fn address_refresh_interval(&self) -> Duration {
        Duration::from_secs(self.address_refresh_interval_secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ClientConfig::default();
        assert_eq!(config.server_urls.len(), 1);
        assert_eq!(config.server_urls[0], "http://localhost:8080");
        assert_eq!(config.heartbeat_interval_secs, 30);
        assert_eq!(config.heartbeat_ttl_secs, 90);
        assert_eq!(config.http_retry_times, 5);
        assert_eq!(config.http_retry_interval_ms, 100);
        assert_eq!(config.websocket_ping_interval_secs, 30);
        assert_eq!(config.cache_ttl_secs, 900);
        assert_eq!(config.address_refresh_interval_secs, 60);
    }

    #[test]
    fn test_custom_config() {
        let config = ClientConfig {
            server_urls: vec!["http://node1:8080".into(), "http://node2:8080".into()],
            heartbeat_interval_secs: 10,
            heartbeat_ttl_secs: 30,
            http_retry_times: 3,
            http_retry_interval_ms: 200,
            websocket_ping_interval_secs: 60,
            cache_ttl_secs: 600,
            address_refresh_interval_secs: 120,
            enable_metrics: true,
        };
        assert_eq!(config.server_urls.len(), 2);
        assert!(config.enable_metrics);
    }

    #[test]
    fn test_validation() {
        let config = ClientConfig {
            heartbeat_ttl_secs: 20,
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("TTL must be at least 3x heartbeat interval"));
    }
}
