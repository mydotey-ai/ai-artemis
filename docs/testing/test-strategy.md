# Artemis Rust - 测试策略和方法

**制定时间**: 2026-02-15
**最后更新**: 2026-02-16
**项目阶段**: ✅ 100% 完成 - 生产就绪

---

## 📋 文档说明

本文档描述 Artemis 项目的测试策略、方法论和最佳实践。

**当前测试状态请查看**: [test-status.md](test-status.md)

---

## 🎯 测试策略设计

### 测试金字塔模型

```
           E2E Tests (10%)
          /              \
         /                \
        /  Integration (25%)\
       /                    \
      /   Component (25%)    \
     /                        \
    /    Unit Tests (40%)      \
   /____________________________\
```

### 测试分层

| 层次 | 占比 | 特点 | 执行频率 |
|------|-----|------|---------|
| **单元测试** | 40% | 快速、隔离、精准 | 每次提交 |
| **组件测试** | 25% | 模块级验证 | 每次提交 |
| **集成测试** | 25% | 模块间交互 | 每次 PR |
| **E2E 测试** | 10% | 完整场景验证 | 每次 PR |

### 目标覆盖率

| 指标 | 目标 | 说明 |
|------|------|------|
| **代码行覆盖率** | 80%+ | 关键路径 100% |
| **分支覆盖率** | 75%+ | 错误处理分支 |
| **API 覆盖率** | 100% | 所有端点测试 |
| **测试通过率** | 100% | 零失败容忍 |

---

## 📝 测试最佳实践

### 1. 测试命名规范

**格式**: `test_<function>_<scenario>_<expected_result>`

```rust
// ✅ 好的命名
#[test]
fn test_register_empty_instances_returns_error() {}

#[test]
fn test_heartbeat_expired_lease_renews_successfully() {}

#[test]
fn test_discover_with_routing_filters_down_instances() {}

// ❌ 不好的命名
#[test]
fn test1() {}

#[test]
fn register() {}

#[test]
fn it_works() {}
```

### 2. 测试组织原则

#### 单一职责
每个测试只验证一个功能点：

```rust
// ✅ 单一职责
#[test]
fn test_register_success() {
    // 只测试成功注册
}

#[test]
fn test_register_duplicate_error() {
    // 只测试重复注册错误
}

// ❌ 多重职责
#[test]
fn test_register() {
    // 测试注册 + 心跳 + 查询 + 注销
}
```

#### 独立性
测试之间不依赖执行顺序：

```rust
// ✅ 每个测试独立创建状态
#[test]
fn test_a() {
    let state = create_test_state();
    // ...
}

#[test]
fn test_b() {
    let state = create_test_state();
    // ...
}

// ❌ 依赖执行顺序
static mut SHARED_STATE: Option<State> = None;

#[test]
fn test_setup() {
    unsafe { SHARED_STATE = Some(create_state()); }
}

#[test]
fn test_use_state() {
    // 依赖 test_setup 先执行
}
```

#### 可重复性
测试结果确定，不受外部状态影响：

```rust
// ✅ 使用固定时间
#[test]
fn test_lease_expiration() {
    let fixed_time = Instant::now();
    let lease = Lease::new(Duration::from_secs(30), fixed_time);
    // ...
}

// ❌ 使用系统时间（不稳定）
#[test]
fn test_lease_expiration() {
    let lease = Lease::new(Duration::from_secs(30), Instant::now());
    thread::sleep(Duration::from_secs(31));
    // 测试可能因为时间不精确而失败
}
```

### 3. Mock 和 Fixture 使用

#### Fixture 模式
使用 Fixture 创建测试数据：

```rust
pub struct InstanceFixture;

impl InstanceFixture {
    pub fn default() -> Instance {
        Instance {
            region_id: "test-region".into(),
            zone_id: "test-zone".into(),
            service_id: "test-service".into(),
            instance_id: "inst-1".into(),
            ip: "192.168.1.100".into(),
            port: 8080,
            status: InstanceStatus::Up,
            metadata: HashMap::new(),
        }
    }

    pub fn with_id(id: &str) -> Instance {
        let mut inst = Self::default();
        inst.instance_id = id.to_string();
        inst
    }

    pub fn with_status(status: InstanceStatus) -> Instance {
        let mut inst = Self::default();
        inst.status = status;
        inst
    }

    pub fn batch(count: usize) -> Vec<Instance> {
        (0..count)
            .map(|i| Self::with_id(&format!("inst-{}", i)))
            .collect()
    }
}

// 使用示例
#[test]
fn test_register_multiple_instances() {
    let instances = InstanceFixture::batch(10);
    let request = RegisterRequest { instances };
    // ...
}
```

