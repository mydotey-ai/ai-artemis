# Phase 19-22: 功能差距修复计划

**创建日期**: 2026-02-15
**优先级**: P2 (高级功能补充)
**预估总工时**: 13 天
**目标**: 补齐 Rust 版本与 Java 版本的功能差距,提升 API 完整度从 66% 到 85%

---

## 📋 差距概览

根据 `rust-java-complete-comparison.md` 分析,Rust 版本与 Java 版本存在以下主要差距:

| 差距项 | 影响程度 | 缺失 API 数 | 优先级 | 预估工时 |
|-------|---------|-----------|--------|---------|
| **分组实例绑定** | ⚠️ 中 | 6 个 | **P1 高** | 5 天 |
| **Discovery Lookup** | ⚠️ 中 | 1 个 | **P1 高** | 2 天 |
| **状态查询 API** | 🟡 低 | 12 个 | **P2 中** | 4 天 |
| **GET 查询参数** | 🟡 低 | 6 个 | **P3 低** | 2 天 |

**总计**: 25 个缺失 API,13 天工时

---

## Phase 19: 分组实例绑定 API

### 优先级: P1 高优先级 (建议优先实施)

### 问题描述

**Java 版本功能**:
- 支持手动添加实例到分组 (`insert-group-instances.json`)
- 支持从分组移除实例 (`delete-group-instances.json`)
- 支持批量添加服务实例 (`insert-service-instances.json`)

**Rust 当前实现**:
- 仅支持只读查询 `GET /api/routing/groups/{key}/instances`
- 分组实例关系自动从注册实例中筛选 (基于 metadata)
- **无法手动控制实例分组关系**

**影响评估**:
- ❌ 无法手动调整分组成员
- ❌ 无法临时将特定实例加入特定分组
- ❌ 无法实现灵活的分组管理策略

### 实施计划

#### Task 19.1: 数据模型扩展 (1 天)

**目标**: 定义分组实例绑定的数据结构

**文件**: `artemis-core/src/model/group.rs`

**实现**:

```rust
/// 分组实例绑定关系 (手动绑定)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GroupInstanceBinding {
    /// 绑定 ID (自动生成)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// 分组 ID
    pub group_id: i64,
    /// 实例 ID
    pub instance_id: String,
    /// Region ID
    pub region_id: String,
    /// Zone ID
    pub zone_id: String,
    /// Service ID
    pub service_id: String,
    /// 绑定类型 (manual | auto)
    pub binding_type: BindingType,
    /// 创建时间 (Unix timestamp)
    pub created_at: i64,
    /// 操作人 ID
    pub operator_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BindingType {
    /// 手动绑定 (通过 API 添加)
    Manual,
    /// 自动绑定 (通过 metadata 匹配)
    Auto,
}
```

**数据库 Schema** (添加到 `migrations/001_initial_schema.sql`):

```sql
CREATE TABLE IF NOT EXISTS group_instance_bindings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id INTEGER NOT NULL,
    instance_id TEXT NOT NULL,
    region_id TEXT NOT NULL,
    zone_id TEXT NOT NULL,
    service_id TEXT NOT NULL,
    binding_type TEXT NOT NULL CHECK(binding_type IN ('manual', 'auto')),
    created_at INTEGER NOT NULL,
    operator_id TEXT NOT NULL,
    UNIQUE(group_id, instance_id, region_id, zone_id)
);

CREATE INDEX idx_group_bindings ON group_instance_bindings(group_id);
CREATE INDEX idx_instance_bindings ON group_instance_bindings(instance_id, region_id, zone_id);
```

---

#### Task 19.2: DAO 层实现 (1 天)

**目标**: 实现分组实例绑定的持久化操作

**文件**: `artemis-management/src/dao/group_instance_dao.rs`

**实现**:

```rust
use artemis_core::model::group::GroupInstanceBinding;
use sea_orm::{Database, DbConn, EntityTrait, QueryFilter, ColumnTrait};
use anyhow::Result;

pub struct GroupInstanceDao {
    db: DbConn,
}

impl GroupInstanceDao {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }

    /// 插入分组实例绑定
    pub async fn insert(&self, binding: &GroupInstanceBinding) -> Result<i64> {
        let stmt = r#"
            INSERT INTO group_instance_bindings
            (group_id, instance_id, region_id, zone_id, service_id, binding_type, created_at, operator_id)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        let result = sea_orm::Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            stmt,
            vec![
                binding.group_id.into(),
                binding.instance_id.clone().into(),
                binding.region_id.clone().into(),
                binding.zone_id.clone().into(),
                binding.service_id.clone().into(),
                binding.binding_type.to_string().into(),
                binding.created_at.into(),
                binding.operator_id.clone().into(),
            ],
        );

        self.db.execute(result).await?;
        Ok(self.db.last_insert_id())
    }

    /// 删除分组实例绑定
    pub async fn delete(&self, group_id: i64, instance_id: &str, region_id: &str, zone_id: &str) -> Result<bool> {
        let stmt = r#"
            DELETE FROM group_instance_bindings
            WHERE group_id = ? AND instance_id = ? AND region_id = ? AND zone_id = ?
        "#;

        let result = sea_orm::Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            stmt,
            vec![
                group_id.into(),
                instance_id.into(),
                region_id.into(),
                zone_id.into(),
            ],
        );

        let exec_result = self.db.execute(result).await?;
        Ok(exec_result.rows_affected() > 0)
    }

    /// 获取分组的所有绑定实例
    pub async fn get_by_group(&self, group_id: i64) -> Result<Vec<GroupInstanceBinding>> {
        let stmt = r#"
            SELECT * FROM group_instance_bindings WHERE group_id = ?
        "#;

        // SeaORM 查询实现...
        todo!()
    }

    /// 获取实例的所有分组绑定
    pub async fn get_by_instance(&self, instance_id: &str, region_id: &str, zone_id: &str) -> Result<Vec<GroupInstanceBinding>> {
        // 实现查询逻辑...
        todo!()
    }

    /// 批量插入绑定 (用于服务实例批量添加)
    pub async fn batch_insert(&self, bindings: &[GroupInstanceBinding]) -> Result<usize> {
        // 实现批量插入逻辑...
        todo!()
    }
}
```

---

#### Task 19.3: GroupManager 扩展 (1 天)

**目标**: 在 GroupManager 中添加实例绑定管理功能

**文件**: `artemis-management/src/group.rs`

**实现**:

```rust
use artemis_core::model::group::GroupInstanceBinding;
use crate::dao::group_instance_dao::GroupInstanceDao;

impl GroupManager {
    /// 添加实例到分组 (手动绑定)
    pub async fn add_instance_to_group(
        &self,
        group_id: i64,
        instance_id: String,
        region_id: String,
        zone_id: String,
        service_id: String,
        operator_id: String,
    ) -> Result<()> {
        // 1. 验证分组存在
        let group = self.get_group_by_id(group_id)
            .ok_or_else(|| anyhow::anyhow!("Group not found"))?;

        // 2. 验证实例存在 (从 CacheManager 查询)
        // ...

        // 3. 创建绑定记录
        let binding = GroupInstanceBinding {
            id: None,
            group_id,
            instance_id,
            region_id,
            zone_id,
            service_id,
            binding_type: BindingType::Manual,
            created_at: chrono::Utc::now().timestamp(),
            operator_id,
        };

        // 4. 持久化
        if let Some(dao) = &self.dao {
            dao.insert(&binding).await?;
        }

        // 5. 记录审计日志
        self.audit_manager.log_operation(
            "add_instance_to_group",
            &format!("group:{}, instance:{}", group_id, binding.instance_id),
            &binding.operator_id,
        ).await;

        Ok(())
    }

    /// 从分组移除实例
    pub async fn remove_instance_from_group(
        &self,
        group_id: i64,
        instance_id: String,
        region_id: String,
        zone_id: String,
        operator_id: String,
    ) -> Result<()> {
        // 1. 删除绑定记录
        if let Some(dao) = &self.dao {
            dao.delete(group_id, &instance_id, &region_id, &zone_id).await?;
        }

        // 2. 记录审计日志
        self.audit_manager.log_operation(
            "remove_instance_from_group",
            &format!("group:{}, instance:{}", group_id, instance_id),
            &operator_id,
        ).await;

        Ok(())
    }

    /// 获取分组的实例列表 (包含手动绑定 + 自动匹配)
    pub async fn get_group_instances(&self, group_id: i64) -> Result<Vec<Instance>> {
        // 1. 获取手动绑定的实例
        let manual_bindings = if let Some(dao) = &self.dao {
            dao.get_by_group(group_id).await?
        } else {
            vec![]
        };

        // 2. 获取自动匹配的实例 (基于 metadata)
        // ...

        // 3. 合并并去重
        // 优先级: 手动绑定 > 自动匹配

        Ok(instances)
    }

    /// 批量添加服务实例到分组
    pub async fn batch_add_service_instances(
        &self,
        service_id: &str,
        region_id: &str,
        zone_id: &str,
        group_id: i64,
        operator_id: String,
    ) -> Result<usize> {
        // 1. 获取服务的所有实例
        let instances = self.cache_manager.get_service_instances(service_id, region_id, zone_id).await?;

        // 2. 批量创建绑定记录
        let bindings: Vec<GroupInstanceBinding> = instances.iter().map(|inst| {
            GroupInstanceBinding {
                id: None,
                group_id,
                instance_id: inst.instance_id.clone(),
                region_id: inst.region_id.clone(),
                zone_id: inst.zone_id.clone(),
                service_id: inst.service_id.clone(),
                binding_type: BindingType::Manual,
                created_at: chrono::Utc::now().timestamp(),
                operator_id: operator_id.clone(),
            }
        }).collect();

        // 3. 批量插入
        let count = if let Some(dao) = &self.dao {
            dao.batch_insert(&bindings).await?
        } else {
            0
        };

        Ok(count)
    }
}
```

