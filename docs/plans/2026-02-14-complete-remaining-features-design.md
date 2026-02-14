# Artemis Rust 剩余功能完整实施设计

**设计日期**: 2026-02-14
**设计人**: Claude Sonnet 4.5
**目标**: 完整实现 Artemis Rust 版本剩余的所有高级管理功能,完全对齐 Java 版本

---

## 📋 设计概述

### 目标功能

基于功能审查报告 (`docs/FEATURE_COMPARISON_REPORT_2026-02-14.md`),需要实现以下功能:

| 功能模块 | 当前状态 | 目标状态 | 优先级 |
|---------|---------|---------|--------|
| **分组路由** | 15% (仅框架) | 100% | P1 |
| **数据持久化** | 0% | 100% | P1 |
| **Zone 管理** | 0% | 100% | P2 |
| **金丝雀发布** | 0% | 100% | P2 |

### 实施策略

**方案**: 分阶段渐进式实施 (4 个 Phase)

**原则**:
- ✅ 完全对齐 Java 版本 (API、数据模型、行为)
- ✅ 每个 Phase 独立可测试和部署
- ✅ 风险可控,逐步交付价值

### 总体时间估算

- **Phase 13**: 分组路由核心 (5-7 天)
- **Phase 14**: 数据持久化 (4-6 天)
- **Phase 15**: Zone 管理 (2-3 天)
- **Phase 16**: 金丝雀发布 (2-3 天)
- **总计**: 13-19 天

---

## 🏗️ 整体架构设计

### 系统分层架构

```
┌─────────────────────────────────────────────────────────────────┐
│                       HTTP API Layer                             │
│  artemis-web/src/api/                                            │
│  - group.rs (27+ endpoints for routing/group management)        │
│  - zone.rs (5 endpoints for zone management)                    │
│  - canary.rs (1 endpoint for canary config)                     │
└─────────────────────────────────────────────────────────────────┘
                                ↓
┌─────────────────────────────────────────────────────────────────┐
│                    Business Logic Layer                          │
│  artemis-management/src/                                         │
│  ┌───────────────┬──────────────┬──────────────┬──────────────┐ │
│  │ GroupManager  │ RouteManager │ ZoneManager  │CanaryManager │ │
│  │ (分组管理)     │ (路由管理)    │ (Zone管理)   │(金丝雀管理)   │ │
│  └───────────────┴──────────────┴──────────────┴──────────────┘ │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │            RouteEngine (路由策略引擎)                      │   │
│  │  - WeightedRoundRobinStrategy (加权轮询)                  │   │
│  │  - CloseByVisitStrategy (就近访问)                        │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                                ↓
┌─────────────────────────────────────────────────────────────────┐
│                  Persistence Layer (Phase 14)                    │
│  artemis-management/src/persistence/                             │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ DatabasePool (sqlx - 支持 MySQL/PostgreSQL/SQLite)        │   │
│  └──────────────────────────────────────────────────────────┘   │
│  ┌────────────┬─────────────┬────────────┬──────────────────┐   │
│  │ GroupRepo  │ RouteRepo   │ ZoneRepo   │ CanaryRepo       │   │
│  │ InstanceRepo│            │            │                  │   │
│  └────────────┴─────────────┴────────────┴──────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                                ↓
┌─────────────────────────────────────────────────────────────────┐
│                        Data Storage                              │
│  Phase 13: DashMap (内存存储,重启丢失)                            │
│  Phase 14+: Database (MySQL/PostgreSQL/SQLite - 持久化存储)      │
└─────────────────────────────────────────────────────────────────┘
```

### 集成点

**1. 发现服务过滤器链** (Phase 13)

```rust
// artemis-server/src/discovery/mod.rs

pub struct DiscoveryServiceImpl {
    cache_manager: Arc<CacheManager>,
    filters: Vec<Arc<dyn DiscoveryFilter>>,
}

// 过滤器执行顺序 (顺序很重要!)
fn apply_filters(service: &mut Service) {
    // 1. StatusFilter - 过滤 down/unhealthy 实例
    // 2. ManagementDiscoveryFilter - 过滤拉出的实例/服务器
    // 3. ZoneDiscoveryFilter - 过滤拉出的 Zone (Phase 15)
    // 4. CanaryDiscoveryFilter - 金丝雀 IP 过滤 (Phase 16)
    // 5. GroupRoutingFilter - 分组路由策略 (Phase 13, 最后应用)
}
```

**2. 数据持久化集成** (Phase 14)

Manager 层通过 Repository trait 访问数据,支持两种模式:

- **Phase 13**: 仅内存 (DashMap)
- **Phase 14+**: 数据库 + 内存缓存

---

## 📦 Phase 13: 分组路由核心 (5-7 天)

### 目标

实现完整的分组路由功能,包括:
- ✅ 服务分组 CRUD
- ✅ 路由规则 CRUD
- ✅ 路由规则分组关联管理
- ✅ 分组标签管理
- ✅ 分组实例管理
- ✅ 两种路由策略引擎 (加权轮询 + 就近访问)
- ✅ 27 个 HTTP API
- ✅ 集成到发现服务

### 核心数据模型

```rust
// artemis-core/src/model/group.rs (新增文件)

/// 服务分组
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceGroup {
    /// 分组 ID (主键)
    pub group_id: String,
    /// 服务 ID
    pub service_id: String,
    /// 地区 ID
    pub region_id: String,
    /// 可用区 ID (可选)
    pub zone_id: Option<String>,
    /// 分组名称
    pub name: String,
    /// 应用 ID
    pub app_id: String,
    /// 描述
    pub description: Option<String>,
    /// 状态
    pub status: GroupStatus,
    /// 分组类型
    pub group_type: GroupType,
}

/// 分组状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GroupStatus {
    Active,    // 激活
    Inactive,  // 未激活
}

/// 分组类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GroupType {
    Physical,  // 物理分组 - 显式管理实例列表
    Logical,   // 逻辑分组 - 基于规则动态匹配实例
}

/// 路由规则分组关联 (带权重)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRuleGroup {
    /// 路由规则 ID
    pub route_rule_id: String,
    /// 分组 ID
    pub group_id: String,
    /// 权重 (1-100)
    pub weight: u32,
    /// 是否可发布
    pub unreleasable: bool,
}

/// 分组标签
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupTag {
    /// 分组 ID
    pub group_id: String,
    /// 标签键
    pub tag_key: String,
    /// 标签值
    pub tag_value: String,
}

/// 分组实例关联
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInstance {
    /// 分组 ID
    pub group_id: String,
    /// 实例键
    pub instance_key: InstanceKey,
}

/// 分组操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupOperation {
    /// 分组 ID
    pub group_id: String,
    /// 操作类型
    pub operation: String,
    /// 操作人 ID
    pub operator_id: String,
    /// Token
    pub token: Option<String>,
}
```

### 扩展 RouteRule 模型

```rust
// artemis-core/src/model/route.rs (扩展现有文件)

/// 路由规则 (扩展)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRule {
    /// 数据库主键 (自增 ID)
    pub route_rule_id: Option<i64>,
    /// 路由 ID (业务键)
    pub route_id: String,
    /// 服务 ID
    pub service_id: String,
    /// 规则名称
    pub name: String,
    /// 描述
    pub description: Option<String>,
    /// 状态
    pub status: RouteRuleStatus,
    /// 路由策略
    pub strategy: RouteStrategy,
    /// 关联的分组 (带权重)
    pub groups: Vec<RouteRuleGroup>,
}

/// 路由规则状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RouteRuleStatus {
    Active,    // 激活
    Inactive,  // 未激活
}

/// 路由策略 (已存在,确保包含这两个)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RouteStrategy {
    WeightedRoundRobin,  // 加权轮询
    CloseByVisit,        // 就近访问
}
```

### 路由策略引擎

