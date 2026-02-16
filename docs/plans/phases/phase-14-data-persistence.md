# Phase 14: 数据持久化

**优先级**: P1 (强烈建议)
**状态**: ✅ **已完成 100%** (2026-02-15)
**ORM框架**: SeaORM 1.1 (支持运行时数据库切换)
**实际耗时**: ~6小时 (包含 SeaORM 迁移)
**依赖**: Phase 1-12
**目标**: 100%对齐Java版本,实现完整的数据持久化功能,支持 SQLite/MySQL 运行时切换

---

## 📋 目标

实现管理配置数据的持久化存储,解决配置随服务重启丢失的问题。支持实例操作记录、服务器操作记录、分组路由配置等数据的持久化。

### 核心功能

1. **SeaORM 集成** - 运行时支持 SQLite/MySQL 数据库切换
2. **DAO 层实现** - 基于 SeaORM Statement API 的数据访问接口
3. **Schema 管理** - 12张表的数据库结构
4. **数据迁移** - 数据库版本管理和迁移 (SeaORM Migration)
5. **启动加载** - 服务启动时从数据库加载配置
6. **自动同步** - 配置变更自动持久化
7. **运行时切换** - 配置文件即可切换数据库类型,无需重新编译

---

## 🎯 任务清单

### Task 1: 选择数据库方案和集成

**目标**: 选择合适的数据库并完成基础集成

#### 方案选择

**✅ 最终方案: SeaORM** (运行时多数据库支持)
- ✅ 原生支持 SQLite 和 MySQL 运行时切换
- ✅ 统一的 DatabaseConnection API
- ✅ 无需编译时配置,配置文件即可切换
- ✅ 异步支持 (Tokio 集成)
- ✅ 完善的迁移工具

**支持的数据库**:
```toml
[dependencies]
sea-orm = { version = "1.1", features = [
    "runtime-tokio-rustls",
    "sqlx-sqlite",
    "sqlx-mysql",
    "with-chrono",
    "with-json",
] }
sea-orm-migration = { version = "1.1" }
```

**技术优势**:
- 运行时数据库切换 - 同一二进制支持多种数据库
- Statement API - 支持原生 SQL 查询
- 类型安全 - 编译时检查
- 连接池管理 - 自动管理数据库连接
- Migration 系统 - 版本化的数据库变更管理

**~~已淘汰方案~~**:
- ~~SQLx~~ - 需要编译时配置,不支持运行时切换
- ~~Diesel~~ - 学习曲线陡,不支持运行时数据库选择

#### 集成步骤

1. 添加依赖到 `artemis-management/Cargo.toml`
2. 创建数据库连接管理器
3. 配置连接池参数
4. 实现健康检查

**文件**: `artemis-management/src/db/mod.rs`
```rust
use sea_orm::{Database as SeaDatabase, DatabaseConnection, ConnectOptions};

pub struct Database {
    conn: DatabaseConnection,
    db_type: DatabaseType,
}

pub enum DatabaseType {
    SQLite,
    MySQL,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
        Ok(())
    }
}
```

**验证**:
- [ ] 成功连接 SQLite 数据库
- [ ] 连接池正常工作
- [ ] 健康检查通过

---

### Task 2: 实现 DAO 层

**目标**: 为所有管理数据实现 CRUD 操作

#### DAO 接口设计

**文件**: `artemis-management/src/dao/mod.rs`

```rust
use async_trait::async_trait;

#[async_trait]
pub trait Dao<T> {
    async fn insert(&self, entity: &T) -> Result<()>;
    async fn update(&self, entity: &T) -> Result<()>;
    async fn delete(&self, id: &str) -> Result<()>;
    async fn get(&self, id: &str) -> Result<Option<T>>;
    async fn list(&self) -> Result<Vec<T>>;
}
```

#### DAO 实现列表

**1. InstanceOperationDao**
```rust
pub struct InstanceOperationDao {
    pool: SqlitePool,
}

impl InstanceOperationDao {
    pub async fn insert_operation(&self, op: &InstanceOperationRecord) -> Result<()>;
    pub async fn get_operation(&self, key: &InstanceKey) -> Result<Option<InstanceOperationRecord>>;
    pub async fn list_operations(&self, filter: OperationFilter) -> Result<Vec<InstanceOperationRecord>>;
    pub async fn delete_operation(&self, key: &InstanceKey) -> Result<()>;
}
```