---

#### Task 19.4: API 端点实现 (2 天)

**目标**: 实现 3 个 HTTP API 端点

**文件**: `artemis-web/src/api/routing.rs`

**实现**:

**1. 添加实例到分组**

```rust
/// POST /api/routing/groups/{group_key}/instances
#[derive(Debug, Deserialize)]
pub struct AddInstanceToGroupRequest {
    pub instance_id: String,
    pub region_id: String,
    pub zone_id: String,
    pub service_id: String,
    pub operator_id: String,
}

pub async fn add_instance_to_group(
    State(state): State<AppState>,
    Path(group_key): Path<String>,
    Json(req): Json<AddInstanceToGroupRequest>,
) -> impl IntoResponse {
    // 1. 解析 group_key 获取 group_id
    let group_id = state.group_manager
        .get_group_by_key(&group_key)
        .await
        .map(|g| g.group_id.unwrap())
        .ok_or_else(|| anyhow::anyhow!("Group not found"))?;

    // 2. 调用 GroupManager 添加实例
    state.group_manager
        .add_instance_to_group(
            group_id,
            req.instance_id,
            req.region_id,
            req.zone_id,
            req.service_id,
            req.operator_id,
        )
        .await?;

    Json(ApiResponse::success("Instance added to group"))
}
```

**2. 从分组移除实例**

```rust
/// DELETE /api/routing/groups/{group_key}/instances/{instance_id}
#[derive(Debug, Deserialize)]
pub struct RemoveInstanceQuery {
    pub region_id: String,
    pub zone_id: String,
    pub operator_id: String,
}

pub async fn remove_instance_from_group(
    State(state): State<AppState>,
    Path((group_key, instance_id)): Path<(String, String)>,
    Query(query): Query<RemoveInstanceQuery>,
) -> impl IntoResponse {
    // 解析 group_id
    let group_id = state.group_manager
        .get_group_by_key(&group_key)
        .await
        .map(|g| g.group_id.unwrap())
        .ok_or_else(|| anyhow::anyhow!("Group not found"))?;

    // 移除实例
    state.group_manager
        .remove_instance_from_group(
            group_id,
            instance_id,
            query.region_id,
            query.zone_id,
            query.operator_id,
        )
        .await?;

    Json(ApiResponse::success("Instance removed from group"))
}
```

**3. 批量添加服务实例**

```rust
/// POST /api/routing/services/{service_id}/instances
#[derive(Debug, Deserialize)]
pub struct BatchAddServiceInstancesRequest {
    pub region_id: String,
    pub zone_id: String,
    pub group_id: i64,
    pub operator_id: String,
}

pub async fn batch_add_service_instances(
    State(state): State<AppState>,
    Path(service_id): Path<String>,
    Json(req): Json<BatchAddServiceInstancesRequest>,
) -> impl IntoResponse {
    let count = state.group_manager
        .batch_add_service_instances(
            &service_id,
            &req.region_id,
            &req.zone_id,
            req.group_id,
            req.operator_id,
        )
        .await?;

    Json(ApiResponse::success(format!("Added {} instances to group", count)))
}
```

