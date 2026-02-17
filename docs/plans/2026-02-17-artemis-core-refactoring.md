# Artemis Core 重构实施计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**目标**: 精简 artemis-core 为核心协议层，将 server/management 特有功能移到对应 crate

**架构**: 按业务领域拆分 - 核心协议留在 core，server 基础设施移到 server，management 模型移到 management

**技术栈**: Rust, Cargo workspace, 文件迁移 + 导入路径更新

**设计文档**: `docs/plans/2026-02-17-artemis-core-refactoring-design.md`

---

## 前置准备

### Task 0: 创建备份和验证基线

**Step 1: 创建备份分支**

```bash
git checkout -b refactor/artemis-core
```

Expected: `Switched to a new branch 'refactor/artemis-core'`

**Step 2: 验证当前编译状态**

```bash
cargo build --workspace
```

Expected: 编译成功，零警告

**Step 3: 验证当前测试状态**

```bash
cargo test --workspace --no-fail-fast
```

Expected: 所有测试通过

**Step 4: 记录当前代码行数**

```bash
find artemis-core/src -name "*.rs" -exec wc -l {} + | tail -1
```

Expected: 显示总行数（约 2000+ 行）

---

## Phase 1: 准备目标 Crate 的目录结构

### Task 1: 在 artemis-server 创建新模块目录

**Files:**
- Create: `artemis-server/src/model/mod.rs`
- Create: `artemis-server/src/traits/mod.rs`

**Step 1: 创建 artemis-server/src/model/ 目录和 mod.rs**

```bash
mkdir -p artemis-server/src/model
```

**Step 2: 创建 artemis-server/src/model/mod.rs**

Create file: `artemis-server/src/model/mod.rs`

```rust
//! Server-specific data models

pub mod lease;
pub mod replication;

pub use lease::Lease;
pub use replication::*;
```

**Step 3: 创建 artemis-server/src/traits/mod.rs（占位）**

```bash
mkdir -p artemis-server/src/traits
```

Create file: `artemis-server/src/traits/mod.rs`

```rust
//! Service traits for server business logic

pub mod discovery;
pub mod registry;

pub use discovery::DiscoveryService;
pub use registry::RegistryService;
```

**Step 4: 提交**

```bash
git add artemis-server/src/model/mod.rs artemis-server/src/traits/mod.rs
git commit -m "chore: 创建 artemis-server 新模块目录结构"
```

---

### Task 2: 在 artemis-management 创建新模块目录

**Files:**
- Create: `artemis-management/src/model/mod.rs`

**Step 1: 创建 artemis-management/src/model/ 目录**

```bash
mkdir -p artemis-management/src/model
```

**Step 2: 创建 artemis-management/src/model/mod.rs**

Create file: `artemis-management/src/model/mod.rs`

```rust
//! Management-specific data models

pub mod canary;
pub mod group;
pub mod management;
pub mod route;
pub mod status;
pub mod zone;

pub use canary::*;
pub use group::*;
pub use management::*;
pub use route::*;
pub use status::*;
pub use zone::*;
```

**Step 3: 提交**

```bash
git add artemis-management/src/model/mod.rs
git commit -m "chore: 创建 artemis-management 新模块目录结构"
```

---

## Phase 2: 迁移文件到 artemis-server

### Task 3: 迁移 config 模块

**Files:**
- Move: `artemis-core/src/config/` -> `artemis-server/src/config/`

**Step 1: 移动 config 目录**

```bash
git mv artemis-core/src/config artemis-server/src/config
```

**Step 2: 更新 artemis-server/src/lib.rs（添加 config 模块）**

Modify: `artemis-server/src/lib.rs`

在文件开头添加：
```rust
pub mod config;
```

**Step 3: 验证编译（只编译 artemis-server）**

```bash
cargo build -p artemis-server 2>&1 | head -50
```

Expected: 编译通过或只有导入路径错误（下一步修复）

**Step 4: 查找 artemis-server 中使用 artemis_core::config 的地方**

```bash
grep -r "artemis_core::config" artemis-server/src
```

**Step 5: 替换导入路径**

在 artemis-server 中，将所有 `use artemis_core::config` 替换为 `use crate::config`

```bash
find artemis-server/src -name "*.rs" -type f -exec sed -i 's/artemis_core::config/crate::config/g' {} +
```

**Step 6: 验证编译**

```bash
cargo build -p artemis-server
```

Expected: 编译成功

**Step 7: 提交**

```bash
git add -A
git commit -m "refactor: 迁移 config 模块从 artemis-core 到 artemis-server"
```

---

### Task 4: 迁移 telemetry 模块