```rust
// artemis-server/src/routing/mod.rs (新增目录和文件)

use artemis_core::model::{Instance, RouteRuleGroup, RouteStrategy};

/// 路由上下文 (包含客户端信息)
pub struct RouteContext {
    /// 客户端 IP
    pub client_ip: String,
    /// 客户端 Region
    pub client_region: Option<String>,
    /// 客户端 Zone
    pub client_zone: Option<String>,
}

/// 路由策略 Trait
#[async_trait]
pub trait RouteStrategy: Send + Sync {
    /// 根据策略选择实例
    async fn select_instances(
        &self,
        instances: &[Instance],
        groups: &[RouteRuleGroup],
        context: &RouteContext,
    ) -> Vec<Instance>;
}

// ========== 加权轮询策略 ==========

/// 加权轮询策略
pub struct WeightedRoundRobinStrategy {
    /// 轮询计数器: service_id -> counter
    counters: Arc<DashMap<String, AtomicUsize>>,
}

impl WeightedRoundRobinStrategy {
    pub fn new() -> Self {
        Self {
            counters: Arc::new(DashMap::new()),
        }
    }
}

#[async_trait]
impl RouteStrategy for WeightedRoundRobinStrategy {
    async fn select_instances(
        &self,
        instances: &[Instance],
        groups: &[RouteRuleGroup],
        context: &RouteContext,
    ) -> Vec<Instance> {
        // 实现加权轮询算法:
        // 1. 按分组 ID 分类实例
        // 2. 计算总权重
        // 3. 使用轮询计数器选择分组
        // 4. 在选中的分组内使用简单轮询选择实例

        // 伪代码:
        // total_weight = sum(group.weight for group in groups)
        // counter = get_and_increment_counter(service_id)
        // selected_group = select_by_weight(counter % total_weight, groups)
        // return instances_in_group(selected_group)
    }
}

// ========== 就近访问策略 ==========

/// 就近访问策略
pub struct CloseByVisitStrategy;

impl CloseByVisitStrategy {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RouteStrategy for CloseByVisitStrategy {
    async fn select_instances(
        &self,
        instances: &[Instance],
        groups: &[RouteRuleGroup],
        context: &RouteContext,
    ) -> Vec<Instance> {
        // 实现就近访问算法:
        // 1. 优先返回同 Region 实例
        // 2. 其次返回同 Zone 实例
        // 3. 最后返回跨 Region 实例

        // 伪代码:
        // if client_region is available:
        //     same_region = filter(instances, region == client_region)
        //     if same_region is not empty:
        //         return same_region
        //
        // if client_zone is available:
        //     same_zone = filter(instances, zone == client_zone)
        //     if same_zone is not empty:
        //         return same_zone
        //
        // return instances (all)
    }
}

// ========== 路由引擎 ==========

/// 路由引擎 - 统一入口
pub struct RouteEngine {
    weighted_rr: WeightedRoundRobinStrategy,
    close_by: CloseByVisitStrategy,
}

impl RouteEngine {
    pub fn new() -> Self {
        Self {
            weighted_rr: WeightedRoundRobinStrategy::new(),
            close_by: CloseByVisitStrategy::new(),
        }
    }

    pub async fn apply_route_rule(
        &self,
        instances: Vec<Instance>,
        rule: &RouteRule,
        context: &RouteContext,
    ) -> Vec<Instance> {
        match rule.strategy {
            RouteStrategy::WeightedRoundRobin => {
                self.weighted_rr.select_instances(&instances, &rule.groups, context).await
            }
            RouteStrategy::CloseByVisit => {
                self.close_by.select_instances(&instances, &rule.groups, context).await
            }
        }
    }
}
```

### Manager 层实现

```rust
// artemis-management/src/group.rs (完整重写)

use artemis_core::model::{ServiceGroup, GroupStatus, GroupType, GroupTag, GroupInstance, GroupOperation};
use dashmap::DashMap;
use std::sync::Arc;

/// 分组管理器
#[derive(Clone)]
pub struct GroupManager {
    /// 分组存储: group_id -> ServiceGroup
    groups: Arc<DashMap<String, ServiceGroup>>,
    /// 分组标签存储: group_id -> Vec<GroupTag>
    group_tags: Arc<DashMap<String, Vec<GroupTag>>>,
    /// 分组实例存储: group_id -> Vec<GroupInstance>
    group_instances: Arc<DashMap<String, Vec<GroupInstance>>>,
    /// 分组操作记录: group_id -> Vec<GroupOperation>
    group_operations: Arc<DashMap<String, Vec<GroupOperation>>>,
}

impl GroupManager {
    pub fn new() -> Self {
        Self {
            groups: Arc::new(DashMap::new()),
            group_tags: Arc::new(DashMap::new()),
            group_instances: Arc::new(DashMap::new()),
            group_operations: Arc::new(DashMap::new()),
        }
    }

    // ========== 分组 CRUD ==========

    pub fn insert_groups(&self, groups: Vec<ServiceGroup>) -> Result<()> {
        for group in groups {
            self.groups.insert(group.group_id.clone(), group);
        }
        Ok(())
    }

    pub fn update_groups(&self, groups: Vec<ServiceGroup>) -> Result<()> {
        for group in groups {
            if !self.groups.contains_key(&group.group_id) {
                return Err(anyhow::anyhow!("Group not found: {}", group.group_id));
            }
            self.groups.insert(group.group_id.clone(), group);
        }
        Ok(())
    }

    pub fn delete_groups(&self, group_ids: Vec<String>) -> Result<()> {
        for group_id in group_ids {
            self.groups.remove(&group_id);
            self.group_tags.remove(&group_id);
            self.group_instances.remove(&group_id);
        }
        Ok(())
    }

    pub fn get_group(&self, group_id: &str) -> Option<ServiceGroup> {
        self.groups.get(group_id).map(|g| g.clone())
    }

    pub fn get_all_groups(&self, service_id: &str) -> Vec<ServiceGroup> {
        self.groups
            .iter()
            .filter(|g| g.service_id == service_id)
            .map(|g| g.clone())
            .collect()
    }

    pub fn get_groups(&self, filter: GroupFilter) -> Vec<ServiceGroup> {
        // 按条件过滤分组
    }

    // ========== 分组标签 CRUD ==========

    pub fn insert_group_tags(&self, tags: Vec<GroupTag>) -> Result<()> { /* ... */ }
    pub fn update_group_tags(&self, tags: Vec<GroupTag>) -> Result<()> { /* ... */ }
    pub fn delete_group_tags(&self, group_id: &str, tag_keys: Vec<String>) -> Result<()> { /* ... */ }
    pub fn get_group_tags(&self, group_id: &str) -> Vec<GroupTag> { /* ... */ }

    // ========== 分组实例 CRUD ==========

    pub fn insert_group_instances(&self, group_id: &str, instances: Vec<GroupInstance>) -> Result<()> { /* ... */ }
    pub fn delete_group_instances(&self, group_id: &str, instance_keys: Vec<InstanceKey>) -> Result<()> { /* ... */ }
    pub fn get_group_instances(&self, group_id: &str) -> Vec<GroupInstance> { /* ... */ }

    // ========== 分组操作 ==========

    pub fn operate_group(&self, operation: GroupOperation) -> Result<()> { /* ... */ }
    pub fn get_group_operations(&self, group_id: &str) -> Vec<GroupOperation> { /* ... */ }
}
```

```rust
// artemis-management/src/route.rs (完整重写)

use artemis_core::model::{RouteRule, RouteRuleGroup};
use dashmap::DashMap;
use std::sync::Arc;

/// 路由规则管理器
#[derive(Clone)]
pub struct RouteManager {
    /// 路由规则存储: route_id -> RouteRule
    rules: Arc<DashMap<String, RouteRule>>,
    /// 路由规则分组关联: route_rule_id -> Vec<RouteRuleGroup>
    rule_groups: Arc<DashMap<String, Vec<RouteRuleGroup>>>,
}

impl RouteManager {
    pub fn new() -> Self {
        Self {
            rules: Arc::new(DashMap::new()),
            rule_groups: Arc::new(DashMap::new()),
        }
    }

    // ========== 路由规则 CRUD ==========

    pub fn insert_route_rules(&self, rules: Vec<RouteRule>) -> Result<()> { /* ... */ }
    pub fn update_route_rules(&self, rules: Vec<RouteRule>) -> Result<()> { /* ... */ }
    pub fn delete_route_rules(&self, rule_ids: Vec<String>) -> Result<()> { /* ... */ }
    pub fn get_route_rule(&self, rule_id: &str) -> Option<RouteRule> { /* ... */ }
    pub fn get_all_route_rules(&self, service_id: &str) -> Vec<RouteRule> { /* ... */ }
    pub fn create_route_rule(&self, rule: RouteRule) -> Result<String> { /* 返回 route_id */ }

    // ========== 路由规则分组关联 CRUD ==========

    pub fn insert_route_rule_groups(&self, rule_groups: Vec<RouteRuleGroup>) -> Result<()> { /* ... */ }
    pub fn update_route_rule_groups(&self, rule_groups: Vec<RouteRuleGroup>) -> Result<()> { /* ... */ }
    pub fn delete_route_rule_groups(&self, rule_id: &str, group_ids: Vec<String>) -> Result<()> { /* ... */ }

    /// 发布路由规则分组 (使规则生效)
    pub fn release_route_rule_groups(&self, rule_id: &str) -> Result<()> {
        // 1. 验证规则存在
        // 2. 验证所有关联的分组都存在
        // 3. 将规则状态设置为 Active
        // 4. 触发配置重载事件
    }

    pub fn get_route_rule_groups(&self, rule_id: &str) -> Vec<RouteRuleGroup> { /* ... */ }
}
```

