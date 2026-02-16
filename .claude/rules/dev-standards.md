# Artemis 开发规范

## 开发环境

### 一键启动开发环境

使用 `dev.sh` 脚本可以快速启动完整的开发环境（后端服务 + Web Console）：

```bash
# 启动开发环境
./scripts/dev.sh start              # 启动后端 + 前端
./scripts/dev.sh start 1            # 启动单节点 + 前端
./scripts/dev.sh start 3 9000       # 启动 3 节点集群 (端口从 9000 开始) + 前端

# 查看状态
./scripts/dev.sh status             # 查看所有服务状态

# 查看日志
./scripts/dev.sh logs               # 查看所有日志
./scripts/dev.sh logs backend       # 仅查看后端日志
./scripts/dev.sh logs frontend      # 仅查看前端日志

# 重启服务
./scripts/dev.sh restart            # 重启所有服务
./scripts/dev.sh restart 1          # 重启为单节点

# 停止服务
./scripts/dev.sh stop               # 停止所有服务

# 清理环境
./scripts/dev.sh clean              # 清理日志和数据文件
```

**访问地址**:
- **Web Console**: http://localhost:5173
- **API 端点**: http://localhost:8080 (或指定的端口)

详见: [`docs/development.md`](../../docs/development.md)

---

## 代码质量规范

### 运行测试

```bash
# 所有测试
cargo test --workspace

# 集成测试
cargo test --test integration_test

# 性能基准
cargo bench --package artemis-server

# 代码覆盖率
cargo tarpaulin --out Html --output-dir coverage
```

### 代码检查

```bash
# 格式化
cargo fmt --all

# Lint 检查 (必须零警告)
cargo clippy --workspace -- -D warnings

# 构建所有 crate
cargo build --workspace

# 检查依赖是否过期
cargo outdated
```

### 提交规范

- **零编译警告**: 所有提交必须通过 `cargo clippy` 检查
- **测试通过**: 所有单元测试和集成测试必须通过
- **格式统一**: 使用 `cargo fmt` 格式化所有代码
- **提交信息**: 遵循 Conventional Commits 规范
  - `feat:` - 新功能
  - `fix:` - 修复 bug
  - `docs:` - 文档更新
  - `refactor:` - 代码重构
  - `test:` - 测试相关
  - `chore:` - 构建/工具链相关

---

## 模块开发规范

### Crate 依赖关系

```
artemis (CLI)
├── artemis-web
│   ├── artemis-server
│   └── artemis-management
└── artemis-client

artemis-server
└── artemis-core

artemis-management
├── artemis-core
└── sea-orm
```

**依赖规则**:
- **禁止循环依赖**: Crate 之间不允许循环依赖
- **核心在底层**: `artemis-core` 只定义数据结构和 Trait,不依赖其他 crate
- **客户端独立**: `artemis-client` 可独立使用,不依赖服务端 crate

### 错误处理

使用统一的错误类型 `ArtemisError`:

```rust
use artemis_core::error::ArtemisError;

pub fn my_function() -> Result<T, ArtemisError> {
    // 使用 ? 操作符传播错误
    let value = some_operation()?;

    // 自定义错误
    if !valid {
        return Err(ArtemisError::InvalidInput("reason".to_string()));
    }

    Ok(value)
}
```

### 日志规范

使用 `tracing` crate:

```rust
use tracing::{info, warn, error, debug};

// 信息日志
info!("Service registered: {}", instance_id);

// 警告日志
warn!("Heartbeat timeout for instance: {}", instance_id);

// 错误日志
error!("Failed to replicate data: {:?}", error);

// 调试日志 (仅开发环境)
debug!("Cache hit for service: {}", service_id);
```

### 并发编程

优先使用 Tokio 异步和 DashMap:

```rust
use dashmap::DashMap;
use tokio::sync::RwLock;

// 推荐: DashMap (lock-free)
let cache: DashMap<String, ServiceInstance> = DashMap::new();
cache.insert(key, value);

// 避免: 标准库 Mutex (性能差)
// let cache = Arc::new(Mutex::new(HashMap::new()));
```

---

## 测试规范

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_instance() {
        let service = RegistryService::new();
        let instance = create_test_instance();

        let result = service.register(instance).await;
        assert!(result.is_ok());
    }
}
```

### 集成测试

使用 `scripts/test-*.sh` 脚本进行端到端测试:

```bash
# 运行所有集成测试
./scripts/run-tests.sh

# 运行特定测试
./scripts/test-cluster-api.sh
./scripts/test-instance-management.sh
```

### 性能测试

使用 Criterion:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_register(c: &mut Criterion) {
    c.bench_function("register_instance", |b| {
        b.iter(|| {
            service.register(black_box(instance.clone()))
        });
    });
}

criterion_group!(benches, bench_register);
criterion_main!(benches);
```

---

## 数据库开发规范

### SeaORM 使用

```rust
use sea_orm::{Database, DatabaseConnection, Statement};

// 连接数据库
let db = Database::connect(&db_url).await?;

// 使用 Statement API (推荐)
let stmt = Statement::from_sql_and_values(
    DbBackend::Sqlite,
    "INSERT INTO groups (id, name) VALUES (?, ?)",
    vec![id.into(), name.into()],
);
db.execute(stmt).await?;
```

### 运行时数据库切换

```bash
# SQLite 模式 (开发环境)
DB_TYPE=sqlite ./scripts/cluster.sh start

# MySQL 模式 (生产环境)
DB_TYPE=mysql DB_URL="mysql://user:pass@host:3306/artemis" ./scripts/cluster.sh start
```

---

## 文档规范

遵循 [`.claude/rules/doc.md`](doc.md) 的文档组织规范：

### 版本化文档
- `docs/plans/` - 设计和计划文档
- `docs/testing/` - 测试文档
- `.claude/rules/` - AI 上下文规则

### 非版本化文档
- `docs/reports/` - 进度报告 (临时)
- `docs/archive/` - 归档文档 (历史)

---

## 依赖管理

### 版本锁定

所有依赖版本在 workspace `Cargo.toml` 中统一管理:

```toml
[workspace.dependencies]
tokio = { version = "1.40", features = ["full"] }
axum = "0.7"
sea-orm = { version = "1.2", features = ["runtime-tokio-rustls", "sqlx-sqlite", "sqlx-mysql"] }
dashmap = "6.1"
```

### 依赖更新策略

- **安全补丁**: 及时更新包含安全修复的版本
- **小版本更新**: 每月检查一次 `cargo outdated`
- **大版本升级**: 需要充分测试后才能升级

---

## 许可证

- **MIT OR Apache-2.0** 双许可证
- 可自由用于商业和开源项目
- 所有源文件必须包含许可证头部

---

## 相关文档

- **项目上下文**: [.claude/rules/project.md](project.md)
- **文档组织**: [.claude/rules/doc.md](doc.md)
- **实施路线图**: [`docs/plans/implementation-roadmap.md`](../../docs/plans/implementation-roadmap.md)
