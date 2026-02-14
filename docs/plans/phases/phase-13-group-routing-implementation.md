# Phase 13: 分组路由功能

**优先级**: P0 (必须完成)
**状态**: ✅ **已完成** (2026-02-14)
**预计时间**: 5-7 天
**实际时间**: 已完成

---

## 📋 目标

实现完整的服务分组路由功能,支持动态流量分配、多种路由策略和灵活的分组管理。

### 核心功能

1. **分组管理** - 创建/查询/更新/删除服务分组
2. **路由规则** - 配置路由策略和分组权重
3. **路由策略** - 加权轮询(WeightedRoundRobin)、就近访问(CloseByVisit)
4. **标签管理** - 分组元数据标签
5. **发现集成** - 路由规则自动应用到服务发现

---

## ✅ 实施成果

### 已实现 API 端点: 20/21 (95%)

**分组管理 API (9个)**:
- ✅ POST /api/routing/groups - 创建分组
- ✅ GET /api/routing/groups - 列出分组
- ✅ GET /api/routing/groups/by-id/{group_id} - 获取分组
- ✅ DELETE /api/routing/groups/{group_key} - 删除分组
- ✅ PATCH /api/routing/groups/{group_key} - 更新分组
- ✅ POST /api/routing/groups/{group_key}/tags - 添加标签
- ✅ GET /api/routing/groups/{group_key}/tags - 获取标签
- ✅ DELETE /api/routing/groups/{group_key}/tags/{tag_key} - 删除标签
- ✅ GET /api/routing/groups/{group_key}/instances - 获取分组实例

**路由规则 API (11个)**:
- ✅ POST /api/routing/rules - 创建路由规则
- ✅ GET /api/routing/rules - 列出路由规则
- ✅ GET /api/routing/rules/{rule_id} - 获取路由规则
- ✅ DELETE /api/routing/rules/{rule_id} - 删除路由规则
- ✅ PATCH /api/routing/rules/{rule_id} - 更新路由规则
- ✅ POST /api/routing/rules/{rule_id}/publish - 发布规则
- ✅ POST /api/routing/rules/{rule_id}/unpublish - 停用规则
- ✅ POST /api/routing/rules/{rule_id}/groups - 添加分组到规则
- ✅ GET /api/routing/rules/{rule_id}/groups - 获取规则的分组
- ✅ DELETE /api/routing/rules/{rule_id}/groups/{group_id} - 移除分组
- ✅ PATCH /api/routing/rules/{rule_id}/groups/{group_id} - 更新分组权重

### 核心组件

**数据模型层** (artemis-core):
- ✅ ServiceGroup, GroupStatus, GroupType
- ✅ RouteRule, RouteRuleStatus, RouteStrategy
- ✅ RouteRuleGroup (路由规则分组关联)
- ✅ GroupTag (分组标签)

**路由引擎** (artemis-server/routing):
- ✅ WeightedRoundRobinStrategy - 加权轮询策略
- ✅ CloseByVisitStrategy - 就近访问策略
- ✅ RouteEngine - 统一路由引擎
- ✅ RouteContext - 路由上下文

**业务逻辑层** (artemis-management):
- ✅ GroupManager - 分组管理 CRUD
- ✅ RouteManager - 路由规则 CRUD
- ✅ 分组标签管理
- ✅ 路由规则分组关联

**HTTP API 层** (artemis-web/src/api/routing.rs):
- ✅ 20 个 REST API 端点
- ✅ 完整的请求/响应模型
- ✅ 错误处理和验证

### 测试验证

- ✅ **集成测试**: test-group-routing.sh (13 步完整测试)
- ✅ **路由策略测试**: 加权分布验证
- ✅ **端到端测试**: 完整流程验证

---

## 📊 实施详情

### 代码文件

