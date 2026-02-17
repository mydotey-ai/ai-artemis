# Phase 29: 管理 API 重构 - 分离管理功能到 artemis-management

## 概述

**目标**: 将管理相关的 HTTP API 处理器从 artemis-web 迁移到 artemis-management,实现更清晰的职责分离。

**当前状态**:
- ✅ 业务逻辑已在 artemis-management (AuthManager, GroupManager, RouteManager 等)
- ❌ HTTP API 处理器仍在 artemis-web/src/api/
- ❌ artemis-web 同时承担核心服务和管理功能的 HTTP 适配

**重构目标**:
- artemis-management 提供完整的管理 API 模块 (HTTP handlers + 路由)
- artemis-web 仅作为轻量级路由聚合层,合并核心服务和管理服务的端点
- 更清晰的模块边界和职责分离

---

## 架构对比

### 当前架构 (Before)

```
artemis-web (HTTP 层)
├── api/
│   ├── auth.rs          ❌ 认证 API (应属于 management)
│   ├── management.rs    ❌ 实例操作 API (应属于 management)
│   ├── routing.rs       ❌ 分组/路由 API (应属于 management)
│   ├── audit.rs         ❌ 审计日志 API (应属于 management)
│   ├── zone.rs          ❌ Zone API (应属于 management)
│   ├── canary.rs        ❌ 金丝雀 API (应属于 management)
│   ├── registry.rs      ✅ 服务注册 API (核心功能)
│   ├── discovery.rs     ✅ 服务发现 API (核心功能)
│   ├── replication.rs   ✅ 数据复制 API (核心功能)
│   ├── status.rs        ✅ 集群状态 API (核心功能)
│   └── metrics.rs       ✅ 监控指标 API (核心功能)
├── middleware/
│   └── jwt.rs           ❌ JWT 中间件 (应属于 management)
├── server.rs            ⚠️ 路由定义 (需简化)
└── state.rs             ⚠️ 全局状态 (需简化)

artemis-management (业务逻辑层)
├── auth/                ✅ 认证/授权业务逻辑
├── instance.rs          ✅ 实例操作业务逻辑
├── group.rs             ✅ 分组管理业务逻辑
├── route.rs             ✅ 路由规则业务逻辑
├── zone.rs              ✅ Zone 管理业务逻辑
├── canary.rs            ✅ 金丝雀业务逻辑
└── audit.rs             ✅ 审计日志业务逻辑
```

### 目标架构 (After)

```
artemis-web (轻量级路由聚合层)
├── api/
│   ├── registry.rs      ✅ 服务注册 API
│   ├── discovery.rs     ✅ 服务发现 API
│   ├── replication.rs   ✅ 数据复制 API
│   ├── status.rs        ✅ 集群状态 API
│   └── metrics.rs       ✅ 监控指标 API
├── server.rs            🔧 简化的路由聚合
└── state.rs             🔧 仅核心服务状态

artemis-management (完整管理层 - 业务逻辑 + HTTP API)
├── web/                 ✨ 新增: HTTP API 模块
│   ├── api/
│   │   ├── auth.rs      🔄 认证 API (迁移自 artemis-web)
│   │   ├── instance.rs  🔄 实例操作 API (迁移自 management.rs)
│   │   ├── routing.rs   🔄 分组/路由 API (迁移自 artemis-web)
│   │   ├── audit.rs     🔄 审计日志 API (迁移自 artemis-web)
│   │   ├── zone.rs      🔄 Zone API (迁移自 artemis-web)
│   │   ├── canary.rs    🔄 金丝雀 API (迁移自 artemis-web)
│   │   └── mod.rs
│   ├── middleware/
│   │   ├── jwt.rs       🔄 JWT 中间件 (迁移自 artemis-web)
│   │   └── mod.rs
│   ├── routes.rs        ✨ 管理路由定义
│   ├── state.rs         ✨ 管理状态
│   └── mod.rs
├── auth/                ✅ 认证/授权业务逻辑
├── instance.rs          ✅ 实例操作业务逻辑
├── group.rs             ✅ 分组管理业务逻辑
├── route.rs             ✅ 路由规则业务逻辑
├── zone.rs              ✅ Zone 管理业务逻辑
├── canary.rs            ✅ 金丝雀业务逻辑
└── audit.rs             ✅ 审计日志业务逻辑
```

---

## 重构任务清单

### 1. 创建 artemis-management 的 Web API 模块

#### 1.1 创建目录结构

