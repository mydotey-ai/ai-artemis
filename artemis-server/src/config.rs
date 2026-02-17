use artemis_core::error::{ArtemisError, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArtemisConfig {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub cluster: ClusterConfig,
    #[serde(default)]
    pub replication: ReplicationConfig,
    #[serde(default)]
    pub lease: LeaseConfig,
    #[serde(default)]
    pub cache: CacheConfig,
    #[serde(default)]
    pub ratelimit: RateLimitConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database: Option<DatabaseConfig>,
    // 保留旧的 registry 字段以兼容
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry: Option<RegistryConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_node_id")]
    pub node_id: String,
    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,
    #[serde(default = "default_peer_port")]
    pub peer_port: u16,
    #[serde(default = "default_region")]
    pub region: String,
    #[serde(default = "default_zone")]
    pub zone: String,

    // 保留旧字段以兼容
    #[serde(skip)]
    pub host: String,
    #[serde(skip)]
    pub port: u16,
    #[serde(skip)]
    pub region_id: String,
    #[serde(skip)]
    pub zone_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClusterConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peers: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationConfig {
    #[serde(default = "default_replication_enabled")]
    pub enabled: bool,
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    #[serde(default = "default_batch_interval_ms")]
    pub batch_interval_ms: u64,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseConfig {
    #[serde(default = "default_ttl_secs")]
    pub ttl_secs: u64,
    #[serde(default = "default_cleanup_interval_secs")]
    pub cleanup_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    #[serde(default = "default_cache_enabled")]
    pub enabled: bool,
    #[serde(default = "default_expiry_secs")]
    pub expiry_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    #[serde(default = "default_ratelimit_enabled")]
    pub enabled: bool,
    #[serde(default = "default_requests_per_second")]
    pub requests_per_second: u32,
    #[serde(default = "default_burst_size")]
    pub burst_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_log_format")]
    pub format: String,
}

// 保留旧的 RegistryConfig 以兼容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    #[serde(with = "humantime_serde")]
    pub lease_ttl: Duration,
    #[serde(with = "humantime_serde")]
    pub eviction_interval: Duration,
    pub rate_limit_rps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// 数据库类型: sqlite, mysql
    #[serde(default = "default_db_type")]
    pub db_type: String,
    /// 数据库连接 URL
    /// SQLite: "sqlite://artemis.db" 或 "sqlite::memory:"
    /// MySQL: "mysql://user:password@localhost:3306/artemis"
    pub url: String,
    /// 最大连接数
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

fn default_db_type() -> String {
    "sqlite".to_string()
}
fn default_max_connections() -> u32 {
    10
}

// Default functions
fn default_node_id() -> String {
    uuid::Uuid::new_v4().to_string()
}
fn default_listen_addr() -> String {
    "0.0.0.0:8080".to_string()
}
fn default_peer_port() -> u16 {
    9090
}
fn default_region() -> String {
    "default".to_string()
}
fn default_zone() -> String {
    "default".to_string()
}

fn default_replication_enabled() -> bool {
    true
}
fn default_timeout_secs() -> u64 {
    5
}
fn default_batch_size() -> usize {
    100
}
fn default_batch_interval_ms() -> u64 {
    100
}
fn default_max_retries() -> u32 {
    3
}

fn default_ttl_secs() -> u64 {
    30
}
fn default_cleanup_interval_secs() -> u64 {
    60
}

fn default_cache_enabled() -> bool {
    true
}
fn default_expiry_secs() -> u64 {
    300
}

fn default_ratelimit_enabled() -> bool {
    true
}
fn default_requests_per_second() -> u32 {
    10000
}
fn default_burst_size() -> u32 {
    5000
}

fn default_log_level() -> String {
    "info".to_string()
}
fn default_log_format() -> String {
    "pretty".to_string()
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            node_id: default_node_id(),
            listen_addr: default_listen_addr(),
            peer_port: default_peer_port(),
            region: default_region(),
            zone: default_zone(),
            host: "0.0.0.0".to_string(),
            port: 8080,
            region_id: "default".to_string(),
            zone_id: "default".to_string(),
        }
    }
}

impl Default for ReplicationConfig {
    fn default() -> Self {
        Self {
            enabled: default_replication_enabled(),
            timeout_secs: default_timeout_secs(),
            batch_size: default_batch_size(),
            batch_interval_ms: default_batch_interval_ms(),
            max_retries: default_max_retries(),
        }
    }
}

impl Default for LeaseConfig {
    fn default() -> Self {
        Self {
            ttl_secs: default_ttl_secs(),
            cleanup_interval_secs: default_cleanup_interval_secs(),
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self { enabled: default_cache_enabled(), expiry_secs: default_expiry_secs() }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: default_ratelimit_enabled(),
            requests_per_second: default_requests_per_second(),
            burst_size: default_burst_size(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self { level: default_log_level(), format: default_log_format() }
    }
}

impl ArtemisConfig {
    /// Load configuration from a TOML file
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            ArtemisError::Configuration(format!("Failed to read config file {}: {}", path, e))
        })?;

        let mut config: ArtemisConfig = toml::from_str(&content)
            .map_err(|e| ArtemisError::Configuration(format!("Failed to parse TOML: {}", e)))?;

        // 填充兼容字段
        config.server.host =
            config.server.listen_addr.split(':').next().unwrap_or("0.0.0.0").to_string();
        config.server.port = config
            .server
            .listen_addr
            .split(':')
            .nth(1)
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);
        config.server.region_id = config.server.region.clone();
        config.server.zone_id = config.server.zone.clone();

        Ok(config)
    }

    /// Get listen socket address
    pub fn listen_addr(&self) -> std::net::SocketAddr {
        self.server.listen_addr.parse().unwrap_or_else(|_| "0.0.0.0:8080".parse().unwrap())
    }
}
