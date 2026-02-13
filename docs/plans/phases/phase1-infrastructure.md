# 阶段1: 项目基础设施

> **For Claude:** 按Task顺序执行，每个Task包含详细步骤和验证命令

**目标:** 创建Workspace项目结构，初始化所有crate

**预计任务数:** 2个Task

---

## Task 1.1: 创建Workspace项目结构

**Files:**
- Create: `Cargo.toml`
- Create: `rust-toolchain.toml`
- Create: `.rustfmt.toml`
- Update: `.gitignore`

**Step 1: 创建workspace根配置**

```toml
# Cargo.toml
[workspace]
members = [
    "artemis-core",
    "artemis-server",
    "artemis-web",
    "artemis-management",
    "artemis-client",
    "artemis",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.85"
authors = ["Artemis Contributors"]
license = "MIT OR Apache-2.0"

[workspace.dependencies]
# 异步运行时
tokio = { version = "1.41", features = ["full"] }
tokio-util = { version = "0.7", features = ["codec"] }

# Web框架
axum = { version = "0.7", features = ["ws", "macros"] }
tower = { version = "0.5", features = ["full"] }
tower-http = { version = "0.6", features = ["cors", "trace", "compression"] }

# 序列化
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 并发数据结构
dashmap = "6.1"
parking_lot = "0.12"

# HTTP客户端
reqwest = { version = "0.12", features = ["json"] }

# WebSocket
tokio-tungstenite = "0.24"

# 数据库
sqlx = { version = "0.8", features = ["runtime-tokio", "mysql", "chrono", "json"] }

# 时间处理
chrono = { version = "0.4", features = ["serde"] }

# 限流
governor = "0.7"

# 日志和追踪
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# 配置
config = "0.14"
figment = { version = "0.10", features = ["toml", "env"] }

# 错误处理
thiserror = "2.0"
anyhow = "1.0"

# UUID
uuid = { version = "1.11", features = ["v4", "serde"] }

# CLI
clap = { version = "4.5", features = ["derive"] }

# 异步trait
async-trait = "0.1"

# 其他工具
bytes = "1.8"
futures = "0.3"
```

**Step 2: 创建工具链配置**

```toml
# rust-toolchain.toml
[toolchain]
channel = "1.85"
edition = "2024"
components = ["rustfmt", "clippy"]
```

**Step 3: 创建rustfmt配置**

```toml
# .rustfmt.toml
edition = "2024"
max_width = 100
use_small_heuristics = "Max"
```

**Step 4: 更新.gitignore**

添加以下内容到.gitignore:

```
# Rust
/target
Cargo.lock
**/*.rs.bk
*.pdb
.env
artemis.toml
*.log
```

**Step 5: 验证配置文件**

```bash
cat Cargo.toml
cat rust-toolchain.toml
cat .rustfmt.toml
```

Expected: 所有配置文件创建成功

**Step 6: 提交基础配置**

```bash
git add Cargo.toml rust-toolchain.toml .rustfmt.toml .gitignore
git commit -m "chore: setup workspace structure and toolchain

- Add workspace Cargo.toml with 6 crates
- Configure Rust 1.85 toolchain
- Add rustfmt configuration
- Update gitignore for Rust project

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 1.2: 创建所有crate目录

**Files:**
- Create: `artemis-core/Cargo.toml` + `artemis-core/src/lib.rs`
- Create: `artemis-server/Cargo.toml` + `artemis-server/src/lib.rs`
- Create: `artemis-web/Cargo.toml` + `artemis-web/src/lib.rs`
- Create: `artemis-management/Cargo.toml` + `artemis-management/src/lib.rs`
- Create: `artemis-client/Cargo.toml` + `artemis-client/src/lib.rs`
- Create: `artemis/Cargo.toml` + `artemis/src/main.rs`

**Step 1: 创建artemis-core**

```bash
mkdir -p artemis-core/src
```

```toml
# artemis-core/Cargo.toml
[package]
name = "artemis-core"
version.workspace = true
edition.workspace = true

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
chrono = { workspace = true }
thiserror = { workspace = true }
async-trait = { workspace = true }
parking_lot = { workspace = true }
uuid = { workspace = true }
```

```rust
// artemis-core/src/lib.rs
//! Artemis Core - 核心数据模型和trait定义

pub mod model;
pub mod traits;
pub mod error;
pub mod config;
pub mod utils;
```

**Step 2: 创建artemis-server**

```bash
mkdir -p artemis-server/src
```

```toml
# artemis-server/Cargo.toml
[package]
name = "artemis-server"
version.workspace = true
edition.workspace = true

