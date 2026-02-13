# Artemis服务注册中心 - Rust重写产品规格说明书

**版本**: 1.0.0
**日期**: 2026-02-13
**原项目**: 携程Artemis SOA服务注册中心 (Java 1.5.16)
**目标**: 基于Rust重写，解决GC问题，提升性能和稳定性

---

## 目录

1. [项目概述](#1-项目概述)
2. [系统架构](#2-系统架构)
3. [核心数据模型](#3-核心数据模型)
4. [功能模块详细规格](#4-功能模块详细规格)
5. [API规格](#5-api规格)
6. [非功能性需求](#6-非功能性需求)
7. [技术选型](#7-技术选型)
8. [实施路线图](#8-实施路线图)
9. [兼容性与迁移](#9-兼容性与迁移)
10. [附录](#10-附录)

---

## 1. 项目概述

### 1.1 项目背景

Artemis是携程开发的SOA服务注册中心，用于微服务架构中的服务注册与发现，功能类似于Netflix Eureka。当前Java版本（1.5.16）在生产环境中面临显著的GC问题，导致服务抖动和延迟不可控。

**核心痛点**：
- 频繁的心跳请求产生大量短生命周期对象
- ConcurrentHashMap扩容引发长时间GC停顿（百毫秒级）
- 垃圾回收导致心跳超时和客户端重连
- 服务稳定性和响应延迟受GC影响

### 1.2 重写目标

**主要目标**：
1. **消除GC问题**：利用Rust无GC特性，实现确定性延迟
2. **提升性能**：降低延迟，提高吞吐量，优化内存使用
3. **保持兼容**：API和协议兼容，支持平滑迁移
4. **增强稳定性**：更好的并发控制和错误处理

**次要目标**：
1. 简化配置管理
2. 改进可观测性
3. 降低运维复杂度
4. 优化资源利用率

### 1.3 核心特性

- ✅ 服务自注册、自发现
- ✅ 实例变更实时推送（WebSocket）
- ✅ 实例拉入/拉出管理
- ✅ 分组路由（Group Routing）
- ✅ 多区域/多Zone支持
- ✅ 集群节点间数据复制
- ✅ 租约机制和自动过期
- ✅ 版本化缓存和增量同步
- ✅ 限流保护

### 1.4 性能指标

**目标指标**（相比Java版本）：
- P99延迟 < 10ms（Java版本：50-200ms）
- 心跳处理能力 > 100k QPS/实例
- 内存占用减少 50%+
- 无GC停顿
- 支持 100k+ 服务实例

---

## 2. 系统架构

### 2.1 总体架构

```
┌─────────────────────────────────────────────────────────────┐
│                    客户端层 (Client SDK)                     │
│  ┌──────────────────┐              ┌───────────────────┐   │
│  │ Registry Client  │              │ Discovery Client  │   │
│  │  - 注册实例      │              │  - 查询服务       │   │
│  │  - 自动心跳      │              │  - 本地缓存       │   │
│  │  - 失败重试      │              │  - 变更监听       │   │
│  └──────────────────┘              └───────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                            ▲
                            │ HTTP/HTTPS + WebSocket
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    API网关层 (Web Layer)                     │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ REST API Controllers + WebSocket Handlers            │  │
│  │  - 注册/发现端点  - WebSocket推送  - 管理端点       │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                            ▲
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                  核心服务层 (Service Layer)                  │
│  ┌──────────────┐  ┌───────────────┐  ┌────────────────┐  │
│  │Registry      │  │Discovery      │  │Management      │  │
│  │Service       │  │Service        │  │Service         │  │
│  │- 注册/注销   │  │- 服务查询     │  │- 实例操作      │  │
│  │- 心跳续约    │  │- 增量同步     │  │- 分组管理      │  │
│  │- 限流保护    │  │- 版本缓存     │  │- 路由规则      │  │
│  └──────────────┘  └───────────────┘  └────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                            ▲
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                  数据存储层 (Storage Layer)                  │
│  ┌──────────────────┐              ┌───────────────────┐   │
│  │ 内存存储          │              │ MySQL持久化       │   │
│  │ - 服务注册表     │              │ - 管理数据        │   │
│  │ - 租约管理       │              │ - 操作日志        │   │
│  │ - 版本缓存       │              │ - 配置数据        │   │
│  │ - 变更队列       │              │                   │   │
│  └──────────────────┘              └───────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                            ▲
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                  集群管理层 (Cluster Layer)                  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ 节点管理 | 数据复制 | 健康检查 | 状态同步             │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 模块划分

**Rust重写的模块结构**：

```
artemis/
├── artemis-core/           # 核心库 (lib)
│   ├── model/             # 数据模型
│   ├── traits/            # 核心trait定义
│   ├── error/             # 错误类型
│   ├── config/            # 配置管理
│   └── utils/             # 工具函数
│
├── artemis-server/         # 服务端 (bin + lib)
│   ├── registry/          # 注册服务
│   ├── discovery/         # 发现服务
│   ├── lease/             # 租约管理
│   ├── cache/             # 版本缓存
│   ├── cluster/           # 集群管理
│   ├── replication/       # 数据复制
│   ├── ratelimiter/       # 限流器
│   └── storage/           # 存储抽象
│
├── artemis-web/            # Web层 (bin)
│   ├── api/               # REST API
│   ├── websocket/         # WebSocket处理
│   └── middleware/        # 中间件
│
├── artemis-management/     # 管理功能 (lib)
│   ├── instance/          # 实例操作
│   ├── server/            # 服务器操作
│   ├── group/             # 分组管理
│   ├── route/             # 路由规则
│   └── dao/               # 数据访问
│
├── artemis-client/         # 客户端SDK (lib)
│   ├── registry/          # 注册客户端
│   ├── discovery/         # 发现客户端
│   └── websocket/         # WebSocket客户端
│
└── artemis-cli/            # 命令行工具 (bin)
```

### 2.3 数据流

**注册流程**：
```
Client → HTTP POST → API Layer → RateLimiter → RegistryService
  → RegistryRepository → LeaseManager → ClusterReplication → Response
```

**心跳流程**：
```
Client → HTTP POST → API Layer → RegistryService
  → LeaseManager.renew() → (optional) Replication → Response
```

**发现流程**：
```
Client → HTTP POST/GET → API Layer → DiscoveryService
  → VersionedCache → DiscoveryFilter → Response
```

**实时推送流程**：
```
RegistryChange → ChangeQueue → InstanceChangeManager
  → WebSocket Publisher → Client WebSocket Handler
```

---

## 3. 核心数据模型

### 3.1 Instance（服务实例）

**描述**: 代表一个服务实例的完整信息。

**Rust定义**：
```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Instance {
    /// 区域ID
    pub region_id: String,

    /// 可用区ID
    pub zone_id: String,

    /// 分组ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,

    /// 服务ID（大小写不敏感）
    pub service_id: String,

    /// 实例唯一标识
    pub instance_id: String,

    /// 机器名
    #[serde(skip_serializing_if = "Option::is_none")]
    pub machine_name: Option<String>,

    /// IP地址
    pub ip: String,

    /// 端口
    pub port: u16,

    /// 协议 (http/https)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,

    /// 访问URL
    pub url: String,

    /// 健康检查URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_check_url: Option<String>,

    /// 实例状态
    pub status: InstanceStatus,

    /// 元数据
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

/// 实例唯一键（用于索引）
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct InstanceKey {
    pub region_id: String,
    pub zone_id: String,
    pub service_id: String,
    pub group_id: String,
    pub instance_id: String,
}

impl Instance {
    /// 获取实例的唯一键
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

**设计说明**：
- `service_id` 大小写不敏感，统一转小写处理
- `group_id` 可选，默认为空字符串
- 使用 `InstanceKey` 作为HashMap的键
- 支持序列化/反序列化（serde）

### 3.2 Service（服务）

**描述**: 代表一个服务及其所有实例的聚合。

**Rust定义**：
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    /// 服务ID
    pub service_id: String,

    /// 服务元数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,

    /// 物理实例列表
    pub instances: Vec<Instance>,

    /// 逻辑实例列表（用于管理）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logic_instances: Option<Vec<Instance>>,

    /// 路由规则列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_rules: Option<Vec<RouteRule>>,
}
```

### 3.3 Lease（租约）

**描述**: 用于管理实例生命周期的租约机制。

**Rust定义**：
```rust
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct Lease {
    /// 实例键
    key: InstanceKey,

    /// 创建时间
    creation_time: Instant,

    /// 最近续约时间
    renewal_time: Arc<Mutex<Instant>>,

    /// 驱逐时间（可选）
    eviction_time: Arc<Mutex<Option<Instant>>>,

    /// 生存时间（TTL）
    ttl: Duration,
}

impl Lease {
    /// 创建新租约
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

    /// 续约
    pub async fn renew(&self) {
        let mut renewal_time = self.renewal_time.lock().await;
        *renewal_time = Instant::now();
    }

    /// 检查是否过期
    pub async fn is_expired(&self) -> bool {
        let renewal_time = self.renewal_time.lock().await;
        renewal_time.elapsed() > self.ttl
    }

    /// 标记为已驱逐
    pub async fn mark_evicted(&self) {
        let mut eviction_time = self.eviction_time.lock().await;
        *eviction_time = Some(Instant::now());
    }
}
```

**配置参数**：
- 普通实例TTL：20秒
- 遗留实例TTL：90秒
- 清理任务间隔：1秒

### 3.4 RouteRule（路由规则）

**描述**: 服务的路由规则配置。

**Rust定义**：
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRule {
    /// 路由规则ID
    pub route_id: String,

    /// 路由策略
    pub strategy: RouteStrategy,

    /// 服务分组列表
    pub groups: Vec<ServiceGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RouteStrategy {
    /// 加权轮询
    WeightedRoundRobin,

    /// 就近访问
    CloseByVisit,
}
```

### 3.5 ServiceGroup（服务分组）

**描述**: 服务的分组配置，用于路由和管理。

**Rust定义**：
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceGroup {
    /// 分组键
    pub group_key: String,

    /// 权重（用于加权路由）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<u32>,

    /// 实例ID列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_ids: Option<Vec<String>>,

    /// 实例列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instances: Option<Vec<Instance>>,

    /// 元数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}
```

### 3.6 InstanceChange（实例变更）

**描述**: 实例变更事件，用于推送通知。

**Rust定义**：
```rust
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceChange {
    /// 变更的实例
    pub instance: Instance,

    /// 变更类型
    pub change_type: ChangeType,

    /// 变更时间
    pub change_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChangeType {
    /// 新增实例
    New,

    /// 删除实例
    Delete,

    /// 实例属性变更
    Change,

    /// 重新加载
    Reload,
}
```

### 3.7 请求/响应模型

**注册请求**：
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    /// 要注册的实例列表
    pub instances: Vec<Instance>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterResponse {
    /// 响应状态
    pub response_status: ResponseStatus,

    /// 注册失败的实例
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_instances: Option<Vec<Instance>>,
}
```

**心跳请求**：
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct HeartbeatRequest {
    /// 实例键列表
    pub instance_keys: Vec<InstanceKey>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeartbeatResponse {
    pub response_status: ResponseStatus,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_instance_keys: Option<Vec<InstanceKey>>,
}
```

**发现请求**：
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct GetServiceRequest {
    /// 服务发现配置
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
```

**通用响应状态**：
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseStatus {
    pub error_code: ErrorCode,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
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

---

## 4. 功能模块详细规格

### 4.1 注册服务 (Registry Service)

#### 4.1.1 功能描述

负责处理服务实例的注册、心跳续约和注销操作。

#### 4.1.2 核心接口

**Trait定义**：
```rust
use async_trait::async_trait;

#[async_trait]
pub trait RegistryService: Send + Sync {
    /// 注册实例
    async fn register(&self, request: RegisterRequest) -> Result<RegisterResponse>;

    /// 心跳续约
    async fn heartbeat(&self, request: HeartbeatRequest) -> Result<HeartbeatResponse>;

    /// 注销实例
    async fn unregister(&self, request: UnregisterRequest) -> Result<UnregisterResponse>;
}
```

#### 4.1.3 处理流程

**注册流程**：
1. **限流检查**：使用限流器检查请求频率
2. **参数验证**：验证实例信息完整性
3. **存储实例**：将实例存入内存仓储
4. **创建租约**：为实例创建租约
5. **集群复制**：异步复制到其他节点
6. **返回响应**：包含成功/失败的实例列表

**心跳流程**：
1. **限流检查**
2. **查找租约**：根据实例键查找租约
3. **续约**：更新租约的续约时间
4. **集群复制**（可选）
5. **返回响应**

**注销流程**：
1. **限流检查**
2. **移除实例**：从内存仓储删除
3. **删除租约**：删除对应租约
4. **推送变更**：推送DELETE类型的实例变更
5. **集群复制**
6. **返回响应**

#### 4.1.4 数据存储

**RegistryRepository接口**：
```rust
use dashmap::DashMap;
use std::sync::Arc;

pub struct RegistryRepository {
    /// 服务映射：ServiceId -> Service
    services: Arc<DashMap<String, Service>>,

    /// 实例映射：InstanceKey -> Instance
    instances: Arc<DashMap<InstanceKey, Instance>>,

    /// 租约管理器
    lease_manager: Arc<LeaseManager>,

    /// 遗留租约管理器
    legacy_lease_manager: Arc<LeaseManager>,

    /// 实例变更队列
    instance_changes: Arc<InstanceChangeQueue>,
}

impl RegistryRepository {
    /// 注册实例
    pub async fn register_instance(&self, instance: Instance) -> Result<()> {
        let key = instance.key();

        // 存储实例
        self.instances.insert(key.clone(), instance.clone());

        // 更新服务
        self.update_service(&instance).await?;

        // 创建租约
        self.lease_manager.register_lease(key, instance).await?;

        // 推送变更
        self.instance_changes.push(InstanceChange {
            instance,
            change_type: ChangeType::New,
            change_time: Utc::now(),
        }).await;

        Ok(())
    }

    /// 续约
    pub async fn renew_lease(&self, key: &InstanceKey) -> Result<()> {
        self.lease_manager.renew_lease(key).await
    }

    /// 注销实例
    pub async fn unregister_instance(&self, key: &InstanceKey) -> Result<Instance> {
        // 移除实例
        let instance = self.instances.remove(key)
            .ok_or(Error::InstanceNotFound)?
            .1;

        // 更新服务
        self.update_service_after_removal(key).await?;

        // 删除租约
        self.lease_manager.cancel_lease(key).await?;

        // 推送变更
        self.instance_changes.push(InstanceChange {
            instance: instance.clone(),
            change_type: ChangeType::Delete,
            change_time: Utc::now(),
        }).await;

        Ok(instance)
    }

    /// 更新服务（添加实例后）
    async fn update_service(&self, instance: &Instance) -> Result<()> {
        let service_id = instance.service_id.to_lowercase();

        self.services.entry(service_id.clone())
            .and_modify(|service| {
                // 更新现有实例或添加新实例
                if let Some(pos) = service.instances.iter()
                    .position(|i| i.key() == instance.key()) {
                    service.instances[pos] = instance.clone();
                } else {
                    service.instances.push(instance.clone());
                }
            })
            .or_insert_with(|| Service {
                service_id,
                metadata: None,
                instances: vec![instance.clone()],
                logic_instances: None,
                route_rules: None,
            });

        Ok(())
    }
}
```

**设计要点**：
- 使用 `DashMap` 实现高并发的HashMap
- 服务和实例分别存储，便于查询
- 变更队列使用有序集合（按时间戳）
- 支持两种租约管理器（普通和遗留）

#### 4.1.5 限流保护

**RateLimiter实现**：
```rust
use std::sync::Arc;
use governor::{Quota, RateLimiter as GovernorRateLimiter, DefaultDirectRateLimiter};
use nonzero_ext::*;

pub struct RateLimiter {
    register_limiter: Arc<DefaultDirectRateLimiter>,
    heartbeat_limiter: Arc<DefaultDirectRateLimiter>,
    unregister_limiter: Arc<DefaultDirectRateLimiter>,
}

impl RateLimiter {
    pub fn new(config: &RateLimiterConfig) -> Self {
        Self {
            register_limiter: Arc::new(
                GovernorRateLimiter::direct(
                    Quota::per_second(nonzero!(config.register_qps))
                )
            ),
            heartbeat_limiter: Arc::new(
                GovernorRateLimiter::direct(
                    Quota::per_second(nonzero!(config.heartbeat_qps))
                )
            ),
            unregister_limiter: Arc::new(
                GovernorRateLimiter::direct(
                    Quota::per_second(nonzero!(config.unregister_qps))
                )
            ),
        }
    }

    pub fn check_register(&self) -> Result<()> {
        self.register_limiter.check()
            .map_err(|_| Error::RateLimited)?;
        Ok(())
    }
}
```

**配置参数**：
- `register_qps`: 注册请求QPS限制（默认：10000）
- `heartbeat_qps`: 心跳请求QPS限制（默认：100000）
- `unregister_qps`: 注销请求QPS限制（默认：10000）

### 4.2 租约管理 (Lease Manager)

#### 4.2.1 功能描述

管理实例的生命周期，通过租约机制实现实例的自动过期和清理。

#### 4.2.2 核心接口

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

pub struct LeaseManager {
    /// 租约映射：InstanceKey -> Lease
    leases: Arc<DashMap<InstanceKey, Arc<Lease>>>,

    /// TTL配置
    ttl: Duration,

    /// 清理任务间隔
    clean_interval: Duration,

    /// 事件监听器
    listeners: Arc<RwLock<Vec<Arc<dyn LeaseEventListener>>>>,
}

#[async_trait]
pub trait LeaseEventListener: Send + Sync {
    /// 租约过期事件
    async fn on_lease_expired(&self, key: &InstanceKey, instance: &Instance);
}

impl LeaseManager {
    pub fn new(config: LeaseManagerConfig) -> Self {
        Self {
            leases: Arc::new(DashMap::new()),
            ttl: config.ttl,
            clean_interval: config.clean_interval,
            listeners: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 注册租约
    pub async fn register_lease(&self, key: InstanceKey, instance: Instance) -> Result<()> {
        let lease = Arc::new(Lease::new(key.clone(), self.ttl));
        self.leases.insert(key, lease);
        Ok(())
    }

    /// 续约
    pub async fn renew_lease(&self, key: &InstanceKey) -> Result<()> {
        let lease = self.leases.get(key)
            .ok_or(Error::LeaseNotFound)?;
        lease.renew().await;
        Ok(())
    }

    /// 取消租约
    pub async fn cancel_lease(&self, key: &InstanceKey) -> Result<()> {
        self.leases.remove(key);
        Ok(())
    }

    /// 启动清理任务
    pub fn start_clean_task(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut ticker = interval(self.clean_interval);

            loop {
                ticker.tick().await;
                self.clean_expired_leases().await;
            }
        });
    }

    /// 清理过期租约
    async fn clean_expired_leases(&self) {
        let mut expired_keys = Vec::new();

        // 收集过期的租约
        for entry in self.leases.iter() {
            if entry.value().is_expired().await {
                expired_keys.push(entry.key().clone());
            }
        }

        // 通知监听器并删除
        for key in expired_keys {
            if let Some((_, lease)) = self.leases.remove(&key) {
                // 标记为已驱逐
                lease.mark_evicted().await;

                // 通知监听器
                let listeners = self.listeners.read().await;
                for listener in listeners.iter() {
                    // 这里需要从仓储获取instance
                    // listener.on_lease_expired(&key, &instance).await;
                }
            }
        }
    }

    /// 添加事件监听器
    pub async fn add_listener(&self, listener: Arc<dyn LeaseEventListener>) {
        self.listeners.write().await.push(listener);
    }
}
```

#### 4.2.3 租约安全检查

**LeaseUpdateSafeChecker**：
```rust
use std::time::{Duration, Instant};

pub struct LeaseUpdateSafeChecker {
    /// 安全时间窗口
    safe_window: Duration,

    /// 上次检查时间
    last_check: Arc<Mutex<Instant>>,
}

impl LeaseUpdateSafeChecker {
    pub fn new(safe_window: Duration) -> Self {
        Self {
            safe_window,
            last_check: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// 检查当前是否可以安全删除租约
    pub async fn is_safe_to_delete(&self) -> bool {
        let last = self.last_check.lock().await;
        last.elapsed() >= self.safe_window
    }

    /// 更新检查时间
    pub async fn update_check_time(&self) {
        let mut last = self.last_check.lock().await;
        *last = Instant::now();
    }
}
```

**用途**：防止网络抖动导致大量实例被误删除。

#### 4.2.4 配置参数

```toml
[lease_manager]
# 租约TTL（秒）
ttl = 20

# 清理任务运行间隔（毫秒）
clean_interval = 1000

# 初始容量
initial_capacity = 50000

# 安全窗口（秒）
safe_window = 30
```

### 4.3 发现服务 (Discovery Service)

#### 4.3.1 功能描述

提供服务查询、全量/增量同步功能，支持版本化缓存和过滤器链。

#### 4.3.2 核心接口

```rust
#[async_trait]
pub trait DiscoveryService: Send + Sync {
    /// 批量查询服务
    async fn lookup(&self, request: LookupRequest) -> Result<LookupResponse>;

    /// 获取单个服务
    async fn get_service(&self, request: GetServiceRequest) -> Result<GetServiceResponse>;

    /// 获取所有服务
    async fn get_services(&self, request: GetServicesRequest) -> Result<GetServicesResponse>;

    /// 获取服务增量
    async fn get_services_delta(&self, request: GetServicesDeltaRequest)
        -> Result<GetServicesDeltaResponse>;
}
```

#### 4.3.3 版本化缓存

**VersionedCache设计**：
```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::BTreeMap;

pub struct VersionedCacheManager {
    /// 版本化数据：Timestamp -> Services
    versions: Arc<RwLock<BTreeMap<i64, Arc<Vec<Service>>>>>,

    /// 最大保留版本数
    max_versions: usize,

    /// 刷新间隔
    refresh_interval: Duration,

    /// 数据源
    repository: Arc<RegistryRepository>,
}

impl VersionedCacheManager {
    pub fn new(config: CacheConfig, repository: Arc<RegistryRepository>) -> Self {
        Self {
            versions: Arc::new(RwLock::new(BTreeMap::new())),
            max_versions: config.max_versions,
            refresh_interval: config.refresh_interval,
            repository,
        }
    }

    /// 启动定时刷新任务
    pub fn start_refresh_task(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut ticker = interval(self.refresh_interval);

            loop {
                ticker.tick().await;
                if let Err(e) = self.refresh_cache().await {
                    error!("Failed to refresh cache: {:?}", e);
                }
            }
        });
    }

    /// 刷新缓存
    async fn refresh_cache(&self) -> Result<()> {
        // 从仓储获取所有服务
        let services = self.repository.get_all_services().await?;

        // 生成新版本
        let version = Utc::now().timestamp_millis();
        let services_arc = Arc::new(services);

        // 更新版本映射
        let mut versions = self.versions.write().await;
        versions.insert(version, services_arc);

        // 清理旧版本（保留最新的N个）
        while versions.len() > self.max_versions {
            if let Some(oldest) = versions.keys().next().copied() {
                versions.remove(&oldest);
            }
        }

        Ok(())
    }

    /// 获取最新版本
    pub async fn get_latest(&self) -> Result<(i64, Arc<Vec<Service>>)> {
        let versions = self.versions.read().await;
        versions.iter().next_back()
            .map(|(v, s)| (*v, Arc::clone(s)))
            .ok_or(Error::CacheEmpty)
    }

    /// 获取增量（从指定版本到最新版本的变更）
    pub async fn get_delta(&self, from_version: i64)
        -> Result<(i64, HashMap<String, Vec<InstanceChange>>)> {
        let versions = self.versions.read().await;

        // 获取from_version的数据
        let old_services = versions.range(..=from_version).next_back()
            .map(|(_, s)| s)
            .ok_or(Error::VersionNotFound)?;

        // 获取最新版本的数据
        let (new_version, new_services) = versions.iter().next_back()
            .ok_or(Error::CacheEmpty)?;

        // 计算差异
        let delta = self.compute_delta(old_services, new_services).await;

        Ok((*new_version, delta))
    }

    /// 计算两个版本之间的差异
    async fn compute_delta(
        &self,
        old: &[Service],
        new: &[Service]
    ) -> HashMap<String, Vec<InstanceChange>> {
        // 实现差异计算逻辑
        // 返回每个服务的实例变更列表
        todo!()
    }
}
```

**配置参数**：
```toml
[versioned_cache]
# 刷新间隔（秒）
refresh_interval = 30

# 最大保留版本数
max_versions = 3
```

#### 4.3.4 发现过滤器

**Filter接口**：
```rust
#[async_trait]
pub trait DiscoveryFilter: Send + Sync {
    /// 过滤服务
    async fn filter(&self, service: &mut Service, config: &DiscoveryConfig) -> Result<()>;
}

pub struct DiscoveryFilterChain {
    filters: Vec<Arc<dyn DiscoveryFilter>>,
}

impl DiscoveryFilterChain {
    pub fn new() -> Self {
        Self {
            filters: vec![
                Arc::new(GroupDiscoveryFilter::new()),
                Arc::new(ManagementDiscoveryFilter::new()),
            ],
        }
    }

    pub async fn apply(&self, service: &mut Service, config: &DiscoveryConfig) -> Result<()> {
        for filter in &self.filters {
            filter.filter(service, config).await?;
        }
        Ok(())
    }
}
```

**GroupDiscoveryFilter**：根据分组过滤实例
**ManagementDiscoveryFilter**：应用管理规则（拉入/拉出）

#### 4.3.5 处理流程

**GetService流程**：
1. **节点状态检查**：确认节点可提供发现服务
2. **Zone权限检查**：验证是否允许跨Zone访问
3. **获取缓存数据**：从版本缓存获取服务
4. **应用过滤器**：执行过滤器链
5. **返回响应**

**GetServicesDelta流程**：
1. 状态和权限检查
2. **获取增量**：从版本缓存计算差异
3. 应用过滤器
4. **构造变更映射**：按服务组织InstanceChange
5. 返回响应

### 4.4 实时推送 (WebSocket Push)

#### 4.4.1 功能描述

通过WebSocket实现服务变更的实时推送，支持客户端订阅特定服务。

#### 4.4.2 WebSocket处理器

**Axum实现示例**：
```rust
use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade, Message},
        State,
    },
    response::Response,
};
use std::sync::Arc;

pub struct WebSocketHandler {
    session_manager: Arc<SessionManager>,
    change_manager: Arc<InstanceChangeManager>,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(handler): State<Arc<WebSocketHandler>>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, handler))
}

async fn handle_socket(socket: WebSocket, handler: Arc<WebSocketHandler>) {
    let (mut sender, mut receiver) = socket.split();

    // 接收订阅消息
    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            // 解析订阅请求
            if let Ok(subscribe) = serde_json::from_str::<SubscribeRequest>(&text) {
                // 注册会话
                let session_id = handler.session_manager
                    .register_session(&subscribe.service_id, sender).await;

                // 订阅变更推送
                handler.change_manager
                    .subscribe(&subscribe.service_id, session_id).await;
            }
        }
    }
}
```

#### 4.4.3 会话管理

```rust
use tokio::sync::mpsc;
use futures::stream::SplitSink;

pub struct SessionManager {
    /// ServiceId -> SessionId -> Sender
    sessions: Arc<DashMap<String, DashMap<String, SessionSender>>>,
}

pub type SessionSender = mpsc::UnboundedSender<Message>;

impl SessionManager {
    pub async fn register_session(
        &self,
        service_id: &str,
        sender: SessionSender
    ) -> String {
        let session_id = Uuid::new_v4().to_string();

        self.sessions.entry(service_id.to_string())
            .or_insert_with(DashMap::new)
            .insert(session_id.clone(), sender);

        session_id
    }

    pub async fn unregister_session(&self, service_id: &str, session_id: &str) {
        if let Some(sessions) = self.sessions.get(service_id) {
            sessions.remove(session_id);
        }
    }

    pub async fn broadcast(&self, service_id: &str, change: &InstanceChange) -> Result<()> {
        if let Some(sessions) = self.sessions.get(service_id) {
            let message = Message::Text(serde_json::to_string(change)?);

            for entry in sessions.iter() {
                let _ = entry.value().send(message.clone());
            }
        }
        Ok(())
    }
}
```

#### 4.4.4 变更管理器

```rust
pub struct InstanceChangeManager {
    /// 变更队列（从RegistryRepository轮询）
    change_queue: Arc<InstanceChangeQueue>,

    /// 会话管理器
    session_manager: Arc<SessionManager>,

    /// 轮询间隔
    poll_interval: Duration,
}

impl InstanceChangeManager {
    pub fn start_poll_task(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut ticker = interval(self.poll_interval);

            loop {
                ticker.tick().await;
                self.poll_and_push().await;
            }
        });
    }

    async fn poll_and_push(&self) {
        // 从队列获取变更
        let changes = self.change_queue.drain().await;

        // 按服务ID分组
        let mut grouped: HashMap<String, Vec<InstanceChange>> = HashMap::new();
        for change in changes {
            grouped.entry(change.instance.service_id.clone())
                .or_default()
                .push(change);
        }

        // 推送到对应的会话
        for (service_id, changes) in grouped {
            for change in changes {
                let _ = self.session_manager.broadcast(&service_id, &change).await;
            }
        }
    }
}
```

#### 4.4.5 WebSocket端点

- `/ws/service-change` - 单个服务变更订阅
- `/ws/all-services-change` - 所有服务变更订阅
- `/ws/heartbeat` - 心跳保活
- `/ws/metric` - 指标推送

### 4.5 集群管理 (Cluster Management)

#### 4.5.1 功能描述

管理集群节点，实现节点发现、健康检查、状态同步。

#### 4.5.2 节点管理

**Node结构**：
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// 节点ID
    pub node_id: String,

    /// Zone ID
    pub zone_id: String,

    /// 节点URL
    pub url: String,

    /// 节点状态
    pub status: NodeStatus,

    /// 是否可服务注册
    pub can_service_registry: bool,

    /// 是否可服务发现
    pub can_service_discovery: bool,

    /// 元数据
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum NodeStatus {
    Starting,
    Up,
    Down,
    Unknown,
}
```

**NodeManager**：
```rust
pub struct NodeManager {
    /// 本地节点
    local_node: Arc<RwLock<Node>>,

    /// 集群管理器
    cluster_manager: Arc<ClusterManager>,

    /// 配置
    config: NodeConfig,
}

impl NodeManager {
    pub async fn initialize(&self) -> Result<()> {
        // 初始化本地节点
        let mut node = self.local_node.write().await;
        node.status = NodeStatus::Starting;

        // 执行初始化检查
        self.run_initializers().await?;

        // 标记为UP
        node.status = NodeStatus::Up;
        node.can_service_registry = true;
        node.can_service_discovery = true;

        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        let mut node = self.local_node.write().await;
        node.status = NodeStatus::Down;
        node.can_service_registry = false;
        node.can_service_discovery = false;

        Ok(())
    }

    pub async fn get_status(&self) -> NodeStatus {
        self.local_node.read().await.status
    }

    pub async fn can_service_registry(&self) -> bool {
        self.local_node.read().await.can_service_registry
    }

    pub async fn can_service_discovery(&self) -> bool {
        self.local_node.read().await.can_service_discovery
    }
}
```

#### 4.5.3 集群管理

**ClusterManager**：
```rust
pub struct ClusterManager {
    /// 集群节点：ZoneId -> Vec<Node>
    nodes: Arc<RwLock<HashMap<String, Vec<Node>>>>,

    /// 变更监听器
    listeners: Arc<RwLock<Vec<Arc<dyn ClusterChangeListener>>>>,

    /// 配置源（支持动态更新）
    config_source: Arc<dyn ConfigSource>,
}

#[async_trait]
pub trait ClusterChangeListener: Send + Sync {
    async fn on_cluster_changed(&self, cluster: &ServiceCluster);
}

impl ClusterManager {
    pub async fn get_cluster(&self) -> ServiceCluster {
        let nodes = self.nodes.read().await;

        ServiceCluster {
            nodes: nodes.clone(),
        }
    }

    pub async fn get_nodes_by_zone(&self, zone_id: &str) -> Vec<Node> {
        let nodes = self.nodes.read().await;
        nodes.get(zone_id).cloned().unwrap_or_default()
    }

    pub async fn get_up_registry_nodes(&self) -> Vec<Node> {
        let nodes = self.nodes.read().await;
        nodes.values()
            .flatten()
            .filter(|n| n.status == NodeStatus::Up && n.can_service_registry)
            .cloned()
            .collect()
    }

    pub async fn refresh_from_config(&self) -> Result<()> {
        // 从配置源刷新节点列表
        let new_nodes = self.config_source.load_cluster_nodes().await?;

        let mut nodes = self.nodes.write().await;
        *nodes = new_nodes;

        // 通知监听器
        let cluster = ServiceCluster {
            nodes: nodes.clone(),
        };

        let listeners = self.listeners.read().await;
        for listener in listeners.iter() {
            listener.on_cluster_changed(&cluster).await;
        }

        Ok(())
    }
}
```

#### 4.5.4 健康检查

```rust
pub struct HealthChecker {
    cluster_manager: Arc<ClusterManager>,
    check_interval: Duration,
    timeout: Duration,
    http_client: reqwest::Client,
}

impl HealthChecker {
    pub fn start_check_task(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut ticker = interval(self.check_interval);

            loop {
                ticker.tick().await;
                self.check_all_nodes().await;
            }
        });
    }

    async fn check_all_nodes(&self) {
        let cluster = self.cluster_manager.get_cluster().await;

        for nodes in cluster.nodes.values() {
            for node in nodes {
                let is_healthy = self.check_node(node).await;

                // 更新节点状态
                if !is_healthy && node.status == NodeStatus::Up {
                    // 标记为DOWN
                    self.mark_node_down(&node.node_id).await;
                } else if is_healthy && node.status == NodeStatus::Down {
                    // 恢复为UP
                    self.mark_node_up(&node.node_id).await;
                }
            }
        }
    }

    async fn check_node(&self, node: &Node) -> bool {
        let health_url = format!("{}/api/status/node.json", node.url);

        match timeout(self.timeout, self.http_client.get(&health_url).send()).await {
            Ok(Ok(response)) if response.status().is_success() => true,
            _ => false,
        }
    }
}
```

### 4.6 数据复制 (Replication)

#### 4.6.1 功能描述

将注册、心跳、注销操作异步复制到集群的其他节点，保证数据一致性。

#### 4.6.2 复制管理器

**RegistryReplicationManager**：
```rust
pub struct RegistryReplicationManager {
    /// 批量任务执行器
    batching_executor: Arc<TaskExecutor<RegisterTask>>,

    /// 单任务执行器
    single_executor: Arc<TaskExecutor<RegisterTask>>,

    /// 集群管理器
    cluster_manager: Arc<ClusterManager>,

    /// HTTP客户端
    http_client: Arc<ReplicationHttpClient>,
}

impl RegistryReplicationManager {
    /// 复制注册操作
    pub async fn replicate_register(&self, instances: Vec<Instance>) -> Result<()> {
        let nodes = self.cluster_manager.get_up_registry_nodes().await;

        for node in nodes {
            let task = RegisterTask {
                target_node: node.clone(),
                instances: instances.clone(),
            };

            // 加入批量执行器队列
            self.batching_executor.submit(task).await?;
        }

        Ok(())
    }

    /// 复制心跳操作
    pub async fn replicate_heartbeat(&self, keys: Vec<InstanceKey>) -> Result<()> {
        let nodes = self.cluster_manager.get_up_registry_nodes().await;

        for node in nodes {
            let task = HeartbeatTask {
                target_node: node.clone(),
                instance_keys: keys.clone(),
            };

            self.batching_executor.submit(task).await?;
        }

        Ok(())
    }

    /// 复制注销操作
    pub async fn replicate_unregister(&self, keys: Vec<InstanceKey>) -> Result<()> {
        let nodes = self.cluster_manager.get_up_registry_nodes().await;

        for node in nodes {
            let task = UnregisterTask {
                target_node: node.clone(),
                instance_keys: keys.clone(),
            };

            self.single_executor.submit(task).await?;
        }

        Ok(())
    }
}
```

#### 4.6.3 任务执行器

**通用TaskExecutor**：
```rust
use tokio::sync::mpsc;

pub struct TaskExecutor<T> {
    /// 任务队列
    task_queue: mpsc::UnboundedSender<T>,

    /// 工作线程数
    worker_count: usize,
}

impl<T> TaskExecutor<T>
where
    T: Task + Send + 'static,
{
    pub fn new(config: ExecutorConfig, processor: Arc<dyn TaskProcessor<T>>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        // 启动工作线程
        for _ in 0..config.worker_count {
            let rx_clone = rx.clone();
            let processor_clone = Arc::clone(&processor);

            tokio::spawn(async move {
                Self::worker_loop(rx_clone, processor_clone).await;
            });
        }

        Self {
            task_queue: tx,
            worker_count: config.worker_count,
        }
    }

    pub async fn submit(&self, task: T) -> Result<()> {
        self.task_queue.send(task)
            .map_err(|_| Error::TaskQueueFull)?;
        Ok(())
    }

    async fn worker_loop(
        mut rx: mpsc::UnboundedReceiver<T>,
        processor: Arc<dyn TaskProcessor<T>>
    ) {
        while let Some(task) = rx.recv().await {
            if let Err(e) = processor.process(task).await {
                error!("Task processing failed: {:?}", e);
            }
        }
    }
}
```

**BatchingTaskProcessor**：
```rust
pub struct BatchingTaskProcessor {
    http_client: Arc<ReplicationHttpClient>,
    batch_size: usize,
    batch_timeout: Duration,
}

#[async_trait]
impl TaskProcessor<RegisterTask> for BatchingTaskProcessor {
    async fn process(&self, task: RegisterTask) -> Result<()> {
        // 批量发送注册请求
        let request = RegisterRequest {
            instances: task.instances,
        };

        let url = format!("{}/api/replication/registry/register.json", task.target_node.url);

        self.http_client.post(&url, &request).await?;

        Ok(())
    }
}
```

#### 4.6.4 复制API

**与普通API的区别**：
- 路径前缀：`/api/replication/` 而非 `/api/`
- **不再次复制**：复制接收端不再触发二次复制
- **宽松验证**：减少参数验证，信任源节点

### 4.7 管理功能 (Management)

#### 4.7.1 功能描述

提供运维管理能力，包括实例拉入/拉出、服务器操作、分组管理、路由规则配置。

#### 4.7.2 实例操作

**InstanceOperationService**：
```rust
pub struct InstanceOperationService {
    dao: Arc<InstanceOperationDao>,
    discovery_filter: Arc<ManagementDiscoveryFilter>,
}

impl InstanceOperationService {
    /// 操作实例（拉入/拉出）
    pub async fn operate_instance(&self, request: OperateInstanceRequest) -> Result<()> {
        let operation = InstanceOperation {
            region_id: request.region_id,
            service_id: request.service_id,
            instance_id: request.instance_id,
            operation_type: request.operation_type,
            operator_id: request.operator_id,
            token: Uuid::new_v4().to_string(),
            is_complete: false,
            created_at: Utc::now(),
        };

        // 保存到数据库
        self.dao.insert_operation(operation).await?;

        // 记录日志
        self.dao.insert_log(InstanceOperationLog {
            operation_id: operation.token.clone(),
            message: format!("Instance {} operated: {:?}",
                operation.instance_id, operation.operation_type),
            created_at: Utc::now(),
        }).await?;

        Ok(())
    }

    /// 检查实例是否被拉出
    pub async fn is_instance_down(&self, key: &InstanceKey) -> Result<bool> {
        self.dao.is_instance_down(key).await
    }

    /// 获取实例操作
    pub async fn get_operations(&self, filter: OperationFilter) -> Result<Vec<InstanceOperation>> {
        self.dao.query_operations(filter).await
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OperateInstanceRequest {
    pub region_id: String,
    pub service_id: String,
    pub instance_id: String,
    pub operation_type: OperationType,
    pub operator_id: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OperationType {
    /// 拉出（下线）
    PullOut,

    /// 拉入（上线）
    PullIn,
}
```

**ManagementDiscoveryFilter**：
```rust
pub struct ManagementDiscoveryFilter {
    operation_service: Arc<InstanceOperationService>,
}

#[async_trait]
impl DiscoveryFilter for ManagementDiscoveryFilter {
    async fn filter(&self, service: &mut Service, _config: &DiscoveryConfig) -> Result<()> {
        // 过滤掉被拉出的实例
        let mut filtered_instances = Vec::new();

        for instance in &service.instances {
            let is_down = self.operation_service
                .is_instance_down(&instance.key())
                .await?;

            if !is_down {
                filtered_instances.push(instance.clone());
            }
        }

        service.instances = filtered_instances;
        Ok(())
    }
}
```

#### 4.7.3 服务分组

**GroupService**：
```rust
pub struct GroupService {
    dao: Arc<GroupDao>,
}

impl GroupService {
    /// 创建服务分组
    pub async fn create_group(&self, group: ServiceGroup) -> Result<()> {
        self.dao.insert_group(group).await
    }

    /// 添加实例到分组
    pub async fn add_instances_to_group(
        &self,
        group_id: &str,
        instance_ids: Vec<String>
    ) -> Result<()> {
        for instance_id in instance_ids {
            self.dao.insert_group_instance(GroupInstance {
                group_id: group_id.to_string(),
                instance_id,
                created_at: Utc::now(),
            }).await?;
        }
        Ok(())
    }

    /// 查询分组
    pub async fn get_group(&self, group_id: &str) -> Result<ServiceGroup> {
        self.dao.query_group(group_id).await
    }
}
```

#### 4.7.4 路由规则

**RouteRuleService**：
```rust
pub struct RouteRuleService {
    dao: Arc<RouteRuleDao>,
}

impl RouteRuleService {
    /// 创建路由规则
    pub async fn create_route_rule(&self, rule: RouteRule) -> Result<()> {
        // 保存路由规则
        self.dao.insert_route_rule(&rule).await?;

        // 保存规则-分组关联
        for group in &rule.groups {
            self.dao.insert_route_rule_group(RouteRuleGroup {
                route_id: rule.route_id.clone(),
                group_key: group.group_key.clone(),
                weight: group.weight,
                created_at: Utc::now(),
            }).await?;
        }

        Ok(())
    }

    /// 查询服务的路由规则
    pub async fn get_route_rules(&self, service_id: &str) -> Result<Vec<RouteRule>> {
        self.dao.query_route_rules(service_id).await
    }
}
```

#### 4.7.5 数据库Schema

**核心表结构**（MySQL）：

```sql
-- 实例操作表
CREATE TABLE instance_operation (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    region_id VARCHAR(50) NOT NULL,
    service_id VARCHAR(100) NOT NULL,
    instance_id VARCHAR(100) NOT NULL,
    operation_type VARCHAR(20) NOT NULL,
    operator_id VARCHAR(50),
    token VARCHAR(50) NOT NULL UNIQUE,
    is_complete BOOLEAN DEFAULT FALSE,
    extensions TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_service_instance (service_id, instance_id),
    INDEX idx_token (token)
);

-- 实例操作日志表
CREATE TABLE instance_operation_log (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    operation_token VARCHAR(50) NOT NULL,
    message TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_operation_token (operation_token)
);

-- 服务分组表
CREATE TABLE service_group (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    group_key VARCHAR(100) NOT NULL UNIQUE,
    service_id VARCHAR(100) NOT NULL,
    weight INT,
    extensions TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_service_id (service_id)
);

-- 分组实例关联表
CREATE TABLE service_group_instance (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    group_id VARCHAR(100) NOT NULL,
    instance_id VARCHAR(100) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE KEY uk_group_instance (group_id, instance_id),
    INDEX idx_group_id (group_id)
);

-- 路由规则表
CREATE TABLE service_route_rule (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    route_id VARCHAR(100) NOT NULL UNIQUE,
    service_id VARCHAR(100) NOT NULL,
    strategy VARCHAR(50) NOT NULL,
    extensions TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_service_id (service_id)
);

-- 路由规则-分组关联表
CREATE TABLE service_route_rule_group (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    route_id VARCHAR(100) NOT NULL,
    group_key VARCHAR(100) NOT NULL,
    weight INT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_route_id (route_id)
);
```

**DAO实现**（使用SQLx）：
```rust
use sqlx::{MySqlPool, FromRow};

pub struct InstanceOperationDao {
    pool: MySqlPool,
}

impl InstanceOperationDao {
    pub async fn insert_operation(&self, op: InstanceOperation) -> Result<()> {
        sqlx::query(
            "INSERT INTO instance_operation
             (region_id, service_id, instance_id, operation_type, operator_id, token, is_complete)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&op.region_id)
        .bind(&op.service_id)
        .bind(&op.instance_id)
        .bind(&op.operation_type)
        .bind(&op.operator_id)
        .bind(&op.token)
        .bind(op.is_complete)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn is_instance_down(&self, key: &InstanceKey) -> Result<bool> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM instance_operation
             WHERE service_id = ? AND instance_id = ?
             AND operation_type = 'pullout' AND is_complete = FALSE"
        )
        .bind(&key.service_id)
        .bind(&key.instance_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0 > 0)
    }
}
```

---

## 5. API规格

### 5.1 REST API端点

**基础路径**: `/api`

#### 5.1.1 注册API

**POST /api/registry/register.json**

注册一个或多个服务实例。

**请求体**：
```json
{
  "instances": [
    {
      "regionId": "sha",
      "zoneId": "sha-z1",
      "serviceId": "my-service",
      "instanceId": "instance-001",
      "ip": "192.168.1.10",
      "port": 8080,
      "url": "http://192.168.1.10:8080",
      "healthCheckUrl": "http://192.168.1.10:8080/health",
      "status": "up",
      "metadata": {
        "version": "1.0.0"
      }
    }
  ]
}
```

**响应**：
```json
{
  "responseStatus": {
    "errorCode": "success"
  },
  "failedInstances": []
}
```

---

**POST /api/registry/heartbeat.json**

发送心跳以续约实例租约。

**请求体**：
```json
{
  "instanceKeys": [
    {
      "regionId": "sha",
      "zoneId": "sha-z1",
      "serviceId": "my-service",
      "groupId": "",
      "instanceId": "instance-001"
    }
  ]
}
```

**响应**：
```json
{
  "responseStatus": {
    "errorCode": "success"
  },
  "failedInstanceKeys": []
}
```

---

**POST /api/registry/unregister.json**

注销实例。

**请求体**：
```json
{
  "instanceKeys": [...]
}
```

**响应**：同心跳响应

---

#### 5.1.2 发现API

**GET/POST /api/discovery/service.json**

获取单个服务的实例列表。

**请求体（POST）**：
```json
{
  "discoveryConfig": {
    "serviceId": "my-service",
    "regionId": "sha",
    "zoneId": "sha-z1"
  }
}
```

**查询参数（GET）**：
- `serviceId`
- `regionId`
- `zoneId`

**响应**：
```json
{
  "responseStatus": {
    "errorCode": "success"
  },
  "service": {
    "serviceId": "my-service",
    "instances": [
      {
        "instanceId": "instance-001",
        "ip": "192.168.1.10",
        "port": 8080,
        "status": "up",
        ...
      }
    ],
    "routeRules": [...]
  }
}
```

---

**GET/POST /api/discovery/services.json**

获取所有服务。

**请求体**：
```json
{
  "regionId": "sha",
  "zoneId": "sha-z1"
}
```

**响应**：
```json
{
  "responseStatus": {
    "errorCode": "success"
  },
  "services": [
    {...},
    {...}
  ],
  "version": 1707886723000
}
```

---

**POST /api/discovery/services-delta.json**

获取服务增量变更。

**请求体**：
```json
{
  "regionId": "sha",
  "zoneId": "sha-z1",
  "version": 1707886700000
}
```

**响应**：
```json
{
  "responseStatus": {
    "errorCode": "success"
  },
  "serviceChanges": {
    "my-service": [
      {
        "instance": {...},
        "changeType": "new",
        "changeTime": "2024-02-14T10:05:23Z"
      }
    ]
  },
  "version": 1707886723000
}
```

---

#### 5.1.3 集群API

**POST /api/cluster/nodes.json**

获取集群所有节点。

**响应**：
```json
{
  "responseStatus": {
    "errorCode": "success"
  },
  "nodes": {
    "sha-z1": [
      {
        "nodeId": "node-001",
        "zoneId": "sha-z1",
        "url": "http://192.168.1.100:8080/artemis-service",
        "status": "UP",
        "canServiceRegistry": true,
        "canServiceDiscovery": true
      }
    ]
  }
}
```

---

**POST /api/cluster/up-registry-nodes.json**

获取可提供注册服务的节点。

**POST /api/cluster/up-discovery-nodes.json**

获取可提供发现服务的节点。

---

#### 5.1.4 状态API

**POST /api/status/node.json**

获取当前节点状态。

**响应**：
```json
{
  "responseStatus": {
    "errorCode": "success"
  },
  "node": {
    "nodeId": "node-001",
    "status": "UP",
    "canServiceRegistry": true,
    "canServiceDiscovery": true
  }
}
```

---

**POST /api/status/leases.json**

获取租约统计信息。

**响应**：
```json
{
  "responseStatus": {
    "errorCode": "success"
  },
  "totalLeases": 12345,
  "expiredLeases": 10
}
```

---

#### 5.1.5 管理API

**POST /api/management/instance/operate**

操作实例（拉入/拉出）。

**请求体**：
```json
{
  "regionId": "sha",
  "serviceId": "my-service",
  "instanceId": "instance-001",
  "operationType": "pullout",
  "operatorId": "admin"
}
```

**POST /api/management/group/create**

创建服务分组。

**POST /api/management/route-rule/create**

创建路由规则。

---

### 5.2 WebSocket API

**连接端点**：

- `ws://host:port/ws/service-change`
- `ws://host:port/ws/all-services-change`
- `ws://host:port/ws/heartbeat`

**订阅消息**（客户端发送）：
```json
{
  "serviceId": "my-service",
  "regionId": "sha",
  "zoneId": "sha-z1"
}
```

**变更推送**（服务端发送）：
```json
{
  "instance": {
    "serviceId": "my-service",
    "instanceId": "instance-001",
    ...
  },
  "changeType": "new",
  "changeTime": "2024-02-14T10:05:23Z"
}
```

---

## 6. 非功能性需求

### 6.1 性能需求

**延迟**：
- P99延迟 < 10ms（注册/心跳/发现）
- P999延迟 < 50ms
- 无GC停顿

**吞吐量**：
- 单节点支持 100k+ QPS（心跳）
- 单节点支持 50k+ QPS（注册/注销）
- 单节点支持 100k+ QPS（发现查询）

**容量**：
- 支持 100k+ 服务实例
- 支持 10k+ 服务
- 内存占用 < 4GB（100k实例场景）

### 6.2 可靠性需求

**可用性**：
- 集群可用性 99.99%
- 单节点故障不影响服务
- 支持滚动升级

**数据一致性**：
- 最终一致性模型
- 复制延迟 < 100ms
- 租约过期准确性 > 99%

**容错**：
- 节点间自动故障转移
- 客户端自动重试和降级
- 网络分区恢复

### 6.3 可扩展性需求

**水平扩展**：
- 支持动态增减节点
- 无状态设计，便于扩容
- 支持跨数据中心部署

**垂直扩展**：
- 充分利用多核CPU
- 支持大内存配置

### 6.4 安全性需求

**认证授权**（可选）：
- 支持Token认证
- 支持RBAC权限控制

**传输安全**：
- 支持HTTPS
- 支持WSS（WebSocket Secure）

**数据安全**：
- 敏感配置加密存储
- 审计日志

### 6.5 可观测性需求

**日志**：
- 结构化日志（JSON格式）
- 分级日志（Error/Warn/Info/Debug）
- 请求追踪（trace_id）

**指标**：
- 请求QPS/延迟/错误率
- 租约数量/过期率
- 内存/CPU使用率
- 集群节点状态

**监控**：
- Prometheus集成
- 健康检查端点
- 自定义指标导出

**追踪**：
- OpenTelemetry集成（可选）
- 分布式追踪

### 6.6 可维护性需求

**配置管理**：
- 支持配置文件（TOML/YAML）
- 支持环境变量覆盖
- 部分配置支持热更新

**部署**：
- 容器化部署（Docker）
- K8s支持
- 单二进制文件

**运维**：
- 优雅启动/关闭
- 健康检查端点
- 管理API

---

## 7. 技术选型

### 7.1 Web框架

**推荐**: Axum

**理由**：
- 基于Tokio，性能优秀
- 类型安全的路由和处理器
- 中间件系统强大
- 与Tower生态集成
- WebSocket支持完善

**备选**: Actix-web（更成熟，但类型安全性稍弱）

### 7.2 异步运行时

**选择**: Tokio

**理由**：
- Rust异步事实标准
- 生态最完善
- 高性能
- 稳定可靠

### 7.3 并发数据结构

**DashMap**: 并发HashMap
- 高性能
- 简单易用
- 适合读写混合场景

**Crossbeam**: 无锁队列和通道
- 用于任务调度
- 高吞吐

**parking_lot**: 更快的Mutex/RwLock
- 替代标准库锁
- 性能更优

### 7.4 数据库

**MySQL**: 管理数据持久化
- 使用SQLx（异步、编译期检查）
- 连接池（sqlx::Pool）

### 7.5 序列化

**Serde + serde_json**
- JSON序列化/反序列化
- 性能优秀
- 生态完善

### 7.6 HTTP客户端

**Reqwest**
- 异步HTTP客户端
- 连接池支持
- 易用性好

### 7.7 WebSocket

**tokio-tungstenite**
- Tokio集成
- 异步WebSocket
- 客户端/服务端支持

### 7.8 配置管理

**config-rs** + **figment**
- 多源配置（文件、环境变量）
- 类型安全
- 热更新支持

### 7.9 日志

**tracing + tracing-subscriber**
- 结构化日志
- 异步性能好
- 生态丰富（与OpenTelemetry集成）

### 7.10 指标

**Prometheus Client**
- metrics库
- 导出标准Prometheus指标

### 7.11 限流

**governor**
- 基于Token Bucket算法
- 高性能
- 易用

### 7.12 时间处理

**chrono** 或 **time**
- 日期时间处理
- 时区支持

### 7.13 UUID

**uuid**
- 生成唯一标识

### 7.14 错误处理

**thiserror** + **anyhow**
- thiserror：定义错误类型
- anyhow：错误传播

### 7.15 测试

**tokio::test**: 异步测试
**mockall**: Mock框架
**criterion**: 性能基准测试

---

## 8. 实施路线图

### 8.1 MVP阶段（第1-2个月）

**目标**: 实现核心功能，单节点运行

**里程碑**：
1. **Week 1-2**: 核心数据模型和错误处理
   - 定义Instance、Service、Lease等核心类型
   - 实现序列化/反序列化
   - 错误类型定义

2. **Week 3-4**: 注册服务
   - RegistryService trait和实现
   - RegistryRepository（内存存储）
   - 基础限流器

3. **Week 5-6**: 租约管理
   - LeaseManager实现
   - 定时清理任务
   - 租约安全检查器

4. **Week 7-8**: 发现服务和Web层
   - DiscoveryService实现
   - REST API端点（Axum）
   - 基本的健康检查

**交付物**：
- 可运行的单节点Artemis服务器
- 支持注册、心跳、注销、查询API
- 通过基础功能测试

### 8.2 完整功能阶段（第3-4个月）

**目标**: 完善功能，支持集群

**里程碑**：
1. **Week 9-10**: 版本化缓存
   - VersionedCacheManager
   - 增量同步API
   - Delta计算

2. **Week 11-12**: WebSocket推送
   - WebSocket处理器
   - 会话管理
   - 实时推送

3. **Week 13-14**: 集群管理
   - NodeManager
   - ClusterManager
   - 健康检查

4. **Week 15-16**: 数据复制
   - RegistryReplicationManager
   - 任务执行器
   - 复制API

**交付物**：
- 支持集群部署
- 实时推送功能
- 数据复制和一致性

### 8.3 管理功能阶段（第5个月）

**目标**: 实现管理API和持久化

**里程碑**：
1. **Week 17-18**: 数据库层
   - MySQL Schema设计
   - DAO实现（SQLx）
   - 连接池配置

2. **Week 19-20**: 管理功能
   - 实例操作（拉入/拉出）
   - 服务分组
   - 路由规则
   - ManagementDiscoveryFilter

**交付物**：
- 完整的管理API
- 数据库持久化
- 运维管理功能

### 8.4 优化和稳定性阶段（第6个月）

**目标**: 性能优化、测试、文档

**里程碑**：
1. **Week 21-22**: 性能优化
   - 基准测试和性能分析
   - 内存优化
   - 并发优化
   - 达到性能目标

2. **Week 23**: 测试
   - 单元测试补全（覆盖率 > 80%）
   - 集成测试
   - 压力测试
   - 混沌测试

3. **Week 24**: 文档和工具
   - API文档
   - 部署文档
   - 运维手册
   - 客户端SDK文档

**交付物**：
- 生产就绪的Artemis Rust版本
- 完整文档
- 测试报告

### 8.5 迁移阶段（第7个月）

**目标**: 平滑迁移，生产验证

**步骤**：
1. **灰度发布**
   - 选择非核心服务测试
   - 观察稳定性和性能
   - 收集反馈

2. **双写模式**
   - 同时向Java和Rust版本注册
   - 对比数据一致性
   - 切换发现流量

3. **全量迁移**
   - 逐步扩大范围
   - 监控关键指标
   - 应急回滚预案

4. **Java版本下线**
   - 确认Rust版本稳定
   - 停止Java服务
   - 清理资源

**交付物**：
- 生产环境稳定运行
- 迁移完成报告

---

## 9. 兼容性与迁移

### 9.1 API兼容性

**保证**：
- REST API路径完全一致
- 请求/响应JSON格式兼容
- 错误码保持一致
- WebSocket协议兼容

**差异**（可接受）：
- HTTP响应头可能不同
- 内部实现细节
- 性能指标

### 9.2 客户端兼容性

**Java客户端**：
- 无需修改即可连接Rust服务端
- 所有API保持兼容
- WebSocket协议一致

**新客户端**：
- 提供Rust版本SDK
- 提供Go版本SDK（可选）

### 9.3 数据兼容性

**注册数据**：
- 内存存储，无迁移问题
- 通过心跳自然刷新

**管理数据**：
- MySQL Schema兼容
- 支持从Java版本的数据库直接启动
- 扩展字段（EXTENSIONS）保持JSON格式

### 9.4 配置兼容性

**映射关系**：
```toml
# Rust配置 <- Java配置

[server]
port = 8080  # <- artemis.service.port

[registry]
ttl = 20  # <- artemis.service.registry.instance.lease-manager.lease.ttl

[cluster]
nodes = ["http://localhost:8080"]  # <- artemis.service.cluster.nodes

[rate_limiter]
register_qps = 10000  # <- artemis.service.registry.rate-limiter.register.qps
```

**迁移工具**：
- 提供配置转换脚本
- 验证配置完整性

### 9.5 迁移策略

**推荐方案**: 滚动升级

1. **准备阶段**
   - 部署Rust节点到集群
   - 配置为同一个集群
   - 开启数据复制

2. **流量切换**
   - 客户端SDK配置包含Rust节点
   - 逐步增加Rust节点权重
   - 监控错误率和延迟

3. **下线Java节点**
   - 停止Java节点心跳
   - 等待租约过期
   - 下线Java进程

4. **验证**
   - 确认所有服务正常
   - 性能指标符合预期
   - 无错误告警

**回滚预案**：
- 快速重启Java节点
- 切换客户端配置
- 数据通过复制恢复

---

## 10. 附录

### 10.1 术语表

| 术语 | 说明 |
|------|------|
| Instance | 服务实例，代表一个可访问的服务节点 |
| Service | 服务，包含多个实例的逻辑聚合 |
| Lease | 租约，用于管理实例生命周期的机制 |
| TTL | Time To Live，租约的生存时间 |
| Region | 区域，通常对应一个数据中心或地理位置 |
| Zone | 可用区，Region内的物理隔离单元 |
| Group | 分组，用于实例分组和路由 |
| RouteRule | 路由规则，定义服务的流量分配策略 |
| Replication | 复制，集群节点间的数据同步机制 |
| Delta | 增量，两个版本之间的差异 |
| Pull In/Out | 拉入/拉出，运维操作，控制实例是否可被发现 |

### 10.2 配置参考

**完整配置示例**（`config.toml`）：

```toml
[server]
# 服务监听地址
host = "0.0.0.0"
port = 8080

# 节点信息
node_id = "node-001"
zone_id = "sha-z1"
region_id = "sha"

[registry]
# 租约TTL（秒）
ttl = 20
# 遗留租约TTL
legacy_ttl = 90
# 允许跨Zone注册
allow_from_other_zone = true

[discovery]
# 允许跨Zone发现
allow_from_other_zone = true

[lease_manager]
# 清理任务间隔（毫秒）
clean_interval = 1000
# 清理线程数
clean_threads = 2
# 初始容量
initial_capacity = 50000
# 安全窗口（秒）
safe_window = 30

[versioned_cache]
# 刷新间隔（秒）
refresh_interval = 30
# 最大版本数
max_versions = 3

[rate_limiter]
# 注册QPS限制
register_qps = 10000
# 心跳QPS限制
heartbeat_qps = 100000
# 注销QPS限制
unregister_qps = 10000

[cluster]
# 集群节点列表
nodes = [
    "http://192.168.1.100:8080",
    "http://192.168.1.101:8080",
]
# 健康检查间隔（秒）
health_check_interval = 10
# 健康检查超时（秒）
health_check_timeout = 5

[replication]
# 批量任务执行器线程数
batching_threads = 20
# 单任务执行器线程数
single_threads = 10
# 批量大小
batch_size = 100
# 批量超时（毫秒）
batch_timeout = 500

[database]
# MySQL连接URL
url = "mysql://artemis:password@localhost:3306/artemis"
# 最大连接数
max_connections = 20
# 最小连接数
min_connections = 5
# 连接超时（秒）
connect_timeout = 10

[websocket]
# WebSocket心跳间隔（秒）
heartbeat_interval = 30
# 变更推送间隔（毫秒）
push_interval = 100

[logging]
# 日志级别
level = "info"
# 日志格式（json/text）
format = "json"
# 日志文件路径（可选）
file = "/var/log/artemis/artemis.log"

[metrics]
# Prometheus导出端点
enabled = true
port = 9090
```

### 10.3 性能基准测试

**测试环境**：
- CPU: 16核
- 内存: 32GB
- 网络: 千兆以太网

**测试工具**：
- wrk2（HTTP压测）
- 自定义WebSocket客户端

**测试场景**：

1. **注册性能**
   - 并发: 1000
   - 每批次实例数: 10
   - 目标QPS: 50k
   - 预期P99延迟: < 10ms

2. **心跳性能**
   - 并发: 5000
   - 每批次实例数: 100
   - 目标QPS: 100k
   - 预期P99延迟: < 5ms

3. **发现性能**
   - 并发: 2000
   - 目标QPS: 100k
   - 预期P99延迟: < 5ms

4. **WebSocket推送性能**
   - 连接数: 10k
   - 推送频率: 每秒1000个变更
   - 预期推送延迟: < 100ms

**基准测试命令**：
```bash
# 注册性能测试
wrk -t16 -c1000 -d60s --latency \
  -s register.lua \
  http://localhost:8080/api/registry/register.json

# 心跳性能测试
wrk -t16 -c5000 -d60s --latency \
  -s heartbeat.lua \
  http://localhost:8080/api/registry/heartbeat.json

# 发现性能测试
wrk -t16 -c2000 -d60s --latency \
  http://localhost:8080/api/discovery/services.json
```

### 10.4 监控指标

**业务指标**：
- `artemis_registry_requests_total{operation}`: 注册请求总数
- `artemis_registry_request_duration_seconds{operation}`: 注册请求延迟
- `artemis_registry_errors_total{operation}`: 注册错误总数
- `artemis_discovery_requests_total`: 发现请求总数
- `artemis_discovery_request_duration_seconds`: 发现请求延迟
- `artemis_lease_count`: 当前租约数量
- `artemis_lease_expired_total`: 过期租约总数
- `artemis_instance_count`: 当前实例数量
- `artemis_service_count`: 当前服务数量
- `artemis_websocket_connections`: WebSocket连接数
- `artemis_websocket_messages_sent_total`: WebSocket消息发送总数
- `artemis_replication_tasks_total{result}`: 复制任务总数（成功/失败）

**系统指标**：
- `artemis_memory_usage_bytes`: 内存使用量
- `artemis_cpu_usage_percent`: CPU使用率
- `artemis_goroutines_count`: Goroutine数量（Tokio任务数）

**集群指标**：
- `artemis_cluster_nodes_total{status}`: 集群节点数（按状态）
- `artemis_cluster_replication_lag_seconds`: 复制延迟

### 10.5 部署示例

#### 10.5.1 命令行 + 配置文件部署

这是最简单直接的部署方式，适合开发环境、测试环境或小规模生产环境。

**步骤1: 编译和构建**

```bash
# 克隆项目
git clone https://github.com/your-org/artemis-rust.git
cd artemis-rust

# 构建Release版本
cargo build --release

# 二进制文件位于 target/release/artemis-server
ls -lh target/release/artemis-server
```

**步骤2: 准备配置文件**

创建配置文件 `/etc/artemis/config.toml`：

```toml
[server]
host = "0.0.0.0"
port = 8080
node_id = "node-001"
zone_id = "sha-z1"
region_id = "sha"

[registry]
ttl = 20
legacy_ttl = 90
allow_from_other_zone = true

[discovery]
allow_from_other_zone = true

[lease_manager]
clean_interval = 1000
clean_threads = 2
initial_capacity = 50000

[cluster]
nodes = [
    "http://192.168.1.101:8080",
    "http://192.168.1.102:8080"
]
health_check_interval = 10

[database]
url = "mysql://artemis:password@192.168.1.200:3306/artemis"
max_connections = 20

[logging]
level = "info"
format = "json"
file = "/var/log/artemis/artemis.log"

[metrics]
enabled = true
port = 9090
```

**步骤3: 初始化数据库**

```bash
# 连接MySQL
mysql -h 192.168.1.200 -u artemis -p

# 创建数据库
CREATE DATABASE IF NOT EXISTS artemis CHARACTER SET utf8mb4;

# 导入Schema
mysql -h 192.168.1.200 -u artemis -p artemis < deployment/schema.sql
```

**步骤4: 命令行启动**

```bash
# 方式1: 使用配置文件
./target/release/artemis-server --config /etc/artemis/config.toml

# 方式2: 配置文件 + 命令行参数覆盖
./target/release/artemis-server \
    --config /etc/artemis/config.toml \
    --server.port 8081 \
    --server.node-id node-002

# 方式3: 仅命令行参数（用于快速测试）
./target/release/artemis-server \
    --server.host 0.0.0.0 \
    --server.port 8080 \
    --server.node-id node-001 \
    --database.url "mysql://artemis:password@localhost/artemis"

# 方式4: 环境变量 + 配置文件
export ARTEMIS_SERVER__NODE_ID=node-003
export ARTEMIS_DATABASE__URL="mysql://artemis:password@localhost/artemis"
./target/release/artemis-server --config /etc/artemis/config.toml
```

**命令行参数说明**：

```
artemis-server [OPTIONS]

OPTIONS:
    -c, --config <FILE>              配置文件路径 [默认: /etc/artemis/config.toml]
    --server.host <HOST>             服务监听地址 [默认: 0.0.0.0]
    --server.port <PORT>             服务监听端口 [默认: 8080]
    --server.node-id <ID>            节点ID [必需]
    --server.zone-id <ID>            Zone ID [默认: default-zone]
    --server.region-id <ID>          Region ID [默认: default-region]
    --database.url <URL>             数据库连接URL
    --logging.level <LEVEL>          日志级别 [debug|info|warn|error]
    --logging.format <FORMAT>        日志格式 [json|text]
    --metrics.enabled <BOOL>         是否启用指标导出 [默认: true]
    --metrics.port <PORT>            指标导出端口 [默认: 9090]
    -h, --help                       显示帮助信息
    -V, --version                    显示版本信息
```

**步骤5: 验证服务**

```bash
# 检查服务状态
curl http://localhost:8080/api/status/node.json

# 检查集群状态
curl http://localhost:8080/api/status/cluster.json

# 检查指标
curl http://localhost:9090/metrics
```

**步骤6: Systemd服务管理（推荐用于生产环境）**

创建Systemd服务文件 `/etc/systemd/system/artemis.service`：

```ini
[Unit]
Description=Artemis Service Registry
Documentation=https://github.com/your-org/artemis-rust
After=network.target mysql.service
Wants=mysql.service

[Service]
Type=simple
User=artemis
Group=artemis
WorkingDirectory=/opt/artemis

# 环境变量
Environment="RUST_LOG=info"
Environment="RUST_BACKTRACE=1"

# 启动命令
ExecStart=/opt/artemis/bin/artemis-server --config /etc/artemis/config.toml

# 优雅停止（30秒超时）
ExecStop=/bin/kill -SIGTERM $MAINPID
TimeoutStopSec=30

# 重启策略
Restart=on-failure
RestartSec=10
StartLimitInterval=5min
StartLimitBurst=3

# 资源限制
LimitNOFILE=65536
LimitNPROC=32768

# 安全设置
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/log/artemis /var/lib/artemis

# 日志
StandardOutput=journal
StandardError=journal
SyslogIdentifier=artemis

[Install]
WantedBy=multi-user.target
```

**Systemd服务管理命令**：

```bash
# 创建artemis用户
sudo useradd -r -s /bin/false artemis

# 创建目录
sudo mkdir -p /opt/artemis/bin
sudo mkdir -p /etc/artemis
sudo mkdir -p /var/log/artemis
sudo mkdir -p /var/lib/artemis

# 复制二进制文件
sudo cp target/release/artemis-server /opt/artemis/bin/
sudo chmod +x /opt/artemis/bin/artemis-server

# 设置权限
sudo chown -R artemis:artemis /opt/artemis
sudo chown -R artemis:artemis /var/log/artemis
sudo chown -R artemis:artemis /var/lib/artemis
sudo chown artemis:artemis /etc/artemis/config.toml

# 重载systemd
sudo systemctl daemon-reload

# 启动服务
sudo systemctl start artemis

# 查看状态
sudo systemctl status artemis

# 查看日志
sudo journalctl -u artemis -f

# 开机自启
sudo systemctl enable artemis

# 停止服务
sudo systemctl stop artemis

# 重启服务
sudo systemctl restart artemis

# 重新加载配置（如果支持）
sudo systemctl reload artemis
```

**步骤7: 日志管理（Logrotate）**

创建日志轮转配置 `/etc/logrotate.d/artemis`：

```
/var/log/artemis/*.log {
    daily
    rotate 30
    compress
    delaycompress
    notifempty
    missingok
    copytruncate
    create 0640 artemis artemis
}
```

**步骤8: 多节点部署示例**

**节点1配置** (`/etc/artemis/config.toml`):
```toml
[server]
host = "0.0.0.0"
port = 8080
node_id = "node-001"
zone_id = "sha-z1"

[cluster]
nodes = [
    "http://192.168.1.101:8080",  # 本节点
    "http://192.168.1.102:8080",  # 节点2
    "http://192.168.1.103:8080"   # 节点3
]

[database]
url = "mysql://artemis:password@192.168.1.200:3306/artemis"
```

**节点2配置** (`/etc/artemis/config.toml`):
```toml
[server]
host = "0.0.0.0"
port = 8080
node_id = "node-002"
zone_id = "sha-z1"

[cluster]
nodes = [
    "http://192.168.1.101:8080",
    "http://192.168.1.102:8080",  # 本节点
    "http://192.168.1.103:8080"
]

[database]
url = "mysql://artemis:password@192.168.1.200:3306/artemis"
```

**节点3配置** (`/etc/artemis/config.toml`):
```toml
[server]
host = "0.0.0.0"
port = 8080
node_id = "node-003"
zone_id = "sha-z1"

[cluster]
nodes = [
    "http://192.168.1.101:8080",
    "http://192.168.1.102:8080",
    "http://192.168.1.103:8080"   # 本节点
]

[database]
url = "mysql://artemis:password@192.168.1.200:3306/artemis"
```

**批量部署脚本** (`deploy.sh`):

```bash
#!/bin/bash
set -e

NODES=("192.168.1.101" "192.168.1.102" "192.168.1.103")
NODE_IDS=("node-001" "node-002" "node-003")
BINARY="target/release/artemis-server"
REMOTE_USER="root"

echo "开始批量部署 Artemis 集群..."

for i in "${!NODES[@]}"; do
    NODE="${NODES[$i]}"
    NODE_ID="${NODE_IDS[$i]}"

    echo ">>> 部署节点: $NODE ($NODE_ID)"

    # 上传二进制文件
    scp $BINARY $REMOTE_USER@$NODE:/opt/artemis/bin/

    # 上传配置文件（需要提前准备好各节点的配置）
    scp configs/$NODE_ID/config.toml $REMOTE_USER@$NODE:/etc/artemis/

    # 上传systemd服务文件
    scp deployment/artemis.service $REMOTE_USER@$NODE:/etc/systemd/system/

    # 重启服务
    ssh $REMOTE_USER@$NODE << 'EOF'
        systemctl daemon-reload
        systemctl restart artemis
        systemctl status artemis --no-pager
EOF

    echo "<<< 节点 $NODE 部署完成"
    echo ""
done

echo "所有节点部署完成！"
echo "验证集群状态..."

for NODE in "${NODES[@]}"; do
    echo ">>> 检查节点: $NODE"
    curl -s http://$NODE:8080/api/status/node.json | jq .
done
```

**步骤9: 监控和健康检查**

```bash
# 健康检查脚本 (health_check.sh)
#!/bin/bash
ENDPOINT="http://localhost:8080/api/status/node.json"

check_health() {
    response=$(curl -s -o /dev/null -w "%{http_code}" $ENDPOINT)
    if [ "$response" = "200" ]; then
        echo "✓ Artemis 服务健康"
        return 0
    else
        echo "✗ Artemis 服务异常 (HTTP $response)"
        return 1
    fi
}

check_health

# 集成到监控系统（Nagios/Zabbix）
# UserParameter=artemis.health,/usr/local/bin/artemis_health_check.sh
```

**步骤10: 备份和恢复**

```bash
# 备份脚本 (backup.sh)
#!/bin/bash
BACKUP_DIR="/var/backups/artemis"
DATE=$(date +%Y%m%d_%H%M%S)

# 备份配置
cp /etc/artemis/config.toml $BACKUP_DIR/config_$DATE.toml

# 备份数据库
mysqldump -h 192.168.1.200 -u artemis -p artemis > $BACKUP_DIR/artemis_$DATE.sql

# 压缩备份
tar -czf $BACKUP_DIR/artemis_backup_$DATE.tar.gz \
    $BACKUP_DIR/config_$DATE.toml \
    $BACKUP_DIR/artemis_$DATE.sql

# 清理临时文件
rm $BACKUP_DIR/config_$DATE.toml
rm $BACKUP_DIR/artemis_$DATE.sql

# 保留最近30天的备份
find $BACKUP_DIR -name "artemis_backup_*.tar.gz" -mtime +30 -delete

echo "备份完成: artemis_backup_$DATE.tar.gz"
```

**故障排查**：

```bash
# 查看实时日志
tail -f /var/log/artemis/artemis.log

# 查看systemd日志
journalctl -u artemis -f

# 查看最近的错误
journalctl -u artemis -p err -n 50

# 检查端口监听
netstat -tlnp | grep artemis
# 或
ss -tlnp | grep artemis

# 检查进程
ps aux | grep artemis-server

# 检查资源使用
top -p $(pgrep artemis-server)
```

**优势**：
- ✅ 部署简单，无需容器化
- ✅ 资源开销最小
- ✅ 易于调试和故障排查
- ✅ 适合传统运维流程
- ✅ 完全掌控进程生命周期

**适用场景**：
- 开发和测试环境
- 小规模生产环境（< 10台服务器）
- 物理机或虚拟机部署
- 有systemd的Linux系统

---

#### 10.5.2 Docker Compose部署

**Docker Compose**：
```yaml
version: '3.8'

services:
  artemis-node1:
    image: artemis-rust:latest
    container_name: artemis-node1
    ports:
      - "8081:8080"
      - "9091:9090"
    environment:
      - ARTEMIS_SERVER__NODE_ID=node-001
      - ARTEMIS_SERVER__ZONE_ID=sha-z1
      - ARTEMIS_CLUSTER__NODES=http://artemis-node2:8080
      - ARTEMIS_DATABASE__URL=mysql://artemis:password@mysql:3306/artemis
    volumes:
      - ./config.toml:/etc/artemis/config.toml
    depends_on:
      - mysql
    restart: unless-stopped

  artemis-node2:
    image: artemis-rust:latest
    container_name: artemis-node2
    ports:
      - "8082:8080"
      - "9092:9090"
    environment:
      - ARTEMIS_SERVER__NODE_ID=node-002
      - ARTEMIS_SERVER__ZONE_ID=sha-z1
      - ARTEMIS_CLUSTER__NODES=http://artemis-node1:8080
      - ARTEMIS_DATABASE__URL=mysql://artemis:password@mysql:3306/artemis
    volumes:
      - ./config.toml:/etc/artemis/config.toml
    depends_on:
      - mysql
    restart: unless-stopped

  mysql:
    image: mysql:8.0
    container_name: artemis-mysql
    environment:
      - MYSQL_ROOT_PASSWORD=rootpassword
      - MYSQL_DATABASE=artemis
      - MYSQL_USER=artemis
      - MYSQL_PASSWORD=password
    ports:
      - "3306:3306"
    volumes:
      - mysql-data:/var/lib/mysql
      - ./schema.sql:/docker-entrypoint-initdb.d/schema.sql
    restart: unless-stopped

  prometheus:
    image: prom/prometheus:latest
    container_name: artemis-prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
    restart: unless-stopped

volumes:
  mysql-data:
  prometheus-data:
```

#### 10.5.3 Kubernetes部署

**Kubernetes部署**（StatefulSet示例）：
```yaml
apiVersion: v1
kind: Service
metadata:
  name: artemis-headless
spec:
  clusterIP: None
  selector:
    app: artemis
  ports:
    - name: http
      port: 8080
    - name: metrics
      port: 9090

---

apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: artemis
spec:
  serviceName: artemis-headless
  replicas: 3
  selector:
    matchLabels:
      app: artemis
  template:
    metadata:
      labels:
        app: artemis
    spec:
      containers:
      - name: artemis
        image: artemis-rust:latest
        ports:
        - containerPort: 8080
          name: http
        - containerPort: 9090
          name: metrics
        env:
        - name: POD_NAME
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        - name: ARTEMIS_SERVER__NODE_ID
          value: "$(POD_NAME)"
        - name: ARTEMIS_SERVER__ZONE_ID
          value: "k8s-zone"
        - name: ARTEMIS_CLUSTER__NODES
          value: "http://artemis-0.artemis-headless:8080,http://artemis-1.artemis-headless:8080,http://artemis-2.artemis-headless:8080"
        - name: ARTEMIS_DATABASE__URL
          valueFrom:
            secretKeyRef:
              name: artemis-secret
              key: database-url
        volumeMounts:
        - name: config
          mountPath: /etc/artemis
        livenessProbe:
          httpGet:
            path: /api/status/node.json
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /api/status/node.json
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
        resources:
          requests:
            memory: "2Gi"
            cpu: "1000m"
          limits:
            memory: "4Gi"
            cpu: "2000m"
      volumes:
      - name: config
        configMap:
          name: artemis-config
```

### 10.6 故障排查指南

**常见问题**：

1. **租约过期过快**
   - 检查心跳频率是否正常
   - 检查网络延迟
   - 增加TTL配置

2. **集群复制延迟高**
   - 检查节点间网络
   - 增加复制线程数
   - 检查目标节点负载

3. **内存占用过高**
   - 检查实例数量
   - 检查版本缓存配置
   - 检查WebSocket连接数

4. **发现返回空服务**
   - 检查Zone配置
   - 检查过滤器逻辑
   - 检查管理操作（是否被拉出）

**日志分析**：
```bash
# 查看错误日志
grep "ERROR" /var/log/artemis/artemis.log

# 查看租约过期
grep "lease_expired" /var/log/artemis/artemis.log

# 查看复制失败
grep "replication_failed" /var/log/artemis/artemis.log
```

### 10.7 参考资料

**原项目**：
- GitHub: https://github.com/ctripcorp/artemis
- 文档: 项目README和Wiki

**Rust生态**：
- Axum: https://github.com/tokio-rs/axum
- Tokio: https://tokio.rs/
- DashMap: https://github.com/xacrimon/dashmap
- SQLx: https://github.com/launchbadge/sqlx

**相关项目**：
- Netflix Eureka: https://github.com/Netflix/eureka
- Consul: https://www.consul.io/
- Nacos: https://nacos.io/

---

## 结语

本规格说明书详细描述了Artemis服务注册中心Rust重写的完整技术规格，包括：

- ✅ 完整的数据模型定义（Rust类型）
- ✅ 详细的功能模块规格（接口、实现、流程）
- ✅ 完整的API规格（REST + WebSocket）
- ✅ 明确的非功能性需求（性能、可靠性、可扩展性）
- ✅ 具体的技术选型建议
- ✅ 清晰的实施路线图（6个月计划）
- ✅ 兼容性和迁移策略
- ✅ 配置、部署、监控、故障排查指南

**重写的核心价值**：
1. **消除GC问题**：Rust无GC特性，确保延迟可控
2. **提升性能**：预期P99延迟从50-200ms降至<10ms
3. **降低成本**：内存占用减少50%+，资源利用率更高
4. **增强稳定性**：类型安全、内存安全、并发安全

**下一步行动**：
1. 搭建Rust项目骨架
2. 实现核心数据模型
3. 按MVP路线图逐步开发
4. 持续测试和优化
5. 灰度迁移到生产环境

本规格说明书将作为Rust重写项目的技术蓝图和指导文档，确保重写工作有序推进，最终交付一个高性能、高可靠的服务注册中心。

---

**文档版本**: 1.0.0
**最后更新**: 2026-02-13
**维护者**: Artemis Rust Rewrite Team