```bash
mkdir -p artemis-management/src/web/api
mkdir -p artemis-management/src/web/middleware
```

**新增文件**:
- `artemis-management/src/web/mod.rs` - Web 模块入口
- `artemis-management/src/web/api/mod.rs` - API 模块入口
- `artemis-management/src/web/middleware/mod.rs` - 中间件模块入口
- `artemis-management/src/web/routes.rs` - 路由定义
- `artemis-management/src/web/state.rs` - 管理状态

#### 1.2 更新 Cargo.toml 依赖

在 `artemis-management/Cargo.toml` 中添加 Web 框架依赖:

```toml
[dependencies]
# 现有依赖...
artemis-core = { path = "../artemis-core" }
sea-orm = { workspace = true }
tokio = { workspace = true }
dashmap = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tracing = { workspace = true }
chrono = { workspace = true }
argon2 = { workspace = true }
jsonwebtoken = { workspace = true }
thiserror = { workspace = true }

# 新增: Web 框架依赖
axum = { workspace = true }           # HTTP 框架
tower = { workspace = true }          # 中间件层
tower-http = { workspace = true }     # HTTP 中间件
```

---

### 2. 迁移 HTTP 处理器

#### 2.1 迁移认证 API

**源文件**: `artemis-web/src/api/auth.rs` (376 行)
**目标文件**: `artemis-management/src/web/api/auth.rs`

**迁移内容**:
- ✅ 所有请求/响应数据模型 (LoginRequest, LoginResponse 等)
- ✅ 所有 API 处理器函数 (login, logout, refresh_token, get_current_user 等)
- ✅ 辅助函数 (extract_user_id 等)

**API 端点** (共 14 个):
```
POST   /api/auth/login
POST   /api/auth/logout
POST   /api/auth/refresh
GET    /api/auth/user
GET    /api/auth/permissions
POST   /api/auth/password/change
POST   /api/auth/password/reset/{user_id}
GET    /api/auth/sessions
DELETE /api/auth/sessions/{session_id}
POST   /api/auth/check-permission
GET    /api/auth/users
POST   /api/auth/users
GET    /api/auth/users/{user_id}
PUT    /api/auth/users/{user_id}
DELETE /api/auth/users/{user_id}
PATCH  /api/auth/users/{user_id}/status
GET    /api/auth/users/{user_id}/login-history
GET    /api/auth/roles
```

#### 2.2 迁移实例操作 API

**源文件**: `artemis-web/src/api/management.rs` (265 行)
**目标文件**: `artemis-management/src/web/api/instance.rs`

**API 端点** (共 8 个):
```
POST /api/management/instance/operate-instance.json
POST /api/management/instance/get-instance-operations.json
POST /api/management/instance/is-instance-down.json
POST /api/management/server/operate-server.json
POST /api/management/server/is-server-down.json
POST /api/management/all-instance-operations.json
GET  /api/management/all-instance-operations.json
POST /api/management/all-server-operations.json
GET  /api/management/all-server-operations.json
```

#### 2.3 迁移分组和路由 API

**源文件**: `artemis-web/src/api/routing.rs` (1,063 行 - 最大文件)
**目标文件**: `artemis-management/src/web/api/routing.rs`

**API 端点** (共 23 个):
```
# 分组管理 (9 个)
POST   /api/routing/groups
GET    /api/routing/groups
GET    /api/routing/groups/{group_id}
DELETE /api/routing/groups/{group_key}
PATCH  /api/routing/groups/{group_key}
POST   /api/routing/groups/{group_key}/tags
GET    /api/routing/groups/{group_key}/tags
DELETE /api/routing/groups/{group_key}/tags/{tag_key}
GET    /api/routing/groups/{group_key}/instances
POST   /api/routing/groups/{group_key}/instances
DELETE /api/routing/groups/{group_key}/instances/{instance_id}
POST   /api/routing/services/{service_id}/instances

# 路由规则管理 (11 个)
POST   /api/routing/rules
GET    /api/routing/rules
GET    /api/routing/rules/{rule_id}
DELETE /api/routing/rules/{rule_id}
PATCH  /api/routing/rules/{rule_id}
POST   /api/routing/rules/{rule_id}/publish
POST   /api/routing/rules/{rule_id}/unpublish
POST   /api/routing/rules/{rule_id}/groups
GET    /api/routing/rules/{rule_id}/groups
DELETE /api/routing/rules/{rule_id}/groups/{group_id}
PATCH  /api/routing/rules/{rule_id}/groups/{group_id}
```

