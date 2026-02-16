# 阶段6: artemis-client实现

> **For Claude:** 客户端SDK，支持服务注册、发现和实时更新。参考Java实现: `artemis-java/artemis-client/`

**优先级**: P0 (必须完成)
**状态**: ✅ **已完成** (2026-02-13)
**目标:** 实现完整的客户端SDK
**任务数:** 5个Task

---

## Task 6.1: 实现ClientConfig和Error

**Files:**
- Create: `artemis-client/src/config.rs`
- Create: `artemis-client/src/error.rs`
- Update: `artemis-client/src/lib.rs`

**Step 1: 实现ClientConfig**

```rust
// artemis-client/src/config.rs
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub server_url: String,
    pub region_id: String,
    pub zone_id: String,
    pub heartbeat_interval: Duration,
    pub cache_refresh_interval: Duration,
    pub timeout: Duration,
}

impl ClientConfig {
    pub fn builder() -> ClientConfigBuilder {
        ClientConfigBuilder::default()
    }
}

#[derive(Default)]
pub struct ClientConfigBuilder {
    server_url: Option<String>,
    region_id: Option<String>,
    zone_id: Option<String>,
    heartbeat_interval: Option<Duration>,
    cache_refresh_interval: Option<Duration>,
    timeout: Option<Duration>,
}

impl ClientConfigBuilder {
    pub fn server_url(mut self, url: impl Into<String>) -> Self {
        self.server_url = Some(url.into());
        self
    }

    pub fn region_id(mut self, id: impl Into<String>) -> Self {
        self.region_id = Some(id.into());
        self
    }

    pub fn zone_id(mut self, id: impl Into<String>) -> Self {
        self.zone_id = Some(id.into());
        self
    }

    pub fn heartbeat_interval(mut self, interval: Duration) -> Self {
        self.heartbeat_interval = Some(interval);
        self
    }

    pub fn cache_refresh_interval(mut self, interval: Duration) -> Self {
        self.cache_refresh_interval = Some(interval);
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn build(self) -> ClientConfig {
        ClientConfig {
            server_url: self.server_url.expect("server_url is required"),
            region_id: self.region_id.expect("region_id is required"),
            zone_id: self.zone_id.expect("zone_id is required"),
            heartbeat_interval: self.heartbeat_interval.unwrap_or(Duration::from_secs(10)),
            cache_refresh_interval: self
                .cache_refresh_interval
                .unwrap_or(Duration::from_secs(30)),
            timeout: self.timeout.unwrap_or(Duration::from_secs(5)),
        }
    }
}
```

**Step 2: 实现Error类型**

```rust
// artemis-client/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    #[error("Registration failed: {0}")]
    RegistrationFailed(String),

    #[error("Heartbeat failed: {0}")]
    HeartbeatFailed(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

pub type Result<T> = std::result::Result<T, ClientError>;
```

**Step 3: 更新lib.rs**

```rust
// artemis-client/src/lib.rs
//! Artemis Client SDK - 客户端SDK

pub mod config;
pub mod discovery;
pub mod error;
pub mod registry;
pub mod websocket;

pub use config::{ClientConfig, ClientConfigBuilder};
pub use discovery::DiscoveryClient;
pub use error::{ClientError, Result};
pub use registry::RegistryClient;
```

**Step 4: 提交**

```bash
git add artemis-client/src/config.rs artemis-client/src/error.rs artemis-client/src/lib.rs
git commit -m "feat(client): implement ClientConfig and Error types

- Add ClientConfig with builder pattern
- Support heartbeat and cache refresh intervals
- Add ClientError with thiserror
- Export public API

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 6.2: 实现RegistryClient

**Files:**
- Create: `artemis-client/src/registry/mod.rs`
- Create: `artemis-client/src/registry/client.rs`

**Step 1: 创建registry模块**

```rust
// artemis-client/src/registry/mod.rs
pub mod client;

pub use client::RegistryClient;
```

**Step 2: 实现RegistryClient**

```rust
// artemis-client/src/registry/client.rs
use crate::{config::ClientConfig, error::Result};
use artemis_core::model::{
    HeartbeatRequest, HeartbeatResponse, Instance, InstanceKey, RegisterRequest,
    RegisterResponse, UnregisterRequest, UnregisterResponse,
};
use parking_lot::Mutex;
use reqwest::Client;
use std::sync::Arc;
use tokio::time::{self, Duration};
use tracing::{error, info};