**Files:**
- Move: `artemis-core/src/telemetry/` -> `artemis-server/src/telemetry/`

**Step 1: 移动 telemetry 目录**

```bash
git mv artemis-core/src/telemetry artemis-server/src/telemetry
```

**Step 2: 更新 artemis-server/src/lib.rs（添加 telemetry 模块）**

Modify: `artemis-server/src/lib.rs`

添加：
```rust
pub mod telemetry;
```

**Step 3: 查找使用 artemis_core::telemetry 的地方**

```bash
grep -r "artemis_core::telemetry" artemis-server/src artemis-web/src artemis/src
```

**Step 4: 替换导入路径**

在 artemis-server 中：
```bash
find artemis-server/src -name "*.rs" -type f -exec sed -i 's/use artemis_core::telemetry/use crate::telemetry/g' {} +
```

在 artemis-web 中：
```bash
find artemis-web/src -name "*.rs" -type f -exec sed -i 's/use artemis_core::telemetry/use artemis_server::telemetry/g' {} +
```

在 artemis/src 中：
```bash
find artemis/src -name "*.rs" -type f -exec sed -i 's/use artemis_core::telemetry/use artemis_server::telemetry/g' {} +
```

**Step 5: 验证编译**

```bash
cargo build -p artemis-server -p artemis-web -p artemis
```

Expected: 编译成功

**Step 6: 提交**

```bash
git add -A
git commit -m "refactor: 迁移 telemetry 模块从 artemis-core 到 artemis-server"
```

---

### Task 5: 迁移 utils.rs

**Files:**
- Move: `artemis-core/src/utils.rs` -> `artemis-server/src/utils.rs`

**Step 1: 移动 utils.rs**

```bash
git mv artemis-core/src/utils.rs artemis-server/src/utils.rs
```

**Step 2: 更新 artemis-server/src/lib.rs（添加 utils 模块）**

Modify: `artemis-server/src/lib.rs`

添加：
```rust
pub mod utils;
```

**Step 3: 查找使用 artemis_core::utils 的地方**

```bash
grep -r "artemis_core::utils" artemis-server/src
```

**Step 4: 替换导入路径**

```bash
find artemis-server/src -name "*.rs" -type f -exec sed -i 's/use artemis_core::utils/use crate::utils/g' {} +
```

**Step 5: 验证编译**

```bash
cargo build -p artemis-server
```

Expected: 编译成功

**Step 6: 提交**

```bash
git add -A
git commit -m "refactor: 迁移 utils.rs 从 artemis-core 到 artemis-server"
```

---

### Task 6: 迁移 traits 模块

**Files:**
- Move: `artemis-core/src/traits/registry.rs` -> `artemis-server/src/traits/registry.rs`
- Move: `artemis-core/src/traits/discovery.rs` -> `artemis-server/src/traits/discovery.rs`
- Delete: `artemis-core/src/traits/mod.rs`

**Step 1: 移动 traits 文件**

```bash
git mv artemis-core/src/traits/registry.rs artemis-server/src/traits/registry.rs
git mv artemis-core/src/traits/discovery.rs artemis-server/src/traits/discovery.rs
git rm artemis-core/src/traits/mod.rs
rmdir artemis-core/src/traits
```

**Step 2: 更新 artemis-server/src/lib.rs（添加 traits 模块）**

Modify: `artemis-server/src/lib.rs`

添加：
```rust
pub mod traits;
```

**Step 3: 查找所有使用 artemis_core::traits 的地方**

```bash
grep -r "use artemis_core::traits" artemis-server/src artemis-web/src
```

**Step 4: 替换导入路径 - artemis-server**

```bash
find artemis-server/src -name "*.rs" -type f -exec sed -i 's/use artemis_core::traits/use crate::traits/g' {} +
```

**Step 5: 替换导入路径 - artemis-web**

```bash
find artemis-web/src -name "*.rs" -type f -exec sed -i 's/use artemis_core::traits/use artemis_server::traits/g' {} +
```

**Step 6: 验证编译**

```bash
cargo build -p artemis-server -p artemis-web
```

Expected: 编译成功

**Step 7: 提交**

```bash
git add -A
git commit -m "refactor: 迁移 traits 模块从 artemis-core 到 artemis-server"
```

---

### Task 7: 迁移 model/lease.rs 和 model/replication.rs

**Files:**
- Move: `artemis-core/src/model/lease.rs` -> `artemis-server/src/model/lease.rs`
- Move: `artemis-core/src/model/replication.rs` -> `artemis-server/src/model/replication.rs`

**Step 1: 移动文件**