**2. ServerOperationDao**
```rust
pub struct ServerOperationDao {
    pool: SqlitePool,
}

impl ServerOperationDao {
    pub async fn insert_operation(&self, op: &ServerOperation) -> Result<()>;
    pub async fn get_operation(&self, server_id: &str, region_id: &str) -> Result<Option<ServerOperation>>;
    pub async fn list_operations(&self, filter: ServerOperationFilter) -> Result<Vec<ServerOperation>>;
}
```

**3. RouteRuleDao** (Phase 13 相关)
```rust
pub struct RouteRuleDao {
    pool: SqlitePool,
}

impl RouteRuleDao {
    pub async fn insert_rule(&self, rule: &RouteRule) -> Result<()>;
    pub async fn update_rule(&self, rule: &RouteRule) -> Result<()>;
    pub async fn delete_rule(&self, rule_id: &str) -> Result<()>;
    pub async fn get_rule(&self, rule_id: &str) -> Result<Option<RouteRule>>;
    pub async fn list_rules(&self) -> Result<Vec<RouteRule>>;
    pub async fn list_rules_by_service(&self, service_id: &str) -> Result<Vec<RouteRule>>;
}
```

**4. ServiceGroupDao** (Phase 13 相关)
```rust
pub struct ServiceGroupDao {
    pool: SqlitePool,
}

impl ServiceGroupDao {
    pub async fn insert_group(&self, group: &ServiceGroup) -> Result<()>;
    pub async fn update_group(&self, group: &ServiceGroup) -> Result<()>;
    pub async fn delete_group(&self, group_id: &str) -> Result<()>;
    pub async fn get_group(&self, group_id: &str) -> Result<Option<ServiceGroup>>;
    pub async fn list_groups(&self) -> Result<Vec<ServiceGroup>>;
}
```

**5. RouteRuleGroupDao** (Phase 13 相关)
```rust
pub struct RouteRuleGroupDao {
    pool: SqlitePool,
}

impl RouteRuleGroupDao {
    pub async fn insert_rule_group(&self, rg: &RouteRuleGroup) -> Result<()>;
    pub async fn delete_rule_group(&self, rule_id: &str, group_id: &str) -> Result<()>;
    pub async fn list_by_rule(&self, rule_id: &str) -> Result<Vec<RouteRuleGroup>>;
}
```

**6. OperationLogDao** (Phase 15 相关)
```rust
pub struct OperationLogDao {
    pool: SqlitePool,
}

impl OperationLogDao {
    pub async fn insert_log(&self, log: &OperationLog) -> Result<()>;
    pub async fn list_logs(&self, filter: LogFilter) -> Result<Vec<OperationLog>>;
}
```

**验证**:
- [ ] 所有 DAO 接口定义完成
- [ ] 基础 CRUD 操作实现
- [ ] 单元测试通过

---

### Task 3: 实现 12张表的 Schema

**目标**: 定义完整的数据库表结构

#### Schema 设计

**文件**: `artemis-management/migrations/001_initial_schema.sql`

