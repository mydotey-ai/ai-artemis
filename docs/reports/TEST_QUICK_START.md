# Artemis 测试快速开始指南

**版本**: v1.0 | **更新时间**: 2026-02-15

---

## 🎯 测试现状概览

| 测试类型 | 当前状态 | 目标状态 | 完成度 |
|---------|---------|---------|--------|
| **单元测试** | 105 个测试 | 200+ 个测试 | 52% |
| **集成测试** | 13 个脚本 | 25+ 个场景 | 52% |
| **代码覆盖率** | ~60% | 80%+ | 75% |
| **性能测试** | 5 个基准 | 10+ 个基准 | 50% |

---

## 🚀 快速运行测试

### 运行所有单元测试
```bash
# 运行所有 workspace 的单元测试
cargo test --workspace --lib

# 运行单个 crate 的测试
cargo test -p artemis-server --lib

# 运行特定测试
cargo test test_register_success
```

### 运行集成测试
```bash
# 1. 构建项目
cargo build --release

# 2. 运行端到端测试
cargo test --test integration_tests

# 3. 运行集成测试脚本
./test-cluster-api.sh          # 集群 API 测试
./test-instance-management.sh  # 实例管理测试
./test-group-routing.sh         # 分组路由测试
./test-persistence.sh           # 数据持久化测试

# 4. 运行所有 scripts 目录下的测试
for script in scripts/test-*.sh; do
    bash "$script"
done
```

### 运行性能基准测试
```bash
# 运行所有基准测试
cargo bench

# 运行特定基准测试
cargo bench --bench performance -- register

# 生成性能报告
cargo bench -- --save-baseline baseline-2026-02-15
```

### 生成代码覆盖率报告
```bash
# 安装 cargo-llvm-cov
cargo install cargo-llvm-cov

# 生成 HTML 覆盖率报告
cargo llvm-cov --html --open

# 生成 lcov 格式 (CI 使用)
cargo llvm-cov --lcov --output-path lcov.info
```

---

## 📋 优先测试任务清单

### 🔴 高优先级 (本周完成)

#### 1. Web 层 API Handler 测试
**任务**: 为所有 HTTP handler 添加单元测试

**文件位置**: `artemis-web/src/handlers/`

**测试模板**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_handler_success() {
        // TODO: 实现正常路径测试
    }

    #[tokio::test]
    async fn test_handler_invalid_input() {
        // TODO: 实现错误处理测试
    }

    #[tokio::test]
    async fn test_handler_rate_limit() {
        // TODO: 实现限流测试
    }
}
```

**预计工作量**: 3-4 天 (50-60 个测试)

#### 2. 核心服务层单元测试
**任务**: 为 RegistryServiceImpl、DiscoveryServiceImpl、ReplicationManager 添加单元测试

**文件位置**:
- `artemis-server/src/registry/service_impl.rs`
- `artemis-server/src/discovery/service_impl.rs`
- `artemis-server/src/replication/manager.rs`

**测试示例**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_new_instance() {
        let repo = RegistryRepository::new();
        let lease_mgr = Arc::new(LeaseManager::new(Duration::from_secs(30)));
        let cache = Arc::new(VersionedCacheManager::new());
        let change_mgr = Arc::new(InstanceChangeManager::new());

        let service = RegistryServiceImpl::new(
            repo, lease_mgr, cache, change_mgr, None
        );

        let instance = create_test_instance("inst-1");
        let request = RegisterRequest { instances: vec![instance] };

        let response = service.register(request).await.unwrap();
        assert_eq!(response.response_status.error_code, ErrorCode::Success);
    }

    #[tokio::test]
    async fn test_register_duplicate_instance() {
        // TODO: 测试重复注册逻辑
    }
}
```

**预计工作量**: 3-4 天 (35-40 个测试)

#### 3. DAO 层持久化测试
**任务**: 为所有 DAO 添加单元测试

**文件位置**: `artemis-management/src/dao/`

**测试示例**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{Database, DatabaseConnection};

    async fn create_test_db() -> DatabaseConnection {
        Database::connect("sqlite::memory:").await.unwrap()
    }

    #[tokio::test]
    async fn test_insert_group() {
        let db = create_test_db().await;
        let dao = GroupDao::new(db.clone());

        let group = ServiceGroup {
            group_id: "test-group".to_string(),
            region_id: "us-east".to_string(),
            // ... 其他字段
        };

        let result = dao.insert(&group).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_transaction_rollback() {
        // TODO: 测试事务回滚
    }
}
```

**预计工作量**: 2-3 天 (40+ 个测试)

---

### 🟡 中优先级 (下周完成)

#### 4. 端到端场景测试扩展
**任务**: 添加完整的端到端测试场景

**文件位置**: `artemis/tests/e2e_scenarios.rs` (新建)

**测试场景**:
1. 完整服务生命周期 (注册 → 发现 → 心跳 → 注销)
2. 集群复制完整流程 (3 节点 + 数据一致性)
3. 分组路由端到端 (创建分组 → 绑定实例 → 路由)
4. 数据持久化端到端 (写入 → 重启 → 恢复)
5. 实例管理端到端 (拉入/拉出 → 过滤)

**预计工作量**: 3-4 天 (15-20 个测试)

#### 5. 性能基准测试扩展
**任务**: 添加大规模和高并发性能测试

**文件位置**: `artemis-server/benches/performance.rs`

**新增基准**:
```rust
fn bench_register_10k_instances(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("register_10k_instances", |b| {
        b.iter(|| {
            rt.block_on(async {
                let service = create_test_service();
                let instances: Vec<Instance> = (0..10000)
                    .map(create_test_instance)
                    .collect();
                let request = RegisterRequest { instances };
                service.register(request).await.unwrap();
            });
        });
    });
}
```

**预计工作量**: 2-3 天 (5 个新基准)

---

## 🛠️ 测试工具安装

### 1. 安装代码覆盖率工具
```bash
# cargo-llvm-cov (推荐,跨平台)
cargo install cargo-llvm-cov

