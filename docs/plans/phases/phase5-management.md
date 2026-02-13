# 阶段5: artemis-management实现

> **For Claude:** 管理功能和持久化层。参考Java实现: `artemis-java/artemis-management/`

**目标:** 实现管理功能、数据库持久化、分组和路由规则

**预计任务数:** 4个Task

---

## Task 5.1: 创建MySQL Schema

**Files:**
- Create: `artemis-management/migrations/001_initial_schema.sql`

**Step 1: 创建migrations目录**

```bash
mkdir -p artemis-management/migrations
```

**Step 2: 创建数据库Schema**

```sql
-- artemis-management/migrations/001_initial_schema.sql

-- 实例操作记录表
CREATE TABLE IF NOT EXISTS instance_operation (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    region_id VARCHAR(64) NOT NULL,
    zone_id VARCHAR(64) NOT NULL,
    service_id VARCHAR(128) NOT NULL,
    instance_id VARCHAR(128) NOT NULL,
    operation VARCHAR(32) NOT NULL,
    operation_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    complete BOOLEAN DEFAULT FALSE,
    metadata JSON,
    INDEX idx_service (service_id),
    INDEX idx_instance (instance_id),
    INDEX idx_operation_time (operation_time)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- 服务分组表
CREATE TABLE IF NOT EXISTS service_group (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    region_id VARCHAR(64) NOT NULL,
    zone_id VARCHAR(64) NOT NULL,
    service_id VARCHAR(128) NOT NULL,
    group_key VARCHAR(128) NOT NULL,
    weight INT DEFAULT 100,
    instance_ids TEXT,
    metadata JSON,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    UNIQUE KEY uk_group (service_id, group_key)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- 路由规则表
CREATE TABLE IF NOT EXISTS route_rule (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    region_id VARCHAR(64) NOT NULL,
    zone_id VARCHAR(64) NOT NULL,
    service_id VARCHAR(128) NOT NULL,
    route_id VARCHAR(128) NOT NULL,
    strategy VARCHAR(64) NOT NULL,
    groups JSON NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    UNIQUE KEY uk_route (service_id, route_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
```

**Step 3: 提交**

```bash
git add artemis-management/migrations/
git commit -m "feat(management): create MySQL schema

- Add instance_operation table
- Add service_group table
- Add route_rule table
- Add appropriate indexes

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 5.2: 实现DAO层

**Files:**
- Create: `artemis-management/src/dao/mod.rs`
- Create: `artemis-management/src/dao/instance_operation.rs`

**Step 1: 创建dao模块**

```rust
// artemis-management/src/dao/mod.rs
pub mod instance_operation;
pub mod group;
pub mod route;

pub use instance_operation::InstanceOperationDao;
```

**Step 2: 实现InstanceOperationDao**

```rust
// artemis-management/src/dao/instance_operation.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{MySqlPool, Row};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceOperation {
    pub id: i64,
    pub region_id: String,
    pub zone_id: String,
    pub service_id: String,
    pub instance_id: String,
    pub operation: String,
    pub operation_time: DateTime<Utc>,
    pub complete: bool,
    pub metadata: Option<serde_json::Value>,
}

pub struct InstanceOperationDao {
    pool: MySqlPool,
}

impl InstanceOperationDao {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, op: &InstanceOperation) -> anyhow::Result<i64> {
        let result = sqlx::query(
            "INSERT INTO instance_operation
             (region_id, zone_id, service_id, instance_id, operation, operation_time, complete, metadata)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&op.region_id)
        .bind(&op.zone_id)
        .bind(&op.service_id)
        .bind(&op.instance_id)
        .bind(&op.operation)
        .bind(&op.operation_time)
        .bind(op.complete)
        .bind(&op.metadata)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_id() as i64)
    }

    pub async fn get_by_id(&self, id: i64) -> anyhow::Result<Option<InstanceOperation>> {
        let row = sqlx::query(
            "SELECT id, region_id, zone_id, service_id, instance_id, operation,
             operation_time, complete, metadata FROM instance_operation WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| InstanceOperation {
            id: r.get("id"),
            region_id: r.get("region_id"),
            zone_id: r.get("zone_id"),
            service_id: r.get("service_id"),
            instance_id: r.get("instance_id"),
            operation: r.get("operation"),
            operation_time: r.get("operation_time"),
            complete: r.get("complete"),
            metadata: r.get("metadata"),
        }))
    }

    pub async fn mark_complete(&self, id: i64) -> anyhow::Result<()> {
        sqlx::query("UPDATE instance_operation SET complete = TRUE WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_pending_operations(&self) -> anyhow::Result<Vec<InstanceOperation>> {
        let rows = sqlx::query(
            "SELECT id, region_id, zone_id, service_id, instance_id, operation,
             operation_time, complete, metadata FROM instance_operation
             WHERE complete = FALSE ORDER BY operation_time ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .iter()
            .map(|r| InstanceOperation {
                id: r.get("id"),
                region_id: r.get("region_id"),
                zone_id: r.get("zone_id"),
                service_id: r.get("service_id"),
                instance_id: r.get("instance_id"),
                operation: r.get("operation"),
                operation_time: r.get("operation_time"),
                complete: r.get("complete"),
                metadata: r.get("metadata"),
            })
            .collect())
    }
}
```

**Step 3: 创建group和route DAO占位**

```rust
// artemis-management/src/dao/group.rs
//! 服务分组DAO
//! TODO: 实现GroupDao
```

```rust
// artemis-management/src/dao/route.rs
//! 路由规则DAO
//! TODO: 实现RouteDao
```

**Step 4: 提交**

```bash
git add artemis-management/src/dao/
git commit -m "feat(management): implement InstanceOperationDao