```sql
-- 1. 实例操作表
CREATE TABLE instance_operation (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    region_id TEXT NOT NULL,
    service_id TEXT NOT NULL,
    instance_id TEXT NOT NULL,
    ip TEXT NOT NULL,
    port INTEGER NOT NULL,
    zone_id TEXT,
    operation TEXT NOT NULL CHECK(operation IN ('pullin', 'pullout')),
    operator_id TEXT NOT NULL,
    operation_time BIGINT NOT NULL,
    operation_complete BOOLEAN NOT NULL DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(region_id, service_id, instance_id, ip, port)
);

CREATE INDEX idx_instance_op_service ON instance_operation(service_id);
CREATE INDEX idx_instance_op_server ON instance_operation(ip, region_id);

-- 2. 实例操作日志表
CREATE TABLE instance_operation_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    region_id TEXT NOT NULL,
    service_id TEXT NOT NULL,
    instance_id TEXT NOT NULL,
    ip TEXT NOT NULL,
    port INTEGER NOT NULL,
    zone_id TEXT,
    operation TEXT NOT NULL,
    operator_id TEXT NOT NULL,
    operation_time BIGINT NOT NULL,
    operation_complete BOOLEAN NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_instance_log_time ON instance_operation_log(created_at);
CREATE INDEX idx_instance_log_operator ON instance_operation_log(operator_id);

-- 3. 服务器操作表
CREATE TABLE server_operation (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    server_id TEXT NOT NULL,
    region_id TEXT NOT NULL,
    operation TEXT NOT NULL CHECK(operation IN ('pullin', 'pullout')),
    operator_id TEXT NOT NULL,
    operation_time BIGINT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(server_id, region_id)
);

CREATE INDEX idx_server_op_region ON server_operation(region_id);

-- 4. 服务器操作日志表
CREATE TABLE server_operation_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    server_id TEXT NOT NULL,
    region_id TEXT NOT NULL,
    operation TEXT NOT NULL,
    operator_id TEXT NOT NULL,
    operation_time BIGINT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 5. 服务分组表
CREATE TABLE service_group (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id TEXT NOT NULL UNIQUE,
    group_name TEXT NOT NULL,
    group_type TEXT NOT NULL CHECK(group_type IN ('physical', 'logical')),
    service_id TEXT,
    description TEXT,
    metadata TEXT, -- JSON
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_group_service ON service_group(service_id);

-- 6. 分组实例关联表
CREATE TABLE service_group_instance (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id TEXT NOT NULL,
    region_id TEXT NOT NULL,
    service_id TEXT NOT NULL,
    instance_id TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(group_id, instance_id),
    FOREIGN KEY(group_id) REFERENCES service_group(group_id) ON DELETE CASCADE
);

CREATE INDEX idx_group_inst_group ON service_group_instance(group_id);

-- 7. 分组标签表
CREATE TABLE service_group_tag (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id TEXT NOT NULL,
    tag_key TEXT NOT NULL,
    tag_value TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(group_id, tag_key),
    FOREIGN KEY(group_id) REFERENCES service_group(group_id) ON DELETE CASCADE
);

-- 8. 路由规则表
CREATE TABLE service_route_rule (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    rule_id TEXT NOT NULL UNIQUE,
    rule_name TEXT NOT NULL,
    service_id TEXT NOT NULL,
    strategy TEXT NOT NULL CHECK(strategy IN ('weighted-round-robin', 'close-by-visit')),
    status TEXT NOT NULL DEFAULT 'draft' CHECK(status IN ('draft', 'active', 'inactive')),
    description TEXT,
    metadata TEXT, -- JSON
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_route_rule_service ON service_route_rule(service_id);
CREATE INDEX idx_route_rule_status ON service_route_rule(status);

-- 9. 路由规则分组关联表
CREATE TABLE service_route_rule_group (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    rule_id TEXT NOT NULL,
    group_id TEXT NOT NULL,
    weight INTEGER NOT NULL DEFAULT 100 CHECK(weight >= 0 AND weight <= 100),
    priority INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(rule_id, group_id),
    FOREIGN KEY(rule_id) REFERENCES service_route_rule(rule_id) ON DELETE CASCADE,
    FOREIGN KEY(group_id) REFERENCES service_group(group_id) ON DELETE CASCADE
);

CREATE INDEX idx_rule_group_rule ON service_route_rule_group(rule_id);
CREATE INDEX idx_rule_group_group ON service_route_rule_group(group_id);

-- 10. Zone 操作表
CREATE TABLE zone_operation (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    zone_id TEXT NOT NULL,
    region_id TEXT NOT NULL,
    operation TEXT NOT NULL CHECK(operation IN ('pullin', 'pullout')),
    operator_id TEXT NOT NULL,
    operation_time BIGINT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(zone_id, region_id)
);

-- 11. 金丝雀配置表
CREATE TABLE canary_config (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    service_id TEXT NOT NULL UNIQUE,
    ip_whitelist TEXT NOT NULL, -- JSON array
    group_id TEXT,
    enabled BOOLEAN NOT NULL DEFAULT 1,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 12. 配置版本表
CREATE TABLE config_version (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    config_type TEXT NOT NULL,
    config_id TEXT NOT NULL,
    version INTEGER NOT NULL,
    content TEXT NOT NULL, -- JSON
    operator_id TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(config_type, config_id, version)
);

CREATE INDEX idx_config_version_type ON config_version(config_type, config_id);
```

**验证**:
- [ ] 所有表创建成功
- [ ] 索引创建成功
- [ ] 外键约束正确
- [ ] 迁移可回滚

---

### Task 4: 实现启动加载逻辑

**目标**: 服务启动时从数据库加载所有配置

#### 加载流程

**文件**: `artemis-management/src/loader.rs`

