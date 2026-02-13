# Artemis Rust 重写实施计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 使用Rust重写Artemis服务注册中心，消除GC问题，实现P99延迟<10ms，支持100k+实例

**Architecture:** Workspace多Crate架构，包含artemis-core（核心模型）、artemis-server（业务逻辑）、artemis-web（Web层）、artemis-management（管理功能）、artemis-client（客户端SDK）、artemis（CLI程序）6个crate。一次性完整实现，并行开发所有模块。

**Tech Stack:** Rust 2024, Tokio, Axum, DashMap, parking_lot, SQLx, Governor, Serde, Clap

---

## 阶段1: 项目基础设施

### Task 1.1: 创建Workspace项目结构

**Files:**
- Create: `Cargo.toml`
- Create: `rust-toolchain.toml`
- Create: `.rustfmt.toml`
- Create: `.gitignore`

**Step 1: 创建workspace根配置**

```toml
# Cargo.toml
[workspace]
members = [
    "artemis-core",
    "artemis-server",
    "artemis-web",
    "artemis-management",
    "artemis-client",
    "artemis",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.85"
authors = ["Artemis Contributors"]
license = "MIT OR Apache-2.0"

[workspace.dependencies]
# 异步运行时
tokio = { version = "1.41", features = ["full"] }
tokio-util = { version = "0.7", features = ["codec"] }

# Web框架
axum = { version = "0.7", features = ["ws", "macros"] }
tower = { version = "0.5", features = ["full"] }
tower-http = { version = "0.6", features = ["cors", "trace", "compression"] }

# 序列化
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 并发数据结构
dashmap = "6.1"
parking_lot = "0.12"

# HTTP客户端
reqwest = { version = "0.12", features = ["json"] }

# WebSocket
tokio-tungstenite = "0.24"

# 数据库
sqlx = { version = "0.8", features = ["runtime-tokio", "mysql", "chrono", "json"] }

# 时间处理
chrono = { version = "0.4", features = ["serde"] }

# 限流
governor = "0.7"

# 日志和追踪
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# 配置
config = "0.14"
figment = { version = "0.10", features = ["toml", "env"] }

# 错误处理
thiserror = "2.0"
anyhow = "1.0"

# UUID
uuid = { version = "1.11", features = ["v4", "serde"] }

# CLI
clap = { version = "4.5", features = ["derive"] }

# 异步trait
async-trait = "0.1"

# 其他工具
bytes = "1.8"
futures = "0.3"
```

**Step 2: 创建工具链配置**

```toml
# rust-toolchain.toml
[toolchain]
channel = "1.85"
edition = "2024"
components = ["rustfmt", "clippy"]
```

**Step 3: 创建rustfmt配置**

```toml
# .rustfmt.toml
edition = "2024"
max_width = 100
use_small_heuristics = "Max"
```

**Step 4: 更新.gitignore**

```
# .gitignore
/target
Cargo.lock
**/*.rs.bk
*.pdb
.env
artemis.toml
*.log
```

**Step 5: 提交基础配置**

```bash
git add Cargo.toml rust-toolchain.toml .rustfmt.toml .gitignore
git commit -m "chore: setup workspace structure and toolchain"
```

### Task 1.2: 创建所有crate目录

**Files:**
- Create: `artemis-core/Cargo.toml`
- Create: `artemis-core/src/lib.rs`
- Create: `artemis-server/Cargo.toml`
- Create: `artemis-server/src/lib.rs`
- Create: `artemis-web/Cargo.toml`
- Create: `artemis-web/src/lib.rs`
- Create: `artemis-management/Cargo.toml`
- Create: `artemis-management/src/lib.rs`
- Create: `artemis-client/Cargo.toml`
- Create: `artemis-client/src/lib.rs`
- Create: `artemis/Cargo.toml`
- Create: `artemis/src/main.rs`

**Step 1: 创建artemis-core**

```bash
mkdir -p artemis-core/src
```

```toml
# artemis-core/Cargo.toml
[package]
name = "artemis-core"
version.workspace = true
edition.workspace = true

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
chrono = { workspace = true }
thiserror = { workspace = true }
async-trait = { workspace = true }
parking_lot = { workspace = true }
uuid = { workspace = true }
```

```rust
// artemis-core/src/lib.rs
//! Artemis Core - 核心数据模型和trait定义

pub mod model;
pub mod traits;
pub mod error;
pub mod config;
pub mod utils;
```

**Step 2: 创建artemis-server**

```bash
mkdir -p artemis-server/src
```