```bash
git mv artemis-core/src/model/lease.rs artemis-server/src/model/lease.rs
git mv artemis-core/src/model/replication.rs artemis-server/src/model/replication.rs
```

**Step 2: 更新 artemis-server/src/lib.rs（添加 model 模块）**

Modify: `artemis-server/src/lib.rs`

添加：
```rust
pub mod model;
```

**Step 3: 查找使用这些模型的地方**

```bash
grep -r "artemis_core::model::Lease" artemis-server/src
grep -r "artemis_core::model::.*replication" artemis-server/src artemis-web/src
```

**Step 4: 替换导入路径 - artemis-server**

在 artemis-server 中，将 `use artemis_core::model::{InstanceKey, Lease}` 改为：
```rust
use artemis_core::model::InstanceKey;
use crate::model::Lease;
```

手动检查并更新相关文件（主要是 `artemis-server/src/lease/manager.rs`）

**Step 5: 替换 replication 相关导入**

```bash
find artemis-server/src -name "*.rs" -type f -exec sed -i 's/use artemis_core::model::replication/use crate::model::replication/g' {} +
find artemis-web/src -name "*.rs" -type f -exec sed -i 's/use artemis_core::model::replication/use artemis_server::model::replication/g' {} +
```

**Step 6: 验证编译**

```bash
cargo build -p artemis-server -p artemis-web
```

Expected: 编译成功

**Step 7: 提交**

```bash
git add -A
git commit -m "refactor: 迁移 lease 和 replication 模型到 artemis-server"
```

---

## Phase 3: 迁移文件到 artemis-management

### Task 8: 迁移 management 相关模型

**Files:**
- Move: `artemis-core/src/model/management.rs` -> `artemis-management/src/model/management.rs`
- Move: `artemis-core/src/model/group.rs` -> `artemis-management/src/model/group.rs`
- Move: `artemis-core/src/model/route.rs` -> `artemis-management/src/model/route.rs`
- Move: `artemis-core/src/model/zone.rs` -> `artemis-management/src/model/zone.rs`
- Move: `artemis-core/src/model/canary.rs` -> `artemis-management/src/model/canary.rs`
- Move: `artemis-core/src/model/status.rs` -> `artemis-management/src/model/status.rs`

**Step 1: 移动文件**

```bash
git mv artemis-core/src/model/management.rs artemis-management/src/model/management.rs
git mv artemis-core/src/model/group.rs artemis-management/src/model/group.rs
git mv artemis-core/src/model/route.rs artemis-management/src/model/route.rs
git mv artemis-core/src/model/zone.rs artemis-management/src/model/zone.rs
git mv artemis-core/src/model/canary.rs artemis-management/src/model/canary.rs
git mv artemis-core/src/model/status.rs artemis-management/src/model/status.rs
```

**Step 2: 更新 artemis-management/src/lib.rs（添加 model 模块）**

Modify: `artemis-management/src/lib.rs`

在文件开头添加：
```rust
pub mod model;
```

**Step 3: 查找使用这些模型的地方**

```bash
grep -r "artemis_core::model::.*Group" artemis-management/src artemis-server/src artemis-web/src
grep -r "artemis_core::model::.*Route" artemis-management/src artemis-server/src artemis-web/src
grep -r "artemis_core::model::.*Zone" artemis-management/src artemis-web/src
grep -r "artemis_core::model::.*Canary" artemis-management/src artemis-web/src
grep -r "artemis_core::model::.*Operation" artemis-management/src artemis-web/src
```

**Step 4: 替换导入路径 - artemis-management**

```bash
# 将 artemis_core::model 改为 crate::model（在 artemis-management 内部）
find artemis-management/src -name "*.rs" -type f ! -path "*/model/*" -exec sed -i 's/use artemis_core::model::\(ServiceGroup\|GroupTag\|GroupType\|GroupStatus\|GroupOperation\|GroupInstance\|RouteRule\|RouteRuleGroup\|RouteStrategy\|RouteRuleStatus\|ZoneOperation\|CanaryConfig\|InstanceOperation\|ServerOperation\)/use crate::model::\1/g' {} +
```

**Step 5: 替换导入路径 - artemis-server**

```bash
# artemis-server 中使用 artemis-management::model
find artemis-server/src -name "*.rs" -type f -exec sed -i 's/use artemis_core::model::\(ServiceGroup\|RouteRule\|RouteStrategy\|RouteRuleGroup\)/use artemis_management::model::\1/g' {} +
```

**Step 6: 替换导入路径 - artemis-web**

