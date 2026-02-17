# Artemis Core 重构设计

**日期**: 2026-02-17
**状态**: ✅ 设计已确认
**目标**: 精简 artemis-core，让 client 只依赖必需的核心协议部分

---

## 1. 重构目标

### 当前问题

artemis-core 包含过多内容，导致：
- **artemis-client** 依赖了许多不需要的 server/management 特有类型
- **模块职责不清晰**：config、telemetry、traits 等 server 特有功能混在 core 中
- **依赖关系混乱**：core 变成了一个"大杂烩"

### 期望结果

- **artemis-core**：只包含 client/server 共享的核心协议（Instance、Service、基础请求类型）
- **artemis-server**：包含所有 server 特有的基础设施（config、telemetry、traits、租约、复制）
- **artemis-management**：包含所有管理功能的模型（分组、路由、Zone、金丝雀）
- **artemis-client**：依赖更轻量，编译更快

---

## 2. 架构设计

### 2.1 重构后的模块划分

#### artemis-core（精简后）

**保留内容**（client/server 共享的核心协议）：

```
artemis-core/
├── src/
│   ├── lib.rs
│   ├── error.rs                # ArtemisError（共享错误类型）
│   └── model/
│       ├── mod.rs
│       ├── instance.rs         # Instance, InstanceKey, InstanceStatus
│       ├── service.rs          # Service
│       ├── change.rs           # InstanceChange（WebSocket 推送）
│       └── request.rs          # 精简版：只保留 client 需要的请求类型
└── Cargo.toml
```

**request.rs 保留的类型**：
- `RegisterRequest`, `RegisterResponse`
- `HeartbeatRequest`, `HeartbeatResponse`
- `UnregisterRequest`, `UnregisterResponse`
- `GetServiceRequest`, `GetServiceResponse`
- `GetServicesRequest`, `GetServicesResponse`
- `DiscoveryConfig`
- `ResponseStatus`

#### artemis-server（新增内容）

**从 artemis-core 迁移**：

```
artemis-server/
├── src/
│   ├── config/                 # 从 artemis-core 迁移
│   ├── telemetry/              # 从 artemis-core 迁移
│   ├── utils.rs                # 从 artemis-core 迁移
│   ├── traits/                 # 从 artemis-core 迁移
│   │   ├── registry.rs         # RegistryService trait
│   │   └── discovery.rs        # DiscoveryService trait
│   └── model/                  # 新增：从 artemis-core 迁移
│       ├── lease.rs            # Lease（租约管理）
│       └── replication.rs      # 集群复制相关
│
│   # 现有模块保持不变
│   ├── cache/
│   ├── discovery/
│   ├── lease/
│   ├── registry/
│   ├── routing/
│   └── websocket/
└── Cargo.toml
```

#### artemis-management（新增内容）

**从 artemis-core 迁移**：

```
artemis-management/
├── src/
│   └── model/                  # 新增：从 artemis-core 迁移
│       ├── management.rs       # InstanceOperation, ServerOperation
│       ├── group.rs            # ServiceGroup, GroupInstance, GroupOperation
│       ├── route.rs            # RouteRule, RouteRuleGroup, RouteStrategy
│       ├── zone.rs             # ZoneOperation, ZoneOperationRecord
│       ├── canary.rs           # CanaryConfig
│       └── status.rs           # 管理相关的状态类型
│
│   # 现有模块保持不变
│   ├── audit.rs
│   ├── canary.rs
│   ├── dao/
│   ├── group.rs
│   ├── instance.rs
│   ├── route.rs
│   └── zone.rs
└── Cargo.toml
```

### 2.2 依赖关系

```
artemis-client ──> artemis-core

artemis-server ──> artemis-core

artemis-management ──> artemis-core

artemis-web ──> artemis-core
            ──> artemis-server
            ──> artemis-management

artemis ──> artemis-web
```