**路由注册** (在 `artemis-web/src/server.rs`):

```rust
.route("/api/routing/groups/:group_key/instances", post(add_instance_to_group))
.route("/api/routing/groups/:group_key/instances/:instance_id", delete(remove_instance_from_group))
.route("/api/routing/services/:service_id/instances", post(batch_add_service_instances))
```

---

#### Task 19.5: 单元测试 + 集成测试 (1 天)

**文件**: `artemis-management/src/group.rs` (单元测试)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_instance_to_group() {
        let manager = GroupManager::new_for_test();

        // 创建分组
        let group_id = manager.create_group(...).await.unwrap();

        // 添加实例
        let result = manager.add_instance_to_group(
            group_id,
            "inst-1".to_string(),
            "us-east".to_string(),
            "zone-1".to_string(),
            "my-service".to_string(),
            "admin".to_string(),
        ).await;

        assert!(result.is_ok());

        // 验证实例已添加
        let instances = manager.get_group_instances(group_id).await.unwrap();
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0].instance_id, "inst-1");
    }

    #[tokio::test]
    async fn test_remove_instance_from_group() {
        // 实现移除测试...
    }

    #[tokio::test]
    async fn test_batch_add_service_instances() {
        // 实现批量添加测试...
    }
}
```

**集成测试脚本**: `scripts/test-group-instance-binding.sh`

```bash
#!/bin/bash
set -e

BASE_URL="http://localhost:8080"

echo "=== Phase 19: 分组实例绑定集成测试 ==="

# 1. 创建分组
echo "1. 创建分组..."
GROUP_RESPONSE=$(curl -s -X POST "$BASE_URL/api/routing/groups" \
  -H "Content-Type: application/json" \
  -d '{
    "service_id": "test-service",
    "region_id": "us-east",
    "zone_id": "zone-1",
    "name": "test-group",
    "group_type": "physical"
  }')

GROUP_KEY=$(echo $GROUP_RESPONSE | jq -r '.data.group_key')
echo "✓ 分组已创建: $GROUP_KEY"

# 2. 注册测试实例
echo "2. 注册测试实例..."
curl -s -X POST "$BASE_URL/api/registry/register.json" \
  -H "Content-Type: application/json" \
  -d '{
    "instances": [{
      "region_id": "us-east",
      "zone_id": "zone-1",
      "service_id": "test-service",
      "instance_id": "inst-1",
      "ip": "10.0.0.1",
      "port": 8080,
      "url": "http://10.0.0.1:8080",
      "status": "up"
    }]
  }'

echo "✓ 实例已注册"

# 3. 添加实例到分组
echo "3. 添加实例到分组..."
curl -s -X POST "$BASE_URL/api/routing/groups/$GROUP_KEY/instances" \
  -H "Content-Type: application/json" \
  -d '{
    "instance_id": "inst-1",
    "region_id": "us-east",
    "zone_id": "zone-1",
    "service_id": "test-service",
    "operator_id": "admin"
  }'

echo "✓ 实例已添加到分组"

# 4. 查询分组实例
echo "4. 查询分组实例..."
INSTANCES=$(curl -s "$BASE_URL/api/routing/groups/$GROUP_KEY/instances")
INSTANCE_COUNT=$(echo $INSTANCES | jq '.data | length')

if [ "$INSTANCE_COUNT" -eq 1 ]; then
  echo "✓ 分组实例数量正确: $INSTANCE_COUNT"
else
  echo "✗ 分组实例数量错误: expected 1, got $INSTANCE_COUNT"
  exit 1
fi

# 5. 移除实例
echo "5. 移除实例..."
curl -s -X DELETE "$BASE_URL/api/routing/groups/$GROUP_KEY/instances/inst-1?region_id=us-east&zone_id=zone-1&operator_id=admin"

echo "✓ 实例已移除"

# 6. 验证移除
INSTANCES=$(curl -s "$BASE_URL/api/routing/groups/$GROUP_KEY/instances")
INSTANCE_COUNT=$(echo $INSTANCES | jq '.data | length')