[dependencies]
artemis-core = { path = "../artemis-core" }
tokio = { workspace = true }
dashmap = { workspace = true }
parking_lot = { workspace = true }
async-trait = { workspace = true }
governor = { workspace = true }
tracing = { workspace = true }
chrono = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
reqwest = { workspace = true }
anyhow = { workspace = true }
```

```rust
// artemis-server/src/lib.rs
//! Artemis Server - 业务逻辑实现

pub mod registry;
pub mod discovery;
pub mod lease;
pub mod cache;
pub mod cluster;
pub mod replication;
pub mod ratelimiter;
pub mod storage;
```

**Step 3: 创建artemis-web**

```bash
mkdir -p artemis-web/src
```

```toml
# artemis-web/Cargo.toml
[package]
name = "artemis-web"
version.workspace = true
edition.workspace = true

[dependencies]
artemis-core = { path = "../artemis-core" }
artemis-server = { path = "../artemis-server" }
axum = { workspace = true }
tower = { workspace = true }
tower-http = { workspace = true }
tokio = { workspace = true }
tokio-util = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tracing = { workspace = true }
anyhow = { workspace = true }
dashmap = { workspace = true }
futures = { workspace = true }
chrono = { workspace = true }
```

```rust
// artemis-web/src/lib.rs
//! Artemis Web - HTTP/WebSocket API层

pub mod server;
pub mod state;
pub mod api;
pub mod websocket;
pub mod middleware;
```

**Step 4: 创建artemis-management**

```bash
mkdir -p artemis-management/src
```

```toml
# artemis-management/Cargo.toml
[package]
name = "artemis-management"
version.workspace = true
edition.workspace = true

[dependencies]
artemis-core = { path = "../artemis-core" }
artemis-server = { path = "../artemis-server" }
sqlx = { workspace = true }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
chrono = { workspace = true }
uuid = { workspace = true }
tracing = { workspace = true }
axum = { workspace = true }
anyhow = { workspace = true }
```

```rust
// artemis-management/src/lib.rs
//! Artemis Management - 管理功能和持久化

pub mod instance;
pub mod group;
pub mod route;
pub mod dao;
pub mod api;
```

**Step 5: 创建artemis-client**

```bash
mkdir -p artemis-client/src
```

```toml
# artemis-client/Cargo.toml
[package]
name = "artemis-client"
version.workspace = true
edition.workspace = true
description = "Artemis Service Registry Client SDK"
license.workspace = true

[dependencies]
artemis-core = { path = "../artemis-core" }
reqwest = { workspace = true }
tokio = { workspace = true }
tokio-tungstenite = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
async-trait = { workspace = true }
parking_lot = { workspace = true }
tracing = { workspace = true }
futures = { workspace = true }
```

```rust
// artemis-client/src/lib.rs
//! Artemis Client SDK - 客户端SDK

pub mod config;
pub mod registry;
pub mod discovery;
pub mod websocket;
pub mod error;
```

**Step 6: 创建artemis CLI**

```bash
mkdir -p artemis/src
```

```toml
# artemis/Cargo.toml
[package]
name = "artemis"
version.workspace = true
edition.workspace = true
description = "Artemis Service Registry - CLI and Server"

[[bin]]
name = "artemis"
path = "src/main.rs"

[dependencies]
artemis-core = { path = "../artemis-core" }
artemis-server = { path = "../artemis-server" }
artemis-web = { path = "../artemis-web" }
artemis-management = { path = "../artemis-management" }
clap = { workspace = true }
tokio = { workspace = true }
figment = { workspace = true }
toml = "0.8"
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
reqwest = { workspace = true }
serde_json = { workspace = true }
sqlx = { workspace = true }
anyhow = { workspace = true }
```

```rust
// artemis/src/main.rs
//! Artemis CLI - 可执行程序入口

fn main() {
    println!("Artemis Service Registry");
}
```

**Step 7: 验证workspace编译**

```bash
cargo check --workspace
```

Expected: 成功编译所有crate（可能有unused warnings）

**Step 8: 提交crate结构**

```bash
git add .
git commit -m "chore: create all crate directories and basic structure

- Create artemis-core (models, traits, errors)
- Create artemis-server (registry, discovery, lease)
- Create artemis-web (HTTP/WebSocket API)
- Create artemis-management (persistence, admin)
- Create artemis-client (SDK)
- Create artemis (CLI binary)

All crates compile successfully.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## 阶段1完成标准

- ✅ Workspace配置正确
- ✅ 6个crate全部创建
- ✅ `cargo check --workspace` 通过
- ✅ Git提交完成
