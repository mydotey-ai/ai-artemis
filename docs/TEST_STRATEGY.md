# Artemis Rust - 全面测试方案

**制定时间**: 2026-02-15
**项目阶段**: 功能完成 (100%)
**当前测试覆盖**: 约 60-70%

---

## 📊 当前测试现状分析

### 1. 已有测试资产

#### 单元测试 (Unit Tests)
- **测试文件数量**: 40+ 文件包含单元测试
- **测试函数数量**: 105+ 个测试
- **覆盖模块**:
  - ✅ artemis-core: 5 个文件 (数据模型、Telemetry)
  - ✅ artemis-server: 15 个文件 (路由、租约、缓存、复制)
  - ✅ artemis-management: 7 个文件 (分组、路由、实例、审计)
  - ✅ artemis-client: 6 个文件 (配置、重试、过滤器、地址管理)
  - ✅ artemis-web: 1 个文件 (WebSocket 会话)

#### 集成测试 (Integration Tests)
- **端到端测试**: `artemis/tests/integration_tests.rs` (3 个测试场景)
- **客户端企业功能测试**: `artemis-client/tests/enterprise_features.rs` (7 个测试)
- **集成测试脚本**: 13 个 Shell 脚本
  - ✅ test-cluster-api.sh - 集群 API 测试
  - ✅ test-instance-management.sh - 实例管理 (13 步)
  - ✅ test-group-routing.sh - 分组路由 (13 步)
  - ✅ test-persistence.sh - 数据持久化
  - ✅ test-management.sh - 管理功能
  - ✅ test-group-instance-binding.sh - 分组实例绑定 (9 步)
  - ✅ test-discovery-lookup.sh - 服务发现查询
  - ✅ test-status-api.sh - 状态查询 API (12 步)
  - ✅ test-get-query-params.sh - GET 查询参数 (7 步)
  - ✅ test-audit-logs.sh - 审计日志 (11 步)
  - ✅ test-all-operations.sh - 批量操作查询 (11 步)
  - ✅ test-batch-replication.sh - 批量复制 (8 步)
  - ✅ cluster.sh - 集群管理 (启动/停止/状态)

#### 性能测试 (Benchmarks)
- **Criterion 基准测试**: `artemis-server/benches/performance.rs`
  - 注册性能 (1/10/100 实例)
  - 心跳性能
  - 发现查询性能

### 2. 测试覆盖缺口分析

#### 🔴 高优先级缺口 (P0)
1. **Web 层 API 测试严重不足**
   - `artemis-web/src/handlers/` 所有 HTTP handler 缺少单元测试
   - 101 个 API 端点仅靠集成测试覆盖
   - 缺少错误处理、边界条件测试

2. **核心服务层缺少独立单元测试**
   - `RegistryServiceImpl` - 0 个单元测试 (仅集成测试)
   - `DiscoveryServiceImpl` - 0 个单元测试
   - `ReplicationManager` - 0 个单元测试

3. **数据库持久化层测试不足**
   - 4 个 DAO (GroupDao, RouteRuleDao, ZoneOperationDao, CanaryConfigDao) 缺少独立测试
   - SeaORM 迁移后的事务测试缺失
   - 数据库切换 (SQLite ↔ MySQL) 缺少验证测试

4. **错误处理和边界条件测试不足**
   - 网络故障场景
   - 并发冲突场景
   - 资源耗尽场景

#### 🟡 中优先级缺口 (P1)
5. **集群复制的压力测试**
   - 批量复制在大数据量下的表现
   - 网络分区恢复测试
   - 数据一致性验证

6. **WebSocket 实时推送测试不足**
   - 连接断线重连测试
   - 大量订阅者性能测试
   - 消息顺序保证测试

7. **限流器测试覆盖不足**
   - 高并发下的限流准确性
   - 分布式限流测试

#### 🟢 低优先级缺口 (P2)
8. **性能回归测试自动化**
   - 缺少 CI/CD 集成的性能基线
   - 缺少性能趋势监控

9. **混沌工程测试**
   - 节点故障注入
   - 网络延迟/丢包模拟

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

### 目标测试覆盖率
- **代码行覆盖率**: 80%+ (当前 ~60%)
- **分支覆盖率**: 75%+
- **关键路径覆盖率**: 100% (注册、发现、心跳、复制)

---

## 📋 详细测试计划