```toml
# artemis-server/Cargo.toml
[package]
name = "artemis-server"
version.workspace = true
edition.workspace = true

[dependencies]
artemis-core = { path = "../artemis-core" }
tokio = { workspace = true }
dashmap = { workspace = true }
parking_lot = { workspace = true }
async-trait = { workspace = true }
governor = { workspace = true }
tracing = { workspace = true }
chrono = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
reqwest = { workspace = true }
anyhow = { workspace = true }
```

```rust
// artemis-server/src/lib.rs
//! Artemis Server - 业务逻辑实现

pub mod registry;
pub mod discovery;
pub mod lease;
pub mod cache;
pub mod cluster;
pub mod replication;
pub mod ratelimiter;
pub mod storage;
```

**Step 3: 创建artemis-web**

```bash
mkdir -p artemis-web/src
```

```toml
# artemis-web/Cargo.toml
[package]
name = "artemis-web"
version.workspace = true
edition.workspace = true

[dependencies]
artemis-core = { path = "../artemis-core" }
artemis-server = { path = "../artemis-server" }
axum = { workspace = true }
tower = { workspace = true }
tower-http = { workspace = true }
tokio = { workspace = true }
tokio-util = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tracing = { workspace = true }
anyhow = { workspace = true }
dashmap = { workspace = true }
futures = { workspace = true }
chrono = { workspace = true }
```

```rust
// artemis-web/src/lib.rs
//! Artemis Web - HTTP/WebSocket API层

pub mod server;
pub mod state;
pub mod api;
pub mod websocket;
pub mod middleware;
```

**Step 4: 创建artemis-management**

```bash
mkdir -p artemis-management/src
```

```toml
# artemis-management/Cargo.toml
[package]
name = "artemis-management"
version.workspace = true
edition.workspace = true

[dependencies]
artemis-core = { path = "../artemis-core" }
artemis-server = { path = "../artemis-server" }
sqlx = { workspace = true }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
chrono = { workspace = true }
uuid = { workspace = true }
tracing = { workspace = true }
axum = { workspace = true }
anyhow = { workspace = true }
```

```rust
// artemis-management/src/lib.rs
//! Artemis Management - 管理功能和持久化

pub mod instance;
pub mod group;
pub mod route;
pub mod dao;
pub mod api;
```

**Step 5: 创建artemis-client**

```bash
mkdir -p artemis-client/src
```

```toml
# artemis-client/Cargo.toml
[package]
name = "artemis-client"
version.workspace = true
edition.workspace = true
description = "Artemis Service Registry Client SDK"
license.workspace = true

[dependencies]
artemis-core = { path = "../artemis-core" }
reqwest = { workspace = true }
tokio = { workspace = true }
tokio-tungstenite = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
async-trait = { workspace = true }
parking_lot = { workspace = true }
tracing = { workspace = true }
futures = { workspace = true }
```

```rust
// artemis-client/src/lib.rs
//! Artemis Client SDK - 客户端SDK

pub mod config;
pub mod registry;
pub mod discovery;
pub mod websocket;
pub mod error;
```

**Step 6: 创建artemis CLI**

```bash
mkdir -p artemis/src
```

```toml
# artemis/Cargo.toml
[package]
name = "artemis"
version.workspace = true
edition.workspace = true
description = "Artemis Service Registry - CLI and Server"

[[bin]]
name = "artemis"
path = "src/main.rs"

[dependencies]
artemis-core = { path = "../artemis-core" }
artemis-server = { path = "../artemis-server" }
artemis-web = { path = "../artemis-web" }
artemis-management = { path = "../artemis-management" }
clap = { workspace = true }
tokio = { workspace = true }
figment = { workspace = true }
toml = "0.8"
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
reqwest = { workspace = true }
serde_json = { workspace = true }
sqlx = { workspace = true }
anyhow = { workspace = true }
```

```rust
// artemis/src/main.rs
//! Artemis CLI - 可执行程序入口

fn main() {
    println!("Artemis Service Registry");
}
```

**Step 7: 验证workspace编译**

```bash
cargo check --workspace
```

Expected: 成功编译所有crate

**Step 8: 提交crate结构**

```bash
git add .
git commit -m "chore: create all crate directories and basic structure"
```

---

## 阶段2: artemis-core实现

### Task 2.1: 实现核心数据模型 - Instance

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
git commit -m "feat(core): implement Instance model with tests"
```

### Task 2.2: 实现Service、Lease、RouteRule模型

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
git commit -m "feat(core): implement Service, Lease, RouteRule, InstanceChange models"
```

### Task 2.3: 实现Request/Response模型

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
git commit -m "feat(core): implement Request/Response models"
```

由于篇幅限制，我将继续创建完整的实施计划...

继续实施计划内容...