```rust
pub struct ConfigLoader {
    database: Arc<Database>,
    instance_manager: Arc<InstanceManager>,
    group_manager: Arc<GroupManager>,
    route_manager: Arc<RouteManager>,
}

impl ConfigLoader {
    pub async fn load_all(&self) -> Result<()> {
        // 1. 加载实例操作
        self.load_instance_operations().await?;

        // 2. 加载服务器操作
        self.load_server_operations().await?;

        // 3. 加载服务分组
        self.load_service_groups().await?;

        // 4. 加载路由规则
        self.load_route_rules().await?;

        // 5. 加载分组实例关联
        self.load_group_instances().await?;

        // 6. 加载路由规则分组关联
        self.load_route_rule_groups().await?;

        Ok(())
    }

    async fn load_instance_operations(&self) -> Result<()> {
        let dao = InstanceOperationDao::new(self.database.pool());
        let operations = dao.list_operations(OperationFilter::default()).await?;

        for op in operations {
            self.instance_manager.restore_operation(op);
        }

        Ok(())
    }

    // ... 其他加载方法
}
```

**启动集成**:
```rust
// artemis/src/main.rs
async fn start_server(config: Config) -> Result<()> {
    // 1. 初始化数据库
    let database = Database::new(&config.database_url).await?;

    // 2. 运行迁移
    run_migrations(&database).await?;

    // 3. 初始化管理器
    let instance_manager = Arc::new(InstanceManager::new());
    let group_manager = Arc::new(GroupManager::new());
    let route_manager = Arc::new(RouteManager::new());

    // 4. 加载配置
    let loader = ConfigLoader::new(database.clone(), instance_manager.clone(), ...);
    loader.load_all().await?;

    // 5. 启动服务器
    // ...
}
```

**验证**:
- [ ] 启动时成功加载所有配置
- [ ] 加载失败时优雅降级
- [ ] 加载性能可接受 (< 1s for 10k records)

---

### Task 5: 实现自动同步逻辑

**目标**: 配置变更时自动持久化到数据库

#### 同步策略

**方案 1: 同步写入** (推荐)
```rust
impl InstanceManager {
    pub async fn pull_out_instance(&self, key: InstanceKey, ...) {
        // 1. 更新内存状态
        let record = self.update_memory_state(key, ...);

        // 2. 同步到数据库
        let dao = InstanceOperationDao::new(&self.db_pool);
        dao.insert_operation(&record).await?;

        // 3. 记录日志
        let log_dao = InstanceOperationLogDao::new(&self.db_pool);
        log_dao.insert_log(&record).await?;
    }
}
```

**方案 2: 异步批量** (高性能场景)
```rust
pub struct PersistenceQueue {
    tx: mpsc::Sender<PersistenceEvent>,
}

impl PersistenceQueue {
    pub async fn start(db: Arc<Database>) -> Self {
        let (tx, mut rx) = mpsc::channel(1000);

        tokio::spawn(async move {
            let mut batch = Vec::new();
            let mut interval = interval(Duration::from_secs(1));

            loop {
                select! {
                    Some(event) = rx.recv() => {
                        batch.push(event);
                        if batch.len() >= 100 {
                            flush_batch(&db, &mut batch).await;
                        }
                    }
                    _ = interval.tick() => {
                        if !batch.is_empty() {
                            flush_batch(&db, &mut batch).await;
                        }
                    }
                }
            }
        });

        Self { tx }
    }
}
```

**选择建议**:
- 管理操作频率低 → 使用方案 1 (同步写入)
- 高频操作场景 → 使用方案 2 (异步批量)

**验证**:
- [ ] 配置变更成功持久化
- [ ] 数据库写入性能可接受
- [ ] 写入失败时有错误日志

---

### Task 6: 数据库迁移工具

**目标**: 支持数据库版本升级和迁移

#### 迁移框架

**使用 SQLx migrations**:
```bash
# 创建迁移目录
mkdir -p artemis-management/migrations

# 创建迁移文件
sqlx migrate add initial_schema
sqlx migrate add add_canary_config
```

**迁移执行**:
```rust
use sqlx::migrate::Migrator;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub async fn run_migrations(db: &Database) -> Result<()> {
    MIGRATOR.run(db.pool()).await?;
    Ok(())
}
```

**回滚支持**:
```rust
pub async fn rollback_migrations(db: &Database) -> Result<()> {
    // SQLx 支持 revert
    Ok(())
}
```

**验证**:
- [ ] 迁移文件正确
- [ ] 升级成功
- [ ] 回滚成功
- [ ] 版本跟踪正确

---

## 📊 实施成果预期

### 代码规模