### Phase 1: 补充核心单元测试 (P0 - 2 周)

#### 1.1 Web 层 API Handler 测试
**文件**: `artemis-web/src/handlers/*.rs`

```rust
// 需要为每个 handler 添加测试
// 示例: test_register_handler.rs

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_register_success() {
        // 正常注册
    }

    #[tokio::test]
    async fn test_register_empty_instances() {
        // 边界条件: 空实例列表
    }

    #[tokio::test]
    async fn test_register_invalid_instance() {
        // 错误处理: 无效实例数据
    }

    #[tokio::test]
    async fn test_register_rate_limit() {
        // 限流触发
    }
}
```

**测试覆盖**:
- ✅ 正常路径 (Happy Path)
- ✅ 边界条件 (空列表、单个/批量实例)
- ✅ 错误处理 (无效输入、缺失字段)
- ✅ 限流触发
- ✅ 并发请求

**预计测试数量**: 50-60 个测试 (每个 handler 5-6 个)

#### 1.2 核心服务层单元测试
**文件**:
- `artemis-server/src/registry/service_impl.rs`
- `artemis-server/src/discovery/service_impl.rs`
- `artemis-server/src/replication/manager.rs`

```rust
// 示例: test_registry_service.rs

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_register_new_instance() {
        // 测试首次注册逻辑
    }

    #[tokio::test]
    async fn test_register_duplicate_instance() {
        // 测试重复注册
    }

    #[tokio::test]
    async fn test_heartbeat_extends_lease() {
        // 验证心跳更新租约
    }

    #[tokio::test]
    async fn test_unregister_removes_instance() {
        // 验证注销删除实例
    }

    #[tokio::test]
    async fn test_concurrent_registrations() {
        // 并发注册测试
    }
}
```

**测试覆盖**:
- RegistryServiceImpl: 15+ 测试
- DiscoveryServiceImpl: 12+ 测试
- ReplicationManager: 10+ 测试

#### 1.3 数据库持久化层测试
**文件**: `artemis-management/src/dao/*.rs`

```rust
// 示例: test_group_dao.rs

#[cfg(test)]
mod tests {
    use artemis_management::db::create_test_db;

    #[tokio::test]
    async fn test_insert_group() {
        let db = create_test_db().await;
        let dao = GroupDao::new(db.clone());
        // 测试插入
    }

    #[tokio::test]
    async fn test_update_group() {
        // 测试更新
    }

    #[tokio::test]
    async fn test_delete_group() {
        // 测试删除
    }

    #[tokio::test]
    async fn test_transaction_rollback() {
        // 测试事务回滚
    }

    #[tokio::test]
    async fn test_concurrent_updates() {
        // 测试并发更新
    }
}
```

**测试覆盖**:
- 每个 DAO: 8-10 个测试
- 事务处理测试
- 数据库切换测试 (SQLite ↔ MySQL)

**预计测试数量**: 40+ 个测试

---

### Phase 2: 增强集成测试 (P0 - 1.5 周)

#### 2.1 端到端场景测试扩展
**文件**: `artemis/tests/e2e_scenarios.rs` (新建)

**测试场景**:
1. **完整服务生命周期**
   - 注册 → 发现 → 心跳 → 健康检查 → 注销
   - WebSocket 订阅 + 实时推送

2. **集群复制完整流程**
   - 3 节点集群
   - 注册到节点 A → 复制到节点 B/C
   - 验证数据一致性
   - 节点故障 + 恢复

3. **分组路由端到端**
   - 创建分组 → 绑定实例 → 配置规则 → 服务发现
   - 验证加权轮询 + 就近访问

4. **数据持久化端到端**
   - 配置写入 → 服务重启 → 配置恢复
   - SQLite/MySQL 双模式测试

5. **实例管理端到端**
   - 拉入/拉出 → 服务发现过滤 → 状态查询

**预计测试数量**: 15-20 个场景测试

#### 2.2 错误恢复测试
**文件**: `artemis/tests/error_recovery.rs` (新建)

**测试场景**:
1. 网络故障恢复
2. 数据库连接失败恢复
3. 内存耗尽保护
4. 并发冲突解决
5. WebSocket 断线重连

**预计测试数量**: 10-12 个测试

---

### Phase 3: 性能和压力测试 (P1 - 1 周)