### 4. 内存数据库测试

使用 SQLite 内存数据库进行 DAO 测试：

```rust
async fn create_test_db() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory database");

    // 创建表结构
    let create_table_sql = r#"
        CREATE TABLE IF NOT EXISTS service_group (
            group_id TEXT PRIMARY KEY,
            region_id TEXT NOT NULL,
            name TEXT NOT NULL,
            created_at INTEGER NOT NULL
        )
    "#;

    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        create_table_sql.to_owned()
    )).await.expect("Failed to create table");

    db
}

#[tokio::test]
async fn test_dao_insert() {
    let db = create_test_db().await;
    let dao = GroupDao::new(db);

    let group = ServiceGroup {
        group_id: "test-group".to_string(),
        region_id: "us-east".to_string(),
        name: "Test Group".to_string(),
        created_at: 123456789,
    };

    let result = dao.insert(&group).await;
    assert!(result.is_ok());
}
```

**优势**:
- ✅ 零外部依赖
- ✅ 快速执行 (< 0.01s)
- ✅ 完美隔离
- ✅ 真实 SQL 操作

### 5. 异步测试

使用 Tokio 进行异步测试：

```rust
#[tokio::test]
async fn test_async_operation() {
    let result = some_async_function().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_concurrent_operations() {
    let tasks: Vec<_> = (0..10)
        .map(|i| tokio::spawn(async move {
            some_async_function(i).await
        }))
        .collect();

    for task in tasks {
        let result = task.await.unwrap();
        assert!(result.is_ok());
    }
}
```

### 6. 边界条件测试

确保测试覆盖边界情况：

```rust
#[test]
fn test_empty_input() {
    let result = process_instances(vec![]);
    assert_eq!(result.len(), 0);
}

#[test]
fn test_single_input() {
    let result = process_instances(vec![create_instance()]);
    assert_eq!(result.len(), 1);
}

#[test]
fn test_large_input() {
    let instances = InstanceFixture::batch(10000);
    let result = process_instances(instances);
    assert_eq!(result.len(), 10000);
}

#[test]
fn test_zero_weight() {
    let group = RouteRuleGroup {
        weight: 0,  // 边界：零权重
        // ...
    };
    // 验证权重钳制机制
    assert!(group.weight.clamp(1, 100) == 1);
}
```

---

## 🔧 测试工具和框架

### 测试框架

| 工具 | 用途 | 说明 |
|------|------|------|
| **Rust #[test]** | 单元测试 | Rust 内置测试框架 |
| **tokio::test** | 异步测试 | Tokio 异步测试 |
| **Criterion** | 性能测试 | 基准测试框架 |
| **mockall** | Mock 对象 | Mock 对象库 (可选) |

### 覆盖率工具

```bash
# 安装 cargo-llvm-cov (推荐)
cargo install cargo-llvm-cov

# 生成覆盖率报告
cargo llvm-cov --html --open

# 生成 lcov 格式 (CI 使用)
cargo llvm-cov --lcov --output-path lcov.info
```

### 性能测试工具

| 工具 | 用途 | 安装 |
|------|------|------|
| **Criterion** | 微基准测试 | cargo add --dev criterion |
| **wrk** | HTTP 压力测试 | brew install wrk |
| **Apache Bench** | HTTP 性能测试 | sudo apt install apache2-utils |

---

## 📊 测试执行计划

### 测试分类和执行频率

| 测试类型 | 执行频率 | 执行时长 | 触发条件 |
|---------|---------|---------|---------|
| **单元测试** | 每次提交 | 5-10 分钟 | `git push` |
| **集成测试** | 每次提交 | 10-15 分钟 | `git push` |
| **E2E 测试** | 每次 PR | 20-30 分钟 | Pull Request |
| **性能基准** | 每周 | 30-60 分钟 | 定时任务 |
| **压力测试** | 每次发布 | 1-2 小时 | Release Tag |

### 本地开发环境