# 或者 cargo-tarpaulin (仅 Linux)
cargo install cargo-tarpaulin
```

### 2. 安装性能测试工具
```bash
# wrk (HTTP 压力测试)
# macOS
brew install wrk

# Ubuntu/Debian
sudo apt-get install wrk

# Apache Bench
sudo apt-get install apache2-utils
```

### 3. 配置 Git Hooks (可选)
```bash
# 创建 pre-commit hook
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
echo "Running tests before commit..."
cargo test --workspace --lib
if [ $? -ne 0 ]; then
    echo "Tests failed! Commit aborted."
    exit 1
fi
echo "All tests passed!"
EOF

chmod +x .git/hooks/pre-commit
```

---

## 📊 查看测试报告

### 单元测试报告
```bash
# 详细输出
cargo test --workspace --lib -- --nocapture

# 只显示失败的测试
cargo test --workspace --lib -- --test-threads=1

# 生成 JUnit XML 报告
cargo test --workspace --lib -- -Z unstable-options --format json > test-results.json
```

### 代码覆盖率报告
```bash
# HTML 报告 (浏览器打开)
cargo llvm-cov --html --open

# 终端查看
cargo llvm-cov

# 生成报告文件
cargo llvm-cov --lcov --output-path coverage.lcov
```

### 性能基准测试报告
```bash
# 查看历史报告
ls -la target/criterion/

# 比较两次基准测试
cargo bench -- --save-baseline before
# ... 修改代码 ...
cargo bench -- --baseline before
```

---

## 🐛 常见问题

### Q1: 测试运行缓慢怎么办?
**A**: 使用并行测试和过滤:
```bash
# 增加并行线程
cargo test --workspace --lib -- --test-threads=8

# 只运行特定模块的测试
cargo test -p artemis-server registry::

# 跳过慢速测试 (标记为 #[ignore])
cargo test --workspace --lib -- --ignored
```

### Q2: 集成测试端口冲突怎么办?
**A**: 使用随机端口或顺序端口:
```rust
use std::sync::atomic::{AtomicU16, Ordering};
static PORT_COUNTER: AtomicU16 = AtomicU16::new(18080);

#[tokio::test]
async fn test_something() {
    let port = PORT_COUNTER.fetch_add(1, Ordering::SeqCst);
    let server = start_test_server(port).await;
    // ...
}
```

### Q3: 数据库测试如何隔离?
**A**: 使用内存数据库:
```rust
async fn create_test_db() -> DatabaseConnection {
    // 每次测试使用新的内存数据库
    Database::connect("sqlite::memory:").await.unwrap()
}
```

### Q4: 如何调试失败的测试?
**A**: 启用日志和详细输出:
```bash
# 启用 tracing 日志
RUST_LOG=debug cargo test test_name -- --nocapture

# 只运行单个测试
cargo test test_name -- --exact --nocapture
```

---

## 📈 测试度量指标

### 当前指标 (2026-02-15)
```
单元测试数量: 105
集成测试场景: 13
代码行覆盖率: ~60%
测试执行时间: ~15 分钟 (本地)
性能基准数量: 5
```

### 目标指标 (2026-03-15)
```
单元测试数量: 200+
集成测试场景: 25+
代码行覆盖率: 80%+
测试执行时间: < 30 分钟 (CI)
性能基准数量: 10+
```

---

## 🔗 相关文档

- **详细测试方案**: `docs/TEST_STRATEGY.md`
- **项目完成报告**: `docs/reports/project-completion.md`
- **性能报告**: `docs/reports/performance/performance-report.md`
- **集群管理指南**: `CLUSTER.md`

---

## 💡 最佳实践提示

1. **先写测试,再写代码** (TDD)
   - 明确需求
   - 快速反馈
   - 更好的设计

2. **保持测试简单**
   - 一个测试一个断言
   - 明确的测试命名
   - 避免复杂逻辑

3. **定期运行测试**
   - 每次提交前运行单元测试
   - 每次 PR 前运行所有测试
   - 定期检查代码覆盖率

4. **测试也需要重构**
   - 消除重复代码
   - 提取通用 Fixture
   - 保持测试可读性

---

**开始你的第一个测试**:
```bash
# 1. 克隆仓库
git clone https://github.com/mydotey/ai-artemis
cd ai-artemis

# 2. 运行现有测试
cargo test --workspace

# 3. 查看覆盖率
cargo llvm-cov --html --open

# 4. 选择一个任务开始编写测试!
```

**祝测试顺利!** 🚀