#### 3.1 扩展性能基准测试
**文件**: `artemis-server/benches/performance.rs`

**新增基准测试**:
```rust
// 1. 大规模注册
fn bench_register_10k_instances(c: &mut Criterion) {
    // 10,000 实例批量注册
}

// 2. 高并发心跳
fn bench_concurrent_heartbeats(c: &mut Criterion) {
    // 1000 并发心跳请求
}

// 3. 复杂查询性能
fn bench_discovery_with_routing(c: &mut Criterion) {
    // 分组路由下的服务发现
}

// 4. WebSocket 广播性能
fn bench_websocket_broadcast(c: &mut Criterion) {
    // 1000 订阅者广播
}

// 5. 数据库持久化性能
fn bench_dao_operations(c: &mut Criterion) {
    // DAO 批量操作
}
```

**性能目标**:
- 10k 实例注册: < 500ms
- 并发心跳 (1000 QPS): P99 < 1ms
- 服务发现 (100k 实例): < 5ms
- WebSocket 广播 (1000 订阅者): < 100ms

#### 3.2 压力测试脚本
**文件**: `scripts/stress-test.sh` (新建)

```bash
#!/bin/bash
# 压力测试脚本

# 1. 启动 3 节点集群
./cluster.sh start

# 2. 注册 10,000 实例
for i in {1..10000}; do
    # 批量注册 (100 个/批)
done

# 3. 高并发查询 (1000 QPS 持续 5 分钟)
wrk -t10 -c100 -d300s http://localhost:8080/api/discovery/service.json

# 4. 监控指标收集
curl http://localhost:8080/metrics | grep artemis_

# 5. 清理
./cluster.sh stop
```

---

### Phase 4: 专项测试 (P1 - 1 周)

#### 4.1 WebSocket 实时推送测试
**文件**: `artemis-web/tests/websocket_tests.rs` (新建)

**测试覆盖**:
- 大量订阅者 (1000+) 性能测试
- 断线重连机制测试
- 消息顺序保证测试
- Ping/Pong 健康检查测试

**预计测试数量**: 8-10 个测试

#### 4.2 集群复制一致性测试
**文件**: `artemis-server/tests/replication_consistency.rs` (新建)

**测试场景**:
1. 网络分区后数据同步
2. 批量复制数据完整性
3. 复制失败重试机制
4. 防复制循环机制
5. 节点动态加入/离开

**预计测试数量**: 10-12 个测试

#### 4.3 限流器专项测试
**文件**: `artemis-server/src/ratelimiter/limiter.rs` (增强现有测试)

**测试覆盖**:
- 高并发限流准确性 (>1000 QPS)
- 限流恢复测试 (Token 补充)
- 多服务限流隔离
- 限流配置热更新

**预计测试数量**: 6-8 个测试

---

### Phase 5: 测试基础设施建设 (P2 - 1 周)

#### 5.1 测试工具和 Fixture
**文件**: `artemis/tests/common/mod.rs` (新建)

```rust
// 通用测试工具

pub struct TestServer {
    pub port: u16,
    pub handle: JoinHandle<()>,
}

impl TestServer {
    pub async fn start() -> Self {
        // 启动测试服务器
    }

    pub async fn stop(self) {
        // 停止服务器
    }

    pub fn client_config(&self) -> ClientConfig {
        // 返回客户端配置
    }
}

pub struct TestCluster {
    pub nodes: Vec<TestServer>,
}

impl TestCluster {
    pub async fn start(node_count: usize) -> Self {
        // 启动多节点集群
    }

    pub async fn stop(self) {
        // 停止所有节点
    }
}

pub fn create_test_instance(id: &str) -> Instance {
    // 创建测试实例
}

pub async fn create_test_db() -> DatabaseConnection {
    // 创建测试数据库 (in-memory SQLite)
}
```

#### 5.2 代码覆盖率集成
**工具**: `cargo-tarpaulin` 或 `cargo-llvm-cov`

**配置文件**: `.cargo/config.toml`
```toml
[target.'cfg(all())']
rustflags = ["-C", "instrument-coverage"]

[build]
rustflags = ["-C", "instrument-coverage"]
```

**CI 集成**: `.github/workflows/coverage.yml`
```yaml
name: Code Coverage

on: [push, pull_request]

jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - name: Install cargo-llvm-cov
        run: cargo install cargo-llvm-cov
      - name: Generate coverage
        run: cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
      - name: Upload to codecov.io
        uses: codecov/codecov-action@v3
        with:
          files: lcov.info
```