- Add InstanceOperation model
- Implement CRUD operations with SQLx
- Support pending operations query
- Add group and route DAO placeholders

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 5.3: 实现实例操作管理器

**Files:**
- Create: `artemis-management/src/instance/mod.rs`
- Create: `artemis-management/src/instance/operations.rs`

**Step 1: 创建instance模块**

```rust
// artemis-management/src/instance/mod.rs
pub mod operations;

pub use operations::InstanceOperationsManager;
```

**Step 2: 实现InstanceOperationsManager**

```rust
// artemis-management/src/instance/operations.rs
use crate::dao::InstanceOperationDao;
use chrono::Utc;
use sqlx::MySqlPool;
use tracing::info;

pub struct InstanceOperationsManager {
    dao: InstanceOperationDao,
}

impl InstanceOperationsManager {
    pub fn new(pool: MySqlPool) -> Self {
        Self {
            dao: InstanceOperationDao::new(pool),
        }
    }

    pub async fn record_operation(
        &self,
        region_id: &str,
        zone_id: &str,
        service_id: &str,
        instance_id: &str,
        operation: &str,
    ) -> anyhow::Result<i64> {
        info!(
            "Recording operation: {} for instance {}/{}",
            operation, service_id, instance_id
        );

        let op = crate::dao::instance_operation::InstanceOperation {
            id: 0,
            region_id: region_id.to_string(),
            zone_id: zone_id.to_string(),
            service_id: service_id.to_string(),
            instance_id: instance_id.to_string(),
            operation: operation.to_string(),
            operation_time: Utc::now(),
            complete: false,
            metadata: None,
        };

        self.dao.create(&op).await
    }

    pub async fn complete_operation(&self, id: i64) -> anyhow::Result<()> {
        self.dao.mark_complete(id).await
    }

    pub async fn get_pending_operations(
        &self,
    ) -> anyhow::Result<Vec<crate::dao::instance_operation::InstanceOperation>> {
        self.dao.get_pending_operations().await
    }
}
```

**Step 3: 更新lib.rs**

```rust
// artemis-management/src/lib.rs
//! Artemis Management - 管理功能和持久化

pub mod api;
pub mod dao;
pub mod group;
pub mod instance;
pub mod route;

pub use instance::InstanceOperationsManager;
```

**Step 4: 提交**

```bash
git add artemis-management/src/instance/ artemis-management/src/lib.rs
git commit -m "feat(management): implement InstanceOperationsManager

- Record instance operations to database
- Mark operations as complete
- Query pending operations

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 5.4: 实现ManagementDiscoveryFilter

**Files:**
- Create: `artemis-management/src/filter/mod.rs`
- Create: `artemis-management/src/filter/management_filter.rs`
- Update: `artemis-management/src/lib.rs`

**Step 1: 实现ManagementDiscoveryFilter**

```rust
// artemis-management/src/filter/mod.rs
pub mod management_filter;

pub use management_filter::ManagementDiscoveryFilter;
```

```rust
// artemis-management/src/filter/management_filter.rs
use crate::dao::InstanceOperationDao;
use artemis_core::model::{DiscoveryConfig, Service};
use async_trait::async_trait;
use std::collections::HashSet;
use std::sync::Arc;

/// 管理发现过滤器 - 应用实例操作规则（拉入/拉出）
pub struct ManagementDiscoveryFilter {
    dao: Arc<InstanceOperationDao>,
}

impl ManagementDiscoveryFilter {
    pub fn new(dao: Arc<InstanceOperationDao>) -> Self {
        Self { dao }
    }