**设计原则**：
- **artemis-core** 是最底层，不依赖任何其他 artemis crate
- **artemis-management** 不依赖 artemis-server，保持依赖关系简单
- **artemis-web** 整合所有功能，依赖 core/server/management

---

## 3. 迁移计划

### 3.1 文件移动

#### 从 artemis-core 移到 artemis-server

```bash
# 移动整个目录
artemis-core/src/config/        -> artemis-server/src/config/
artemis-core/src/telemetry/     -> artemis-server/src/telemetry/
artemis-core/src/traits/        -> artemis-server/src/traits/

# 移动单个文件
artemis-core/src/utils.rs       -> artemis-server/src/utils.rs

# 移动模型文件
artemis-core/src/model/lease.rs       -> artemis-server/src/model/lease.rs
artemis-core/src/model/replication.rs -> artemis-server/src/model/replication.rs
```

#### 从 artemis-core 移到 artemis-management

```bash
artemis-core/src/model/management.rs -> artemis-management/src/model/management.rs
artemis-core/src/model/group.rs      -> artemis-management/src/model/group.rs
artemis-core/src/model/route.rs      -> artemis-management/src/model/route.rs
artemis-core/src/model/zone.rs       -> artemis-management/src/model/zone.rs
artemis-core/src/model/canary.rs     -> artemis-management/src/model/canary.rs
artemis-core/src/model/status.rs     -> artemis-management/src/model/status.rs
```

#### artemis-core 保留

```bash
artemis-core/src/lib.rs
artemis-core/src/error.rs
artemis-core/src/model/
  ├── mod.rs          # 需要精简
  ├── instance.rs
  ├── service.rs
  ├── change.rs
  └── request.rs      # 需要精简
```

### 3.2 精简 artemis-core/src/model/request.rs

**移除的类型**（移到 artemis-web 或其他 server 端 crate）：
- 所有 management 相关的请求（OperateInstanceRequest、OperateServerRequest 等）
- 所有 replication 相关的请求
- 其他 server 端特有的请求类型

### 3.3 更新模块导出

#### artemis-core/src/lib.rs

```rust
pub mod error;
pub mod model;

pub use error::ArtemisError;
pub use model::*;
```

#### artemis-server/src/lib.rs

```rust
pub mod config;       // 新增
pub mod telemetry;    // 新增
pub mod utils;        // 新增
pub mod traits;       // 新增
pub mod model;        // 新增

// 现有模块保持不变
pub mod cache;
pub mod discovery;
pub mod lease;
pub mod registry;
pub mod routing;
pub mod websocket;
```

#### artemis-management/src/lib.rs

```rust
pub mod model;        // 新增

// 现有模块保持不变
pub mod audit;
pub mod canary;
pub mod dao;
pub mod group;
pub mod instance;
pub mod route;
pub mod zone;
```

---

## 4. 导入路径更新

### 4.1 artemis-client

**无需变化**（因为依赖的类型仍在 artemis-core）：
```rust
use artemis_core::model::*;
use artemis_core::model::{Instance, InstanceStatus, InstanceChange};
use artemis_core::ArtemisError;
```

### 4.2 artemis-server

**之前**：
```rust
use artemis_core::model::{Instance, RouteRule, Lease};
use artemis_core::traits::RegistryService;
use artemis_core::config::ServerConfig;
```

**之后**：
```rust
use artemis_core::model::{Instance, InstanceKey};
use crate::model::{Lease, Replication};  // 从本地 model 导入
use crate::traits::RegistryService;       // 从本地 traits 导入
use crate::config::ServerConfig;          // 从本地 config 导入
```

### 4.3 artemis-management

**之前**：
```rust
use artemis_core::model::{ServiceGroup, GroupTag, RouteRule};
use artemis_core::model::{ZoneOperation, CanaryConfig};
```