#### 5.3 CI/CD 测试流水线
**文件**: `.github/workflows/tests.yml`

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

  e2e-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - name: Build
        run: cargo build --release
      - name: Run cluster tests
        run: ./test-cluster-api.sh
      - name: Run instance management tests
        run: ./test-instance-management.sh
      - name: Run group routing tests
        run: ./test-group-routing.sh

  benchmarks:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - name: Run benchmarks
        run: cargo bench --no-run
```

---

## 📊 测试执行计划

### 测试分类和执行频率

| 测试类型 | 执行频率 | 执行时长 | 触发条件 |
|---------|---------|---------|---------|
| **单元测试** | 每次提交 | 5-10 分钟 | `git push` |
| **集成测试** | 每次提交 | 10-15 分钟 | `git push` |
| **端到端测试** | 每次 PR | 20-30 分钟 | Pull Request |
| **性能基准测试** | 每周 | 30-60 分钟 | 定时任务 |
| **压力测试** | 每次发布 | 1-2 小时 | Release Tag |
| **代码覆盖率** | 每次提交 | 15-20 分钟 | `git push` |

### 测试环境配置

#### 本地开发环境
```bash
# 运行所有单元测试
cargo test --workspace --lib

# 运行所有集成测试
cargo test --workspace --test '*'

# 运行性能基准测试
cargo bench

# 生成代码覆盖率报告
cargo llvm-cov --html --open
```

#### CI/CD 环境
- **GitHub Actions**: 自动化测试流水线
- **Docker**: 隔离测试环境
- **SQLite**: 单元测试数据库
- **MySQL**: 集成测试数据库 (Docker Compose)

---

## 🎯 测试指标和目标

### 短期目标 (1 个月)
- ✅ 代码行覆盖率: 60% → **80%**
- ✅ Web 层测试: 0% → **90%**
- ✅ 核心服务层测试: 30% → **85%**
- ✅ DAO 层测试: 0% → **80%**
- ✅ 集成测试场景: 13 个 → **25 个**

### 中期目标 (2 个月)
- ✅ 代码行覆盖率: **85%+**
- ✅ 分支覆盖率: **75%+**
- ✅ 性能回归测试自动化
- ✅ 压力测试报告自动生成
- ✅ 测试执行时间: < 30 分钟 (CI)

### 长期目标 (3 个月)
- ✅ 代码行覆盖率: **90%+**
- ✅ 混沌工程测试集成
- ✅ 生产环境监控集成
- ✅ 自动化性能基线管理

---

## 📝 测试最佳实践

### 1. 测试命名规范
```rust
// 格式: test_<function>_<scenario>_<expected_result>
#[test]
fn test_register_empty_instances_returns_error() {}

#[test]
fn test_heartbeat_expired_lease_renews_successfully() {}

#[test]
fn test_discover_with_routing_filters_down_instances() {}
```

### 2. 测试组织原则
- **单一职责**: 每个测试只验证一个功能点
- **独立性**: 测试之间不依赖执行顺序
- **可重复性**: 测试结果确定,不受外部状态影响
- **快速反馈**: 单元测试 < 1s,集成测试 < 10s

### 3. Mock 和 Fixture 使用
```rust
// 使用 trait 隔离依赖
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::*;

    mock! {
        RegistryRepo {}
        impl RegistryRepository for RegistryRepo {
            async fn get_instance(&self, key: &InstanceKey) -> Option<Instance>;
        }
    }

    #[tokio::test]
    async fn test_with_mock() {
        let mut mock_repo = MockRegistryRepo::new();
        mock_repo.expect_get_instance()
            .with(eq(InstanceKey::new("test", "zone", "svc", "inst")))
            .times(1)
            .returning(|_| Some(create_test_instance()));
    }
}
```

### 4. 测试数据管理
```rust
// 使用 Fixture 创建测试数据
pub struct InstanceFixture;

impl InstanceFixture {
    pub fn default() -> Instance {
        Instance {
            region_id: "test".into(),
            zone_id: "zone".into(),
            service_id: "service".into(),
            instance_id: "inst-1".into(),
            // ... 其他字段
        }
    }

    pub fn with_id(id: &str) -> Instance {
        Self::default().with_instance_id(id)
    }