if [ "$INSTANCE_COUNT" -eq 0 ]; then
  echo "✓ 实例已成功移除"
else
  echo "✗ 实例移除失败"
  exit 1
fi

echo ""
echo "=== ✅ Phase 19 测试通过 ==="
```

---

### Phase 19 总结

**预估工时**: 5 天
**交付成果**:
- ✅ 3 个新 API 端点
- ✅ GroupInstanceBinding 数据模型
- ✅ GroupInstanceDao 持久化
- ✅ GroupManager 功能扩展
- ✅ 单元测试 + 集成测试脚本

**API 完整度提升**: 64/101 → 67/101 = **66.3%**

---

## Phase 20: Discovery Lookup API

### 优先级: P1 高优先级 (建议实施)

### 问题描述

**Java 版本功能**:
```java
POST /api/discovery/lookup.json
{
  "discoveryConfig": {
    "serviceId": "my-service",
    "regionId": "us-east"
  }
}
// 返回: 单个实例 (负载均衡选择)
```

**Rust 当前实现**:
- `POST /api/discovery/service.json` - 返回所有实例
- 客户端需自行实现负载均衡选择

**影响**: 客户端需额外实现负载均衡逻辑

---

### 实施计划

#### Task 20.1: 负载均衡策略实现 (1 天)

**文件**: `artemis-server/src/discovery/load_balancer.rs`

```rust
use artemis_core::model::instance::Instance;
use rand::Rng;
use std::sync::atomic::{AtomicUsize, Ordering};

/// 负载均衡策略
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LoadBalanceStrategy {
    /// 随机选择
    Random,
    /// 轮询
    RoundRobin,
    /// 加权轮询
    WeightedRoundRobin,
}

/// 负载均衡器
pub struct LoadBalancer {
    round_robin_counter: AtomicUsize,
}

impl LoadBalancer {
    pub fn new() -> Self {
        Self {
            round_robin_counter: AtomicUsize::new(0),
        }
    }

    /// 选择单个实例
    pub fn select(
        &self,
        instances: &[Instance],
        strategy: LoadBalanceStrategy,
    ) -> Option<Instance> {
        if instances.is_empty() {
            return None;
        }

        match strategy {
            LoadBalanceStrategy::Random => self.random_select(instances),
            LoadBalanceStrategy::RoundRobin => self.round_robin_select(instances),
            LoadBalanceStrategy::WeightedRoundRobin => self.weighted_select(instances),
        }
    }

    fn random_select(&self, instances: &[Instance]) -> Option<Instance> {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..instances.len());
        Some(instances[index].clone())
    }

    fn round_robin_select(&self, instances: &[Instance]) -> Option<Instance> {
        let index = self.round_robin_counter.fetch_add(1, Ordering::Relaxed) % instances.len();
        Some(instances[index].clone())
    }

    fn weighted_select(&self, instances: &[Instance]) -> Option<Instance> {
        // 基于实例 metadata 中的 weight 字段
        // 如果没有权重,默认为 1
        let total_weight: u32 = instances.iter()
            .map(|inst| {
                inst.metadata.as_ref()
                    .and_then(|m| m.get("weight"))
                    .and_then(|w| w.parse::<u32>().ok())
                    .unwrap_or(1)
            })
            .sum();

        if total_weight == 0 {
            return self.random_select(instances);
        }

        let mut rng = rand::thread_rng();
        let mut random_weight = rng.gen_range(0..total_weight);

        for inst in instances {
            let weight = inst.metadata.as_ref()
                .and_then(|m| m.get("weight"))
                .and_then(|w| w.parse::<u32>().ok())
                .unwrap_or(1);

            if random_weight < weight {
                return Some(inst.clone());
            }
            random_weight -= weight;
        }

        self.random_select(instances)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_select() {
        let lb = LoadBalancer::new();
        let instances = vec![
            create_test_instance("inst-1"),
            create_test_instance("inst-2"),
            create_test_instance("inst-3"),
        ];

        let selected = lb.select(&instances, LoadBalanceStrategy::Random);
        assert!(selected.is_some());
    }

    #[test]
    fn test_round_robin() {
        let lb = LoadBalancer::new();
        let instances = vec![
            create_test_instance("inst-1"),
            create_test_instance("inst-2"),
            create_test_instance("inst-3"),
        ];

        // 连续选择应该轮询
        let ids: Vec<String> = (0..6)
            .map(|_| lb.select(&instances, LoadBalanceStrategy::RoundRobin).unwrap().instance_id)
            .collect();

        assert_eq!(ids, vec!["inst-1", "inst-2", "inst-3", "inst-1", "inst-2", "inst-3"]);
    }
}
```

---

#### Task 20.2: Lookup API 实现 (1 天)

**文件**: `artemis-web/src/api/discovery.rs`

```rust
use artemis_server::discovery::load_balancer::{LoadBalancer, LoadBalanceStrategy};