### 发现服务集成

```rust
// artemis-server/src/discovery/filter.rs (新增)

use artemis_core::model::{Service, DiscoveryConfig};
use artemis_management::route::RouteManager;
use artemis_server::routing::RouteEngine;

/// 分组路由过滤器
pub struct GroupRoutingFilter {
    route_manager: Arc<RouteManager>,
    route_engine: Arc<RouteEngine>,
}

impl GroupRoutingFilter {
    pub fn new(route_manager: Arc<RouteManager>, route_engine: Arc<RouteEngine>) -> Self {
        Self { route_manager, route_engine }
    }
}

#[async_trait]
impl DiscoveryFilter for GroupRoutingFilter {
    async fn filter(&self, service: &mut Service, config: &DiscoveryConfig) -> Result<()> {
        // 1. 查询服务的路由规则
        let rules = self.route_manager.get_all_route_rules(&service.service_id);

        // 2. 如果没有规则,不做过滤
        if rules.is_empty() {
            return Ok(());
        }

        // 3. 应用第一个激活的规则 (可以扩展为支持多规则)
        if let Some(active_rule) = rules.iter().find(|r| r.status == RouteRuleStatus::Active) {
            // 4. 构建路由上下文
            let context = RouteContext {
                client_ip: config.client_ip.clone().unwrap_or_default(),
                client_region: config.region_id.clone(),
                client_zone: config.zone_id.clone(),
            };

            // 5. 应用路由策略
            let filtered_instances = self.route_engine
                .apply_route_rule(service.instances.clone(), active_rule, &context)
                .await;

            // 6. 替换实例列表
            service.instances = filtered_instances;

            info!(
                "Applied route rule {} to service {}, {} instances remaining",
                active_rule.route_id,
                service.service_id,
                service.instances.len()
            );
        }

        Ok(())
    }
}
```

### HTTP API 层

```rust
// artemis-web/src/api/group.rs (新增文件,约 500-600 行)

use axum::{Router, Json};
use artemis_core::model::*;
use artemis_management::{GroupManager, RouteManager};

// ========== Request/Response 模型 ==========

#[derive(Deserialize)]
pub struct InsertGroupsRequest {
    pub groups: Vec<ServiceGroup>,
}

#[derive(Serialize)]
pub struct InsertGroupsResponse {
    pub status: ResponseStatus,
}

// ... 其他 26 个 API 的 Request/Response 模型

// ========== API Handlers ==========

pub async fn insert_groups(
    State(state): State<Arc<AppState>>,
    Json(req): Json<InsertGroupsRequest>,
) -> Json<InsertGroupsResponse> {
    match state.group_manager.insert_groups(req.groups) {
        Ok(_) => Json(InsertGroupsResponse {
            status: ResponseStatus::success(),
        }),
        Err(e) => Json(InsertGroupsResponse {
            status: ResponseStatus::error(&e.to_string()),
        }),
    }
}

pub async fn update_groups(/* ... */) -> Json<UpdateGroupsResponse> { /* ... */ }
pub async fn delete_groups(/* ... */) -> Json<DeleteGroupsResponse> { /* ... */ }
pub async fn get_groups(/* ... */) -> Json<GetGroupsResponse> { /* ... */ }
pub async fn get_all_groups(/* ... */) -> Json<GetAllGroupsResponse> { /* ... */ }

// 路由规则 API (6 个)
pub async fn insert_route_rules(/* ... */) -> Json<InsertRouteRulesResponse> { /* ... */ }
pub async fn update_route_rules(/* ... */) -> Json<UpdateRouteRulesResponse> { /* ... */ }
pub async fn delete_route_rules(/* ... */) -> Json<DeleteRouteRulesResponse> { /* ... */ }
pub async fn get_route_rules(/* ... */) -> Json<GetRouteRulesResponse> { /* ... */ }
pub async fn get_all_route_rules(/* ... */) -> Json<GetAllRouteRulesResponse> { /* ... */ }
pub async fn create_route_rule(/* ... */) -> Json<CreateRouteRuleResponse> { /* ... */ }

// 路由规则分组 API (6 个)
pub async fn insert_route_rule_groups(/* ... */) -> Json<InsertRouteRuleGroupsResponse> { /* ... */ }
pub async fn update_route_rule_groups(/* ... */) -> Json<UpdateRouteRuleGroupsResponse> { /* ... */ }
pub async fn delete_route_rule_groups(/* ... */) -> Json<DeleteRouteRuleGroupsResponse> { /* ... */ }
pub async fn release_route_rule_groups(/* ... */) -> Json<ReleaseRouteRuleGroupsResponse> { /* ... */ }
pub async fn get_route_rule_groups(/* ... */) -> Json<GetRouteRuleGroupsResponse> { /* ... */ }
pub async fn get_all_route_rule_groups(/* ... */) -> Json<GetAllRouteRuleGroupsResponse> { /* ... */ }

// 分组标签 API (5 个)
pub async fn insert_group_tags(/* ... */) -> Json<InsertGroupTagsResponse> { /* ... */ }
pub async fn update_group_tags(/* ... */) -> Json<UpdateGroupTagsResponse> { /* ... */ }
pub async fn delete_group_tags(/* ... */) -> Json<DeleteGroupTagsResponse> { /* ... */ }
pub async fn get_group_tags(/* ... */) -> Json<GetGroupTagsResponse> { /* ... */ }
pub async fn get_all_group_tags(/* ... */) -> Json<GetAllGroupTagsResponse> { /* ... */ }

// 分组实例 API (3 个)
pub async fn insert_group_instances(/* ... */) -> Json<InsertGroupInstancesResponse> { /* ... */ }
pub async fn delete_group_instances(/* ... */) -> Json<DeleteGroupInstancesResponse> { /* ... */ }
pub async fn get_group_instances(/* ... */) -> Json<GetGroupInstancesResponse> { /* ... */ }

// ========== Router 配置 ==========

pub fn create_group_router() -> Router<Arc<AppState>> {
    Router::new()
        // 分组 API
        .route("/api/management/group/insert-groups.json", post(insert_groups))
        .route("/api/management/group/update-groups.json", post(update_groups))
        .route("/api/management/group/delete-groups.json", post(delete_groups))
        .route("/api/management/group/get-groups.json", post(get_groups))
        .route("/api/management/group/get-all-groups.json", get(get_all_groups))

        // 路由规则 API
        .route("/api/management/group/insert-route-rules.json", post(insert_route_rules))
        .route("/api/management/group/update-route-rules.json", post(update_route_rules))
        .route("/api/management/group/delete-route-rules.json", post(delete_route_rules))
        .route("/api/management/group/get-route-rules.json", post(get_route_rules))
        .route("/api/management/group/get-all-route-rules.json", get(get_all_route_rules))
        .route("/api/management/group/create-route-rule.json", post(create_route_rule))

        // 路由规则分组 API
        .route("/api/management/group/insert-route-rule-groups.json", post(insert_route_rule_groups))
        .route("/api/management/group/update-route-rule-groups.json", post(update_route_rule_groups))
        .route("/api/management/group/delete-route-rule-groups.json", post(delete_route_rule_groups))
        .route("/api/management/group/release-route-rule-groups.json", post(release_route_rule_groups))
        .route("/api/management/group/get-route-rule-groups.json", post(get_route_rule_groups))
        .route("/api/management/group/get-all-route-rule-groups.json", get(get_all_route_rule_groups))

        // 分组标签 API
        .route("/api/management/group/insert-group-tags.json", post(insert_group_tags))
        .route("/api/management/group/update-group-tags.json", post(update_group_tags))
        .route("/api/management/group/delete-group-tags.json", post(delete_group_tags))
        .route("/api/management/group/get-group-tags.json", post(get_group_tags))
        .route("/api/management/group/get-all-group-tags.json", get(get_all_group_tags))

        // 分组实例 API
        .route("/api/management/group/insert-group-instances.json", post(insert_group_instances))
        .route("/api/management/group/delete-group-instances.json", post(delete_group_instances))
        .route("/api/management/group/get-group-instances.json", post(get_group_instances))
}
```