    pub fn batch(count: usize) -> Vec<Instance> {
        (0..count).map(|i| Self::with_id(&format!("inst-{}", i))).collect()
    }
}
```

---

## 🚀 实施路线图

### Week 1-2: Phase 1 (核心单元测试)
- [ ] Day 1-3: Web 层 API Handler 测试 (15 个 handler × 5 测试 = 75 个测试)
- [ ] Day 4-6: 核心服务层测试 (RegistryService, DiscoveryService)
- [ ] Day 7-10: 数据库持久化层测试 (4 DAO × 10 测试 = 40 个测试)

**交付物**:
- 115+ 新增单元测试
- 代码覆盖率提升至 75%+

### Week 3: Phase 2 (集成测试增强)
- [ ] Day 1-3: 端到端场景测试 (5 个场景 × 3-4 测试 = 15-20 个测试)
- [ ] Day 4-5: 错误恢复测试 (10-12 个测试)

**交付物**:
- 25-32 新增集成测试
- 集成测试场景从 13 个增加到 25+ 个

### Week 4: Phase 3 (性能和压力测试)
- [ ] Day 1-2: 扩展性能基准测试 (5 个新基准)
- [ ] Day 3-4: 压力测试脚本开发
- [ ] Day 5: 性能报告自动化

**交付物**:
- 5 个新性能基准测试
- 压力测试脚本
- 性能报告模板

### Week 5: Phase 4 (专项测试)
- [ ] Day 1-2: WebSocket 实时推送测试 (8-10 个测试)
- [ ] Day 3-4: 集群复制一致性测试 (10-12 个测试)
- [ ] Day 5: 限流器专项测试 (6-8 个测试)

**交付物**:
- 24-30 专项测试
- WebSocket/集群/限流专项测试报告

### Week 6: Phase 5 (测试基础设施)
- [ ] Day 1-2: 测试工具和 Fixture 开发
- [ ] Day 3: 代码覆盖率集成
- [ ] Day 4-5: CI/CD 测试流水线配置

**交付物**:
- 通用测试工具库
- CI/CD 自动化流水线
- 代码覆盖率报告集成

---

## 📈 成功标准

### 定量指标
- ✅ **代码覆盖率**: 80%+ (当前 60%)
- ✅ **单元测试数量**: 200+ (当前 105)
- ✅ **集成测试场景**: 25+ (当前 13)
- ✅ **性能基准测试**: 10+ (当前 5)
- ✅ **测试执行时间**: < 30 分钟 (CI)
- ✅ **测试通过率**: 100%

### 定性指标
- ✅ 所有核心功能有完整测试覆盖
- ✅ 所有 API 端点有单元测试 + 集成测试
- ✅ 错误处理和边界条件有明确测试
- ✅ 性能回归可自动检测
- ✅ CI/CD 流水线稳定运行

---

## 🔧 工具和依赖

### 测试框架
- **单元测试**: Rust 内置 `#[test]`
- **异步测试**: `tokio::test`
- **性能测试**: Criterion
- **Mock**: mockall (可选)

### 覆盖率工具
- **cargo-llvm-cov**: LLVM-based 覆盖率工具
- **cargo-tarpaulin**: 覆盖率工具 (Linux only)

### CI/CD
- **GitHub Actions**: 自动化流水线
- **Docker**: 隔离测试环境
- **Docker Compose**: 多服务测试环境

### 性能测试
- **wrk**: HTTP 压力测试
- **Apache Bench**: HTTP 性能测试
- **k6**: 现代化负载测试

---

## 📚 参考资料

### Rust 测试最佳实践
- [The Rust Book - Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Rust API Guidelines - Testing](https://rust-lang.github.io/api-guidelines/documentation.html#examples-use-crate-not-crate-name-c-example)

### 工具文档
- [Criterion.rs](https://bheisler.github.io/criterion.rs/book/)
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)
- [Tokio Testing](https://tokio.rs/tokio/topics/testing)

---

## 📞 联系和反馈

**制定人**: Claude Sonnet 4.5
**审核人**: koqizhao
**版本**: v1.0
**更新时间**: 2026-02-15

---

**下一步行动**:
1. 审核和批准测试方案
2. 创建 GitHub Issues 跟踪任务
3. 分配资源和时间表
4. 开始 Phase 1 实施
