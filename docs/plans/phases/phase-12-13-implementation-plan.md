# 服务分组路由和实例管理功能实施计划

**日期**: 2026-02-14
**目标**: 实现 Java 版本的分组路由和实例管理功能
**工作量估算**: 3-5 天

---

## 1. 功能概述

### 1.1 服务分组路由 (Group Routing)

**核心功能**:
- 服务分组定义 (Group CRUD)
- 路由规则管理 (RouteRule CRUD)
- 路由规则与分组关联 (带权重)
- 权重路由策略实现
- 规则发布/生效机制

**使用场景**:
- 流量分组 (灰度环境、测试环境)
- 权重路由 (A/B 测试、灰度发布)
- 就近访问 (跨 Zone 路由优化)

### 1.2 实例管理 (Instance Management)

**核心功能**:
- 实例拉入/拉出 (Pull-in/Pull-out)
- 服务器批量拉入/拉出
- 操作状态查询
- 在发现服务中应用过滤

**使用场景**:
- 运维手动下线实例 (不影响注册状态)
- 服务器维护 (批量下线)
- 临时流量控制

---

## 2. 数据模型设计

### 2.1 核心模型 (artemis-core/src/model/)

#### Group (服务分组)
```rust
pub struct Group {
    pub group_id: Option<i64>,        // 自动生成
    pub service_id: String,            // 所属服务
    pub region_id: String,
    pub zone_id: String,
    pub name: String,                  // 分组名称
    pub app_id: Option<String>,
    pub description: Option<String>,
    pub status: GroupStatus,           // active/inactive
    pub metadata: Option<HashMap<String, String>>,
}

pub fn group_key(&self) -> String {
    format!("{}:{}:{}:{}", service_id, region_id, zone_id, name)
}
```

#### RouteRule (路由规则)
```rust
pub struct RouteRule {
    pub route_rule_id: Option<i64>,
    pub route_id: String,              // service_id + name
    pub service_id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: RouteRuleStatus,       // active/inactive
    pub strategy: RouteStrategy,       // 路由策略
    pub groups: Vec<ServiceGroup>,     // 关联的分组
}
```

#### RouteRuleGroup (规则分组关联)
```rust
pub struct RouteRuleGroup {
    pub route_rule_group_id: Option<i64>,
    pub route_rule_id: i64,
    pub group_id: i64,
    pub weight: Option<u32>,           // 已发布权重
    pub unreleased_weight: Option<u32>, // 未发布权重
}
```

#### InstanceOperation (实例操作)
```rust
pub enum InstanceOperation {
    PullIn,   // 拉入
    PullOut,  // 拉出
}

pub struct InstanceOperationRecord {
    pub instance_key: InstanceKey,
    pub operation: InstanceOperation,
    pub operation_complete: bool,
    pub operator_id: String,
    pub token: Option<String>,
}
```

### 2.2 存储设计 (内存存储,不需要 MySQL)

**artemis-management 模块**:
```rust
// 分组存储: group_key -> Group
groups: Arc<DashMap<String, Group>>

// 路由规则存储: route_id -> RouteRule
route_rules: Arc<DashMap<String, RouteRule>>

// 规则分组关联: route_rule_id -> Vec<RouteRuleGroup>
route_rule_groups: Arc<DashMap<i64, Vec<RouteRuleGroup>>>

// 实例操作存储: instance_key -> InstanceOperation
instance_operations: Arc<DashMap<String, InstanceOperation>>

// 服务器操作存储: server_id:region_id -> ServerOperation
server_operations: Arc<DashMap<String, ServerOperation>>
```

---

## 3. 实施步骤

### Phase 1: 数据模型和核心逻辑 (1 天)

#### 3.1 更新 artemis-core
- ✅ 添加 Group, RouteRule, RouteRuleGroup 模型
- ✅ 添加 InstanceOperation, ServerOperation 模型
- ✅ 添加 Request/Response 类型

#### 3.2 实现 GroupManager (artemis-management/src/group.rs)
```rust
impl GroupManager {
    // CRUD 操作
    pub fn create_group(&self, group: Group) -> Result<i64>;
    pub fn get_group(&self, group_key: &str) -> Option<Group>;
    pub fn update_group(&self, group: Group) -> Result<()>;
    pub fn delete_group(&self, group_key: &str) -> Result<()>;
    pub fn list_groups(&self, service_id: Option<&str>) -> Vec<Group>;

    // 分组实例管理
    pub fn add_instance_to_group(&self, group_key: &str, instance_id: &str);
    pub fn remove_instance_from_group(&self, group_key: &str, instance_id: &str);
    pub fn get_group_instances(&self, group_key: &str) -> Vec<String>;
}
```