### 测试策略

**单元测试**:

```rust
// artemis-server/src/routing/tests.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_weighted_round_robin_strategy() {
        // 创建 3 个分组,权重 50:30:20
        // 创建 100 个实例
        // 调用策略 1000 次
        // 验证分配比例接近 50:30:20
    }

    #[tokio::test]
    async fn test_close_by_visit_strategy() {
        // 创建不同 region/zone 的实例
        // 模拟不同客户端位置
        // 验证返回最近的实例
    }
}

// artemis-management/src/group.rs (tests)

#[cfg(test)]
mod tests {
    #[test]
    fn test_group_crud() {
        // 测试分组的增删改查
    }

    #[test]
    fn test_group_tags() {
        // 测试标签管理
    }
}
```

**集成测试脚本**:

```bash
# test-group-routing.sh

#!/bin/bash

echo "========== Phase 13: Group Routing Integration Test =========="

# 1. 注册测试服务实例
echo "Step 1: Register test instances..."
# 注册 10 个实例,分属 3 个不同的分组

# 2. 创建服务分组
echo "Step 2: Create service groups..."
curl -X POST http://localhost:8080/api/management/group/insert-groups.json -d '{
  "groups": [
    {"group_id": "group-1", "service_id": "test-service", ...},
    {"group_id": "group-2", "service_id": "test-service", ...},
    {"group_id": "group-3", "service_id": "test-service", ...}
  ]
}'

# 3. 将实例分配到分组
echo "Step 3: Assign instances to groups..."

# 4. 创建路由规则
echo "Step 4: Create route rule..."
curl -X POST http://localhost:8080/api/management/group/create-route-rule.json -d '{
  "route_id": "rule-1",
  "service_id": "test-service",
  "strategy": "weighted-round-robin",
  "groups": [
    {"group_id": "group-1", "weight": 50},
    {"group_id": "group-2", "weight": 30},
    {"group_id": "group-3", "weight": 20}
  ]
}'

# 5. 发布路由规则
echo "Step 5: Release route rule..."
curl -X POST http://localhost:8080/api/management/group/release-route-rule-groups.json -d '{
  "route_rule_id": "rule-1"
}'

# 6. 调用发现服务 100 次,统计实例分布
echo "Step 6: Call discovery service 100 times..."
for i in {1..100}; do
  curl -X POST http://localhost:8080/api/discovery/service.json -d '{
    "discovery_config": {"service_id": "test-service", ...}
  }'
done

# 7. 验证分布比例接近 50:30:20
echo "Step 7: Verify distribution..."

echo "========== All Tests Passed! =========="
```

### Phase 13 完成标准

- ✅ 所有 27 个 API 实现并通过单元测试
- ✅ 两种路由策略实现并通过算法测试
- ✅ 集成测试脚本通过 (分布比例误差 < 5%)
- ✅ 代码零警告 (cargo clippy)
- ✅ 文档更新 (README, API 文档)

---

## 💾 Phase 14: 数据持久化 (4-6 天)

### 目标

实现多数据库支持的持久化层:
- ✅ 支持 MySQL/PostgreSQL/SQLite
- ✅ 12 张表 Schema + 数据库迁移
- ✅ Repository 模式实现
- ✅ Manager 层集成持久化
- ✅ 启动时从数据库加载配置

### 技术选型

**数据库 ORM**: `sqlx` (异步 + 编译时检查 + 多数据库支持)

**依赖**:
```toml
[dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio", "mysql", "postgres", "sqlite"] }
```

### 数据库抽象层

```rust
// artemis-management/src/persistence/mod.rs (新增目录)

pub mod pool;
pub mod repository;
pub mod migrations;

pub use pool::{DatabasePool, DatabaseConfig, DatabaseType};
pub use repository::*;
```

```rust
// artemis-management/src/persistence/pool.rs

use sqlx::{Pool, Any, AnyPool};

/// 数据库类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DatabaseType {
    MySQL,
    PostgreSQL,
    SQLite,
}

/// 数据库配置
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub db_type: DatabaseType,
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

impl DatabaseConfig {
    /// 从环境变量或配置文件加载
    pub fn from_env() -> Result<Self> {
        let db_type = std::env::var("ARTEMIS_DB_TYPE")
            .unwrap_or_else(|_| "sqlite".to_string());

        let db_type = match db_type.as_str() {
            "mysql" => DatabaseType::MySQL,
            "postgres" | "postgresql" => DatabaseType::PostgreSQL,
            "sqlite" => DatabaseType::SQLite,
            _ => return Err(anyhow::anyhow!("Invalid database type: {}", db_type)),
        };

        let url = std::env::var("ARTEMIS_DB_URL")
            .unwrap_or_else(|_| match db_type {
                DatabaseType::SQLite => "sqlite:artemis.db".to_string(),
                DatabaseType::MySQL => "mysql://root:password@localhost/artemis".to_string(),
                DatabaseType::PostgreSQL => "postgres://postgres:password@localhost/artemis".to_string(),
            });

        Ok(Self {
            db_type,
            url,
            max_connections: 10,
            min_connections: 1,
        })
    }
}

/// 统一的数据库连接池
pub struct DatabasePool {
    pool: AnyPool,
    db_type: DatabaseType,
}

impl DatabasePool {
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        let pool = sqlx::any::AnyPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .connect(&config.url)
            .await?;

        Ok(Self {
            pool,
            db_type: config.db_type,
        })
    }

    pub fn pool(&self) -> &AnyPool {
        &self.pool
    }

    pub fn db_type(&self) -> DatabaseType {
        self.db_type
    }

    /// 运行数据库迁移
    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await?;
        Ok(())
    }
}
```

### 数据库 Schema

