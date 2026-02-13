use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtemisConfig {
    pub server: ServerConfig,
    pub registry: RegistryConfig,
    pub cluster: ClusterConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database: Option<DatabaseConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub region_id: String,
    pub zone_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    #[serde(with = "humantime_serde")]
    pub lease_ttl: Duration,
    #[serde(with = "humantime_serde")]
    pub eviction_interval: Duration,
    pub rate_limit_rps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_nodes: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

impl Default for ArtemisConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                region_id: "default".to_string(),
                zone_id: "default".to_string(),
            },
            registry: RegistryConfig {
                lease_ttl: Duration::from_secs(30),
                eviction_interval: Duration::from_secs(10),
                rate_limit_rps: 1000,
            },
            cluster: ClusterConfig { enabled: false, peer_nodes: None },
            database: None,
        }
    }
}