**之后**：
```rust
use artemis_core::model::Instance;        // 核心模型仍从 core 导入
use crate::model::{ServiceGroup, GroupTag, RouteRule};
use crate::model::{ZoneOperation, CanaryConfig};
```

### 4.4 artemis-web

**之前**：
```rust
use artemis_core::model::*;
use artemis_core::traits::{RegistryService, DiscoveryService};
```

**之后**：
```rust
use artemis_core::model::{Instance, Service, RegisterRequest};
use artemis_server::traits::{RegistryService, DiscoveryService};
use artemis_management::model::{RouteRule, GroupInstance};
```

---

## 5. 错误处理与依赖

### 5.1 ArtemisError 的处理

**artemis-core/src/error.rs** 保留，因为：
- Client 和 Server 都需要统一的错误类型
- 错误类型是协议的一部分

各 crate 可以扩展自己的错误类型（可选）。

### 5.2 循环依赖检查

**最终依赖图**：
```
artemis-client ──> artemis-core

artemis-server ──> artemis-core

artemis-management ──> artemis-core
                   (不依赖 artemis-server)

artemis-web ──> artemis-core
            ──> artemis-server
            ──> artemis-management
```

**设计保证**：无循环依赖，依赖关系清晰

---

## 6. 测试和验证策略

### 6.1 编译验证

```bash
# 按依赖顺序编译
cargo build -p artemis-core
cargo build -p artemis-server
cargo build -p artemis-management
cargo build -p artemis-client
cargo build -p artemis-web
cargo build -p artemis

# 全部编译
cargo build --workspace
```

### 6.2 测试验证

```bash
# 单元测试
cargo test --workspace

# 集成测试
cargo test --test integration_test

# Clippy 检查（必须零警告）
cargo clippy --workspace -- -D warnings

# 格式检查
cargo fmt --all -- --check
```

### 6.3 功能验证

```bash
# 启动开发环境
./scripts/dev.sh start

# 运行集成测试脚本
./scripts/test-instance-management.sh
./scripts/test-cluster-api.sh
```

### 6.4 回滚策略

如果重构后出现严重问题：
1. **Git 回滚**：`git reset --hard HEAD~1`
2. **分支策略**：在新分支 `refactor/artemis-core` 上进行，验证通过后合并

---

## 7. 潜在风险与缓解措施

### 风险 1：编译错误

**风险**：导入路径遗漏或错误
**缓解**：仔细检查所有 `use` 语句，按模块逐一验证

### 风险 2：测试失败

**风险**：模块移动后测试找不到类型
**缓解**：更新测试代码的导入路径

### 风险 3：运行时错误

**风险**：序列化/反序列化问题
**缓解**：运行集成测试验证 API 兼容性

---

## 8. 预期结果

重构完成后：

| 指标 | 之前 | 之后 | 改进 |
|------|------|------|------|
| **artemis-core 代码量** | ~2000 行 | ~500 行 | **减少 75%** |
| **artemis-client 依赖** | 依赖整个 core | 只依赖核心协议 | **依赖更轻** |
| **编译速度** | 基准 | 提升 | **client 编译更快** |
| **模块职责** | 混乱 | 清晰 | **易维护** |

**代码质量**：
- ✅ 零编译警告
- ✅ 所有测试通过
- ✅ 依赖关系清晰，无循环依赖

---

## 9. 实施策略

**方案**：一次性重构（方案 2）

**理由**：
- 快速完成，没有中间状态
- 最终代码结构清晰
- 项目已完成，可以集中时间处理编译错误

**步骤**：
1. 一次性移动所有文件到目标位置
2. 一次性修改所有 crate 的导入路径
3. 一次性修复所有编译错误
4. 运行完整测试套件验证
5. 提交到 git（或先在分支验证）

---

## 10. 总结

这次重构将 artemis-core 精简为真正的"核心协议层"，让各模块职责更清晰，依赖关系更合理。重构后，artemis-client 将只依赖必需的核心类型，编译速度更快，维护成本更低。