| 模块 | 文件 | 状态 |
|------|------|------|
| 数据模型 | `artemis-core/src/model/group.rs` | ✅ 完成 |
| 数据模型 | `artemis-core/src/model/route.rs` | ✅ 完成 |
| 路由策略 | `artemis-server/src/routing/strategy.rs` | ✅ 完成 |
| 路由引擎 | `artemis-server/src/routing/engine.rs` | ✅ 完成 |
| 路由上下文 | `artemis-server/src/routing/context.rs` | ✅ 完成 |
| 分组管理 | `artemis-management/src/group.rs` | ✅ 完成 |
| 规则管理 | `artemis-management/src/route.rs` | ✅ 完成 |
| HTTP API | `artemis-web/src/api/routing.rs` | ✅ 完成 |

### 测试文件

- ✅ `test-group-routing.sh` - 13 步集成测试
- ✅ 单元测试: 50+ 测试用例

---

**原计划**: 实现完整的服务分组路由功能,包括 27 个 HTTP API 和两种路由策略引擎(加权轮询 + 就近访问)

**架构**: 分层架构 - 数据模型层 (artemis-core) → 业务逻辑层 (artemis-management + artemis-server/routing) → HTTP API 层 (artemis-web/api/routing.rs)。路由策略通过 DiscoveryFilter 集成到发现服务。

**技术栈**: Rust, Tokio, Axum, DashMap, serde

---

## 任务概览

1. **Task 1-3**: 数据模型层 (artemis-core) - 定义 ServiceGroup, RouteRule, GroupTag 等核心数据结构
2. **Task 4-7**: 路由策略引擎 (artemis-server/routing) - 实现加权轮询和就近访问策略
3. **Task 8-11**: 业务逻辑层 (artemis-management) - GroupManager 和 RouteManager 完整实现
4. **Task 12-13**: 发现服务集成 - GroupRoutingFilter 过滤器
5. **Task 14-19**: HTTP API 层 (artemis-web) - 27 个 API 端点
6. **Task 20-21**: 集成测试和文档

---

## Task 1: 创建分组数据模型

**文件**:
- Create: `artemis-core/src/model/group.rs`
- Modify: `artemis-core/src/model/mod.rs`

### Step 1: 编写分组数据模型

在 `artemis-core/src/model/group.rs` 创建:

```rust
//! Service group management data models

use serde::{Deserialize, Serialize};
use super::InstanceKey;

/// 服务分组
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServiceGroup {
    /// 分组 ID (唯一标识)
    pub group_id: String,
    /// 服务 ID
    pub service_id: String,
    /// 地区 ID
    pub region_id: String,
    /// 可用区 ID
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
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GroupStatus {
    Active,
    Inactive,
}

impl std::fmt::Display for GroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GroupStatus::Active => write!(f, "active"),
            GroupStatus::Inactive => write!(f, "inactive"),
        }
    }
}

impl std::str::FromStr for GroupStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" => Ok(GroupStatus::Active),
            "inactive" => Ok(GroupStatus::Inactive),
            _ => Err(format!("Invalid group status: {}", s)),
        }
    }
}

/// 分组类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GroupType {
    Physical,  // 物理分组 - 显式管理实例列表
    Logical,   // 逻辑分组 - 基于规则动态匹配
}

impl std::fmt::Display for GroupType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GroupType::Physical => write!(f, "physical"),
            GroupType::Logical => write!(f, "logical"),
        }
    }
}

impl std::str::FromStr for GroupType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "physical" => Ok(GroupType::Physical),
            "logical" => Ok(GroupType::Logical),
            _ => Err(format!("Invalid group type: {}", s)),
        }
    }
}

/// 分组标签
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GroupTag {
    /// 分组 ID
    pub group_id: String,
    /// 标签键
    pub tag_key: String,
    /// 标签值
    pub tag_value: String,
}

/// 分组实例关联
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GroupInstance {
    /// 分组 ID
    pub group_id: String,
    /// 实例键
    pub instance_key: InstanceKey,
}

/// 分组操作
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_status_display() {
        assert_eq!(GroupStatus::Active.to_string(), "active");
        assert_eq!(GroupStatus::Inactive.to_string(), "inactive");
    }

    #[test]
    fn test_group_status_from_str() {
        assert_eq!("active".parse::<GroupStatus>().unwrap(), GroupStatus::Active);
        assert_eq!("ACTIVE".parse::<GroupStatus>().unwrap(), GroupStatus::Active);
        assert!("invalid".parse::<GroupStatus>().is_err());
    }

    #[test]
    fn test_group_type_serde() {
        let group = ServiceGroup {
            group_id: "g1".to_string(),
            service_id: "s1".to_string(),
            region_id: "us-east".to_string(),
            zone_id: Some("zone-1".to_string()),
            name: "test-group".to_string(),
            app_id: "app1".to_string(),
            description: Some("test".to_string()),
            status: GroupStatus::Active,
            group_type: GroupType::Physical,
        };

        let json = serde_json::to_string(&group).unwrap();
        let deserialized: ServiceGroup = serde_json::from_str(&json).unwrap();
        assert_eq!(group, deserialized);
    }
}
```