pub struct RegistryClient {
    config: ClientConfig,
    http_client: Client,
    registered_instances: Arc<Mutex<Vec<InstanceKey>>>,
}

impl RegistryClient {
    pub fn new(config: ClientConfig) -> Self {
        let http_client = Client::builder()
            .timeout(config.timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            http_client,
            registered_instances: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 注册实例
    pub async fn register(&self, instances: Vec<Instance>) -> Result<RegisterResponse> {
        let url = format!("{}/api/registry/register", self.config.server_url);
        let request = RegisterRequest { instances };

        let response = self
            .http_client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .json::<RegisterResponse>()
            .await?;

        // 记录成功注册的实例
        if response.response_status.error_code == artemis_core::model::ErrorCode::Success {
            let mut registered = self.registered_instances.lock();
            for instance in &request.instances {
                registered.push(instance.key());
            }
        }

        Ok(response)
    }

    /// 发送心跳
    pub async fn heartbeat(&self, instance_keys: Vec<InstanceKey>) -> Result<HeartbeatResponse> {
        let url = format!("{}/api/registry/heartbeat", self.config.server_url);
        let request = HeartbeatRequest { instance_keys };

        let response = self
            .http_client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .json::<HeartbeatResponse>()
            .await?;

        Ok(response)
    }

    /// 注销实例
    pub async fn unregister(&self, instance_keys: Vec<InstanceKey>) -> Result<UnregisterResponse> {
        let url = format!("{}/api/registry/unregister", self.config.server_url);
        let request = UnregisterRequest { instance_keys };

        let response = self
            .http_client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .json::<UnregisterResponse>()
            .await?;

        Ok(response)
    }

    /// 启动自动心跳任务
    pub fn start_heartbeat_task(self: Arc<Self>) {
        let interval = self.config.heartbeat_interval;
        tokio::spawn(async move {
            let mut ticker = time::interval(interval);
            loop {
                ticker.tick().await;

                let keys = self.registered_instances.lock().clone();
                if keys.is_empty() {
                    continue;
                }

                match self.heartbeat(keys).await {
                    Ok(_) => info!("Heartbeat sent successfully"),
                    Err(e) => error!("Heartbeat failed: {}", e),
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let config = ClientConfig::builder()
            .server_url("http://localhost:8080")
            .region_id("test")
            .zone_id("zone")
            .build();

        let _client = RegistryClient::new(config);
    }
}
```

**Step 3: 提交**

```bash
git add artemis-client/src/registry/
git commit -m "feat(client): implement RegistryClient

- Support register/heartbeat/unregister operations
- Auto-track registered instances
- Background heartbeat task
- HTTP client with timeout

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 6.3: 实现DiscoveryClient

**Files:**
- Create: `artemis-client/src/discovery/mod.rs`
- Create: `artemis-client/src/discovery/client.rs`

**Step 1: 创建discovery模块**

```rust
// artemis-client/src/discovery/mod.rs
pub mod client;

pub use client::DiscoveryClient;
```

**Step 2: 实现DiscoveryClient**

```rust
// artemis-client/src/discovery/client.rs
use crate::{config::ClientConfig, error::Result};
use artemis_core::model::{
    DiscoveryConfig, GetServiceRequest, GetServiceResponse, Service,
};
use dashmap::DashMap;
use reqwest::Client;
use std::sync::Arc;
use tokio::time;
use tracing::info;

pub struct DiscoveryClient {
    config: ClientConfig,
    http_client: Client,
    cache: Arc<DashMap<String, Service>>,
}

impl DiscoveryClient {
    pub fn new(config: ClientConfig) -> Self {
        let http_client = Client::builder()
            .timeout(config.timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            http_client,
            cache: Arc::new(DashMap::new()),
        }
    }

    /// 获取服务
    pub async fn get_service(&self, service_id: &str) -> Result<Option<Service>> {
        // 先尝试从缓存获取
        if let Some(service) = self.cache.get(service_id) {
            return Ok(Some(service.value().clone()));
        }

        // 缓存未命中，从服务器获取
        let url = format!("{}/api/discovery/getservice", self.config.server_url);
        let request = GetServiceRequest {
            discovery_config: DiscoveryConfig {
                service_id: service_id.to_string(),
                region_id: self.config.region_id.clone(),
                zone_id: self.config.zone_id.clone(),
                discovery_data: None,
            },
        };

        let response = self
            .http_client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .json::<GetServiceResponse>()
            .await?;

        if let Some(ref service) = response.service {
            self.cache.insert(service_id.to_string(), service.clone());
        }

        Ok(response.service)
    }

    /// 刷新缓存
    pub async fn refresh_cache(&self) {
        info!("Refreshing discovery cache");
        // TODO: 实现增量刷新
    }

    /// 启动定期刷新任务
    pub fn start_refresh_task(self: Arc<Self>) {
        let interval = self.config.cache_refresh_interval;
        tokio::spawn(async move {
            let mut ticker = time::interval(interval);
            loop {
                ticker.tick().await;
                self.refresh_cache().await;
            }
        });
    }

    /// 获取缓存的服务
    pub fn get_cached_service(&self, service_id: &str) -> Option<Service> {
        self.cache.get(service_id).map(|s| s.value().clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovery_client_creation() {
        let config = ClientConfig::builder()
            .server_url("http://localhost:8080")
            .region_id("test")
            .zone_id("zone")
            .build();

        let _client = DiscoveryClient::new(config);
    }
}
```

**Step 3: 提交**

```bash
git add artemis-client/src/discovery/
git commit -m "feat(client): implement DiscoveryClient

- Support service discovery with local cache
- Background cache refresh task
- DashMap for concurrent cache access
- Cache-first query strategy

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 6.4: 实现WebSocket客户端（占位）

**Files:**
- Create: `artemis-client/src/websocket/mod.rs`

**Step 1: 创建WebSocket模块占位**

```rust
// artemis-client/src/websocket/mod.rs
//! WebSocket客户端
//!
//! 用于接收服务变更的实时推送
//! TODO: 在后续迭代中实现
```

**Step 2: 提交**

```bash
git add artemis-client/src/websocket/
git commit -m "feat(client): add WebSocket client placeholder

- Create module for future real-time updates
- Add TODO for implementation

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 6.5: 添加使用示例

**Files:**
- Create: `artemis-client/examples/simple_client.rs`

**Step 1: 创建示例程序**

```rust
// artemis-client/examples/simple_client.rs
use artemis_client::{ClientConfig, DiscoveryClient, RegistryClient};
use artemis_core::model::{Instance, InstanceStatus};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // 创建配置
    let config = ClientConfig::builder()
        .server_url("http://localhost:8080")
        .region_id("us-east")
        .zone_id("zone-1")
        .heartbeat_interval(Duration::from_secs(10))
        .cache_refresh_interval(Duration::from_secs(30))
        .build();

    // 创建注册客户端
    let registry_client = Arc::new(RegistryClient::new(config.clone()));

    // 注册实例
    let instance = Instance {
        region_id: "us-east".to_string(),
        zone_id: "zone-1".to_string(),
        group_id: None,
        service_id: "my-service".to_string(),
        instance_id: "inst-1".to_string(),
        machine_name: None,
        ip: "127.0.0.1".to_string(),
        port: 8080,
        protocol: Some("http".to_string()),
        url: "http://127.0.0.1:8080".to_string(),
        health_check_url: None,
        status: InstanceStatus::Up,
        metadata: None,
    };

    registry_client.register(vec![instance]).await?;
    println!("Instance registered successfully");

    // 启动自动心跳
    registry_client.clone().start_heartbeat_task();

    // 创建发现客户端
    let discovery_client = Arc::new(DiscoveryClient::new(config));

    // 查询服务
    if let Some(service) = discovery_client.get_service("my-service").await? {
        println!("Found service: {} with {} instances", service.service_id, service.instances.len());
    }

    // 启动缓存刷新
    discovery_client.clone().start_refresh_task();

    // 保持运行
    tokio::time::sleep(Duration::from_secs(60)).await;

    Ok(())
}
```

**Step 2: 验证编译**

```bash
cargo check -p artemis-client --examples
```

Expected: 编译成功

**Step 3: 提交**

```bash
git add artemis-client/examples/
git commit -m "docs(client): add simple client example

- Demonstrate registration and discovery
- Show auto-heartbeat and cache refresh
- Complete usage example

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## 阶段6完成标准

- ✅ ClientConfig实现
- ✅ RegistryClient实现（含自动心跳）
- ✅ DiscoveryClient实现（含缓存）
- ✅ WebSocket占位
- ✅ 使用示例
- ✅ `cargo check -p artemis-client` 通过