```sql
-- migrations/20260214000001_create_tables.sql

-- ========== 1. 实例操作表 ==========
CREATE TABLE IF NOT EXISTS instance (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    service_id VARCHAR(100) NOT NULL,
    instance_id VARCHAR(100) NOT NULL,
    region_id VARCHAR(50) NOT NULL,
    zone_id VARCHAR(50),
    group_id VARCHAR(50),
    operation VARCHAR(20) NOT NULL,  -- pullout/pullin
    operation_complete BOOLEAN NOT NULL DEFAULT FALSE,
    operator_id VARCHAR(100),
    token VARCHAR(200),
    datachange_lasttime TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    UNIQUE KEY uk_instance (service_id, instance_id, region_id, zone_id, group_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE INDEX idx_instance_service ON instance(service_id);

-- ========== 2. 实例操作日志表 ==========
CREATE TABLE IF NOT EXISTS instance_log (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    service_id VARCHAR(100) NOT NULL,
    instance_id VARCHAR(100) NOT NULL,
    region_id VARCHAR(50) NOT NULL,
    zone_id VARCHAR(50),
    group_id VARCHAR(50),
    operation VARCHAR(20) NOT NULL,
    operation_complete BOOLEAN NOT NULL,
    operator_id VARCHAR(100),
    token VARCHAR(200),
    datachange_lasttime TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_instance_log_service (service_id, instance_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- ========== 3. 服务器操作表 ==========
CREATE TABLE IF NOT EXISTS server (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    server_id VARCHAR(100) NOT NULL,
    region_id VARCHAR(50) NOT NULL,
    operation VARCHAR(20) NOT NULL,  -- pullout/pullin
    operation_complete BOOLEAN NOT NULL DEFAULT FALSE,
    operator_id VARCHAR(100),
    token VARCHAR(200),
    datachange_lasttime TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    UNIQUE KEY uk_server (server_id, region_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- ========== 4. 服务器操作日志表 ==========
CREATE TABLE IF NOT EXISTS server_log (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    server_id VARCHAR(100) NOT NULL,
    region_id VARCHAR(50) NOT NULL,
    operation VARCHAR(20) NOT NULL,
    operation_complete BOOLEAN NOT NULL,
    operator_id VARCHAR(100),
    token VARCHAR(200),
    datachange_lasttime TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_server_log (server_id, region_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- ========== 5. 服务分组表 ==========
CREATE TABLE IF NOT EXISTS service_group (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    group_id VARCHAR(100) NOT NULL,
    service_id VARCHAR(100) NOT NULL,
    region_id VARCHAR(50) NOT NULL,
    zone_id VARCHAR(50),
    name VARCHAR(100) NOT NULL,
    app_id VARCHAR(100) NOT NULL,
    description VARCHAR(500),
    status VARCHAR(20) NOT NULL,  -- active/inactive
    group_type VARCHAR(20) NOT NULL,  -- physical/logical
    datachange_lasttime TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    UNIQUE KEY uk_group (group_id),
    INDEX idx_group_service (service_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- ========== 6. 服务分组日志表 ==========
CREATE TABLE IF NOT EXISTS service_group_log (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    group_id VARCHAR(100) NOT NULL,
    service_id VARCHAR(100) NOT NULL,
    region_id VARCHAR(50) NOT NULL,
    zone_id VARCHAR(50),
    name VARCHAR(100) NOT NULL,
    app_id VARCHAR(100) NOT NULL,
    description VARCHAR(500),
    status VARCHAR(20) NOT NULL,
    group_type VARCHAR(20) NOT NULL,
    datachange_lasttime TIMESTAMP DEFAULT CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- ========== 7. 服务分组标签表 ==========
CREATE TABLE IF NOT EXISTS service_group_tag (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    group_id VARCHAR(100) NOT NULL,
    tag_key VARCHAR(50) NOT NULL,
    tag_value VARCHAR(200) NOT NULL,
    datachange_lasttime TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    UNIQUE KEY uk_group_tag (group_id, tag_key),
    INDEX idx_tag_group (group_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- ========== 8. 服务分组实例表 ==========
CREATE TABLE IF NOT EXISTS service_group_instance (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    group_id VARCHAR(100) NOT NULL,
    service_id VARCHAR(100) NOT NULL,
    instance_id VARCHAR(100) NOT NULL,
    region_id VARCHAR(50) NOT NULL,
    zone_id VARCHAR(50),
    datachange_lasttime TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    UNIQUE KEY uk_group_instance (group_id, service_id, instance_id, region_id),
    INDEX idx_instance_group (group_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- ========== 9. 路由规则表 ==========
CREATE TABLE IF NOT EXISTS service_route_rule (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    route_id VARCHAR(100) NOT NULL,
    service_id VARCHAR(100) NOT NULL,
    name VARCHAR(100) NOT NULL,
    description VARCHAR(500),
    status VARCHAR(20) NOT NULL,  -- active/inactive
    strategy VARCHAR(50) NOT NULL,  -- weighted-round-robin/close-by-visit
    datachange_lasttime TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    UNIQUE KEY uk_route (route_id),
    INDEX idx_route_service (service_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- ========== 10. 路由规则日志表 ==========
CREATE TABLE IF NOT EXISTS service_route_rule_log (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    route_id VARCHAR(100) NOT NULL,
    service_id VARCHAR(100) NOT NULL,
    name VARCHAR(100) NOT NULL,
    description VARCHAR(500),
    status VARCHAR(20) NOT NULL,
    strategy VARCHAR(50) NOT NULL,
    datachange_lasttime TIMESTAMP DEFAULT CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- ========== 11. 路由规则分组关联表 ==========
CREATE TABLE IF NOT EXISTS service_route_rule_group (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    route_rule_id VARCHAR(100) NOT NULL,
    group_id VARCHAR(100) NOT NULL,
    weight INT NOT NULL DEFAULT 1,
    unreleasable BOOLEAN NOT NULL DEFAULT FALSE,
    datachange_lasttime TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    UNIQUE KEY uk_rule_group (route_rule_id, group_id),
    INDEX idx_rule (route_rule_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- ========== 12. Zone 操作表 ==========
CREATE TABLE IF NOT EXISTS zone (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    zone_id VARCHAR(50) NOT NULL,
    region_id VARCHAR(50) NOT NULL,
    operation VARCHAR(20) NOT NULL,  -- pullout/pullin
    operation_complete BOOLEAN NOT NULL DEFAULT FALSE,
    operator_id VARCHAR(100),
    token VARCHAR(200),
    datachange_lasttime TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    UNIQUE KEY uk_zone (zone_id, region_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- ========== 13. Zone 操作日志表 ==========
CREATE TABLE IF NOT EXISTS zone_log (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    zone_id VARCHAR(50) NOT NULL,
    region_id VARCHAR(50) NOT NULL,
    operation VARCHAR(20) NOT NULL,
    operation_complete BOOLEAN NOT NULL,
    operator_id VARCHAR(100),
    token VARCHAR(200),
    datachange_lasttime TIMESTAMP DEFAULT CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
```

**SQLite 版本** (自动生成):
```sql
-- migrations/20260214000001_create_tables_sqlite.sql
-- 使用 INTEGER PRIMARY KEY AUTOINCREMENT 替代 BIGINT AUTO_INCREMENT
-- 使用 TEXT 替代 VARCHAR
-- 其他语法调整
```

**PostgreSQL 版本** (自动生成):
```sql
-- migrations/20260214000001_create_tables_postgres.sql
-- 使用 BIGSERIAL 替代 BIGINT AUTO_INCREMENT
-- 使用 TIMESTAMP WITH TIME ZONE
-- 其他语法调整
```

### Repository 实现

```rust
// artemis-management/src/persistence/repository/group.rs

use sqlx::{AnyPool, FromRow};
use artemis_core::model::ServiceGroup;

#[async_trait]
pub trait GroupRepository: Send + Sync {
    async fn insert(&self, group: &ServiceGroup) -> Result<()>;
    async fn update(&self, group: &ServiceGroup) -> Result<()>;
    async fn delete(&self, group_id: &str) -> Result<()>;
    async fn get(&self, group_id: &str) -> Result<Option<ServiceGroup>>;
    async fn get_by_service(&self, service_id: &str) -> Result<Vec<ServiceGroup>>;
    async fn get_all(&self) -> Result<Vec<ServiceGroup>>;
}

pub struct SqlGroupRepository {
    pool: Arc<AnyPool>,
}

impl SqlGroupRepository {
    pub fn new(pool: Arc<AnyPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GroupRepository for SqlGroupRepository {
    async fn insert(&self, group: &ServiceGroup) -> Result<()> {
        sqlx::query(
            "INSERT INTO service_group
             (group_id, service_id, region_id, zone_id, name, app_id, description, status, group_type)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&group.group_id)
        .bind(&group.service_id)
        .bind(&group.region_id)
        .bind(&group.zone_id)
        .bind(&group.name)
        .bind(&group.app_id)
        .bind(&group.description)
        .bind(group.status.to_string())
        .bind(group.group_type.to_string())
        .execute(self.pool.as_ref())
        .await?;

        // 同时插入日志
        self.insert_log(group).await?;

        Ok(())
    }

    async fn update(&self, group: &ServiceGroup) -> Result<()> {
        sqlx::query(
            "UPDATE service_group
             SET service_id = ?, region_id = ?, zone_id = ?, name = ?,
                 app_id = ?, description = ?, status = ?, group_type = ?
             WHERE group_id = ?"
        )
        .bind(&group.service_id)
        .bind(&group.region_id)
        .bind(&group.zone_id)
        .bind(&group.name)
        .bind(&group.app_id)
        .bind(&group.description)
        .bind(group.status.to_string())
        .bind(group.group_type.to_string())
        .bind(&group.group_id)
        .execute(self.pool.as_ref())
        .await?;

        self.insert_log(group).await?;

        Ok(())
    }

    async fn delete(&self, group_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM service_group WHERE group_id = ?")
            .bind(group_id)
            .execute(self.pool.as_ref())
            .await?;
        Ok(())
    }

    async fn get(&self, group_id: &str) -> Result<Option<ServiceGroup>> {
        let row = sqlx::query_as::<_, ServiceGroupRow>(
            "SELECT * FROM service_group WHERE group_id = ?"
        )
        .bind(group_id)
        .fetch_optional(self.pool.as_ref())
        .await?;

        Ok(row.map(Into::into))
    }

    async fn get_by_service(&self, service_id: &str) -> Result<Vec<ServiceGroup>> {
        let rows = sqlx::query_as::<_, ServiceGroupRow>(
            "SELECT * FROM service_group WHERE service_id = ?"
        )
        .bind(service_id)
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn get_all(&self) -> Result<Vec<ServiceGroup>> {
        let rows = sqlx::query_as::<_, ServiceGroupRow>(
            "SELECT * FROM service_group"
        )
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    // 私有方法: 插入日志
    async fn insert_log(&self, group: &ServiceGroup) -> Result<()> {
        sqlx::query(
            "INSERT INTO service_group_log
             (group_id, service_id, region_id, zone_id, name, app_id, description, status, group_type)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&group.group_id)
        .bind(&group.service_id)
        .bind(&group.region_id)
        .bind(&group.zone_id)
        .bind(&group.name)
        .bind(&group.app_id)
        .bind(&group.description)
        .bind(group.status.to_string())
        .bind(group.group_type.to_string())
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }
}

// 数据库行映射
#[derive(FromRow)]
struct ServiceGroupRow {
    group_id: String,
    service_id: String,
    region_id: String,
    zone_id: Option<String>,
    name: String,
    app_id: String,
    description: Option<String>,
    status: String,
    group_type: String,
}

impl From<ServiceGroupRow> for ServiceGroup {
    fn from(row: ServiceGroupRow) -> Self {
        Self {
            group_id: row.group_id,
            service_id: row.service_id,
            region_id: row.region_id,
            zone_id: row.zone_id,
            name: row.name,
            app_id: row.app_id,
            description: row.description,
            status: row.status.parse().unwrap_or(GroupStatus::Inactive),
            group_type: row.group_type.parse().unwrap_or(GroupType::Physical),
        }
    }
}
```