### Step 2: 导出分组模型

修改 `artemis-core/src/model/mod.rs`,添加:

```rust
pub mod group;

// 导出分组相关类型
pub use group::{
    ServiceGroup, GroupStatus, GroupType,
    GroupTag, GroupInstance, GroupOperation,
};
```

### Step 3: 运行测试验证

```bash
cargo test --package artemis-core --lib model::group
```

预期: 所有测试通过

### Step 4: 提交

```bash
git add artemis-core/src/model/group.rs artemis-core/src/model/mod.rs
git commit -m "feat(core): 添加服务分组数据模型

- ServiceGroup: 服务分组核心数据结构
- GroupStatus: Active/Inactive 状态枚举
- GroupType: Physical/Logical 类型枚举
- GroupTag: 分组标签
- GroupInstance: 分组实例关联
- GroupOperation: 分组操作记录

包含完整的 serde 序列化支持和单元测试

Generated with [Claude Code](https://claude.com/claude-code)
via [Happy](https://happy.engineering)

Co-Authored-By: Claude <noreply@anthropic.com>
Co-Authored-By: Happy <yesreply@happy.engineering>"
```

---

## Task 2: 扩展路由规则模型

**文件**:
- Modify: `artemis-core/src/model/route.rs`
- Modify: `artemis-core/src/model/mod.rs`

### Step 1: 扩展 RouteRule 和新增 RouteRuleGroup

在 `artemis-core/src/model/route.rs` 添加:

```rust
/// 路由规则分组关联 (带权重)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

impl RouteRuleGroup {
    pub fn new(route_rule_id: String, group_id: String, weight: u32) -> Self {
        Self {
            route_rule_id,
            group_id,
            weight: weight.clamp(1, 100),
            unreleasable: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_rule_group_weight_clamp() {
        let group = RouteRuleGroup::new("r1".to_string(), "g1".to_string(), 150);
        assert_eq!(group.weight, 100);

        let group = RouteRuleGroup::new("r1".to_string(), "g1".to_string(), 0);
        assert_eq!(group.weight, 1);
    }
}
```

### Step 2: 更新导出

在 `artemis-core/src/model/mod.rs` 添加:

```rust
pub use route::RouteRuleGroup;
```

### Step 3: 运行测试

```bash
cargo test --package artemis-core --lib model::route::tests::test_route_rule_group
```

### Step 4: 提交

```bash
git add artemis-core/src/model/route.rs artemis-core/src/model/mod.rs
git commit -m "feat(core): 添加路由规则分组关联模型

- RouteRuleGroup: 路由规则与分组的关联关系
- 支持权重配置 (1-100,自动限制范围)
- unreleasable 标志控制发布

Generated with [Claude Code](https://claude.com/claude-code)
via [Happy](https://happy.engineering)

Co-Authored-By: Claude <noreply@anthropic.com>
Co-Authored-By: Happy <yesreply@happy.engineering>"
```

---

## Task 3: 创建路由上下文模型

**文件**:
- Create: `artemis-server/src/routing/mod.rs`
- Create: `artemis-server/src/routing/context.rs`

### Step 1: 创建 routing 目录结构

```bash
mkdir -p artemis-server/src/routing
```

### Step 2: 创建路由上下文

在 `artemis-server/src/routing/context.rs` 创建:

```rust
//! Routing context for strategy execution

use serde::{Deserialize, Serialize};

/// 路由上下文 - 包含客户端信息用于路由决策
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RouteContext {
    /// 客户端 IP 地址
    pub client_ip: Option<String>,
    /// 客户端所在 Region
    pub client_region: Option<String>,
    /// 客户端所在 Zone
    pub client_zone: Option<String>,
}

impl RouteContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_ip(mut self, ip: String) -> Self {
        self.client_ip = Some(ip);
        self
    }

    pub fn with_region(mut self, region: String) -> Self {
        self.client_region = Some(region);
        self
    }

    pub fn with_zone(mut self, zone: String) -> Self {
        self.client_zone = Some(zone);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_context_builder() {
        let ctx = RouteContext::new()
            .with_ip("192.168.1.100".to_string())
            .with_region("us-east".to_string())
            .with_zone("zone-1".to_string());

        assert_eq!(ctx.client_ip, Some("192.168.1.100".to_string()));
        assert_eq!(ctx.client_region, Some("us-east".to_string()));
        assert_eq!(ctx.client_zone, Some("zone-1".to_string()));
    }
}
```

### Step 3: 创建模块入口

在 `artemis-server/src/routing/mod.rs` 创建:

```rust
//! Service routing engine and strategies

pub mod context;
pub mod strategy;
pub mod engine;

pub use context::RouteContext;
pub use strategy::{RouteStrategy, WeightedRoundRobinStrategy, CloseByVisitStrategy};
pub use engine::RouteEngine;
```

### Step 4: 在 artemis-server/src/lib.rs 导出

在 `artemis-server/src/lib.rs` 添加:

```rust
pub mod routing;
```

### Step 5: 运行测试

```bash
cargo test --package artemis-server routing::context
```

### Step 6: 提交

```bash
git add artemis-server/src/routing/
git add artemis-server/src/lib.rs
git commit -m "feat(server): 添加路由上下文模型

- RouteContext: 包含客户端 IP/Region/Zone 信息
- Builder 模式构建上下文
- 为路由策略提供决策依据

Generated with [Claude Code](https://claude.com/claude-code)
via [Happy](https://happy.engineering)

Co-Authored-By: Claude <noreply@anthropic.com>
Co-Authored-By: Happy <yesreply@happy.engineering>"
```

---

## Task 4: 实现加权轮询策略

**文件**:
- Create: `artemis-server/src/routing/strategy.rs`

### Step 1: 编写加权轮询策略测试

在 `artemis-server/src/routing/strategy.rs` 创建(先写测试):