#### 2.4 迁移审计日志 API

**源文件**: `artemis-web/src/api/audit.rs` (443 行)
**目标文件**: `artemis-management/src/web/api/audit.rs`

**API 端点** (共 9 个):
```
GET  /api/management/audit/logs
GET  /api/management/audit/instance-logs
GET  /api/management/audit/server-logs
POST /api/management/log/group-logs.json
POST /api/management/log/route-rule-logs.json
POST /api/management/log/route-rule-group-logs.json
POST /api/management/log/zone-operation-logs.json
POST /api/management/log/group-instance-logs.json
POST /api/management/log/service-instance-logs.json
```

#### 2.5 迁移 Zone 管理 API

**源文件**: `artemis-web/src/api/zone.rs` (188 行)
**目标文件**: `artemis-management/src/web/api/zone.rs`

**API 端点** (共 5 个):
```
POST   /api/management/zone/pull-out
POST   /api/management/zone/pull-in
GET    /api/management/zone/status/{zone_id}/{region_id}
GET    /api/management/zone/operations
DELETE /api/management/zone/{zone_id}/{region_id}
```

#### 2.6 迁移金丝雀 API

**源文件**: `artemis-web/src/api/canary.rs` (219 行)
**目标文件**: `artemis-management/src/web/api/canary.rs`

**API 端点** (共 5 个):
```
POST   /api/management/canary/config
GET    /api/management/canary/config/{service_id}
POST   /api/management/canary/enable
DELETE /api/management/canary/config/{service_id}
GET    /api/management/canary/configs
```

---

### 3. 迁移中间件

#### 3.1 迁移 JWT 中间件

**源文件**: `artemis-web/src/middleware/jwt.rs` (43 行)
**目标文件**: `artemis-management/src/web/middleware/jwt.rs`

**功能**:
- 从 `Authorization` header 提取 JWT token
- 验证 token 有效性
- 将 `user_id` 注入请求扩展

---

### 4. 创建管理层路由

#### 4.1 管理状态 (ManagementState)

**文件**: `artemis-management/src/web/state.rs`

```rust
use std::sync::Arc;
use crate::{
    AuthManager, InstanceManager, GroupManager, RouteManager,
    ZoneManager, CanaryManager, AuditManager
};

#[derive(Clone)]
pub struct ManagementState {
    pub auth_manager: Arc<AuthManager>,
    pub instance_manager: Arc<InstanceManager>,
    pub group_manager: Arc<GroupManager>,
    pub route_manager: Arc<RouteManager>,
    pub zone_manager: Arc<ZoneManager>,
    pub canary_manager: Arc<CanaryManager>,
    pub audit_manager: Arc<AuditManager>,
}
```

#### 4.2 管理路由定义

**文件**: `artemis-management/src/web/routes.rs`

```rust
use axum::{Router, middleware};
use tower_http::cors::CorsLayer;

pub fn management_routes(state: ManagementState) -> Router {
    // 公开路由 (无需认证)
    let public_routes = Router::new()
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/roles", get(auth::get_roles))
        .with_state(state.clone());

    // 受保护路由 (需要 JWT 认证)
    let protected_routes = Router::new()
        // 认证相关
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/refresh", post(auth::refresh_token))
        .route("/api/auth/user", get(auth::get_current_user))
        // ... 其他受保护路由 ...
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            jwt::jwt_auth,
        ))
        .with_state(state.clone());

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(CorsLayer::permissive())
}
```

---

### 5. 更新 artemis-web 路由聚合

#### 5.1 简化 AppState

**文件**: `artemis-web/src/state.rs`

```rust
use artemis_management::ManagementState;
use artemis_server::{RegistryServiceImpl, DiscoveryServiceImpl, ...};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    // 核心服务
    pub registry_service: Arc<RegistryServiceImpl>,
    pub discovery_service: Arc<DiscoveryServiceImpl>,
    pub cache: Arc<VersionedCacheManager>,
    pub session_manager: Arc<SessionManager>,
    pub cluster_manager: Option<Arc<ClusterManager>>,
    pub replication_manager: Option<Arc<ReplicationManager>>,
    pub load_balancer: Arc<LoadBalancer>,
    pub status_service: Arc<StatusService>,

    // 管理功能 (委托给 management 层)
    // 移除: instance_manager, group_manager, route_manager, zone_manager,
    //       canary_manager, audit_manager, auth_manager
}
```

#### 5.2 简化路由定义

