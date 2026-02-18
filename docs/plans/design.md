# Artemis Rust 重写项目 - 详细设计文档

**版本**: 1.0.0
**日期**: 2026-02-13
**实施方式**: 一次性完整实现
**架构方案**: Workspace多Crate架构

---

## 目录

1. [项目概述](#1-项目概述)
2. [项目结构和模块划分](#2-项目结构和模块划分)
3. [核心数据模型（artemis-core）](#3-核心数据模型artemis-core)
4. [业务逻辑层（artemis-server）](#4-业务逻辑层artemis-server)
5. [Web层（artemis-web）](#5-web层artemis-web)
6. [管理功能层（artemis-management）](#6-管理功能层artemis-management)
7. [客户端SDK（artemis-client）](#7-客户端sdkartemis-client)
8. [CLI可执行程序（artemis）](#8-cli可执行程序artemis)
9. [技术栈总结](#9-技术栈总结)
10. [实施清单](#10-实施清单)

---

## 1. 项目概述

### 1.1 背景

Artemis是携程开发的SOA服务注册中心，Java版本存在严重的GC问题。本项目使用Rust重写，目标是：

- 消除GC导致的性能问题
- P99延迟 < 10ms
- 支持100k+服务实例
- 保持API完全兼容

### 1.2 设计原则

- **一次性完整实现**：直接构建完整项目结构，并行开发所有模块
- **模块化设计**：使用Workspace多Crate架构，职责分离
- **零成本抽象**：trait编译时内联，无运行时开销
- **类型安全**：充分利用Rust类型系统
- **高并发**：使用DashMap、parking_lot等高性能并发库

---

## 2. 项目结构和模块划分

### 2.1 Workspace结构

```toml
# 根目录 Cargo.toml
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
rust-version = "1.93"
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
sea-orm = { version = "1.1", features = ["runtime-tokio-rustls", "sqlx-sqlite", "sqlx-mysql"] }

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

### 2.2 六个Crate架构

#### 1. artemis-core (lib)
- **职责**: 核心类型、trait定义、错误类型
- **依赖**: 最小化（serde、chrono、thiserror）
- **导出**: 所有模型和trait

#### 2. artemis-server (lib)
- **职责**: 业务逻辑实现
- **依赖**: artemis-core + tokio + dashmap + governor
- **模块**: registry、discovery、lease、cache、cluster、replication

#### 3. artemis-web (lib)
- **职责**: HTTP/WebSocket API层
- **依赖**: artemis-core + artemis-server + axum + tower
- **导出**: API路由和WebSocket处理器

#### 4. artemis-management (lib)
- **职责**: 管理功能和持久化
- **依赖**: artemis-core + artemis-server + sqlx
- **模块**: instance、group、route、dao

#### 5. artemis-client (lib)
- **职责**: 客户端SDK（独立发布）
- **依赖**: artemis-core + reqwest + tokio-tungstenite
- **注意**: 独立于bin，供外部用户使用

#### 6. artemis (bin only)
- **职责**: 唯一可执行程序
- **依赖**: artemis-core + artemis-server + artemis-web + artemis-management + clap
- **注意**: 不依赖artemis-client

### 2.3 目录结构

```
artemis/
├── Cargo.toml (workspace)
├── .rustfmt.toml
├── rust-toolchain.toml
│
├── artemis-core/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── model/
│       │   ├── mod.rs
│       │   ├── instance.rs
│       │   ├── service.rs
│       │   ├── lease.rs
│       │   ├── route.rs
│       │   ├── change.rs
│       │   └── request.rs
│       ├── traits/
│       │   ├── mod.rs
│       │   ├── registry.rs
│       │   ├── discovery.rs
│       │   └── storage.rs
│       ├── error.rs
│       ├── config.rs
│       └── utils.rs
│
├── artemis-server/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── registry/
│       │   ├── mod.rs
│       │   ├── service.rs
│       │   └── repository.rs
│       ├── discovery/
│       │   ├── mod.rs
│       │   ├── service.rs
│       │   ├── filter.rs
│       │   └── zone_filter.rs
│       ├── lease/
│       │   ├── mod.rs
│       │   ├── manager.rs
│       │   └── safe_checker.rs
│       ├── cache/
│       │   ├── mod.rs
│       │   ├── versioned.rs
│       │   └── delta.rs
│       ├── cluster/
│       │   ├── mod.rs
│       │   ├── manager.rs
│       │   └── node.rs
│       ├── replication/
│       │   ├── mod.rs
│       │   ├── manager.rs
│       │   ├── executor.rs
│       │   └── queue.rs
│       ├── ratelimiter/
│       │   ├── mod.rs
│       │   └── limiter.rs
│       └── storage/
│           ├── mod.rs
│           └── memory.rs
│
├── artemis-web/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── server.rs
│       ├── state.rs
│       ├── api/
│       │   ├── mod.rs
│       │   ├── registry.rs
│       │   ├── discovery.rs
│       │   ├── health.rs
│       │   └── routes.rs
│       ├── websocket/
│       │   ├── mod.rs
│       │   ├── handler.rs
│       │   ├── session.rs
│       │   └── publisher.rs
│       └── middleware/
│           ├── mod.rs
│           ├── logging.rs
│           ├── metrics.rs
│           └── error.rs
│
├── artemis-management/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── instance/
│       │   ├── mod.rs
│       │   ├── operations.rs
│       │   └── filter.rs
│       ├── group/
│       │   ├── mod.rs
│       │   ├── manager.rs
│       │   └── service.rs
│       ├── route/
│       │   ├── mod.rs
│       │   ├── manager.rs
│       │   └── strategy.rs
│       ├── dao/
│       │   ├── mod.rs
│       │   ├── instance_dao.rs
│       │   ├── server_dao.rs
│       │   ├── group_dao.rs
│       │   ├── route_dao.rs
│       │   └── schema.sql
│       └── api/
│           ├── mod.rs
│           ├── instance.rs
│           ├── group.rs
│           └── route.rs
│
├── artemis-client/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── config.rs
│       ├── registry/
│       │   ├── mod.rs
│       │   ├── client.rs
│       │   └── heartbeat.rs
│       ├── discovery/
│       │   ├── mod.rs
│       │   ├── client.rs
│       │   ├── cache.rs
│       │   └── refresh.rs
│       ├── websocket/
│       │   ├── mod.rs
│       │   ├── client.rs
│       │   └── listener.rs
│       └── error.rs
│
└── artemis/
    ├── Cargo.toml
    └── src/
        ├── main.rs
        ├── server.rs
        ├── commands/
        │   ├── mod.rs
        │   ├── service.rs
        │   ├── instance.rs
        │   └── config.rs
        └── utils/
            ├── mod.rs
            └── table.rs
```

### 2.4 rust-toolchain.toml

```toml
[toolchain]
channel = "1.93"
edition = "2024"
components = ["rustfmt", "clippy"]
```

---

## 3. 核心数据模型（artemis-core）

### 3.1 Instance模型

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
    pub service_id: String,        // 大小写不敏感
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
    pub service_id: String,      // 已转小写
    pub group_id: String,        // 空字符串表示无分组
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
```

### 3.2 Service模型

```rust
// artemis-core/src/model/service.rs
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
```

### 3.3 Lease模型

```rust
// artemis-core/src/model/lease.rs
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::Mutex;

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
}
```

### 3.4 Trait定义

```rust
// artemis-core/src/traits/registry.rs
use async_trait::async_trait;

#[async_trait]
pub trait RegistryService: Send + Sync {
    async fn register(&self, request: RegisterRequest) -> Result<RegisterResponse>;
    async fn heartbeat(&self, request: HeartbeatRequest) -> Result<HeartbeatResponse>;
    async fn unregister(&self, request: UnregisterRequest) -> Result<UnregisterResponse>;
}

#[async_trait]
pub trait DiscoveryService: Send + Sync {
    async fn get_service(&self, request: GetServiceRequest) -> Result<GetServiceResponse>;
    async fn get_services(&self, request: GetServicesRequest) -> Result<GetServicesResponse>;
    async fn get_services_delta(&self, request: GetServicesDeltaRequest) -> Result<GetServicesDeltaResponse>;
}
```

### 3.5 错误处理

```rust
// artemis-core/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ArtemisError {
    #[error("Instance not found: {0}")]
    InstanceNotFound(String),

    #[error("Lease not found: {0}")]
    LeaseNotFound(String),

    #[error("Rate limited")]
    RateLimited,

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Service unavailable")]
    ServiceUnavailable,

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, ArtemisError>;
```

### 3.6 配置结构

```rust
// artemis-core/src/config.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArtemisConfig {
    pub server: ServerConfig,
    pub registry: RegistryConfig,
    pub discovery: DiscoveryConfig,
    pub cluster: ClusterConfig,
    pub database: Option<DatabaseConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub worker_threads: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegistryConfig {
    pub lease_ttl_secs: u64,
    pub legacy_lease_ttl_secs: u64,
    pub clean_interval_ms: u64,
    pub rate_limiter: RateLimiterConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RateLimiterConfig {
    pub register_qps: u32,
    pub heartbeat_qps: u32,
    pub unregister_qps: u32,
}
```

---

## 4. 业务逻辑层（artemis-server）

### 4.1 RegistryService实现

核心注册服务实现，包含：
- 限流检查
- 实例注册/注销
- 心跳续约
- 异步复制

### 4.2 RegistryRepository

使用DashMap实现高并发内存存储：
- `services: DashMap<String, Service>` - 服务映射
- `instances: DashMap<InstanceKey, Instance>` - 实例映射
- 自动更新服务和实例关系

### 4.3 LeaseManager

租约生命周期管理：
- 创建租约
- 续约
- 后台清理任务（定时扫描过期租约）
- 事件监听器机制

### 4.4 VersionedCacheManager

版本化缓存支持增量同步：
- `BTreeMap<Timestamp, Services>` 存储多版本
- 定时刷新任务
- Delta计算

### 4.5 RateLimiter

基于Governor的Token Bucket限流：
- 注册限流
- 心跳限流
- 注销限流

### 4.6 ClusterManager & ReplicationManager

集群管理和数据复制：
- 节点健康检查
- 复制任务队列
- 异步复制执行器

---

## 5. Web层（artemis-web）

### 5.1 API路径（与Java版本兼容）

```
POST   /api/v1/registry/register         - 注册实例
POST   /api/v1/registry/heartbeat        - 心跳续约
POST   /api/v1/registry/unregister       - 注销实例

POST   /api/v1/discovery/service         - 查询单个服务
POST   /api/v1/discovery/services        - 查询所有服务
POST   /api/v1/discovery/services/delta  - 增量同步

GET    /api/v1/ws/instance-change        - WebSocket连接

GET    /health                           - 健康检查
GET    /ready                            - 就绪检查
```

### 5.2 WebSocket实时推送

- 基于broadcast channel实现订阅发布
- SessionManager管理所有WebSocket连接
- 实例变更自动推送到订阅客户端

### 5.3 中间件

- TraceLayer: 请求追踪
- CorsLayer: 跨域支持
- CompressionLayer: 响应压缩
- 自定义错误处理中间件

---

## 6. 管理功能层（artemis-management）

### 6.1 实例操作

- **拉入（Pull In）**: 标记实例为可用
- **拉出（Pull Out）**: 标记实例为不可用
- 操作记录持久化到MySQL

### 6.2 ManagementDiscoveryFilter

过滤被拉出的实例：
- 从数据库查询拉出记录
- 在服务查询时自动过滤

### 6.3 分组管理

- 创建/更新/删除服务分组
- 分组权重配置
- 实例与分组关联

### 6.4 路由规则

- 加权轮询（Weighted Round Robin）
- 就近访问（Close By Visit）
- 路由规则持久化

### 6.5 MySQL Schema

```sql
-- 实例操作记录表
CREATE TABLE instance_operations (
    id VARCHAR(64) PRIMARY KEY,
    region_id VARCHAR(64) NOT NULL,
    service_id VARCHAR(128) NOT NULL,
    instance_id VARCHAR(128) NOT NULL,
    operation_type ENUM('pull_in', 'pull_out') NOT NULL,
    operator VARCHAR(64) NOT NULL,
    operated_at TIMESTAMP NOT NULL,
    INDEX idx_region_service (region_id, service_id),
    INDEX idx_instance (region_id, service_id, instance_id, operated_at)
);

-- 服务分组表
CREATE TABLE service_groups (
    group_key VARCHAR(128) PRIMARY KEY,
    service_id VARCHAR(128) NOT NULL,
    weight INT,
    metadata JSON,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_service (service_id)
);

-- 路由规则表
CREATE TABLE route_rules (
    route_id VARCHAR(64) PRIMARY KEY,
    service_id VARCHAR(128) NOT NULL,
    strategy ENUM('weighted-round-robin', 'close-by-visit') NOT NULL,
    groups JSON NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_service (service_id)
);
```

---

## 7. 客户端SDK（artemis-client）

### 7.1 RegistryClient

- 自动注册
- 自动心跳（后台任务）
- 优雅关闭（Drop时自动注销）

### 7.2 DiscoveryClient

- 本地缓存
- 定时刷新任务
- 缓存未命中时从服务器拉取

### 7.3 WebSocketClient

- 实时接收实例变更
- broadcast channel分发事件
- 自动重连（TODO）

### 7.4 使用示例

```rust
use artemis_client::*;

#[tokio::main]
async fn main() {
    let config = ClientConfig::default();

    // 注册
    let registry_client = RegistryClient::new(config.clone());
    registry_client.register(vec![instance]).await.unwrap();

    // 发现
    let discovery_client = DiscoveryClient::new(config.clone());
    let service = discovery_client.get_service("my-service", "us-east", "zone-1").await.unwrap();

    // WebSocket
    let ws_client = WebSocketClient::new("http://localhost:8080".to_string());
    let mut change_rx = ws_client.subscribe();

    while let Ok(change) = change_rx.recv().await {
        println!("Instance changed: {:?}", change);
    }
}
```

---

## 8. CLI可执行程序（artemis）

### 8.1 命令结构

```
artemis
├── server              # 启动服务器
├── service
│   ├── list           # 列出所有服务
│   └── get            # 查询服务详情
├── instance
│   ├── pull-in        # 拉入实例
│   └── pull-out       # 拉出实例
└── config
    ├── convert        # 转换Java配置
    └── validate       # 验证配置
```

### 8.2 使用示例

```bash
# 启动服务器
artemis server --config artemis.toml

# 查询服务
artemis service list --region us-east
artemis service get --service-id my-service

# 实例管理
artemis instance pull-in --region us-east --service-id my-service --instance-id inst-1
artemis instance pull-out --region us-east --service-id my-service --instance-id inst-1

# 配置工具
artemis config convert --from artemis.properties --to artemis.toml
artemis config validate --file artemis.toml
```

### 8.3 配置文件示例

```toml
[server]
host = "0.0.0.0"
port = 8080
worker_threads = 4

[registry]
lease_ttl_secs = 20
legacy_lease_ttl_secs = 90
clean_interval_ms = 1000

[registry.rate_limiter]
register_qps = 10000
heartbeat_qps = 100000
unregister_qps = 10000

[discovery]
cache_refresh_interval_secs = 30
max_cache_versions = 100

[cluster]
enabled = true
nodes = ["http://node1:8080", "http://node2:8080"]

[database]
url = "mysql://user:password@localhost:3306/artemis"
max_connections = 10
```

---

## 9. 技术栈总结

### 9.1 核心技术

| 组件 | 选型 | 版本 | 用途 |
|------|------|------|------|
| Rust Edition | 2024 | 1.93+ | 语言版本 |
| Tokio | tokio | 1.41 | 异步运行时 |
| Axum | axum | 0.7 | Web框架 |
| DashMap | dashmap | 6.1 | 并发HashMap |
| parking_lot | parking_lot | 0.12 | 高性能锁 |
| SeaORM | sea-orm | 1.1 | 数据库ORM |
| Governor | governor | 0.7 | 限流器 |
| Serde | serde | 1.0 | 序列化 |

### 9.2 性能优化点

- **DashMap**: 无锁并发HashMap，替代`Arc<RwLock<HashMap>>`
- **parking_lot**: 比std::Mutex快2-5倍
- **零拷贝**: Arc共享数据，避免clone
- **异步I/O**: Tokio异步运行时
- **连接池**: SQLx数据库连接池

### 9.3 并发模型

- **服务注册**: DashMap并发写入，无需额外锁
- **心跳处理**: 无锁续约，仅更新Instant
- **租约清理**: 单独后台任务，不阻塞主流程
- **WebSocket推送**: broadcast channel多播

---

## 10. 实施清单

### 10.1 基础设施

- [ ] 创建Workspace项目结构
- [ ] 配置rust-toolchain.toml
- [ ] 配置.rustfmt.toml和clippy规则
- [ ] 初始化6个crate的Cargo.toml
- [ ] 配置workspace依赖

### 10.2 artemis-core

- [ ] 定义Instance、Service等核心模型
- [ ] 实现InstanceKey（Hash + Eq）
- [ ] 定义RegistryService、DiscoveryService trait
- [ ] 实现ArtemisError错误类型
- [ ] 实现配置结构（ArtemisConfig）
- [ ] 编写核心模型的单元测试

### 10.3 artemis-server

- [ ] 实现RegistryRepository（DashMap）
- [ ] 实现LeaseManager和租约清理任务
- [ ] 实现RegistryServiceImpl
- [ ] 实现DiscoveryServiceImpl
- [ ] 实现VersionedCacheManager
- [ ] 实现RateLimiter
- [ ] 实现ClusterManager
- [ ] 实现ReplicationManager
- [ ] 编写集成测试

### 10.4 artemis-web

- [ ] 实现Axum路由配置
- [ ] 实现Registry API Handler
- [ ] 实现Discovery API Handler
- [ ] 实现WebSocket Handler
- [ ] 实现SessionManager
- [ ] 实现中间件（日志、CORS、压缩）
- [ ] 实现健康检查
- [ ] 编写API测试

### 10.5 artemis-management

- [ ] 设计并创建MySQL Schema
- [ ] 实现InstanceOperationDao
- [ ] 实现GroupDao、RouteDao
- [ ] 实现InstanceOperationsManager
- [ ] 实现ManagementDiscoveryFilter
- [ ] 实现GroupManager
- [ ] 实现RouteManager
- [ ] 实现管理API Handler
- [ ] 编写数据库测试

### 10.6 artemis-client

- [ ] 实现ClientConfig
- [ ] 实现RegistryClient
- [ ] 实现自动心跳任务
- [ ] 实现DiscoveryClient
- [ ] 实现本地缓存和刷新任务
- [ ] 实现WebSocketClient
- [ ] 实现ChangeListenerManager
- [ ] 编写客户端集成测试
- [ ] 编写使用示例文档

### 10.7 artemis (bin)

- [ ] 实现CLI主入口（clap）
- [ ] 实现server子命令
- [ ] 实现service子命令
- [ ] 实现instance子命令
- [ ] 实现config子命令
- [ ] 实现Java配置转换逻辑
- [ ] 编写CLI测试

### 10.8 测试和文档

- [ ] 编写单元测试（覆盖率>80%）
- [ ] 编写集成测试
- [ ] 编写性能基准测试（criterion）
- [ ] 编写API文档
- [ ] 编写部署文档
- [ ] 编写迁移指南
- [ ] 编写客户端SDK使用文档

### 10.9 性能优化

- [ ] 性能基准测试
- [ ] 内存优化
- [ ] 并发优化
- [ ] 达到P99 < 10ms目标

### 10.10 生产就绪

- [ ] 添加Prometheus指标
- [ ] 添加分布式追踪（OpenTelemetry）
- [ ] 配置优雅关闭
- [ ] 添加健康检查
- [ ] Docker镜像
- [ ] Kubernetes部署配置
- [ ] 监控和告警配置

---

## 附录

### A. 与Java版本API兼容性

所有REST API路径和JSON格式完全兼容Java版本，现有客户端无需修改即可使用。

### B. 数据迁移

- 注册数据：内存存储，无需迁移（心跳自然刷新）
- 管理数据：MySQL Schema兼容，可直接使用Java版本数据库

### C. 配置迁移

提供`artemis config convert`命令自动转换Java properties到TOML格式。

---

**文档结束**