#### 3.3 实现 RouteManager (artemis-management/src/route.rs)
```rust
impl RouteManager {
    // 路由规则 CRUD
    pub fn create_route_rule(&self, rule: RouteRule) -> Result<i64>;
    pub fn get_route_rule(&self, route_id: &str) -> Option<RouteRule>;
    pub fn update_route_rule(&self, rule: RouteRule) -> Result<()>;
    pub fn delete_route_rule(&self, route_id: &str) -> Result<()>;
    pub fn list_route_rules(&self, service_id: Option<&str>) -> Vec<RouteRule>;

    // 规则分组关联
    pub fn add_group_to_rule(&self, rule_id: i64, group_id: i64, weight: u32);
    pub fn update_group_weight(&self, rule_id: i64, group_id: i64, weight: u32);
    pub fn release_weights(&self, rule_id: i64);  // 发布未发布的权重
    pub fn remove_group_from_rule(&self, rule_id: i64, group_id: i64);
}
```

#### 3.4 实现 InstanceManager (artemis-management/src/instance.rs)
```rust
impl InstanceManager {
    // 实例操作
    pub fn pull_out_instance(&self, key: &InstanceKey, operator_id: String) -> Result<()>;
    pub fn pull_in_instance(&self, key: &InstanceKey, operator_id: String) -> Result<()>;
    pub fn is_instance_down(&self, key: &InstanceKey) -> bool;
    pub fn get_instance_operations(&self, key: &InstanceKey) -> Vec<InstanceOperation>;

    // 服务器操作
    pub fn pull_out_server(&self, server_id: &str, region_id: &str, operator_id: String) -> Result<()>;
    pub fn pull_in_server(&self, server_id: &str, region_id: &str, operator_id: String) -> Result<()>;
    pub fn is_server_down(&self, server_id: &str, region_id: &str) -> bool;
}
```

### Phase 2: 路由策略实现 (1 天)

#### 3.5 实现路由策略 (artemis-management/src/routing/)
```rust
pub trait RouteStrategy {
    fn select_instances(&self, rule: &RouteRule, all_instances: &[Instance]) -> Vec<Instance>;
}

pub struct WeightedRoundRobinStrategy;
impl RouteStrategy for WeightedRoundRobinStrategy {
    fn select_instances(&self, rule: &RouteRule, all_instances: &[Instance]) -> Vec<Instance> {
        // 1. 按分组过滤实例
        // 2. 根据权重计算实例分布
        // 3. 轮询选择实例
    }
}

pub struct CloseByVisitStrategy;
impl RouteStrategy for CloseByVisitStrategy {
    fn select_instances(&self, rule: &RouteRule, all_instances: &[Instance]) -> Vec<Instance> {
        // 1. 按 Zone 优先级排序
        // 2. 优先返回同 Zone 实例
        // 3. 按权重分配跨 Zone 实例
    }
}
```

### Phase 3: HTTP API 实现 (1 天)

#### 3.6 分组路由 API (artemis-web/src/handlers/management/)

**分组管理**:
- `POST /api/management/group/create-group.json` - 创建分组
- `POST /api/management/group/update-group.json` - 更新分组
- `POST /api/management/group/delete-group.json` - 删除分组
- `POST /api/management/group/get-groups.json` - 查询分组
- `POST /api/management/group/get-all-groups.json` - 查询所有分组

**路由规则管理**:
- `POST /api/management/route/create-route-rule.json` - 创建路由规则
- `POST /api/management/route/update-route-rule.json` - 更新路由规则
- `POST /api/management/route/delete-route-rule.json` - 删除路由规则
- `POST /api/management/route/get-route-rules.json` - 查询路由规则

**规则分组关联**:
- `POST /api/management/route/add-group-to-rule.json` - 添加分组到规则
- `POST /api/management/route/update-group-weight.json` - 更新分组权重
- `POST /api/management/route/release-weights.json` - 发布权重
- `POST /api/management/route/remove-group-from-rule.json` - 移除分组

#### 3.7 实例管理 API

**实例操作**:
- `POST /api/management/instance/operate-instance.json` - 操作实例 (pull-in/pull-out)
- `POST /api/management/instance/get-instance-operations.json` - 查询实例操作
- `POST /api/management/instance/is-instance-down.json` - 查询实例是否被拉出

**服务器操作**:
- `POST /api/management/server/operate-server.json` - 操作服务器
- `POST /api/management/server/is-server-down.json` - 查询服务器是否被拉出

### Phase 4: 集成和过滤 (1 天)

#### 3.8 集成到发现服务 (artemis-server/src/discovery/)

**ManagementDiscoveryFilter**:
```rust
impl DiscoveryFilter for ManagementDiscoveryFilter {
    fn filter(&self, service: &mut Service, _config: &DiscoveryConfig) {
        // 1. 移除被 pull-out 的实例
        self.remove_down_instances(&mut service.instances);

        // 2. 应用路由规则
        if let Some(route_rules) = &service.route_rules {
            self.apply_route_rules(service, route_rules);
        }
    }
}

fn remove_down_instances(&self, instances: &mut Vec<Instance>) {
    instances.retain(|inst| {
        !self.instance_manager.is_instance_down(&inst.to_key())
    });
}

fn apply_route_rules(&self, service: &mut Service, rules: &[RouteRule]) {
    for rule in rules {
        if rule.status == RouteRuleStatus::Active {
            let strategy = RouteStrategyFactory::create(&rule.strategy);
            service.instances = strategy.select_instances(rule, &service.instances);
        }
    }
}
```