**文件**: `artemis-web/src/server.rs`

```rust
use artemis_management::web::management_routes;

pub async fn run_server(config: ServerConfig) -> Result<(), ArtemisError> {
    // 初始化核心服务状态
    let core_state = AppState { ... };

    // 初始化管理状态
    let management_state = ManagementState { ... };

    // 核心服务路由
    let core_routes = Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(metrics))
        .route("/api/registry/register", post(registry::register))
        .route("/api/discovery/service", post(discovery::get_service))
        .route("/api/status/cluster", get(status::cluster_status))
        .route("/api/replication/replicate", post(replication::replicate))
        .with_state(core_state);

    // 管理服务路由 (来自 artemis-management)
    let mgmt_routes = management_routes(management_state);

    // 合并所有路由
    let app = Router::new()
        .merge(core_routes)
        .merge(mgmt_routes);

    // 启动服务器
    let listener = TcpListener::bind(&config.addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

---

### 6. 导出管理 Web API

#### 6.1 更新 artemis-management/src/lib.rs

```rust
pub mod api;
pub mod audit;
pub mod auth;
pub mod canary;
pub mod dao;
pub mod db;
pub mod group;
pub mod instance;
pub mod loader;
pub mod model;
pub mod route;
pub mod zone;
pub mod web;  // ✨ 新增: Web API 模块

pub use audit::AuditManager;
pub use auth::AuthManager;
pub use canary::CanaryManager;
pub use db::Database;
pub use group::GroupManager;
pub use instance::InstanceManager;
pub use loader::ConfigLoader;
pub use route::RouteManager;
pub use zone::ZoneManager;
pub use web::{ManagementState, management_routes};  // ✨ 导出 Web API
```

---

### 7. 清理 artemis-web

#### 7.1 删除已迁移的文件

```bash
rm artemis-web/src/api/auth.rs
rm artemis-web/src/api/management.rs
rm artemis-web/src/api/routing.rs
rm artemis-web/src/api/audit.rs
rm artemis-web/src/api/zone.rs
rm artemis-web/src/api/canary.rs
rm artemis-web/src/middleware/jwt.rs
```

#### 7.2 更新 artemis-web/src/api/mod.rs

移除已迁移模块的引用:

```rust
// 移除:
// pub mod auth;
// pub mod management;
// pub mod routing;
// pub mod audit;
// pub mod zone;
// pub mod canary;