```bash
# artemis-web 中使用 artemis-management::model
find artemis-web/src -name "*.rs" -type f -exec sed -i 's/use artemis_core::model::\(ServiceGroup\|GroupTag\|GroupType\|GroupStatus\|GroupOperation\|GroupInstance\|RouteRule\|RouteRuleGroup\|RouteStrategy\|RouteRuleStatus\|ZoneOperation\|CanaryConfig\|InstanceOperation\|ServerOperation\|BindingType\)/use artemis_management::model::\1/g' {} +
```

**Step 7: 手动检查 model 内部的依赖**

检查 `artemis-management/src/model/*.rs` 文件中的导入，确保：
- 使用 `use super::*;` 或 `use crate::model::*;` 来引用同模块的类型
- 使用 `use artemis_core::model::{Instance, InstanceKey};` 来引用核心类型

**Step 8: 验证编译**

```bash
cargo build -p artemis-management -p artemis-server -p artemis-web
```

Expected: 可能有编译错误，需要手动调整

**Step 9: 修复编译错误**

根据编译错误逐一修复导入路径

**Step 10: 验证编译成功**

```bash
cargo build -p artemis-management -p artemis-server -p artemis-web
```

Expected: 编译成功

**Step 11: 提交**

```bash
git add -A
git commit -m "refactor: 迁移 management 模型到 artemis-management"
```

---

## Phase 4: 精简 artemis-core

### Task 9: 精简 artemis-core/src/model/request.rs

**Files:**
- Modify: `artemis-core/src/model/request.rs`

**Step 1: 备份当前文件**

```bash
cp artemis-core/src/model/request.rs artemis-core/src/model/request.rs.bak
```

**Step 2: 检查 request.rs 的内容**

```bash
grep "^pub struct\|^pub enum" artemis-core/src/model/request.rs
```

**Step 3: 识别需要保留的类型**

保留的类型（client 需要）：
- `RegisterRequest`, `RegisterResponse`
- `HeartbeatRequest`, `HeartbeatResponse`
- `UnregisterRequest`, `UnregisterResponse`
- `GetServiceRequest`, `GetServiceResponse`
- `GetServicesRequest`, `GetServicesResponse`
- `DiscoveryConfig`
- `ResponseStatus`

**Step 4: 手动编辑 request.rs**

编辑 `artemis-core/src/model/request.rs`，只保留上述类型，删除其他所有类型

**Step 5: 查找被删除类型的使用位置**

```bash
# 查找哪些地方使用了被删除的类型
grep -r "OperateInstanceRequest\|OperateServerRequest\|IsInstanceDownRequest" artemis-web/src artemis-management/src
```

**Step 6: 将被删除的类型移到 artemis-web**

在 `artemis-web/src/api/` 目录中，将这些请求类型定义在各自的 API 模块中

例如，在 `artemis-web/src/api/management.rs` 中定义 `OperateInstanceRequest` 等

**Step 7: 验证编译**

```bash
cargo build -p artemis-core
```

Expected: 编译成功

**Step 8: 验证其他 crate**

```bash
cargo build -p artemis-web -p artemis-management
```

Expected: 可能有编译错误，需要调整导入路径

**Step 9: 修复编译错误**

根据编译错误调整 artemis-web 中的导入路径

**Step 10: 提交**

```bash
git add -A
git commit -m "refactor: 精简 artemis-core request.rs，只保留 client 需要的类型"
```

---

### Task 10: 更新 artemis-core/src/model/mod.rs

**Files:**
- Modify: `artemis-core/src/model/mod.rs`

**Step 1: 编辑 artemis-core/src/model/mod.rs**

Modify: `artemis-core/src/model/mod.rs`

删除已移走的模块导入：
```rust
// 删除这些行
pub mod group;
pub mod lease;
pub mod management;
pub mod replication;
pub mod route;
pub mod zone;
pub mod canary;
pub mod status;
```

只保留：
```rust
pub mod change;
pub mod instance;
pub mod request;
pub mod service;

pub use change::{ChangeType, InstanceChange};
pub use instance::{Instance, InstanceKey, InstanceStatus};
pub use request::*;
pub use service::Service;
```

**Step 2: 验证编译**

```bash
cargo build -p artemis-core
```

Expected: 编译成功

**Step 3: 提交**

```bash
git add artemis-core/src/model/mod.rs
git commit -m "refactor: 更新 artemis-core model/mod.rs，移除已迁移的模块"
```

---

### Task 11: 更新 artemis-core/src/lib.rs

**Files:**
- Modify: `artemis-core/src/lib.rs`

**Step 1: 编辑 artemis-core/src/lib.rs**

Modify: `artemis-core/src/lib.rs`

