# 阶段2: artemis-core实现

> **For Claude:** 核心数据模型、trait定义、错误处理。这是整个系统的基础。

**目标:** 实现所有核心数据模型、trait接口和错误类型

**预计任务数:** 5个Task

---

## Task 2.1: 实现核心数据模型 - Instance

**Files:**
- Create: `artemis-core/src/model/mod.rs`
- Create: `artemis-core/src/model/instance.rs`

**Step 1: 创建model模块**

```rust
// artemis-core/src/model/mod.rs
pub mod instance;
pub mod service;
pub mod lease;
pub mod route;
pub mod change;
pub mod request;

pub use instance::{Instance, InstanceKey, InstanceStatus};
pub use service::{Service, ServiceGroup};
pub use lease::Lease;
pub use route::{RouteRule, RouteStrategy};
pub use change::{InstanceChange, ChangeType};
pub use request::*;
```

**Step 2: 实现Instance模型**

```rust
// artemis-core/src/model/instance.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Instance {
    pub region_id: String,
    pub zone_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    pub service_id: String,
    pub instance_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub machine_name: Option<String>,
    pub ip: String,
    pub port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_check_url: Option<String>,
    pub status: InstanceStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum InstanceStatus {
    Starting,
    Up,
    Down,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstanceKey {
    pub region_id: String,
    pub zone_id: String,
    pub service_id: String,
    pub group_id: String,
    pub instance_id: String,
}

impl Instance {
    pub fn key(&self) -> InstanceKey {
        InstanceKey {
            region_id: self.region_id.clone(),
            zone_id: self.zone_id.clone(),
            service_id: self.service_id.to_lowercase(),
            group_id: self.group_id.clone().unwrap_or_default(),
            instance_id: self.instance_id.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_key_generation() {
        let instance = Instance {
            region_id: "us-east".to_string(),
            zone_id: "zone-1".to_string(),
            group_id: Some("group-a".to_string()),
            service_id: "MyService".to_string(),
            instance_id: "inst-1".to_string(),
            machine_name: None,
            ip: "192.168.1.1".to_string(),
            port: 8080,
            protocol: Some("http".to_string()),
            url: "http://192.168.1.1:8080".to_string(),
            health_check_url: None,
            status: InstanceStatus::Up,
            metadata: None,
        };

        let key = instance.key();
        assert_eq!(key.service_id, "myservice"); // 转小写
        assert_eq!(key.group_id, "group-a");
    }

    #[test]
    fn test_instance_serde() {
        let instance = Instance {
            region_id: "us-east".to_string(),
            zone_id: "zone-1".to_string(),
            group_id: None,
            service_id: "test-service".to_string(),
            instance_id: "inst-1".to_string(),
            machine_name: None,
            ip: "192.168.1.1".to_string(),
            port: 8080,
            protocol: None,
            url: "http://192.168.1.1:8080".to_string(),
            health_check_url: None,
            status: InstanceStatus::Up,
            metadata: None,
        };

        let json = serde_json::to_string(&instance).unwrap();
        let deserialized: Instance = serde_json::from_str(&json).unwrap();
        assert_eq!(instance, deserialized);
    }
}
```

**Step 3: 运行测试**

```bash
cargo test -p artemis-core
```

Expected: 2 tests passed

**Step 4: 提交**

```bash
git add artemis-core/src/model/
git commit -m "feat(core): implement Instance model with tests

- Add Instance struct with all fields
- Add InstanceStatus enum
- Add InstanceKey for hash-based lookups
- Service ID is case-insensitive (lowercase)
- Add serde support for JSON serialization
- Add unit tests for key generation and serialization

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 2.2: 实现Service、Lease、RouteRule模型

**Files:**
- Create: `artemis-core/src/model/service.rs`
- Create: `artemis-core/src/model/lease.rs`
- Create: `artemis-core/src/model/route.rs`
- Create: `artemis-core/src/model/change.rs`

**Step 1: 实现Service模型**

```rust
// artemis-core/src/model/service.rs
use super::instance::Instance;
use super::route::RouteRule;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub service_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    pub instances: Vec<Instance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logic_instances: Option<Vec<Instance>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_rules: Option<Vec<RouteRule>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceGroup {
    pub group_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instances: Option<Vec<Instance>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}
```

**Step 2: 实现Lease模型**

```rust
// artemis-core/src/model/lease.rs
use super::instance::InstanceKey;
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Lease {
    key: InstanceKey,
    creation_time: Instant,
    renewal_time: Arc<Mutex<Instant>>,
    eviction_time: Arc<Mutex<Option<Instant>>>,
    ttl: Duration,
}