#[derive(Debug, Deserialize)]
pub struct LookupRequest {
    pub discovery_config: DiscoveryConfig,
    #[serde(default = "default_load_balance_strategy")]
    pub strategy: LoadBalanceStrategy,
}

fn default_load_balance_strategy() -> LoadBalanceStrategy {
    LoadBalanceStrategy::Random
}

#[derive(Debug, Serialize)]
pub struct LookupResponse {
    pub instance: Option<Instance>,
}

/// POST /api/discovery/lookup.json
pub async fn lookup(
    State(state): State<AppState>,
    Json(req): Json<LookupRequest>,
) -> Json<LookupResponse> {
    // 1. 获取服务的所有实例
    let service_response = state.discovery_service
        .get_service(&req.discovery_config)
        .await
        .unwrap_or_default();

    // 2. 使用负载均衡器选择单个实例
    let selected_instance = state.load_balancer.select(
        &service_response.instances,
        req.strategy,
    );

    Json(LookupResponse {
        instance: selected_instance,
    })
}
```

**路由注册** (在 `artemis-web/src/server.rs`):

```rust
.route("/api/discovery/lookup.json", post(lookup))
```

**AppState 扩展**:

```rust
pub struct AppState {
    // ... 现有字段
    pub load_balancer: Arc<LoadBalancer>,
}
```

---

### Phase 20 总结

**预估工时**: 2 天
**交付成果**:
- ✅ 1 个新 API 端点 (`/api/discovery/lookup.json`)
- ✅ LoadBalancer 实现 (3 种策略)
- ✅ 单元测试

**API 完整度提升**: 67/101 → 68/101 = **67.3%**

---

## Phase 21: 状态查询 API (可选)

### 优先级: P2 中优先级 (可选实施)

**预估工时**: 4 天
**新增 API**: 4 个

详细设计见 `rust-java-complete-comparison.md` 第 7.2 节

---

## Phase 22: GET 查询参数支持 (可选)

### 优先级: P3 低优先级 (可选实施)

**预估工时**: 2 天
**新增 API**: 6 个 GET 端点

详细设计见 `rust-java-complete-comparison.md` 第 7.3 节

---

## 实施建议

### 推荐方案

**第一阶段** (高优先级 - 7 天):
1. ✅ **Phase 19**: 分组实例绑定 (5 天)
2. ✅ **Phase 20**: Discovery Lookup (2 天)

**第二阶段** (可选 - 6 天):
3. ⚠️ **Phase 21**: 状态查询 API (4 天)
4. ⚠️ **Phase 22**: GET 查询参数 (2 天)

### 预期收益

**完成 Phase 19-20 后**:
- API 完整度: 66% → **67%**
- 功能完整度: 95% → **98%**
- **分组管理灵活性大幅提升**
- **客户端使用更便捷**

**完成全部 Phase 19-22 后**:
- API 完整度: 66% → **78%**
- 功能完整度: 95% → **100%**
- **完全对齐 Java 核心 API**

---

## 附录: 测试计划

### 单元测试覆盖

- GroupInstanceDao (10 个测试)
- GroupManager (15 个测试)
- LoadBalancer (8 个测试)

### 集成测试覆盖

- Phase 19: 分组实例绑定集成测试 (6 步)
- Phase 20: Lookup API 集成测试 (3 步)

### 性能测试

- 分组实例查询性能 (< 10ms)
- Lookup API 性能 (< 1ms)
- 批量添加性能 (1000 实例 < 100ms)

---

**文档版本**: 1.0.0
**创建时间**: 2026-02-15
**下一次更新**: 实施完成后