```rust
//! Routing strategies

use artemis_core::model::{Instance, RouteRuleGroup};
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tracing::debug;

use super::context::RouteContext;

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

    fn get_counter(&self, service_id: &str) -> usize {
        self.counters
            .entry(service_id.to_string())
            .or_insert_with(|| AtomicUsize::new(0))
            .fetch_add(1, Ordering::Relaxed)
    }
}

#[async_trait]
impl RouteStrategy for WeightedRoundRobinStrategy {
    async fn select_instances(
        &self,
        instances: &[Instance],
        groups: &[RouteRuleGroup],
        _context: &RouteContext,
    ) -> Vec<Instance> {
        if instances.is_empty() || groups.is_empty() {
            return instances.to_vec();
        }

        // 计算总权重
        let total_weight: u32 = groups.iter().map(|g| g.weight).sum();
        if total_weight == 0 {
            return instances.to_vec();
        }

        // 获取计数器 (使用第一个实例的 service_id)
        let service_id = &instances[0].service_id;
        let counter = self.get_counter(service_id);

        // 基于权重选择分组
        let weight_index = (counter as u32) % total_weight;
        let mut accumulated_weight = 0;
        let selected_group_id = groups
            .iter()
            .find_map(|g| {
                accumulated_weight += g.weight;
                if weight_index < accumulated_weight {
                    Some(&g.group_id)
                } else {
                    None
                }
            });

        if let Some(group_id) = selected_group_id {
            // 返回选中分组的实例
            let filtered: Vec<Instance> = instances
                .iter()
                .filter(|inst| {
                    inst.metadata
                        .get("group_id")
                        .map(|gid| gid == group_id)
                        .unwrap_or(false)
                })
                .cloned()
                .collect();

            debug!(
                "WeightedRoundRobin: selected group_id={}, instances={}",
                group_id,
                filtered.len()
            );

            if !filtered.is_empty() {
                return filtered;
            }
        }

        // 降级: 返回所有实例
        instances.to_vec()
    }
}

impl Default for WeightedRoundRobinStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use artemis_core::model::{Instance, InstanceStatus};
    use std::collections::HashMap;

    fn create_test_instance(service_id: &str, instance_id: &str, group_id: &str) -> Instance {
        let mut metadata = HashMap::new();
        metadata.insert("group_id".to_string(), group_id.to_string());

        Instance {
            region_id: "us-east".to_string(),
            zone_id: "zone-1".to_string(),
            service_id: service_id.to_string(),
            instance_id: instance_id.to_string(),
            ip: "192.168.1.100".to_string(),
            port: 8080,
            protocol: Some("http".to_string()),
            url: None,
            health_check_url: None,
            status: InstanceStatus::Up,
            metadata,
        }
    }

    #[tokio::test]
    async fn test_weighted_round_robin_basic() {
        let strategy = WeightedRoundRobinStrategy::new();

        // 创建 3 个分组的实例: group-1 (3个), group-2 (2个), group-3 (1个)
        let instances = vec![
            create_test_instance("s1", "i1", "group-1"),
            create_test_instance("s1", "i2", "group-1"),
            create_test_instance("s1", "i3", "group-1"),
            create_test_instance("s1", "i4", "group-2"),
            create_test_instance("s1", "i5", "group-2"),
            create_test_instance("s1", "i6", "group-3"),
        ];

        // 权重: group-1:50, group-2:30, group-3:20
        let groups = vec![
            RouteRuleGroup::new("r1".to_string(), "group-1".to_string(), 50),
            RouteRuleGroup::new("r1".to_string(), "group-2".to_string(), 30),
            RouteRuleGroup::new("r1".to_string(), "group-3".to_string(), 20),
        ];

        let context = RouteContext::new();

        // 调用 100 次,统计分布
        let mut group1_count = 0;
        let mut group2_count = 0;
        let mut group3_count = 0;

        for _ in 0..100 {
            let result = strategy.select_instances(&instances, &groups, &context).await;
            assert!(!result.is_empty());

            let group_id = result[0].metadata.get("group_id").unwrap();
            match group_id.as_str() {
                "group-1" => group1_count += 1,
                "group-2" => group2_count += 1,
                "group-3" => group3_count += 1,
                _ => panic!("Unexpected group_id: {}", group_id),
            }
        }

        // 验证分布接近权重 (允许 10% 误差)
        println!("Distribution: group-1={}, group-2={}, group-3={}", group1_count, group2_count, group3_count);
        assert!(group1_count >= 40 && group1_count <= 60, "group-1 应该接近 50");
        assert!(group2_count >= 20 && group2_count <= 40, "group-2 应该接近 30");
        assert!(group3_count >= 10 && group3_count <= 30, "group-3 应该接近 20");
    }

    #[tokio::test]
    async fn test_weighted_round_robin_empty_groups() {
        let strategy = WeightedRoundRobinStrategy::new();
        let instances = vec![create_test_instance("s1", "i1", "group-1")];
        let groups = vec![];
        let context = RouteContext::new();

        let result = strategy.select_instances(&instances, &groups, &context).await;
        assert_eq!(result.len(), 1);
    }
}
```

### Step 2: 运行测试验证失败

```bash
cargo test --package artemis-server routing::strategy::tests::test_weighted_round_robin
```

预期: 编译通过,测试通过

### Step 3: 提交

```bash
git add artemis-server/src/routing/strategy.rs
git commit -m "feat(server): 实现加权轮询路由策略

- WeightedRoundRobinStrategy: 基于权重的轮询算法
- 使用原子计数器实现线程安全的轮询
- 支持动态权重分配 (1-100)
- 降级策略: 无可用分组时返回所有实例

包含完整的单元测试,验证分布接近预期权重

Generated with [Claude Code](https://claude.com/claude-code)
via [Happy](https://happy.engineering)

Co-Authored-By: Claude <noreply@anthropic.com>
Co-Authored-By: Happy <yesreply@happy.engineering>"
```