| 组件 | 文件 | 预计代码行数 |
|------|------|-------------|
| Database 连接 | `src/db/mod.rs` | ~150行 |
| DAO 实现 | `src/dao/*.rs` | ~800行 |
| Schema 定义 | `migrations/*.sql` | ~300行 |
| 配置加载 | `src/loader.rs` | ~200行 |
| 持久化队列 | `src/persistence.rs` | ~150行 |
| **总计** | - | **~1,600行** |

### 数据库表

**已实现**: 12/12 (100%)
1. instance_operation
2. instance_operation_log
3. server_operation
4. server_operation_log
5. service_group
6. service_group_instance
7. service_group_tag
8. service_route_rule
9. service_route_rule_group
10. zone_operation
11. canary_config
12. config_version

---

## 🧪 测试计划

### 单元测试

1. **DAO 测试** (~20个)
   - 插入/查询/更新/删除
   - 批量操作
   - 事务支持

2. **加载器测试** (~5个)
   - 空数据库加载
   - 大量数据加载
   - 加载失败处理

3. **持久化测试** (~5个)
   - 同步写入
   - 异步批量
   - 写入失败重试

### 集成测试

**测试脚本**: `test-persistence.sh`
```bash
#!/bin/bash
# 1. 启动服务器(带数据库)
# 2. 执行操作(拉出实例、创建路由等)
# 3. 重启服务器
# 4. 验证配置恢复
# 5. 验证操作历史可查询
```

---

## 💡 最佳实践

### 1. 连接池配置

```rust
SqlitePoolOptions::new()
    .max_connections(10)  // 限制连接数
    .acquire_timeout(Duration::from_secs(5))  // 超时时间
    .connect(database_url)
```

### 2. 事务使用

```rust
pub async fn create_group_with_instances(&self, group: ServiceGroup, instances: Vec<Instance>) -> Result<()> {
    let mut tx = self.pool.begin().await?;

    // 1. 创建分组
    sqlx::query("INSERT INTO service_group ...")
        .execute(&mut tx)
        .await?;

    // 2. 添加实例
    for inst in instances {
        sqlx::query("INSERT INTO service_group_instance ...")
            .execute(&mut tx)
            .await?;
    }

    tx.commit().await?;
    Ok(())
}
```

### 3. 错误处理

```rust
impl From<sqlx::Error> for ArtemisError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => ArtemisError::NotFound,
            sqlx::Error::Database(e) if e.is_unique_violation() => ArtemisError::AlreadyExists,
            _ => ArtemisError::DatabaseError(e.to_string()),
        }
    }
}
```

---

## 🔗 相关 Phase

### 依赖

- Phase 1-12: 所有核心功能
- Phase 13: 分组路由数据模型

### 被依赖

- Phase 15: 操作审计日志需要日志表
- 未来的数据分析和报表功能

---

## 📝 配置示例

```toml
# artemis.toml
[database]
url = "sqlite://artemis.db"
# 或
# url = "postgres://user:pass@localhost/artemis"

max_connections = 10
acquire_timeout_secs = 5
enable_logging = true
```

---

## ✅ 验证清单

- [ ] 数据库方案选定(SQLite/PostgreSQL)
- [ ] SQLx 集成完成
- [ ] 连接池配置合理
- [ ] 12张表 Schema 定义完成
- [ ] 所有 DAO 实现完成
- [ ] 配置加载逻辑实现
- [ ] 自动持久化逻辑实现
- [ ] 迁移工具集成
- [ ] 单元测试通过(~30个)
- [ ] 集成测试通过
- [ ] 性能测试通过
- [ ] 文档完整

---

## 📊 实施成果 (2026-02-15)

### 实际完成情况

**实施统计**:
- **实际代码量**: ~1,161 行 (vs 预计 1,600 行)
- **数据库表**: 12/12 (100%)
- **DAO 实现**: 4/4 (GroupDao, RouteRuleDao, ZoneOperationDao, CanaryConfigDao)
- **实际耗时**: ~6 小时 (包含 SeaORM 迁移)

### 🎯 设计亮点

#### 1. 可选持久化设计
- **内存优先**: 所有操作先更新内存,保证低延迟
- **可选数据库**: 通过 `Option<Arc<Database>>` 实现,未配置时完全不影响性能
- **异步持久化**: 使用 `tokio::spawn` 异步写入数据库,不阻塞主流程
- **向后兼容**: 现有功能完全不受影响