    /// 获取需要拉出的实例ID集合
    async fn get_pulled_out_instances(&self, service_id: &str) -> anyhow::Result<HashSet<String>> {
        let operations = self.dao.get_pending_operations().await?;

        let pulled_out: HashSet<String> = operations
            .iter()
            .filter(|op| {
                op.service_id == service_id && op.operation == "pull-out" && !op.complete
            })
            .map(|op| op.instance_id.clone())
            .collect();

        Ok(pulled_out)
    }

    /// 获取需要拉入的实例ID集合
    async fn get_pulled_in_instances(&self, service_id: &str) -> anyhow::Result<HashSet<String>> {
        let operations = self.dao.get_pending_operations().await?;

        let pulled_in: HashSet<String> = operations
            .iter()
            .filter(|op| {
                op.service_id == service_id && op.operation == "pull-in" && !op.complete
            })
            .map(|op| op.instance_id.clone())
            .collect();

        Ok(pulled_in)
    }
}

#[async_trait]
impl artemis_server::discovery::DiscoveryFilter for ManagementDiscoveryFilter {
    async fn filter(&self, service: &mut Service, _config: &DiscoveryConfig) -> anyhow::Result<()> {
        // 获取拉出的实例
        let pulled_out = self.get_pulled_out_instances(&service.service_id).await?;

        // 过滤掉被拉出的实例
        if !pulled_out.is_empty() {
            service.instances.retain(|inst| !pulled_out.contains(&inst.instance_id));
        }

        // 注意：拉入操作通常不需要在这里处理，
        // 因为拉入的实例应该已经在注册表中
        // 这里只是示例，可以根据需求添加逻辑

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dao::instance_operation::InstanceOperation;
    use artemis_core::model::{Instance, InstanceStatus};
    use chrono::Utc;
    use sqlx::MySqlPool;

    async fn create_test_service() -> Service {
        Service {
            service_id: "test-service".to_string(),
            metadata: None,
            instances: vec![
                Instance {
                    region_id: "test".to_string(),
                    zone_id: "zone".to_string(),
                    group_id: None,
                    service_id: "test-service".to_string(),
                    instance_id: "inst-1".to_string(),
                    machine_name: None,
                    ip: "127.0.0.1".to_string(),
                    port: 8080,
                    protocol: None,
                    url: "http://127.0.0.1:8080".to_string(),
                    health_check_url: None,
                    status: InstanceStatus::Up,
                    metadata: None,
                },
                Instance {
                    region_id: "test".to_string(),
                    zone_id: "zone".to_string(),
                    group_id: None,
                    service_id: "test-service".to_string(),
                    instance_id: "inst-2".to_string(),
                    machine_name: None,
                    ip: "127.0.0.2".to_string(),
                    port: 8080,
                    protocol: None,
                    url: "http://127.0.0.2:8080".to_string(),
                    health_check_url: None,
                    status: InstanceStatus::Up,
                    metadata: None,
                },
            ],
            logic_instances: None,
            route_rules: None,
        }
    }

    // 注意：实际测试需要数据库连接，这里只是示例
    // 在集成测试中会有完整的测试
}
```

**Step 2: 更新lib.rs导出**

```rust
// artemis-management/src/lib.rs
//! Artemis Management - 管理功能和持久化

pub mod api;
pub mod dao;
pub mod filter;
pub mod group;
pub mod instance;
pub mod route;

pub use filter::ManagementDiscoveryFilter;
pub use instance::InstanceOperationsManager;
```

**Step 3: 添加artemis-server依赖**

更新 `artemis-management/Cargo.toml`:

```toml
[dependencies]
artemis-core = { path = "../artemis-core" }
artemis-server = { path = "../artemis-server" }  # 新增
sqlx = { workspace = true }
# ... 其他依赖 ...
```

**Step 4: 创建其他占位模块**

```rust
// artemis-management/src/api/mod.rs
//! 管理API接口
//! TODO: 实现管理API handlers
```

```rust
// artemis-management/src/group/mod.rs
//! 服务分组管理
//! TODO: 实现GroupManager
```

```rust
// artemis-management/src/route/mod.rs
//! 路由规则管理
//! TODO: 实现RouteManager
```

**Step 5: 验证编译**

```bash
cargo check -p artemis-management
```

Expected: 编译成功

**Step 6: 提交**

```bash
git add artemis-management/
git commit -m "feat(management): implement ManagementDiscoveryFilter

- Add ManagementDiscoveryFilter for pull-in/pull-out operations
- Filter instances based on operation status
- Integrate with DiscoveryFilter trait
- Add placeholder modules for API, group, route

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## 阶段5完成标准

- ✅ MySQL Schema定义
- ✅ InstanceOperationDao实现
- ✅ InstanceOperationsManager实现
- ✅ ManagementDiscoveryFilter完整实现
- ✅ Group和Route模块占位
- ✅ `cargo check -p artemis-management` 通过