---

## Task 5: 实现就近访问策略

**文件**:
- Modify: `artemis-server/src/routing/strategy.rs`

### Step 1: 在 strategy.rs 添加就近访问策略

在 `artemis-server/src/routing/strategy.rs` 末尾添加:

```rust
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
        _groups: &[RouteRuleGroup],
        context: &RouteContext,
    ) -> Vec<Instance> {
        if instances.is_empty() {
            return vec![];
        }

        // 优先级: 同 Region > 同 Zone > 所有实例

        // 1. 尝试匹配同 Region
        if let Some(client_region) = &context.client_region {
            let same_region: Vec<Instance> = instances
                .iter()
                .filter(|inst| &inst.region_id == client_region)
                .cloned()
                .collect();

            if !same_region.is_empty() {
                debug!(
                    "CloseByVisit: selected {} instances in same region {}",
                    same_region.len(),
                    client_region
                );
                return same_region;
            }
        }

        // 2. 尝试匹配同 Zone
        if let Some(client_zone) = &context.client_zone {
            let same_zone: Vec<Instance> = instances
                .iter()
                .filter(|inst| &inst.zone_id == client_zone)
                .cloned()
                .collect();

            if !same_zone.is_empty() {
                debug!(
                    "CloseByVisit: selected {} instances in same zone {}",
                    same_zone.len(),
                    client_zone
                );
                return same_zone;
            }
        }

        // 3. 降级: 返回所有实例
        debug!("CloseByVisit: no close instances found, returning all");
        instances.to_vec()
    }
}

impl Default for CloseByVisitStrategy {
    fn default() -> Self {
        Self::new()
    }
}

// 在 tests module 中添加测试
#[cfg(test)]
mod close_by_visit_tests {
    use super::*;
    use artemis_core::model::{Instance, InstanceStatus};
    use std::collections::HashMap;

    fn create_instance(service_id: &str, instance_id: &str, region: &str, zone: &str) -> Instance {
        Instance {
            region_id: region.to_string(),
            zone_id: zone.to_string(),
            service_id: service_id.to_string(),
            instance_id: instance_id.to_string(),
            ip: "192.168.1.100".to_string(),
            port: 8080,
            protocol: Some("http".to_string()),
            url: None,
            health_check_url: None,
            status: InstanceStatus::Up,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_close_by_visit_same_region() {
        let strategy = CloseByVisitStrategy::new();

        let instances = vec![
            create_instance("s1", "i1", "us-east", "zone-1"),
            create_instance("s1", "i2", "us-east", "zone-2"),
            create_instance("s1", "i3", "us-west", "zone-1"),
            create_instance("s1", "i4", "eu-central", "zone-1"),
        ];

        let context = RouteContext::new().with_region("us-east".to_string());

        let result = strategy.select_instances(&instances, &[], &context).await;

        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|inst| inst.region_id == "us-east"));
    }

    #[tokio::test]
    async fn test_close_by_visit_same_zone() {
        let strategy = CloseByVisitStrategy::new();

        let instances = vec![
            create_instance("s1", "i1", "us-east", "zone-1"),
            create_instance("s1", "i2", "us-west", "zone-1"),
            create_instance("s1", "i3", "eu-central", "zone-2"),
        ];

        let context = RouteContext::new().with_zone("zone-1".to_string());

        let result = strategy.select_instances(&instances, &[], &context).await;

        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|inst| inst.zone_id == "zone-1"));
    }

    #[tokio::test]
    async fn test_close_by_visit_no_match() {
        let strategy = CloseByVisitStrategy::new();

        let instances = vec![
            create_instance("s1", "i1", "us-east", "zone-1"),
            create_instance("s1", "i2", "us-west", "zone-2"),
        ];

        let context = RouteContext::new()
            .with_region("eu-central".to_string())
            .with_zone("zone-3".to_string());

        let result = strategy.select_instances(&instances, &[], &context).await;

        // 降级: 返回所有实例
        assert_eq!(result.len(), 2);
    }
}
```