#### 3.9 服务器级别过滤
```rust
fn remove_down_servers(&self, instances: &mut Vec<Instance>) {
    instances.retain(|inst| {
        let server_id = &inst.ip;
        let region_id = &inst.region_id;
        !self.instance_manager.is_server_down(server_id, region_id)
    });
}
```

### Phase 5: 测试和文档 (1 天)

#### 3.10 单元测试
- GroupManager 测试 (CRUD, 分组实例管理)
- RouteManager 测试 (规则管理, 权重发布)
- InstanceManager 测试 (拉入拉出, 状态查询)
- 路由策略测试 (权重分配, 就近访问)

#### 3.11 集成测试
- 端到端分组路由测试
- 实例拉出后发现服务过滤测试
- 服务器拉出后批量过滤测试
- 权重路由策略验证

#### 3.12 文档更新
- API 文档 (Swagger/README)
- 使用示例
- 实施总结

---

## 4. API 示例

### 4.1 分组路由示例

```bash
# 1. 创建灰度分组
curl -X POST http://localhost:8080/api/management/group/create-group.json \
  -H "Content-Type: application/json" \
  -d '{
    "service_id": "my-service",
    "region_id": "us-east",
    "zone_id": "zone-1",
    "name": "canary",
    "status": "active"
  }'

# 2. 创建路由规则
curl -X POST http://localhost:8080/api/management/route/create-route-rule.json \
  -H "Content-Type: application/json" \
  -d '{
    "service_id": "my-service",
    "name": "canary-rule",
    "strategy": "weighted-round-robin",
    "status": "active"
  }'

# 3. 添加分组到规则 (10% 流量到灰度)
curl -X POST http://localhost:8080/api/management/route/add-group-to-rule.json \
  -H "Content-Type: application/json" \
  -d '{
    "route_rule_id": 1,
    "group_id": 1,
    "unreleased_weight": 10
  }'

# 4. 发布权重
curl -X POST http://localhost:8080/api/management/route/release-weights.json \
  -H "Content-Type: application/json" \
  -d '{
    "route_rule_id": 1
  }'
```

### 4.2 实例管理示例

```bash
# 1. 拉出实例 (下线)
curl -X POST http://localhost:8080/api/management/instance/operate-instance.json \
  -H "Content-Type: application/json" \
  -d '{
    "instance_key": {
      "service_id": "my-service",
      "instance_id": "inst-1",
      "region_id": "us-east"
    },
    "operation": "pullout",
    "operator_id": "admin",
    "operation_complete": true
  }'

# 2. 查询实例是否被拉出
curl -X POST http://localhost:8080/api/management/instance/is-instance-down.json \
  -H "Content-Type: application/json" \
  -d '{
    "instance_key": {
      "service_id": "my-service",
      "instance_id": "inst-1",
      "region_id": "us-east"
    }
  }'

# 3. 拉入实例 (恢复)
curl -X POST http://localhost:8080/api/management/instance/operate-instance.json \
  -H "Content-Type: application/json" \
  -d '{
    "instance_key": {
      "service_id": "my-service",
      "instance_id": "inst-1",
      "region_id": "us-east"
    },
    "operation": "pullin",
    "operator_id": "admin",
    "operation_complete": true
  }'
```

---

## 5. 技术要点

### 5.1 权重路由算法

**加权轮询 (Weighted Round Robin)**:
```rust
// 权重: group1=70, group2=20, group3=10
// 总权重: 100
// 生成实例列表: [group1]*70 + [group2]*20 + [group3]*10
// 轮询索引递增,模取总数
```

**就近访问 (Close By Visit)**:
```rust
// 1. 按 Zone 分组: same_zone, cross_zone
// 2. 优先返回 same_zone 实例
// 3. 如果 same_zone 为空,返回 cross_zone (按权重)
```

### 5.2 原子操作保证

```rust
// 使用 DashMap 保证并发安全
groups.insert(key, value);  // 原子操作
route_rules.entry(key).and_modify(|v| *v = new_value);  // 原子更新
```

### 5.3 过滤器链

```rust
// 过滤器执行顺序:
// 1. ManagementDiscoveryFilter (移除 pull-out 实例)
// 2. GroupDiscoveryFilter (应用分组路由)
// 3. 自定义过滤器
```

---

## 6. 简化方案 (MVP)

**如需快速交付,可以简化为**:

### 6.1 MVP 功能
- ✅ 实例拉入/拉出 (核心运维功能)
- ✅ 实例拉出后在发现服务中过滤
- ✅ 基本的分组路由 (单分组,无权重)
- ⚠️ 暂不实现复杂的权重路由策略
- ⚠️ 暂不实现规则发布机制

### 6.2 后续迭代
- Phase 2: 权重路由策略
- Phase 3: 规则发布/回滚
- Phase 4: 操作审计日志

---

**预计完成时间**: 3-5 天 (按优先级逐步实现)
**优先级**: P2 (可选,但对高级场景有价值)