impl Lease {
    pub fn new(key: InstanceKey, ttl: Duration) -> Self {
        let now = Instant::now();
        Self {
            key,
            creation_time: now,
            renewal_time: Arc::new(Mutex::new(now)),
            eviction_time: Arc::new(Mutex::new(None)),
            ttl,
        }
    }

    pub fn renew(&self) {
        *self.renewal_time.lock() = Instant::now();
    }

    pub fn is_expired(&self) -> bool {
        self.renewal_time.lock().elapsed() > self.ttl
    }

    pub fn mark_evicted(&self) {
        *self.eviction_time.lock() = Some(Instant::now());
    }

    pub fn key(&self) -> &InstanceKey {
        &self.key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::instance::InstanceKey;
    use std::thread::sleep;

    #[test]
    fn test_lease_expiration() {
        let key = InstanceKey {
            region_id: "test".to_string(),
            zone_id: "zone".to_string(),
            service_id: "service".to_string(),
            group_id: String::new(),
            instance_id: "inst".to_string(),
        };

        let lease = Lease::new(key, Duration::from_millis(100));

        assert!(!lease.is_expired());
        sleep(Duration::from_millis(150));
        assert!(lease.is_expired());
    }

    #[test]
    fn test_lease_renewal() {
        let key = InstanceKey {
            region_id: "test".to_string(),
            zone_id: "zone".to_string(),
            service_id: "service".to_string(),
            group_id: String::new(),
            instance_id: "inst".to_string(),
        };

        let lease = Lease::new(key, Duration::from_millis(100));

        sleep(Duration::from_millis(60));
        lease.renew();
        sleep(Duration::from_millis(60));

        assert!(!lease.is_expired());
    }
}
```

**Step 3: 实现RouteRule模型**

```rust
// artemis-core/src/model/route.rs
use super::service::ServiceGroup;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRule {
    pub route_id: String,
    pub strategy: RouteStrategy,
    pub groups: Vec<ServiceGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RouteStrategy {
    WeightedRoundRobin,
    CloseByVisit,
}
```

**Step 4: 实现InstanceChange模型**

```rust
// artemis-core/src/model/change.rs
use super::instance::Instance;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceChange {
    pub instance: Instance,
    pub change_type: ChangeType,
    pub change_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChangeType {
    New,
    Delete,
    Change,
    Reload,
}
```

**Step 5: 运行测试**

```bash
cargo test -p artemis-core
```

Expected: All tests pass

**Step 6: 提交**

```bash
git add artemis-core/src/model/
git commit -m "feat(core): implement Service, Lease, RouteRule, InstanceChange models

- Add Service and ServiceGroup models
- Add Lease with TTL and renewal logic
- Add RouteRule with strategy enum
- Add InstanceChange for delta tracking
- Add comprehensive tests for Lease behavior

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 2.3: 实现Request/Response模型

**Files:**
- Create: `artemis-core/src/model/request.rs`

**Step 1: 实现请求响应模型**

```rust
// artemis-core/src/model/request.rs
use super::instance::{Instance, InstanceKey};
use super::service::Service;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ===== 注册 =====

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub instances: Vec<Instance>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub response_status: ResponseStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_instances: Option<Vec<Instance>>,
}

// ===== 心跳 =====

#[derive(Debug, Serialize, Deserialize)]
pub struct HeartbeatRequest {
    pub instance_keys: Vec<InstanceKey>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeartbeatResponse {
    pub response_status: ResponseStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_instance_keys: Option<Vec<InstanceKey>>,
}

// ===== 注销 =====

#[derive(Debug, Serialize, Deserialize)]
pub struct UnregisterRequest {
    pub instance_keys: Vec<InstanceKey>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnregisterResponse {
    pub response_status: ResponseStatus,
}

// ===== 发现 =====

#[derive(Debug, Serialize, Deserialize)]
pub struct GetServiceRequest {
    pub discovery_config: DiscoveryConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    pub service_id: String,
    pub region_id: String,
    pub zone_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discovery_data: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetServiceResponse {
    pub response_status: ResponseStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<Service>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetServicesRequest {
    pub region_id: String,
    pub zone_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetServicesResponse {
    pub response_status: ResponseStatus,
    pub services: Vec<Service>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetServicesDeltaRequest {
    pub region_id: String,
    pub zone_id: String,
    pub since_timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetServicesDeltaResponse {
    pub response_status: ResponseStatus,
    pub services: Vec<Service>,
    pub current_timestamp: i64,
}

// ===== 通用响应状态 =====

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseStatus {
    pub error_code: ErrorCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

impl ResponseStatus {
    pub fn success() -> Self {
        Self {
            error_code: ErrorCode::Success,
            error_message: None,
        }
    }

    pub fn error(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            error_code: code,
            error_message: Some(message.into()),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ErrorCode {
    Success,
    BadRequest,
    ServiceUnavailable,
    RateLimited,
    InternalError,
}
```

**Step 2: 提交**

```bash
git add artemis-core/src/model/request.rs
git commit -m "feat(core): implement Request/Response models

- Add Register/Heartbeat/Unregister request/response
- Add GetService/GetServices/GetServicesDelta models
- Add ResponseStatus with ErrorCode enum
- All models support JSON serialization

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 2.4: 实现Trait定义

**Files:**
- Create: `artemis-core/src/traits/mod.rs`
- Create: `artemis-core/src/traits/registry.rs`
- Create: `artemis-core/src/traits/discovery.rs`

**Step 1: 创建traits模块**

```rust
// artemis-core/src/traits/mod.rs
pub mod registry;
pub mod discovery;

pub use registry::RegistryService;
pub use discovery::DiscoveryService;
```

**Step 2: 定义RegistryService trait**

```rust
// artemis-core/src/traits/registry.rs
use crate::model::{HeartbeatRequest, HeartbeatResponse};
use crate::model::{RegisterRequest, RegisterResponse};
use crate::model::{UnregisterRequest, UnregisterResponse};
use async_trait::async_trait;

#[async_trait]
pub trait RegistryService: Send + Sync {
    /// 注册服务实例
    async fn register(&self, request: RegisterRequest) -> RegisterResponse;

    /// 心跳续约
    async fn heartbeat(&self, request: HeartbeatRequest) -> HeartbeatResponse;

    /// 注销服务实例
    async fn unregister(&self, request: UnregisterRequest) -> UnregisterResponse;
}
```

**Step 3: 定义DiscoveryService trait**

```rust
// artemis-core/src/traits/discovery.rs
use crate::model::{GetServiceRequest, GetServiceResponse};
use crate::model::{GetServicesRequest, GetServicesResponse};
use crate::model::{GetServicesDeltaRequest, GetServicesDeltaResponse};
use async_trait::async_trait;

#[async_trait]
pub trait DiscoveryService: Send + Sync {
    /// 查询单个服务
    async fn get_service(&self, request: GetServiceRequest) -> GetServiceResponse;

    /// 查询所有服务
    async fn get_services(&self, request: GetServicesRequest) -> GetServicesResponse;

    /// 查询服务变更（增量）
    async fn get_services_delta(
        &self,
        request: GetServicesDeltaRequest,
    ) -> GetServicesDeltaResponse;
}
```

**Step 4: 验证编译**

```bash
cargo check -p artemis-core
```

Expected: 编译成功

**Step 5: 提交**

```bash
git add artemis-core/src/traits/
git commit -m "feat(core): define RegistryService and DiscoveryService traits

- Add RegistryService with register/heartbeat/unregister
- Add DiscoveryService with get_service/get_services/get_services_delta
- Use async_trait for async methods
- Traits are Send + Sync for multi-threaded use

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 2.5: 实现错误处理和配置

**Files:**
- Create: `artemis-core/src/error.rs`
- Create: `artemis-core/src/config.rs`
- Create: `artemis-core/src/utils.rs`

**Step 1: 实现错误类型**

```rust
// artemis-core/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ArtemisError {
    #[error("Invalid instance: {0}")]
    InvalidInstance(String),

    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    #[error("Instance not found: {0}")]
    InstanceNotFound(String),

    #[error("Lease expired for instance: {0}")]
    LeaseExpired(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    InternalError(String),
}

pub type Result<T> = std::result::Result<T, ArtemisError>;
```

**Step 2: 实现配置结构**

```rust
// artemis-core/src/config.rs
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
            cluster: ClusterConfig {
                enabled: false,
                peer_nodes: None,
            },
            database: None,
        }
    }
}
```

**Step 3: 创建utils模块（占位）**

```rust
// artemis-core/src/utils.rs
//! 实用工具函数
```

**Step 4: 添加humantime-serde依赖**

更新 `artemis-core/Cargo.toml`:

```toml
[dependencies]
# ... existing dependencies ...
humantime-serde = "1.1"
```

**Step 5: 验证编译**

```bash
cargo check -p artemis-core
cargo test -p artemis-core
```

Expected: 编译成功，所有测试通过

**Step 6: 提交**

```bash
git add artemis-core/
git commit -m "feat(core): implement error handling and configuration

- Add ArtemisError with thiserror
- Add ArtemisConfig with server/registry/cluster/database config
- Add humantime-serde for duration serialization
- Add utils module placeholder

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## 阶段2完成标准

- ✅ 所有核心模型实现完成
- ✅ Trait定义清晰
- ✅ 错误处理完整
- ✅ 配置结构完善
- ✅ `cargo test -p artemis-core` 全部通过
- ✅ 文档注释完整