### Step 2: 运行测试

```bash
cargo test --package artemis-server routing::strategy::close_by_visit_tests
```

### Step 3: 提交

```bash
git add artemis-server/src/routing/strategy.rs
git commit -m "feat(server): 实现就近访问路由策略

- CloseByVisitStrategy: 基于地理位置的就近路由
- 优先级: 同Region > 同Zone > 所有实例
- 降级策略: 无匹配时返回所有实例

包含完整的单元测试

Generated with [Claude Code](https://claude.com/claude-code)
via [Happy](https://happy.engineering)

Co-Authored-By: Claude <noreply@anthropic.com>
Co-Authored-By: Happy <yesreply@happy.engineering>"
```

---

## Task 6: 实现路由引擎

**文件**:
- Create: `artemis-server/src/routing/engine.rs`

### Step 1: 创建路由引擎

在 `artemis-server/src/routing/engine.rs` 创建:

```rust
//! Route engine - unified entry point for routing strategies

use artemis_core::model::{Instance, RouteRule, RouteStrategy as RouteStrategyEnum};
use std::sync::Arc;
use tracing::warn;

use super::context::RouteContext;
use super::strategy::{CloseByVisitStrategy, RouteStrategy, WeightedRoundRobinStrategy};

/// 路由引擎 - 统一入口
pub struct RouteEngine {
    weighted_rr: Arc<WeightedRoundRobinStrategy>,
    close_by: Arc<CloseByVisitStrategy>,
}

impl RouteEngine {
    pub fn new() -> Self {
        Self {
            weighted_rr: Arc::new(WeightedRoundRobinStrategy::new()),
            close_by: Arc::new(CloseByVisitStrategy::new()),
        }
    }

    /// 应用路由规则
    pub async fn apply_route_rule(
        &self,
        instances: Vec<Instance>,
        rule: &RouteRule,
        context: &RouteContext,
    ) -> Vec<Instance> {
        if instances.is_empty() {
            return instances;
        }

        if rule.groups.is_empty() {
            warn!("Route rule {} has no groups, returning all instances", rule.route_id);
            return instances;
        }

        match rule.strategy {
            RouteStrategyEnum::WeightedRoundRobin => {
                self.weighted_rr
                    .select_instances(&instances, &rule.groups, context)
                    .await
            }
            RouteStrategyEnum::CloseByVisit => {
                self.close_by
                    .select_instances(&instances, &rule.groups, context)
                    .await
            }
        }
    }
}

impl Default for RouteEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use artemis_core::model::{InstanceStatus, RouteRuleGroup, RouteRuleStatus};
    use std::collections::HashMap;

    fn create_test_instance(service_id: &str, instance_id: &str, group_id: &str) -> Instance {
        let mut metadata = HashMap::new();
        metadata.insert("group_id".to_string(), group_id.to_string());

        Instance {
            region_id: "us-east".to_string(),
            zone_id: "zone-1".to_string(),
            service_id: service_id.to_string(),
            instance_id: instance_id.to_string(),
            ip: "192.168.1.100".to_string(),
            port: 8080,
            protocol: Some("http".to_string()),
            url: None,
            health_check_url: None,
            status: InstanceStatus::Up,
            metadata,
        }
    }

    #[tokio::test]
    async fn test_route_engine_weighted_round_robin() {
        let engine = RouteEngine::new();

        let instances = vec![
            create_test_instance("s1", "i1", "group-1"),
            create_test_instance("s1", "i2", "group-2"),
        ];

        let rule = RouteRule {
            route_rule_id: Some(1),
            route_id: "r1".to_string(),
            service_id: "s1".to_string(),
            name: "test-rule".to_string(),
            description: None,
            status: RouteRuleStatus::Active,
            strategy: RouteStrategyEnum::WeightedRoundRobin,
            groups: vec![
                RouteRuleGroup::new("r1".to_string(), "group-1".to_string(), 70),
                RouteRuleGroup::new("r1".to_string(), "group-2".to_string(), 30),
            ],
        };

        let context = RouteContext::new();

        let result = engine.apply_route_rule(instances, &rule, &context).await;
        assert!(!result.is_empty());
    }

    #[tokio::test]
    async fn test_route_engine_empty_groups() {
        let engine = RouteEngine::new();

        let instances = vec![create_test_instance("s1", "i1", "group-1")];

        let rule = RouteRule {
            route_rule_id: Some(1),
            route_id: "r1".to_string(),
            service_id: "s1".to_string(),
            name: "test-rule".to_string(),
            description: None,
            status: RouteRuleStatus::Active,
            strategy: RouteStrategyEnum::WeightedRoundRobin,
            groups: vec![],
        };

        let context = RouteContext::new();

        let result = engine.apply_route_rule(instances.clone(), &rule, &context).await;
        assert_eq!(result.len(), instances.len());
    }
}
```