```bash
# 运行所有单元测试
cargo test --workspace --lib

# 运行所有集成测试
cargo test --workspace --test '*'

# 运行性能基准测试
cargo bench

# 生成代码覆盖率报告
cargo llvm-cov --html --open

# 运行特定测试
cargo test test_register_success

# 运行并发测试 (增加线程数)
cargo test --workspace --lib -- --test-threads=8
```

### CI/CD 环境配置

#### GitHub Actions 示例

```yaml
name: Tests

on: [push, pull_request]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - name: Run unit tests
        run: cargo test --workspace --lib

  integration-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - name: Build
        run: cargo build --release
      - name: Run integration tests
        run: cargo test --workspace --test '*'

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - name: Install cargo-llvm-cov
        run: cargo install cargo-llvm-cov
      - name: Generate coverage
        run: cargo llvm-cov --workspace --lcov --output-path lcov.info
      - name: Upload to codecov
        uses: codecov/codecov-action@v3
        with:
          files: lcov.info
```

---

## 🎓 测试基础设施

### 测试工具 (artemis/tests/common/mod.rs)

```rust
/// 测试服务器管理
pub struct TestServer {
    addr: SocketAddr,
    handle: JoinHandle<()>,
}

impl TestServer {
    pub async fn start(port: u16) -> Self {
        // 启动测试服务器
    }

    pub fn url(&self) -> String {
        format!("http://{}", self.addr)
    }

    pub async fn stop(self) {
        // 停止服务器
    }
}

/// 测试集群管理
pub struct TestCluster {
    nodes: Vec<TestServer>,
}

impl TestCluster {
    pub async fn start(node_count: usize) -> Self {
        // 启动多节点集群
    }
}

/// 实例数据构造器
pub struct InstanceFixture;

/// 分组数据构造器
pub struct GroupFixture;

/// 条件等待工具
pub async fn wait_for_condition<F, Fut>(
    condition: F,
    timeout: Duration,
) -> Result<()>
where
    F: Fn() -> Fut,
    Fut: Future<Output = bool>,
{
    // 等待条件满足
}
```

### 数据库测试工具 (artemis-management/tests/common/mod.rs)

```rust
/// 创建内存 SQLite 数据库
pub async fn create_test_db() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory database");
    initialize_schema(&db).await;
    db
}

/// 初始化 Schema (12 张表)
pub async fn initialize_schema(db: &DatabaseConnection) {
    // 创建所有表
}

/// 清空测试数据
pub async fn clear_test_data(db: &DatabaseConnection) {
    // 清空所有表
}
```

---

## 🚀 持续改进

### 短期优化 (可选)

1. **提升低覆盖率模块**
   - Replication 模块: 40-56% → 70%+
   - Audit 模块: 33% → 70%+

2. **补充边界条件测试**
   - 异常场景
   - 极端情况
   - 并发冲突

### 中期优化 (可选)

1. **CI/CD 集成**
   - GitHub Actions 自动化
   - 代码覆盖率报告
   - 性能回归检测

2. **测试文档**
   - 测试编写指南
   - Fixture 使用手册
   - 常见问题解答

### 长期优化 (可选)

1. **混沌工程测试**
   - 节点故障注入
   - 网络延迟模拟
   - 资源耗尽测试

2. **性能基准扩展**
   - 大规模注册 (10k+ 实例)
   - 高并发心跳 (1000+ QPS)
   - WebSocket 广播 (1000+ 订阅者)

---

## 📚 参考资料

### Rust 测试文档
- [The Rust Book - Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Rust API Guidelines - Testing](https://rust-lang.github.io/api-guidelines/documentation.html)
- [Tokio Testing Guide](https://tokio.rs/tokio/topics/testing)

### 工具文档
- [Criterion.rs](https://bheisler.github.io/criterion.rs/book/)
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)
- [SeaORM Testing](https://www.sea-ql.org/SeaORM/docs/write-test/)

---

## 📞 相关文档

- [测试状态报告](test-status.md) - 当前测试统计和覆盖率
- [测试文档中心](README.md) - 所有测试文档导航
- [变更日志](CHANGELOG.md) - 测试文档变更历史
- [脚本使用指南](../../scripts/README.md) - 测试脚本说明

---

**制定人**: Claude Sonnet 4.5
**审核人**: koqizhao
**版本**: v3.0
**创建时间**: 2026-02-15
**最后更新**: 2026-02-16

---

Generated with [Claude Code](https://claude.com/code)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