// 保留核心服务:
pub mod discovery;
pub mod metrics;
pub mod registry;
pub mod replication;
pub mod status;
```

#### 7.3 更新 artemis-web/src/middleware/mod.rs

```rust
// 移除:
// pub mod jwt;
```

---

## 实施步骤

### Step 1: 创建管理 Web 模块结构 (估计: 30 分钟)

1. 创建目录和基础文件
2. 更新 artemis-management/Cargo.toml 依赖
3. 创建 ManagementState 和基础路由框架

### Step 2: 迁移中间件 (估计: 15 分钟)

1. 迁移 JWT 中间件到 artemis-management/src/web/middleware/jwt.rs
2. 测试中间件功能

### Step 3: 迁移认证 API (估计: 30 分钟)

1. 迁移 auth.rs 到 artemis-management/src/web/api/auth.rs
2. 更新导入路径
3. 测试所有认证 API 端点

### Step 4: 迁移实例操作 API (估计: 20 分钟)

1. 迁移 management.rs 到 artemis-management/src/web/api/instance.rs
2. 更新导入路径
3. 测试实例操作 API 端点

### Step 5: 迁移分组和路由 API (估计: 45 分钟)

1. 迁移 routing.rs 到 artemis-management/src/web/api/routing.rs
2. 处理复杂的路由规则逻辑
3. 测试所有分组和路由 API 端点

### Step 6: 迁移审计日志 API (估计: 25 分钟)

1. 迁移 audit.rs 到 artemis-management/src/web/api/audit.rs
2. 更新导入路径
3. 测试审计日志查询 API

### Step 7: 迁移 Zone 和金丝雀 API (估计: 25 分钟)

1. 迁移 zone.rs 和 canary.rs
2. 更新导入路径
3. 测试 Zone 和金丝雀 API 端点

### Step 8: 完善管理路由定义 (估计: 30 分钟)

1. 在 routes.rs 中定义所有管理路由
2. 配置认证和 CORS 中间件
3. 导出 management_routes 函数

### Step 9: 更新 artemis-web 路由聚合 (估计: 30 分钟)

1. 简化 AppState
2. 更新 server.rs 使用 management_routes
3. 清理已迁移的文件和导入

### Step 10: 集成测试 (估计: 60 分钟)

1. 运行所有单元测试
2. 运行集成测试脚本
3. 测试 Web Console 集成
4. 验证所有 API 端点可用

### Step 11: 文档更新 (估计: 30 分钟)

1. 更新架构设计文档
2. 更新 API 文档
3. 更新 README

---

## 测试计划

### 单元测试

- [ ] artemis-management/src/web/api/ 下的所有 API 处理器
- [ ] JWT 中间件验证逻辑
- [ ] ManagementState 初始化

### 集成测试

- [ ] 运行现有集成测试脚本:
  - `./scripts/test-auth-api.sh`
  - `./scripts/test-instance-management.sh`
  - `./scripts/test-routing-api.sh`
  - `./scripts/test-audit-api.sh`
  - `./scripts/test-zone-api.sh`
  - `./scripts/test-canary-api.sh`

- [ ] Web Console 集成测试:
  - 登录/登出功能
  - 服务和实例管理
  - 分组和路由配置
  - 审计日志查看

### 性能测试

- [ ] 基准测试 (与重构前对比):
  - P99 延迟保持 < 0.5ms
  - 吞吐量保持 10,000+ QPS
  - 内存占用无显著增加

---

## 预期收益

### 1. 更清晰的职责分离

- ✅ artemis-management 成为完整的管理层 (业务逻辑 + HTTP API)
- ✅ artemis-web 简化为轻量级路由聚合层
- ✅ 核心服务和管理功能完全解耦

### 2. 更好的可维护性

- ✅ 管理功能集中在一个 crate,易于理解和维护
- ✅ 减少跨 crate 的依赖和耦合
- ✅ 更容易独立测试和部署管理功能

### 3. 更灵活的部署选项

- ✅ 可以单独部署管理服务 (例如,独立的管理控制台)
- ✅ 可以独立扩展管理服务和核心服务
- ✅ 更容易实现多租户和权限隔离

### 4. 更好的代码组织

- ✅ artemis-web 代码量减少约 2,500 行 (6 个 API 文件 + 中间件)
- ✅ artemis-management 成为功能完整的管理层
- ✅ 模块边界更加清晰

---

## 风险和注意事项

### 1. 依赖管理

**风险**: artemis-management 引入 Axum 等 Web 框架依赖,增加编译时间
**缓解**: 使用 workspace 共享依赖,影响有限

### 2. 兼容性

**风险**: API 端点路径或响应格式变更
**缓解**: 保持所有 API 端点和响应格式不变,仅移动代码位置

### 3. 测试覆盖

**风险**: 迁移过程中可能引入 bug
**缓解**:
- 保留所有现有测试
- 运行完整的集成测试套件
- Web Console 端到端测试

### 4. 文档更新

**风险**: 文档与代码不同步
**缓解**: 同步更新架构文档、API 文档和 README

---

## 成功标准

1. ✅ **零功能回退**: 所有现有功能正常工作
2. ✅ **测试通过**: 所有单元测试和集成测试通过
3. ✅ **性能保持**: 性能指标与重构前一致
4. ✅ **零编译警告**: `cargo clippy` 零警告
5. ✅ **文档同步**: 所有文档更新完毕

---

## 相关文档

- **架构设计**: [`docs/plans/design.md`](../design.md)
- **实施路线图**: [`docs/plans/implementation-roadmap.md`](../implementation-roadmap.md)
- **开发规范**: [`.claude/rules/dev-standards.md`](../../../.claude/rules/dev-standards.md)
- **Web Console 文档**: [`docs/web-console/README.md`](../../web-console/README.md)

---

## 时间估算

| 步骤 | 预估时间 |
|------|---------|
| 创建模块结构 | 30 分钟 |
| 迁移中间件 | 15 分钟 |
| 迁移认证 API | 30 分钟 |
| 迁移实例操作 API | 20 分钟 |
| 迁移分组和路由 API | 45 分钟 |
| 迁移审计日志 API | 25 分钟 |
| 迁移 Zone 和金丝雀 API | 25 分钟 |
| 完善管理路由 | 30 分钟 |
| 更新 artemis-web | 30 分钟 |
| 集成测试 | 60 分钟 |
| 文档更新 | 30 分钟 |
| **总计** | **约 5.5 小时** |

---

**Phase 29 状态**: 📝 规划阶段

**最后更新**: 2026-02-17