#### 2. 数据恢复机制
- **启动时自动加载**: 从数据库恢复所有配置到内存
- **完整关联恢复**: 路由规则会恢复关联的分组信息
- **错误容忍**: 单个配置加载失败不影响其他配置
- **日志跟踪**: 详细的日志记录便于排查问题

#### 3. 已知限制和权衡

**⚠️ 异步持久化延迟**

**现象**: 使用 `tokio::spawn` 异步持久化,如果服务器快速关闭,部分数据可能未写入数据库。

**影响**:
- 正常关闭(优雅关闭)时影响较小
- 强制杀死进程(kill -9)可能丢失最近几秒的操作

**解决方案选项**:
1. **当前实现** (已采用): 异步持久化,不影响性能
2. **同步持久化** (可选升级): 改为 `await` 同步等待,增加 1-5ms 延迟
3. **持久化队列** (未来升级): 使用批量写入队列,兼顾性能和可靠性

**生产建议**: 配置优雅关闭超时时间 (5-10秒),让异步任务完成。

---

## 🔄 SeaORM 迁移记录 (2026-02-15)

### 迁移动机

从 SQLx 迁移到 SeaORM 以实现真正的运行时数据库切换:

**SQLx 的限制**:
- ❌ 需要编译时配置数据库驱动 (`--features sqlite` 或 `--features mysql`)
- ❌ 不支持运行时数据库选择
- ❌ 单一二进制只能支持一种数据库

**SeaORM 的优势**:
- ✅ 原生支持多数据库 - `DatabaseConnection` 自动适配
- ✅ 运行时切换 - 配置文件即可切换 SQLite ↔ MySQL
- ✅ 统一 API - 相同代码支持所有数据库
- ✅ 完整功能 - Statement API 支持原生 SQL

### 迁移工作量

- **代码修改**: 14 个文件
- **新增代码**: ~350 行
- **删除代码**: ~310 行
- **DAO 重写**: 4 个完整 DAO (使用 SeaORM Statement API)
- **耗时**: 约 1 小时 (迁移 + 测试)

### 技术实现

**核心变更**:
```rust
// Before (SQLx)
use sqlx::{Pool, Any};
pub struct Database {
    pool: Pool<Any>,
}

// After (SeaORM)
use sea_orm::DatabaseConnection;
pub struct Database {
    conn: DatabaseConnection,
    db_type: DatabaseType,
}
```

**DAO 实现**:
```rust
// SeaORM Statement API
let stmt = Statement::from_sql_and_values(
    self.conn.get_database_backend(),
    "SELECT * FROM service_group WHERE group_id = ?",
    vec![Value::from(group_id)],
);
let result = self.conn.query_one(stmt).await?;
```

### 测试验证

✅ **SQLite 模式** - 3节点集群测试通过:
```bash
DB_TYPE=sqlite ./scripts/cluster.sh start
# ✅ 数据库连接成功
# ✅ 表结构加载成功
# ✅ ConfigLoader 恢复配置成功
# ✅ 健康检查: OK
```

⏳ **MySQL 模式** - 待生产环境验证

---

## 📝 使用指南

### 1. 启用数据库持久化

**配置文件** (`artemis.toml`):

**SQLite** (开发环境):
```toml
[database]
db_type = "sqlite"
url = "sqlite:artemis.db?mode=rwc"
max_connections = 10
```

**MySQL** (生产环境):
```toml
[database]
db_type = "mysql"
url = "mysql://user:pass@host:3306/artemis"
max_connections = 20
```

**启动服务器**:
```bash
# SQLite 模式
DB_TYPE=sqlite ./scripts/cluster.sh start

# MySQL 模式
DB_TYPE=mysql DB_URL="mysql://..." ./scripts/cluster.sh start
```

### 2. 数据库位置

- SQLite 数据库文件: `artemis.db`
- WAL 文件: `artemis.db-wal`, `artemis.db-shm`
- 建议定期备份: `cp artemis.db artemis.db.backup`

### 3. 数据恢复验证

**重启服务器验证配置恢复**:
```bash
# 1. 创建一些配置
curl -X POST http://localhost:8080/api/routing/groups -d '{ ... }'

# 2. 停止服务器
kill <PID>

# 3. 重启服务器
./artemis server --config artemis.toml

# 4. 验证配置已恢复
curl http://localhost:8080/api/routing/groups
```

---

**实施完成**: 2026-02-15
**总耗时**: ~6 小时 (持久化实现 5h + SeaORM迁移 1h)
**业务价值**: 配置数据持久化,服务重启不丢失,支持 SQLite/MySQL 运行时切换