**类似实现**:
- `RouteRuleRepository`
- `InstanceRepository`
- `ServerRepository`
- `ZoneRepository`

### Manager 层改造

```rust
// artemis-management/src/group.rs (Phase 14 改造)

pub struct GroupManager {
    // 内存缓存 (读取优化)
    groups: Arc<DashMap<String, ServiceGroup>>,
    group_tags: Arc<DashMap<String, Vec<GroupTag>>>,
    group_instances: Arc<DashMap<String, Vec<GroupInstance>>>,

    // Repository (持久化存储)
    repository: Option<Arc<dyn GroupRepository>>,
    tag_repository: Option<Arc<dyn GroupTagRepository>>,
    instance_repository: Option<Arc<dyn GroupInstanceRepository>>,
}

impl GroupManager {
    /// 仅内存模式 (Phase 13)
    pub fn new() -> Self {
        Self {
            groups: Arc::new(DashMap::new()),
            group_tags: Arc::new(DashMap::new()),
            group_instances: Arc::new(DashMap::new()),
            repository: None,
            tag_repository: None,
            instance_repository: None,
        }
    }

    /// 持久化模式 (Phase 14)
    pub fn with_persistence(
        repo: Arc<dyn GroupRepository>,
        tag_repo: Arc<dyn GroupTagRepository>,
        instance_repo: Arc<dyn GroupInstanceRepository>,
    ) -> Self {
        Self {
            groups: Arc::new(DashMap::new()),
            group_tags: Arc::new(DashMap::new()),
            group_instances: Arc::new(DashMap::new()),
            repository: Some(repo),
            tag_repository: Some(tag_repo),
            instance_repository: Some(instance_repo),
        }
    }

    /// 从数据库加载所有数据到内存 (启动时调用)
    pub async fn load_from_database(&self) -> Result<()> {
        if let Some(repo) = &self.repository {
            let groups = repo.get_all().await?;
            for group in groups {
                self.groups.insert(group.group_id.clone(), group);
            }
            info!("Loaded {} groups from database", self.groups.len());
        }

        // 加载标签和实例...

        Ok(())
    }

    /// 插入分组 (读写数据库)
    pub async fn insert_groups(&self, groups: Vec<ServiceGroup>) -> Result<()> {
        // 1. 写入数据库
        if let Some(repo) = &self.repository {
            for group in &groups {
                repo.insert(group).await?;
            }
        }

        // 2. 更新内存缓存
        for group in groups {
            self.groups.insert(group.group_id.clone(), group);
        }

        Ok(())
    }

    /// 获取分组 (优先读内存)
    pub async fn get_group(&self, group_id: &str) -> Result<Option<ServiceGroup>> {
        // 1. 先查内存缓存
        if let Some(group) = self.groups.get(group_id) {
            return Ok(Some(group.clone()));
        }

        // 2. 缓存未命中,查数据库
        if let Some(repo) = &self.repository {
            if let Some(group) = repo.get(group_id).await? {
                // 3. 更新缓存
                self.groups.insert(group.group_id.clone(), group.clone());
                return Ok(Some(group));
            }
        }

        Ok(None)
    }

    // 其他方法类似改造...
}
```

### 启动流程改造

```rust
// artemis/src/main.rs

#[tokio::main]
async fn main() -> Result<()> {
    // 1. 加载配置
    let db_config = DatabaseConfig::from_env()?;

    // 2. 初始化数据库连接池
    let db_pool = if db_config.enabled {
        let pool = DatabasePool::new(db_config).await?;
        pool.migrate().await?;  // 运行数据库迁移
        Some(Arc::new(pool))
    } else {
        None
    };

    // 3. 初始化 Repository
    let group_repo = db_pool.as_ref().map(|pool| {
        Arc::new(SqlGroupRepository::new(pool.pool().clone())) as Arc<dyn GroupRepository>
    });

    // 4. 初始化 Manager (带持久化)
    let group_manager = if let Some(repo) = group_repo {
        let manager = GroupManager::with_persistence(repo, ...);
        manager.load_from_database().await?;  // 启动时加载数据
        Arc::new(manager)
    } else {
        Arc::new(GroupManager::new())  // 仅内存模式
    };

    // 5. 初始化 Web 服务
    let app_state = AppState {
        group_manager,
        // ...
    };

    // 6. 启动服务器
    start_server(app_state).await
}
```

### Phase 14 完成标准

- ✅ 数据库连接池和抽象层实现
- ✅ 12 张表 Schema + 数据库迁移脚本
- ✅ 所有 Repository 实现并通过单元测试
- ✅ Manager 层成功集成持久化
- ✅ 启动时从数据库加载配置
- ✅ 集成测试: 写入 → 重启 → 验证数据恢复
- ✅ 支持 MySQL/PostgreSQL/SQLite 三种数据库

---

## 🌐 Phase 15: Zone 管理 (2-3 天)

### 目标

实现 Zone 级别的批量操作管理:
- ✅ Zone 拉入/拉出操作
- ✅ Zone 状态查询
- ✅ Zone 操作历史
- ✅ 5 个 HTTP API
- ✅ 集成到发现服务过滤器
- ✅ 持久化到数据库

### 核心实现

```rust
// artemis-management/src/zone.rs (新增文件)

use artemis_core::model::{Operation, InstanceKey};
use dashmap::DashMap;
use std::sync::Arc;

/// Zone 操作记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneOperation {
    pub zone_id: String,
    pub region_id: String,
    pub operation: Operation,  // PullOut | PullIn
    pub operation_complete: bool,
    pub operator_id: String,
    pub token: Option<String>,
}

/// Zone 管理器
#[derive(Clone)]
pub struct ZoneManager {
    /// Zone 操作存储: zone_key (zone_id:region_id) -> ZoneOperation
    zone_operations: Arc<DashMap<String, ZoneOperation>>,
    /// Repository (持久化)
    repository: Option<Arc<dyn ZoneRepository>>,
}

impl ZoneManager {
    pub fn new() -> Self {
        Self {
            zone_operations: Arc::new(DashMap::new()),
            repository: None,
        }
    }

    pub fn with_repository(repo: Arc<dyn ZoneRepository>) -> Self {
        Self {
            zone_operations: Arc::new(DashMap::new()),
            repository: Some(repo),
        }
    }

    /// 从数据库加载
    pub async fn load_from_database(&self) -> Result<()> {
        if let Some(repo) = &self.repository {
            let operations = repo.get_all().await?;
            for op in operations {
                let key = Self::zone_key(&op.zone_id, &op.region_id);
                self.zone_operations.insert(key, op);
            }
        }
        Ok(())
    }

    /// 操作 Zone
    pub async fn operate_zone(&self, operation: ZoneOperation) -> Result<()> {
        let key = Self::zone_key(&operation.zone_id, &operation.region_id);

        // 写入数据库
        if let Some(repo) = &self.repository {
            repo.insert(&operation).await?;
        }

        // 更新内存
        if operation.operation == Operation::PullOut && operation.operation_complete {
            self.zone_operations.insert(key, operation);
        } else if operation.operation == Operation::PullIn && operation.operation_complete {
            self.zone_operations.remove(&key);
        }

        Ok(())
    }

    /// 查询 Zone 是否被拉出
    pub fn is_zone_down(&self, zone_id: &str, region_id: &str) -> bool {
        let key = Self::zone_key(zone_id, region_id);
        if let Some(op) = self.zone_operations.get(&key) {
            return op.operation == Operation::PullOut && op.operation_complete;
        }
        false
    }

    /// 获取 Zone 操作记录
    pub async fn get_zone_operations(&self, zone_id: &str, region_id: &str) -> Result<Vec<ZoneOperation>> {
        if let Some(repo) = &self.repository {
            repo.get_by_zone(zone_id, region_id).await
        } else {
            Ok(vec![])
        }
    }

    /// 获取所有 Zone 操作
    pub async fn get_all_zone_operations(&self, region_id: &str) -> Result<Vec<ZoneOperation>> {
        if let Some(repo) = &self.repository {
            repo.get_by_region(region_id).await
        } else {
            Ok(vec![])
        }
    }

    fn zone_key(zone_id: &str, region_id: &str) -> String {
        format!("{}:{}", zone_id, region_id)
    }
}
```