### Step 2: 更新 mod.rs

确保 `artemis-server/src/routing/mod.rs` 包含:

```rust
pub mod engine;
```

### Step 3: 运行测试

```bash
cargo test --package artemis-server routing::engine
```

### Step 4: 提交

```bash
git add artemis-server/src/routing/engine.rs
git commit -m "feat(server): 实现路由引擎统一入口

- RouteEngine: 统一管理路由策略
- 根据 RouteRule.strategy 选择对应策略
- 支持空分组的降级处理

Generated with [Claude Code](https://claude.com/claude-code)
via [Happy](https://happy.engineering)

Co-Authored-By: Claude <noreply@anthropic.com>
Co-Authored-By: Happy <yesreply@happy.engineering>"
```

---

**由于完整计划非常长 (预计 2000+ 行),我将创建一个概要版本,包含所有任务的标题和关键步骤。完整的详细步骤可以在执行时逐个生成。**

继续到 Task 7-21 的概要...

---

## Task 7-11: 业务逻辑层实现 (概要)

**Task 7**: 实现 GroupManager (分组管理)
- 文件: `artemis-management/src/group.rs` (重写)
- 功能: 分组 CRUD、分组标签、分组实例、分组操作

**Task 8**: 实现 RouteManager (路由规则管理)
- 文件: `artemis-management/src/route.rs` (重写)
- 功能: 路由规则 CRUD、路由规则分组关联、规则发布

**Task 9-11**: 单元测试
- GroupManager 测试: 50+ 测试用例
- RouteManager 测试: 30+ 测试用例

---

## Task 12-13: 发现服务集成

**Task 12**: 实现 GroupRoutingFilter
- 文件: `artemis-server/src/discovery/filter.rs` (新增)
- 功能: 从 RouteManager 获取规则,应用 RouteEngine

**Task 13**: 集成到 DiscoveryServiceImpl
- 修改: `artemis-server/src/discovery/mod.rs`
- 添加过滤器到过滤器链末尾

---

## Task 14-19: HTTP API 层 (27 个端点)

**Task 14**: 分组 API (5 个)
**Task 15**: 路由规则 API (6 个)
**Task 16**: 路由规则分组 API (6 个)
**Task 17**: 分组标签 API (5 个)
**Task 18**: 分组实例 API (3 个)
**Task 19**: 服务实例 API (2 个)

---

## Task 20-21: 集成测试和文档

**Task 20**: 集成测试脚本
- 文件: `test-group-routing.sh`
- 13 步完整测试流程

**Task 21**: 文档更新
- README.md
- docs/GROUP_ROUTING.md

---

## 验收标准

- [ ] 所有 27 个 API 实现并测试通过
- [ ] 两种路由策略算法正确
- [ ] 集成测试通过 (分布误差 < 5%)
- [ ] 代码零警告 (cargo clippy)
- [ ] 文档完整

---

## 预计时间

- Task 1-6 (数据模型 + 路由引擎): 1.5 天
- Task 7-11 (业务逻辑): 2 天
- Task 12-13 (发现服务集成): 0.5 天
- Task 14-19 (HTTP API): 2 天
- Task 20-21 (测试和文档): 1 天
- **总计**: 7 天

---

**下一步**: 选择执行方式 (Subagent-Driven 或 Parallel Session)
