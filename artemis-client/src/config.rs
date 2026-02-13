#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub server_url: String,
    pub heartbeat_interval_secs: u64,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self { server_url: "http://localhost:8080".to_string(), heartbeat_interval_secs: 30 }
    }
}