### 发现服务集成

```rust
// artemis-server/src/discovery/filter.rs (新增)

/// Zone 过滤器
pub struct ZoneDiscoveryFilter {
    zone_manager: Arc<ZoneManager>,
}

impl ZoneDiscoveryFilter {
    pub fn new(zone_manager: Arc<ZoneManager>) -> Self {
        Self { zone_manager }
    }
}

#[async_trait]
impl DiscoveryFilter for ZoneDiscoveryFilter {
    async fn filter(&self, service: &mut Service, config: &DiscoveryConfig) -> Result<()> {
        let before_count = service.instances.len();

        service.instances.retain(|inst| {
            !self.zone_manager.is_zone_down(&inst.zone_id, &inst.region_id)
        });

        let filtered = before_count - service.instances.len();
        if filtered > 0 {
            info!(
                "ZoneDiscoveryFilter: filtered {} instances from zone-pulled-out zones",
                filtered
            );
        }

        Ok(())
    }
}
```

### HTTP API

```rust
// artemis-web/src/api/zone.rs (新增文件)

/// 操作 Zone API
pub async fn operate_zone(
    State(state): State<Arc<AppState>>,
    Json(req): Json<OperateZoneRequest>,
) -> Json<OperateZoneResponse> {
    let operation = ZoneOperation {
        zone_id: req.zone_id,
        region_id: req.region_id,
        operation: req.operation.parse().unwrap(),
        operation_complete: req.operation_complete,
        operator_id: req.operator_id,
        token: req.token,
    };

    match state.zone_manager.operate_zone(operation).await {
        Ok(_) => Json(OperateZoneResponse {
            status: ResponseStatus::success(),
        }),
        Err(e) => Json(OperateZoneResponse {
            status: ResponseStatus::error(&e.to_string()),
        }),
    }
}

// 其他 4 个 API...

pub fn create_zone_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/management/zone/operate-zone.json", post(operate_zone))
        .route("/api/management/zone/is-zone-down.json", post(is_zone_down))
        .route("/api/management/zone/get-zone-operations.json", post(get_zone_operations))
        .route("/api/management/zone/get-all-zone-operations.json", get(get_all_zone_operations))
}
```

### 测试

```bash
# test-zone-management.sh

#!/bin/bash

echo "========== Phase 15: Zone Management Test =========="

# 1. 注册实例到不同 Zone
echo "Step 1: Register instances in different zones..."

# 2. 拉出 Zone
echo "Step 2: Pull-out zone-1..."
curl -X POST http://localhost:8080/api/management/zone/operate-zone.json -d '{
  "zone_id": "zone-1",
  "region_id": "us-east",
  "operation": "pullout",
  "operation_complete": true
}'

# 3. 验证发现服务过滤 zone-1 实例
echo "Step 3: Verify discovery filters zone-1 instances..."

# 4. 拉入 Zone
echo "Step 4: Pull-in zone-1..."

# 5. 验证实例恢复
echo "Step 5: Verify instances restored..."

echo "========== All Tests Passed! =========="
```

---

## 🎯 Phase 16: 金丝雀发布 (2-3 天)

### 目标

实现基于 IP 白名单的金丝雀发布:
- ✅ 金丝雀配置管理
- ✅ IP 白名单路由
- ✅ 1 个 HTTP API
- ✅ 集成到发现服务过滤器
- ✅ 持久化到数据库

### 核心实现

```rust
// artemis-management/src/canary.rs (新增文件)

/// 金丝雀配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryConfig {
    pub service_id: String,
    pub region_id: String,
    pub canary_ips: Vec<String>,  // IP 白名单
    pub enabled: bool,
}

/// 金丝雀管理器
#[derive(Clone)]
pub struct CanaryManager {
    /// 配置存储: service_key (service_id:region_id) -> CanaryConfig
    configs: Arc<DashMap<String, CanaryConfig>>,
    /// Repository
    repository: Option<Arc<dyn CanaryRepository>>,
}

impl CanaryManager {
    pub fn new() -> Self {
        Self {
            configs: Arc::new(DashMap::new()),
            repository: None,
        }
    }

    /// 更新金丝雀 IP 白名单
    pub async fn update_canary_ips(&self, config: CanaryConfig) -> Result<()> {
        let key = Self::service_key(&config.service_id, &config.region_id);

        // 写入数据库
        if let Some(repo) = &self.repository {
            repo.upsert(&config).await?;
        }

        // 更新内存
        self.configs.insert(key, config);

        Ok(())
    }

    /// 获取金丝雀配置
    pub fn get_canary_config(&self, service_id: &str, region_id: &str) -> Option<CanaryConfig> {
        let key = Self::service_key(service_id, region_id);
        self.configs.get(&key).map(|c| c.clone())
    }

    fn service_key(service_id: &str, region_id: &str) -> String {
        format!("{}:{}", service_id, region_id)
    }
}
```

### 发现服务集成

```rust
// artemis-server/src/discovery/filter.rs (新增)

/// 金丝雀过滤器
pub struct CanaryDiscoveryFilter {
    canary_manager: Arc<CanaryManager>,
}

#[async_trait]
impl DiscoveryFilter for CanaryDiscoveryFilter {
    async fn filter(&self, service: &mut Service, config: &DiscoveryConfig) -> Result<()> {
        // 1. 获取金丝雀配置
        if let Some(canary_config) = self.canary_manager.get_canary_config(
            &service.service_id,
            &config.region_id
        ) {
            if !canary_config.enabled {
                return Ok(());
            }

            // 2. 获取客户端 IP (从请求上下文)
            let client_ip = config.client_ip.as_ref()
                .ok_or_else(|| anyhow::anyhow!("Client IP not available"))?;

            // 3. 判断是否在白名单
            let is_canary_client = canary_config.canary_ips.contains(client_ip);

            // 4. 过滤实例
            service.instances.retain(|inst| {
                let is_canary_instance = inst.metadata
                    .get("canary")
                    .map(|v| v == "true")
                    .unwrap_or(false);

                if is_canary_client {
                    // 白名单内 - 仅返回金丝雀实例
                    is_canary_instance
                } else {
                    // 非白名单 - 过滤金丝雀实例
                    !is_canary_instance
                }
            });

            info!(
                "CanaryDiscoveryFilter: client_ip={}, is_canary={}, instances={}",
                client_ip,
                is_canary_client,
                service.instances.len()
            );
        }

        Ok(())
    }
}
```

### HTTP API

```rust
// artemis-web/src/api/canary.rs (新增文件)

pub async fn update_canary_ips(
    State(state): State<Arc<AppState>>,
    Json(req): Json<UpdateCanaryIPsRequest>,
) -> Json<UpdateCanaryIPsResponse> {
    let config = CanaryConfig {
        service_id: req.service_id,
        region_id: req.region_id,
        canary_ips: req.canary_ips,
        enabled: req.enabled.unwrap_or(true),
    };

    match state.canary_manager.update_canary_ips(config).await {
        Ok(_) => Json(UpdateCanaryIPsResponse {
            status: ResponseStatus::success(),
        }),
        Err(e) => Json(UpdateCanaryIPsResponse {
            status: ResponseStatus::error(&e.to_string()),
        }),
    }
}

pub fn create_canary_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/management/canary/update-canary-ips.json", post(update_canary_ips))
}
```