精简为：
```rust
//! Artemis Core - 核心数据模型和协议定义

pub mod error;
pub mod model;

pub use error::ArtemisError;
pub use model::*;
```

删除已移走的模块：
```rust
// 删除这些行（如果存在）
pub mod config;
pub mod telemetry;
pub mod traits;
pub mod utils;
```

**Step 2: 验证编译**

```bash
cargo build -p artemis-core
```

Expected: 编译成功

**Step 3: 提交**

```bash
git add artemis-core/src/lib.rs
git commit -m "refactor: 精简 artemis-core lib.rs，只导出核心协议"
```

---

## Phase 5: 全面验证

### Task 12: 验证所有 crate 编译

**Step 1: 清理构建缓存**

```bash
cargo clean
```

**Step 2: 按依赖顺序编译**

```bash
cargo build -p artemis-core
cargo build -p artemis-server
cargo build -p artemis-management
cargo build -p artemis-client
cargo build -p artemis-web
cargo build -p artemis
```

Expected: 每个 crate 都编译成功

**Step 3: 全部编译**

```bash
cargo build --workspace
```

Expected: 编译成功，零警告

**Step 4: 运行 Clippy**

```bash
cargo clippy --workspace -- -D warnings
```

Expected: 零警告

**Step 5: 格式检查**

```bash
cargo fmt --all -- --check
```

Expected: 格式正确

**Step 6: 提交（如果有格式修改）**

```bash
cargo fmt --all
git add -A
git commit -m "chore: 代码格式化"
```

---

### Task 13: 运行测试

**Step 1: 运行所有单元测试**

```bash
cargo test --workspace --lib
```

Expected: 所有单元测试通过

**Step 2: 运行集成测试**

```bash
cargo test --workspace --test "*"
```

Expected: 所有集成测试通过

**Step 3: 运行 integration_test**

```bash
cargo test --test integration_test
```

Expected: 通过

**Step 4: 提交（如果有测试修复）**

```bash
git add -A
git commit -m "fix: 修复重构后的测试问题"
```

---

### Task 14: 功能验证

**Step 1: 启动开发环境**

```bash
./scripts/dev.sh start
```

Expected: 后端和前端都成功启动

**Step 2: 查看日志**

```bash
./scripts/dev.sh logs backend | head -50
```

Expected: 无错误日志

**Step 3: 运行集成测试脚本**

```bash
./scripts/test-instance-management.sh
```

Expected: 所有测试通过

**Step 4: 停止开发环境**

```bash
./scripts/dev.sh stop
```

---

### Task 15: 统计重构结果

**Step 1: 统计 artemis-core 代码行数**

```bash
find artemis-core/src -name "*.rs" -exec wc -l {} + | tail -1
```

Expected: 约 500-600 行（减少约 75%）

**Step 2: 列出 artemis-core 保留的文件**

```bash
find artemis-core/src -name "*.rs" -type f
```

Expected:
```
artemis-core/src/lib.rs
artemis-core/src/error.rs
artemis-core/src/model/mod.rs
artemis-core/src/model/instance.rs
artemis-core/src/model/service.rs
artemis-core/src/model/change.rs
artemis-core/src/model/request.rs
```

**Step 3: 验证依赖关系**

```bash
cargo tree -p artemis-client -i artemis-core
cargo tree -p artemis-server -i artemis-core
cargo tree -p artemis-management -i artemis-core
```

Expected: 所有 crate 都正确依赖 artemis-core

---

### Task 16: 最终提交和合并

**Step 1: 查看所有变更**

```bash
git log --oneline --graph
```

**Step 2: 创建最终总结提交**

```bash
git add -A
git commit -m "refactor: 完成 artemis-core 重构

- artemis-core 精简为核心协议层（~500 行）
- server 特有功能迁移到 artemis-server
- management 模型迁移到 artemis-management
- 所有测试通过，零编译警告

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

**Step 3: 合并到 main（可选）**

```bash
git checkout main
git merge refactor/artemis-core
```

**Step 4: 推送到远程（可选）**

```bash
git push origin main
```

---

## 回滚策略

如果在任何步骤遇到无法解决的问题：

```bash
# 回滚到重构前的状态
git checkout main
git branch -D refactor/artemis-core
```

---

## 预期成果

- ✅ artemis-core 从 ~2000 行精简到 ~500 行（减少 75%）
- ✅ artemis-client 依赖更轻量，只依赖核心协议
- ✅ artemis-server 包含所有 server 特有基础设施
- ✅ artemis-management 包含所有管理功能模型
- ✅ 所有测试通过，零编译警告
- ✅ 依赖关系清晰，无循环依赖