### 测试

```bash
# test-canary.sh

#!/bin/bash

echo "========== Phase 16: Canary Release Test =========="

# 1. 注册普通实例和金丝雀实例
echo "Step 1: Register normal and canary instances..."
# 金丝雀实例设置 metadata: {"canary": "true"}

# 2. 配置金丝雀 IP 白名单
echo "Step 2: Configure canary IP whitelist..."
curl -X POST http://localhost:8080/api/management/canary/update-canary-ips.json -d '{
  "service_id": "test-service",
  "region_id": "us-east",
  "canary_ips": ["192.168.1.100", "192.168.1.101"],
  "enabled": true
}'

# 3. 从白名单 IP 发现服务 - 应该只返回金丝雀实例
echo "Step 3: Discover from canary IP..."
curl -X POST http://localhost:8080/api/discovery/service.json \
  -H "X-Forwarded-For: 192.168.1.100" \
  -d '{"discovery_config": {"service_id": "test-service", ...}}'

# 4. 从非白名单 IP 发现服务 - 应该只返回普通实例
echo "Step 4: Discover from normal IP..."
curl -X POST http://localhost:8080/api/discovery/service.json \
  -H "X-Forwarded-For: 192.168.1.200" \
  -d '{"discovery_config": {"service_id": "test-service", ...}}'

echo "========== All Tests Passed! =========="
```

---

## 🔄 数据流和错误处理

### 完整数据流

```
Client Request → HTTP API
    ↓
Business Logic Layer (Manager)
    ↓
┌─────────────────────────────────┐
│ Write Path:                     │
│ 1. Write to Database (强一致)   │
│ 2. Update Memory Cache          │
└─────────────────────────────────┘
    ↓
┌─────────────────────────────────┐
│ Read Path:                      │
│ 1. Read from Memory Cache       │
│ 2. Cache Miss → Read Database   │
│ 3. Update Cache                 │
└─────────────────────────────────┘
    ↓
Discovery Service (Apply Filters)
    ↓
1. StatusFilter
2. ManagementDiscoveryFilter (实例/服务器拉出)
3. ZoneDiscoveryFilter (Zone 拉出)
4. CanaryDiscoveryFilter (金丝雀 IP)
5. GroupRoutingFilter (分组路由策略)
    ↓
Return Filtered Instances
```

### 错误处理策略

**1. 数据库连接失败**:
```rust
// 启动时连接失败 → 退出程序并记录错误
// 运行时连接失败 → 降级为仅内存模式,记录告警
```

**2. 数据库写入失败**:
```rust
// 事务回滚,返回错误给客户端
// 内存缓存不更新 (保持数据一致性)
```

**3. 数据库读取失败**:
```rust
// 使用内存缓存数据
// 异步重试读取
// 记录错误日志
```

**4. 路由策略执行失败**:
```rust
// 降级为返回所有实例 (不应用路由)
// 记录错误,不中断请求
```

**5. 并发冲突**:
```rust
// 使用乐观锁 (version 字段)
// 冲突时返回 409 Conflict
// 客户端重试
```

---

## 📊 测试策略

### 单元测试

**每个模块独立测试**:
- RouteEngine 路由策略测试
- Manager CRUD 操作测试
- Repository 数据库 CRUD 测试 (使用 SQLite 内存模式)

### 集成测试

**Phase 13**: `test-group-routing.sh`
- 完整的分组路由流程测试
- 验证两种路由策略

**Phase 14**: `test-persistence.sh`
- 写入配置 → 重启服务 → 验证数据恢复

**Phase 15**: `test-zone-management.sh`
- Zone 拉出/拉入测试

**Phase 16**: `test-canary.sh`
- 金丝雀 IP 白名单测试

**端到端测试**: `test-all-features.sh`
- 综合测试所有功能

### 性能测试

使用 Criterion benchmark:

```rust
// benches/routing_benchmark.rs

#[bench]
fn bench_weighted_round_robin(b: &mut Bencher) {
    // 10k 实例,100 个分组
    // 测试路由策略性能
}

#[bench]
fn bench_database_write(b: &mut Bencher) {
    // 测试数据库写入性能
}
```

---

## 📈 性能目标

### 延迟目标

| 操作 | Phase 13 (内存) | Phase 14 (数据库) | 目标 |
|------|----------------|------------------|------|
| 路由策略执行 | < 1ms | < 1ms | < 2ms |
| Group CRUD | < 0.5ms | < 10ms | < 20ms |
| 发现服务 (含路由) | < 1ms | < 2ms | < 5ms |

### 吞吐量目标

- 发现服务 QPS: 10,000+ (保持现有水平)
- 管理 API QPS: 1,000+

### 资源占用

- 内存增长: < 500MB (10k 分组 + 100k 实例)
- 数据库连接: 10-20 个

---

## 📝 文档和交付

### 文档更新

**Phase 13**:
- README.md - 新增分组路由功能介绍
- API 文档 - 新增 27 个 API 端点
- docs/GROUP_ROUTING.md - 分组路由使用指南

**Phase 14**:
- README.md - 新增数据库配置说明
- docs/DATABASE_SETUP.md - 数据库安装和配置
- docs/PERSISTENCE.md - 持久化架构文档

**Phase 15/16**:
- README.md - 更新功能列表
- API 文档 - 新增 6 个 API 端点

### 交付清单

**Phase 13**:
- ✅ 代码实现 (6-8 个新文件,1000+ 行)
- ✅ 单元测试 (50+ 测试)
- ✅ 集成测试脚本
- ✅ 文档更新

**Phase 14**:
- ✅ 代码实现 (10+ 个文件,800+ 行)
- ✅ 数据库迁移脚本 (3 个数据库)
- ✅ 单元测试
- ✅ 集成测试脚本
- ✅ 文档更新

**Phase 15**:
- ✅ 代码实现 (3-4 个文件,300+ 行)
- ✅ 测试
- ✅ 文档

**Phase 16**:
- ✅ 代码实现 (3-4 个文件,200+ 行)
- ✅ 测试
- ✅ 文档

---

## 🎯 成功标准

### Phase 13 完成标准

1. ✅ 所有 27 个 API 实现并通过测试
2. ✅ 两种路由策略算法正确 (分布误差 < 5%)
3. ✅ 集成测试通过
4. ✅ 代码零警告
5. ✅ 文档完整

### Phase 14 完成标准

1. ✅ 支持 MySQL/PostgreSQL/SQLite
2. ✅ 所有 Repository 通过测试
3. ✅ 重启后数据正确恢复
4. ✅ 性能达标 (管理 API < 20ms)

### Phase 15/16 完成标准

1. ✅ 功能实现并通过测试
2. ✅ 集成到发现服务
3. ✅ 持久化正常工作

### 最终验收标准

1. ✅ **功能完整度**: 100% 对齐 Java 版本
2. ✅ **测试覆盖**: 单元 + 集成 + 性能全通过
3. ✅ **性能达标**: 所有性能目标满足
4. ✅ **代码质量**: 零警告,clippy 通过
5. ✅ **文档完整**: 用户文档 + API 文档 + 设计文档

---

## 🚀 实施时间线

### Week 1: Phase 13 (分组路由)
- Day 1-2: 数据模型 + Manager 层
- Day 3-4: 路由策略引擎 + 发现服务集成
- Day 5-7: HTTP API + 测试

### Week 2: Phase 14 (数据持久化)
- Day 1-2: 数据库抽象层 + Schema
- Day 3-4: Repository 实现
- Day 5-6: Manager 层改造 + 测试

### Week 3: Phase 15 & 16
- Day 1-2: Zone 管理
- Day 3: 金丝雀发布
- Day 4-5: 综合测试 + 文档

---

## 📚 参考资料

- Java 版本源码: `artemis-java/`
- 功能审查报告: `docs/FEATURE_COMPARISON_REPORT_2026-02-14.md`
- 现有实现: `artemis-management/src/` (Phase 13 框架)
- 数据库 Schema: `artemis-java/artemis-management/src/main/resources/artemis-management.sql`

---

**设计版本**: 1.0
**审批状态**: 待审批
**下一步**: 创建实施计划 (使用 writing-plans skill)
